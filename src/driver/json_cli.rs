//! One-shot JSON/JSONL coding-agent CLIs.
//!
//! These agents expose real headless transports, but not a shared long-lived
//! protocol. Keep their process lifecycle here and vary only argv and event
//! extraction. ACP and RPC providers stay on their richer native drivers.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use anyhow::anyhow;
use parking_lot::Mutex;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::driver::{DriverControl, DriverEventSender, DriverStartOptions};
use crate::model::{
    ActivityKind, DriverEvent, InteractionMode, ProviderKind, ProviderResumeCursor, RuntimeMode,
};

struct Config {
    provider: ProviderKind,
    binary: std::path::PathBuf,
    cwd: std::path::PathBuf,
    model: Option<String>,
    cursor: Mutex<Option<String>>,
    child: Mutex<Option<Child>>,
    running: AtomicBool,
    events: DriverEventSender,
}

pub struct JsonCliDriver {
    config: Arc<Config>,
}

impl JsonCliDriver {
    pub fn start(
        provider: ProviderKind,
        options: DriverStartOptions,
        events: DriverEventSender,
    ) -> anyhow::Result<Self> {
        if options.mode != RuntimeMode::FullAccess
            || options.interaction_mode != InteractionMode::Build
        {
            return Err(anyhow!(
                "{}'s headless CLI currently supports Build with Full access only",
                provider.display_name()
            ));
        }
        let cursor = match options.provider_cursor {
            Some(cursor) if cursor.provider() == provider => Some(cursor.native_id().to_owned()),
            Some(cursor) => {
                return Err(anyhow!(
                    "cannot resume {} from a {} cursor",
                    provider.display_name(),
                    cursor.provider().display_name()
                ));
            }
            None => None,
        };
        let _ = events.send(DriverEvent::Connected {
            provider_cursor: cursor
                .clone()
                .map(|session_id| ProviderResumeCursor::External {
                    kind: provider,
                    session_id,
                    session_file: None,
                }),
        });
        Ok(Self {
            config: Arc::new(Config {
                provider,
                binary: options.binary,
                cwd: options.cwd,
                model: options.model,
                cursor: Mutex::new(cursor),
                child: Mutex::new(None),
                running: AtomicBool::new(false),
                events,
            }),
        })
    }
}

impl DriverControl for JsonCliDriver {
    fn prompt(&self, prompt: String) {
        if self.config.running.swap(true, Ordering::AcqRel) {
            let _ = self.config.events.send(DriverEvent::Error(format!(
                "{} is already running a turn",
                self.config.provider.display_name()
            )));
            return;
        }
        let config = self.config.clone();
        let _ = thread::Builder::new()
            .name(format!("pengpilot-{}-turn", config.provider.id()))
            .spawn(move || {
                run_turn(&config, prompt);
                config.running.store(false, Ordering::Release);
            });
    }

    fn cancel(&self) {
        if let Some(child) = self.config.child.lock().as_mut() {
            let _ = child.kill();
        }
    }

    fn respond(&self, _request_id: String, _option_id: String) {}

    fn rollback(&self, _turns: usize) -> anyhow::Result<Option<ProviderResumeCursor>> {
        anyhow::bail!(
            "{} does not expose native rollback through its headless CLI",
            self.config.provider.display_name()
        )
    }
}

impl Drop for JsonCliDriver {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn run_turn(config: &Arc<Config>, prompt: String) {
    let _ = config.events.send(DriverEvent::TurnStarted);
    let cursor = config.cursor.lock().clone();
    let mut command = crate::command_env::command(&config.binary);
    command.current_dir(&config.cwd);
    let antigravity_log = (config.provider == ProviderKind::Antigravity)
        .then(|| std::env::temp_dir().join(format!("pengpilot-agy-{}.log", Uuid::new_v4())));
    let stdin_prompt = configure_command(
        &mut command,
        config.provider,
        &prompt,
        config.model.as_deref(),
        cursor.as_deref(),
        antigravity_log.as_deref(),
    );
    command
        .stdin(if stdin_prompt {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match crate::command_env::spawn(&mut command) {
        Ok(child) => child,
        Err(error) => {
            finish_error(
                config,
                format!(
                    "failed to start {}: {error}",
                    config.provider.display_name()
                ),
            );
            return;
        }
    };
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    if stdin_prompt && let Some(mut stdin) = child.stdin.take() {
        let payload = json!({
            "type": "user",
            "message": {"role": "user", "content": [{"type": "text", "text": prompt}]}
        });
        let _ = writeln!(stdin, "{payload}");
    }
    *config.child.lock() = Some(child);

    let mut state = StreamState::default();
    if let Some(stdout) = stdout {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<Value>(line) {
                Ok(value) => handle_value(config, &value, &mut state),
                Err(_) => {
                    state.saw_text = true;
                    let _ = config
                        .events
                        .send(DriverEvent::TextDelta(format!("{line}\n")));
                }
            }
        }
    }
    let mut stderr_text = String::new();
    if let Some(mut stderr) = stderr {
        let _ = stderr.read_to_string(&mut stderr_text);
    }
    let status = config
        .child
        .lock()
        .take()
        .and_then(|mut child| child.wait().ok());
    let success = status.is_some_and(|status| status.success());
    if let Some(path) = antigravity_log {
        if let Some(session_id) = read_antigravity_session_id(&path) {
            *config.cursor.lock() = Some(session_id.clone());
            let _ = config.events.send(DriverEvent::Connected {
                provider_cursor: Some(ProviderResumeCursor::External {
                    kind: config.provider,
                    session_id,
                    session_file: None,
                }),
            });
        }
        let _ = std::fs::remove_file(path);
    }
    if !success {
        let detail = stderr_text.trim();
        let message = if detail.is_empty() {
            format!(
                "{} process exited before completing",
                config.provider.display_name()
            )
        } else {
            format!("{}: {detail}", config.provider.display_name())
        };
        let _ = config.events.send(DriverEvent::Error(message.clone()));
        let _ = config.events.send(DriverEvent::TurnFinished {
            success: false,
            summary: Some(message),
        });
    } else {
        let _ = config.events.send(DriverEvent::TurnFinished {
            success: true,
            summary: None,
        });
    }
}

fn configure_command(
    command: &mut std::process::Command,
    provider: ProviderKind,
    prompt: &str,
    model: Option<&str>,
    cursor: Option<&str>,
    antigravity_log: Option<&std::path::Path>,
) -> bool {
    match provider {
        ProviderKind::Antigravity => {
            command.args(["-p", prompt, "--dangerously-skip-permissions"]);
            if let Some(path) = antigravity_log {
                command.arg("--log-file").arg(path);
            }
            if let Some(model) = model {
                command.args(["--model", model]);
            }
            if let Some(cursor) = cursor {
                command.args(["--conversation", cursor]);
            }
        }
        ProviderKind::CodeBuddy => {
            command.args([
                "-p",
                "--output-format",
                "stream-json",
                "--input-format",
                "stream-json",
                "--verbose",
                "--permission-mode",
                "bypassPermissions",
                "--disallowedTools",
                "AskUserQuestion",
                "EnterPlanMode",
                "ExitPlanMode",
            ]);
            if let Some(model) = model {
                command.args(["--model", model]);
            }
            if let Some(cursor) = cursor {
                command.args(["--resume", cursor]);
            }
            return true;
        }
        ProviderKind::Copilot => {
            command.args([
                "-p",
                prompt,
                "--output-format",
                "json",
                "--allow-all",
                "--no-ask-user",
            ]);
            if let Some(model) = model {
                command.args(["--model", model]);
            }
            if let Some(cursor) = cursor {
                command.args(["--resume", cursor]);
            }
        }
        ProviderKind::DevEco => {
            command.args(["run", "--format", "json", "--dangerously-skip-permissions"]);
            if let Some(cwd) = command.get_current_dir().map(std::path::Path::to_owned) {
                command.arg("--dir").arg(cwd);
            }
            if let Some(model) = model {
                command.args(["--model", model]);
            }
            if let Some(cursor) = cursor {
                command.args(["--session", cursor]);
            }
            command.arg(prompt);
        }
        ProviderKind::Qwen => {
            command.args(["-p", prompt, "--output-format", "stream-json"]);
            if let Some(model) = model {
                command.args(["--model", model]);
            }
            if let Some(cursor) = cursor {
                command.args(["--resume", cursor]);
            }
            command.arg("--yolo");
        }
        _ => unreachable!("non-JSON CLI routed to JSON driver"),
    }
    false
}

fn read_antigravity_session_id(path: &std::path::Path) -> Option<String> {
    let log = std::fs::read_to_string(path).ok()?;
    log.lines().rev().find_map(|line| {
        let id = line
            .split_once("conversation=")?
            .1
            .split(',')
            .next()?
            .trim();
        (id.len() == 36 && Uuid::parse_str(id).is_ok()).then(|| id.to_owned())
    })
}

#[derive(Default)]
struct StreamState {
    saw_text: bool,
    saw_delta: bool,
}

fn handle_value(config: &Config, value: &Value, state: &mut StreamState) {
    if let Some(session_id) = session_id(value) {
        let changed = config.cursor.lock().as_deref() != Some(session_id);
        if changed {
            *config.cursor.lock() = Some(session_id.to_owned());
            let _ = config.events.send(DriverEvent::Connected {
                provider_cursor: Some(ProviderResumeCursor::External {
                    kind: config.provider,
                    session_id: session_id.to_owned(),
                    session_file: None,
                }),
            });
        }
    }

    let event_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if event_type.contains("error") {
        if let Some(message) = first_string(value, &["message", "text", "error"]) {
            let _ = config.events.send(DriverEvent::Error(message.to_owned()));
        }
    }
    if event_type.contains("reasoning") {
        if let Some(text) = nested_string(value, &["data", "deltaContent"])
            .or_else(|| nested_string(value, &["data", "content"]))
        {
            let _ = config
                .events
                .send(DriverEvent::ReasoningDelta(text.to_owned()));
        }
    }

    if event_type == "assistant.message_delta"
        && let Some(text) = nested_string(value, &["data", "deltaContent"])
    {
        state.saw_delta = true;
        state.saw_text = true;
        let _ = config.events.send(DriverEvent::TextDelta(text.to_owned()));
        return;
    }

    if matches!(event_type, "tool_use" | "tool.execution_start") {
        let tool = first_string(value, &["tool", "name"])
            .or_else(|| nested_string(value, &["part", "tool"]))
            .unwrap_or("tool");
        let id = first_string(value, &["callId", "id"])
            .or_else(|| nested_string(value, &["part", "callID"]))
            .map(str::to_owned);
        let _ = config.events.send(DriverEvent::Activity {
            id,
            kind: ActivityKind::from_tool_name(tool),
            title: tool.to_owned(),
            detail: value
                .get("input")
                .or_else(|| value.pointer("/part/state/input"))
                .map(Value::to_string),
            complete: false,
        });
    }

    let mut texts = Vec::new();
    collect_content_text(value.get("message"), &mut texts, &config.events);
    if texts.is_empty() && !state.saw_delta {
        if event_type == "text" {
            if let Some(text) =
                first_string(value, &["text"]).or_else(|| nested_string(value, &["part", "text"]))
            {
                texts.push(text.to_owned());
            }
        } else if event_type == "assistant.message" {
            if let Some(text) = nested_string(value, &["data", "content"]) {
                texts.push(text.to_owned());
            }
        } else if event_type == "result" {
            if let Some(text) = first_string(value, &["result"]) {
                texts.push(text.to_owned());
            }
        } else if let Some(payloads) = value.get("payloads").and_then(Value::as_array) {
            texts.extend(
                payloads
                    .iter()
                    .filter_map(|payload| payload.get("text").and_then(Value::as_str))
                    .map(str::to_owned),
            );
        }
    }
    for text in texts {
        if !text.is_empty() {
            state.saw_text = true;
            let _ = config.events.send(DriverEvent::TextDelta(text));
        }
    }
}

fn collect_content_text(
    value: Option<&Value>,
    texts: &mut Vec<String>,
    events: &DriverEventSender,
) {
    let Some(value) = value else { return };
    let content = value.get("content").unwrap_or(value);
    let Some(blocks) = content.as_array() else {
        return;
    };
    for block in blocks {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    texts.push(text.to_owned());
                }
            }
            Some("thinking") => {
                if let Some(text) = block.get("thinking").and_then(Value::as_str) {
                    let _ = events.send(DriverEvent::ReasoningDelta(text.to_owned()));
                }
            }
            _ => {}
        }
    }
}

fn session_id(value: &Value) -> Option<&str> {
    first_string(value, &["sessionId", "session_id"])
        .or_else(|| nested_string(value, &["data", "sessionId"]))
        .or_else(|| nested_string(value, &["part", "sessionID"]))
}

fn first_string<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
}

fn nested_string<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    path.iter()
        .try_fold(value, |value, key| value.get(*key))?
        .as_str()
}

fn finish_error(config: &Config, message: String) {
    let _ = config.events.send(DriverEvent::Error(message.clone()));
    let _ = config.events.send(DriverEvent::TurnFinished {
        success: false,
        summary: Some(message),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::test_event_channel;

    #[test]
    fn copilot_delta_and_session_are_preserved() {
        let (events, received) = test_event_channel();
        let config = Config {
            provider: ProviderKind::Copilot,
            binary: "copilot".into(),
            cwd: ".".into(),
            model: None,
            cursor: Mutex::new(None),
            child: Mutex::new(None),
            running: AtomicBool::new(false),
            events,
        };
        let mut state = StreamState::default();
        handle_value(
            &config,
            &json!({"type":"session.start","data":{"sessionId":"s1"}}),
            &mut state,
        );
        handle_value(
            &config,
            &json!({"type":"assistant.message_delta","data":{"deltaContent":"hi"}}),
            &mut state,
        );
        assert!(matches!(
            received.recv().unwrap(),
            DriverEvent::Connected { .. }
        ));
        assert!(matches!(received.recv().unwrap(), DriverEvent::TextDelta(text) if text == "hi"));
    }

    #[test]
    fn qwen_content_blocks_keep_reasoning_and_text_order() {
        let (events, received) = test_event_channel();
        let config = Config {
            provider: ProviderKind::Qwen,
            binary: "qwen".into(),
            cwd: ".".into(),
            model: None,
            cursor: Mutex::new(None),
            child: Mutex::new(None),
            running: AtomicBool::new(false),
            events,
        };
        handle_value(
            &config,
            &json!({"type":"assistant","message":{"content":[{"type":"thinking","thinking":"why"},{"type":"text","text":"done"}]}}),
            &mut StreamState::default(),
        );
        assert!(
            matches!(received.recv().unwrap(), DriverEvent::ReasoningDelta(text) if text == "why")
        );
        assert!(matches!(received.recv().unwrap(), DriverEvent::TextDelta(text) if text == "done"));
    }

    #[test]
    fn deveco_native_text_shape_is_streamed() {
        let (events, received) = test_event_channel();
        let config = Config {
            provider: ProviderKind::DevEco,
            binary: ProviderKind::DevEco.command().into(),
            cwd: ".".into(),
            model: None,
            cursor: Mutex::new(None),
            child: Mutex::new(None),
            running: AtomicBool::new(false),
            events,
        };
        handle_value(
            &config,
            &json!({"type":"text","sessionID":"d1","part":{"text":"dev"}}),
            &mut StreamState::default(),
        );
        assert!(matches!(
            received.recv().unwrap(),
            DriverEvent::TextDelta(text) if text == "dev"
        ));
    }

    #[test]
    fn native_headless_argv_uses_each_verified_protocol() {
        fn args(provider: ProviderKind) -> (bool, Vec<String>) {
            let mut command = std::process::Command::new(provider.command());
            command.current_dir("/tmp/project");
            let stdin = configure_command(
                &mut command,
                provider,
                "task",
                Some("model"),
                Some("session"),
                Some(std::path::Path::new("/tmp/agy.log")),
            );
            (
                stdin,
                command
                    .get_args()
                    .map(|arg| arg.to_string_lossy().into_owned())
                    .collect(),
            )
        }

        let (stdin, codebuddy) = args(ProviderKind::CodeBuddy);
        assert!(stdin);
        assert!(
            codebuddy
                .windows(2)
                .any(|pair| pair == ["--output-format", "stream-json"])
        );
        assert!(
            args(ProviderKind::Copilot)
                .1
                .windows(2)
                .any(|pair| pair == ["--output-format", "json"])
        );
        assert!(
            args(ProviderKind::DevEco)
                .1
                .windows(3)
                .any(|part| part == ["run", "--format", "json"])
        );
        assert!(
            args(ProviderKind::Qwen)
                .1
                .windows(2)
                .any(|pair| pair == ["--output-format", "stream-json"])
        );
        assert!(
            args(ProviderKind::Antigravity)
                .1
                .windows(2)
                .any(|pair| pair == ["--conversation", "session"])
        );
    }
}

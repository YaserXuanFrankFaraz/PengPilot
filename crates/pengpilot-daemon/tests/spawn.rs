use std::path::Path;

use pengpilot_client::DaemonProcess;
use pengpilot_protocol::{Command, ResponsePayload};
use uuid::Uuid;

#[test]
fn spawns_and_serves_load_task_state() {
    let process = DaemonProcess::spawn(Path::new(env!("CARGO_BIN_EXE_pengpilot-daemon")))
        .expect("spawn pengpilot-daemon");
    let payload = process
        .client()
        .request(Uuid::nil(), Uuid::nil(), Command::LoadTaskState)
        .expect("LoadTaskState");
    assert!(matches!(payload, ResponsePayload::TaskState { .. }));
    let payload = process
        .client()
        .request(Uuid::nil(), Uuid::nil(), Command::GetSettings)
        .expect("GetSettings");
    assert!(matches!(payload, ResponsePayload::Settings { .. }));
}

# PengPilot Development Handoff

_Last updated: 2026-08-18, v0.1.20 + daemon WS /v1 (see §3)_

This document lets a fresh coding agent continue PengPilot R&D without
re-deriving context. Read it top to bottom; the "Next actions" section at the
end is the immediate starting point.

---

## 1. Status at a glance

| Item | State |
| --- | --- |
| Repo | `YaserXuanFrankFaraz/PengPilot`, branch `main` |
| Version | **0.1.20** (latest GitHub release, tagged `v0.1.20`) |
| Tests | **651 green** (`pengpilot` 305+10, `pengpilot-core` 275, `pengpilot-protocol` 57, `pengpilot-daemon` 3, `pengpilot-client` 1); 18 ignored driver live-tests live in core |
| Working tree | Clean except the user's uncommitted `src/app/sidebar.rs` (1-line theme tweak `.bg(sidebar)→surface`) — **never commit it; the user owns it** |
| Runtime | `bun ./scripts/dev.ts` owns `PengPilot Debug.app`; AGENTS.md governs its use |
| **Highest-priority work** | **Daemon migration (Phase 1 → 5)**, not yet complete |

Health checks: `cargo test` at workspace root; the dev watcher builds the Debug
app. Release rules, product restraint, performance and accessibility
requirements are binding in `AGENTS.md` and `RELEASING.md`.

---

## 2. Architecture — today and the target

### Today: app + protocol + core + daemon WS + thin client

- Cargo workspace: `pengpilot` + `pengpilot-protocol` + `pengpilot-core` +
  `pengpilot-daemon` + `pengpilot-client`. `pengpilot-core` holds the headless
  engine plus `serve()` (JSON-RPC over WebSocket `/v1`). The desktop app still
  calls the engine in-process; UI, md, transcript assembly, and the GPUI
  `terminal` widget stay in the app.
- `crates/pengpilot-protocol`: serde-only wire value types, no I/O. Envelope
  types (`ClientMessage` / `ServerMessage` / `ReplayCursor` /
  `WireDriverEvent`, `PROTOCOL_VERSION=3`). `Command` covers session runtime
  + probe/task-state; settings / attachments / workspace / drafts wait until
  those value types live here.
- `crates/pengpilot-daemon`: binds loopback, requires `PENGPILOT_DAEMON_TOKEN`,
  prints `DaemonReady` JSON on stdout, serves `PengPilotBackend`.
- `crates/pengpilot-client`: `DaemonClient` only (no supervisor yet). Core
  tests use it; the app does not depend on it yet.
- `src/model.rs` re-exports the protocol types so every `crate::model::X` path
  keeps working (no call-site churn). This is the deliberate "re-export bridge"
  pattern from waku's `src/lib.rs`. `src/main.rs` does the same for `i18n`,
  `identity`, `library`, `persistence`, `model_catalog`, pools/sessions,
  `usage`, `usage_history`, `composer_complete`, `skills`, `computer_use`,
  and `driver`.

### Target: waku's daemon + WebSocket RPC layout

Upstream waku (`egoist/waku`, checkout at **`/tmp/waku`, ~`1c65597`**) is the
reference. It is a 4-crate workspace:

- `waku-protocol` — pure serde contract: `protocol.rs` (JSON-RPC `Command` /
  `ResponsePayload` / `WireDriverEvent`), `PROTOCOL_VERSION=3`, value types.
- `waku-core` — headless engine (the daemon implementation): `driver/`,
  `persistence` (SQLite), `workspace` (git/fs), `usage`, `blob_store`,
  `command_env`, `skills`, settings.
- `waku-daemon` — thin binary: `--bind`, `--parent-pid`, `DaemonReady` line on
  stdout, `Backend::handle(Request)->ResponsePayload` seam + WS JSON-RPC server.
- `waku-client` — desktop side: `DaemonClient` (WebSocket), `DaemonSupervisor`/
  `DaemonProcess` (spawn/health/hot-swap), `WorkspaceClient`, `StateStore::remote`
  proxy, app-side `RemoteDriverControl`.

The desktop `src/` keeps only UI, md rendering, and transcript assembly; all
engine work lives in the daemon process and speaks JSON-RPC over one WS endpoint
with sequenced events, `ReplayCursor` resume, and epoch versioning.

PengPilot's migration mirrors waku's own history (waku's `WakuBackend` was
itself split out of the old monolith). **Use pengpilot naming; align structure.**
Porting waku's `daemon/server/client` code is the sanctioned approach (GPL-3.0).

---

## 3. Daemon migration plan & progress

User decisions (2026-08-18): **phased, each phase runnable + committed; port
waku upstream; structural alignment (pengpilot names, waku layout); push phases
1–5 to completion without asking; produce a packaged release at each clean
checkpoint** (Agent switches are expected; `v0.1.20` is the current fallback).

| Phase | Scope | Status |
| --- | --- | --- |
| 0 | Baseline: 0.1.19 (`9a0d3b3`) sizes/tests; freeze feature ports | ✅ |
| 1 | Workspace + `pengpilot-protocol`: move wire value types out of `src/model.rs`, re-export bridge | ✅ including `DriverEvent` (image_url, not GPUI) |
| 2 | `crates/pengpilot-core` (engine) + `crates/pengpilot-daemon` (thin binary, in-process backend first) | ✅ engine + in-process daemon; GPUI `terminal.rs` stays in app |
| 3 | `crates/pengpilot-client` WS RPC: app becomes a remote client (big milestone) | **In progress** (serve + DaemonClient landed; app still in-process) |
| 4 | Packaging/dev/release: embed daemon in `.app`, watcher runs both, size gates | ⬜ |
| 5 | Re-align to waku mainline; maintain provider verification | ⬜ |
| — | Package-hygiene pass (after daemonization): shrink DMG/ZIP/App | ⬜ |

### Phase 1 commits

- `a64388b` — workspace + protocol crate + `ProviderKind`/`ProviderResumeCursor` extracted.
- `282e3a8` / `4c24a5c` — i18n into protocol crate; `RuntimeMode`, `InteractionMode`,
  `ProviderModel`, `ProviderModelOption`, `FavoriteModel`, `ProviderAgentPreset` extracted.
- `e72277f` — `work` module (WorkflowStatus/Quadrant/FocusZone/InboxCollection) moved whole.
- `26cce3b` — v0.1.20 checkpoint release.
- session leaf types — `SessionWorkspace`, `SessionStatus`, `TurnStatus`,
  `QueuedMessage`, `Message`/`MessageRole`/`MessageAttachment`, `Checkpoint`/
  `CheckpointStatus`/`CheckpointFile`, `AgentTurn`, `ContextUsage`,
  `ActivityKind`, plus `unix_time`/`unix_time_millis`, live in
  `crates/pengpilot-protocol/src/session.rs` and are re-exported from
  `src/model.rs`. The `ActivityKind` tool-name test moved with them.
- `Project` + `projectless` predicates (`home_directory`, `workspace_root`,
  `is_projectless_path`, `is_legacy_root_path`) live in the protocol crate;
  `src/projectless.rs` keeps `create_workspace` and re-exports the predicates.
- `ProviderProbe` lives in `pengpilot-protocol::model`; discovery is
  `model_catalog::discover_probe_models`.
- `AgentSession` + transcript layer (`ReportedCommand`, `ActivityItem`,
  `ReasoningBlock`, `TranscriptBlock`, background-work/permission/user-input
  values, activity diff helpers) live in `crates/pengpilot-protocol/src/agent.rs`.
  Session tests moved with them. `DriverEvent` / `RuntimeEventCursor` live in
  `pengpilot-protocol::model`. `ComputerUseUpdated` carries `image_url`
  (PNG data URL); the app decodes it to `gpui::Image` in streaming.

### Phase 1 remaining — exact analysis (already done, don't re-explore)

Value types are extracted. `src/model.rs` is the re-export bridge.

**`ProviderProbe`**: extracted. Catalog fill lives in
`model_catalog::discover_probe_models`; the app caller is `runtime.rs`.

**`Project` / `projectless`**: extracted. Engine-side `create_workspace` /
migration stay in `src/projectless.rs`. Do not rebuild the deleted Box::leak
placeholder.

**`DriverEvent`**: extracted. Phase 3 still needs waku's `WireDriverEvent` /
`event_to_wire` for JSON-RPC (the in-process enum is not the wire envelope).

### Phase 2 commits

- `pengpilot-core` crate: `command_env`, `git_commit`, `git_branch`,
  `blob_store`, `worktree`, `checkpoint`. App re-exports keep `crate::*` paths.
- Persistence cluster: protocol gained `identity`, `AppLanguage`,
  `ThemePreference`, and `ComputerAppGrant`. `library.rs` + `persistence.rs`
  + core `build.rs` (drizzle migrations) live in `pengpilot-core`. App
  `build.rs` only watches `locales`. `remember_model_traits` /
  `model_traits_for` are `pub` because glob re-exports cannot leak
  `pub(crate)`. GPUI `Theme` and computer-use helpers stay in the app.
- Catalog cluster: `model_catalog`, `deepseek_pool` / `deepseek_session`,
  `opencode_pool` / `opencode_session`. Core now has `rust-i18n` + `tr!`
  (re-exports protocol `i18n`). `pub(crate)` pool/session APIs became `pub`
  for the app re-export.
- Driver cluster: protocol gained `DriverEvent`, `PlanUsage`, and
  `ComputerUseState.image_url`. Core gained `driver/`, remaining session
  adapters, `computer_use` helper I/O, and `usage` fetch. App streaming
  decodes the preview PNG; `ComputerUsePreview` still holds `gpui::Image`.
- In-process daemon (superseded): four `PengPilotBackend` commands landed
  before WS. Provider `--version` probe lives in `pengpilot-core::model`.
- Composer/skills/usage: `composer_complete`, `skills`, and `usage_history`
  live in core. `SkillSource::icon` keeps asset path strings (no `crate::ui`).
  The GPUI `src/terminal.rs` widget stays in the app; daemon-owned PTY waits
  until `OpenTerminal` is backed by a real `DaemonTerminal`.
- WebSocket slice: `serve()` + `EventSink` hub + `DaemonClient`. Token auth,
  origin allowlist, sequenced replay. Unimplemented `Command` variants return
  `Ack`. App still in-process.

### Phase 3 remaining

Wire the desktop app as a WS client (`DaemonSupervisor` / `DaemonProcess`,
`StateStore::remote`, `RemoteDriverControl`). Port remaining `Command` types
(settings, attachments, workspace, drafts, skills/usage-history catalogs).
Daemon PTY.

### Phase 2–5 starting notes (from the waku map)

- Engine modules to move into core (Phase 2): `driver/`, `persistence.rs`,
  `git_commit.rs`, `git_branch.rs`, `worktree.rs`, `checkpoint.rs`,
  `blob_store.rs`, `composer_complete.rs` (discovery side), `usage.rs`,
  `usage_history.rs`, `model_catalog.rs`, `deepseek_pool.rs`, `opencode_pool.rs`,
  `command_env.rs`, `computer_use.rs`, `skills.rs`, session adapters, terminal.
- Re-add PengPilot-only providers (OMP/Kiro/Hermes/Trae via ACP) to the core
  driver dispatch.
- Phase 3: port `protocol.rs`/`server.rs`/`client.rs`/`process.rs` from waku
  (`/tmp/waku`), wire the app's `runtime.rs`/`streaming.rs`/persistence to RPC.
- Phase 4: `scripts/bundle.sh`/`release.ts`/`dev.ts` embed + run the daemon;
  Sparkle updates carry it inside the `.app`.

---

## 4. Recent R&D history (this effort)

### Upstream waku ports landed (all on main, all tested)

| Commit | Feature (waku source commit) |
| --- | --- |
| `a8b137e` | `chore: apply cargo fmt` (user-confirmed keep) |
| `0057097` | Toasts visible while Settings is open (port `adaf22c`) |
| `0e8aab5` | Commit messages pinned to a cheap model tier — Claude `claude-haiku-4-5` low effort (live-verified); Codex `gpt-5.6-luna` + `model_reasoning_effort="none"` (port `7e81c90`) |
| `fa07f43` | Re-engage transcript tail-following when the reader scrolls back (port `1c65597`) |
| `7e9035b` | Sidebar/right-panel 200 ms width slide + notify gating; reduce-motion honored (ports `fab7825`+`7028cc2`+`4c483bc`) |
| `2cdcaa6` | Inline diffs in the transcript — provider file-edit → unified-diff normalization (`similar` crate), shared review-row renderer (port `26e9e5c`) |
| `0477aa8` | Removed 10 unsupported provider integrations (`json_cli` driver deleted) |

### Provider catalog (user decision, rafted in docs/README)

Supported = **waku official − Amp + {OMP, Hermes, Trae, Kiro} = 11 providers**
(Claude, Codex, Cursor, DeepSeek, Grok, OpenCode, Pi, OMP, Hermes, Trae, Kiro).
Amp stays in the enum but is hidden from the UI. Copilot was removed entirely
(`3ec5af2`); the other 10 stubs removed (`0477aa8`).

### Verification status (honest matrix, keep updating per release)

Live-tested `PENG_OK`: **Claude 2.1.233, Codex 0.147.0, Grok, Pi, OMP 17.3.5,
Cursor (cursor-agent), OpenCode 1.18.18, Trae 0.120.52 (full ACP turn)**, dsh
0.1.0-rc.6 (host startup + HTTP 200). **Kiro is installed but NOT authenticated**
— skip until `kiro-cli login` works (see `kiro-auth-gate` notes). Codex usage
limit resets **Aug 20** (cheap-tier commit model unverified until then).
Disclosure notes must be honest per `RELEASING.md` gates.

### Releases this effort touched

v0.1.14 (kiro no-browser fix + quit-black-screen fix) → v0.1.15 (Copilot
removal; corrected verification) → user's parallel v0.1.16–0.1.19 (media
library/Imagine, icon/chrome, packaging) → **v0.1.20** (daemon checkpoint).

Size baselines (all recorded, use for the package-hygiene phase):
- 0.1.15: DMG 15,709,373 / ZIP 15,793,302 / App 28,540 KB
- 0.1.19: DMG 14,817,667 / ZIP 14,908,747 / App 27,772 KB
- 0.1.20: DMG 14,896,618 / ZIP 14,963,103 / App 27,980 KB

---

## 5. Gotchas for the next agent

- **The user edits this repo in parallel.** Watch `git status` before big moves.
  Never `git add -A` (it swept `.memsearch/` + a stray temp file before); stage
  exact paths. `cargo fmt --all` reformats the user's not-yet-fmt'd files
  (e.g. `image_preview`, `library_page`, `render`, `sidebar`, `tests`) — revert
  those before committing, they belong to the user. `sidebar.rs` currently has
  the user's uncommitted 1-line tweak.
- **No feature additions while migrating** (user's freeze); migration commits
  must not sweep unrelated changes.
- **Each phase/checkpoint**: `cargo test` (all members), commit separately,
  push; produce a packaged release at clean checkpoints (the user relies on
  these as switch-Agent fallbacks).
- Commit messages: **no backticks in `-m`** (zsh executes them).
- `.env`/Mify credentials must never enter the repo or commits.
- The dev watcher (`bun ./scripts/dev.ts`) may not be running — confirm before
  relying on it (AGENTS.md).

---

## 6. Reference pointers

- `AGENTS.md`, `RELEASING.md`, `CHANGELOG.md`, `docs/providers.md`,
  `README.md` — binding product/dev rules and the verification matrix.
- Upstream waku checkout for porting: **`/tmp/waku`** (latest `main`).
- This ZCode session's memory (continuity context incl. diagnostics, decisions):
  `~/.zcode/cli/memories/projects/pengpilot-757098dfd71f24c9/memory/`
  (esp. `daemon-migration-plan.md`, `waku-upstream-merge-assessment.md`,
  `supported-provider-catalog.md`, `commit-hygiene-worktree.md`).
- Provider ACP wire notes and latency tuning: see those same memory files.

---

## 7. Next actions

1. Continue **Phase 3**: `DaemonSupervisor` / spawn, then point the app's
   `runtime.rs` at `DaemonClient` so the UI process no longer owns the engine.
2. After daemonization, run the **package-hygiene pass** (sizes vs the 0.1.20
   baseline table in §4).

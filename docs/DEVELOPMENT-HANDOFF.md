# PengPilot Development Handoff

_Last updated: 2026-08-18, v0.1.20 + session leaf types (see §3)_

This document lets a fresh coding agent continue PengPilot R&D without
re-deriving context. Read it top to bottom; the "Next actions" section at the
end is the immediate starting point.

---

## 1. Status at a glance

| Item | State |
| --- | --- |
| Repo | `YaserXuanFrankFaraz/PengPilot`, branch `main` |
| Version | **0.1.20** (latest GitHub release, tagged `v0.1.20`) |
| Tests | **627 green** (`pengpilot` 611+10, `pengpilot-protocol` 6) |
| Working tree | Clean except the user's uncommitted `src/app/sidebar.rs` (1-line theme tweak `.bg(sidebar)→surface`) — **never commit it; the user owns it** |
| Runtime | `bun ./scripts/dev.ts` owns `PengPilot Debug.app`; AGENTS.md governs its use |
| **Highest-priority work** | **Daemon migration (Phase 1 → 5)**, not yet complete |

Health checks: `cargo test` at workspace root; the dev watcher builds the Debug
app. Release rules, product restraint, performance and accessibility
requirements are binding in `AGENTS.md` and `RELEASING.md`.

---

## 2. Architecture — today and the target

### Today: single-crate app + a young protocol crate

- Root crate `pengpilot` (the macOS/GPUI desktop app; binary `pengpilot`, plus
  `src/bin/pengpilot_js_repl.rs`). Everything — provider drivers, SQLite
  persistence, git, usage, UI — runs in **one process**.
- **Cargo workspace** (added in `a64388b`): `members = [".", "crates/pengpilot-protocol"]`.
- `crates/pengpilot-protocol` (`pengpilot-protocol`, v0.1.20): serde-only wire
  value types, no I/O. Carries its own `rust_i18n::i18n!("../../locales")` +
  an equivalent `tr!` macro (shares the process-global locale with the app).
  Modules: `model` (ProviderKind, ProviderResumeCursor, RuntimeMode,
  InteractionMode, ProviderModel/Option, FavoriteModel, ProviderAgentPreset),
  `work` (WorkflowStatus, Quadrant, FocusZone, InboxCollection).
- `src/model.rs` re-exports the protocol types so every `crate::model::X` path
  keeps working (no call-site churn). This is the deliberate "re-export bridge"
  pattern from waku's `src/lib.rs`.

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
| 1 | Workspace + `pengpilot-protocol`: move wire value types out of `src/model.rs`, re-export bridge | **In progress** (~2/3) |
| 2 | `crates/pengpilot-core` (engine) + `crates/pengpilot-daemon` (thin binary, in-process backend first) | ⬜ |
| 3 | `crates/pengpilot-client` WS RPC: app becomes a remote client (big milestone) | ⬜ |
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

### Phase 1 remaining — exact analysis (already done, don't re-explore)

`AgentSession` stays app-side until the interlocking transcript layer moves.
`Project` is extracted.

**`AgentSession` cannot be extracted alone**: it references `ReportedCommand`
(`src/model.rs:1307`) and `TranscriptBlock` (`:2589`), and `TranscriptBlock`
chains to `ActivityItem` → `ReasoningBlock` plus custom serde
(`serialize_transcript_activities`/`deserialize_transcript_activities`). That
whole interlocking layer should move together in one large step (best with a
fresh full budget).

**`ProviderProbe`**: extracted. Catalog fill lives in
`model_catalog::discover_probe_models`; the app caller is `runtime.rs`.

**`Project` / `projectless`**: extracted. Engine-side `create_workspace` /
migration stay in `src/projectless.rs`. Do not rebuild the deleted Box::leak
placeholder.

**`DriverEvent` cannot be extracted as-is**: `ComputerUseUpdated` holds
`Arc<gpui::Image>` (UI-coupled). Phase 3 should introduce a wire-serializable
`WireDriverEvent` subset + an `event_from_wire` decoder (mirror waku's
`driver_wire.rs` / daemon `event_to_wire`).

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

1. Land the `AgentSession`/`TranscriptBlock`/`ActivityItem`/`ReasoningBlock`
   interlocking layer as one large step; then the remaining session/settings
   wire types.
2. Enter **Phase 2** (core crate + daemon binary, in-process backend), then
   **Phase 3** (WS RPC + `WireDriverEvent`), **Phase 4** (packaging), **Phase 5**
   (mainline alignment). Package a release at each clean checkpoint.
3. After daemonization, run the **package-hygiene pass** (sizes vs the 0.1.20
   baseline table in §4).

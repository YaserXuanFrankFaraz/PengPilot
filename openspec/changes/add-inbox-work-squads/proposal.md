## Why

PengPilot's advantage is that it is already a small native control plane: Rust, GPUI, one selected transcript in memory, a virtualized session list, and the local CLIs you already installed. Craft Agents has the interaction that list is missing — inbox, flags, zone focus, status under the fingers. Multica has the work model — a board of issues, and a squad of named agents that are just different CLIs. The job is to take those two behaviors and keep PengPilot's process small. It is not to become another Electron shell, another hosted board, or another in-process agent runtime.

## What Changes

- Keep the GPUI app as the only UI process. No extra WebView for the shell, no Node/bun sidecar, no new embedding or graph engine.
- Port Craft **interactions**: three-zone keyboard focus, unfinished list, flags, auto-archive on complete, row actions that work with keyboard and pointer.
- Port Multica **board + work** as an Eisenhower matrix: each quadrant contains 待开始 → 进行中 → 待人审. Completing auto-archives; Archive rows keep a quadrant label.
- Port Multica **squads of CLIs**: a named profile is a thin wrapper around an existing `ProviderKind`; a squad is a leader profile plus member profiles; only the leader starts; members start when mentioned.
- Chat-first New Task stays. A finished run still does not mark the work done.

## Capabilities

### New Capabilities

- `session-inbox`: Native list and kanban shell — three live columns, complete archives automatically, list-cheap rows.
- `work-items`: Work records distinct from provider sessions, with assignee, workflow, comments, and an execution log.
- `agent-profiles`: Named local wrappers around already-supported CLIs.
- `squads`: Leader-routed groups of those profiles. Multiple CLIs, one coordinator, no merged process.

### Modified Capabilities

- None. This repository has no archived OpenSpec specs yet.

## Impact

- **Stay small**: additive SQLite columns and a few narrow tables. No new runtime crates. Only the selected session's transcript is hydrated. Board columns virtualize like the current sidebar.
- **UI**: split today's date-grouped sidebar into nav + list/board; reuse transcript, composer, and right panel.
- **Runtime**: assignment and mention still go through existing drivers. A squad does not spawn a PengPilot-owned LLM.
- **Deferred**: YuuMira Knowledge Cockpit lives in `add-knowledge-cockpit` and must not start until this change has shipped and stayed lean.
- **Not in scope**: Multica server/team cloud, Craft Sources/automations, custom status editors, Gantt, Session Goals, QMD/embeddings, Electron/Tauri rewrite.

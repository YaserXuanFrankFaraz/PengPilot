## Context

See `proposal.md` for why. PengPilot is a GPUI desktop app that already runs Codex, Claude, Grok, OpenCode, Pi, and the rest as child CLIs. The session list is virtualized and backed by narrow SQLite rows; transcripts live in `session_details` and load only when a session is opened. That split is the memory advantage. Craft (Electron) and Multica (web + daemon) are product references only.

## Goals / Non-Goals

**Goals:**
- Keep the app a single native process plus the agent CLIs the user starts.
- Give existing sessions Craft-quality triage (unfinished list, flag, zone focus) on a standard kanban.
- Give work a Multica board: same records, columns by workflow status.
- Let several installed CLIs act as a squad without a new runtime.

**Non-Goals:**
- Rewriting the shell in Electron, Tauri, or a bundled web UI.
- Adding a JS sidecar, vector index, knowledge graph engine, or second SQLite for embeddings.
- Multica Cloud, daemons, team ACL, Slack/Lark.
- Craft Sources, automations, theme packs, custom status editors.
- Knowledge Cockpit (see `add-knowledge-cockpit`).
- Session Goals, Gantt, swimlanes.

## Decisions

### 1. Leanness is a design constraint, not a later pass
Rules for every new surface:

- No new heavy crate. Prefer types + SQLite + existing `list()`.
- Render reads memory only. Vault walks, git, and subprocesses stay off the frame path.
- Hydrate one selected transcript. Inbox, board, and pickers use narrow rows.
- A miss in a cache is "not ready", never a sync scan.
- Profiles and squads are small structs. Do not keep member transcripts loaded because they are on a squad.

Alternative: embed Craft's webui in a WKWebView. Rejected — PengPilot already pays for one optional browser webview; the shell must stay GPUI or the memory story is gone.

### 2. Two-layer rollout
Phase A: four kanban statuses, flag, nav + list + board on **sessions**.
Phase B: work items, profiles, squads. Board prefers work items; leftover sessions show as chats.

Knowledge is not a Phase C of this change.

### 3. Craft is interaction, not chrome
Take: `Cmd+1/2/3`, unfinished list, Flagged, status + flag on the row, compact one-column leading pane.
Leave: rounded floating cards, motion-heavy panel stacks, hierarchical labels, Sources/Skills/Automations as sibling products.

```
[ Nav ][ Inbox list or board ][ Transcript / work detail ][ existing right panel ]
```

Implementation stays in `inbox.rs` / `nav_rail.rs` / a board widget. Zed's GPUI lists and focus are the how-to reference.

`sidebar_width` is the list only. After inserting the nav, every consumer of "how wide is the left chrome" must go through `sidebar_material_width`: native tint, traffic-light clearance, `fitted_panel_widths`, and transcript wrap. Board view hides the list titlebar, so its own header must use `leftover_traffic_light_clearance` — the lights are window-absolute. Do not add a second settings control that the list footer already owns.

### 4. Multica board is first-class
Two independent axes:

- **Progress** (how far): 待开始 → 进行中 → 待人审 → 完成并归档
- **Quadrant** (what matters): 重要×紧急, Eisenhower. Default new work is 重要不紧急.

The default board is a 2×2 matrix. Each cell contains only 待开始, 进行中, and 待人审. Marking 完成并归档 removes the card from the board and puts it in Archive. Archive rows keep the quadrant as a label (重要且紧急 / 重要不紧急 / 紧急不重要 / 不紧急不重要). Moving across quadrants does not change progress. There is no 阻塞 column.

Each progress group is its own virtualized `list()`. Do not build one element tree of every card in every quadrant.

Alternative: only a linear 3-column board. Rejected — the user wants 四象限为底、格内走进度.
Alternative: a 阻塞 column. Rejected — not a progress stage.

### 5. Session stays the runner
Work item = record. Profile = name + `ProviderKind` + model + instructions. Squad = leader + members. Starting work still creates `AgentSession` and the current driver. Completing a session does not set work to `done`.

### 6. Multi-CLI squad, not a mega-agent
A squad may mix Codex, Claude, Grok, and the rest because each member is a profile. PengPilot never merges those CLIs. Only the leader session starts on assign. Mentions use `mention://profile/<uuid>`.

### 7. Persistence
Narrow tables only: `work_items`, `work_item_details`, `work_item_comments`, `agent_profiles`, `squads`, `squad_members`. Sessions and work items gain `workflow_status`, `important`, `urgent`, `flagged`, plus `work_item_id` / `agent_profile_id` on sessions. `done` is archive membership. No knowledge tables in this change.

### 8. i18n and a11y
`tr!` for new chrome. Status is icon + text. Board and list are fully keyboard-operable.

## Risks / Trade-offs

- [Three-zone shell uses more resident views] → compact mode shares one leading column; only the selected detail hydrates a transcript.
- [Board of hundreds of cards] → virtualize per column; Archive columns stay off the live board.
- [Squad fan-out] → never start every member; dedup in-flight leader sessions.
- [Scope creep back to YuuMira/Multica the product] → knowledge, cloud, and automations stay other changes.

## Migration Plan

1. Additive columns; existing sessions become `todo`, unflagged.
2. Ship unfinished list, four-quadrant board with inner progress, and Archive-on-done on sessions.
3. Ship work items, profiles, squads.
4. Rollback is unused nav; columns are nullable.

## Open Questions

None that block. Four-quadrant overlay on the same board remains a later change if still wanted.

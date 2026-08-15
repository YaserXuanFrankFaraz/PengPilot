## 1. Persistence and domain

- [x] 1.1 Add workflow_status, important, urgent, flagged, work_item_id, and agent_profile_id to the Drizzle sessions table and generate a Rust-applied migration
- [x] 1.2 Add work_items (including important and urgent), work_item_details, work_item_comments, agent_profiles, squads, and squad_members tables with list-narrow columns only
- [x] 1.3 Introduce Rust types for WorkflowStatus, Quadrant (important × urgent), Assignee, WorkItem, AgentProfile, and Squad without changing SessionStatus
- [x] 1.4 Backfill existing sessions to workflow_status=todo, important=true, urgent=false, flagged=false, and keep listing a scan of narrow rows
- [x] 1.5 Cover the four progress stages, done-means-archived, quadrant independence, backfill, and assignee serialization with unit tests

## 2. Craft interaction shell

- [x] 2.1 Split the current sidebar into a focusable nav rail and a virtualized middle list (`inbox.rs` / `nav_rail.rs`) without adding a WebView
- [x] 2.2 Add the unfinished list, Archive (done), and Flagged, all driven by cached snapshots
- [x] 2.3 Show workflow status and flag on each row without loading transcripts; pair color with icon or text
- [x] 2.4 Add Cmd+1/2/3 zone focus, row keyboard actions for status/flag, and command-palette entries
- [x] 2.5 Localize new chrome in locales/app.yml, zh-CN.yml, and ja.yml
- [x] 2.6 Unit-test collection membership, grouping cache, and keyboard action routing (no visual test)

## 3. Multica board

- [x] 3.1 Add a 2×2 Eisenhower board; each quadrant contains only 待开始, 进行中, and 待人审
- [ ] 3.2 Virtualize each progress group with `list()` so off-screen cards are not built
- [x] 3.3 Moves inside a quadrant change only progress; moves across quadrants change only importance/urgency
- [x] 3.4 Marking 完成并归档 removes the card from the board and lists it in Archive with a quadrant label
- [x] 3.5 After work items exist, make the Work nav default to the four-quadrant board

## 4. Agent profiles (CLI identities)

- [ ] 4.1 Add create/edit/archive UI for local profiles (name, existing provider, model, instructions)
- [ ] 4.2 Let New Task pick either a raw provider or a profile; stamp agent_profile_id when a profile is used
- [ ] 4.3 Supply profile instructions through the provider's existing mechanism, or a single first-turn visible block
- [ ] 4.4 Keep raw-provider starts working with a null profile id; do not add a new process type

## 5. Work items

- [ ] 5.1 Add create/detail surfaces for work items: title, description, status, assignee, comments, execution log
- [ ] 5.2 Implement assign-and-start and assign-without-start
- [ ] 5.3 Bind sessions to work items; allow attaching a chat-first session later
- [ ] 5.4 Keep session completion from marking work done; auto-revert sole failed in_progress work to todo
- [ ] 5.5 Execution log can open, stop, and retry a run; retry uses the original profile
- [ ] 5.6 Prefer work items on the board while still listing orphan sessions as chats

## 6. Multi-CLI squads

- [ ] 6.1 Add create/edit/archive UI: name, leader profile, member profiles (any installed CLI), role notes, squad instructions
- [ ] 6.2 Assigning a squad and confirming start queues only the leader session and prepends protocol + roster + instructions
- [ ] 6.3 Parse structured mention tokens in work comments; queue the mentioned profile without changing assignee
- [ ] 6.4 Apply leader re-awaken, self-comment ignore, and in-flight dedup rules from the squad spec
- [ ] 6.5 Archiving a squad reassigns its work to the former leader and removes it from pickers
- [ ] 6.6 Unit-test routing, dedup, mixed-provider rosters, and archive transfer

## 7. Apply-time validation

- [ ] 7.1 Wait for the existing dev watcher rebuild (do not start a second watcher) and exercise inbox, board move, assign, and a squad mention in the signed debug app
- [ ] 7.2 Confirm inbox and board scrolling do not hit disk or spawn processes from render, and that only the selected transcript is loaded
- [ ] 7.3 Confirm Cargo.toml gained no JS runtime, embedding, or graph-engine dependency
- [ ] 7.4 Run `openspec validate add-inbox-work-squads` before requesting archive

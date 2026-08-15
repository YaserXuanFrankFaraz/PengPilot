## Purpose

Gives PengPilot Craft-quality list interaction and an Eisenhower board whose every quadrant uses the same three live progress stages, without adding a second UI runtime or hydrating work that is not on screen.

## ADDED Requirements

### Requirement: Three-zone shell
The app SHALL present a nav rail, a work list, and a detail surface as three independently focusable zones. The existing transcript, composer, and right panel SHALL remain the detail surface for the selected session or work item. The shell SHALL be implementable as native GPUI panels and MUST NOT introduce a WebView, JavaScript runtime, or Electron-style floating card chrome for these zones.

#### Scenario: Focus moves between zones
- **WHEN** the user presses the platform equivalent of `Cmd+1`, `Cmd+2`, or `Cmd+3`
- **THEN** keyboard focus moves to the nav rail, the work list, or the detail/composer surface respectively
- **AND** the focused zone shows a visible focus treatment

#### Scenario: Compact width
- **WHEN** the window is too narrow to show nav rail and work list side by side
- **THEN** the shell SHALL collapse to a single leading column that can switch between nav and list
- **AND** the user can still reach both surfaces by keyboard

### Requirement: Progress is four named stages; done archives
Work progress SHALL use exactly these stages, in order: `todo` (待开始), `in_progress` (进行中), `in_review` (待人审), `done` (完成并归档). The live board SHALL show only the first three. Setting status to `done` SHALL archive the work automatically and remove it from every quadrant. There is no `blocked` or `backlog` workflow status. The nav rail SHALL expose the board, an unfinished list, Archive (all `done` work), and Flagged. Flagged is a pin, not a progress stage. Archive rows SHALL show the work's Eisenhower quadrant as a text label, not by color alone.

#### Scenario: Unfinished work is live
- **WHEN** a session or work item is `todo`, `in_progress`, or `in_review`
- **THEN** it SHALL appear in the unfinished list
- **AND** it SHALL appear in the matching progress group of its quadrant
- **AND** it SHALL NOT appear in Archive

#### Scenario: Completing archives automatically
- **WHEN** the user marks a row 完成并归档 (status `done`)
- **THEN** the row SHALL leave the unfinished list
- **AND** it SHALL appear in Archive with its quadrant label still visible
- **AND** it SHALL leave every quadrant on the live board
- **AND** any live provider process SHALL keep running until the user explicitly stops it

#### Scenario: Archive shows which quadrant it came from
- **WHEN** the user opens Archive
- **THEN** each row SHALL show a quadrant label such as 重要且紧急 or 重要不紧急
- **AND** that label SHALL match the importance and urgency stored on the work item

#### Scenario: Reopening leaves Archive
- **WHEN** the user sets an archived row back to `todo`, `in_progress`, or `in_review`
- **THEN** the row SHALL leave Archive
- **AND** it SHALL return to the unfinished list and to the matching live group in its quadrant

#### Scenario: Flag is not a stage
- **WHEN** the user flags an archived row
- **THEN** the row SHALL appear in Flagged
- **AND** its workflow status SHALL remain `done`

### Requirement: Eisenhower quadrant is independent of progress
Every work item SHALL have an importance flag and an urgency flag, forming one of four quadrants: 重要且紧急, 重要不紧急, 紧急不重要, 不紧急不重要. Changing the quadrant MUST NOT change progress. Changing progress MUST NOT change the quadrant. New work SHALL default to 重要不紧急 unless the user picks another quadrant. Chat-first sessions without a work item MAY stay unquadranted until attached.

#### Scenario: Move a card to another quadrant
- **WHEN** the user moves `PP-12` from 重要且紧急 to 重要不紧急
- **THEN** its progress SHALL stay `in_progress`
- **AND** it SHALL appear in 进行中 inside 重要不紧急

#### Scenario: Same three live stages in every quadrant
- **WHEN** the four-quadrant board is shown
- **THEN** each quadrant SHALL present only 待开始, 进行中, and 待人审
- **AND** a live card SHALL appear in exactly one quadrant and exactly one of those three groups

### Requirement: Workflow status is distinct from runtime status
Every listable session or work item SHALL have a workflow status from the fixed set `todo`, `in_progress`, `in_review`, `done`. Runtime `SessionStatus` (`idle`, `connecting`, `working`, `waiting`, `failed`) SHALL continue to describe the live provider process and MUST NOT be used as a progress stage or a quadrant.

#### Scenario: Working session still has a workflow state
- **WHEN** a session is `working` and its workflow status is `todo`
- **THEN** the list row SHALL show both a runtime activity indicator and the `待开始` label or icon
- **AND** the row SHALL remain in 待开始

#### Scenario: Status change is keyboard reachable
- **WHEN** a list row is focused
- **THEN** the user SHALL be able to change workflow status and toggle the flag from the row menu, the command palette, or documented shortcut keys
- **AND** the same actions SHALL be available by pointer

### Requirement: List rows stay list-cheap
The unfinished list, Archive, Flagged, and board columns SHALL render from narrow persisted fields only. A row builder MUST NOT load transcripts, walk the filesystem, or spawn subprocesses.

#### Scenario: Long inbox stays virtualized
- **WHEN** a collection contains more rows than fit on screen
- **THEN** the list SHALL virtualize with `list()`
- **AND** per-frame work SHALL stay proportional to visible rows

#### Scenario: Grouping does not rebuild session state
- **WHEN** the user groups the list by date or by workflow status
- **THEN** grouping SHALL use a cache refreshed at most once per frame
- **AND** changing the selected row SHALL NOT rebuild the whole collection

### Requirement: The default board is four quadrants with progress inside
The Work surface SHALL default to a 2×2 Eisenhower board once work items exist. Each quadrant SHALL contain only the three live progress groups 待开始, 进行中, and 待人审. A list view of unfinished work remains available. Moving a card between progress groups in the same quadrant SHALL change only progress. Moving a card to another quadrant SHALL change only importance and urgency. Marking 完成并归档 SHALL take the card off the board and into Archive. Each progress group SHALL virtualize so off-screen cards are not built.

#### Scenario: Board progress move updates only progress
- **WHEN** the user moves a card from 待开始 to 待人审 inside 重要且紧急
- **THEN** the work item workflow status SHALL become `in_review`
- **AND** its quadrant SHALL stay 重要且紧急
- **AND** the unfinished list SHALL show 待人审

#### Scenario: Board is keyboard operable
- **WHEN** a board card is focused
- **THEN** arrow keys SHALL move between cards and columns
- **AND** a documented key SHALL move the card to another column without requiring a pointer

#### Scenario: Off-screen cards stay unbuilt
- **WHEN** a column contains more cards than fit on screen
- **THEN** only visible cards SHALL be constructed for that frame
- **AND** scrolling SHALL not load transcripts

### Requirement: Shell stays a single native process
Inbox, board, and nav SHALL run in the existing GPUI process. They MUST NOT spawn a sidecar, embed a web app, or keep more than the selected session's transcript in memory for those surfaces.

#### Scenario: Selecting a card hydrates one transcript
- **WHEN** the user selects a different row or card
- **THEN** the detail surface MAY load that session's transcript
- **AND** the previously selected transcript SHALL be eligible to drop from memory once unused

#### Scenario: No extra runtime for the shell
- **WHEN** the inbox or board is open
- **THEN** the process tree SHALL NOT include a Node, bun, or Electron helper started by PengPilot for that UI

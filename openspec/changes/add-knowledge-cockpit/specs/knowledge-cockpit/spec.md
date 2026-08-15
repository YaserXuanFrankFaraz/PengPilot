## Purpose

Gives PengPilot a local Knowledge Cockpit so agents compile a Markdown wiki once, keep its links and health visible, and answer Ask Wiki from the user's own pages instead of rediscovering sources on every query.

## ADDED Requirements

### Requirement: Vault on disk is the source of truth
The system SHALL treat a knowledge base as an attached local directory of Markdown files (an Obsidian-compatible vault). PengPilot SHALL persist only vault metadata (id, display name, path, enabled, default flag, last-scan timestamps). Official pages, drafts, manifests, and graph exports MUST live in the vault. The cockpit MAY be disabled. Disabling MUST NOT delete vault files.

#### Scenario: Attach an existing vault
- **WHEN** the user attaches a folder that already contains Markdown pages
- **THEN** the cockpit SHALL list that vault
- **AND** it SHALL NOT copy the files into PengPilot's app data directory

#### Scenario: Init a new vault
- **WHEN** the user creates a vault at an empty folder
- **THEN** the system SHALL write a minimal wiki layout (index, log, categories, drafts directory)
- **AND** later scans SHALL find those files on disk

#### Scenario: Missing folder degrades
- **WHEN** a previously attached vault path is missing
- **THEN** the cockpit SHALL show an explicit unavailable state
- **AND** it MUST NOT crash or block the rest of the app

### Requirement: Pages, drafts, and inbox are list-cheap
The Knowledge nav surface SHALL show vaults, official pages grouped by category, a draft inbox, and recent knowledge tasks. Those lists SHALL render from a background-built snapshot. A row builder MUST NOT walk the filesystem, parse the vault, or spawn a process.

#### Scenario: First open scans in the background
- **WHEN** the user opens Knowledge and no current snapshot exists
- **THEN** the UI SHALL show a not-ready state
- **AND** a background scan SHALL populate pages, drafts, and counts, then notify

#### Scenario: Draft inbox holds unpromoted pages
- **WHEN** the vault contains Markdown under the drafts directory
- **THEN** those files SHALL appear in the knowledge inbox
- **AND** promoting a reviewed draft SHALL move it to an official category in the vault

### Requirement: Compile tasks use existing sessions
Ingest, session distillation, quality repair, graph export, and Ask Wiki SHALL run as ordinary PengPilot sessions (optionally bound to a work item), with the vault as the working directory and wiki skills available. Completing a compile session MUST NOT rewrite the vault except through that session's file edits. The cockpit SHALL record each such run in a knowledge task list.

#### Scenario: Ingest becomes a reviewable draft batch
- **WHEN** the user asks to ingest one or more source files into the vault
- **THEN** the system SHALL start a session that writes draft pages plus source-summary pages
- **AND** official pages SHALL change only after the user confirms the draft review

#### Scenario: Distill a session into the wiki
- **WHEN** the user distills a listed PengPilot session into the default vault
- **THEN** a compile session SHALL run against only that session's content
- **AND** a later Ask Wiki about the same topic SHALL be able to cite the new or updated pages

### Requirement: Ask Wiki answers from the vault
Ask Wiki SHALL start a knowledge task whose prompt instructs the agent to search and read the attached vault and to cite page titles or wikilinks. The answer MUST be written back as a task result the user can open. Ask Wiki MUST NOT claim knowledge that is not in the vault without marking it as outside the wiki.

#### Scenario: Ask Wiki from the cockpit
- **WHEN** the user submits "What are the key points of X?" in Ask Wiki
- **THEN** a session SHALL start against the current default vault
- **AND** the Knowledge task list SHALL show that run as in progress until it settles

#### Scenario: Empty vault is honest
- **WHEN** the user asks the wiki and the vault has no official pages
- **THEN** the task result SHALL say the wiki has no matching pages
- **AND** it MUST NOT invent citations

### Requirement: Health report is visible and actionable
The cockpit SHALL show a health snapshot with at least structure, connections, freshness, and coverage or credibility. Each dimension SHALL have a label plus a numeric or ordinal score, never color alone. Choosing a finding SHALL open the related page, draft, or a repair task.

#### Scenario: Orphan page is a connection finding
- **WHEN** an official page has no incoming or outgoing wikilink
- **THEN** the connections dimension SHALL list it
- **AND** the user SHALL be able to start a quality-repair session from that finding

#### Scenario: Stale scan does not block Ask Wiki
- **WHEN** the last health snapshot is older than the latest vault write
- **THEN** the cockpit SHALL mark health as stale
- **AND** Ask Wiki SHALL still run against the current files

### Requirement: Graph is a derived export
The cockpit SHALL display a knowledge graph from a derived export in the vault (nodes with id, label, category; links with source, target, relation). The graph is not the source of truth. Rebuilding the graph SHALL be an explicit or scheduled compile task. Selecting a node SHALL open the matching page if it exists.

#### Scenario: Graph missing
- **WHEN** the vault has pages but no graph export yet
- **THEN** the graph view SHALL offer a generate action
- **AND** generating SHALL create or refresh the export file without deleting official pages

#### Scenario: Graph render uses the snapshot
- **WHEN** the graph view is on screen
- **THEN** it SHALL read only the cached export snapshot
- **AND** a miss SHALL show not-ready rather than parse JSON on the UI thread

### Requirement: Default vault is the compile target
The user SHALL be able to mark one attached vault as default. Compile and Ask Wiki tasks started from the cockpit SHALL target the default vault. Sessions started outside Knowledge MAY still receive that vault path only when the user or a profile explicitly attaches it.

#### Scenario: Switch default
- **WHEN** the user sets vault B as default
- **THEN** the next Ask Wiki SHALL use vault B
- **AND** vault A files SHALL remain untouched

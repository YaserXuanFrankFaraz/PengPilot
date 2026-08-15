## Purpose

Lets a leader CLI receive work and route it to other installed CLIs, so PengPilot can run a Multica-style squad as several existing provider sessions rather than one merged agent.

## ADDED Requirements

### Requirement: Squad has one leader and many members
A squad SHALL have a name, exactly one leader agent profile, optional squad instructions, and zero or more members. Members MAY be agent profiles bound to different providers. The leader SHALL automatically be a member. A profile MAY belong to many squads. The system MUST NOT start a combined process for the squad.

#### Scenario: Create a squad
- **WHEN** the user creates a squad named "Delivery" and selects profile "Lead" as leader
- **THEN** the squad SHALL be saved
- **AND** "Lead" SHALL appear in the roster as leader

#### Scenario: Members can be different CLIs
- **WHEN** the leader profile uses Codex and a member profile uses Claude Code
- **THEN** the squad SHALL accept both
- **AND** each later run SHALL start as a separate session on that profile's provider

#### Scenario: Leader cannot be removed directly
- **WHEN** the user tries to remove the current leader from the roster
- **THEN** the system SHALL refuse
- **AND** the user MUST choose another leader first

### Requirement: Assigning a squad starts only the leader
Assigning a work item that is not `done` to a squad and confirming start SHALL make the squad the assignee and queue exactly one run for the leader profile. The system MUST NOT start a run for every member.

#### Scenario: Squad assignment queues the leader
- **WHEN** the user assigns `PP-12` in `todo` to squad "Delivery" and confirms start
- **THEN** one session SHALL start for the leader profile
- **AND** no member session SHALL start until the leader routes to that member

#### Scenario: Assign without start is quiet
- **WHEN** the user assigns a work item to a squad and chooses not to start
- **THEN** the squad SHALL become the assignee
- **AND** no leader session SHALL start until the user explicitly starts it

### Requirement: Leader briefing includes protocol, roster, and instructions
Each time the leader run starts or is re-awakened on a work item assigned to its squad, the system SHALL append three briefing blocks to the leader's profile instructions: a fixed operating protocol, the current roster with mention tokens, and the squad instructions. The protocol SHALL tell the leader to move assigned work to `in_progress` (进行中) on first pickup, route by mention, record a short evaluation, stop after routing, and move the work to `in_review` (待人审) only when the overall goal is met. The leader MUST NOT change the work item's Eisenhower quadrant unless the user asked.

#### Scenario: Leader sees a copyable roster
- **WHEN** a leader session starts on squad-assigned work
- **THEN** the briefing SHALL include one mention token per active member
- **AND** a plain-text `@name` that is not a mention token SHALL NOT start a member run

### Requirement: Mention is the routing signal
A mention of an agent profile in a work-item comment or leader message SHALL queue a run for that profile on the same work item without changing the work item's assignee. A mention of a squad SHALL awaken only that squad's leader and MUST NOT change the assignee. The leader's own comments MUST NOT re-trigger the leader.

#### Scenario: Leader mention starts a member
- **WHEN** the leader posts a comment that mentions member profile "Frontend"
- **THEN** the system SHALL queue a session for "Frontend" on the same work item
- **AND** the work item assignee SHALL remain the squad

#### Scenario: Explicit mention does not also awaken the leader
- **WHEN** a member comment mentions another profile
- **THEN** the mentioned profile SHALL be queued
- **AND** the leader SHALL NOT be queued for that same comment
- **AND** if the commenting profile is an agent reporting a result and also mentions another agent, the leader SHALL still be awakened to re-evaluate

#### Scenario: Dedup in-flight leader work
- **WHEN** the leader already has a queued or running session on the work item
- **THEN** a new leader trigger SHALL NOT enqueue a second overlapping leader session

### Requirement: Routing does not mean the work is done
Dispatching a member MUST NOT set the work item to `done` or `in_review`. The parent work item SHOULD stay `in_progress` while members work. `done` remains a human confirmation.

#### Scenario: First dispatch keeps work in progress
- **WHEN** the leader routes `PP-12` to one member
- **THEN** `PP-12` SHALL be `in_progress` after first pickup
- **AND** it SHALL NOT become `in_review` solely because a member run completed

### Requirement: Archive is one-way
Archiving a squad SHALL remove it from pickers and mention menus. Work items currently assigned to that squad SHALL be reassigned to the former leader profile. Historical comments and execution rows SHALL remain.

#### Scenario: Archive transfers open work
- **WHEN** the user archives squad "Delivery" that owns `PP-12`
- **THEN** `PP-12` SHALL be assigned to the former leader profile
- **AND** "Delivery" SHALL no longer appear in the assignee picker

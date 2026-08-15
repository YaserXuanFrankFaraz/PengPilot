## Purpose

Separates a lasting piece of work from a provider run so PengPilot can assign, review, retry, and discuss work without treating a finished session as the whole job.

## ADDED Requirements

### Requirement: Work item is the unit of work
The system SHALL persist a work item as a first-class record with at least: identifier, human-readable key, title, optional description, workflow status, importance, urgency, optional assignee, optional project, timestamps, and an execution log. A work item MUST be able to exist with zero sessions. New work SHALL default to important and not urgent.

#### Scenario: Create work without starting a run
- **WHEN** the user creates a work item with a title and does not start it
- **THEN** the work item SHALL be saved
- **AND** no provider session SHALL be created

#### Scenario: Work item has a stable key
- **WHEN** a work item is created
- **THEN** it SHALL receive a workspace-local key such as `PP-12`
- **AND** that numeric sequence SHALL increase monotonically in the local database

### Requirement: Session is one execution of work
A provider session SHALL remain the execution vehicle. A session MAY belong to at most one work item. One work item MAY have many sessions over time. Completing, failing, or cancelling a session MUST NOT by itself set the work item to `done`.

#### Scenario: First assignment starts a run
- **WHEN** the user assigns a work item that is not `done` to an agent profile and confirms start
- **THEN** the system SHALL create a session bound to that work item and profile
- **AND** the session SHALL appear in the work item's execution log

#### Scenario: Finished run is not finished work
- **WHEN** a bound session's runtime status becomes `idle` after a successful turn
- **THEN** the work item workflow status SHALL remain unchanged unless the agent or user explicitly updates it
- **AND** the user SHALL still be able to add comments or start another run

#### Scenario: Chat-first session has no work item
- **WHEN** the user starts a new session from the existing New Task flow without filing work
- **THEN** the session SHALL work as it does today
- **AND** the user MAY later attach it to a new or existing work item

### Requirement: Workflow transitions are explicit
Members and agents MAY set a work item to any workflow status. The system SHALL NOT infer `done` from a completed session. The only automatic workflow change the system SHALL make in this change is: if a run fails, the work item is `in_progress`, and no other run on that work item is active or retrying, the work item SHALL return to `todo`.

#### Scenario: Agent moves work into review
- **WHEN** an assigned agent sets the work item status to `in_review`
- **THEN** the work item SHALL appear in 待人审 inside its current quadrant
- **AND** the change SHALL be recorded on the work item timeline

#### Scenario: Failed sole run returns work to todo
- **WHEN** the only active run on an `in_progress` work item fails and no retry is queued
- **THEN** the work item SHALL move to `todo` (待开始)

#### Scenario: Assign without start does not start a run
- **WHEN** the user assigns a `todo` work item to an agent profile and chooses not to start
- **THEN** the assignee SHALL be stored
- **AND** no session SHALL start until the user explicitly starts it

### Requirement: Execution log is per work item
The work-item detail view SHALL list every bound session as an execution row with trigger, agent profile, runtime status, and timestamps. The user SHALL be able to open a run's transcript, stop an active run, and retry a failed or cancelled run.

#### Scenario: Retry keeps the original profile
- **WHEN** the user retries a failed run after the work item's assignee has changed
- **THEN** the retry SHALL use the profile that executed the original run
- **AND** a new session SHALL be appended to the same work item's log

#### Scenario: Changing assignee does not stop a live run
- **WHEN** the user changes the work item assignee while a session is working
- **THEN** that session SHALL continue until the user stops it from the execution log
- **AND** a new run SHALL start for the new assignee only if the assign-and-start path is used

### Requirement: Work detail stays native
Selecting a work item SHALL show its title, description, workflow status, assignee, comments, and execution log in the detail zone. Opening a run SHALL reuse the existing transcript and composer. The work detail MUST be reachable by keyboard and MUST NOT encode status only with color.

#### Scenario: Open run from the log
- **WHEN** the user activates an execution-log row
- **THEN** the detail zone SHALL show that session's transcript and composer
- **AND** a way back to the work item SHALL remain available

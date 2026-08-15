## Purpose

Lets the user treat each installed coding CLI as a named teammate — a thin local profile over an existing PengPilot provider — without starting a second agent runtime.

## ADDED Requirements

### Requirement: Profile is a local identity
The system SHALL persist agent profiles with at least: identifier, display name, provider, optional model, optional instructions, and default access/interaction settings that the selected provider supports. A profile is not a long-running process and MUST NOT add a new executable type. It SHALL bind to one existing `ProviderKind` that PengPilot already drives. Profiles SHALL be local to this machine and MUST NOT require a PengPilot account.

#### Scenario: Create a named profile
- **WHEN** the user creates a profile named "Reviewer" bound to Codex with written instructions
- **THEN** the profile SHALL appear in assignee and session-start pickers
- **AND** later sessions started from that profile SHALL use that provider, model, and instructions

#### Scenario: Provider still has to be installed
- **WHEN** the bound provider CLI is missing from `PATH` and no explicit binary is configured
- **THEN** the profile SHALL remain visible
- **AND** starting a run SHALL fail with the existing provider-detection error rather than deleting the profile

### Requirement: Profiles can be chosen without replacing raw providers
The New Task flow SHALL continue to allow starting a session from a raw `ProviderKind`. Choosing a profile SHALL populate provider, model, and instructions for that session and store the profile id on the session.

#### Scenario: Raw provider start still works
- **WHEN** the user starts a session by picking Claude Code with no profile
- **THEN** the session SHALL start as it does today
- **AND** `agent_profile_id` SHALL be empty

#### Scenario: Profile start stamps the session
- **WHEN** the user starts a session from the "Reviewer" profile
- **THEN** the session SHALL record that profile id
- **AND** the list row SHALL show the profile display name in addition to the provider

### Requirement: Assignee picker lists profiles
When assigning a work item, the assignee picker SHALL list agent profiles and squads. Selecting a profile and confirming start SHALL create a session for that profile. Selecting a profile with "don't start" SHALL store the assignee only.

#### Scenario: Assign and start
- **WHEN** the user assigns work item `PP-12` to profile "Reviewer" and confirms start
- **THEN** a session SHALL be created with that profile and bound to `PP-12`

#### Scenario: Archived profile cannot be assigned
- **WHEN** a profile has been archived
- **THEN** it SHALL NOT appear as an assignable assignee
- **AND** existing sessions that used it SHALL keep their stored profile id for history

### Requirement: Instructions are prepended once per run
When a run starts from a profile that has instructions, the system SHALL supply those instructions to the provider session using the provider's supported mechanism (system/instructions field or an initial hidden/user bootstrap the provider already accepts). The transcript MUST NOT show private provider control markers.

#### Scenario: Instructions reach the run
- **WHEN** a profile with non-empty instructions starts a new session
- **THEN** the provider process SHALL receive those instructions before the first user prompt of that run

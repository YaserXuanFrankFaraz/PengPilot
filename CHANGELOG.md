# Changelog

All notable changes to PengPilot. This file is the **source of truth for the release
notes shown in the in-app updater**: [`scripts/release.ts`](scripts/release.ts)
extracts the section whose heading matches the version being released
(`Cargo.toml`) and publishes it next to the update, so Sparkle shows it in
the update prompt.

Format follows [Keep a Changelog](https://keepachangelog.com). Add a new
`## [<version>]` section at the top for each release, matching the version in
`Cargo.toml`.

Write release notes for the final product users receive, not the development
history. When a feature is still unreleased, fold its fixes and refinements into
the original feature bullet instead of adding separate entries for them.

## [unreleased]

- Enlarge the transparent PengPilot mark in the new-task empty state.
- Use official provider marks with one shared visual footprint in Settings and
  the Chat Box picker.
- Hide Kimi Code from the featured and public catalogs while keeping existing
  sessions compatible.

## [0.1.9]

- Replace the new-task empty-state sparkle with the PengPilot app icon.
- Remove the unsupported OpenClaw integration.
- Remove `(CLI)` from Chat Box provider short names and label DeepSeek Harness
  without that suffix in Settings.
- Hide Prime Agent from the featured catalog and public provider lists until it
  passes real end-to-end validation and demonstrates user demand. Existing
  Prime Agent sessions remain compatible; any future restoration uses the same
  detected/undetected alphabetical ordering without default priority.

## [0.1.8]

- Focus the provider catalog on 15 CLIs, labeled with `(CLI)` in menus.
  Prime Agent is listed first as the recursive self-evolving agent; then
  Claude Code, Codex, Cursor, OpenCode, Grok Build, Kiro, Trae, DeepSeek
  Harness, Hermes Agent, OpenClaw, GitHub Copilot, Kimi Code, Pi, and Oh My
  Pi. Existing sessions on other CLIs still run.
- Replace the old Inbox navigation with All Tasks and Four-Quadrant boards,
  plus a compact Four-Quadrant shortcut beside New Task
- Add independently scrollable workflow lanes, persistent card drag-and-drop,
  and a global Completed or Archived lane shared by every quadrant
- Clarify workflow and quadrant menu labels and use one consistent
  Four-Quadrant icon across both entry points

## [0.1.7]

- Fix Computer Use startup failures caused by macOS Unix socket path limits
- Make each Board workflow lane independently scrollable
- Keep Board card titles to one truncated line and support persistent drag-and-drop
  across quadrants and workflow lanes

## [0.1.6]

- Add session Inbox views for unfinished, flagged, archived, and board work
- Expand the CLI catalog to 24 providers, including Prime Agent, with PATH-aware
  grouping and alphabetical sorting
- Remove anonymous product analytics, its setting, installation identifier,
  network sender, dependency, and release configuration
- Keep local Usage history while collecting no PengPilot telemetry
- Focus current releases on Apple silicon macOS; Intel Macs, Windows, and Linux
  remain untested and unsupported

## [0.1.5]

- Reduce the macOS ad-hoc release size while preserving runtime features
- Clarify PengPilot's independent identity and thank the Waku project

## [0.1.4]

- Replace the PengPilot app icon with the latest eagle-and-P artwork

## [0.1.3]

- Refine the PengPilot app icon with the latest eagle-and-P artwork

## [0.1.2]

- Replace the PengPilot app icon with the new red, blue, and silver artwork

## [0.1.1]

- Refresh the app icon with the new white-background artwork
- Match Hermes Agent icon sizing across provider settings and menus
- Correct PengPilot release URLs and local ad-hoc release configuration
- Refresh the bilingual README while preserving Waku attribution and GPL duties

## [0.1.0]

- Rebrand the app as PengPilot with new bundle IDs, data paths, and icon
- Add Oh My Pi, Kiro CLI, and Hermes Agent CLI providers
- Add Grok Build history to Usage charts and cost aggregation

## [0.0.13]

- Add DeepSeek Harness provider
- Render user message as Markdown and linkify bare URLs
- Share one resident OpenCode serve per workspace across sessions

## [0.0.12]

- Inherit the login-shell environment for provider commands
- Fix model traits across provider switches
- Keep branch change counts current and include untracked files
- Normalize SIGCHLD for provider children
- Fix Grok model discovery

## [0.0.11]

- Fix provider detection for CLIs installed through shell PATH managers such as
  nvm and fnm
- Show models registered by Pi extensions
- Fix the model picker closing when entering a space in search
- Fix duplicate transcript history and lost interaction mode when resuming ACP
  sessions

## [0.0.10]

- Fix crash in due to IME composition
- Fix typo

## [0.0.9]

- Add OpenCode Go support in usage popover
- Fix app icon
- Fix Cursor model detection

## [0.0.8]

- Initial release

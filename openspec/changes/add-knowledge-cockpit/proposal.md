## Why

YuuMira's Knowledge Cockpit is still a wanted follow-up: compile a local Markdown wiki, score health, show a graph, Ask Wiki. It is parked so PengPilot can first ship a small GPUI inbox, board, and multi-CLI squad without a vault indexer in the same change.

## What Changes

- After `add-inbox-work-squads` is archived, add a local vault cockpit as specified in `specs/knowledge-cockpit/spec.md`.
- Still no QMD/embeddings sidecar. Vault files stay on disk.

## Capabilities

### New Capabilities

- `knowledge-cockpit`: Local Markdown vault cockpit — pages, drafts, health, graph, ingest, and Ask Wiki.

### Modified Capabilities

- None until inbox/board/squads exist as main specs.

## Impact

- Must not start while `add-inbox-work-squads` is open.
- Must keep the same leanness rules: background scans, no render I/O, no extra JS runtime.

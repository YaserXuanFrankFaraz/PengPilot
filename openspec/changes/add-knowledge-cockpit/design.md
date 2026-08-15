## Context

See `proposal.md`. Behavior is already specified. This change exists so knowledge work does not inflate the inbox/board/squads ship.

## Goals / Non-Goals

**Goals:**
- Land the cockpit only after the GPUI board and squads are in use.

**Non-Goals:**
- Starting this change in parallel with `add-inbox-work-squads`.
- QMD, llama.cpp, or auto-import of `~/.craft-agent`.

## Decisions

### 1. Parked on purpose
Do not apply this change until `add-inbox-work-squads` is archived and the app still has no extra UI runtime.

### 2. Reuse sessions
Ask Wiki and ingest are ordinary PengPilot sessions, same as the parent design.

## Risks / Trade-offs

- [Vault snapshots grow RSS] → cap snapshot size; miss = not-ready; never hold page bodies for the whole vault.

## Migration Plan

Attach or init a vault only when the user asks. No automatic import.

## Open Questions

None until this change is unparked.

# Replace Kata with the local Markdown tracker

Status: done
Owner: codex
Branch: codex/harp-file-tracker

## What to build

Use Matt Pocock's local Markdown tracking convention for Harp. Retire the repository Kata binding and route agents and Harp skills to Git-managed tickets. Preserve existing history and unrelated work.

## Acceptance criteria

- [x] Repository guidance selects local Markdown and supersedes historical Kata instructions.
- [x] Remove the Kata repository binding without deleting external history.
- [x] Specs, tickets, triage states, evidence, and ownership have a documented file convention.
- [x] Focused checks pass and the change lands locally; no prerequisite rebuild or push.

## Blocked by

None.

## Verification

All five skill validators pass; local Markdown links resolve; Git includes the ticket despite scratch naming; staged whitespace checks pass. Local landing is recorded by the commit containing this ticket. Full build prerequisites are excluded by the user-authorized documentation gate.

## Comments

The user requested this tracker replacement immediately after authorizing focused documentation checks and local landing. Existing dirty primary-checkout files must remain untouched.

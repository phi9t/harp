# 04 check-runner: Check runner: timeout, streamed log, receipt v2

Type: feature
Status: claimed
Owner: collab-ops-04
Blocked by: 01
Claimed: 2026-10-12T00:39:05Z expires 2026-10-14T00:39:05Z

Triage: ready-for-agent

## What to build
check builds the Integration commit, runs the stored command with a timeout and a log that streams live, and records a Check receipt with argv, outcome (including timeout) and log hash. The landing lock is not held across the run.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 6-10). Use the project glossary terms.

## Declared paths
- `tools/collab/`

## Acceptance
- [ ] A hung check ends as a distinct timeout outcome.
- [ ] The log is readable while the check runs.
- [ ] The receipt records argv, outcome and log hash.
- [ ] Another landing is not blocked for the length of a check.
- [ ] A dirty workspace, changed HEAD or moved trunk still fails the check.

## Comments
- 2026-10-12T00:39:05Z collab-ops-04: claimed until 2026-10-14T00:39:05Z

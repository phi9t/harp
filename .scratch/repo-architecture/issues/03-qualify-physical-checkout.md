# 03: Qualify Git checkout before physical landing

Status: ready-for-agent
Owner: unassigned
Parent: [Repository architecture](../spec.md)

## What to build

Fix the demonstrated ignored-attributes mutation-before-refusal failure. Define the supported effective checkout contract, including destination-only ignored root/nested attributes, before any branch/index/worktree mutation. Retain exact-subject approval and stale-base refusal.

## Acceptance criteria

- [ ] The ignored .gitattributes CRLF reproducer refuses before changing branch, index or worktree bytes.
- [ ] Nested ignored attributes and other unsupported checkout conversion policies receive the same pre-mutation treatment.
- [ ] A successful landing synchronizes exact approved commit, index and raw tracked bytes.
- [ ] Crash recovery distinguishes an unapplied intent from an already applied exact landing without applying another candidate.

## Blocked by

None. Independent work may start without unrelated workload implementations.

## Verification

Use disposable real Git repositories and SQLite state. Snapshot HEAD, raw index and tracked bytes before refused cases. Run the existing exact fast-forward/no-squash and interrupted-landing regressions. No source-text assertions.

No implementation or qualification result is recorded yet.

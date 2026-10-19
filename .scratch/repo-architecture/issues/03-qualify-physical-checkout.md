# 03: Qualify Git checkout before physical landing

Status: done
Owner: Codex on codex/repo-architecture
Parent: [Repository architecture](../spec.md)

## What to build

Fix the demonstrated ignored-attributes mutation-before-refusal failure. Define the supported effective checkout contract, including destination-only ignored root/nested attributes, before any branch/index/worktree mutation. Retain exact-subject approval and stale-base refusal.

## Acceptance criteria

- [x] The ignored .gitattributes CRLF reproducer refuses before changing branch, index or worktree bytes.
- [x] Nested ignored attributes and other unsupported checkout conversion policies receive the same pre-mutation treatment.
- [x] A successful landing synchronizes exact approved commit, index and raw tracked bytes.
- [x] Crash recovery distinguishes an unapplied intent from an already applied exact landing without applying another candidate.

## Blocked by

None. Independent work may start without unrelated workload implementations.

## Verification

Use disposable real Git repositories and SQLite state. Snapshot HEAD, raw index and tracked bytes before refused cases. Run the existing exact fast-forward/no-squash and interrupted-landing regressions. No source-text assertions.

See the implementation progress below.

## Implementation progress

Historical checkpoints below are superseded by the completion record at the end.

Physical landing rejects destination-only .gitattributes, including ignored nested files, before branch/index/worktree mutation. Both real Git/SQLite refusal regressions and ordinary landing cases passed in the Linux lifecycle matrix. Final whole-candidate gate remains pending.

## Completion

Implemented and verified in body commit `c32c6f0f4a2de73e8067be0361121bb841ab4622`. The complete-candidate
changed-surface plan passes, including the bounded verification repairs recorded in
[the repository qualification](../../../evidence/provenance/repository-qualification.json).
Native runtime, benchmark and supervisor receipts bind the composed runtime bytes;
real Codex one/two/four-worker runs and all eight explicitly approved landings are
recorded separately in [Codex qualification](../../../evidence/provenance/codex-qualification.json).
Retained workloads and proof/PDF consumers passed their relevant checks.

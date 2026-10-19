# 02: Protect owner acceptance inputs and admitted change scope

Status: done
Owner: Codex on codex/repo-architecture
Parent: [Repository architecture](../spec.md)

## What to build

Fix the demonstrated candidate-controlled-checker bypass. Freeze owner oracle bytes and dependencies separately from the candidate, admit a machine-readable change policy, and reject out-of-policy candidates before successful verification or approval. Keep external black-box checks outside candidate-controlled interpreters when process trust requires it.

## Acceptance criteria

- [x] A candidate that rewrites only its checker and leaves the wrong answer cannot receive an acceptable verification proof.
- [x] Candidate imports, build configuration and generated code cannot shadow or replace a claimed independent oracle.
- [x] The proof binds candidate identity, admitted change policy, oracle identity and execution evidence.
- [x] Legitimate owner-approved changes to tests/configuration remain possible through the admitted policy. Missing or ambiguous policy fails admission with an actionable error.

## Blocked by

None. Independent work may start without unrelated workload implementations.

## Verification

Convert the audit public-CLI/namespace reproducer into a durable integration regression. Check the original bad candidate and a correct candidate against an externally controlled oracle, plus an allowed test-change task and a forbidden-path change. Do not mock the same checker logic under test.

See the implementation progress below.

## Implementation progress

Historical checkpoints below are superseded by the completion record at the end.

Owner oracle bytes freeze separately from worker inputs; checks require isolated pinned Python or qualified system tools. Capture and independent verification enforce allowed_paths. The Linux positive frozen-oracle case and both checker-replacement/scope counterexamples pass after the private mount repair. The preceding full native matrix passed 201 of 202, with that exact positive case subsequently repaired and rerun. Final source-bound integrated receipt remains pending.

## Completion

Implemented and verified in body commit `c32c6f0f4a2de73e8067be0361121bb841ab4622`. The complete-candidate
changed-surface plan passes, including the bounded verification repairs recorded in
[the repository qualification](../../../evidence/provenance/repository-qualification.json).
Native runtime, benchmark and supervisor receipts bind the composed runtime bytes;
real Codex one/two/four-worker runs and all eight explicitly approved landings are
recorded separately in [Codex qualification](../../../evidence/provenance/codex-qualification.json).
Retained workloads and proof/PDF consumers passed their relevant checks.

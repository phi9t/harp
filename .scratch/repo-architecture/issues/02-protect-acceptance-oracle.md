# 02: Protect owner acceptance inputs and admitted change scope

Status: in-progress
Owner: Codex on codex/repo-architecture
Parent: [Repository architecture](../spec.md)

## What to build

Fix the demonstrated candidate-controlled-checker bypass. Freeze owner oracle bytes and dependencies separately from the candidate, admit a machine-readable change policy, and reject out-of-policy candidates before successful verification or approval. Keep external black-box checks outside candidate-controlled interpreters when process trust requires it.

## Acceptance criteria

- [ ] A candidate that rewrites only its checker and leaves the wrong answer cannot receive an acceptable verification proof.
- [ ] Candidate imports, build configuration and generated code cannot shadow or replace a claimed independent oracle.
- [ ] The proof binds candidate identity, admitted change policy, oracle identity and execution evidence.
- [ ] Legitimate owner-approved changes to tests/configuration remain possible through the admitted policy. Missing or ambiguous policy fails admission with an actionable error.

## Blocked by

None. Independent work may start without unrelated workload implementations.

## Verification

Convert the audit public-CLI/namespace reproducer into a durable integration regression. Check the original bad candidate and a correct candidate against an externally controlled oracle, plus an allowed test-change task and a forbidden-path change. Do not mock the same checker logic under test.

See the implementation progress below.

## Implementation progress

Owner oracle bytes freeze separately from worker inputs; checks require isolated pinned Python or qualified system tools. Capture and independent verification enforce allowed_paths. The Linux positive frozen-oracle case and both checker-replacement/scope counterexamples pass after the private mount repair. The preceding full native matrix passed 201 of 202, with that exact positive case subsequently repaired and rerun. Final source-bound integrated receipt remains pending.

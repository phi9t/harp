# 02: Protect owner acceptance inputs and admitted change scope

Status: ready-for-agent
Owner: unassigned
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

No implementation or qualification result is recorded yet.

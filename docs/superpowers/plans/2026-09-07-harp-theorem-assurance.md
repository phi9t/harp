# Harp theorem assurance implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development. Complete specification review, then quality review, for each task.

**Goal:** Make the terminal statement and every sensitive step inspectable without changing the terminal proof.

**Architecture:** An unindexed Lean audit module expands the published statement. A canonical manuscript explains the seven audit checks. Existing registration and publication owners integrate the manuscript.

**Tech stack:** Lean 4.32.1, pinned Mathlib, Rust publication contracts, Markdown, Atlas, PDF builder.

Baseline: 2e5d57da339d83b90bb631e0b1939958bd7369a6.
Workspace: codex/harp-mathematics-spec. User authorized implementation on 2026-09-07. No push, merge, deployment, or external outreach.
Specification: ../specs/2026-09-07-harp-theorem-assurance-design.md.

## Task 1: Expanded statement and semantic checks

Files: create formalization/lean/CrouzeixTextbook/HarpStatementAudit.lean
and HarpStatementAuditTests.lean. Root import is integrated in Task 3.

- [x] Write a compile client requesting an expanded proposition and equivalence, then run it against the baseline and record the missing-declaration failure.
- [x] Define textbookBound with ordinary Euclidean operator evaluation and an explicit unit-vector quadratic-form set. Do not define it using MainTheoremStatement.
- [x] Prove textbookBound_iff_mainTheoremStatement, nonemptiness and boundedness of the polynomial-modulus image. Use compactness only with its actual provider.
- [x] Add concrete norm and boundary checks for zero, constant, one-dimensional, nonzero nilpotent, and zero-modulus cases.
- [x] Add compile-rejected correspondence fixtures for a wrong norm, missing unit condition, reordered quantifiers, and an added terminal premise. Rejection concerns exact type correspondence, not universal falsity.
- [x] Compile both modules with lake env lean from formalization/lean. Record standard axioms and check provider closure for terminal shortcuts.
- [x] Obtain separate specification and quality reviews. Fix blocking findings before Task 2.

Expected proposition shape, with all instance arguments explicit in the implementation:

```lean
∀ (A : Matrix n n ℂ) (p : Polynomial ℂ),
  ‖Matrix.toEuclideanCLM (Polynomial.aeval A p)‖ ≤
    2 * sSup ((fun z : ℂ => ‖p.eval z‖) ''
      {z | ∃ x : EuclideanSpace ℂ n, ‖x‖ = 1 ∧
        inner ℂ x (Matrix.toEuclideanCLM A x) = z})
```

## Task 2: Canonical audit manuscript

Scheduling clarification: the parent may draft the source-backed exposition
while Task 1 compiles, since it concerns the existing proof. Keep acceptance
pending and reconcile every new declaration link after Task 1 review. This
does not authorize a second concurrent implementation subagent.

Files: knowledge/crouzeix_textbook/harp_mathematical_audit.md;
docs/workstream/harp-mathematics/theorem-assurance-review.md.

- [x] Inspect Euclidean, Definitions, Statements, Harp construction, recurrence, perturbation, MainTheorem and Consequences owners.
- [x] Write the seven source-backed checks from the accepted spec with hypotheses, calculations, declaration links and verdicts. Include all five boundary examples.
- [x] Distinguish scalar mass from matrix mass 2I, common core from horizon witnesses, displacement replacement sign, and horizon/matrix/domain limits.
- [x] Explain shared LS support honestly. Do not claim external review or a fourth proof.
- [x] Record compile commands, findings and repairs in the execution record, with local non-hermetic labels.
- [x] Obtain mathematical/specification review, then exposition/correspondence quality review.

## Task 3: Integration and acceptance

Files: formalization/lean/CrouzeixTextbook.lean, canonical index and Chapter 36;
existing managed-document registry, corpus/export and tests. Parent owns shared files.

- [x] Add a failing managed-document resolution test before registering the manuscript.
- [x] Register one nonchapter identity, link index and Chapter 36, and import audit tests.
- [x] Generate receipts using the maintained compiler and publisher, never hand-enter hashes.
- [x] Run focused registration and Lean checks. Freeze all program slices before the full gate.
- [x] Run mise run verify, rebuild and visually inspect full PDF plus existing standalone editions, and inspect actual desktop/mobile reader math and Lean links.
- [x] Record locally-audited only after both reviews and formal checks pass. External human review remains pending.
- [x] Commit explicit reviewed path groups only after the full gate. Do not push or land.

## Prerequisite record

Warm cache and pinned toolchain preflight passed; baseline route build passed.
Foundations issues 8mq3 and m4qy remain open. This independent audit does not remove their blockers.

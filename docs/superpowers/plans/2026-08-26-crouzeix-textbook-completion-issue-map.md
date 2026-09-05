# Crouzeix textbook completion issue map implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert the approved completion PRD and seven wave plans into vertical Kata issues that finish the textbook without losing proof, Lean, source, exercise, or publication correspondence.

**Architecture:** Keep the detailed mathematical steps in the existing wave plans. This document assigns one tracker-ready slice to each bounded proof mechanism, chapter, review, or publication gate. Slices own canonical prose, local Lean, contracts, exercises, focused tests, and their share of publication output together. Shared generated files move through one serial integration issue per wave.

**Tech Stack:** Markdown, Lean 4.32.1, Mathlib 4.32.1, Rust publication tooling, JSON v2 contracts, Atlas, mise, Kata 0.14.3.

---

## Approved source

The requirements are in
`docs/superpowers/specs/2026-08-26-crouzeix-textbook-completion-prd.md`.
The detailed execution steps remain in:

- `docs/superpowers/plans/2026-08-23-crouzeix-textbook-wave-1-chapter-33.md`
- `docs/superpowers/plans/2026-08-23-crouzeix-textbook-wave-2-jin.md`
- `docs/superpowers/plans/2026-08-23-crouzeix-textbook-wave-3-lorist-schwenninger.md`
- `docs/superpowers/plans/2026-08-23-crouzeix-textbook-wave-4-common-machinery.md`
- `docs/superpowers/plans/2026-08-23-crouzeix-textbook-wave-5-analysis-operator.md`
- `docs/superpowers/plans/2026-08-23-crouzeix-textbook-wave-6-foundations.md`

Wave 0 publication work is complete and provides the contract and receipt
machinery used by every issue below. This map starts at CFT-33-003.

## File responsibilities

Each chapter issue changes one canonical Markdown chapter, its matching
`formalization/lean/CrouzeixTextbook/` chapter module, and its six theorem and
exercise rows under `content/crouzeix_textbook/`. It may update source and
claim ledgers when the slice introduces a source-backed statement.

Chapter issues do not publish shared generated files while another issue in
the wave is active. The wave integration issue owns:

- generated textbook ledgers;
- `formalization/lean/CrouzeixTextbook/Correspondence.lean`;
- `atlas/src/content/generated/corpus.json`;
- `atlas/dist/harp-atlas.html` and its receipt;
- `docs/import-receipt.md`.

The integration issue regenerates these files from settled canonical inputs.
It refreshes `docs/import-receipt.md` last.

## Reader stories

- **R1, foundations.** An undergraduate-trained ML researcher can enter at
  Chapter 1 and reach the proof chapters without an external prerequisite
  text.
- **R2, source-derived proofs.** The reader can reconstruct Jin and
  Lorist--Schwenninger from displayed equations and cited prerequisites.
- **R3, Harp route.** The reader can explain Harp's derived finite-horizon
  proof and its relationship to the source-derived routes.
- **R4, Lean audit.** The reader can follow every formal theorem and exercise
  to a compiler-validated Lean declaration, provider, type, and axiom receipt.
- **R5, claim boundaries.** The reader can distinguish proof, analogy,
  numerical evidence, source attribution, local reproduction, and external
  review status.

## Common issue acceptance contract

Every implementation issue must satisfy all applicable points below in
addition to the slice-specific steps in its source wave plan.

- [ ] The issue delivers a complete theorem or chapter path through prose,
  Lean, contracts, exercises, and focused tests.
- [ ] The prose contains motivation, exact statements, a hypothesis ledger,
  proof details, a worked instance, a boundary case, history, and a bounded ML
  analogy.
- [ ] Formal theorem cards name the public Lean declaration, substantive
  provider, readable type map, source link, fingerprint, dependencies, axioms,
  and verification target.
- [ ] Every formal exercise has a distinct theorem statement. Every prose-only
  exercise has a complete written solution and honest Lean status.
- [ ] Focused test filters select at least one test and pass.
- [ ] The applicable provider target and `lean-crouzeix-textbook` pass.
- [ ] The issue stages explicit paths and leaves unrelated work untouched.
- [ ] Shared generated outputs change only in the wave integration issue.

## Proposed issue catalog

The proposed granularity is intentionally uneven. Load-bearing terminal
algebra gets one issue per mechanism. Foundation and analysis work gets one
issue per six-card chapter. This keeps a chapter internally coherent without
turning a difficult proof into a month-long issue.

### Wave 1, Chapter 33

1. **CTB-001: Derive LS source Equation (3).** Blocked by: none. Stories: R2,
   R4, R5. Source: Wave 1 Task 3.
2. **CTB-002: Prove the LS scalar endpoint.** Blocked by: CTB-001. Stories:
   R2, R4, R5. Source: Wave 1 Task 4, CFT-33-004 and CFT-33-005.
3. **CTB-003: Assemble the LS perturbation workshop.** Blocked by: CTB-002.
   Stories: R2, R4, R5. Source: Wave 1 Task 5, CFT-33-006 and chapter context.
4. **CTB-004: Review and publish Chapter 33.** Blocked by: CTB-003. Stories:
   R2, R4, R5. Source: Wave 1 release steps and the PRD review gates.

### Wave 2, Jin

5. **CTB-005: Freeze the Jin completion contract.** Blocked by: CTB-004.
   Stories: R2, R4, R5. Source: Wave 2 Task 1.
6. **CTB-006: Reconstruct the Chapter 30 completion interface.** Blocked by:
   CTB-005. Stories: R2, R4. Source: Wave 2 Task 2.
7. **CTB-007: Expand the Chapter 31 cancellation algebra.** Blocked by:
   CTB-006. Stories: R2, R4, R5. Source: Wave 2 Task 3.
8. **CTB-008: Derive the Chapter 32 norm and limit endpoint.** Blocked by:
   CTB-007. Stories: R2, R4, R5. Source: Wave 2 Task 4.
9. **CTB-009: Integrate Jin history, ML transfer, and route review.** Blocked
   by: CTB-008. Stories: R2, R5. Source: Wave 2 Task 5.
10. **CTB-010: Review and publish the Jin wave.** Blocked by: CTB-009.
    Stories: R2, R4, R5. Source: Wave 2 Task 6.

### Wave 3, LS realization and three-route comparison

11. **CTB-011: Freeze the LS and three-route comparison contract.** Blocked
    by: CTB-010. Stories: R2, R3, R4, R5. Source: Wave 3 Task 1.
12. **CTB-012: Construct the Chapter 34 boundary dilation data.** Blocked by:
    CTB-011. Stories: R2, R4. Source: Wave 3 Task 2.
13. **CTB-013: Prove Chapter 34 moment compression.** Blocked by: CTB-012.
    Stories: R2, R4. Source: Wave 3 Task 3.
14. **CTB-014: Prove Chapter 34 multiplier and realization bounds.** Blocked
    by: CTB-013. Stories: R2, R4, R5. Source: Wave 3 Task 4.
15. **CTB-015: Derive the Chapter 35 consequences.** Blocked by: CTB-014 and
    CTB-010. Stories: R2, R3, R4. Source: Wave 3 Task 5.
16. **CTB-016: Build the Chapter 35 three-route comparison.** Blocked by:
    CTB-015. Stories: R2, R3, R4, R5. Source: Wave 3 Task 6 as reconciled by
    the completion PRD.
17. **CTB-017: Review and publish the three-route wave.** Blocked by: CTB-016.
    Stories: R2, R3, R4, R5. Source: Wave 3 Task 7.

### Wave 4, common machinery

18. **CTB-018: Freeze the common-trunk contract.** Blocked by: CTB-017.
    Stories: R2, R3, R4. Source: Wave 4 Task 1.
19. **CTB-019: Deepen Chapter 25 convex boundaries and Cauchy layers.**
    Blocked by: CTB-018. Stories: R1, R2, R4. Source: Wave 4 Task 2.
20. **CTB-020: Deepen Chapter 26 double-layer operators.** Blocked by:
    CTB-019. Stories: R1, R2, R4. Source: Wave 4 Task 3.
21. **CTB-021: Explain the Chapter 27 `1 + sqrt 2` obstruction.** Blocked by:
    CTB-020. Stories: R1, R2, R4, R5. Source: Wave 4 Task 4.
22. **CTB-022: Build the Chapter 28 complete power family.** Blocked by:
    CTB-021. Stories: R1, R2, R3, R4. Source: Wave 4 Task 5.
23. **CTB-023: Complete Chapter 29 normalization and sharpness.** Blocked by:
    CTB-022. Stories: R1, R2, R3, R4, R5. Source: Wave 4 Task 6.
24. **CTB-024: Review and publish the common trunk.** Blocked by: CTB-023.
    Stories: R1, R2, R3, R4, R5. Source: Wave 4 Task 7.

### Wave 5, analysis and operator theory

25. **CTB-025: Freeze the analysis and operator contract.** Blocked by:
    CTB-024. Stories: R1, R4. Source: Wave 5 Task 1.
26. **CTB-026: Complete Chapter 13 metric and normed foundations.** Blocked
    by: CTB-025. Stories: R1, R4. Source: Wave 5 Task 2.
27. **CTB-027: Complete Chapter 14 operator series.** Blocked by: CTB-026.
    Stories: R1, R2, R4. Source: Wave 5 Task 3.
28. **CTB-028: Complete Chapter 15 complex differentiability.** Blocked by:
    CTB-027. Stories: R1, R2, R4. Source: Wave 5 Task 4.
29. **CTB-029: Complete Chapter 16 Cauchy consequences.** Blocked by: CTB-028.
    Stories: R1, R2, R4. Source: Wave 5 Task 5.
30. **CTB-030: Complete Chapter 17 operator functional calculus.** Blocked
    by: CTB-029. Stories: R1, R2, R4. Source: Wave 5 Task 6.
31. **CTB-031: Complete Chapter 18 positive-real analysis.** Blocked by:
    CTB-030. Stories: R1, R2, R4. Source: Wave 5 Task 7.
32. **CTB-032: Complete Chapter 19 normality and nonnormality.** Blocked by:
    CTB-031. Stories: R1, R2, R5. Source: Wave 5 Task 8.
33. **CTB-033: Complete Chapter 20 numerical ranges.** Blocked by: CTB-032.
    Stories: R1, R2, R3, R4. Source: Wave 5 Task 9.
34. **CTB-034: Complete Chapter 21 spectral sets.** Blocked by: CTB-033.
    Stories: R1, R2, R3, R4, R5. Source: Wave 5 Task 10.
35. **CTB-035: Complete Chapter 22 positive maps.** Blocked by: CTB-034.
    Stories: R1, R2, R4, R5. Source: Wave 5 Task 11.
36. **CTB-036: Complete Chapter 23 compression and dilation.** Blocked by:
    CTB-035. Stories: R1, R2, R3, R4. Source: Wave 5 Task 12.
37. **CTB-037: Complete Chapter 24 Gramians and matrix order.** Blocked by:
    CTB-036. Stories: R1, R2, R3, R4. Source: Wave 5 Task 13.
38. **CTB-038: Integrate analysis history, ML examples, and notation.**
    Blocked by: CTB-037. Stories: R1, R5. Source: Wave 5 Task 14.
39. **CTB-039: Review and publish the analysis and operator wave.** Blocked
    by: CTB-038. Stories: R1, R2, R3, R4, R5. Source: Wave 5 Task 15.

### Wave 6, Lax-style foundations

40. **CTB-040: Freeze the foundations contract.** Blocked by: CTB-039.
    Stories: R1, R4. Source: Wave 6 Task 1.
41. **CTB-041: Complete Chapter 1 objects and representations.** Blocked by:
    CTB-040. Stories: R1, R4, R5. Source: Wave 6 Task 2.
42. **CTB-042: Complete Chapter 2 vector spaces and quotients.** Blocked by:
    CTB-041. Stories: R1, R4. Source: Wave 6 Task 3.
43. **CTB-043: Complete Chapter 3 kernels, images, and exact structure.**
    Blocked by: CTB-042. Stories: R1, R4. Source: Wave 6 Task 4.
44. **CTB-044: Complete Chapter 4 coordinates and duality.** Blocked by:
    CTB-043. Stories: R1, R4. Source: Wave 6 Task 5.
45. **CTB-045: Complete Chapter 5 determinant, trace, and exterior algebra.**
    Blocked by: CTB-044. Stories: R1, R4. Source: Wave 6 Task 6.
46. **CTB-046: Complete Chapter 6 eigenvalues and polynomial algebra.**
    Blocked by: CTB-045. Stories: R1, R2, R4. Source: Wave 6 Task 7.
47. **CTB-047: Review the Part I checkpoint.** Blocked by: CTB-046. Stories:
    R1, R4. Source: Wave 6 Task 8.
48. **CTB-048: Complete Chapter 7 inner-product geometry.** Blocked by:
    CTB-047. Stories: R1, R2, R4. Source: Wave 6 Task 9.
49. **CTB-049: Complete Chapter 8 positivity and Gram geometry.** Blocked by:
    CTB-048. Stories: R1, R2, R4. Source: Wave 6 Task 10.
50. **CTB-050: Complete Chapter 9 operator norms and singular values.**
    Blocked by: CTB-049. Stories: R1, R2, R3, R4. Source: Wave 6 Task 11.
51. **CTB-051: Complete Chapter 10 multilinear maps and tensors.** Blocked
    by: CTB-050. Stories: R1, R4. Source: Wave 6 Task 12.
52. **CTB-052: Complete Chapter 11 differentiation as linear
    approximation.** Blocked by: CTB-051. Stories: R1, R5. Source: Wave 6
    Task 13.
53. **CTB-053: Complete Chapter 12 differential forms and Stokes.** Blocked
    by: CTB-052. Stories: R1, R4, R5. Source: Wave 6 Task 14.
54. **CTB-054: Integrate foundation history, ML examples, and notation.**
    Blocked by: CTB-053. Stories: R1, R5. Source: Wave 6 Task 15.
55. **CTB-055: Review and publish the foundations wave.** Blocked by: CTB-054.
    Stories: R1, R4, R5. Source: Wave 6 Task 16, excluding final program
    declaration until Wave 7.

### Wave 7, whole-book integration

56. **CTB-056: Normalize notation and build reader routes.** Blocked by:
    CTB-055. Stories: R1, R2, R3. Source: completion PRD Wave 7.
57. **CTB-057: Audit history, source status, and ML boundaries.** Blocked by:
    CTB-055. Stories: R2, R3, R5. Source: completion PRD review gates.
58. **CTB-058: Audit all theorem and exercise correspondence.** Blocked by:
    CTB-055. Stories: R4, R5. Source: completion PRD whole-book criteria.
59. **CTB-059: Run independent mathematical and formal reviews.** Blocked by:
    CTB-056, CTB-057, and CTB-058. Stories: R1, R2, R3, R4, R5. Source:
    completion PRD review gates.
60. **CTB-060: Publish and verify the complete textbook.** Blocked by:
    CTB-059. Stories: R1, R2, R3, R4, R5. Source: completion PRD whole-book
    acceptance criteria.

## Dependency policy

The issue graph follows proof-risk authoring order. It is mostly serial on
purpose. Canonical chapter work may run in parallel only after an issue owner
proves that the write sets and mathematical prerequisites do not overlap.

CTB-056, CTB-057, and CTB-058 may run concurrently after CTB-055 because they
own different audits. They must not publish generated files. CTB-059 integrates
their findings and CTB-060 alone refreshes the final generated set and import
receipt.

## Tracker publication plan

Kata is the Harp intent ledger. The executable is
`/opt/homebrew/bin/kata` at version 0.14.3. Do not publish issues until the
owner approves this granularity and dependency graph.

After approval:

- [x] **Step 1: Inspect existing issues and labels**

Run:

```sh
/opt/homebrew/bin/kata search "Crouzeix textbook" --workspace "$PWD"
/opt/homebrew/bin/kata labels --workspace "$PWD"
```

Expected: existing related work and the project's current label vocabulary.
Reuse matching open issues instead of creating duplicates.

- [x] **Step 2: Reconcile and publish the approved catalog**

Use the `to-issues` issue template. Publish in numeric order so every blocker
reference uses a real Kata ID. Give each new issue the idempotency key
`crouzeix-textbook-completion/CTB-NNN` with its numeric suffix, metadata
`program=crouzeix-textbook-completion`, and only labels returned by Step 1.
Omit the blocker flag for CTB-001. Do not create a native Kata parent because
Kata 0.14.3 treats a parent as a blocking predecessor.

- [x] **Step 3: Verify and record the published graph**

Run:

```sh
/opt/homebrew/bin/kata search "crouzeix-textbook-completion" \
  --workspace "$PWD" --format json
/opt/homebrew/bin/kata ready --workspace "$PWD" --format json
```

Expected: one issue for every approved unmatched slice, blocker references to
real Kata IDs, and only CTB-001 or an approved existing predecessor ready at
the start. Add the CTB-to-Kata mapping to this plan and commit only the plan
reconciliation and mapping. Do not close or modify a pre-existing parent
issue.

## Kata mapping

The approved catalog was published on 2026-08-26. Kata reports 60 open issues,
with CTB-001 as the only initially ready issue.

| Catalog | Kata | Catalog | Kata | Catalog | Kata |
| --- | --- | --- | --- | --- | --- |
| CTB-001 | `0sw5` | CTB-021 | `j3zc` | CTB-041 | `m4qy` |
| CTB-002 | `gmq4` | CTB-022 | `f7j5` | CTB-042 | `x5fp` |
| CTB-003 | `b0dg` | CTB-023 | `0hhs` | CTB-043 | `6bsx` |
| CTB-004 | `29sz` | CTB-024 | `yp03` | CTB-044 | `q8h9` |
| CTB-005 | `nrqr` | CTB-025 | `3261` | CTB-045 | `a2ry` |
| CTB-006 | `zfsv` | CTB-026 | `fx6b` | CTB-046 | `9hxf` |
| CTB-007 | `3b5x` | CTB-027 | `2700` | CTB-047 | `1t51` |
| CTB-008 | `s6rf` | CTB-028 | `hg37` | CTB-048 | `3ya3` |
| CTB-009 | `w51e` | CTB-029 | `92zd` | CTB-049 | `tdga` |
| CTB-010 | `3ggn` | CTB-030 | `q81q` | CTB-050 | `wstr` |
| CTB-011 | `8tcp` | CTB-031 | `dvm6` | CTB-051 | `0wc6` |
| CTB-012 | `we5g` | CTB-032 | `1r2p` | CTB-052 | `raf4` |
| CTB-013 | `q0qc` | CTB-033 | `efcd` | CTB-053 | `r60f` |
| CTB-014 | `ped7` | CTB-034 | `gkgr` | CTB-054 | `xj38` |
| CTB-015 | `234s` | CTB-035 | `17d3` | CTB-055 | `dx0r` |
| CTB-016 | `m00v` | CTB-036 | `0f59` | CTB-056 | `csp4` |
| CTB-017 | `qmsc` | CTB-037 | `nz6f` | CTB-057 | `6j3f` |
| CTB-018 | `bwfs` | CTB-038 | `3tk3` | CTB-058 | `64xv` |
| CTB-019 | `q2rg` | CTB-039 | `fsg8` | CTB-059 | `57rx` |
| CTB-020 | `t0rz` | CTB-040 | `8mq3` | CTB-060 | `00zs` |

## Plan self-review

- Spec coverage: CTB-001 through CTB-055 cover every unfinished mathematical
  wave. CTB-056 through CTB-060 cover the PRD's integration, audit, review, and
  release requirements.
- Verticality: every implementation slice owns prose, Lean, contracts,
  exercises, and focused tests. Integration issues own shared generated files.
- Provider truth: Wave 3 compares Jin, LS, and Harp and records Harp as
  derived. Compiler receipts determine imports.
- Exercise truth: formal exercises require distinct theorem statements;
  prose-only questions keep written solutions and honest Lean status.
- Landability: each issue has one predecessor chain and one green focused
  gate. Review and generated publication happen before the next wave starts.

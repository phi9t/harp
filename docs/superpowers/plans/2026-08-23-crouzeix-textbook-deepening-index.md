# Crouzeix Textbook Deepening Program Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the existing 35-chapter Crouzeix proof-route atlas into a self-contained, reconstructible proof textbook with exact Lean 4 correspondence for a frontier mathematical-ML audience.

**Architecture:** Deliver one publication-integrity foundation followed by six content waves. Each content wave lands vertical theorem slices: prose, historical context, ML transfer, exercises, Lean proofs, dependency receipts, generated ledgers, and verification advance together.

**Tech Stack:** Markdown and Obsidian wikilinks, Rust 1.92 and Serde, Lean 4.32.1 with Mathlib 4.32.1, JSON contracts, TypeScript/Vitest Atlas, mise, Git worktrees.

---

## Required reading and invariants

- Read `docs/superpowers/specs/2026-08-23-crouzeix-textbook-deepening-design.md` before executing any wave.
- Work only in an isolated `codex/` feature worktree. Do not mutate the primary checkout.
- Preserve all existing CFT theorem IDs, exercise IDs, and public Lean declaration names.
- Keep canonical prose under `knowledge/crouzeix_textbook/`, contracts under `content/crouzeix_textbook/`, and Lean under `formalization/lean/CrouzeixTextbook/`.
- Keep the book text-only. Do not add required diagrams, images, notebooks, or interactive assets.
- Never run `lake update`, `lake --try-cache exe cache get Mathlib`, or `mise run lean-cache`. Missing warm-cache state is a blocked precondition.
- Do not add `sorry`, `admit`, project axioms, `opaque`, `unsafe`, `native_decide`, or `implemented_by` to the textbook proof surface.
- A compile proves type correctness only. It does not by itself establish prose completeness, exact correspondence, source fidelity, historical priority, ML applicability, or editorial quality.
- Every commit stages explicit paths; never use `git add .` or `git add -A`.
- Refresh `docs/import-receipt.md` only after every other tracked byte in the wave is settled.

## Execution order

| Wave | Plan | Exit state |
|---:|---|---|
| 0 | `2026-08-23-crouzeix-textbook-wave-0-publication-integrity.md` | Version-two contracts and the Rust publisher truthfully expose incomplete correspondence. |
| 1 | `2026-08-23-crouzeix-textbook-wave-1-chapter-33.md` | Chapter 33 is the reference reconstructible theorem workshop. |
| 2 | `2026-08-23-crouzeix-textbook-wave-2-jin.md` | Chapters 30–32 reconstruct the Jin route end to end. |
| 3 | `2026-08-23-crouzeix-textbook-wave-3-lorist-schwenninger.md` | Chapters 34–35 finish the independent LS branch and comparison surface. |
| 4 | `2026-08-23-crouzeix-textbook-wave-4-common-machinery.md` | Chapters 25–29 supply the common analytic and sharpness trunk. |
| 5 | `2026-08-23-crouzeix-textbook-wave-5-analysis-operator.md` | Chapters 13–24 supply analysis, functional calculus, and operator theory. |
| 6 | `2026-08-23-crouzeix-textbook-wave-6-foundations.md` | Chapters 1–12 complete the Lax-style linear algebra and geometry foundation. |

The order is deliberate. Wave 1 pressure-tests the publisher on the deepest
single proof before either terminal route is rewritten. Later waves may refine
the schemas only by first adding a failing Wave 0 regression test and updating
the design specification.

## The vertical-slice completion rule

A CFT item is complete only when all of the following are true:

- its exact prose anchor resolves once;
- its theorem card contains purpose, exact statement, hypothesis ledger,
  roadmap, reconstructible proof, and a boundary example;
- its historical context has an ADR-0001 class, exact source, locator, review
  status, and reproduction status;
- its ML analogy names the mapping, exact transfer, non-transfer boundary, and
  a concrete diagnostic;
- its Lean mode is truthful and the declaration type, source position,
  fingerprint, dependencies, and axioms match a compiled receipt;
- every formal exercise has a distinct compiled solution theorem;
- its pedagogical prerequisites and derived kernel dependencies are valid;
- generated ledgers, corpus JSON, and Atlas output are current;
- focused Rust, Lean, and presentation tests pass.

## Standard per-chapter RED/GREEN loop

- [ ] Add mutation tests that fail for the chapter's current generic anchor,
  alias-only proof, reused exercise solution, missing theorem-card field, or
  incorrect dependency edge.
- [ ] Change the six existing CFT rows to exact anchors and truthful modes;
  never manufacture new IDs merely to improve counts.
- [ ] Write the mathematical statements and proofs before writing summaries,
  then add labeled context and exercises around those proofs.
- [ ] Add or expose exact Lean declarations and six distinct exercise solution
  declarations in `CrouzeixTextbook.Exercises.ChapterNN`.
- [ ] Run the chapter receipt check, `mise run lean-crouzeix-textbook`, and the
  applicable provider-isolated target.
- [ ] Publish ledgers, corpus, and Atlas together; run the wave gate.
- [ ] Review the rendered text as a frontier ML researcher with uneven proof
  recall, then commit prose, formalization, and generated outputs in explicit
  coherent groups.

## Program completion gate

After Wave 6, run:

```sh
PATH=/opt/homebrew/bin:$PATH mise run verify
git diff --check
git status --short
```

Expected: the full repository gate passes, the worktree contains only intended
changes or is clean after the final commit, all 35 chapters report
`prose_proof_status=reconstructible` and
`lean_correspondence_status=exact`, and neither terminal provider's kernel
closure contains the other provider.

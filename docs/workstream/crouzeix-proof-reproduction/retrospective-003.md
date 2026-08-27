# Phase-8 retrospective

## Outcome

**OBSERVED**

The proof program through CPFR-091 reached the bounded local end state recorded
in the durable repository artifacts. [The execution ledger](execution-ledger-003.tsv) and
[the tracker](tracker.org) record Jin, Lorist--Schwenninger, and Harp as
`complete-local` by the end of CPFR-090, with route-specific landings at
`66045b6fd0be878bf76ad0171387e53619bf09f0` for CPFR-085,
`d1749570825afc277fd96ff6c79b532449afbd16` for CPFR-086,
`d9031068a883731cdab7b4eb80c744eda1912c07` for CPFR-087,
`c09a0dc2a9fb6db19e0073d87e333f8fe86d92e7` for CPFR-088,
and `573d92909972f8171e5c35ae3179a1d0181982e9` for CPFR-089. Program
verification then landed at `98af685eabda905076879c15920f2fb7d526b632` and
closed at `a7d7654dc9db7ed4df42809427a7437d707a4e8f`; worktree audit evidence
landed at `87f5130b3415ffc628217f33b24294c9b1c36b89` and closed at
`d39b7b43399f1d90a58dc7567d4760a863416c25`.

[The program verification record](program-verification-003.json) fixes the
claim ceiling. Jin and Lorist--Schwenninger are `complete-local`.
Harp is also `complete-local`, but it is a derived route that reuses exactly
eleven approved lower-level Lorist--Schwenninger modules and is not
mathematically independent. The same record explicitly limits the result: this
phase does not establish peer review, publication, author endorsement, or full
boundedness.

**INTERPRETATION**

Phase 8 completed a local certification program, not a broader mathematical or
publication program. The correct summary is narrower than "the proof is done"
and broader than "three theorems compiled": the repository now has
route-isolated receipts, reviews, a required six-row local bundle, reader
reconciliation, program verification, and a bounded worktree cleanup, all tied
to committed evidence.

## Evidence corpus

**OBSERVED**

This note uses committed repository evidence as authority:

- [execution-ledger-003.tsv](execution-ledger-003.tsv) for monotone phase-state
  transitions and landing commits.
- [tracker.org](tracker.org) for phase scope, verification evidence, repair
  history, and closeout notes.
- [program-verification-003.json](program-verification-003.json) for the final
  program claim boundary, gate measurements, cache digest, bundle digest, and
  reviewer identities.
- [worktree-inventory-002.md](worktree-inventory-002.md) for the cleanup
  boundary, cache-provider/consumer ordering, preserved dirty state, and
  removal results.
- [retrospective-002.md](retrospective-002.md),
  [cpfr-086-retrospective.md](cpfr-086-retrospective.md), and
  [cpfr-087-retrospective.md](cpfr-087-retrospective.md) for earlier
  phase-local process observations and already-distilled lessons.
- [evidence/crouzeix_conjecture/routes/jin/receipt.json](../../../evidence/crouzeix_conjecture/routes/jin/receipt.json),
  [evidence/crouzeix_conjecture/routes/lorist-schwenninger/receipt.json](../../../evidence/crouzeix_conjecture/routes/lorist-schwenninger/receipt.json),
  [evidence/crouzeix_conjecture/routes/harp/receipt.json](../../../evidence/crouzeix_conjecture/routes/harp/receipt.json),
  [evidence/crouzeix_conjecture/reviews/jin.json](../../../evidence/crouzeix_conjecture/reviews/jin.json),
  [evidence/crouzeix_conjecture/reviews/lorist-schwenninger.json](../../../evidence/crouzeix_conjecture/reviews/lorist-schwenninger.json),
  [evidence/crouzeix_conjecture/reviews/harp.json](../../../evidence/crouzeix_conjecture/reviews/harp.json),
  and
  [evidence/crouzeix_conjecture/local_formalization/manifest.tsv](../../../evidence/crouzeix_conjecture/local_formalization/manifest.tsv)
  for phase outputs that bind the route and bundle claims.

[retrospective-002.md](retrospective-002.md) and
[agent-trace-index.md](agent-trace-index.md) remain process observation only.
They are useful for workflow reconstruction, but they are not durable proof
authority and are not used here to widen any theorem, route, or publication
claim.

**INTERPRETATION**

The right evidence hierarchy for this retrospective is ledger, tracker,
receipts, reviews, bundle, inventory, and committed history first; transcript
and trace material can only explain how work happened. That separation matters
because several reviewer approximations and runtime observations were weaker
than the committed closeout artifacts.

## Phase-by-phase variance

**OBSERVED**

| Phase | Planned boundary | Actual boundary | Variance/root cause | Cost | Structural response |
|---|---|---|---|---|---|
| CPFR-085 | Publish and land a full Jin route receipt and review with focused and full verification. | [execution-ledger-003.tsv](execution-ledger-003.tsv) records `landed` at `66045b6fd0be878bf76ad0171387e53619bf09f0` after a post-gate evidence-roster repair; [tracker.org](tracker.org) records the route as `complete-local`. | The route proof chain was correct, but the final evidence roster integration still needed repair after the first full gate. | One post-gate repair and fresh review before final landing. | Treat proof-complete and certification-complete as separate states; a compiling or reviewed route is not yet a landed certificate. |
| CPFR-086 | Publish and promote the six-node Lorist--Schwenninger route and land its verified result. | [execution-ledger-003.tsv](execution-ledger-003.tsv) binds the route to `d1749570825afc277fd96ff6c79b532449afbd16`, while [cpfr-086-retrospective.md](cpfr-086-retrospective.md) records locator repairs, promotion-transaction repairs, Python 3.9 fixture repairs, and a late repository-token repair. | The phase had to recover from incorrect promotion assumptions, lifecycle-fixture drift, and verifier/runtime edge cases before the metadata could close. | Multiple repair commits, review corrections, a focused gate, and a later full repository gate. | Freeze a recovery packet and a pre-review bundle before expensive review and gate work. |
| CPFR-087 | Certify Harp as a derived route with explicit LS reuse and release verification. | [execution-ledger-003.tsv](execution-ledger-003.tsv) records `landed` at `d9031068a883731cdab7b4eb80c744eda1912c07`; [cpfr-087-retrospective.md](cpfr-087-retrospective.md) records one review rewind, post-publication lifecycle/import repairs, and a verifier-driven tracker repair. | The route contract, publication, and focused gate all passed, but controller and reviewer roles blurred and phase-state artifacts were not fully atomic at entry. | One review rewind, one post-publication repair slice, and one metadata-repair slice. | Require controller/reviewer separation, immutable before/after snapshots, and a first-class recovery packet before work begins. |
| CPFR-088 | Publish one six-row local formalization bundle and require it everywhere. | [execution-ledger-003.tsv](execution-ledger-003.tsv) binds the landed bundle to `c09a0dc2a9fb6db19e0073d87e333f8fe86d92e7`; [tracker.org](tracker.org) records post-publication Rust/Python parity repairs and a runner-test timing repair before the definitive full gate. | The bundle publication itself was coherent, but verifier parity and runner timing assumptions were still wrong after publication. | Post-publication parity repairs plus one unrelated runner-test repair before the definitive 689-second full gate. | Separate proof-freeze verification from closeout metadata verification and require read-only before/after bundle snapshots. |
| CPFR-089 | Reconcile canonical prose and generated Atlas outputs to the published evidence. | [execution-ledger-003.tsv](execution-ledger-003.tsv) records landing at `573d92909972f8171e5c35ae3179a1d0181982e9`; [tracker.org](tracker.org) records initial wording defects, regeneration of the corpus/HTML/receipt trio, and a full gate that passed with a quiescent `verify-atlas` rerun. | Reader claims lagged the published evidence, and review still found wording drift after the first reconciliation pass. | One wording-repair cycle and a full 835-second release gate. | Freeze a pre-review reader bundle with tracked diff digest, parity checks, and read-only before/after snapshots. |
| CPFR-090 | Freeze the complete program, run independent reviews, run the serial warm-cache verifier, refresh the receipt last, and land locally. | [execution-ledger-003.tsv](execution-ledger-003.tsv) records evidence landing at `98af685eabda905076879c15920f2fb7d526b632` and closeout at `a7d7654dc9db7ed4df42809427a7437d707a4e8f`; [program-verification-003.json](program-verification-003.json) records the definitive serial verifier at 1,015 seconds, 8,775 Lean jobs, unchanged 123,454-record canonical cache digest `30248fd543222ce8e53c1409aa06e8f4335f651c52160cf059dd6d63ffebe29f`, and program receipt digest `f3fbf06e69717b8b15840e182bbc62c1c240b452177ddc2a751103bafc3a2280`. | The core verifier succeeded, but full-window Atlas metadata invariance could not be claimed; it had to be recorded as an explicit external volatile-metadata caveat rather than a blanket failure. | One 1,015-second serial verifier plus a 684-second final full gate for closeout metadata. | Serialize Lean/Lake and release gates, require a quiescent shared-tool window, and separate strict proof/cache/reader invariance from caveated volatile dependency metadata with the smallest isolated rerun. |
| CPFR-091 | Audit every Crouzeix worktree and remove only reviewed, clean, inactive, contained work in dependency order. | [execution-ledger-003.tsv](execution-ledger-003.tsv) records evidence landing at `87f5130b3415ffc628217f33b24294c9b1c36b89` and closeout at `d39b7b43399f1d90a58dc7567d4760a863416c25`; [worktree-inventory-002.md](worktree-inventory-002.md) records inventory digest `3bef1b809a351b08b112aa5531637e84ec029d433f3b8e6ad0f4283c18e32ded`, six non-force removals, and post-cleanup state of 23 worktrees, 26 branches, 5 contained, 21 unmerged, and 5 dirty preserved. The definitive post-cleanup gate passed in 604 seconds with 8,775 Lean jobs in 28 seconds. | The cleanup plan was narrower than "retire old worktrees": cache-provider/consumer ordering and user-preserved dirty/finalization state constrained the safe deletion set. | One inventory/repair/review cycle and one 604-second post-cleanup full gate. | Record cache provider/consumer edges before work and rescan them immediately before deletion; never use force deletion. |
| Phase-8 finalization | Distill bounded lessons into one retrospective and one plan update, then close the goal without making the footer self-referential. | This draft creates [retrospective-003.md](retrospective-003.md) and updates the [proof-goal plan's `Plan evolution` section](../../superpowers/plans/2026-08-23-crouzeix-end-to-end-proof-goal.md). The controller must land that substantive revision before it creates the review binding, ledger landing row, import-receipt refresh, and separate completion-footer commit. | The execution evidence is broad, but the substantive revision and final control metadata cannot be one self-referential commit. | One reviewed documentation landing followed by one focused footer closeout and final goal validation. | Keep the plan revision and footer finalization as two ordered commits with exact existing commit IDs. |

**INTERPRETATION**

The main variance pattern was not proof failure. It was lifecycle compression:
proof completion, certification completion, reader reconciliation, program
verification, worktree cleanup, and documentation closeout each needed their
own entry and exit states. Where those boundaries were explicit, the phase
closed cleanly. Where they were implicit, later repairs were required.

## Failed assumptions and root causes

**OBSERVED**

The durable record shows repeated assumptions that were too coarse:

- [execution-ledger-003.tsv](execution-ledger-003.tsv) and
  [tracker.org](tracker.org) show that route completion, evidence publication,
  release verification, and landing were distinct states across CPFR-085
  through CPFR-091.
- [cpfr-086-retrospective.md](cpfr-086-retrospective.md) and
  [cpfr-087-retrospective.md](cpfr-087-retrospective.md) show that recovery
  often required reconstructing base and candidate commits, publication state,
  and runtime assumptions after work had already started.
- [tracker.org](tracker.org) shows that CPFR-088 and CPFR-089 both needed
  review-driven repairs after the first intended implementation boundary.
- [program-verification-003.json](program-verification-003.json) shows that
  CPFR-090 had one strict invariant breach only in external Atlas dependency
  metadata, while proof outputs, bundle outputs, and the canonical Lean cache
  stayed fixed.
- [worktree-inventory-002.md](worktree-inventory-002.md) shows that safe
  deletion depended on explicit cache-provider/consumer ordering and preserved
  dirty or user-requested state.

The durable record also rebuts several weaker approximations:

- The canonical Phase-8 cache record is the 123,454-record cache in
  [program-verification-003.json](program-verification-003.json), not the
  134,192-record route-local cache snapshot mentioned in
  [cpfr-087-retrospective.md](cpfr-087-retrospective.md).
- CPFR-088 did incur meaningful cost: [tracker.org](tracker.org) records
  post-publication Rust/Python parity repairs and a runner-test timing repair
  before the definitive full gate.
- CPFR-089 did include rewinds: [tracker.org](tracker.org) records wording
  defects and their repair before final approval.
- A shell or `rg` no-match convention is not a durable proof fact unless the
  committed verifier records it as part of an accepted gate.

**INTERPRETATION**

The root cause was under-specified lifecycle control, not lack of effort or
proof depth. The program needed first-class state transitions, one explicit
recovery packet before work, frozen review bundles, runtime namespace probes
before expensive gates, and a stricter separation between durable invariants
and explicitly caveated volatile metadata.

## Verifier effectiveness

**OBSERVED**

The verifiers were effective when they checked the right boundary:

- Mathematical and route reviews caught load-bearing graph and wording defects
  that compilation alone would not catch, as recorded in
  [tracker.org](tracker.org),
  [cpfr-086-retrospective.md](cpfr-086-retrospective.md), and
  [cpfr-087-retrospective.md](cpfr-087-retrospective.md).
- [program-verification-003.json](program-verification-003.json) records that
  both program reviews passed on the frozen CPFR-090 tree with no Critical or
  Important findings, and that the serial verifier exercised all three route
  validators, three focused route builds, and the full `mise run verify` gate.
- [worktree-inventory-002.md](worktree-inventory-002.md) and
  [execution-ledger-003.tsv](execution-ledger-003.tsv) show that CPFR-091 used
  both pre-cleanup and post-cleanup review plus a full post-cleanup gate
  before closing the audit.

The verifiers were weaker when their role boundary was not explicit enough.
[cpfr-087-retrospective.md](cpfr-087-retrospective.md) records a read-only
verifier sequence that still produced a tracked metadata repair. That is a
process success only in the sense that the defect was found; it is not a clean
independent-review boundary.

**INTERPRETATION**

Verifier quality depended on controller discipline. Independent review works
best when the reviewer is bound to an immutable before/after state and can only
report findings. When the reviewer can also repair, the repository may still
improve, but the evidence boundary becomes harder to reason about and the same
gate has to serve both diagnosis and mutation.

## Cache and proof-iteration efficiency

**OBSERVED**

The durable cache record for the finalized program is in
[program-verification-003.json](program-verification-003.json): CPFR-090's
definitive serial verifier took 1,015 seconds, ran 8,775 Lean jobs, and left
the canonical Lean cache unchanged at 123,454 records with digest
`30248fd543222ce8e53c1409aa06e8f4335f651c52160cf059dd6d63ffebe29f`. The same
record binds the program receipt digest
`f3fbf06e69717b8b15840e182bbc62c1c240b452177ddc2a751103bafc3a2280`.

[tracker.org](tracker.org) records that CPFR-088's definitive full gate took
689 seconds and CPFR-089's full gate took 835 seconds. CPFR-091's release row
in [execution-ledger-003.tsv](execution-ledger-003.tsv) and the closeout text
in [worktree-inventory-002.md](worktree-inventory-002.md) record the
post-cleanup full gate at 604 seconds, with 8,775 Lean jobs in 28 seconds and
the canonical cache still unchanged.

No durable closeout artifact for Phase 8 records cold cache hydration, a
Mathlib cache refresh, or a justified cold rebuild. The recorded successful
gates all ran against the existing shared cache.

**INTERPRETATION**

The cache discipline worked. The right efficiency lesson is not "the gates were
cheap"; several were expensive. The right lesson is that they were expensive in
verification time, not in dependency-rehydration time, because the shared cache
contract held. That is why the next plan should serialize expensive gates, keep
the shared-tool window quiescent, and reject cache hydration or cold rebuild as
part of ordinary proof iteration.

## Context recovery quality

**OBSERVED**

Recovery quality was strong once phases wrote durable state:

- [execution-ledger-003.tsv](execution-ledger-003.tsv) gives a monotone,
  machine-readable phase history from `implementing` through `landed`.
- [program-verification-003.json](program-verification-003.json) gives one
  frozen program record with claim boundaries, reviewer identities, cache
  identity, route/bundle/reader digests, and runtime caveats.
- [worktree-inventory-002.md](worktree-inventory-002.md) gives one frozen
  cleanup packet with the precise pre/post counts, preserved exceptions,
  removal ordering, and dependency scan.

Recovery quality was weaker at phase entry. The CPFR-086 and CPFR-087
retrospectives both record late reconstruction of candidate state, publication
state, and runtime assumptions. The missing packet was not "more narrative"; it
was the absence of one consistent recovery bundle containing base and candidate
commits and trees, phase state, publication state, cache-provider/consumer
links, dirty paths, and licensing boundary.

**INTERPRETATION**

The durable record is good at answering "what closed?" and weaker at answering
"what exactly was the starting boundary for this work?" That is why the plan
should require one recovery packet before work begins, rather than letting each
phase rediscover its own starting state from scattered notes.

## Publication and landing safety

**OBSERVED**

The program preserved the main safety boundaries:

- Publication artifacts remained create-only and route- or bundle-bound, as
  recorded in [tracker.org](tracker.org) and the route receipts under
  [evidence/crouzeix_conjecture/routes/](../../../evidence/crouzeix_conjecture/routes/).
- [program-verification-003.json](program-verification-003.json) records that
  CPFR-090 separated the frozen proof program from later closeout metadata.
- [worktree-inventory-002.md](worktree-inventory-002.md) records six reviewed
  non-force removals and explicit preservation of dirty, divergent, cache-
  providing, and user-preserved state.
- The closeout commits for CPFR-090 and CPFR-091 are separate from their
  evidence landings: `98af685eabda905076879c15920f2fb7d526b632` then
  `a7d7654dc9db7ed4df42809427a7437d707a4e8f`, and
  `87f5130b3415ffc628217f33b24294c9b1c36b89` then
  `d39b7b43399f1d90a58dc7567d4760a863416c25`.

**INTERPRETATION**

The publication and landing model was safe when proof bytes, reader bytes,
closeout metadata, and cleanup actions were treated as different side-effect
classes. The next plan should encode that explicitly: proof or reader changes
invalidate the full gate, while metadata-only closeout should use exact focused
gates plus repository verification unless the governing task requires another
full gate.

## Plan changes

**OBSERVED**

The durable evidence across [tracker.org](tracker.org),
[execution-ledger-003.tsv](execution-ledger-003.tsv),
[program-verification-003.json](program-verification-003.json), and
[worktree-inventory-002.md](worktree-inventory-002.md) supports the following
executable changes:

1. Treat proof-complete and certification-complete as first-class entry and
   exit states.
2. Require one recovery packet before work: base and candidate commits and
   trees, current phase state, publication state, cache provider/consumer
   links, dirty paths, and licensing boundary.
3. Freeze one pre-review bundle: tracked diff digest, cross-language parity,
   lifecycle fixtures, recovery-path smoke, and read-only before/after
   snapshots.
4. Precompute the Seatbelt/runtime namespace and run a fake-probe pass before
   any expensive gate.
5. Serialize Lean/Lake and expensive release gates, require a quiescent
   shared-tool window, and distinguish strict proof/cache/reader invariance
   from explicitly caveated volatile dependency metadata with the smallest
   isolated rerun.
6. Record cache provider/consumer edges and rescan them immediately before
   worktree deletion.
7. Separate proof-freeze full verification from closeout-metadata verification:
   proof, bundle, reader, or runtime changes invalidate the full gate; metadata
   closeout gets exact focused gates plus repository verification unless the
   governing task explicitly requires another full gate.
8. Bind the exact uv-managed interpreter and child `sys.executable` usage, and
   keep a Python 3.9 compatibility probe in the boundary.
9. Refresh the import receipt strictly last and verify again; command
   automation remains deferred.
10. Bind reviewer identity and require controller/reviewer separation plus
    immutable before/after tracked state.

**INTERPRETATION**

These changes are not new scope. They are the minimal controls already implied
by the durable evidence. Encoding them in the plan reduces repeated repair
cycles without changing the claim ceiling.

## Rejected changes

**OBSERVED**

The durable record does not justify the following changes in this phase:

- No cache hydration or cold rebuild.
- No blanket shared-metadata failure when only explicitly caveated volatile
  dependency metadata moved.
- No force deletion.
- No weakening of theorem, evidence, or trust boundaries.
- No repository-wide pre-commit hook in this phase.
- No global skill edits.

The record also supports deferral, not immediate implementation, for:

- a typed ledger evidence object;
- an import-receipt refresh command;
- a worktree safe-remove command;
- a generated Seatbelt policy;
- transactional ticket APIs; and
- a capability-separated verifier runtime.

**INTERPRETATION**

Rejected changes are as important as adopted ones here. The safe boundary for
Phase-8 closeout is to preserve the validated proof/evidence surface, refuse
global enforcement mechanisms that would change the repository contract during a
docs-only closeout, and defer broader tooling redesigns to future design work.

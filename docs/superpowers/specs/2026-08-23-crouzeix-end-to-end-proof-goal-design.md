# Crouzeix end-to-end proof goal design

## Status and relationship to prior design

This document defines the controller architecture for completing and locally
landing the Crouzeix proof program. It is an execution amendment to
`2026-08-22-crouzeix-three-route-proof-completion-design.md`, not a replacement
for that design's mathematical scope, trust boundary, or evidence contracts.

The companion implementation artifact will be a goal-oriented plan at:

```text
docs/superpowers/plans/2026-08-23-crouzeix-end-to-end-proof-goal.md
```

That plan must be usable as the complete goal supplied to a fresh coding
agent. It must tell the agent how to recover current state, choose the next
authorized action, implement and verify each phase, land recoverable
checkpoints, and improve the plan after execution. It must not depend on this
conversation, private agent memory, or an uncommitted transcript.

## Objective

Finish the locally verifiable Jin, Lorist--Schwenninger, and Harp routes for
the finite-dimensional polynomial Crouzeix theorem and the explicitly
supported scalar rational and Hilbert-space consequences. Publish immutable
route evidence, require a six-row local formalization bundle, reconcile the
reader surfaces, pass an independent program verifier, land the work on local
`master`, and audit superseded Crouzeix worktrees.

The result is complete only at the repository's `complete-local` claim level.
A compiling terminal theorem, a tracker label, or an agent report is not the
terminal outcome.

## Terminal condition

The goal is complete only when all of the following are simultaneously true
on local `master`:

1. Jin, Lorist--Schwenninger, and Harp each have a valid route manifest,
   immutable route receipt, and approved proof review.
2. All three route validators report `complete-local` in Python and Rust.
3. The Jin and Lorist--Schwenninger manifests are source-faithful at the
   declared source-correspondence boundary.
4. The Harp manifest is explicitly derived, records all allowed lower-level LS
   reuse, and does not claim full mathematical independence.
5. The dedicated `CrouzeixJin`, `CrouzeixLoristSchwenninger`, and
   `CrouzeixHarp` targets pass using the canonical warm dependency cache.
6. Terminal and named load-bearing declarations have bounded, allowlisted
   axiom reports and route-correct provider reports.
7. The mandatory local formalization bundle contains exactly six bound rows:
   the main theorem and closed-numerical-range consequence for each route.
8. Canonical prose and generated Atlas readers derive their status statements
   from the validated route and bundle artifacts.
9. The exact final tree passes the program verifier, `lean-all`,
   `mise run verify`, `sources verify`, and `repository verify`.
10. Local `master` contains every focused phase landing by fast-forward; no
    push has occurred.
11. Every Crouzeix branch and worktree has a recorded disposition, and only
    clean, fully represented work has been removed.
12. A post-execution review has analyzed the real execution trace, updated the
    goal plan with durable improvements, and passed its own Spec and Standards
    reviews.

The theorem claim ceiling remains the one in the 08-22 design: finite
dimensional polynomial Crouzeix plus the registered scalar rational and
Hilbert-space consequences. The goal does not claim complete boundedness, a
matrix-valued functional calculus, abstract uniform-algebra generality, peer
review, publication, constructivity, or author endorsement.

## Design choice

### Chosen: goal contract with phased execution ledger

The plan combines a stable goal contract with phase-specific state recorded in
the existing tracker. Each phase has explicit entry conditions, owned files,
implementation slices, verifiers, review gates, evidence transitions, landing
rules, and fresh-context recovery instructions. Each verified phase lands
locally before the next phase starts.

This structure gives a fresh agent both the invariant end state and a bounded
next action. Local phase landings provide clean recovery points and prevent a
late failure from trapping all work on one long-lived integration branch.

### Rejected: issue graph without a controller goal

The existing CPFR issues are useful execution units, but they do not by
themselves prove that the entire program converged. An agent could close each
issue while missing cross-phase receipt, bundle, reader, landing, or cleanup
invariants. The tracker remains the progress ledger, while the goal plan owns
the terminal condition and transition rules.

### Rejected: one linear integration branch

A single long branch is easy to describe but weak under context loss, review
rewinds, and immutable evidence publication. Phase-by-phase local landings
make each certified route a recoverable mainline fact while later routes
remain honestly incomplete.

## Authority and state model

### Authority order

At the start of every agent session, authority is resolved in this order:

1. current user instructions and repository `AGENTS.md`;
2. the checked-out Git tree, worktree registrations, and immutable evidence;
3. the tracker entries and their recorded verifier evidence;
4. this goal plan and its last committed checkpoint;
5. prior agent summaries or transcripts.

The goal plan may contain a dated checkpoint for fast orientation, but disk
state wins. A fresh agent must never reset, overwrite, or delete divergent or
dirty work merely because the checkpoint is stale.

### Phase states

Each phase advances monotonically through these states:

```text
unclaimed
-> implementing
-> locally-verified
-> spec-approved
-> evidence-published
-> standards-approved
-> release-verified
-> landed
```

Not every phase publishes mathematical evidence, but it must still record an
explicit `evidence-published` equivalent such as a generated bundle or reader
receipt. A phase never advances because an implementer says it passed. The
tracker transition requires the verifier evidence named by the phase.

Failures rewind to the earliest falsified invariant:

- a slice or route failure returns to `implementing`;
- a Spec finding blocks publication and returns to `implementing`;
- a receipt-validation failure discards only an unpublished candidate and
  returns to the frozen execution boundary;
- a Standards finding that affects mathematics, source mapping, or evidence
  identity invalidates publication and returns to Spec review;
- a release-gate failure reopens the earliest phase whose contract it
  falsifies rather than patching only the aggregate symptom; and
- any source edit after a review invalidates that review unless the reviewer
  explicitly excluded the changed bytes.

### Durable progress record

The tracker is the mutable execution ledger. For each phase it records:

- branch, worktree, owner, agent run, and model;
- base commit and frozen candidate commit or tree identity;
- changed-file inventory and prohibited paths checked;
- exact commands, exit codes, test counts, durations, and important hashes;
- route closure, provider, source, declaration, and axiom identities;
- review verdicts, findings, repairs, and re-review results;
- evidence paths and digests; and
- focused commit and local landing commit.

Large logs remain in their registered evidence locations. The tracker records
bounded summaries and digests, not raw execution transcripts. The stable goal
plan changes only through an approved design change or the final
post-execution improvement phase.

## Non-negotiable execution invariants

### Lean and cache

- The canonical dependency cache is the primary checkout's
  `formalization/lean/.lake`.
- A Lean worktree must link its `.lake` to that exact cache after validating
  the link target and Git common-directory ancestry.
- Never run `lake update`, `lake --try-cache exe cache get Mathlib`, or
  `mise run lean-cache` during proof iteration.
- A missing or inconsistent cache is a blocked prerequisite, not permission to
  hydrate dependencies.
- Harp-owned `.olean` files may be rebuilt.
- Lean commands that can write the shared cache run serially.
- Run no-build preflight before each expensive route or release gate.
- A passing non-Seatbelt Lean command is a local compile result, not a claim of
  hermetic execution. The plan must use the repository's actual containment
  capability and label any weaker result accurately.

### Proof and claim integrity

- Do not add `sorry`, `admit`, custom `axiom`, `opaque`, `unsafe`,
  `native_decide`, or `implemented_by`.
- Jin and LS source claims bind exact upstream identities, source spans, and
  source-correspondence classifications.
- Harp reuse is limited to the enumerated lower-level LS modules.
- A semantic dependency edge cannot be presented as the active Lean proof
  dependency without an explicit qualifier.
- Direct ports, compatibility ports, structural source-correspondent
  refactors, reused nodes, and Harp-derived extractions must remain
  distinguishable in machine-readable evidence.
- The historical Jin and LS receipt directories are immutable. New route
  evidence supplements them.

### Evidence publication

- Publication is two phase: prepare an unreferenced candidate, validate it,
  then atomically promote its digest into the authoritative ledger.
- Candidate directories are create-only and never overwritten.
- No route receipt is published before its manifest and mathematical Spec
  boundary are frozen.
- No approved proof review is published before the bound execution receipt
  exists and validates.
- Every consumer independently recomputes identities rather than trusting
  declared closure, source, provider, or axiom data.
- Python and Rust must accept and reject the same evidence contract.
- Upstream material without a license remains explicitly
  `not-present-at-revision` and `quotation-only`; hashes and locators do not
  authorize vendoring the source tree.

### Git and landing

- New phase work uses an isolated worktree based on current local `master`.
- Preserve unrelated dirty and untracked state.
- Stage explicit paths; never use `git add .`.
- Commit focused concerns that are independently verifiable.
- Land locally by fast-forward only after the phase's release gate.
- Do not push.
- Refresh `docs/import-receipt.md` only after every other tracked byte in the
  landing is settled, then rerun the required verifier.
- Remove a worktree only after proving it is clean, inactive, and fully
  represented on `master`; remove its branch only after the worktree.

## Roles and separation of authority

### Controller

The controller selects the earliest unfinished phase, creates or recovers its
worktree, enforces ownership boundaries, dispatches bounded implementation and
review work, runs final verification, updates the tracker, and performs local
landing. It may not weaken theorem scope or trust boundaries to make a gate
pass.

### Implementer

The implementer owns only the files named for its slice. It follows RED/GREEN
TDD, reports exact evidence, and does not publish, merge, push, clean up
worktrees, or declare the phase complete.

### Mathematical Spec reviewer

The Spec reviewer is read-only and independent. It compares the formal route
to the source or derivation argument, checks theorem statements, hypotheses,
dependency edges, claim ceiling, and reuse/novelty classification, and returns
`PASS` or `FAIL` with Critical, Important, and Minor findings. Critical and
Important findings block publication.

### Standards reviewer

The Standards reviewer runs only after Spec approval and evidence validation.
It reviews implementation quality, validator parity, fail-closed behavior,
filesystem and publication safety, maintainability, and repository standards.
It is not merge authority.

### Independent verifier

The verifier receives only the goal plan, phase base and candidate identities,
candidate worktree, claimed artifact paths and hashes, and relevant source
identities. It does not rely on the implementer's reasoning narrative. It may
read and execute bounded verification commands but may not edit, publish,
merge, clean up, or widen scope.

Its result has this shape:

```text
PASS | FAIL

Critical findings
Important findings
Minor findings
Verified commands and results
Unverified claims
Allowed next transition
```

A phase advances only on `PASS` with no unresolved Critical or Important
finding.

## Verifier hierarchy

### Slice verifier

The slice verifier runs during RED/GREEN iteration and proves the changed
behavior. It uses the smallest relevant combination of:

- focused Python or Rust tests;
- schema and canonical-JSON validation;
- declaration-type and source-excerpt digest recomputation;
- negative mutation tests; and
- one narrow cached Lean target when proof code changes.

The test must be observed failing for the intended reason before production
code changes.

### Phase verifier

The phase verifier runs on a frozen candidate and checks:

- candidate Git tree and dirty-state identity;
- no-build cache, toolchain, target, and provider preflight;
- dedicated route compilation where applicable;
- exact active local and Mathlib closure;
- provider-policy report;
- terminal and named load-bearing `#print axioms` output;
- source or derivation correspondence;
- declaration-type, source-file, source-excerpt, and evidence hashes;
- immutable receipt and proof-review bindings in Python and Rust;
- relevant full Crouzeix lab and Rust source suites; and
- absence of unauthorized publication or mutation.

### Program verifier

The program verifier runs only after the three routes, bundle, and reader
phases have landed. At minimum it executes:

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  validate --route all

MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-all
cargo run -q -p harp -- sources verify
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run verify
cargo run -q -p harp -- repository verify
```

The final plan must expand these commands into an exact ordered gate with
expected exit states and must add checks that:

- local `master` contains each phase landing;
- the six-row bundle is mandatory and valid;
- route receipts bind the same tree, command, closure, provider, declaration,
  and review artifacts being reported;
- no prohibited Lean construct appears in an active route;
- no receipt or generated artifact changed after its bound review; and
- `docs/import-receipt.md` has the final payload digest and was changed last.

## End-to-end phase architecture

### Phase 0: recover and inventory

Purpose: establish the real starting state without destroying interrupted work.

Required actions:

1. Read `AGENTS.md`, the 08-22 design and plan, this goal plan, the tracker,
   and the latest relevant retrospective.
2. Record local `master`, all Crouzeix worktrees and branches, dirty/untracked
   paths, active processes, and relevant Kata state.
3. Validate the canonical `.lake` cache link, pinned toolchain, manifest, and
   required dependency artifacts without invoking Lake.
4. Find the earliest phase that is not demonstrably landed.
5. If a phase worktree is dirty, classify every path as intended phase work,
   controller-owned state, generated output, unrelated user work, or scratch.
6. Preserve all unexplained state and record the recovery decision.

Exit condition: one phase is claimed in one isolated worktree; prior landings
are verified; no state was discarded.

The first execution of this plan is expected to recover the interrupted
CPFR-085 worktree. Its embedded checkpoint is advisory: Jin's provider repair
has been partially applied, while the route manifest may still describe the
older closure. The recovery verifier must inspect current bytes and processes
rather than assuming this remains true.

### Phase 1: certify and land Jin

Purpose: make the complete Jin source-correspondence route truthful, reviewed,
receipt-backed, and locally landed.

Required proof/evidence boundary:

- a dedicated Jin target with no LS or Harp provider imports;
- every named load-bearing step bound to pinned Jin source or an explicit
  source-correspondent/derived classification;
- actual Lean provider dependencies represented rather than merely semantic
  ancestry;
- the rational bridge and Hilbert wrappers consuming the certified terminal
  route;
- the sole Harp-derived polynomial-specialization extraction identified;
- the three historical proof-slice receipts byte-identical; and
- exactly the declared finite rational and Hilbert consequences, within the
  claim ceiling.

Execution sequence:

1. Recover and finish the current TDD repair for the Jin rational bridge,
   route terminal, Hilbert provider wrappers, manifest DAG, and Git-locator
   parser parity.
2. Recompute the active closure, declaration types, source metadata, and
   provider policy from the actual tree.
3. Require unpublished validation to return `mapped / incomplete`, never
   `authored / invalid`.
4. Freeze candidate hashes and obtain independent mathematical/source-fidelity
   Spec approval.
5. After Spec approval, run the frozen route once and collect terminal and
   load-bearing axiom audits.
6. Create an immutable receipt candidate, validate it independently, and
   atomically publish its digest and bound proof review.
7. Obtain Standards approval, run the Jin phase release gate, commit focused
   concerns, fast-forward local `master`, and close CPFR-085.

No route execution output collected before the final Spec freeze may be used
as the immutable publication receipt.

### Phase 2: certify and land Lorist--Schwenninger

Purpose: publish one coherent six-node LS route from the existing theorem and
receipt machinery without rewriting historical v1 evidence.

Required boundary:

- all six canonical graph nodes have current v2 receipts sharing one aggregate
  execution identity;
- source orientation `T = op(P)` and `Q = M_h`, recurrence, scalar
  contradiction, double-layer realization, and both limits match the pinned
  source;
- graph and library inventory promotion is atomic;
- no Jin or Harp provider crosses the route boundary; and
- historical attempts remain intact.

Sequence: recover graph state, write RED receipt/promotion tests, run one
dedicated cached LS execution, prepare all six candidates, independently
validate and review them, promote graph and inventory atomically, publish the
route receipt/review, obtain Standards approval, run the LS phase gate, land,
and close CPFR-086.

### Phase 3: certify and land Harp

Purpose: certify the Harp finite-horizon route as a derived route with explicit
lower-level LS reuse and no terminal-provider substitution.

Required boundary:

- the derivation ledger exposes finite positive cubature, finite atomic
  counting-L2 realization and dimension bound, finite-horizon recurrence,
  perturbation endpoint, double-layer application, both limiting passages,
  and terminal assembly;
- all and only the eleven approved LS support modules are explicit reused-route
  nodes or dependencies;
- no Jin or LS terminal/provider import occurs; and
- the novelty claim distinguishes Harp-owned finite atomic construction from
  reused LS scalar, norm-attainment, boundary, and moment machinery.

Sequence: write and validate the derivation/reuse graph, obtain mathematical
and novelty Spec approval, execute the dedicated cached Harp target, collect
axiom/provider evidence, publish and validate the immutable receipt/review,
obtain Standards approval, run the phase gate, land, and close CPFR-087.

### Phase 4: publish and require the six-row local bundle

Purpose: make the cross-route local evidence boundary mandatory in the same
commit that publishes the genuine bundle.

The exact rows are:

```text
jin-main-theorem
jin-closed-numerical-range
ls-main-theorem
ls-closed-numerical-range
harp-main-theorem
harp-closed-numerical-range
```

The publisher runs one broad cached Crouzeix build, six declaration/axiom
audits, and three provider audits. It must not accept caller-supplied
precomputed command output. Python and Rust must independently reject missing,
altered, aliased, symlinked, cross-route, or stale artifacts. The mandatory
enforcement flag and genuine bundle land atomically. Close CPFR-088 only after
`sources verify` proves deletion of any member fails closed.

### Phase 5: reconcile canonical prose and generated readers

Purpose: make reader-visible proof status a projection of validated evidence.

Update only canonical Crouzeix packet prose, claim ledger, source registry,
provenance record, lab instructions, and their tests. Keep Jin upstream
verification distinct from Harp-local certification. State LS source fidelity
and Harp reuse boundaries exactly. Regenerate Atlas corpus, HTML, and receipt
together; never hand-edit derived outputs. Run packet, link, schema, Atlas
lint/typecheck/test/build, and static-export gates. Land and close CPFR-089.

### Phase 6: independent program verification and review

Purpose: verify the exact integrated tree rather than composing old phase
claims.

Create a dedicated finalization worktree from the local `master` that already
contains the Phase 1--5 landings. Freeze that candidate; run a fresh
mathematical/program Spec review and then a fresh Standards review. Repair
every Critical or Important finding in the finalization worktree and repeat the
affected review. Run the complete program verifier, all route validations, all
three dedicated route builds, `lean-all`, the mandatory bundle verifier,
`mise run verify`, and repository verification. Refresh
`docs/import-receipt.md` only after all other tracked bytes settle, then rerun
the required gate. The finalization branch remains unlanded until the exact
tree passes. Close CPFR-090 only after recording that verified tree and its
focused finalization commits.

### Phase 7: final local landing and worktree audit

Purpose: leave local `master` authoritative and retain no unexplained
Crouzeix execution state.

Fast-forward the verified Phase 6 finalization branch locally to `master`.
Inventory every remaining Crouzeix branch/worktree: head, merge base, unique
commits, patch identity, dirty/untracked paths, active processes, and master
containment. Remove only clean, inactive, fully represented worktrees and then
their branches. Preserve dirty, divergent, or unexplained state with an
explicit disposition. Never touch unrelated worktrees or primary-checkout
untracked paths. Land the inventory and close CPFR-091.

### Phase 8: post-execution review and plan improvement

Purpose: make the next execution safer and faster using evidence from this
run, without rewriting history or weakening claims.

A fresh review agent compares the goal plan with actual execution:

- planned versus actual phase boundaries;
- predicted versus observed commands, test counts, and runtimes;
- every failed test, review finding, and rewind;
- repeated context-loss, ownership, cache, or publication problems;
- prose rules that should have been structural tests;
- redundant, incomplete, or unsafe commands; and
- whether a fresh agent could resume from every recorded checkpoint.

It writes:

```text
docs/workstream/crouzeix-proof-reproduction/retrospective-003.md
```

It then revises the goal plan's `Plan evolution` section and any affected
procedure. Each revision cites execution evidence. Improvements may clarify,
tighten, reorder, or automate the workflow. They may not retroactively weaken
the theorem claim, evidence requirement, human authority, or trust boundary. A
change to those foundations requires a new design.

The revised plan and retrospective receive independent Spec review followed by
Standards review. Phase 8 first lands a focused substantive plan-improvement
commit. A separate completion-footer commit then records the already-existing
Phase 7 program landing and Phase 8 plan-revision commits. This two-commit
sequence avoids asking a Git commit to embed its own identity.

## Fresh-context resume algorithm

A coding agent given only the goal plan performs this algorithm:

1. Read repository guidance and the plan's terminal condition.
2. Inspect current `master`, worktrees, branches, status, active processes,
   tracker states, and evidence paths.
3. Verify prior phase landings from Git and artifacts; do not trust labels.
4. Select the earliest phase not demonstrably `landed`.
5. Recover an existing phase worktree if its state is coherent; otherwise make
   a new isolated worktree from current `master` without deleting the old one.
6. Run the phase entry verifier and record the result.
7. Start at the first failed or absent transition, not at the beginning of the
   phase.
8. Use RED/GREEN implementation slices and keep Lean writes serial.
9. Freeze, review, publish, verify, and land according to the phase contract.
10. Update the tracker with evidence and repeat until the terminal condition is
    satisfied.
11. Run Phase 8 and update the completion footer only after its reviews pass.

The agent does not ask whether to continue after each successful phase. It
continues until the goal is complete or a defined stop condition applies.

## Stop and escalation conditions

The controller stops for user direction only when:

- required authority would expand beyond local code, evidence, and Git state;
- the canonical shared Lean cache is missing or invalid and progress requires
  dependency hydration;
- dirty or divergent state cannot be safely attributed or preserved;
- a requested mathematical or trust-boundary change conflicts with the
  approved design;
- the same external blocking condition has occurred for three consecutive goal
  turns and no meaningful in-scope progress remains; or
- completing the phase requires a push, remote publication, persistent
  service, or other action not authorized by this goal.

Ordinary test failures, review findings, difficult proofs, and repair work are
not stop conditions.

## Goal-plan deliverable requirements

The implementation plan produced from this design must be more specific than
this architecture. It must include:

1. an initial advisory checkpoint with the current CPFR-085 partial repair;
2. exact files and symbols for every task;
3. checkbox-sized RED/GREEN implementation slices;
4. exact commands, environments, expected exit codes, and expected artifact
   counts;
5. reusable prompts for implementer, Spec reviewer, Standards reviewer, phase
   verifier, and post-execution plan reviewer;
6. the ownership boundary between controller-managed tracker state and worker
   files;
7. create-only publication and rewind procedures;
8. focused commit and phase landing boundaries;
9. fresh-context restart instructions at every phase;
10. the program verifier and plan-improvement verifier; and
11. a machine-scannable completion footer.

The plan must not contain unresolved `TBD`, `TODO`, placeholder commands,
unknown expected states, or steps that tell an agent to infer whether a review
or verifier is necessary. It may tell the agent to recompute drift-prone
identities from disk rather than hard-code them.

## Completion footer contract

The goal plan ends with this block. It remains non-final until all referenced
evidence validates:

```yaml
goal_status: in-progress
jin: incomplete
lorist_schwenninger: incomplete
harp: incomplete
local_bundle: absent-or-optional
reader_surfaces: unreconciled
program_verifier: pending
master_landing: null
worktree_audit: pending
post_execution_review: pending
plan_revision: null
```

After the substantive Phase 8 revision passes and lands, the controller
changes the footer to the verified terminal form in a separate commit:

```yaml
goal_status: complete
jin: complete-local
lorist_schwenninger: complete-local
harp: complete-local
local_bundle: required-and-valid
reader_surfaces: reconciled
program_verifier: passed
master_landing: <exact Phase 7 program-landing commit>
worktree_audit: passed
post_execution_review: passed
plan_revision: <exact Phase 8 substantive plan-revision commit>
```

The angle-bracketed values above describe runtime substitutions, not text that
may remain in a completed goal file. The footer-finalization commit itself is
discovered from Git and need not contain its own hash. The footer is a summary
and resume aid. It never substitutes for route manifests, receipts, reviews,
bundle artifacts, tracker evidence, or Git history. A goal verifier must reject
a `complete` footer whose referenced commits are absent, out of order, or whose
referenced state cannot be independently reproduced.

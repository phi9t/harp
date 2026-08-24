# Lorist-Schwenninger route certification design

## Purpose

CPFR-086 certifies Harp's Lorist-Schwenninger route as a complete local Lean
proof with source correspondence. The Lean library already compiles substantive
proofs for the six planned nodes, but the current source graph still binds four
nodes to proposition aliases and marks them blocked. No LS route receipt or
independent proof review has been published.

This phase reconciles that stale tracking state with the declarations Lean
actually checks. It then publishes immutable node and route evidence, after an
independent mathematical review against the pinned manuscript.

## Current state

The shared Lean root is `formalization/lean/`, pinned to
`leanprover/lean4:v4.32.1`. Linked worktrees use the primary checkout's
`formalization/lean/.lake` directory as a machine-local dependency cache. Proof
iteration must not run `lake update`, `lake --try-cache exe cache get Mathlib`,
`mise run lean-cache`, or any equivalent dependency hydration command.

CPFR-085 is landed on local `master`. Its Jin route is `complete-local`, and its
receipt and review are immutable. CPFR-086 therefore starts from a clean,
verified proof base.

The LS source graph has six ordered nodes. Two have historical passed receipts:

- `ls-equation-one-terminal-bound`
- `ls-scalar-contradiction`

Four are marked blocked:

- `ls-power-recurrence`
- `ls-perturbation-lemma`
- `ls-double-layer-realization`
- `ls-terminal-crouzeix`

Those four blocked labels are stale relative to the Lean source. The repository
contains compiling candidate theorems for all four obligations. Compilation
alone does not establish source correspondence, so this phase must review the
theorem statements, source spans, and proof-provider edges before changing any
node to passed.

There is no LS route receipt or proof review. `proof_evidence.py validate
--route lorist-schwenninger --allow-unpublished` therefore reports `authored /
incomplete`.

## Design decision

Each source-graph node will name a concrete theorem declaration that Lean
checks. Proposition aliases such as `PowerRecurrenceStatement`,
`PerturbationLemma`, and `CrouzeixTerminal` may remain as explanatory vocabulary,
but they are not certification witnesses.

Three approaches were considered:

1. Bind each node to the existing concrete theorem that proves the corresponding
   mathematical step.
2. Add bridge theorems whose names match the current aliases.
3. Keep proposition aliases as nodes and infer their proof from downstream
   consumers.

The first approach is selected. It records what Lean checked without adding
duplicate declarations. A bridge theorem is permitted only if source review
shows that an existing theorem is materially weaker than the intended node.
The third approach is rejected because a proposition definition is not proof
evidence.

## Six-node correspondence

The certified graph has exactly these nodes in source order:

| Node | Concrete Lean declaration | Direct certified dependencies |
| --- | --- | --- |
| `ls-equation-one-terminal-bound` | `CrouzeixConjecture.LoristSchwenninger.DilationData.perturbation_mul_target_power_norm_le` | none |
| `ls-power-recurrence` | `CrouzeixConjecture.LoristSchwenninger.DilationData.equation_three_lower_bound` | `ls-equation-one-terminal-bound` |
| `ls-scalar-contradiction` | `CrouzeixConjecture.LoristSchwenninger.scalar_endpoint_le_two` | none |
| `ls-perturbation-lemma` | `CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two` | `ls-power-recurrence`, `ls-scalar-contradiction` |
| `ls-double-layer-realization` | `CrouzeixConjecture.LoristSchwenninger.norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary` | `ls-perturbation-lemma` |
| `ls-terminal-crouzeix` | `CrouzeixConjecture.loristSchwenningerMainTheorem` | `ls-double-layer-realization` |

The dependency list records actual Lean proof-provider dependencies after
contracting helper declarations that are not separate source nodes. It does not
copy narrative dependencies from the manuscript. The terminal node depends on
the double-layer node, which already consumes the perturbation result. Shared
geometry, approximation, and limiting modules belong in the route's shared
foundation roster rather than as invented LS source nodes.

Before this mapping becomes authoritative, tests must inspect the theorem bodies
and reject missing or extra certified provider edges. Normalized declaration
types and statement hashes must be regenerated from the concrete declarations.

## Source contract

The LS source is the pinned manuscript `arxiv:2608.03841v1`. Its registered
identities are:

```text
arxiv:2608.03841v1
sha256:20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a
sha256:b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9
```

The archive digest is
`b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9`.
The `CrouzeixConjecturev2.tex` member has 18,783 bytes, SHA-256
`20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a`,
and 281 lines.

When review needs the source bytes, acquisition writes them only to a fresh
directory under `/private/tmp`. The archive and TeX file are not copied into the
repository. The temporary source remains available until mathematical review
finishes.

The locator grammar accepts only the exact versioned arXiv form specified by
the end-to-end plan. Python, JSON Schema, and Rust must agree on accepted and
rejected identities. Every source node binds the registered manuscript digest,
its exact line range, line count, and excerpt digest.

The initial source spans remain:

- lines 67 through 99 for Equation 1 and the full perturbation argument;
- lines 74 through 90 for the recurrence;
- lines 90 through 98 for the scalar contradiction;
- lines 100 through 124 for the double-layer application;
- lines 125 through 128 for the terminal theorem.

Review may narrow overlapping spans when a smaller range contains the complete
claim and required local definitions. It may not widen a locator without
recomputing its excerpt digest and explaining the additional source dependency.

## Candidate and authority boundaries

The current compiled theorems are candidates until source review approves the
six-node correspondence. During reconciliation:

- the existing graph and inventory remain authoritative for their historical
  state;
- candidate graph and inventory bytes stay outside their final paths;
- no route receipt or review is published;
- no tracker statement calls a blocked node passed;
- historical v1 and existing v2 attempt directories remain unchanged.

After all six node receipts validate and mathematical review passes, one atomic
promotion changes `source-graph.json` and `library-inventory.json` to the new
generation. A reader must never observe one new file with one old file.

## Node execution and receipts

The route publisher performs one cached build of
`CrouzeixLoristSchwenninger`. That build supplies the shared command, toolchain,
cache, source-closure, and Mathlib-artifact identity for all six nodes. The
publisher then runs one declaration-specific axiom audit for each node.

Each node gets a new immutable v2 attempt. Attempt directories use monotonic
numbers. If the expected next path already exists, the publisher validates the
existing attempt and selects the next unused number. It never overwrites or
renumbers prior evidence.

Every passed node receipt binds:

- node ID and concrete declaration;
- source locator, manuscript identity, and statement digest;
- exact dependency receipt identities;
- dedicated aggregate target and successful command result;
- exact theorem-module bytes;
- local source closure and required Mathlib artifact digests;
- declaration-specific axiom output;
- provider-independent closure evidence; and
- an externally computed receipt-file digest that the promoted graph records.

The later route receipt binds the frozen Git commit and tree plus normalized
types for every certified declaration. Keeping these identities at the route
level matches the existing v2 node-receipt schema and avoids rewriting
historical receipt contracts.

The only allowed axioms are `Classical.choice`, `Quot.sound`, and `propext`. The
active route closure must contain no Jin or Harp provider module. Source scanning
rejects `sorry`, `admit`, custom `axiom`, `opaque`, `unsafe`, `native_decide`,
and `implemented_by`.

## Atomic graph and inventory promotion

`ls_promotion.py` owns the promotion boundary. Its public function is:

```python
promote_six_node_route(repository_root, candidate_receipts) -> None
```

The function accepts no caller-supplied command, graph, inventory, environment,
or output bytes. It loads canonical inputs, validates all six candidate receipts,
and derives the graph and inventory. It revalidates source and candidate bytes
immediately before publication.

Promotion uses a staged generation plus a commit marker or an equivalent
descriptor-relative compare-and-replace protocol. The reader recognizes only a
complete committed generation. Pre-commit failures remove owned staging state.
Post-commit durability failures report a committed-publication error and leave
the visible generation intact for recovery. Races never overwrite an existing
valid generation.

## Mathematical review

An independent reviewer receives the frozen candidate commit and tree, the six
normalized theorem types, the complete Lean provider graph, and the pinned TeX.
The review checks:

- the identification of `T`, `Q`, the isometry, and the perturbation family;
- Equation 1 and uniform boundedness of the terminal products;
- inner-product orientation and the operator recurrence;
- the scalar contradiction and all positivity assumptions;
- assembly of the perturbation lemma;
- construction of the boundary multiplier and companion operators;
- the normalized polynomial estimate;
- the simple-spectrum and outer-domain limit steps;
- the terminal theorem's exact claim;
- every direct-source, compatibility-port, and structural-refactor label.

A Critical or Important mathematical finding blocks promotion and publication.
Any mathematical, source-locator, theorem-type, or provider-edge repair creates
a new candidate commit and invalidates the previous review and node receipts.

## Route publication

Before mathematical review, an unpublished route manifest binds the proposed
six nodes, all shared modules, exact closure, terminal type, allowed axioms,
source identities, receipt path, and review path. Its receipt and review digests
are null. While the historical graph remains active, validation reports this as
an authored incomplete candidate. After approved graph promotion, the same
manifest becomes eligible for route publication.

For the LS route, validation also reads the promoted source graph, library
inventory, promotion marker, and six selected v2 receipts. It checks their
semantics and verifies that their current bytes match the route candidate Git
commit. The route receipt therefore covers the promoted six-node evidence chain
through its candidate identity without changing the generic receipt schema.

Publication then proceeds in this order:

1. Run no-build route preflight.
2. Publish the create-only LS route receipt.
3. Bind the receipt file digest in the manifest.
4. Publish the approved review against that receipt and frozen candidate.
5. Bind the review file digest in the manifest.
6. Validate the route as `complete-local` in Python and Rust.

Publication artifacts use the canonical paths beneath:

```text
evidence/crouzeix_conjecture/routes/lorist-schwenninger/
evidence/crouzeix_conjecture/reviews/lorist-schwenninger.json
```

The route receipt records a complete local theorem. It does not claim peer
review, author endorsement, complete boundedness, or independence from shared
Mathlib and Harp foundation modules.

## Failure and recovery behavior

- A missing or mismatched toolchain or cache blocks before invoking Lake.
- A missing dependency artifact blocks without downloading or rebuilding common
  dependencies.
- A build, source-scan, provider, or axiom failure publishes an immutable failed
  attempt when the attempt contract permits it. It does not promote the graph.
- A mathematical review rejection preserves candidate evidence but does not
  publish a route receipt.
- A promotion failure before commit leaves the old graph and inventory active.
- A failure after the promotion commit is reported as committed and retains the
  new generation for inspection.
- A route publication collision preserves the existing destination.
- A failure after route rename is reported as committed publication and never as
  a clean rollback.
- A release-gate failure preserves the branch and worktree for repair. It does
  not change `master`.

## Verification

Focused development uses the narrowest relevant checks. Lean commands run
serially because all worktrees share one dependency cache. The final candidate
must pass:

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  preflight --route lorist-schwenninger

scripts/check_lean_library.sh CrouzeixLoristSchwenninger

python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  validate --route lorist-schwenninger

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_ls_receipts \
  labs.crouzeix_proof_reproduction.tests.test_ls_validation \
  labs.crouzeix_proof_reproduction.tests.test_route_lorist_schwenninger \
  labs.crouzeix_proof_reproduction.tests.test_route_publication -v

cargo test -p harp sources::crouzeix --lib -- --test-threads=1
mise run verify
```

The final route validator must report `complete-local`. The independent Spec and
Standards reviews must have no unresolved Critical or Important findings. The
full warm-cache repository gate must pass on the exact final payload commit.
`docs/import-receipt.md` is refreshed last, followed by repository verification.

## Landing and dependency order

CPFR-086 lands through `git merge --ff-only` onto local `master`. No push occurs.
The worktree remains until CPFR-091 performs the final audit. Kata `harp#2eb2`
closes only after post-merge route and repository validation.

Before implementation begins, Kata must record that CPFR-087 depends on
CPFR-086. Harp's derived route consumes exact LS manifest and receipt identities,
so parallel certification against an unpublished LS route is invalid. The
remaining program order is:

```text
CPFR-086 LS certification
CPFR-087 Harp derived-route certification
CPFR-088 mandatory six-row local proof bundle
CPFR-089 claim and reader reconciliation
CPFR-090 final three-route review and landing
CPFR-091 worktree audit and retirement
```

## Completion criteria

CPFR-086 is complete only when all of the following are true:

- all six graph nodes name concrete compiled theorem declarations;
- all six v2 receipts validate and share one aggregate build identity;
- the graph and inventory expose one committed generation;
- mathematical review approves all six source correspondences;
- the LS route receipt and review are published and hash-bound;
- Python and Rust report the LS route as `complete-local`;
- the cached LS target and full repository gate pass;
- historical v1 attempts and Jin evidence remain byte-identical;
- local `master` advances by fast-forward;
- the execution ledger and tracker record the landing evidence;
- Kata `harp#2eb2` is closed with the landing commit and test commands;
- the goal validator advances to CPFR-087.

## Non-goals

- Rewriting historical v1 attempts.
- Replacing the dedicated LS target with a broad Crouzeix build.
- Treating a proposition alias as proof evidence.
- Claiming that Harp independently reproved LS modules.
- Publishing source bytes whose redistribution status is not established.
- Hydrating or updating common Lean dependencies.
- Starting CPFR-087 before the LS route identity is landed.
- Pushing any branch or commit.

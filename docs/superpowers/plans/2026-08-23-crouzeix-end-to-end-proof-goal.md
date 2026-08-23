# Crouzeix End-to-End Proof Goal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Locally land independently verified, source- or derivation-mapped, immutable `complete-local` proof evidence for the Jin, Lorist--Schwenninger, and Harp Crouzeix routes; require the six-row local bundle; reconcile reader claims; audit worktrees; and improve this plan from the recorded execution.

**Architecture:** Use the existing shared Lean root and canonical dependency cache, but treat each proof route as an isolated evidence boundary. Complete and locally land Jin, LS, and Harp serially so each route becomes a recoverable mainline checkpoint. Publish evidence through create-only staged producers, validate it independently in Python and Rust, and require Spec review before Standards review.

**Tech Stack:** Lean 4.32.1, mathlib 4.32.1, Lake, POSIX shell, Python 3 standard library and `unittest`, Rust/Cargo, JSON Schema draft 2020-12, mise, Git worktrees, Kata.

---

## This file is the coding-agent goal

When this file is supplied as a goal, continue through every unchecked task
without asking whether to proceed. Stop only at a condition listed under
`Global stop conditions`. Disk and Git state are authoritative; the advisory
checkpoint below can become stale.

The terminal condition is all of the following on local `master`:

- Jin, Lorist--Schwenninger, and Harp validate as `complete-local`;
- all three dedicated targets and `lean-all` pass from the canonical shared
  cache without dependency hydration;
- terminal and load-bearing declarations have allowed axiom and provider
  reports;
- the exact six-row local formalization bundle is mandatory and valid;
- canonical prose and generated Atlas outputs match the evidence;
- the final tree passes `mise run verify` and repository verification;
- no push occurred;
- every Crouzeix worktree has an audited disposition; and
- `retrospective-003.md` and a reviewed improvement to this plan are landed.

Do not mark the goal complete because a terminal theorem compiles.

## Required workflow and authority

1. The controller owns tracker transitions, integration, evidence promotion,
   local landing, and cleanup.
2. Use a fresh implementation subagent for each bounded task.
3. Use RED/GREEN TDD for every code or behavior change.
4. After a task is locally verified, obtain an independent Spec review.
5. Fix every Critical or Important Spec finding and request a re-review.
6. Only after Spec approval and any required evidence publication, obtain an
   independent Standards review.
7. Fix every Critical or Important Standards finding and request a re-review.
8. The controller independently reruns the task verifier before committing.
9. Stage explicit paths only. Never use `git add .`.
10. Land locally by `git merge --ff-only`; never push.

The execution-ledger `state` field uses only this ordered enum:

```text
implementing
locally-verified
spec-approved
evidence-published
standards-approved
release-verified
landed
```

The first row for a recovered phase uses `implementing` and records
`recovered-existing-worktree` in `note`.

Reviewer output must use this form:

```text
PASS | FAIL

Critical findings
Important findings
Minor findings
Verified commands and results
Unverified claims
Allowed next transition
```

No reviewer has authority to edit files, publish evidence, merge, delete, or
widen the theorem claim.

## Global invariants

### Shared Lean cache

Use exactly:

```sh
export ELAN_HOME=/private/tmp/harp-mathematical-foundations-elan
export ELAN_TOOLCHAIN=leanprover/lean4:v4.32.1
export PATH="/private/tmp/harp-mathematical-foundations-elan/toolchains/leanprover--lean4---v4.32.1/bin:$PATH"
```

Every proof worktree's `formalization/lean/.lake` must be a symlink resolving to
the primary checkout's `formalization/lean/.lake`. Before a Lean command, run:

```sh
primary=$(git rev-parse --path-format=absolute --git-common-dir)
primary=${primary%/.git}
test "$(readlink formalization/lean/.lake)" = "$primary/formalization/lean/.lake"
PYTHONDONTWRITEBYTECODE=1 python3 \
  labs/crouzeix_proof_reproduction/proof_evidence.py \
  preflight --route all
```

Expected: exit 0, all selected routes `ready`, and `missing_artifacts: []`.

Never run:

```text
lake update
lake --try-cache exe cache get Mathlib
mise run lean-cache
```

Missing cache state is a blocked prerequisite. Harp-owned `.olean` files may
be rebuilt. Run all Lean/Lake commands serially.

### Proof integrity

Reject active occurrences of:

```text
sorry
admit
axiom
opaque
unsafe
native_decide
implemented_by
```

Use the repository's active-source scanner, not a raw substring grep, as the
authoritative check. Allowed axioms are exactly, in canonical order:

```text
Classical.choice
Quot.sound
propext
```

### Evidence integrity

- Never rewrite a historical Jin or LS attempt directory.
- New receipt and bundle directories are create-only.
- A visible unreferenced candidate after a committed rename is recovery state,
  not permission to rerun or delete it.
- Jin upstream revision `565b6a3e0659b6e0785f783b016c3f6d9f171fa5`
  has `license_status: not-present-at-revision` and
  `redistribution_status: quotation-only`. Do not vendor the archive or source
  tree.
- Route manifests, receipts, reviews, and the six-row bundle must validate in
  both Python and Rust.
- A review is invalid after any bound byte changes.

### Git integrity

- Preserve unrelated dirty and untracked paths.
- Do not reset, clean, or delete unexplained state.
- Tests involving Git must use isolated configuration:

```sh
export GIT_CONFIG_NOSYSTEM=1
export GIT_CONFIG_GLOBAL="$(mktemp -d)/gitconfig"
```

- Refresh `docs/import-receipt.md` only after all other tracked bytes in that
  landing are staged.

## Advisory checkpoint at plan creation

Recompute this checkpoint before acting. At plan creation:

- local `master` is `8723d1c01bb6aae1e6eab133a3614c3056cdd002`;
- CPFR-083 and CPFR-084 are landed;
- the active Jin worktree is
  `.worktrees/cpfr-085-jin-certification` on
  `feat/cpfr-085-jin-certification`;
- its canonical `.lake` symlink is correct;
- a Spec review rejected the earlier 20-node Jin manifest with two Important
  findings: false consequence/provider edges and Rust/Python Git-locator
  syntax divergence;
- the interrupted repair added `Crouzeix.Jin.RationalBridge`, rewired the
  finite and Hilbert wrappers to `crouzeixConjecture`, and made Rust reject `#`
  inside a Git path;
- cached `.olean` files for the new bridge, final wrappers, and `CrouzeixJin`
  exist, but the route manifest still describes the old 20-node, 65-module
  closure;
- the actual repaired closure is 66 modules with digest
  `e0498425be2f30edc876eb5e802628169f80dedd54204b1861dfe62715eb5b7f`;
- 21 declaration-type files exist, including
  `jin-holomorphic-rational-bridge.txt` with byte SHA-256
  `217015e879994b4a3aa02ad10025d82d9adacd3d66a6a7793eb1f6290283f477`;
- unpublished Jin validation currently returns `authored / invalid` because
  `Crouzeix.Jin.RationalBridge` is not in the manifest;
- no Jin route receipt or proof review exists; and
- the three historical Jin receipt hashes remain:
  `9e0cf882a8f866a4897a46548c7e387fb0d07b7dee159b3c2260ef9ece1fb563`,
  `b25ca01a109ad93708d474feb60389e6544f5fb54fe64f557e24571cd7de58c8`,
  and `a8b6fdc904ed35de4a533b924bf8847c923f3a50e3cc19785ced4bee5f7da0f9`.

The primary checkout has user-owned untracked paths. Do not modify, stage, or
delete them.

---

### Task 0: Recover the authoritative execution state

**Files:**

- Read: `AGENTS.md`
- Read: `docs/superpowers/specs/2026-08-22-crouzeix-three-route-proof-completion-design.md`
- Read: `docs/superpowers/specs/2026-08-23-crouzeix-end-to-end-proof-goal-design.md`
- Read: `docs/workstream/crouzeix-proof-reproduction/tracker.org`
- Create: `docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv`
- Create: `labs/crouzeix_proof_reproduction/execution_ledger.py`
- Create: `labs/crouzeix_proof_reproduction/goal_validation.py`
- Create: `labs/crouzeix_proof_reproduction/tests/test_execution_ledger.py`
- Create: `labs/crouzeix_proof_reproduction/tests/test_goal_validation.py`
- Modify: `labs/crouzeix_proof_reproduction/proof_evidence.py`

- [ ] **Step 1: Create an isolated goal-control worktree**

From clean local `master`, verify `.worktrees` is ignored and create:

```sh
git check-ignore -q .worktrees
git worktree add .worktrees/crouzeix-goal-control \
  -b feat/crouzeix-goal-control master
```

Do not edit the dirty CPFR-085 worktree in this task.

- [ ] **Step 2: Inventory Git and processes without mutation**

Run from the primary checkout:

```sh
git status --short --branch
git worktree list --porcelain
git for-each-ref --format='%(refname:short) %(objectname)' refs/heads
ps -axo pid,ppid,etime,state,command | \
  rg 'harp|lean|lake|cargo|python3.*crouzeix' || true
```

Record each Crouzeix worktree's head, branch, dirty paths, untracked paths, and
active processes. Do not remove anything.

- [ ] **Step 3: Verify landed prerequisites**

```sh
git merge-base --is-ancestor 6fc3a7d master
git merge-base --is-ancestor 8723d1c master
PYTHONDONTWRITEBYTECODE=1 python3 \
  labs/crouzeix_proof_reproduction/proof_evidence.py \
  preflight --route all
```

Expected: both ancestry checks and preflight exit 0. If the exact commits were
superseded by equivalent later landings, record the containing commits and
prove the required files/tests are present before proceeding.

- [ ] **Step 4: Create the execution ledger**

The TSV header is exact:

```text
phase	ticket	state	branch	worktree	base_commit	candidate_commit	verifier	exit_code	evidence_digest	landing_commit	note
```

Add one CPFR-085 row with state `implementing` and
`note=recovered-existing-worktree`. Use percent-encoding for tabs, newlines,
and `%` inside field values. Add a parser test under
`labs/crouzeix_proof_reproduction/tests/test_execution_ledger.py` before adding
any production parser. The parser must reject duplicate phase transitions,
unknown states, unsafe paths, malformed commit IDs, and non-monotone state.

- [ ] **Step 5: Write the RED goal-footer tests**

Add `proof_evidence.py validate-goal`. Tests must require the fixed goal-plan
path and prove that it:

- reports `in-progress` and the earliest unfinished phase while any footer
  value is non-final;
- rejects missing, duplicate, unknown, reordered, or malformed footer keys;
- rejects `complete` unless the execution ledger has one monotone landed row
  for CPFR-085 through CPFR-091 and a passed Phase 8 plan-review row;
- verifies `master_landing` and `plan_revision` are existing commits in the
  current repository, that both are ancestors of `master`, and that the plan
  revision is not older than the program landing;
- invokes the pure route validators for all three `complete-local` routes;
- requires the mandatory six-row bundle and reconciled generated-reader
  receipt; and
- performs no write, Lean, Lake, Git mutation, or provider call.

The public result type is exact:

```python
@dataclass(frozen=True)
class GoalValidationResult:
    goal_status: str
    earliest_incomplete_phase: str | None
    verified_commits: tuple[str, ...]
    evidence_paths: tuple[str, ...]
```

Run and observe RED because `goal_validation.py` and `validate-goal` do not yet
exist:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_goal_validation -v
```

- [ ] **Step 6: Implement the read-only goal validator**

Use a bounded line parser for the final fenced YAML block; do not add a YAML
dependency. Reject files over 1 MiB and duplicate keys. Reuse
`route_validation.inspect_route`. Until Task 7 creates the public Python
local-bundle verifier, treat a required bundle as incomplete; after Task 7,
call that verifier rather than recreating its contract. Read Git identities
through a fixed hardened
`git rev-parse`/`git merge-base --is-ancestor` adapter with output caps and an
isolated Git configuration.

`validate-goal` prints canonical JSON. Before completion it exits 1 with
`status: incomplete`; after every invariant validates it exits 0 with
`status: complete`. Any malformed or contradictory state exits 1 with
`status: invalid`.

- [ ] **Step 7: Verify the goal-control change**

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_execution_ledger \
  labs.crouzeix_proof_reproduction.tests.test_goal_validation -v
PYTHONDONTWRITEBYTECODE=1 python3 \
  labs/crouzeix_proof_reproduction/tickets.py validate-tracker
PYTHONDONTWRITEBYTECODE=1 python3 \
  labs/crouzeix_proof_reproduction/proof_evidence.py validate-goal
git diff --check
```

Expected: unit tests and tracker validation exit 0. `validate-goal` exits 1
with `status: incomplete` and `earliest_incomplete_phase: cpfr-085`. Obtain a
focused Spec review of the state machine and a Standards review of the
read-only parser/Git boundary. Fix and re-review every Critical or Important
finding. Commit only the ledger/parser/test and goal validator/CLI as:

```text
chore(crouzeix): record proof goal recovery state
```

- [ ] **Step 8: Land goal control and recover the Jin branch**

Fast-forward `feat/crouzeix-goal-control` to local `master`. In the preserved
CPFR-085 worktree, capture the dirty path set and the incoming path set:

```sh
git status --short
git diff --name-only HEAD..master
git diff --name-only
git ls-files --others --exclude-standard
```

If an incoming path overlaps a dirty or untracked path, stop and preserve the
worktree for explicit reconciliation. Otherwise run:

```sh
git merge --ff-only master
```

Confirm the prior dirty Jin paths and their hashes remain present. Then update
only the CPFR-085 tracker subtree to `IMPLEMENTING` with branch, worktree, owner,
agent run, and model. Commit that tracker update separately before resuming
Task 1.

### Task 1: Finish the Jin provider and evidence-map repair

**Files:**

- Create: `formalization/lean/Crouzeix/Jin/RationalBridge.lean`
- Modify: `formalization/lean/CrouzeixConjecture/FinalTheorems.lean`
- Modify: `formalization/lean/CrouzeixConjecture/HilbertSpace.lean`
- Modify: `formalization/lean/CrouzeixConjecture/HilbertSpectralSet.lean`
- Modify: `labs/crouzeix_proof_reproduction/route_validation.py`
- Modify: `labs/crouzeix_proof_reproduction/schemas/route_manifest.schema.json`
- Modify: `crates/harp/src/sources/crouzeix/route.rs`
- Modify: `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/artifact-manifest.json`
- Create/modify: `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json`
- Create: `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/declaration-types/*.txt`
- Create/modify: `labs/crouzeix_proof_reproduction/tests/test_route_jin.py`
- Modify: `labs/crouzeix_proof_reproduction/tests/test_route_validation.py`

- [ ] **Step 1: Reproduce the two reviewed defects**

The Jin test must inspect active Lean text and assert:

```python
self.assertIn("import Crouzeix.Jin.RationalBridge", final_theorems)
self.assertRegex(
    final_theorems,
    r"jinFinalCrouzeixConjecture[\s\S]+?:=\s*crouzeixConjecture A p",
)
self.assertNotIn("jinFinalCrouzeixConjecture", hilbert_space)
self.assertNotIn("jinFinalCrouzeixConjecture", hilbert_spectral_set)
self.assertIn("jinHolomorphicCrouzeixRationalBound", final_theorems)
```

The Rust test must assert:

```rust
let locator = format!(
    "git:{}:Lean/Foo.lean#opaque#L1-L1",
    "a".repeat(40)
);
assert!(parse_source_locator(&locator).is_none());
```

Run the isolated tests against the pre-repair revision or revert only the
candidate hunk in a temporary copy. Record RED showing the provider assertions
or locator assertion fail for the reviewed reasons. Do not rewrite the dirty
worktree merely to recreate RED if the execution ledger already contains the
captured failing output.

- [ ] **Step 2: Complete the Lean provider repair**

The route-local bridge must have this public contract:

```lean
module

public import CrouzeixConjecture.HolomorphicConsequences

@[expose] public section

noncomputable section

open scoped Matrix Matrix.Norms.L2Operator

namespace CrouzeixConjecture

variable {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]

theorem jinHolomorphicCrouzeixRationalBound
    (A : SquareMatrix n) (r : RatFunc ℂ)
    (hfree : RationalPoleFreeOn r (numericalRange A)) :
    ‖rationalMatrixEval r A‖ ≤
      2 * maxRationalModulusOnNumericalRange A r :=
  holomorphicCrouzeixRationalBound A r hfree

end CrouzeixConjecture
```

`FinalTheorems.lean` must retain `module`, use public imports for
`Crouzeix.Jin.Terminal` and `Crouzeix.Jin.RationalBridge`, and retain
`@[expose] public section`. `jinFinalCrouzeixConjecture` remains only as a
compatibility alias proved by `crouzeixConjecture A p`. Both rational wrappers
must invoke `jinHolomorphicCrouzeixRationalBound`. `HilbertSpace.lean` and
`HilbertSpectralSet.lean` retain their `module`, public imports, and
`@[expose] public section`. Their providers must use:

```lean
fun d => crouzeixConjecture (n := Fin d)
```

and must not reference `jinFinalCrouzeixConjecture`. Preserve existing public
module/export syntax unless a compiler error proves it must change.

- [ ] **Step 3: Add correspondence classification to the route contract**

Add required `correspondence_kind` to Python, JSON Schema, and Rust with this
closed enum:

```text
direct-source
compatibility-port
structural-refactor
derived-extraction
shared-foundation
reused-route
```

Enforce the exact cross-field matrix:

| `provenance_kind` | allowed `correspondence_kind` |
|---|---|
| `source` | `direct-source`, `compatibility-port`, `structural-refactor` |
| `derived` | `derived-extraction` |
| `shared-foundation` | `shared-foundation` |
| `reused-route` | `reused-route` |

Add table-driven positive and negative tests in both implementations. Unknown
or mismatched values must fail before filesystem access.

- [ ] **Step 4: Finish Git-locator parity**

Rust `parse_source_locator` must reject `#` in the parsed path before calling
the general safe-path helper. Keep the Python regex and JSON Schema path
alphabet unchanged. Add the malformed locator to shared fixture cases so both
implementations reject it.

- [ ] **Step 5: Produce the exact 21-node Jin manifest**

The final manifest has:

```text
nodes: 21
source nodes: 20
derived nodes: 1
distinct node modules: 21
shared-foundation modules: 45
active closure modules: 66
module_closure_sha256: e0498425be2f30edc876eb5e802628169f80dedd54204b1861dfe62715eb5b7f
consequence declarations: 3
```

Add this node immediately before the finite rational consequence:

```json
{
  "node_id": "jin-holomorphic-rational-bridge",
  "role": "load-bearing",
  "declaration": "CrouzeixConjecture.jinHolomorphicCrouzeixRationalBound",
  "module_path": "Crouzeix/Jin/RationalBridge.lean",
  "dependency_ids": ["jin-fixed-outer-domain-convergence"],
  "provenance_kind": "source",
  "correspondence_kind": "structural-refactor",
  "source_locator": "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:Lean/CrouzeixConjecture/HolomorphicConsequences.lean#L82-L96"
}
```

Fill the remaining digest and path fields from the verified upstream file and
compiler-produced type artifact. Do not copy upstream source bytes. Register
`Lean/CrouzeixConjecture/HolomorphicConsequences.lean` exactly once in the
artifact manifest with byte count, SHA-256, Git locator, and
`not-present-at-revision`.

The changed DAG edges are exact:

```text
jin-holomorphic-rational-bridge <- jin-fixed-outer-domain-convergence
jin-finite-rational-consequence <- jin-holomorphic-rational-bridge
jin-hilbert-polynomial-core <- jin-numerical-range-convexity
jin-hilbert-polynomial-consequence <- jin-terminal-crouzeix, jin-hilbert-polynomial-core
jin-hilbert-spectral-set-core <- jin-hilbert-polynomial-core
jin-hilbert-spectral-set-consequence <- jin-terminal-crouzeix, jin-hilbert-spectral-set-core
```

Classify the two `_of_mainTheorem` core nodes as `structural-refactor`; classify
the polynomial specialization as `derived-extraction`; classify direct ports
and compatibility edits according to the pinned research map.

- [ ] **Step 6: Generate the bridge declaration type from the shared cache**

Run serially from `formalization/lean`:

```sh
printf '%s\n' \
  'import Crouzeix.Jin.RationalBridge' \
  '#check CrouzeixConjecture.jinHolomorphicCrouzeixRationalBound' | \
env \
  ELAN_HOME=/private/tmp/harp-mathematical-foundations-elan \
  ELAN_TOOLCHAIN=leanprover/lean4:v4.32.1 \
  PATH="/private/tmp/harp-mathematical-foundations-elan/toolchains/leanprover--lean4---v4.32.1/bin:/usr/bin:/bin" \
  lake env lean --stdin
```

Write the exact compiler output to
`declaration-types/jin-holomorphic-rational-bridge.txt`; compute its normalized
statement digest with `route_validation.normalized_type_sha256`.

- [ ] **Step 7: Run the Jin repair verifier**

```sh
scripts/check_lean_library.sh CrouzeixJin
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_route_jin \
  labs.crouzeix_proof_reproduction.tests.test_route_validation \
  labs.crouzeix_proof_reproduction.tests.test_proof_evidence -v
cargo test -p harp sources::crouzeix::route --lib -- --test-threads=1
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_formal_target.FormalTargetProductionLockTests -v
cargo fmt --all -- --check
git diff --check
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  validate --route jin --allow-unpublished
```

Expected: all tests and the route build exit 0; the last command exits 1 with
`claim_level: mapped`, `status: incomplete`, and reason `receipt or review is
unpublished`. Verify no review or route receipt path exists.

- [ ] **Step 8: Commit the frozen Jin candidate**

Stage only the route contract/tests, Jin Lean repair, artifact registry, route
manifest, declaration types, execution-ledger row, and CPFR-085 tracker
subtree. Commit as:

```text
feat(crouzeix): complete Jin route correspondence
```

Record the commit, tree, manifest digest, closure digest, and historical
receipt hashes. This is the Spec-review candidate.

### Task 2: Prepare the Jin mathematical review bundle

**Files:**

- Read only: the Task 1 candidate, pinned Jin extraction, and research map
- Record: CPFR-085 tracker subtree and execution ledger

- [ ] **Step 1: Freeze provisional review inputs**

```sh
git status --short
git rev-parse HEAD
git rev-parse HEAD^{tree}
shasum -a 256 \
  labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json \
  labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/artifact-manifest.json
```

Expected: no uncommitted phase-owned file and no receipt/review path. Record
these provisional hashes so Task 4 can prove that Task 3 changed only the
publisher implementation and its tests.

- [ ] **Step 2: Build the reviewer packet without dispatching it**

Assemble the exact paths, commit/tree IDs, manifest and artifact digests,
pinned archive identity, and `Jin mathematical Spec reviewer` prompt. Require
the future review to provide a node-by-node table for all 21 nodes and exact
answers for provider edges, source classification, remote evidence binding,
claim ceiling, and publication boundary. Do not dispatch yet: Task 3 must add
the producer code before the commit bound by the eventual receipt and review
is frozen.

- [ ] **Step 3: Run a controller source/DAG precheck**

Independently compare all 21 nodes to their local declarations and pinned
upstream excerpts. Confirm the rational and Hilbert edges are actual Lean
provider edges. Any discrepancy returns to Task 1 with a failing regression
test. This precheck is not the independent Spec review.

- [ ] **Step 4: Record the prepared boundary**

Record the provisional commit/tree, manifest digest, source-archive digest,
and packet path in the tracker and execution ledger with state
`locally-verified`. Do not record `spec-approved`.

### Task 3: Implement the generic create-only route publisher

**Files:**

- Create: `labs/crouzeix_proof_reproduction/route_publication.py`
- Create: `labs/crouzeix_proof_reproduction/tests/test_route_publication.py`
- Modify: `labs/crouzeix_proof_reproduction/proof_evidence.py`
- Modify: `labs/crouzeix_proof_reproduction/route_validation.py`
- Read/verify: `crates/harp/src/sources/crouzeix/route.rs`

- [ ] **Step 1: Write producer/consumer RED tests**

Write this exact public-interface test before creating the module:

```python
def test_public_route_publisher_signature_has_no_evidence_injection() -> None:
    signature = inspect.signature(route_publication.publish_route_receipt)
    self.assertEqual(
        tuple(signature.parameters),
        ("repository_root", "route_id", "executor"),
    )
    self.assertEqual(
        tuple(route_publication.RoutePublication.__dataclass_fields__),
        ("route_id", "artifact_root", "receipt_path", "receipt_sha256"),
    )
```

The implementation must expose that dataclass and function. `executor` is
keyword-only and exists only for deterministic fake-command tests; production
CLI calls omit it.

Also expose the create-only review publisher:

```python
@dataclass(frozen=True)
class ReviewPublication:
    route_id: str
    review_path: Path
    review_sha256: str

def publish_route_review(
    repository_root: Path,
    route_id: str,
) -> ReviewPublication:
    return _publish_validated_review_candidate(repository_root, route_id)
```

The fixed candidate paths are `.build/crouzeix-route-review-candidates/jin.json`,
`.build/crouzeix-route-review-candidates/lorist-schwenninger.json`, and
`.build/crouzeix-route-review-candidates/harp.json`. The publisher selects one
from the closed route enum. The private implementation
reads the candidate through a bounded no-follow path, validates it against the
published route receipt and manifest contract, creates the final
the corresponding fixed final review path without replacement, then reopens and revalidates
the published bytes.

Add CLI syntax:

```text
proof_evidence.py publish-route --route jin|lorist-schwenninger|harp
proof_evidence.py publish-review --route jin|lorist-schwenninger|harp
```

Tests must prove:

- the public API accepts no caller-supplied stdout, stderr, axiom, provider,
  closure, declaration, toolchain, or cache evidence;
- preflight and an unpublished valid manifest are required before execution;
- one exact route target runs once;
- every terminal/load-bearing declaration is audited;
- command, stdout, stderr, axiom, provider, Mathlib, local closure, and type
  artifacts are hash-bound;
- staged output is validated by `validate_route_bundle` before rename;
- destination and staging paths reject symlinks, hardlinks, traversal, unknown
  members, and ancestor swaps;
- destination publication uses no-replace rename and recursive fsync;
- failure before rename cleans only the owned stage;
- failure after rename returns a typed committed-publication error and
  preserves the visible candidate;
- concurrent publishers cannot overwrite or interleave;
- review candidates with the wrong route, commit, tree, manifest, terminal
  type, self-digest, or an unresolved Critical/Important finding are rejected;
- review publication rejects an absent or invalid route receipt and never
  overwrites an existing review;
- source or dependency cache mutation during execution prevents publication;
- forbidden axioms or provider imports prevent publication; and
- the real validator consumes the untouched producer output.

Define and test this candidate-validation seam in `route_validation.py`:

```python
def validate_route_receipt_candidate(
    repo_root: Path,
    manifest_path: Path,
    candidate_root: Path,
) -> RouteValidationResult:
    return _validate_route_receipt_candidate(
        repo_root.resolve(strict=True), manifest_path, candidate_root
    )
```

`_validate_route_receipt_candidate` is implemented in the same task and reads
the manifest plus untouched staged members from `candidate_root`; it applies
the complete receipt contract without requiring the candidate to appear at the
manifest's final publication path. It rejects a candidate outside the
publisher's pinned staging parent. The public function is read-only. Production
publication must feed the exact staged bytes to this function before rename
and the exact published bytes to it after rename.

Run the focused test and observe RED because the module/command does not exist:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_route_publication -v
```

- [ ] **Step 2: Reuse hardened publication primitives**

Reuse the descriptor-relative, bounded, no-follow, no-replace, fsync, process
group, output-cap, and lock patterns in `ls_receipts.py` and
`local_formalization_evidence.py`. Do not expose their private objects as a new
public API and do not broadly refactor either module before publication.

The route publisher's fixed command is derived from the manifest:

```python
argv = ["scripts/check_lean_library.sh", manifest.build_target]
```

The fixed environment is the shared Lean environment in this plan. The timeout
is 3,600 seconds; stdout and stderr caps are 1 MiB each. The destination is
the route-specific path obtained from the closed mapping
`jin -> evidence/crouzeix_conjecture/routes/jin`,
`lorist-schwenninger -> evidence/crouzeix_conjecture/routes/lorist-schwenninger`,
and `harp -> evidence/crouzeix_conjecture/routes/harp`.

- [ ] **Step 3: Produce the exact route bundle**

The create-only directory contains only:

```text
receipt.json
build/command.json
build/stdout.log
build/stderr.log
audit/axioms.json
audit/provider.json
```

The receipt conforms to
`crouzeix-route-proof-receipt/v1`. Its `candidate_commit` and `candidate_tree`
refer to the clean frozen proof/manifest commit, not the later publication
commit. `receipt_sha256` is the self-digest computed with that field null.

The publisher returns the receipt digest but does not write the route
manifest. The controller inserts the digest with `apply_patch`, making an
unreferenced visible candidate an explicit recoverable state.

- [ ] **Step 4: Add CLI output and recovery errors**

On success, tests assert this canonical JSON structure:

```python
self.assertEqual(payload["artifact_root"], "evidence/crouzeix_conjecture/routes/jin")
self.assertEqual(payload["receipt_path"], "evidence/crouzeix_conjecture/routes/jin/receipt.json")
self.assertRegex(payload["receipt_sha256"], r"^[0-9a-f]{64}$")
self.assertEqual(payload["route_id"], "jin")
self.assertEqual(payload["schema_version"], "crouzeix-route-publication/v1")
self.assertEqual(payload["status"], "published-unreferenced")
```

A committed-publication error must print the visible path and digest and exit
nonzero without cleanup.

- [ ] **Step 5: Verify the publisher**

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_route_publication \
  labs.crouzeix_proof_reproduction.tests.test_route_validation -v
cargo test -p harp sources::crouzeix::route --lib -- --test-threads=1
cargo fmt --all -- --check
git diff --check
```

Expected: all tests pass. Commit as:

```text
feat(crouzeix): add create-only route publisher
```

Before committing, obtain a Spec review of the producer/consumer and recovery
contract, then a Standards review of its filesystem/process safety. Fix and
re-review every Critical or Important finding.

### Task 4: Review, execute, publish, and land Jin

**Files:**

- Create: `evidence/crouzeix_conjecture/routes/jin/*`
- Create: `evidence/crouzeix_conjecture/reviews/jin.json`
- Modify: Jin `route-manifest.json` digest fields only
- Modify: CPFR-085 tracker subtree and execution ledger
- Modify last: `docs/import-receipt.md`

- [ ] **Step 1: Freeze and review the final Spec candidate**

Require clean phase-owned files after the Task 3 publisher commit. Prove that
the manifest, Lean route sources, declaration types, and upstream registry are
byte-identical to the provisional Task 2 packet; otherwise return to Task 1.
Record current `HEAD` and `HEAD^{tree}`, rerun preflight, and dispatch the
independent reviewer with the appendix prompt.

For each Critical or Important finding, add a failing test, implement one
minimal repair, rerun Task 1 Step 7, commit, rebuild the packet, and request a
fresh full Spec review. Do not execute the publisher until the final verdict is
`PASS`. Record reviewer identity, model, run ID, reviewed commit/tree, manifest
digest, findings, and verdict in the tracker and execution ledger. Keep the raw
review outside the authoritative `reviews/jin.json` path until the execution
receipt exists.

- [ ] **Step 2: Publish the Jin receipt once**

```sh
PYTHONDONTWRITEBYTECODE=1 python3 \
  labs/crouzeix_proof_reproduction/proof_evidence.py \
  publish-route --route jin
```

Expected: exit 0 and `status: published-unreferenced`. If the destination
exists, validate it and recover; never rerun blindly. Insert the returned
digest into `receipt_sha256` with `apply_patch`.

- [ ] **Step 3: Publish the bound proof review**

Create `.build/crouzeix-route-review-candidates/jin.json` from the approved Spec
review using the exact `crouzeix-proof-review/v1` fields. Bind the reviewed
commit/tree, manifest contract digest, terminal type digest, reviewer identity,
model, run ID, and all findings/dispositions. Set the self-digest, then run:

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  publish-review --route jin
```

Insert the returned digest into `review_sha256` with `apply_patch`. Preserve a
visible review if a post-publication durability check fails.

- [ ] **Step 4: Validate the complete route independently**

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py validate --route jin
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_route_jin \
  labs.crouzeix_proof_reproduction.tests.test_route_publication \
  labs.crouzeix_proof_reproduction.tests.test_route_validation -v
cargo test -p harp sources::crouzeix::route --lib -- --test-threads=1
```

Expected: validator exit 0 with `complete-local`; all tests pass. Recompute the
three historical receipt hashes and require exact equality to the advisory
checkpoint.

- [ ] **Step 5: Run independent Standards review**

Use the appendix prompt. Critical and Important findings block landing. A
mathematics, mapping, or evidence-identity repair invalidates the Spec review
and receipt; return to Task 2. A code-quality-only repair reruns affected gates
and Standards review.

- [ ] **Step 6: Run the Jin release gate and commit**

```sh
scripts/check_lean_library.sh CrouzeixJin
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  discover -s labs/crouzeix_proof_reproduction/tests -p 'test_*.py' -v
cargo test -p harp sources::crouzeix --lib -- --test-threads=1
cargo fmt --all -- --check
git diff --check
```

Commit evidence, manifest digests, tracker evidence, and execution-ledger rows
as focused commits. Stage all intended files except `docs/import-receipt.md`,
run `cargo run -q -p harp -- repository verify`, patch the reported payload
digest, stage the receipt, and rerun repository verification.

- [ ] **Step 7: Land Jin locally**

From the primary checkout:

```sh
git status --short --branch
git merge --ff-only feat/cpfr-085-jin-certification
python3 labs/crouzeix_proof_reproduction/proof_evidence.py validate --route jin
```

Expected: fast-forward succeeds and validation is `complete-local`. Close Kata
`harp#gzah` and mark CPFR-085 `DONE` with the landing commit. Preserve the
worktree until the final audit.

### Task 5: Certify and land the six-node Lorist--Schwenninger route

**Files:**

- Modify: `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json`
- Modify: `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/library-inventory.json`
- Create: `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/artifact-manifest.json`
- Create: new immutable attempts under `proof-slices/*/attempt-*`
- Create: LS `route-manifest.json` and declaration types
- Modify: `labs/crouzeix_proof_reproduction/route_validation.py`
- Modify: `labs/crouzeix_proof_reproduction/schemas/route_manifest.schema.json`
- Modify: `crates/harp/src/sources/crouzeix/route.rs`
- Create: `labs/crouzeix_proof_reproduction/tests/test_route_lorist_schwenninger.py`
- Create: `evidence/crouzeix_conjecture/routes/lorist-schwenninger/*`
- Create: `evidence/crouzeix_conjecture/reviews/lorist-schwenninger.json`
- Modify: CPFR-086 tracker subtree and execution ledger

- [ ] **Step 1: Create the phase worktree from landed master**

```sh
git worktree add .worktrees/cpfr-086-ls-certification \
  -b feat/cpfr-086-ls-certification master
git_common_dir=$(git rev-parse --path-format=absolute --git-common-dir)
primary_root=${git_common_dir%/.git}
ln -s "$primary_root/formalization/lean/.lake" \
  .worktrees/cpfr-086-ls-certification/formalization/lean/.lake
```

Require `git_common_dir` to end in `/.git` and `primary_root` to be the current
repository's primary checkout before creating the link. Verify preflight before
tests. Claim Kata `harp#2eb2`.

- [ ] **Step 2: Add the pinned arXiv locator contract with RED/GREEN parity**

Extend `source_identities` to accept either `sha256:` followed by 64 lowercase
hex characters or the exact versioned arXiv identity shape below. Extend the
source locator union with a Python, JSON Schema, and Rust equivalent
of this exact regular expression:

```text
^arxiv:[0-9]{4}\.[0-9]{4,5}v[1-9][0-9]*:[A-Za-z0-9._/-]+#L[1-9][0-9]*-L[1-9][0-9]*$
```

For this route the identity is exactly `arxiv:2608.03841v1`, the source archive
SHA-256 is
`b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9`,
and `CrouzeixConjecturev2.tex` has 18,783 bytes and SHA-256
`20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a`.

The new LS artifact manifest has exact top-level fields:

```json
{
  "schema_version": "crouzeix-arxiv-artifact-manifest/v1",
  "source_id": "LS-ARXIV-V1",
  "source_identity": "arxiv:2608.03841v1",
  "archive_sha256": "b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9",
  "manuscript_path": "CrouzeixConjecturev2.tex",
  "manuscript_bytes": 18783,
  "manuscript_sha256": "20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a",
  "line_count": 281
}
```

The validator cross-checks the archive and TeX records against
`evidence/crouzeix_conjecture/source_manifest.tsv`; a test mutation to
`line_count: 0` must fail.
Every LS node uses the same registered manuscript file digest plus its own
line-range excerpt digest. Python, schema, and Rust reject wrong arXiv version,
wrong filename, unregistered identity, archive/file mismatch, zero or short
line count, bad excerpt digest, traversal, extra `#`, and malformed spans.
The LS route manifest's sorted source identities are exact:

```text
arxiv:2608.03841v1
sha256:20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a
sha256:b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9
```

Acquire bytes only into a fresh temporary directory when recomputation is
needed:

```sh
ls_source_tmp=$(mktemp -d /private/tmp/harp-ls-source.XXXXXX)
curl -fsSL 'https://export.arxiv.org/e-print/2608.03841v1' \
  -o "$ls_source_tmp/source.tar.gz"
test "$(shasum -a 256 "$ls_source_tmp/source.tar.gz" | awk '{print $1}')" = \
  b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9
tar -xzf "$ls_source_tmp/source.tar.gz" -C "$ls_source_tmp"
test "$(shasum -a 256 "$ls_source_tmp/CrouzeixConjecturev2.tex" | awk '{print $1}')" = \
  20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a
```

Do not copy the archive or TeX into the repository. Preserve the temporary
directory through Spec review so the reviewer can independently recompute
spans.

- [ ] **Step 3: Write RED graph and receipt tests**

Require exactly these six ordered nodes:

```text
ls-equation-one-terminal-bound
ls-power-recurrence
ls-scalar-contradiction
ls-perturbation-lemma
ls-double-layer-realization
ls-terminal-crouzeix
```

Add tests for the current declarations, exact source spans,
`correspondence_kind`, shared aggregate identity, ordered allowed axioms,
atomic graph/inventory promotion, and rejection of partial, mixed v1/v2, or
cross-provider receipts. Observe RED while four rows remain blocked and no
route manifest exists.

- [ ] **Step 4: Materialize the source-faithful LS manifest**

Map the six declarations named by `ls_contract.NODES`, their exact modules,
dependency IDs, normalized type artifacts, and `arxiv:2608.03841v1` source
locators. Classify direct ports versus structural compatibility refactors.
Keep v1 attempts immutable.

- [ ] **Step 5: Run the dedicated candidate execution once**

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  preflight --route lorist-schwenninger
```

Then invoke `ls_receipts.publish_ls_receipts` through a new closed CLI
subcommand added to `proof_evidence.py`:

```text
proof_evidence.py publish-ls
```

The CLI accepts no command, environment, row, output, or path overrides. It
loads the canonical graph and formal-target root. Under the current state, the
expected next attempts are `attempt-002`, `attempt-001`, `attempt-002`,
`attempt-001`, `attempt-001`, and `attempt-001` in graph order. If any exists
at execution time, recover it and compute the next monotone attempt name rather
than overwrite it.

- [ ] **Step 6: Validate candidates and obtain Spec approval**

Validate all six attempts independently before graph mutation. The Spec review
must check `T = op(P)`, `Q = M_h`, Equation 1, operator recurrence, scalar
contradiction, perturbation lemma, double-layer realization, and inner/outer
limits against the pinned TeX. Repair and re-review every Critical or Important
finding.

- [ ] **Step 7: Promote graph and inventory atomically**

Create focused `ls_promotion.py` and first write this exact interface test:

```python
def test_promotion_api_accepts_only_repository_and_candidate_map() -> None:
    signature = inspect.signature(ls_promotion.promote_six_node_route)
    self.assertEqual(
        tuple(signature.parameters),
        ("repository_root", "candidate_receipts"),
    )
```

The production function returns `None` on success and raises a typed
`protocol.ValidationError` or committed-publication error otherwise. It
validates all six candidates and the unchanged source files before writing
one complete graph and inventory pair through staged no-replace or
compare-and-replace publication. It never leaves one authoritative file at the
new generation while the other remains old. Add crash/race tests around every
write boundary.

- [ ] **Step 8: Publish and validate the LS route**

After the promoted graph, unpublished route manifest, and Spec review are
frozen, use `publish-route --route lorist-schwenninger`, bind its digest, write
the review candidate at
`.build/crouzeix-route-review-candidates/lorist-schwenninger.json`, run
`publish-review --route lorist-schwenninger`, bind its digest, and run:

```sh
scripts/check_lean_library.sh CrouzeixLoristSchwenninger
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  validate --route lorist-schwenninger
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_ls_receipts \
  labs.crouzeix_proof_reproduction.tests.test_ls_validation \
  labs.crouzeix_proof_reproduction.tests.test_route_lorist_schwenninger \
  labs.crouzeix_proof_reproduction.tests.test_route_publication -v
cargo test -p harp sources::crouzeix --lib -- --test-threads=1
```

Expected: route `complete-local`, six current passed nodes, and all gates exit
0. Obtain Standards approval, commit focused concerns, refresh the import
receipt last, fast-forward local `master`, close `harp#2eb2`, and mark CPFR-086
`DONE`.

### Task 6: Certify and land the Harp-derived route

**Files:**

- Create: `labs/crouzeix_proof_reproduction/formal_targets/harp/route-manifest.json`
- Create: Harp declaration-type artifacts
- Create: `labs/crouzeix_proof_reproduction/tests/test_route_harp.py`
- Create: `evidence/crouzeix_conjecture/routes/harp/*`
- Create: `evidence/crouzeix_conjecture/reviews/harp.json`
- Modify: CPFR-087 tracker subtree and execution ledger

- [ ] **Step 1: Create and preflight the Harp phase worktree**

Create `feat/cpfr-087-harp-certification` from the LS landing, link the canonical
cache, run `preflight --route harp`, and claim Kata `harp#2qy3`.

- [ ] **Step 2: Write the RED derivation/reuse tests**

Require nodes for:

```text
positive cubature
finite-measure cubature
finite atomic matrix moments
counting-L2 witness and dimension bound
finite-horizon recurrence
operator recurrence
perturbation endpoint
double-layer application
inner limit
outer limit
terminal theorem
closed-range consequence
```

Require every active `Crouzeix.LoristSchwenninger.*` module to match one of the
eleven allowed support modules and an explicit `reused-route` node or declared
reuse edge. Reject a twelfth module, an LS terminal/provider, any Jin module, a
missing reuse, or a claim that the route is wholly mathematically independent.

- [ ] **Step 3: Write the complete derived manifest**

Use `claim_kind: derived`. Classify Harp-owned nodes as `derived-extraction` or
the plan-approved derived classification and LS dependencies as `reused-route`.
Bind exact declaration types, module paths, dependency IDs, closure, and the LS
route manifest/receipt identities being reused. `source_identities` remains
empty because source ownership is represented through the bound LS route.

- [ ] **Step 4: Validate and review before publication**

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  validate --route harp --allow-unpublished
```

Expected: exit 1, `mapped / incomplete`. The Spec reviewer checks cubature,
moment preservation, finite counting-L2 realization, dimension bound,
finite-horizon recurrence, perturbation endpoint, double-layer instantiation,
limits, theorem assembly, and the novelty/reuse boundary.

- [ ] **Step 5: Publish and validate**

After Spec approval, use `publish-route --route harp`, bind the receipt, write
the review candidate at `.build/crouzeix-route-review-candidates/harp.json`, run
`publish-review --route harp`, bind its digest, and run:

```sh
scripts/check_lean_library.sh CrouzeixHarp
python3 labs/crouzeix_proof_reproduction/proof_evidence.py validate --route harp
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_route_harp \
  labs.crouzeix_proof_reproduction.tests.test_route_publication \
  labs.crouzeix_proof_reproduction.tests.test_route_validation -v
cargo test -p harp sources::crouzeix --lib -- --test-threads=1
```

Expected: route `complete-local`. Obtain Standards approval, commit focused
concerns, refresh the import receipt last, fast-forward local `master`, close
`harp#2qy3`, and mark CPFR-087 `DONE`.

### Task 7: Publish and require the six-row local bundle

**Files:**

- Modify: `labs/crouzeix_proof_reproduction/local_formalization_evidence.py`
- Create: `labs/crouzeix_proof_reproduction/local_formalization_validation.py`
- Modify: `labs/crouzeix_proof_reproduction/tests/test_local_formalization_evidence.py`
- Modify: `crates/harp/src/sources/crouzeix.rs`
- Modify: Rust Crouzeix source tests in the same module
- Create: `evidence/crouzeix_conjecture/local_formalization/*`
- Modify: CPFR-088 tracker subtree and execution ledger

- [ ] **Step 1: Create the bundle worktree and write RED roster tests**

Create `feat/cpfr-088-six-route-bundle` from the Harp landing, link the cache,
and claim Kata `harp#ga15`. Extend `FORMALIZATIONS` from four to exactly these
six sorted IDs:

```text
harp-closed-numerical-range
harp-main-theorem
jin-closed-numerical-range
jin-main-theorem
ls-closed-numerical-range
ls-main-theorem
```

The Jin main declaration is `CrouzeixConjecture.crouzeixConjecture`; the Jin
closed-range declaration is the registered Hilbert spectral-set consequence.
Write Python and Rust RED tests for exact roster, route-manifest/review/receipt
bindings, missing members, route substitution, stale digests, symlinks,
hardlinks, aliases, and partial materialization.

- [ ] **Step 2: Add the read-only Python bundle verifier**

Write this exact public-interface test before creating the implementation:

```python
def test_public_bundle_validator_is_read_only_by_construction() -> None:
    signature = inspect.signature(
        local_formalization_validation.validate_local_formalization_bundle
    )
    self.assertEqual(tuple(signature.parameters), ("repository_root",))
    self.assertEqual(
        tuple(
            local_formalization_validation.LocalBundleValidationResult
            .__dataclass_fields__
        ),
        ("status", "formalization_ids", "route_ids", "manifest_sha256"),
    )
```

The implementation exposes the named dataclass and function. The function has
no executor and is strictly read-only. It mirrors the Rust manifest, roster,
route, path, digest, axiom, provider, and terminal-receipt checks. Add concrete
fixture trees consumed by both language suites and parity mutations for every
rejected field. Wire `goal_validation.py` to this function once the bundle is
required.

- [ ] **Step 3: Extend the publisher and verifier in lockstep**

Change `publish_local_formalization_evidence` to validate all three
`complete-local` routes before any command. Snapshot all route manifests,
receipts, reviews, Lean sources, toolchain, Lake contract, and required Mathlib
artifacts. Run one fixed `scripts/check_lean_library.sh Crouzeix` build, six
axiom audits, and three provider audits. Keep the public API free of precomputed
evidence arguments.

- [ ] **Step 4: Add the closed CLI**

Add:

```text
proof_evidence.py publish-local
```

It takes no path, command, environment, output, or row override. It calls the
publisher at the canonical repository root and prints canonical publication
metadata.

- [ ] **Step 5: Run RED/GREEN tests before real publication**

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_local_formalization_evidence -v
cargo test -p harp sources::crouzeix --lib -- --test-threads=1
```

Expected GREEN after implementation. The tests must consume untouched producer
output in the real validators.

- [ ] **Step 6: Publish once and enable mandatory enforcement atomically**

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  preflight --route all
python3 labs/crouzeix_proof_reproduction/proof_evidence.py publish-local
```

Validate the visible bundle, then set the Rust mandatory-enforcement constant
to `true` in the same commit that adds the genuine bundle. Prove that deleting
each member in a temporary fixture makes verification fail.

- [ ] **Step 7: Review, verify, and land**

Run Spec review of the six claims and their route identities, then Standards
review of publication safety. Run:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_local_formalization_evidence -v
cargo test -p harp sources::crouzeix --lib -- --test-threads=1
cargo run -q -p harp -- sources verify
```

Commit the publisher/verifier and genuine bundle together, refresh import
receipt last, fast-forward local `master`, close `harp#ga15`, and mark CPFR-088
`DONE`.

### Task 8: Reconcile canonical Crouzeix prose and Atlas readers

**Files:**

- Modify: `knowledge/crouzeix_conjecture/crouzeix_conjecture_index.md`
- Modify: `knowledge/crouzeix_conjecture/03_jin_proof_spine.md`
- Modify: `knowledge/crouzeix_conjecture/05_lorist_schwenninger_proof.md`
- Modify: `knowledge/crouzeix_conjecture/06_proof_comparison.md`
- Modify: `knowledge/crouzeix_conjecture/07_jin_lean_verification.md`
- Modify: `knowledge/crouzeix_conjecture/09_status_and_critical_assessment.md`
- Modify: `knowledge/crouzeix_conjecture/10_jin_proof_editorial.md`
- Modify: `knowledge/crouzeix_conjecture/claim_evidence_ledger.md`
- Modify: `knowledge/crouzeix_conjecture/source_registry.md`
- Modify: `evidence/crouzeix_conjecture/PROVENANCE.md`
- Modify: `labs/crouzeix_proof_reproduction/README.md`
- Modify: `crates/harp/tests/crouzeix_conjecture_knowledge_packet.rs`
- Regenerate together: `atlas/src/content/generated/corpus.json`
- Regenerate together: `atlas/dist/harp-atlas.html`
- Regenerate together: `atlas/dist/harp-atlas.receipt.json`
- Modify: CPFR-089 tracker subtree and execution ledger

- [ ] **Step 1: Create the reader worktree and RED claim tests**

Create `docs/cpfr-089-crouzeix-proof-status` from the bundle landing and claim
Kata `harp#bjpn`. Add tests requiring all three route IDs, terminal
declarations, `complete-local` status, route receipt/review paths, six bundle
rows, the finite-dimensional theorem, scalar rational consequence, and Harp's
eleven-module LS reuse boundary. Add assertions rejecting stale text that says
the LS route remains blocked or that Harp is fully independent.

- [ ] **Step 2: Update prose only from validated evidence**

Every completion statement must link to a route manifest, route receipt, proof
review, or six-row bundle record. Keep these distinctions explicit:

- upstream Jin clean-room build history versus Harp-local route certification;
- source-faithful Jin/LS routes versus the Harp-derived route;
- finite polynomial theorem versus scalar rational/Hilbert consequences; and
- local mechanized verification versus publication or author endorsement.

- [ ] **Step 3: Regenerate reader outputs**

```sh
cargo test -p harp --test crouzeix_conjecture_knowledge_packet
cargo run -p harp -- build
cd atlas
corepack pnpm run lint
corepack pnpm run typecheck
corepack pnpm run test
corepack pnpm run build
node scripts/export-static.mjs
node tests/static-export.test.mjs
```

Expected: all commands exit 0; the three generated Atlas artifacts change
together or remain byte-identical together.

- [ ] **Step 4: Review and land**

Spec review checks mathematical and evidence wording. Standards review checks
canonical ownership, locators, generated-file parity, and reader navigation.
Commit canonical prose/tests and generated outputs together, refresh import
receipt last, fast-forward local `master`, close `harp#bjpn`, and mark CPFR-089
`DONE`.

### Task 9: Run the frozen program review and release gate

**Files:**

- Modify only files required by reviewed findings
- Modify: CPFR-090 tracker subtree and execution ledger
- Modify last: `docs/import-receipt.md`

- [ ] **Step 1: Create a finalization worktree**

Create `review/cpfr-090-three-route-finalization` from the Phase 1--5 landed
`master`, link the canonical cache, claim Kata `harp#jzg9`, and record the base
commit/tree. No other phase worktree may run Lean concurrently.

- [ ] **Step 2: Run the final independent Spec review**

Use the `Program Spec reviewer` appendix prompt. It must review all three route
manifests, receipts, reviews, the six-row bundle, theorem types, actual provider
closures, source/derivation correspondence, claim ceiling, and reader claims.
Repair and repeat until PASS.

- [ ] **Step 3: Run the final independent Standards review**

Use the `Program Standards reviewer` prompt. It must attack path safety,
symlink/hardlink/alias cases, bounded reads/output, create-only publication,
concurrent writers, partial publication recovery, Python/Rust parity, cache
discipline, and maintainability. Repair and repeat until PASS. Any repair that
changes a bound mathematical/evidence byte returns to Step 2.

- [ ] **Step 4: Run the exact program verifier**

Run serially in this order:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 \
  labs/crouzeix_proof_reproduction/proof_evidence.py preflight --route all
python3 labs/crouzeix_proof_reproduction/proof_evidence.py validate --route all
scripts/check_lean_library.sh CrouzeixJin
scripts/check_lean_library.sh CrouzeixLoristSchwenninger
scripts/check_lean_library.sh CrouzeixHarp
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run verify
```

`mise run verify` already runs Atlas checks, Rust formatting/lints/workspace
tests, all Crouzeix Python tests, `sources verify`, `lean-all`, and repository
verification. Do not invoke those components a second time unless the first
output is missing or a later edit invalidates it. Expected: every command exits
0. Record subtask test counts, route job counts, timing, and artifact digests.
Verify no receipt, review, or generated artifact changed after its bound
review.

- [ ] **Step 5: Refresh the payload digest last**

Stage every intended file except `docs/import-receipt.md`. Run:

```sh
cargo run -q -p harp -- repository verify
```

Copy the reported expected digest with `apply_patch`, stage the receipt, rerun
`repository verify`, then rerun `mise run verify`. No tracked byte changes
afterward. Commit focused finalization repairs and receipts.

- [ ] **Step 6: Record the frozen PASS**

The phase verifier recomputes the candidate commit/tree, all route and bundle
identities, and the exact test results. Close CPFR-090 only on PASS.

### Task 10: Land the finalization branch and audit Crouzeix worktrees

**Files:**

- Create/modify: `docs/workstream/crouzeix-proof-reproduction/worktree-inventory-002.md`
- Modify: CPFR-091 tracker subtree and execution ledger
- Modify last if payload changes: `docs/import-receipt.md`

- [ ] **Step 1: Fast-forward the verified program tree**

From the primary checkout:

```sh
git status --short --branch
git merge --ff-only review/cpfr-090-three-route-finalization
git rev-parse HEAD
```

Abort if tracked primary-checkout changes exist. Preserve untracked user paths.
Record this commit as the program landing.

- [ ] **Step 2: Inventory every Crouzeix worktree and branch**

For each matching worktree/ref, first assign the literal values copied from the
inventory to `audit_worktree` and `audit_branch`, then record:

```sh
test -n "$audit_worktree" && test -n "$audit_branch"
git -C "$audit_worktree" status --short --branch
git -C "$audit_worktree" rev-parse HEAD
git merge-base master "$audit_branch"
git log --oneline "master..$audit_branch"
git diff --stat "master...$audit_branch"
git merge-base --is-ancestor "$audit_branch" master
```

Also record active processes whose cwd or arguments reference the worktree.
Use explicit paths; do not run recursive deletion commands.

- [ ] **Step 3: Classify and remove only safe worktrees**

Allowed dispositions are:

```text
removed-clean-contained
preserved-dirty
preserved-divergent
preserved-active-process
preserved-unexplained
unrelated-out-of-scope
```

For `removed-clean-contained`, require clean status, no process, and successful
ancestor check. Remove with `git worktree remove "$audit_worktree"`, then
delete the exact local branch with `git branch -d "$audit_branch"`. Never use
force. Clear both variables before classifying the next row.

- [ ] **Step 4: Verify and land the inventory**

```sh
git worktree list --porcelain
git branch --no-merged master
git status --short --branch
python3 labs/crouzeix_proof_reproduction/proof_evidence.py validate --route all
cargo run -q -p harp -- sources verify
PYTHONDONTWRITEBYTECODE=1 python3 \
  labs/crouzeix_proof_reproduction/proof_evidence.py validate-goal
```

At this stage `validate-goal` must still exit 1 because Phase 8 is pending, but
it must report the program landing and worktree audit as valid. Commit the
inventory/tracker update, refresh the import receipt if needed, and close Kata
`harp#yahx` and CPFR-091.

### Task 11: Review execution and improve this goal plan

**Files:**

- Create: `docs/workstream/crouzeix-proof-reproduction/retrospective-003.md`
- Modify: `docs/superpowers/plans/2026-08-23-crouzeix-end-to-end-proof-goal.md`
- Modify: execution ledger and relevant tracker completion evidence
- Modify last: `docs/import-receipt.md`

- [ ] **Step 1: Freeze the execution corpus**

Collect the committed execution ledger, CPFR-085--091 tracker subtrees, Git
history, review artifacts, verifier outputs, route/bundle receipts, and final
worktree inventory. Do not use private memory as evidence.

- [ ] **Step 2: Run the independent post-execution reviewer**

Use the appendix prompt. Require a phase-by-phase table with planned action,
actual action, variance, cause, cost, and structural improvement. It must cover
every failed test, review rewind, cache wait, context loss, ownership conflict,
partial publication, redundant command, and missing automation observed in the
ledger.

- [ ] **Step 3: Write `retrospective-003.md`**

Use these exact sections:

```text
Outcome
Evidence corpus
Phase-by-phase variance
Failed assumptions and root causes
Verifier effectiveness
Cache and proof-iteration efficiency
Context recovery quality
Publication and landing safety
Plan changes
Rejected changes
```

Every plan change cites a committed artifact, tracker evidence row, review
finding, or verifier result. Separate observed facts from interpretations.

- [ ] **Step 4: Improve the plan**

Add a `Plan evolution` section to this file. Tighten or automate commands,
phase boundaries, recovery rules, and tests supported by the retrospective. Do
not weaken theorem scope, evidence requirements, user authority, or trust
boundaries. A foundational change requires a new design document instead.

- [ ] **Step 5: Review and land the substantive plan revision**

Obtain independent Spec review followed by Standards review of the
retrospective and plan diff. Run `mise run verify`, refresh the import receipt
last, and commit as:

```text
docs(crouzeix): improve proof goal from execution
```

Record this commit as `plan_revision`.

- [ ] **Step 6: Finalize the completion footer without self-reference**

Replace the footer at the end of this file with exact existing commit IDs for
the Phase 7 program landing and the Phase 8 substantive plan revision. Run the
goal verifier, repository verifier, and documentation-focused tests. Commit the
footer separately as:

```text
chore(crouzeix): close end-to-end proof goal
```

The footer-finalization commit is discovered from Git; it does not embed its own
hash.

---

## Review and verifier prompt templates

### Implementer prompt

```text
Implement only Task ${TASK_NUMBER} from
docs/superpowers/plans/2026-08-23-crouzeix-end-to-end-proof-goal.md.
Work in ${WORKTREE} on ${BRANCH} from ${BASE_COMMIT}. Other work may
exist; preserve it and edit only the listed owned files. Follow genuine
RED/GREEN TDD and report the observed RED before production changes. Never
hydrate or update Lean dependencies; use the verified primary .lake symlink,
and run Lean serially. Do not publish evidence, edit tracker state, merge,
push, or clean up unless the task explicitly delegates that action. Return
DONE, DONE_WITH_CONCERNS, NEEDS_CONTEXT, or BLOCKED with changed paths, exact
commands/results, and remaining risks.
```

### Jin mathematical Spec reviewer prompt

```text
Review the frozen CPFR-085 candidate read-only against the pinned Jin commit
565b6a3e0659b6e0785f783b016c3f6d9f171fa5 and its verified archive. Check all
21 nodes, declaration types, actual Lean proof providers, dependency edges,
source locators/excerpts, correspondence_kind, one derived extraction, three
consequences, 66-module closure, claim ceiling, license policy, and absence of
receipt/review publication. Recompute rather than trust declared digests.
Return the required structured verdict and a 21-row disposition table. Any
Critical or Important finding blocks route execution and publication.
```

### Route Standards reviewer prompt

```text
Review ${BASE_COMMIT}..${CANDIDATE_COMMIT} read-only for repository and evidence standards.
Attack Python/Rust/schema parity, strict parsing, bounded reads and outputs,
symlink/hardlink/path aliasing, Git identity binding, cache identity, provider
closure, create-only publication, concurrent writers, partial-publication
recovery, immutable history, and maintainability. Run focused negative probes
but do not edit, publish, merge, push, or clean up. Return the required
structured verdict. Critical and Important findings block landing.
```

### Phase verifier prompt

```text
Independently verify phase ${PHASE} using only this goal file, base/candidate
commit IDs, candidate worktree, claimed artifact paths/hashes, and source
identities. Check tree identity and dirty state, run the phase's exact verifier
commands, recompute source/type/closure/provider/axiom/evidence digests, test
negative mutations, verify review ordering and publication state, and confirm
the proposed next transition. Do not rely on implementer narration and do not
edit, publish, merge, push, or clean up. Return the required structured
verdict.
```

### Program Spec reviewer prompt

```text
Review the frozen integrated Crouzeix program read-only. Verify Jin and LS
source correspondence, Harp derivation and eleven-module LS reuse, all theorem
types and active provider dependencies, all route receipts/reviews, the six
bundle claims, the finite polynomial/scalar rational/Hilbert claim ceiling,
and every reader-facing completion statement. Recompute bound identities.
Return the required structured verdict. Any Critical or Important finding
blocks the release gate.
```

### Program Standards reviewer prompt

```text
Review the frozen integrated tree read-only for evidence integrity, filesystem
safety, cache discipline, Python/Rust parity, deterministic generation,
create-only publication and recovery, repository conventions, and maintainable
module boundaries. Attack negative paths and verify the genuine artifacts, not
only fixtures. Return the required structured verdict. Any Critical or
Important finding blocks final landing.
```

### Post-execution plan reviewer prompt

```text
Compare the committed goal plan with the complete execution corpus: tracker
evidence, execution ledger, Git history, reviews, receipts, verifier outputs,
and worktree inventory. For each phase identify planned versus actual actions,
variance, root cause, cost, and a structural improvement. Find repeated rework,
unnecessary Lean work, cache contention, context-loss failures, ambiguous
ownership, unsafe publication transitions, weak verifiers, and prose rules
that should become tests. Do not weaken theorem scope, evidence requirements,
or authority boundaries. Return a retrospective outline and exact proposed
plan edits with evidence locators.
```

---

## Global stop conditions

Stop and request user direction only when:

1. progress requires dependency hydration or cache maintenance;
2. dirty or divergent work cannot be attributed or safely preserved;
3. a requested theorem or trust-boundary change conflicts with the approved
   designs;
4. a push, remote publication, persistent service, or authority expansion is
   required; or
5. the same external blocking condition has persisted for three consecutive
   goal turns with no meaningful in-scope progress.

Test failures, review findings, hard proofs, and repair loops are not stop
conditions.

## Plan evolution

No post-execution revisions have been applied. Phase 8 must replace this
sentence with evidence-linked changes or an evidence-linked statement that no
procedural change was warranted.

## Completion footer

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

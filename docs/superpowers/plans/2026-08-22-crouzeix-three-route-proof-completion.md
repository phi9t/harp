# Crouzeix Three-Route Proof Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring the Jin, Lorist--Schwenninger, and Harp Crouzeix routes from compiling terminal declarations to independently reviewed, route-isolated, source- or derivation-mapped, immutable end-to-end proof evidence.

**Architecture:** Keep one shared Lake project and one warm mathlib cache, add dedicated aggregate targets for all three proof providers, validate a common route-manifest contract, and publish immutable evidence only after a frozen source tree passes route-specific compilation and two-axis review. Finish by making a six-row local formalization bundle mandatory and deriving reader claims from the validated ledgers.

**Tech Stack:** Lean 4.32.1, mathlib 4.32.1, Lake, POSIX shell, Python 3 standard library and `unittest`, Rust/Cargo, Kata, mise, Git worktrees.

---

## Starting state and invariants

Start from local `master` at or after `c8237e3`. Read the companion design and
retrospective before claiming a ticket:

- `docs/superpowers/specs/2026-08-22-crouzeix-three-route-proof-completion-design.md`
- `docs/workstream/crouzeix-proof-reproduction/retrospective-002.md`
- `docs/workstream/crouzeix-proof-reproduction/tracker.org`

These invariants apply to every task:

- Work in a new isolated worktree based on current `master`.
- One implementation owner controls shared files; parallel agents are
  read-only unless disjoint file ownership is written down first.
- Link the worktree's `formalization/lean/.lake` to the canonical cache in the
  primary checkout. Never copy the cache.
- Never run `lake update`, `lake --try-cache exe cache get Mathlib`, or
  `mise run lean-cache`.
- Never alter historical Jin or LS receipt directories.
- Never add `sorry`, `admit`, custom `axiom`, `opaque`, `unsafe`,
  `native_decide`, or `implemented_by`.
- A successful compile is not a source-fidelity or publication claim.
- Do not update `verification_manifest.tsv`; local Harp proof evidence lives
  in the route receipts and local formalization bundle.
- Do not flip `REQUIRE_LOCAL_FORMALIZATION_MANIFEST` until the genuine bundle
  is staged in the same commit.
- Refresh `docs/import-receipt.md` only after every other tracked byte in a
  landing is settled.
- Use `MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml"` for a worktree-local
  invocation; do not mutate persistent mise trust.

## Task 1: Add no-build preflight and route-isolated targets (CPFR-083)

**Files:**

- Create: `formalization/lean/CrouzeixJin.lean`
- Create: `formalization/lean/CrouzeixHarp.lean`
- Modify: `formalization/lean/lakefile.toml`
- Modify: `scripts/check_lean_library.sh`
- Modify: `mise.toml`
- Create: `labs/crouzeix_proof_reproduction/proof_evidence.py`
- Create: `labs/crouzeix_proof_reproduction/tests/test_proof_evidence.py`
- Modify: `crates/harp/tests/lean_library.rs`

- [ ] **Step 1: Freeze RED target and preflight tests**

Add wrapper tests requiring `CrouzeixJin` and `CrouzeixHarp`, including direct
and transitive cross-provider rejection. Add Python tests requiring this
read-only result shape:

```json
{
  "schema_version": "crouzeix-proof-preflight/v1",
  "route_id": "jin",
  "status": "ready",
  "aggregate_module": "CrouzeixJin",
  "toolchain": "leanprover/lean4:v4.32.1",
  "cache_root": "formalization/lean/.lake",
  "missing_artifacts": []
}
```

Run and require RED for the missing targets and command:

```sh
cargo test -p harp --test lean_library -- --test-threads=1
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_proof_evidence -v
```

- [ ] **Step 2: Add the dedicated aggregates**

Use these exact aggregate bodies:

```lean
-- CrouzeixJin.lean
import Crouzeix.Jin.Terminal
import CrouzeixConjecture.HilbertSpectralSet

-- CrouzeixHarp.lean
import Crouzeix.Harp.Consequences
```

Register both as non-default `lean_lib` entries. Extend the wrapper's closed
target enum to include `CrouzeixJin` and `CrouzeixHarp`. Refactor the existing
LS-only closure walker into a route-policy table, not three copied shell
implementations.

- [ ] **Step 3: Implement the read-only preflight**

`proof_evidence.py preflight --route <route>` must canonicalize the repository
root, reject a missing or noncanonical toolchain, validate `lake-manifest.json`,
verify that `.lake` resolves to the approved primary cache, parse the complete
active local import closure, and validate every required Mathlib `.olean`
header. It must not call Lake or write files.

- [ ] **Step 4: Add focused mise tasks**

Add `lean-crouzeix-jin`, `lean-crouzeix-ls`, and `lean-crouzeix-harp`. Keep
`lean-crouzeix` as the broad integration target and `lean-all` as the shared
regression target.

- [ ] **Step 5: Prove GREEN without hydrating dependencies**

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_proof_evidence -v
cargo test -p harp --test lean_library -- --test-threads=1
python3 labs/crouzeix_proof_reproduction/proof_evidence.py preflight --route all
scripts/check_lean_library.sh CrouzeixJin
scripts/check_lean_library.sh CrouzeixLoristSchwenninger
scripts/check_lean_library.sh CrouzeixHarp
```

Expected: all three route targets pass from the existing cache and the
preflight reports no missing dependency artifact. Commit as
`feat(crouzeix): isolate proof route build targets`.

## Task 2: Freeze the common route-manifest contract (CPFR-084)

**Files:**

- Create: `labs/crouzeix_proof_reproduction/route_validation.py`
- Create: `labs/crouzeix_proof_reproduction/schemas/route_manifest.schema.json`
- Create: `labs/crouzeix_proof_reproduction/schemas/route_receipt.schema.json`
- Create: `labs/crouzeix_proof_reproduction/schemas/proof_review.schema.json`
- Create: `labs/crouzeix_proof_reproduction/tests/test_route_validation.py`
- Modify: `labs/crouzeix_proof_reproduction/proof_evidence.py`
- Create: `crates/harp/src/sources/crouzeix/route.rs`
- Modify: `crates/harp/src/sources/crouzeix.rs`

- [ ] **Step 1: Encode the contract as RED tests**

Test exact schemas, unknown fields, duplicate nodes, dependency cycles, missing
terminal nodes, unmapped closure modules, invalid source locators, undeclared
reuse, theorem-type drift, provider-policy drift, forbidden axioms, and a
receipt whose command or closure does not match the route. Use concrete fixture
trees; do not mock the manifest validator on a happy path.

- [ ] **Step 2: Implement strict manifest types**

Expose these validated records:

```python
@dataclass(frozen=True)
class RouteNode:
    node_id: str
    role: str
    declaration: str
    module_path: str
    dependency_ids: tuple[str, ...]
    provenance_kind: str
    source_locator: str | None
    reused_from_route: str | None
    statement_sha256: str

@dataclass(frozen=True)
class RouteManifest:
    route_id: str
    claim_kind: str
    aggregate_module: str
    terminal_declaration: str
    consequence_declarations: tuple[str, ...]
    source_identities: tuple[str, ...]
    nodes: tuple[RouteNode, ...]
```

Allowed `claim_kind` values are `source-faithful` and `derived`. Allowed
`provenance_kind` values are `source`, `shared-foundation`, `reused-route`, and
`derived`. A `source` node requires a pinned locator; a `reused-route` node
requires an exact route and node ID.

- [ ] **Step 3: Derive closure checks from active imports**

Reuse `provider_independence.py` for syntax and closure traversal. Require each
route-local module in the computed closure to be covered by exactly one node or
an explicit shared-foundation module list. Do not infer source correspondence
from namespace names.

- [ ] **Step 4: Add read-only validation CLI**

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py validate --route all
```

The command returns nonzero until all three real manifests and receipts exist;
`--allow-unpublished` validates structure and reports the exact incomplete
claim level without treating it as success.

- [ ] **Step 5: Verify and commit**

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_route_validation -v
python3 -m json.tool labs/crouzeix_proof_reproduction/schemas/route_manifest.schema.json >/dev/null
python3 -m json.tool labs/crouzeix_proof_reproduction/schemas/route_receipt.schema.json >/dev/null
python3 -m json.tool labs/crouzeix_proof_reproduction/schemas/proof_review.schema.json >/dev/null
git diff --check
```

Commit as `feat(crouzeix): define route proof evidence contracts`.

## Task 3: Complete the Jin correspondence and route receipt (CPFR-085)

**Files:**

- Create: `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json`
- Create: `evidence/crouzeix_conjecture/reviews/jin.json`
- Create: `evidence/crouzeix_conjecture/routes/jin/`
- Create: `labs/crouzeix_proof_reproduction/tests/test_route_jin.py`

- [ ] **Step 1: Inventory the entire Jin terminal closure**

Run preflight and generate the active import roster rooted at `CrouzeixJin`.
Map the load-bearing declarations for numerical-range convexity, simple-spectrum
approximation, completion sampling/cancellation, Gramian positivity,
positive-real completion, double-layer realization, fixed-domain convergence,
outer-domain convergence, and terminal polynomial specialization.

Use only pinned identities already registered for Jin. Obtain exact source
locators from the `565b6a3` manuscript and author-maintained formalization map
through the existing acquisition workflow; do not add a dependency on a local
Jin checkout and do not rewrite the three historical proof-slice receipts.

- [ ] **Step 2: Make manifest validation pass before publication**

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  validate --route jin --allow-unpublished
```

Expected: `mapped`, with zero missing closure modules and zero source-identity
errors.

- [ ] **Step 3: Run a frozen independent Spec review**

The reviewer must compare the pinned Jin source against the complete local
route, not only the three existing source-map rows. Every Critical or Important
finding blocks publication. Encode the final verdict in `reviews/jin.json`,
binding the reviewed commit, route-manifest digest, terminal declaration-type
digest, and findings.

- [ ] **Step 4: Publish the Jin route receipt**

Run the dedicated target once, perform terminal and load-bearing axiom audits,
and publish a create-only route directory. Validate it independently in Python
and Rust. Expected allowed axioms, in canonical order:

```text
Classical.choice
Quot.sound
propext
```

- [ ] **Step 5: Verify and commit**

```sh
scripts/check_lean_library.sh CrouzeixJin
python3 labs/crouzeix_proof_reproduction/proof_evidence.py validate --route jin
cargo test -p harp --lib sources::crouzeix::tests -- --test-threads=1
```

Commit as `evidence(crouzeix): certify Jin route`.

## Task 4: Publish and promote the six-node LS route (CPFR-086)

**Files:**

- Modify: `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json`
- Modify: `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/library-inventory.json`
- Create: six new attempt directories under `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/proof-slices/`
- Create: `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/route-manifest.json`
- Create: `evidence/crouzeix_conjecture/reviews/lorist-schwenninger.json`
- Create: `evidence/crouzeix_conjecture/routes/lorist-schwenninger/`
- Create: `labs/crouzeix_proof_reproduction/tests/test_route_lorist_schwenninger.py`

- [ ] **Step 1: Freeze the candidate commit and preflight**

Require the dedicated LS target, exact six-node graph contract, canonical
source identity `arxiv:2608.03841v1`, and warm cache to validate before any
create-only directory appears.

- [ ] **Step 2: Publish candidate receipts exactly once**

Invoke `publish_ls_receipts` through `proof_evidence.py publish-ls`. Expected
new attempts are:

```text
ls-equation-one-terminal-bound/attempt-002
ls-power-recurrence/attempt-001
ls-scalar-contradiction/attempt-002
ls-perturbation-lemma/attempt-001
ls-double-layer-realization/attempt-001
ls-terminal-crouzeix/attempt-001
```

If a `PartialPublicationError` occurs, preserve its candidate map and validate
the visible directories. Do not rerun blindly, overwrite a candidate, or
delete a successfully renamed directory.

- [ ] **Step 3: Review the source correspondence**

Review all six nodes against the pinned TeX, including `T = op(P)`, `Q = M_h`,
Equation 1, the operator recurrence, scalar endpoint, double-layer
realization, and the inner and outer limits. Store the hash-bound verdict in
`reviews/lorist-schwenninger.json`.

- [ ] **Step 4: Promote atomically**

After every candidate and the review pass, update all six graph rows in one
write to the canonical declarations, selected receipt digests, and `passed`.
Update the inventory so all formerly blocked facts are `local_compiled`. A
partial graph promotion is invalid.

- [ ] **Step 5: Validate and commit**

```sh
scripts/check_lean_library.sh CrouzeixLoristSchwenninger
python3 labs/crouzeix_proof_reproduction/proof_evidence.py validate --route lorist-schwenninger
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_ls_receipts \
  labs.crouzeix_proof_reproduction.tests.test_ls_validation -v
cargo test -p harp --lib sources::crouzeix::tests -- --test-threads=1
```

Commit as `evidence(crouzeix): certify Lorist-Schwenninger route`.

## Task 5: Complete the Harp derivation ledger and route receipt (CPFR-087)

**Files:**

- Create: `labs/crouzeix_proof_reproduction/formal_targets/harp/route-manifest.json`
- Create: `evidence/crouzeix_conjecture/reviews/harp.json`
- Create: `evidence/crouzeix_conjecture/routes/harp/`
- Create: `labs/crouzeix_proof_reproduction/tests/test_route_harp.py`

- [ ] **Step 1: Record the exact derivation graph**

Include nodes for positive cubature, finite-measure cubature, finite atomic
matrix moments, the concrete counting-L2 witness and dimension bound,
finite-horizon recurrence, perturbation endpoint, double-layer application,
both limiting passages, and the terminal theorem.

Classify all eleven imported `Crouzeix.LoristSchwenninger.*` modules as exact
`reused-route` dependencies. The manifest must reject an unlisted twelfth LS
module or any LS terminal/provider import.

- [ ] **Step 2: Validate the unpublished graph**

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  validate --route harp --allow-unpublished
```

Expected: `mapped`, explicit LS reuse, no Jin import, and no LS terminal import.

- [ ] **Step 3: Run independent mathematical and novelty reviews**

The Spec reviewer checks the finite positive cubature, moment preservation,
finite counting-L2 realization, horizon-to-infinity argument, and theorem
assembly. The review must describe the route as Harp-derived and
terminal-provider-independent, never as wholly mathematically independent.

- [ ] **Step 4: Publish and verify**

Run `CrouzeixHarp`, audit the terminal and named load-bearing declarations, and
publish a create-only route receipt bound to the route manifest and review.
Validate it through both language implementations.

- [ ] **Step 5: Commit**

```sh
scripts/check_lean_library.sh CrouzeixHarp
python3 labs/crouzeix_proof_reproduction/proof_evidence.py validate --route harp
cargo test -p harp --lib sources::crouzeix::tests -- --test-threads=1
```

Commit as `evidence(crouzeix): certify Harp finite-horizon route`.

## Task 6: Publish and require the six-row local bundle (CPFR-088)

**Files:**

- Modify: `labs/crouzeix_proof_reproduction/local_formalization_evidence.py`
- Modify: `labs/crouzeix_proof_reproduction/tests/test_local_formalization_evidence.py`
- Modify: `crates/harp/src/sources/crouzeix.rs`
- Create: `evidence/crouzeix_conjecture/local_formalization/`

- [ ] **Step 1: Write RED six-route-boundary tests**

Change the expected roster from four to six rows by adding
`jin-main-theorem` and `jin-closed-numerical-range`. Require every row to bind
its route-manifest and review digest. Test missing Jin, LS, or Harp route
evidence, route-receipt drift, and cross-provider substitution.

- [ ] **Step 2: Extend Python and Rust in lockstep**

Keep identical field sets, ordering, allowed axioms, route roots, and provider
policies. Add shared concrete fixtures consumed by both implementations where
possible; do not maintain semantically different synthetic fixtures.

- [ ] **Step 3: Publish exactly once**

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py preflight --route all
python3 labs/crouzeix_proof_reproduction/proof_evidence.py publish-local
```

The publisher runs one broad `Crouzeix` build, six axiom audits, and three
provider audits, then atomically creates the bundle. It must never accept
precomputed command output through its public API.

- [ ] **Step 4: Make the bundle mandatory in the same commit**

Set `REQUIRE_LOCAL_FORMALIZATION_MANIFEST` to `true`, validate the real bundle
in Python and Rust, and prove deleting any member makes `sources verify` fail.

- [ ] **Step 5: Commit**

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_local_formalization_evidence -v
cargo test -p harp --lib sources::crouzeix::tests -- --test-threads=1
cargo run -q -p harp -- sources verify
```

Commit as `evidence(crouzeix): require local proof bundle`.

## Task 7: Reconcile canonical prose and generated readers (CPFR-089)

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
- Regenerate: `atlas/src/content/generated/corpus.json`
- Regenerate: `atlas/dist/harp-atlas.html`
- Regenerate: `atlas/dist/harp-atlas.receipt.json`

- [ ] **Step 1: Make stale claims fail tests**

Add assertions for all three route IDs, receipt paths, exact terminal names,
claim levels, the finite-dimensional polynomial theorem, the scalar rational
consequence, and Harp's explicit LS reuse. Require old wording that says LS's
four nodes remain open to fail once the graph is promoted.

- [ ] **Step 2: Update prose from evidence only**

Distinguish upstream Jin clean-room builds from Harp-local certification. State
that LS is source-faithful to arXiv v1 and that Harp is a derived
finite-horizon route. Preserve publication-status and complete-boundedness
caveats.

- [ ] **Step 3: Regenerate and test together**

```sh
cargo test -p harp --test crouzeix_conjecture_knowledge_packet
cargo run -p harp -- build
cd atlas && corepack pnpm run test && corepack pnpm run build
```

Commit canonical prose and generated outputs together as
`docs(crouzeix): publish three-route proof status`.

## Task 8: Run frozen two-axis review and release gate (CPFR-090)

**Files:**

- Modify only files required to address reviewed findings.
- Modify last: `docs/import-receipt.md`
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`

- [ ] **Step 1: Freeze the candidate and run independent reviews**

Run a Standards review over code, filesystem safety, cache policy, receipt
integrity, and maintainability. Run a separate Spec review over all three
mathematical chains, source correspondence, theorem types, route reuse, and
claim ceilings. Repair and repeat until neither reports a Critical or Important
finding.

- [ ] **Step 2: Run the exact warm-cache gate**

```sh
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" \
ELAN_HOME=/private/tmp/harp-mathematical-foundations-elan \
ELAN_TOOLCHAIN=leanprover/lean4:v4.32.1 \
PATH="/private/tmp/harp-mathematical-foundations-elan/toolchains/leanprover--lean4---v4.32.1/bin:$PATH" \
mise run verify
```

- [ ] **Step 3: Refresh the payload receipt last**

Stage every intended file except `docs/import-receipt.md`, run
`harp repository verify`, copy the reported digest with `apply_patch`, stage
the receipt, and rerun the full gate. No tracked byte may change afterward.

- [ ] **Step 4: Land locally**

Commit in focused concern groups, fast-forward local `master`, verify tree
identity, and do not push.

## Task 9: Audit and retire superseded Crouzeix worktrees (CPFR-091)

**Files:**

- Modify: `docs/workstream/crouzeix-proof-reproduction/worktree-inventory-002.md`
- Modify last if tracked payload changes: `docs/import-receipt.md`

- [ ] **Step 1: Inventory every Crouzeix branch and worktree**

For each item record its head, merge base, unique commits, tracked changes,
untracked paths, running processes, and whether its head is an ancestor of
`master`.

- [ ] **Step 2: Classify before cleanup**

Remove only clean worktrees whose commits are fully represented on `master`.
Preserve or recover any unique commit or local file before removal. In
particular, keep the dirty `crouzeix-rsi-workflow-design` worktree quarantined
until its broad deletion/replacement state has an owner-approved disposition.

- [ ] **Step 3: Delete only audited refs**

Delete local branches only after their worktrees are removed and
`git merge-base --is-ancestor <branch> master` succeeds. Never delete a
divergent branch merely because a newer implementation exists.

- [ ] **Step 4: Record and verify the final inventory**

Run `git worktree list --porcelain`, `git branch --no-merged master`, and
`git status --short --branch` in the primary checkout. Commit only the updated
inventory and final receipt if needed.

## Execution order

Run CPFR-083 and CPFR-084 serially. After the common contracts are frozen, run
CPFR-085, CPFR-086, and CPFR-087 in parallel isolated worktrees because their
route manifests, receipts, reviews, and focused tests are disjoint. Merge them
into one integration branch before CPFR-088; resolve only generated manifest
or import-receipt conflicts there. Read-only source reviews may begin during
CPFR-084, but no route receipt may be published until its manifest contract and
review inputs are frozen. CPFR-091 begins only after CPFR-090 lands.

For every task, update both Kata and the matching `tracker.org` subtree with
the exact test commands, exit statuses, artifact paths, and commit IDs. A fresh
agent should need only the design, this plan, and its one tracker ticket.

## Kata issue map

| Tracker ticket | Kata issue | Dependency |
|---|---|---|
| CPFR-083 | `harp#9y1w` | ready |
| CPFR-084 | `harp#r1gt` | CPFR-083 |
| CPFR-085 | `harp#gzah` | CPFR-084 |
| CPFR-086 | `harp#2eb2` | CPFR-084 |
| CPFR-087 | `harp#2qy3` | CPFR-084 |
| CPFR-088 | `harp#ga15` | CPFR-085, CPFR-086, CPFR-087 |
| CPFR-089 | `harp#bjpn` | CPFR-088 |
| CPFR-090 | `harp#jzg9` | CPFR-089 |
| CPFR-091 | `harp#yahx` | CPFR-090 |

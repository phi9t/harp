# Lorist-Schwenninger Route Certification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Certify and locally land the six-node Lorist-Schwenninger Crouzeix route with concrete Lean theorem witnesses, pinned arXiv provenance, immutable v2 node receipts, atomic graph promotion, an independently reviewed route receipt, and a reusable post-execution record.

**Architecture:** Reuse the compiled LS theorem chain and the existing one-build/six-audit publisher. Extend the shared route contract with exact arXiv provenance, promote the graph and inventory through one recoverable transaction, then reuse the create-only route publisher proven by CPFR-085. Compilation, source correspondence, evidence publication, and landing remain separate states.

**Tech Stack:** Lean 4.32.1, mathlib 4.32.1, Lake, Python 3.9-compatible standard library and `unittest`, Rust/Cargo, JSON Schema draft 2020-12, POSIX shell, mise, Git worktrees, Kata.

---

## Execution contract

This file is an executable goal for CPFR-086. A fresh agent must read
`AGENTS.md`, the paired design, this plan, the tracker subtree, and the current
execution ledger before changing files. Disk and Git state override the
checkpoint below.

The controller owns tracker transitions, evidence publication, commits, local
landing, and Kata mutation. Workers may edit only assigned files. Independent
reviewers are read-only. Do not push. Do not remove the worktree after landing.

Use the canonical cache exactly as follows:

```sh
export ELAN_HOME=/private/tmp/harp-mathematical-foundations-elan
export ELAN_TOOLCHAIN=leanprover/lean4:v4.32.1
export PATH="/private/tmp/harp-mathematical-foundations-elan/toolchains/leanprover--lean4---v4.32.1/bin:/opt/homebrew/bin:/usr/bin:/bin"
```

Never run `lake update`, `lake --try-cache exe cache get Mathlib`, `mise run
lean-cache`, or an equivalent dependency download. Run Lean commands serially.
A missing cache artifact is a blocked precondition, not permission to hydrate.

Reject active `sorry`, `admit`, custom `axiom`, `opaque`, `unsafe`,
`native_decide`, and `implemented_by`. Allowed theorem axioms are exactly:

```text
Classical.choice
Quot.sound
propext
```

Historical LS attempts are immutable. New node attempts and route evidence are
create-only. If a visible unreferenced candidate exists, validate and recover
it. Never delete it and rerun blindly.

## Advisory starting checkpoint

At plan creation:

- local `master` is `7c1963e0862d15937462f6611f98ef8878c21c9a`;
- Jin is `complete-local`;
- the goal validator reports `cpfr-086` as the earliest incomplete phase;
- `CrouzeixLoristSchwenninger` preflight is `ready` with 56 local modules and no
  missing dependency artifacts;
- the LS graph has six rows, two passed and four blocked;
- the code already contains concrete theorem candidates for all six rows;
- only the equation-one and scalar nodes have historical `attempt-001`
  directories;
- no LS route receipt or review exists; and
- Kata `harp#2eb2` is open.

The exact new attempt names expected from this checkpoint are:

```text
ls-equation-one-terminal-bound/attempt-002
ls-power-recurrence/attempt-001
ls-scalar-contradiction/attempt-002
ls-perturbation-lemma/attempt-001
ls-double-layer-realization/attempt-001
ls-terminal-crouzeix/attempt-001
```

Recompute these names immediately before publication. If any path exists, the
publisher must choose the next monotone number.

## Task 1: Establish the implementation worktree and control state

**Files:**

- Read: `AGENTS.md`
- Read: `docs/superpowers/specs/2026-08-24-lorist-schwenninger-route-certification-design.md`
- Read: `docs/superpowers/plans/2026-08-24-lorist-schwenninger-route-certification.md`
- Modify: `docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv`
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`
- Test: `labs/crouzeix_proof_reproduction/tests/test_execution_ledger.py`

- [ ] **Step 1: Verify the planning commits are on local master**

Run from the primary checkout:

```sh
git status --short --branch
git log -3 --oneline
test -f docs/superpowers/specs/2026-08-24-lorist-schwenninger-route-certification-design.md
test -f docs/superpowers/plans/2026-08-24-lorist-schwenninger-route-certification.md
```

Expected: `master` contains both documents. Existing user-owned untracked files
may remain, but no tracked file is dirty. If the documents are only on
`docs/cpfr-086-ls-certification-design`, fast-forward that branch onto `master`
after checking for untracked path collisions.

- [ ] **Step 2: Create or recover the implementation worktree**

Use `.worktrees/cpfr-086-ls-certification` and
`feat/cpfr-086-ls-certification`. If either exists, inspect its branch, head,
tracked changes, untracked files, and relation to `master`; do not overwrite it.
Otherwise run:

```sh
git check-ignore -q .worktrees
git worktree add .worktrees/cpfr-086-ls-certification \
  -b feat/cpfr-086-ls-certification master
```

Expected: a clean named-branch worktree at current `master`.

- [ ] **Step 3: Link and verify the shared cache without invoking Lake**

From the implementation worktree:

```sh
common=$(git rev-parse --path-format=absolute --git-common-dir)
test "$(basename "$common")" = .git
primary_root=${common%/.git}
test "$(git -C "$primary_root" rev-parse --show-toplevel)" = "$primary_root"
test -d "$primary_root/formalization/lean/.lake"
test ! -L "$primary_root/formalization/lean/.lake"
ln -s "$primary_root/formalization/lean/.lake" \
  formalization/lean/.lake
test "$(readlink formalization/lean/.lake)" = \
  "$primary_root/formalization/lean/.lake"
mise trust mise.toml
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  preflight --route lorist-schwenninger
```

Expected: `status` is `ready` and `missing_artifacts` is empty. If `.lake`
already exists, verify it instead of replacing it.

- [ ] **Step 4: Correct the cross-phase Kata dependency and claim CPFR-086**

Run:

```sh
kata edit harp#2qy3 --blocked-by harp#2eb2 \
  --comment "CPFR-087 binds the landed LS manifest and receipt; certification must follow CPFR-086."
kata claim harp#2eb2 \
  --comment "Starting CPFR-086 from the landed Jin baseline under the approved 2026-08-24 design and plan."
kata show harp#2eb2 --format json
kata show harp#2qy3 --format json
```

Expected: `harp#2eb2` is owned by the current operator, and `harp#2qy3` lists
`harp#2eb2` as a blocker.

- [ ] **Step 5: Record the implementing state**

Append one monotone TSV row with:

Set `phase`, `ticket`, `state`, `branch`, and `worktree` to `cpfr-086`,
`CPFR-086`, `implementing`, `feat/cpfr-086-ls-certification`, and
`.worktrees/cpfr-086-ls-certification`. Set `base_commit` to `git rev-parse
master`, `candidate_commit` to `git rev-parse HEAD`, and `note` to
`fresh-worktree;preflight=ready;missing_artifacts=0`.

Change the CPFR-086 tracker heading from `TODO` to `IMPLEMENTING` and fill its
branch, worktree, owner, agent-run, and model properties.

- [ ] **Step 6: Verify and commit the control-state change**

Run:

```sh
python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_execution_ledger -v
python3 labs/crouzeix_proof_reproduction/proof_evidence.py validate-goal
```

Expected: ledger tests pass; goal validation remains incomplete at CPFR-086.
Stage only the tracker and ledger, then commit:

```sh
git add docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv \
  docs/workstream/crouzeix-proof-reproduction/tracker.org
git commit -m "chore(crouzeix): start LS route certification"
```

## Task 2: Add exact arXiv source provenance in Python, schema, and Rust

**Files:**

- Create: `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/artifact-manifest.json`
- Create: `labs/crouzeix_proof_reproduction/tests/test_route_lorist_schwenninger.py`
- Modify: `labs/crouzeix_proof_reproduction/route_validation.py`
- Modify: `labs/crouzeix_proof_reproduction/schemas/route_manifest.schema.json`
- Modify: `crates/harp/src/sources/crouzeix/route.rs`

- [ ] **Step 1: Write RED Python and schema tests for the arXiv identity**

Add constants to `test_route_lorist_schwenninger.py`:

```python
SOURCE_ID = "LS-ARXIV-V1"
SOURCE_IDENTITY = "arxiv:2608.03841v1"
SOURCE_ARCHIVE_SHA256 = "b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9"
SOURCE_FILE_SHA256 = "20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a"
SOURCE_PATH = "CrouzeixConjecturev2.tex"
SOURCE_BYTES = 18_783
SOURCE_LINES = 281
```

Add tests that require:

```python
kind, identity, path, start, end = route_validation._source_locator(
    "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99",
    "source locator",
)
self.assertEqual(
    (kind, identity, path, start, end),
    ("arxiv", SOURCE_IDENTITY, SOURCE_PATH, 67, 99),
)
```

The same test module must reject wrong version syntax, `v0`, wrong filename,
absolute or traversing paths, zero or reversed spans, and an additional `#`.
It must assert that the JSON Schema accepts the exact arXiv locator and identity
and rejects each malformed case.

Run:

```sh
python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_route_lorist_schwenninger -v
```

Expected RED: the identity and locator are rejected by the current SHA-only and
local/Git-only contracts.

- [ ] **Step 2: Add the closed LS artifact manifest**

Create the exact JSON object below with canonical formatting and a final
newline:

```json
{
  "archive_sha256": "b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9",
  "manuscript_bytes": 18783,
  "manuscript_path": "CrouzeixConjecturev2.tex",
  "manuscript_sha256": "20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a",
  "line_count": 281,
  "schema_version": "crouzeix-arxiv-artifact-manifest/v1",
  "source_id": "LS-ARXIV-V1",
  "source_identity": "arxiv:2608.03841v1"
}
```

Tests must cross-check both archive and manuscript records against
`evidence/crouzeix_conjecture/source_manifest.tsv`. Mutating any identity, path,
byte count, digest, or `line_count` to zero must fail.

- [ ] **Step 3: Implement Python parsing and provenance validation**

In `route_validation.py`, define:

```python
ARXIV_IDENTITY_RE = re.compile(r"arxiv:[0-9]{4}\.[0-9]{4,5}v[1-9][0-9]*\Z")
ARXIV_SOURCE_LOCATOR_RE = re.compile(
    r"arxiv:(?P<version>[0-9]{4}\.[0-9]{4,5}v[1-9][0-9]*):"
    r"(?P<path>[A-Za-z0-9._/-]+)"
    r"#L(?P<start>[1-9][0-9]*)-L(?P<end>[1-9][0-9]*)\Z"
)
IDENTITY_RE = re.compile(
    r"(?:sha256:[0-9a-f]{64}|arxiv:[0-9]{4}\.[0-9]{4,5}v[1-9][0-9]*)\Z"
)
```

Extend `_source_locator` to return
`("arxiv", f"arxiv:{version}", path, start, end)`. Add
`_ls_artifact_manifest(repo_root)` and validate exact fields, source-manifest
registration, manuscript path, byte count, line count, archive digest, and file
digest. For an LS arXiv source node, require all three sorted identities in the
route manifest and require node metadata to match the artifact manifest.

- [ ] **Step 4: Implement equivalent JSON Schema and Rust validation**

Add the exact arXiv patterns to `route_manifest.schema.json`. In Rust, replace
the Boolean Git marker with an explicit locator kind:

```rust
enum SourceLocatorKind<'a> {
    Local,
    Git(&'a str),
    Arxiv(&'a str),
}
```

Make `parse_source_locator` return this enum plus path and span. Add a strict LS
artifact-manifest struct with `#[serde(deny_unknown_fields)]`. Cross-check the
same source-manifest rows and exact constants as Python. Keep all reads bounded,
descriptor-relative, and no-follow.

- [ ] **Step 5: Verify parity and commit**

Run:

```sh
python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_route_lorist_schwenninger \
  labs.crouzeix_proof_reproduction.tests.test_route_validation -v
cargo test -p harp sources::crouzeix::route --lib -- --test-threads=1
cargo fmt --all -- --check
git diff --check
```

Expected: all tests pass and Python, schema, and Rust accept and reject the same
locator corpus. Commit only the files in this task:

```sh
git add labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/artifact-manifest.json \
  labs/crouzeix_proof_reproduction/tests/test_route_lorist_schwenninger.py \
  labs/crouzeix_proof_reproduction/route_validation.py \
  labs/crouzeix_proof_reproduction/schemas/route_manifest.schema.json \
  crates/harp/src/sources/crouzeix/route.rs
git commit -m "feat(crouzeix): validate LS arXiv provenance"
```

## Task 3: Freeze the concrete six-node route contract

**Files:**

- Modify: `labs/crouzeix_proof_reproduction/ls_contract.py`
- Modify: `labs/crouzeix_proof_reproduction/tests/test_ls_validation.py`
- Modify: `labs/crouzeix_proof_reproduction/tests/test_ls_receipts.py`
- Extend: `labs/crouzeix_proof_reproduction/tests/test_route_lorist_schwenninger.py`

- [ ] **Step 1: Write RED tests for the exact theorem map**

Require this ordered mapping:

```python
EXPECTED_NODES = {
    "ls-equation-one-terminal-bound": (
        "CrouzeixConjecture.LoristSchwenninger.DilationData."
        "perturbation_mul_target_power_norm_le",
        (),
    ),
    "ls-power-recurrence": (
        "CrouzeixConjecture.LoristSchwenninger.DilationData."
        "equation_three_lower_bound",
        ("ls-equation-one-terminal-bound",),
    ),
    "ls-scalar-contradiction": (
        "CrouzeixConjecture.LoristSchwenninger.scalar_endpoint_le_two",
        (),
    ),
    "ls-perturbation-lemma": (
        "CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two",
        ("ls-power-recurrence", "ls-scalar-contradiction"),
    ),
    "ls-double-layer-realization": (
        "CrouzeixConjecture.LoristSchwenninger."
        "norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary",
        ("ls-perturbation-lemma",),
    ),
    "ls-terminal-crouzeix": (
        "CrouzeixConjecture.loristSchwenningerMainTheorem",
        ("ls-double-layer-realization",),
    ),
}
```

Require the current graph to fail the complete contract because four rows still
name aliases or remain blocked. Require every theorem-body audit to find its
actual provider calls and reject extra narrative-only edges.

Keep the two digest meanings separate. `source-graph.json` and each v2 node
receipt retain the source-claim `statement_sha256` from `ls_contract.NODES`. The
later route manifest computes a different `statement_sha256` from each
normalized Lean declaration type. Never copy a digest from one layer into the
other merely because both fields have the same name.

- [ ] **Step 2: Observe RED**

Run:

```sh
python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_ls_validation \
  labs.crouzeix_proof_reproduction.tests.test_route_lorist_schwenninger -v
```

Expected RED: the live graph remains legacy and the terminal dependency still
contains the redundant perturbation edge.

- [ ] **Step 3: Make `ls_contract.NODES` the sole exact node authority**

Set the terminal dependencies in `ls_contract.py` to only
`("ls-double-layer-realization",)`. Add
`legacy_dependencies=("ls-perturbation-lemma",
"ls-double-layer-realization")` to `LSNodeContract` and use it only when
validating the exact historical graph state. Keep legacy names, statuses,
receipts, and blocked reasons in the existing legacy fields. Do not modify the
authoritative graph yet.

Add theorem-body assertions with these required calls:

```text
equation-three -> recurrence_lower_bound, recurrence_difference_lower_bound
perturbation -> equation_three_lower_bound, displacementSq_le, scalar_endpoint_le_two
double-layer -> dilationDataOfParametricPolynomial, norm_target_le_two
terminal -> norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary,
            norm_polynomialEval_le_of_tendsto,
            tendsto_maxPolynomialModulusOnSet_of_outerApproximation
```

Reject direct use of another terminal provider in every audited body.

- [ ] **Step 4: Verify and commit**

Run the two test modules from Step 2 plus:

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  preflight --route lorist-schwenninger
git diff --check
```

Expected: tests pass; preflight remains `ready`. Commit:

```sh
git add labs/crouzeix_proof_reproduction/ls_contract.py \
  labs/crouzeix_proof_reproduction/tests/test_ls_validation.py \
  labs/crouzeix_proof_reproduction/tests/test_ls_receipts.py \
  labs/crouzeix_proof_reproduction/tests/test_route_lorist_schwenninger.py
git commit -m "test(crouzeix): freeze concrete LS proof graph"
```

## Task 4: Add a closed LS receipt-publication command

**Files:**

- Modify: `labs/crouzeix_proof_reproduction/proof_evidence.py`
- Modify: `labs/crouzeix_proof_reproduction/ls_receipts.py` only if the closed wrapper needs a public result type
- Modify: `labs/crouzeix_proof_reproduction/tests/test_ls_receipts.py`
- Modify: `labs/crouzeix_proof_reproduction/tests/test_proof_evidence.py`

- [ ] **Step 1: Write the RED public-interface tests**

Require a closed CLI command:

```python
with mock.patch.object(ls_receipts, "publish_ls_receipts") as publish:
    exit_code = proof_evidence.main(["publish-ls"])
publish.assert_called_once_with(
    ls_validation.load_route_graph(LS_GRAPH),
    REPO,
    REPO / LS_TARGET,
)
self.assertEqual(exit_code, 0)
```

Also require `publish-ls --path`, `--command`, `--env`, `--row`, and `--output`
to fail argument parsing. The success JSON must contain only:

```text
schema_version
route_id
status
candidates
```

Each candidate contains only `node_id`, `attempt_path`, and `receipt_sha256`, in
`ls_contract.NODE_ORDER`.

- [ ] **Step 2: Observe RED**

Run:

```sh
python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_proof_evidence \
  labs.crouzeix_proof_reproduction.tests.test_ls_receipts -v
```

Expected RED: `publish-ls` is not a recognized subcommand.

- [ ] **Step 3: Implement the closed adapter**

Add constants to `proof_evidence.py`:

```python
LS_TARGET = Path(
    "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger"
)
LS_GRAPH = LS_TARGET / "source-graph.json"
```

Import `ls_receipts` and `ls_validation`. Register `publish-ls` without
arguments. The handler loads the canonical graph and calls:

```python
published = ls_receipts.publish_ls_receipts(
    ls_validation.load_route_graph(repository_root / LS_GRAPH),
    repository_root,
    repository_root / LS_TARGET,
)
```

Normalize `protocol.ValidationError` and `ls_receipts.PartialPublicationError`
to bounded JSON. A partial-publication payload must preserve every visible
candidate path and digest so recovery can proceed without rerunning Lean.

- [ ] **Step 4: Verify and commit**

Run the Step 2 suites. Expected: all pass under system Python 3.9. Commit:

```sh
git add labs/crouzeix_proof_reproduction/proof_evidence.py \
  labs/crouzeix_proof_reproduction/ls_receipts.py \
  labs/crouzeix_proof_reproduction/tests/test_ls_receipts.py \
  labs/crouzeix_proof_reproduction/tests/test_proof_evidence.py
git commit -m "feat(crouzeix): expose closed LS receipt publisher"
```

## Task 5: Publish and independently validate six node candidates

**Files:**

- Create only: new `proof-slices/*/attempt-*` directories under `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/`
- Modify: `docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv`
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`

- [ ] **Step 1: Freeze the executable candidate**

Require a clean worktree, then record:

```sh
git rev-parse HEAD
git rev-parse HEAD^{tree}
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  preflight --route lorist-schwenninger
```

Record the candidate commit and tree in the tracker. Do not append
`locally-verified` yet; that state requires the frozen unpublished route
manifest from Task 7.

- [ ] **Step 2: Publish once**

Run only this command:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 \
  labs/crouzeix_proof_reproduction/proof_evidence.py publish-ls
```

Expected: exit 0, `status: published-unreferenced`, six candidate rows, one
aggregate build, and six axiom audits. Do not run another Lean command while it
is active.

- [ ] **Step 3: Handle an existing or partial publication safely**

If the command reports existing or partially visible candidates, do not remove
them. For each reported attempt, run the committed receipt validator against
the untouched directory. Resume with the valid candidate map. If a candidate
is invalid, stop publication and preserve it for diagnosis.

- [ ] **Step 4: Independently validate the candidate set**

Construct candidate `LSGraphRow` values from `ls_contract.NODES` and the six
returned receipt digests, with every status `passed` and all reason fields
absent. Feed those rows and untouched attempts to
`ls_validation.validate_committed_receipts`. Require:

```text
six validated nodes
one shared aggregate command digest
one shared local closure digest
one shared required Mathlib artifacts digest
six exact declaration names
six allowed axiom results
```

Recompute and record the historical v1 receipt digests before and after. They
must be byte-identical.

- [ ] **Step 5: Commit only immutable candidate attempts**

Stage each new attempt path explicitly and commit:

```sh
git commit -m "evidence(crouzeix): publish LS node candidates"
```

Do not modify the graph, inventory, route manifest, or route publication paths
in this commit.

## Task 6: Implement atomic LS graph and inventory promotion

**Files:**

- Create: `labs/crouzeix_proof_reproduction/ls_promotion.py`
- Create: `labs/crouzeix_proof_reproduction/tests/test_ls_promotion.py`
- Modify: `labs/crouzeix_proof_reproduction/ls_validation.py`
- [ ] **Step 1: Write RED promotion-interface and recovery tests**

Require exactly:

```python
signature = inspect.signature(ls_promotion.promote_six_node_route)
self.assertEqual(tuple(signature.parameters), ("repository_root", "candidate_receipts"))
```

Tests must reject missing, extra, duplicate, wrong-route, mixed-v1/v2, stale,
symlinked, hardlinked, and dependency-inconsistent candidates. Inject failures
before and after each graph, inventory, and marker write. A reader may accept
only the exact old legacy pair or a marker-bound new pair, never a mixed pair.

Expected RED:

```sh
python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_ls_promotion -v
```

The module does not yet exist.

- [ ] **Step 2: Implement a marker-committed two-file transaction**

`promote_six_node_route` validates all candidates and snapshots the old graph,
inventory, and source modules. It derives the new graph and inventory from
`ls_contract.NODES`; callers cannot inject their bytes. It stages canonical new
files. The marker has exactly three fields: `schema_version`, fixed to
`crouzeix-ls-promotion/v1`; `graph_sha256`, computed as
`protocol.sha256_bytes(graph_bytes)`; and `inventory_sha256`, computed as
`protocol.sha256_bytes(inventory_bytes)`. The publisher replaces graph and
inventory only after staging and fsync. It writes `promotion.json` last with
create-only atomic rename. `ls_validation` treats the new passed graph as
authoritative only when both digests match. Without a marker it accepts only the
exact historical two-passed/four-blocked pair. On a caught pre-marker failure,
the publisher restores both original snapshots before returning. A process
crash leaves a fail-closed state that a retry repairs from the staged snapshot
before attempting publication again.

- [ ] **Step 3: Verify and commit without promoting live state**

Run:

```sh
python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_ls_promotion \
  labs.crouzeix_proof_reproduction.tests.test_ls_validation \
  labs.crouzeix_proof_reproduction.tests.test_ls_receipts -v
git diff --check
```

The tests operate on temporary copies. The live graph and inventory must remain
unchanged. Commit only the mechanism and tests:

```sh
git add labs/crouzeix_proof_reproduction/ls_promotion.py \
  labs/crouzeix_proof_reproduction/ls_validation.py \
  labs/crouzeix_proof_reproduction/tests/test_ls_promotion.py
git commit -m "feat(crouzeix): add atomic LS graph promotion"
```

## Task 7: Materialize and freeze the LS route manifest

**Files:**

- Create: `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/route-manifest.json`
- Create: `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/declaration-types/*.txt`
- Extend: `labs/crouzeix_proof_reproduction/tests/test_route_lorist_schwenninger.py`
- Modify: `labs/crouzeix_proof_reproduction/route_publication.py`
- Modify: `crates/harp/src/sources/crouzeix/route.rs` only for parity gaps exposed by RED tests

- [ ] **Step 1: Acquire the pinned source into temporary review storage**

Run:

```sh
ls_source_tmp=$(mktemp -d /private/tmp/harp-ls-source.XXXXXX)
curl -fsSL https://export.arxiv.org/e-print/2608.03841v1 \
  -o "$ls_source_tmp/source.tar.gz"
test "$(shasum -a 256 "$ls_source_tmp/source.tar.gz" | awk '{print $1}')" = \
  b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9
tar -xzf "$ls_source_tmp/source.tar.gz" -C "$ls_source_tmp"
test "$(shasum -a 256 "$ls_source_tmp/CrouzeixConjecturev2.tex" | awk '{print $1}')" = \
  20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a
test "$(wc -c < "$ls_source_tmp/CrouzeixConjecturev2.tex" | tr -d ' ')" = 18783
test "$(awk 'END {print NR}' "$ls_source_tmp/CrouzeixConjecturev2.tex")" = 281
```

Preserve this temporary directory through the mathematical review. Do not add
its bytes to Git.

- [ ] **Step 2: Write RED manifest tests**

Require exactly six nodes with the declaration and dependency map from Task 3.
Roles are five `load-bearing` nodes followed by one `terminal` node.
`consequence_declarations` is empty. Require these correspondence kinds:

```text
ls-equation-one-terminal-bound  direct-source
ls-power-recurrence             structural-refactor
ls-scalar-contradiction         direct-source
ls-perturbation-lemma           structural-refactor
ls-double-layer-realization     structural-refactor
ls-terminal-crouzeix            structural-refactor
```

The source identities are the exact three sorted values from Task 2. The module
closure equals `route_validation.active_local_closure` for
`CrouzeixLoristSchwenninger`. Shared foundation modules equal that closure minus
the six node modules. Require this top-level identity:

```json
{
  "route_id": "lorist-schwenninger",
  "claim_kind": "source-faithful",
  "aggregate_module": "CrouzeixLoristSchwenninger",
  "build_target": "CrouzeixLoristSchwenninger",
  "terminal_declaration": "CrouzeixConjecture.loristSchwenningerMainTheorem",
  "consequence_declarations": [],
  "allowed_axioms": ["Classical.choice", "Quot.sound", "propext"],
  "review_path": "evidence/crouzeix_conjecture/reviews/lorist-schwenninger.json",
  "receipt_path": "evidence/crouzeix_conjecture/routes/lorist-schwenninger/receipt.json",
  "review_sha256": null,
  "receipt_sha256": null
}
```

Run the new LS route test. Expected RED because the manifest and type artifacts
do not exist.

- [ ] **Step 3: Generate the six exact theorem types**

Create one temporary audit file with these exact lines:

```lean
import CrouzeixLoristSchwenninger
#check CrouzeixConjecture.LoristSchwenninger.DilationData.perturbation_mul_target_power_norm_le
#check CrouzeixConjecture.LoristSchwenninger.DilationData.equation_three_lower_bound
#check CrouzeixConjecture.LoristSchwenninger.scalar_endpoint_le_two
#check CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two
#check CrouzeixConjecture.LoristSchwenninger.norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary
#check CrouzeixConjecture.loristSchwenningerMainTheorem
```

Run `lake env lean` once under the pinned environment. Split the six printed
types without editing their mathematical content, remove the temporary file,
and write them to these exact paths:

```text
declaration-types/ls-equation-one-terminal-bound.txt
declaration-types/ls-power-recurrence.txt
declaration-types/ls-scalar-contradiction.txt
declaration-types/ls-perturbation-lemma.txt
declaration-types/ls-double-layer-realization.txt
declaration-types/ls-terminal-crouzeix.txt
```

Compute every
`statement_sha256` with `route_validation.normalized_type_sha256`; do not reuse
the source graph's statement hashes.

- [ ] **Step 4: Create the unpublished manifest**

Write canonical JSON with null publication digests. Every source node carries
the archive SHA-256, manuscript SHA-256, line count 281, and the excerpt digest
recomputed from the temporary TeX for its exact source span. Require these exact
span digests:

```text
L67-L99   d1a0ed50900b980377c602334d0d1969e9506d432321ad8257ed4c4c93e88c29
L74-L90   3dcbda0d7130e436252a5937c25aef7845a4a927a40121bed5e4efb05daec611
L90-L98   6fa4901136095ac5c092ef059895458467d8880782294c896390f23ee729e06a
L100-L124 e115e78d0ff01e9c2f639e3044822c381f97d538ea8f99df1ffd6634ff063af0
L125-L128 e5e712ea98f3578083a8382e8a8772c386129df2fbbe4eabaa628c76a5a55dcb
```

The manifest uses the six concrete declarations and exact provider dependencies
from Task 3.

- [ ] **Step 5: Bind LS artifact metadata into publication snapshots**

For `lorist-schwenninger`, add the LS artifact manifest to both
`route_publication._input_snapshot` and `_require_route_inputs_at_commit`. Also
bind `source-graph.json`, `library-inventory.json`, `promotion.json`, and every
member of the six selected v2 attempt directories. The publisher must call the
LS graph, marker, and committed-receipt validators before executing the route
build. Python and Rust route validation must repeat those checks and prove the
current bound files match the receipt's candidate commit. Add mutation tests for
the artifact manifest, graph, inventory, marker, and one selected attempt member.
Each mutation must fail before publication or make a published route invalid.
Before promotion, `inspect_route(..., allow_unpublished=True)` may accept the
otherwise-valid manifest only as `authored / incomplete` with reason `LS graph
is not promoted`. The create-only route publisher must reject that state before
running Lean. After marker-bound promotion, the same unpublished manifest must
report `mapped / incomplete`.

- [ ] **Step 6: Validate and commit the frozen route candidate**

Run:

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  validate --route lorist-schwenninger --allow-unpublished
python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_route_lorist_schwenninger \
  labs.crouzeix_proof_reproduction.tests.test_route_publication \
  labs.crouzeix_proof_reproduction.tests.test_route_validation -v
cargo test -p harp sources::crouzeix::route --lib -- --test-threads=1
```

Expected before promotion: validation exits 1 with `claim_level: authored`,
`status: incomplete`, and reason `LS graph is not promoted`. Commit the manifest,
six type files, tests, and snapshot binding as:

```text
feat(crouzeix): define LS route manifest
```

Do not append `locally-verified` yet. The route candidate is not frozen until
the reviewed graph is promoted in Task 8. Record the provisional manifest and
type digests in the tracker.

## Task 8: Review and promote the six-node graph

**Files:**

- Modify through publisher: `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json`
- Modify through publisher: `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/library-inventory.json`
- Create through publisher: `labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/promotion.json`
- Modify: `docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv`
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`

- [ ] **Step 1: Run the independent mathematical Spec review**

Give the reviewer the exact Task 7 commit and tree, pinned TeX path and digests,
unpublished route manifest, six theorem types, six node receipts, and audited
provider edges. Require this output structure:

```text
PASS | FAIL

Critical findings
Important findings
Minor findings
Six-node disposition table
Verified commands and results
Unverified claims
Allowed next transition
```

The disposition table must cover Equation 1, recurrence orientation, scalar
contradiction, perturbation assembly, double-layer construction, and the
terminal limiting argument. Critical or Important findings block promotion. A
repair to Lean, source spans, theorem types, node dependencies, or receipt inputs
invalidates affected receipts and requires fresh review.

- [ ] **Step 2: Authorize and perform promotion**

After a passing provisional review, call
`ls_promotion.promote_six_node_route(repository_root, candidate_receipts)` with
the six validated receipt paths and digests. Do not hand-edit graph or inventory.

Expected:

```text
source graph: six passed rows, concrete declarations, six v2 receipt hashes
library inventory: all six certified facts are local_compiled
promotion marker: graph and inventory digests match current bytes
```

- [ ] **Step 3: Verify and commit the promoted evidence**

Run:

```sh
python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_ls_promotion \
  labs.crouzeix_proof_reproduction.tests.test_ls_validation \
  labs.crouzeix_proof_reproduction.tests.test_ls_receipts \
  labs.crouzeix_proof_reproduction.tests.test_route_lorist_schwenninger -v
git diff --check
```

Commit only the graph, inventory, and marker first:

```sh
git add labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json \
  labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/library-inventory.json \
  labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/promotion.json
git commit -m "evidence(crouzeix): promote reviewed LS graph"
```

Record that commit and tree as the route candidate. Verify that every theorem
source, type artifact, source locator, node receipt, and provider edge reviewed
in Step 1 is byte-identical. Ask the reviewer for final binding confirmation
against the promoted commit.
Write the final binding review as canonical `crouzeix-proof-review/v1` candidate
bytes at `.build/crouzeix-route-review-candidates/lorist-schwenninger.json`. It
contains that commit/tree, manifest contract digest, terminal type digest, six
node findings and resolutions, verdict, reviewer identity/model/run ID, and
self-digest. It remains unpublished until Task 9.

Append `locally-verified` followed by `spec-approved`, both using the promoted
candidate commit. Record the final review-candidate digest, six receipt digests,
and promotion-marker digest. Do not append `evidence-published` until the route
receipt and route review exist in Task 9. Commit only tracker and ledger; the
ignored review candidate stays in `.build` for Task 9:

```sh
git add docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv \
  docs/workstream/crouzeix-proof-reproduction/tracker.org
git commit -m "docs(crouzeix): record LS mathematical approval"
```

## Task 9: Publish and bind the LS route evidence

**Files:**

- Create only: `evidence/crouzeix_conjecture/routes/lorist-schwenninger/*`
- Create only: `evidence/crouzeix_conjecture/reviews/lorist-schwenninger.json`
- Modify: LS `route-manifest.json` digest fields only
- Modify: `docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv`
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`

- [ ] **Step 1: Revalidate the final review candidate**

Confirm that the Task 8 review candidate names the promoted candidate commit and
tree, matches the route manifest contract and terminal type digests, and has zero
unresolved Critical or Important findings. If any bound byte changed afterward,
rerun the Task 8 review and replace only the unpublished `.build` candidate. Do
not append duplicate `locally-verified` or `spec-approved` rows.

- [ ] **Step 2: Publish the route receipt once**

Run:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 \
  labs/crouzeix_proof_reproduction/proof_evidence.py \
  publish-route --route lorist-schwenninger
```

Expected: exit 0 and `status: published-unreferenced`. Insert the returned file
digest into `receipt_sha256` with `apply_patch`. If the destination exists,
validate and recover it instead of republishing.

- [ ] **Step 3: Publish the bound review once**

Write canonical review bytes at:

```text
.build/crouzeix-route-review-candidates/lorist-schwenninger.json
```

Bind the exact candidate commit/tree, manifest contract digest, terminal type
digest, reviewer identity/model/run ID, verdict, outcome, source-fidelity check,
and all finding dispositions. Run:

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  publish-review --route lorist-schwenninger
```

Expected: exit 0 and `status: published-unreferenced`. Insert the returned file
digest into `review_sha256`.

- [ ] **Step 4: Validate the complete route**

Run:

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  validate --route lorist-schwenninger
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  labs.crouzeix_proof_reproduction.tests.test_ls_receipts \
  labs.crouzeix_proof_reproduction.tests.test_ls_validation \
  labs.crouzeix_proof_reproduction.tests.test_ls_promotion \
  labs.crouzeix_proof_reproduction.tests.test_route_lorist_schwenninger \
  labs.crouzeix_proof_reproduction.tests.test_route_publication -v
cargo test -p harp sources::crouzeix --lib -- --test-threads=1
```

Expected: route validation reports `complete-local`; all focused suites pass.
Append `evidence-published` with the route receipt and review hashes. Commit the
manifest bindings, route bundle, review, tracker, and ledger:

```sh
git add labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/route-manifest.json \
  evidence/crouzeix_conjecture/routes/lorist-schwenninger \
  evidence/crouzeix_conjecture/reviews/lorist-schwenninger.json \
  docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv \
  docs/workstream/crouzeix-proof-reproduction/tracker.org
git commit -m "feat(crouzeix): certify LS route evidence"
```

## Task 10: Review, release, land, and improve the plan

**Files:**

- Modify: `docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv`
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`
- Create: `docs/workstream/crouzeix-proof-reproduction/cpfr-086-retrospective.md`
- Modify: this plan's `Plan evolution` section
- Modify last: `docs/import-receipt.md`

- [ ] **Step 1: Obtain independent Standards approval**

Review the full CPFR-086 diff from its landed base. Attack Python/Rust/schema
parity, arXiv metadata binding, source and theorem hashes, provider closure,
historical attempt immutability, create-only publication, graph promotion
recovery, path aliasing, symlinks, hardlinks, bounded I/O, process cleanup, and
cache discipline. Every Critical or Important finding requires RED/GREEN repair
and fresh review. Append `standards-approved` only after `PASS`.

- [ ] **Step 2: Run the focused cached release gate**

Run serially:

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  preflight --route lorist-schwenninger
scripts/check_lean_library.sh CrouzeixLoristSchwenninger
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  validate --route lorist-schwenninger
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  discover -s labs/crouzeix_proof_reproduction/tests -p 'test_*.py' -v
cargo test -p harp sources::crouzeix --lib -- --test-threads=1
cargo fmt --all -- --check
git diff --check
```

Expected: all commands exit 0 except no expected-failure CLI probe embedded in
the unit tests. Append `release-verified` with exact counts and timings.

- [ ] **Step 3: Run the full repository gate on the frozen candidate**

Run:

```sh
mise run verify
```

Expected: Atlas, Rust, shell, Python, `lean-all`, and repository verification
all pass. Record the exact Lean job count and timing. Do not edit proof, source,
manifest, receipt, review, or verifier bytes after this point without rerunning
the affected review and full gate.

- [ ] **Step 4: Write the post-execution retrospective and improve this plan**

Create `cpfr-086-retrospective.md` with these exact sections:

```text
Scope and evidence
Planned versus actual work
Failed assumptions and root causes
Verifier findings
Cache and runtime measurements
Publication and recovery behavior
Changes applied to this plan
Rejected changes
```

Every assertion cites a commit, receipt, review, test output, or tracker row.
Update the `Plan evolution` section below with concrete changes supported by
that evidence. Do not weaken the theorem, evidence, or authority boundary. Run
an independent read-only review of the retrospective and plan diff; repair every
Critical or Important finding.

- [ ] **Step 5: Refresh the repository digest and commit final metadata**

Change CPFR-086 to `DONE`, append its `landed` ledger row with the existing
verified payload commit as `landing_commit`, then run `mise run
verify-repository`. Patch `docs/import-receipt.md` with the exact reported
digest, stage it last, and rerun repository verification. Commit:

```sh
git add docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv \
  docs/workstream/crouzeix-proof-reproduction/tracker.org \
  docs/workstream/crouzeix-proof-reproduction/cpfr-086-retrospective.md \
  docs/superpowers/plans/2026-08-24-lorist-schwenninger-route-certification.md \
  docs/import-receipt.md
git commit -m "docs(crouzeix): record LS route landing"
```

- [ ] **Step 6: Fast-forward and validate local master**

From the primary checkout, inventory tracked and untracked state and compare all
incoming added paths against untracked paths. Preserve unrelated files. Run:

```sh
git merge --ff-only feat/cpfr-086-ls-certification
python3 labs/crouzeix_proof_reproduction/proof_evidence.py \
  validate --route lorist-schwenninger
scripts/check_lean_library.sh CrouzeixLoristSchwenninger
mise run verify-repository
python3 labs/crouzeix_proof_reproduction/proof_evidence.py validate-goal
```

Expected: LS is `complete-local`, the focused cached build passes, repository
verification passes, and goal validation names CPFR-087 as the earliest
incomplete phase. Confirm branch divergence is `0 0`. Preserve the worktree.

- [ ] **Step 7: Close the ticket with evidence**

Run `kata close harp#2eb2 --done` with the landing commit and the focused route,
Lean, and repository verification commands as evidence. Confirm `harp#2qy3` is
now ready and remains blocked from having started before this landing.

## Review prompts

### Mathematical Spec review

```text
Review the frozen CPFR-086 candidate read-only against the exact pinned source
at arxiv:2608.03841v1. Recompute the archive and manuscript digests and inspect
all six source spans. Check the exact Lean theorem type and proof body for each
node, actual provider edges after contracting helpers, inner-product orientation,
Equation 1, recurrence, scalar contradiction, perturbation assembly, double-layer
realization, normalized polynomial estimate, limiting argument, and terminal
claim. Check every correspondence_kind. Do not edit, publish, merge, or infer
source fidelity from compilation. Return PASS or FAIL, Critical/Important/Minor
findings, a six-node disposition table, verified commands, unverified claims,
and the allowed next transition.
```

### Standards review

```text
Review the frozen CPFR-086 branch read-only against its approved design and plan.
Attack Python/Rust/schema parity, strict arXiv parsing, source-manifest and
artifact-manifest binding, exact theorem and provider rosters, historical
receipt immutability, bounded reads and subprocess output, symlink/hardlink/path
aliasing, cache identity, one-build/six-audit execution, create-only publication,
atomic graph/inventory promotion, crash recovery, and repository conventions.
Run focused negative probes but do not edit, publish, merge, push, or clean up.
Return PASS or FAIL with Critical, Important, and Minor findings. Critical and
Important findings block landing.
```

### Phase verifier

```text
Independently verify CPFR-086 using only this plan, the candidate worktree,
base/candidate commits, pinned arXiv identities, six node attempts, graph and
inventory promotion marker, route manifest, receipt, review, and tracker ledger.
Recompute hashes, verify immutable history, run the exact focused checks, ensure
the shared cache was not hydrated, and confirm LS reports complete-local. Do not
edit, publish, merge, push, or clean up. Return PASS or FAIL and the allowed next
transition.
```

### Post-execution plan review

```text
Compare this CPFR-086 plan with the committed execution evidence. For every task,
record planned versus actual actions, failures, root causes, command counts,
runtime, cache behavior, review findings, and recovery events. Propose exact
plan edits that would have prevented rework or made a fresh-context restart
safer. Reject edits that weaken source fidelity, theorem scope, evidence
immutability, independent review, human authority, or cache discipline. Return
PASS or FAIL with exact evidence locators and proposed changes.
```

## Stop conditions

Stop for owner direction only when:

1. required progress needs dependency hydration or cache maintenance;
2. the pinned arXiv bytes cannot be reacquired or do not match their registered
   hashes;
3. source review shows that the compiled theorem claim is materially weaker
   than the approved six-node contract and a bridge would widen scope;
4. dirty or divergent work cannot be attributed and preserved;
5. a push, remote publication, persistent service, or authority expansion is
   required; or
6. the same external blocker persists for three consecutive goal turns with no
   meaningful in-scope progress.

Ordinary test failures, review findings, implementation defects, and hard proof
work are not stop conditions.

## Fresh-context resume algorithm

1. Read `AGENTS.md`, this plan, its paired design, the CPFR-086 tracker subtree,
   and the execution ledger.
2. Inspect `master`, the CPFR-086 worktree, Git status, current processes, cache
   link, route artifacts, and attempt directories.
3. Verify prior commits and evidence rather than trusting headings or summaries.
4. Select the first unchecked task whose preconditions hold.
5. If a publication path exists, validate and recover it before running any
   publisher.
6. Run the narrowest test that demonstrates the current transition.
7. Keep all Lean commands serial and use only the primary shared cache.
8. Record each transition in the execution ledger before proceeding.
9. Continue until CPFR-086 is landed or a listed stop condition applies.

## Plan evolution

CPFR-086 execution changed this plan's operating guidance without weakening the
theorem, evidence, authority, source-fidelity, independent-review, or cache
boundaries:

- Expected failures must be phase-aware and fixture-backed. Post-promotion
  tests initially used live state as historical evidence; commit `5422d12`
  replaced that with immutable historical fixtures and phase-aware assertions.
- Review findings must either become regression tests or be explicitly rejected
  with a safety reason before the next gate. Task 6 review findings were fixed
  in commits `259cf6c` and `5e030e6`; the suggestion to treat
  `os.kill(pid, 0)` `PermissionError` as a stale lock remains rejected because
  it could delete a live cross-user lock.
- Sandbox preflight must exercise real tool startup, Git identity, and the
  intended Seatbelt policy before a proof command is treated as meaningful. The
  first sandboxed Lake attempt lacked readable Git/runtime configuration and
  tried forbidden cache replacement; the corrected environment used read-only
  Homebrew Git plus `GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null`
  and produced exactly one remedial sandboxed elaboration of the six theorem
  types while preserving cache identity.
- Final metadata is release-critical. The focused release gate at commit
  `df8da81` passed with route validation `complete-local`, 3,386 cached Lean
  jobs, 739 Python tests passed with 2 skipped, 97 Rust Crouzeix tests passed,
  and unchanged cache snapshot
  `3b8e37fc7ec9e5d8496c56fd89fa58d62293b2bb7fd3ee37276683a08804ce2e`; later
  full `mise run verify` attempts still exposed environment and repository
  verification issues, and current head `d174957` still requires the deliberate
  `docs/import-receipt.md` digest refresh before the final repository gate can
  pass.
- Claim ceilings stay explicit. CPFR-086 may record route claim
  `complete-local`, six node receipts, route receipt digest
  `f672bb002c9d7ebc516683d61ba87b2c1ff2bed891daac0e6781aeb50cc7a01b`, and
  review digest
  `9f3fa1cffafc84b82e64c406cd843f43bbb4b1fc83197d69ed44c3f36d921d0a`. The
  route must not be fast-forwarded until the controller refreshes
  `docs/import-receipt.md`, runs the definitive full repository gate on the
  final metadata commit, and performs the local landing.

## Completion footer

```yaml
phase: cpfr-086
status: complete_local_release_gated_landing_metadata_prepared
route: lorist-schwenninger
claim_level: complete-local
node_receipts: 6
candidate_commit: d1749570825afc277fd96ff6c79b532449afbd16
promoted_payload_commit: 530c645bcfee7ee76591cc44df1ef3b5fdfd00d0
promoted_payload_tree: ee69e85f25e3845ea58b71b13e63c3c9e058b5fd
graph_sha256: ebe63abaf21d2ea620bfaa9be7b154ef39dbe4411535533314c14db264ab3826
inventory_sha256: 97ea21ee0af6c9d0e38cbc566ad0fc46c5f31b2785fb2f575bcc0e23d84c9b8c
route_receipt: f672bb002c9d7ebc516683d61ba87b2c1ff2bed891daac0e6781aeb50cc7a01b
route_receipt_self: b2a82c49454f4c24a29e06f864305ca5b8afba656ae0d5e862651833649e7b3e
review: 9f3fa1cffafc84b82e64c406cd843f43bbb4b1fc83197d69ed44c3f36d921d0a
review_self: b67fbd93714679422cd80c80d78e5b4b76ab03f11042187b74a6a5be4b34196d
manifest_contract_sha256: 38c953c8c70fae2d9e416994ff1684e73f7a63e7e501172fc267e8f706558d7d
mathematical_review: cpfr086-ls-promotion-bind-20260825-c19e PASS no Critical/Important
standards_review: /root/cpfr086_final_standards PASS no Critical/Important/Minor at df8da81 over 11d2820
focused_release_gate: df8da81 preflight ready; 3386 Lean jobs; 739 Python passed and 2 skipped; 97 Rust Crouzeix passed; formatting and diff passed
cache_snapshot: 3b8e37fc7ec9e5d8496c56fd89fa58d62293b2bb7fd3ee37276683a08804ce2e
full_repository_gate: required on final metadata commit before local fast-forward
landing_commit: d1749570825afc277fd96ff6c79b532449afbd16
post_execution_review: PASS Spec and Standards with no findings after legacy-token repair
plan_revision: docs/workstream/crouzeix-proof-reproduction/cpfr-086-retrospective.md
local_fast_forward: pending controller gate
kata_close: pending post-landing evidence
next_phase: cpfr-087
```

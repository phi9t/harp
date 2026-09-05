# Crouzeix Textbook Wave 0 Publication Integrity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace duplicated, string-only textbook validation with one Rust publication module that validates canonical identities, exact prose and Lean correspondence, pedagogical and kernel graphs, and atomically generated outputs.

**Architecture:** Add a deep `crouzeix_textbook` Rust module with strict contract, Markdown, Lean-receipt, and publication boundaries. Migrate contracts to version two, generate the Lean check surface from those contracts, remove duplicated Python/dependency manifests after deletion tests pass, and integrate the publisher into corpus and Atlas generation.

**Tech Stack:** Rust 1.92, Serde/serde_json, pulldown-cmark, Lean 4.32.1 compiler APIs, Mathlib 4.32.1, Markdown, JSON, Vitest, mise.

---

## File structure

```text
crates/harp/src/crouzeix_textbook/
├── mod.rs          # public check/write orchestration
├── contract.rs     # v2 Serde types and semantic validation
├── diagnostic.rs   # stable typed failure surface
├── markdown.rs     # frontmatter, anchor, theorem-card, source validation
├── lean.rs         # check-source generation and receipt validation
└── publish.rs      # deterministic rendering and atomic sibling replacement
crates/harp/tests/
├── crouzeix_textbook.rs
└── fixtures/crouzeix_textbook/valid/...
formalization/lean/CrouzeixTextbook/
├── Correspondence.lean        # generated declaration checks
└── ExportReceipt.lean         # maintained bounded JSON exporter
content/crouzeix_textbook/
├── coverage.json              # v2 theorem contract
├── exercises.json             # v2 exercise contract
└── compatibility_routes.json  # canonical frontmatter ID to legacy route map
```

`theorem_dependencies.json`, `PublicTheorems.lean`, and
`scripts/render_crouzeix_textbook_ledgers.py` are removed only in Task 7 after
replacement behavior has explicit tests.

## Task 1: Freeze the version-two domain model and diagnostics

**Files:**

- Create: `crates/harp/src/crouzeix_textbook/diagnostic.rs`
- Create: `crates/harp/src/crouzeix_textbook/contract.rs`
- Create: `crates/harp/src/crouzeix_textbook/mod.rs`
- Modify: `crates/harp/src/lib.rs`
- Rewrite: `crates/harp/tests/crouzeix_textbook.rs`
- Create: `crates/harp/tests/fixtures/crouzeix_textbook/valid/content/crouzeix_textbook/coverage.json`
- Create: `crates/harp/tests/fixtures/crouzeix_textbook/valid/content/crouzeix_textbook/exercises.json`

- [ ] **Step 1: Add RED tests for strict deserialization and stable errors**

Create tests named `v2_rejects_unknown_and_missing_fields`,
`v2_separates_publication_prose_lean_and_review_status`, and
`diagnostics_name_identity_field_expected_and_observed`. Each test copies the
minimal valid fixture into a fresh `tempfile::TempDir`, mutates one byte-level
field, and calls the production `check` API. Do not point tests at canonical
repository files and do not add an environment-variable override to
production code.

The diagnostic assertion must use this complete public shape:

```rust
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct TextbookDiagnostic {
    pub code: &'static str,
    pub identity: Option<String>,
    pub field: String,
    pub expected: String,
    pub observed: String,
    pub path: Option<std::path::PathBuf>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}
```

Run:

```sh
cargo test -p harp --test crouzeix_textbook v2_ -- --test-threads=1
```

Expected RED: `harp::crouzeix_textbook` and the v2 types do not exist.

- [ ] **Step 2: Implement exact enums and records**

Implement closed Serde enums for:

```rust
#[serde(rename_all = "kebab-case")]
enum PublicationStatus { Draft, Active }
#[serde(rename_all = "kebab-case")]
enum ProseProofStatus { Summary, Reconstructible, NotApplicable }
#[serde(rename_all = "kebab-case")]
enum LeanCorrespondenceStatus { Unmapped, Checkpoint, Exact, NotApplicable }
#[serde(rename_all = "kebab-case")]
enum FormalMode { ProvedHere, ReexportedProof, Definition, Checkpoint, Informal }
```

All contract structs use `#[serde(deny_unknown_fields)]`. `TheoremRow` owns
`item_id`, `chapter`, `kind`, `prose_path`, `anchor`, `source_ids`,
`pedagogical_prerequisites`, the four status values, and an optional
`LeanDeclaration`. `ExerciseRow` owns the fields approved in the design,
including `skills`, `starter`, and a distinct optional `LeanSolution`.
Validate combinations in `TryFrom<RawContracts>` rather than scattering
checks through renderers.

- [ ] **Step 3: Prove GREEN and commit the model**

```sh
cargo fmt --all --check
cargo test -p harp --test crouzeix_textbook v2_ -- --test-threads=1
git diff --check
git add crates/harp/src/crouzeix_textbook crates/harp/src/lib.rs \
  crates/harp/tests/crouzeix_textbook.rs \
  crates/harp/tests/fixtures/crouzeix_textbook
git commit -m "feat(crouzeix-textbook): define publication contracts"
```

### Task 1 review reconciliation — 2026-08-23

After two review cycles, freeze Task 1 at commit `35907a3` on
`codex/crouzeix-textbook`; the worktree is clean. Accepted behavior includes
strict v2 Serde models, accumulated coverage/exercise diagnostics, bounded
regular-file reads, focused stable codes, a two-row isolated fixture, and an
explicit formal-mode matrix. The focused gate is 11/11 v2 tests, 17/17 total
textbook tests, formatting, Clippy, and diff checks.

Two findings remain required Task 1 work rather than follow-ups:

- deserialization diagnostics must use the exact nested Serde path and source
  position; recursive value search may not infer a field from a repeated value;
- prose proof status remains independent of Lean correspondence progress, so a
  reconstructible proved-here or reexported proof may be temporarily unmapped
  during migration.

The next safe action is one bounded repair commit covering only these two
findings and their regression tests, followed by re-review by the same quality
reviewer. Task 2 must not begin until that reviewer reports no Critical or
Important findings. The unrelated Meta-Harness font/manifest byte mismatch
remains outside this task and does not change the focused acceptance gate.

## Task 2: Make frontmatter identity and prose anchors authoritative

**Files:**

- Create: `crates/harp/src/crouzeix_textbook/markdown.rs`
- Modify: `crates/harp/src/crouzeix_textbook/mod.rs`
- Modify: `crates/harp/tests/crouzeix_textbook.rs`
- Create: `content/crouzeix_textbook/compatibility_routes.json`

- [ ] **Step 1: Add RED identity, file-safety, and anchor tests**

Cover duplicate frontmatter IDs, the 38 known frontmatter/corpus mismatches,
missing and duplicate theorem anchors, a symlinked chapter, a hardlinked
chapter, a path escape, a NUL byte, CRLF, a route alias collision, and a
theorem card missing one required field. The parser must use the same heading
slug semantics as corpus rendering, not a second regular-expression slugger.

The required theorem-card subheadings are:

```text
Purpose
Statement
Hypothesis ledger
Proof roadmap
Proof
Boundary case
Pedagogical prerequisites
Lean correspondence
Historical context
ML analogy
```

Run the three most focused tests and require RED:

```sh
cargo test -p harp --test crouzeix_textbook canonical_identity_ -- --test-threads=1
cargo test -p harp --test crouzeix_textbook prose_anchor_ -- --test-threads=1
cargo test -p harp --test crouzeix_textbook rejects_nonregular_ -- --test-threads=1
```

- [ ] **Step 2: Implement allowlisted discovery and compatibility routes**

Discover regular `.md` files only under `knowledge/crouzeix_textbook/` using
`HeldDirectory`; parse frontmatter through the maintained corpus metadata
parser; reject IDs colliding with document IDs, theorem IDs, exercise IDs, or
aliases. Define `compatibility_routes.json` as:

```json
{
  "schema_version": "crouzeix-textbook-compatibility-routes/v1",
  "routes": [
    {
      "canonical_id": "cft-chapter-01-objects-and-representations",
      "legacy_concept_id": "crouzeix-textbook-chapter-01",
      "canonical_path": "knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations.md"
    }
  ]
}
```

Populate all 44 current chapter/support documents from their frontmatter and
old corpus routes. Do not hand-author a second canonical document roster.

- [ ] **Step 3: Prove GREEN and commit identity migration**

```sh
cargo test -p harp --test crouzeix_textbook canonical_identity_ -- --test-threads=1
cargo test -p harp --test crouzeix_textbook prose_anchor_ -- --test-threads=1
cargo test -p harp --test crouzeix_textbook rejects_nonregular_ -- --test-threads=1
git add crates/harp/src/crouzeix_textbook crates/harp/tests/crouzeix_textbook.rs \
  content/crouzeix_textbook/compatibility_routes.json
git commit -m "feat(crouzeix-textbook): adopt canonical frontmatter identity"
```

## Task 3: Validate evidence classes and the pedagogical graph

**Files:**

- Modify: `crates/harp/src/crouzeix_textbook/contract.rs`
- Modify: `crates/harp/src/crouzeix_textbook/markdown.rs`
- Modify: `crates/harp/tests/crouzeix_textbook.rs`
- Modify: `knowledge/crouzeix_textbook/claim_evidence_ledger.md`

- [ ] **Step 1: Add RED tests**

Reject unknown `source_id`, unsupported `DIRECT OBSERVATION`, historical
priority without evidence, absent stable locators, unknown pedagogical nodes,
cycles, and either terminal provider depending on the other. Assert the valid
shape is a common trunk through CFT-29 followed by independent CFT-30..32 and
CFT-33..34 branches, with comparison only in Chapter 35.

- [ ] **Step 2: Implement source-bound context validation**

Accept only ADR-0001 classes `EVIDENCE`, `SOURCE CLAIM`, and `INFERENCE` in the
textbook claim ledger. Keep `review_status` and `reproduction_status` separate.
Change the two endpoint compile facts from `DIRECT OBSERVATION` to `EVIDENCE`
and point them to the immutable route receipts that support the claims.

- [ ] **Step 3: Implement and verify the pedagogical DAG**

Use a deterministic DFS with a reported cycle path. Validate provider policy
after graph validation so an unknown node cannot be misreported as a branch
violation.

```sh
cargo test -p harp --test crouzeix_textbook evidence_ -- --test-threads=1
cargo test -p harp --test crouzeix_textbook pedagogical_graph_ -- --test-threads=1
git add crates/harp/src/crouzeix_textbook crates/harp/tests/crouzeix_textbook.rs \
  knowledge/crouzeix_textbook/claim_evidence_ledger.md
git commit -m "fix(crouzeix-textbook): validate evidence and proof routes"
```

### Task 3 review reconciliation — 2026-08-23

After two review-and-repair cycles, freeze the current Task 3 candidate at
commit `40dde4e` on `codex/crouzeix-textbook`; the worktree is clean. Accepted
behavior includes source-record validation, repository-backed locator
resolution, code-safe multiline field parsing, deterministic full-branch graph
validation, a representative 210-row graph fixture, and immutable receipts for
the proof slices that those receipts actually attest. The focused gate is 15/15
evidence tests, 14/14 pedagogical-graph tests, 66/66 total textbook tests,
58/58 corpus tests, formatting, Clippy, and diff checks.

Five findings remain required Task 3 work rather than follow-ups:

- explicit priority language must cover bounded published-priority phrasings,
  and main and priority locators must be compared by resolved route identity;
- claim sources, including priority sources, must be clickable exact routes to
  source-registry records rather than bare registered IDs;
- every explicit component of a mixed locator must parse and resolve; one valid
  component may not hide an invalid sibling;
- CFT-CL-005 must describe the exact verified Lorist--Schwenninger proof slice
  named by its immutable receipt. The source graph records the full LS terminal
  as blocked, so this task must not relabel an intermediate receipt as terminal
  evidence or manufacture an endpoint claim.

The next safe action is one bounded repair commit covering only these findings
and their black-box regressions, followed by re-review by the same spec and
quality reviewers. Task 4 must not begin until both report no Critical or
Important findings. This reconciliation narrows the earlier shorthand “two
endpoint compile facts” to the exact facts supported by existing immutable
receipts; the eventual full LS endpoint remains work for the LS proof wave.

The repair at `aab9089` closes those five findings and leaves the worktree
clean, with 19/19 evidence tests, 14/14 pedagogical-graph tests, 70/70 total
textbook tests, and 58/58 corpus tests. Its quality re-review identified five
additional Important findings that remain inside Task 3 because they protect
already-approved public and publication contracts:

- source byte offsets must remain parser-internal; the public serialized
  `TextbookDiagnostic` stays at the frozen eight-field Task 1 shape, with an
  exact-key regression;
- every formatted locator component, including a backticked declaration name
  without a slash, must resolve or make the entire locator fail;
- ordinary mathematical uses such as “the first basis vector” and “the first
  proof step” must not trigger historical-priority evidence requirements;
- full-book validation must require Chapters 30--35 and connected provider
  terminals unconditionally, and the selected Jin and LS terminal routes must
  share a Chapter-29 trunk rather than relying on intersected ancestor unions;
- CFT-CL-003 must distinguish two source-backed proof routes from the one
  compiled Jin terminal and the bounded compiled LS intermediate slices.

The duplicate-field source position is a required focused precision fix in the
same parser edit. The next safe action remains one bounded Task 3 repair plus
black-box regressions and re-review. A partial-contract validation mode is not
needed by the approved publication interface and must not be invented here.

## Task 4: Generate and validate the Lean correspondence receipt

**Files:**

- Create: `formalization/lean/CrouzeixTextbook/Correspondence.lean`
- Create: `formalization/lean/CrouzeixTextbook/ExportReceipt.lean`
- Modify: `formalization/lean/CrouzeixTextbook.lean`
- Modify: `formalization/lean/lakefile.toml`
- Create: `crates/harp/src/crouzeix_textbook/lean.rs`
- Modify: `crates/harp/tests/crouzeix_textbook.rs`
- Modify: `scripts/check_lean_library.sh`

- [ ] **Step 1: Add RED receipt tests**

Test missing declarations, stale source positions, type fingerprint drift,
unknown namespace dependencies, provider-crossing dependencies, mode/body
mismatch, changed axioms, oversized receipts, duplicate receipt rows, absolute
paths, and a formal exercise whose solution equals its checkpoint. Also test
that `proved-here` rejects a body that is only a direct constant alias and that
`reexported-proof` requires a substantive underlying declaration.

Use this receipt envelope:

```json
{
  "schema_version": "crouzeix-textbook-lean-receipt/v1",
  "toolchain": "leanprover/lean4:v4.32.1",
  "target": "CrouzeixTextbook",
  "declarations": []
}
```

Each declaration row contains `name`, `kind`, `source_path`, `line`, `column`,
`normalized_type`, `type_sha256`, `direct_dependencies`, and `axioms`.

- [ ] **Step 2: Generate `Correspondence.lean` from contracts**

Generate imports in sorted module order and a `#check` for every applicable
public theorem and exercise solution. The file has a generated warning and no
independent string manifest. Keep `ExportReceipt.lean` maintained: it loads
the compiled environment, traverses constants in types and proof bodies,
normalizes pretty-printing with fixed options, and writes one bounded JSON
receipt to an explicit temporary path supplied on the command line.

- [ ] **Step 3: Integrate the pinned compiler without cache hydration**

Extend `scripts/check_lean_library.sh CrouzeixTextbook` with an optional
`--receipt-output <absolute-temp-file>` accepted only for that target. Resolve
the output before invoking Lean, reject repository paths and symlinks, and
never accept an ambient environment variable for it. The ordinary target
still performs no receipt write.

- [ ] **Step 4: Prove GREEN on fixture and real declarations**

```sh
cargo test -p harp --test crouzeix_textbook lean_receipt_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-jin
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-ls
git add formalization/lean/CrouzeixTextbook formalization/lean/CrouzeixTextbook.lean \
  formalization/lean/lakefile.toml scripts/check_lean_library.sh \
  crates/harp/src/crouzeix_textbook crates/harp/tests/crouzeix_textbook.rs
git commit -m "feat(crouzeix-textbook): verify exact Lean correspondence"
```

### Task 4 review reconciliation — 2026-08-24

After two review-and-repair cycles, freeze the current Task 4 candidate at
commit `6b3864e` on `codex/crouzeix-textbook`; the worktree is clean. Accepted
behavior includes canonical byte-for-byte correspondence generation, 216 typed
compiled-environment receipt rows, exact formal-solution axiom contracts,
compiler-observed direct-alias targets, deterministic bounded output, and
passing CrouzeixTextbook, Jin, and Lorist--Schwenninger gates. The focused gate
is 96 textbook tests, 58 corpus tests, 25 wrapper tests, 21 receipt tests,
formatting, Clippy, and diff checks.

Five Important findings remain Task 4 blockers because they protect the exact
correspondence and filesystem contracts already approved:

- validate every v2 Lean declaration and source path before rendering; reject
  syntax injection, malformed module conversion, unresolved paths, and files
  that are not maintained regular single-linked Lean sources;
- direct-alias recognition must accept only exact eta expansion: every binder
  appears once, in order, with no reordering, duplication, or omission;
- derive Jin and Lorist--Schwenninger provider ownership from the Chapter
  30--34 contract rows and their public/underlying declaration mapping, then
  reject transitive receipt-visible crossings rather than relying on namespace
  prefixes;
- receipt reads must validate the opened descriptor before and after the read,
  including link count, length, device, inode, and regular-file stability;
- receipt creation must require a private caller-owned external parent, avoid
  ambient `TMPDIR` as an authority boundary, create under `umask 077`, and keep
  identity-sensitive validation and cleanup descriptor-bound or protected by
  the private parent contract.

Strict increasing receipt-name order is a required focused determinism fix in
the same validator edit. The next safe action is one bounded Task 4 security
repair with black-box and Lean adversarial regressions, followed by re-review by
the same quality reviewer. Task 5 must not begin until no Critical or Important
findings remain. No partial-contract mode, relaxed path policy, receipt-envelope
expansion, or cache hydration is authorized.

## Task 5: Build deterministic preparation and atomic publication

**Files:**

- Create: `crates/harp/src/crouzeix_textbook/publish.rs`
- Modify: `crates/harp/src/crouzeix_textbook/mod.rs`
- Modify: `crates/harp/src/lib.rs`
- Modify: `crates/harp/src/main.rs`
- Modify: `crates/harp/tests/crouzeix_textbook.rs`

- [ ] **Step 1: Add RED check/write and failure-atomicity tests**

Require `check` to be read-only, `write` to replace every changed sibling, and
a late invalid receipt or blocked output to preserve all previous bytes. Test
symlinked and multiply-linked outputs. The production API is:

```rust
pub enum TextbookPublishMode { Check, Write }

pub struct TextbookPublishResult {
    pub theorem_count: usize,
    pub exercise_count: usize,
    pub exact_correspondence_count: usize,
    pub outputs: Vec<std::path::PathBuf>,
    pub matched: bool,
}

pub fn publish_crouzeix_textbook(
    repo_root: &std::path::Path,
    receipt_path: &std::path::Path,
    mode: TextbookPublishMode,
) -> Result<TextbookPublishResult, AppError>;
```

- [ ] **Step 2: Implement prepare-before-replace**

Render coverage, exercise, pedagogical-dependency, kernel-dependency, status,
and compatibility ledgers into held temporary storage. Validate the entire
prepared set before any replacement, snapshot all destination files, and use
the repository's compare-and-replace discipline. If the filesystem cannot
provide all-or-nothing replacement, stage a generation directory and switch a
single validated manifest pointer rather than claiming transactionality over
independent renames.

- [ ] **Step 3: Add a narrow CLI surface**

Add `harp crouzeix-textbook check --receipt <path>` and
`harp crouzeix-textbook publish --receipt <path>`. The receipt argument is
required; canonical input/output roots are not configurable from the command
line or environment.

- [ ] **Step 4: Verify and commit**

```sh
cargo test -p harp --test crouzeix_textbook publication_ -- --test-threads=1
cargo test -p harp --test cli crouzeix_textbook_ -- --test-threads=1
cargo fmt --all --check
git add crates/harp/src/crouzeix_textbook crates/harp/src/lib.rs \
  crates/harp/src/main.rs crates/harp/tests/crouzeix_textbook.rs crates/harp/tests/cli.rs
git commit -m "feat(crouzeix-textbook): publish verified sibling outputs"
```

### Task 5 review reconciliation — 2026-08-24

After two review-and-repair cycles, freeze the current Task 5 candidate at
commit `feb0062` on `codex/crouzeix-textbook`; the worktree is clean. Accepted
behavior includes deterministic six-ledger generations, one logical manifest
pointer, read-only check mode, bounded FIFO-safe reads, a cooperating
publication lock, post-switch digest readback, and rollback tests. The focused
gate is 109 textbook tests, 58 corpus tests, 8 publication tests, 2 CLI tests,
4 filesystem tests, 2 rollback tests, formatting, Clippy, and diff checks.

Four Important findings remain Task 5 blockers because the approved API claims
an atomic and durable publication boundary:

- replace digest-check-then-blind pathname mutation with descriptor-relative,
  identity-bound pointer exchange/removal and bounded reads; same-bytes inode
  substitution must not satisfy compare-and-replace or rollback;
- hold the generations parent and selected generation as verified directory
  descriptors, enumerate and open all six ledgers relative to those descriptors,
  and bind bytes, digest, size, mode, link count, device, and inode to the same
  opened objects through validation;
- create, normalize, and hold the lock relative to an identity-verified parent,
  enforce caller ownership, mode `0600`, and one link independently of umask,
  and prevent pathname replacement from splitting cooperating writers;
- synchronize the generations parent after generation installation and before
  the pointer switch, then synchronize the pointer parent after the switch so a
  returned success is crash-durable in the declared order.

The repair is explicitly re-sliced:

1. **Task 5a — descriptor-relative publication substrate.** Extract or adapt
   the stronger anchored-directory, snapshot, quarantine/exchange, bounded
   read, ownership/mode, and directory-sync invariants already implemented in
   `context_control::state`; do not continue growing path-only publication
   helpers in parallel.
2. **Task 5b — textbook publication integration.** Rebase `publish.rs` on that
   substrate, require exact `0444` ledger files and validated generation
   directory ownership/mode, preserve the single-pointer design, and add
   deterministic inode-swap, lock-replacement, generation-swap, rollback, and
   sync-order regressions.

Read-only `check` may use a shared lock or bounded retry to avoid reporting a
mixed cooperating publication. Orphaned unreachable generations remain honest
pre-switch artifacts, never current state. The next safe action is this bounded
Task 5a/5b repair followed by re-review by the same quality reviewer. Task 6
must not begin until no Critical or Important findings remain. No Task 6
migration, relaxed durability claim, or cache work is authorized.

The descriptor-relative repair at `a5b229e` closes the four findings above and
leaves the worktree clean, but its decisive quality review identified three
additional Important lifecycle requirements that remain within Task 5a/5b:

- a staged generation owns its identity-bound directory immediately after
  `mkdirat`; every pre-rename error must remove its staged children and
  directory and synchronize the generations parent. Only a successfully
  installed complete digest generation may remain as an honest orphan;
- gate the secure descriptor implementation to macOS and Linux and provide the
  same crate-internal API as typed `fs.unsupported` stubs on every other target;
- define the durable pointer-parent sync as the commit point. Cleanup of the
  displaced old pointer after that point may report a committed/recoverable
  cleanup condition or remain best-effort, but it must not return an ordinary
  failure implying that the prior pointer was preserved. Every real sync
  boundary remains traceable and fault-injectable.

The same bounded repair must synchronize explicit staged-file cleanup, label
same-UID lock-ignoring races as outside the cooperating-writer guarantee, and
validate exact modes only on Harp-owned publication directories rather than on
unrelated repository ancestors. Consolidating `context_control::state` onto
the new descriptor core is a separable follow-up after this program; Task 5
must not widen into that migration now.

The Task 5 candidate at `a6994dd` passes its specification review, but the
original quality reviewer found four remaining Important publication-integrity
gaps. They are reconciled as Task 5a/5b contract work, not Task 6 or a scope
expansion:

- the unsupported descriptor fallback must compile every descriptor method and
  signature used by production publication, including bounded hooked reads;
- the durable pointer-parent `fsync` is the commit point, so no fallible
  readback after it may turn a committed replacement or initial install into an
  ordinary failure result;
- staged file and directory ownership must remain cleanup-capable until the
  complete returned owner is constructed, with explicit faultable parent-sync
  cleanup for setup failures after filesystem creation; and
- rollback after post-switch validation failure must trace and fault every
  rollback file, parent, and cleanup synchronization as
  `PointerRollbackSync`, preserving the previous/absent pointer or a later
  writer according to the existing identity-bound policy.

Repair only these four Task 5 surfaces, strengthen their focused regressions,
and re-run the same scoped publication, filesystem, textbook, CLI, corpus,
formatting, Clippy, and diff gates. Task 6 remains forbidden until the original
quality reviewer reports no Critical or Important findings.

## Task 6: Migrate canonical contracts truthfully

**Files:**

- Modify: `content/crouzeix_textbook/coverage.json`
- Modify: `content/crouzeix_textbook/exercises.json`
- Modify: all 44 Markdown frontmatter blocks under `knowledge/crouzeix_textbook/`
- Modify: `knowledge/crouzeix_textbook/status_and_scope.md`

### Task 6 scope reconciliation — 2026-08-24

The version-two Markdown validator implemented before the canonical migration
currently requires all ten exact-proof theorem-card subsections for every row.
That contradicts the approved Task 6 baseline: a row with
`prose_proof_status=summary` is intentionally proof-exposition-incomplete until
its content wave. Task 6 must not fabricate theorem-card scaffolds or deepen
later-wave prose to satisfy this premature gate.

Before migrating canonical data, add one focused regression proving that a
summary row still requires exactly one stable prose anchor but does not yet
require the ten reconstructible-proof subsections. Restrict the validator's
subsection gate to `prose_proof_status=reconstructible`; keep path, anchor,
identity, source, and heading validation unchanged. Commit this narrow prior
validator defect repair separately, then continue the canonical data migration
described below.

### Task 6 generated-correspondence reconciliation — 2026-08-24

Task 4 made `formalization/lean/CrouzeixTextbook/Correspondence.lean` a derived,
byte-for-byte-checked rendering of the canonical coverage and exercise
contracts. The Task 6 version-two migration changes which public declarations,
underlying proof providers, and distinct exercise solutions the canonical
contracts require. Therefore the existing Task 4 generated file becomes stale
as a direct consequence of the approved Task 6 data migration.

Regenerate and commit `Correspondence.lean` with the canonical Task 6 contract
changes, then compile its fresh receipt without hydrating the Lean cache and
populate source positions and type fingerprints only from that compiled
receipt. This is maintenance of Task 4's derived-file invariant, not Task 7
drift: Task 6 does not change the generator or aggregate imports, and it does
not delete or redirect any legacy validator, manifest, or caller reserved for
Task 7.

### Task 6 legacy-test reconciliation — 2026-08-24

The full Task 6 integration gate still contains four assertions tied to the
version-one canonical field and completion model. Three require the removed
`book_status` field or version-one row shapes, and the legacy correspondence
injection test expects its pre-v2 diagnostic code. Leaving these assertions
unchanged would make the required Task 6 gate reject the approved truthful v2
baseline.

Migrate only those four assertions during Task 6. They must check the v2
incomplete baseline, exact 210 theorem and exercise ID sets, six distinct
Chapter 1 solutions, and the common-trunk/Jin/Lorist--Schwenninger branched
graph. Preserve the legacy generator and its injection-safety test, but assert
the current v2 diagnostic contract. Deleting that generator, its callers, or
the remaining duplicate v1 helpers stays reserved for Task 7.

### Task 6 review-fix cycle 1 — 2026-08-24

The first Task 6 specification review found two Important Step 2 gaps in the
canonical migration. The dependency rewrite retained a near-linear synthetic
chain: 208 of 210 rows have exactly one prerequisite, and most same-chapter
items are serialized by CFT identifier rather than mathematical use. The prose
rewrite also left only 57 distinct theorem targets because Chapters 7--35 map
six rows at a time to the shared `the-six-item-spine` heading.

Repair both gaps within Task 6. Replace the synthetic chain with an acyclic
concept graph whose fan-out, fan-in, roots, and named semantic edges reflect
actual mathematical prerequisites across the common trunk and the independent
Jin and Lorist--Schwenninger routes. In particular, multiplier powers must
precede the corresponding compressed power-moment result. Add structural tests
that reject a near-linear graph as well as named semantic-edge tests; preserve
all 210 IDs, route reachability, the Chapter 29 branch point, and the Chapter 35
join.

The existing edge-order validator must consequently stop treating numeric
order within one chapter as mathematical dependency order. Continue rejecting
dependencies from a later chapter, unknown nodes, and cycles; allow a
same-chapter forward CFT edge when the resulting semantic graph is acyclic.

Give every theorem row its own stable CFT-qualified anchor in the existing
canonical Markdown and update the contract to 210 unique `(prose_path, anchor)`
targets. These are lightweight locator headings only: do not add later-wave
proof content or change exercise anchors and frontmatter. Extend the canonical
integration tests to require unique resolution. This review fix does not alter
Lean declarations, regenerate correspondence, delete Task 7 files, or deepen
another content wave.

### Task 6 review-fix cycle 2 — 2026-08-24

The second Task 6 quality review found three release-blocking contradictions in
the truthful version-two baseline. Reader-facing knowledge ledgers still
describe the retired version-one contract: they advertise 212 public
declarations, 210 exercise solutions, universal compiled-Lean coverage, and the
old linear dependency chain. The Task 5 immutable publication is also absent or
stale for the current version-two contracts, so its check command fails even
with the valid 320-row receipt. Generic production validation additionally
tries to infer semantic graph quality from aggregate degree thresholds, while
the current canonical graph uses artificial edges to satisfy that heuristic.
Finally, nine orphaned version-one test constants and helpers make the required
Clippy gate fail under `-D warnings`.

Treat all three findings as Task 6 truthful-baseline and release-gate work. The
version-two coverage and exercise contracts remain the sole structured
authorities. Task 5's Rust publisher remains the sole owner of immutable Atlas
publication generations and must publish, then check with zero drift, from a
fresh compiled receipt. The retained Python reader-ledger renderer remains the
minimal owner of `lean_coverage_ledger.md`, `theorem_dependency_map.md`, and
`exercise_index.md` until Task 7 deletes that legacy path; migrate it to consume
the version-two coverage and exercise contracts directly, including
pedagogical prerequisites from coverage, without maintaining or consulting a
second dependency schema. Update the canonical claim ledger's obsolete local
compilation statement in place. Do not delete the renderer, the legacy
dependency manifest, its callers, or any other Task 7 path in this repair.

Replace the production degree heuristic with only generic referential,
boundedness, chapter-order, and acyclicity checks. Re-author and audit the
canonical 210-node graph from actual concept use, allowing an explicit reviewed
set of genuine mathematical roots. Canonical integration tests, rather than the
generic validator, own the reviewed chapter-dependency policy, root roster,
proof-critical named edges, independent Jin and Lorist--Schwenninger branches,
Chapter 35 join, and a stable graph fingerprint. Remove artificial root-forcing
edges, including the unrelated determinant dependency of complex
differentiability and the normed-space dependency of positive-real analytic
functions.

Delete only the nine demonstrably unused version-one test symbols reported by
Clippy. Preserve the retained legacy generator and its injection-safety test.
Require the full textbook and corpus tests, the Crouzeix textbook Lean target
and fresh receipt, publisher write followed by a matching check, strict Clippy,
formatting, and diff gates before the Task 6 repair commit.

### Task 6 review-fix cycle 3 — 2026-08-24

The final Task 6 quality review found that the committed Task 5 runtime
publication is not portable through Git. The secure publisher requires the
pointer and persistent lock to be regular owner-only `0600` files and every
immutable generation ledger to be `0444`, while Git records only the executable
bit and recreates these non-executable paths as `0644` in a clean checkout.
Tracking the runtime tree therefore makes repository state contradict the
publisher's exact-mode contract even when the original worktree passed.

Keep the Task 5 security boundary unchanged: do not relax any exact mode,
owner, link, descriptor, or atomic-switch check. Treat
`atlas/src/content/generated/crouzeix_textbook/publication/` as regenerable
runtime state, add an exact root-qualified ignore rule for that tree, and remove
its current pointer, lock, and generation ledgers from the Git index. The
canonical version-two contracts, human knowledge ledgers, publisher code, and
fresh compiled receipt remain the authorities from which an explicit
`crouzeix-textbook publish --receipt ...` recreates the runtime state safely.

Add a clean-checkout lifecycle regression that begins with no publication
tree, proves check mode is read-only and reports the missing generation as not
matched, proves the explicit write creates the complete tree with the existing
exact modes, and proves a subsequent check passes without changing any bytes or
metadata. Add an integration assertion that the repository index contains no
tracked runtime publication path and that the precise ignore rule applies.
Update only focused workflow/test expectations needed to state that publisher
write precedes check and static consumption; do not perform Task 7 deletion or
make corpus compilation create runtime state.

### Task 6 review-fix cycle 4 — 2026-08-25

The final Wave 0 prose review found that canonical chapters outside Chapter 1
still describe related compiled declarations as if they were distinct checked
exercise solutions or reviewed exact prose correspondences. That contradicts
the truthful version-two contracts: all 204 exercises outside Chapter 1 have a
null `lean_solution`, and no theorem row is currently classified as exact
prose-to-Lean correspondence.

Repair only these reader-facing truthfulness claims. Audit all 44 canonical
textbook documents against `coverage.json` and `exercises.json`; preserve the
mathematical content, motivation, and existing Lean navigation links. Where an
exercise has no solution declaration, label any nearby compiled theorem as a
related navigation checkpoint and state that no distinct checked Lean solution
is claimed. Where prose is summary/checkpoint or unmapped, do not imply that
the prose statement is an exact reviewed implementation. Chapter 20's proof
sequence remains an informal roadmap with a related compiled theorem, not an
exact correspondence claim.

Add a bounded contract-derived regression for these overstatement classes. It
must distinguish Chapter 1's six genuine solution declarations from the 204
null solutions and avoid global phrase bans that would reject legitimate exact
or solved rows in later content waves. Do not change contracts, Lean sources,
receipts, Atlas output, or Task 7--9 ownership in this repair.

### Task 6 review-fix cycle 5 — 2026-08-25

The accepted prose repair exposed a verification defect in its regression: it
recognizes editorial sentences and blacklists phrases rather than validating
the structured correspondence contract. This violates the repository rule
that tests verify behavior and contracts instead of source wording.

Replace that regression with a structural projection check. Canonical chapter
frontmatter records counts for distinct checked Lean exercise solutions and
reviewed exact prose correspondences. The test derives the authoritative
counts from `exercises.json` and `coverage.json`, parses the frontmatter fields,
and requires equality chapter by chapter. This preserves Chapter 1's six real
solution declarations, the 204 null solutions elsewhere, and the current zero
exact-correspondence baseline without constraining editorial wording. Preserve
the accepted prose body verbatim; add only the structured frontmatter markers.
Do not change contracts, Lean sources, receipts, Atlas output, or Task 7--9
ownership.

### Task 6 review-fix cycle 6 — 2026-08-25

The structural marker regression still derives its chapter-to-prose mapping by
collecting coverage rows into a `BTreeMap`. Rows for the same chapter silently
overwrite one another, so swapping equal-count chapter targets can leave the
test reading the wrong Markdown while still matching aggregate counts.

Use the authoritative `CHAPTERS` roster, indexed by chapter number, as the
canonical prose identity. Require every coverage row for a chapter to use that
roster path before comparing the chapter markers with the coverage and exercise
contracts. Add a mutation regression that swaps complete `(prose_path, anchor)`
targets between two equal-count chapters and proves the structural projection
rejects the mismatch. Preserve all accepted prose, markers, canonical contracts,
Lean sources, receipts, and Atlas output.

### Task 6 review-fix cycle 7 — 2026-08-25

The final Wave 0 review found two residual Chapter 27 sentences that describe
the displayed quadratic-barrier derivation and the six compiled declarations
as exact formal checks. That contradicts Chapter 27's version-two baseline:
all six prose rows are summaries with unmapped Lean correspondence, and its
exact-correspondence marker is zero.

Repair only those claims. Label the displayed calculation as an informal
textbook derivation and the compiled declarations as related navigation
checkpoints, while preserving every equation, theorem link, exercise, and
frontmatter count. Give the chapter a stable labeled Lean-correspondence
disclosure and validate that structured label together with the Chapter 27
coverage modes; do not restore editorial phrase blacklists. Do not modify
contracts, Lean sources, receipts, Atlas output, or other chapter prose.

- [ ] **Step 1: Convert v1 rows mechanically without inflating claims**

Preserve all 210 CFT IDs and 210 exercise IDs. Map current aliases to
`reexported-proof` only when the public type exactly equals a substantive
underlying proof; map orientation-only aliases to `checkpoint`. Set theorem
prose to `summary` and correspondence to `checkpoint` or `unmapped` until its
content wave upgrades it. Keep chapters `active`; set whole-book exact
correspondence incomplete.

- [ ] **Step 2: Replace generic anchors and synthetic dependencies**

Give every row its existing exact theorem/exercise anchor or add a stable CFT
anchor to the current prose. Author the common-trunk/Jin/LS pedagogical graph;
do not copy the prior 209-edge chain. Populate Lean source paths and
fingerprints only from the compiled receipt.

- [ ] **Step 3: Add distinct solution declarations only where they exist**

Chapter 1 retains its six existing solution theorems. All other formal
exercise rows remain correspondence-incomplete until their content wave adds
distinct solutions; none may reuse the studied checkpoint as a solution.

- [ ] **Step 4: Verify the truthful baseline and commit**

```sh
cargo test -p harp --test crouzeix_textbook -- --test-threads=1
git diff --check
git add content/crouzeix_textbook/coverage.json content/crouzeix_textbook/exercises.json \
  content/crouzeix_textbook/compatibility_routes.json knowledge/crouzeix_textbook
git commit -m "feat(crouzeix-textbook): migrate truthful correspondence data"
```

## Task 7: Delete duplicate validators and manifests

**Files:**

- Delete: `content/crouzeix_textbook/theorem_dependencies.json`
- Delete: `formalization/lean/CrouzeixTextbook/PublicTheorems.lean`
- Delete: `scripts/render_crouzeix_textbook_ledgers.py`
- Modify: `formalization/lean/CrouzeixTextbook.lean`
- Modify: `crates/harp/tests/crouzeix_textbook.rs`
- Modify: `mise.toml`

- [ ] **Step 1: Add deletion tests**

Require the suite to pass when each old file is absent and fail if any caller
still reads it. Require generated pedagogical output to come solely from
`coverage.json`, generated Lean checks solely from contracts, and ledgers
solely from the Rust publisher.

- [ ] **Step 2: Remove the three duplicates and update callers**

Import generated `CrouzeixTextbook.Correspondence` from the aggregate. Replace
the Python render task with the Rust CLI. Do not retain compatibility wrappers
that reimplement schema rules.

- [ ] **Step 3: Verify the deletion test and commit**

```sh
rg -n "theorem_dependencies|PublicTheorems|render_crouzeix_textbook_ledgers" \
  --glob '!docs/**' .
cargo test -p harp --test crouzeix_textbook deletion_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
git add content/crouzeix_textbook formalization/lean/CrouzeixTextbook.lean \
  formalization/lean/CrouzeixTextbook scripts crates/harp/tests/crouzeix_textbook.rs mise.toml
git commit -m "refactor(crouzeix-textbook): remove duplicate publication paths"
```

Expected `rg`: documentation references only.

### Task 7 quality-review reconciliation — 2026-08-24

The first Task 7 quality review found three Important ownership and release-path
gaps. Repair them without entering Task 8 or changing the secure publication
format.

First, the Rust publisher must be the only owner of derived theorem, exercise,
and pedagogical-dependency tables. Keep the three canonical knowledge document
identities and useful reader orientation, but remove their replicated rows,
counts, roots, and graph fingerprint. Turn them into concise reader guides that
route to the immutable Rust-published ledgers and explain the explicit
fresh-receipt publish/check workflow. Add a regression that rejects derived
Markdown tables or rosters in those guides, and behaviorally prove the
published ledger bytes change from canonical contract mutations.

Second, wire the release/static path so a fresh compiled receipt is published
and checked before Atlas consumes canonical inputs. Make `verify-atlas` depend
on the existing non-recursive `crouzeix-textbook-publication` task and add an
ordering regression that checks the actual Atlas release task dependency, not
only commands inside the publication task. Preserve the ignored runtime tree
and its exact modes.

Third, replace the deletion guard's unbounded tracked-file scan with a bounded
enumeration and bounded reads. Include canonical knowledge, exclude only the
allowed documentation tree and exact Task 8-owned generated Atlas artifacts,
and recognize retired basenames, slash and backslash paths, dotted Lean
imports, and relative spellings without rejecting the unrelated Mathematical
Foundations manifest. Add focused temporary-fixture tests for each spelling and
for file-count and byte-limit rejection.

Run the deletion, full textbook, corpus, applicable CLI and Atlas tests, the
Crouzeix textbook Lean target without cache hydration, the fresh-receipt
publication workflow, strict Clippy, formatting, and diff gates. Do not
regenerate Task 8 corpus or Atlas artifacts, merge, or push.

### Task 7 v2-only correspondence reconciliation — 2026-08-25

The Wave 0 integration review found that the generated-correspondence entry
point still falls back to a second version-one parser whenever canonical v2
validation fails. Task 6 retained that compatibility path only until Task 7;
leaving it live now duplicates schema rules and lets invalid or v1 contracts
select a different generator instead of failing closed.

Remove the legacy coverage and exercise types, schema constants, parsing
helpers, declaration sanitizer, and fallback dispatch from
`crates/harp/src/crouzeix_textbook/lean.rs`. Replace the legacy injection test
with explicit v2-only regressions proving that version-one and otherwise
invalid inputs return the typed canonical diagnostics and never generate Lean.
The valid v2 generator and checked-in `Correspondence.lean` bytes must remain
unchanged. Run focused generator and deletion tests, the full textbook and
corpus suites, the Crouzeix textbook Lean target without cache hydration, and
strict formatting, Clippy, and diff gates. Do not regenerate Atlas, prose, or
the import receipt unless canonical generated bytes genuinely change.

## Task 8: Integrate corpus and Atlas with canonical identities

### Task 8 scope reconciliation — 2026-08-24

Canonical textbook identities are selected while corpus links are rendered,
so replacing the per-chapter roster with validated packet registration also
requires the narrow `crates/harp/src/corpus/render.rs` change that threads the
discovered path-to-identity map through document and link routing. This is a
forced corpus integration path, not a new prose or publication owner.

The static Atlas exporter regenerates `atlas/dist/harp-atlas.html` and its
digest-bearing `atlas/dist/harp-atlas.receipt.json` as one verified export.
Treat those two derived files as an inseparable Task 8 output pair; committing
new HTML with a stale receipt would violate the decoded static-export gate.

### Task 8 review-repair scope reconciliation — 2026-08-24

The first Task 8 implementation changed 38 document identities without
publishing their legacy aliases to Atlas, so old `#documents/...` hashes no
longer resolve. The repair adds one bounded, typed alias projection to the
canonical corpus contract and canonicalizes those hashes in the Atlas router;
it does not duplicate documents or retain legacy IDs as document authority.

Corpus must not independently rediscover or reinterpret the textbook packet.
The maintained `crouzeix_textbook` module will expose a read-only validated
registration snapshot containing canonical IDs, legacy aliases, paths, and
already-opened Markdown. Corpus will consume that snapshot after the existing
held-directory, frontmatter, bijection, and namespace checks, without invoking
publisher writes or defining a second compatibility schema and validator.

Because route behavior must survive the complete static-export pipeline, the
repair also extends `atlas/tests/static-export.test.mjs` to inspect the decoded
artifact for the CFT theorem anchors, canonical Lean-ledger route, and embedded
legacy-to-canonical aliases. The corpus JSON, Atlas HTML, and Atlas receipt
remain one regenerated derived-output set.

### Task 8 review-repair cycle 2 scope reconciliation — 2026-08-24

The registration snapshot must hold the filesystem objects it validates, not
reconstruct their names beneath a previously checked root path. This repair may
therefore deepen the shared secure filesystem boundary with descriptor-relative
directory traversal and already-opened bounded file snapshots. The supported
implementation will use held parent and child descriptors, no-follow opens,
regular-file identity and link-count checks, and deterministic replacement-race
tests; the unsupported implementation will retain matching API shape and typed
failure behavior.

Textbook identities and aliases are a projection into the complete compiled
document namespace. Corpus will validate their strict Atlas-compatible grammar
and collisions only after coverage, auxiliary, lesson, reader, and dynamically
registered identities are known. For registered textbook paths, the validated
canonical registration remains authoritative even if another input later names
the same path.

Adding required document aliases changes the canonical corpus contract. The
repair adopts `rsi-technical-atlas/v6` as the sole emitted and accepted schema;
v5 remains a historical stable contract and is rejected rather than silently
reinterpreted. Rust, TypeScript, fixtures, product-contract documentation, and
the corpus/HTML/receipt derived set will advance together, with explicit v5
rejection and v6 acceptance tests.

**Files:**

- Modify: `crates/harp/src/corpus/mod.rs`
- Modify: `crates/harp/src/corpus/contracts.rs`
- Modify: `crates/harp/src/corpus/tests.rs`
- Modify: `atlas/src/content/canonical.test.ts`
- Modify: `atlas/src/app/math.test.ts`
- Regenerate: `atlas/src/content/generated/corpus.json`
- Regenerate: `atlas/dist/harp-atlas.html`

- [ ] **Step 1: Add RED discovery and compatibility tests**

Assert that all 44 textbook documents use frontmatter IDs in corpus output,
all legacy concept routes resolve to those documents, theorem anchors and Lean
links survive static export, and no hard-coded per-chapter roster remains in
`corpus/mod.rs`.

- [ ] **Step 2: Consume publisher registration data**

Replace `CROUZEIX_TEXTBOOK_DOCUMENTS`-style constants with validated discovery
results. Keep corpus compilation read-only; publication must already have
validated generated ledgers.

- [ ] **Step 3: Regenerate and run focused presentation gates**

```sh
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo test -p harp --test crouzeix_textbook -- --test-threads=1
cd atlas && corepack pnpm run test && cd ..
PATH=/opt/homebrew/bin:$PATH mise run verify-atlas
git add crates/harp/src/corpus crates/harp/tests/crouzeix_textbook.rs \
  atlas/src/content/canonical.test.ts atlas/src/app/math.test.ts \
  atlas/src/content/generated/corpus.json atlas/dist/harp-atlas.html
git commit -m "feat(crouzeix-textbook): publish canonical textbook routes"
```

## Task 9: Freeze Wave 0

### Task 9 prerequisite reconciliation — 2026-08-25

The complete gate exposed a pre-existing race in the test expectation for
concurrent completion publication. The implementation preserves exactly one
receipt, but the losing writer can truthfully report either `state.exists`
after acquiring the target lock or `state.lock` when bounded lock acquisition
expires. The textbook branch did not change the episode or state modules.
Import the existing single-path master correction
`0c0909092c71a5d7957ea5e34ac0f25da69153fc` unchanged before freezing Wave 0.
This is a test-only prerequisite that accepts both legitimate losing outcomes;
it does not change runtime behavior, textbook claims, or publication payload.
Keep that upstream commit separate from this reconciliation and from the final
receipt-only commit.

### Task 7 integration reconciliation — indexed content versus LFS materialization

The Task 9 gate also exposed a Task 7 deletion-guard boundary error. The guard
currently enumerates tracked paths from Git but reads their working-tree bytes.
That makes a valid materialized Git LFS artifact appear to be an oversized
tracked source even though the reviewed index object is the small LFS pointer.

Keep the existing tracked-file count and per-blob byte limits, but apply them
to committed index blobs. Resolve each enumerated path through explicit Git
index plumbing, inspect the blob size before reading it, and then read only the
bounded blob. Add a regression that indexes a small LFS-like pointer, replaces
only its working-tree bytes with a materialization larger than the limit, and
proves the guard still scans the index object. Preserve the existing rejection
for a genuinely oversized indexed blob and all retired-name, knowledge, and
Mathematical Foundations identity coverage. This repair must not modify or
stage the Task 9 import receipt.

- [ ] Generate a fresh receipt in a bounded temporary file, run publisher
  write followed by publisher check, and verify the worktree tracks neither
  the receipt nor the regenerable runtime publication tree.
- [ ] Run the complete gate:

```sh
PATH=/opt/homebrew/bin:$PATH mise run verify
git diff --check
git status --short
```

- [ ] Review the complete diff for accidental claims of exact correspondence;
  the expected baseline is intentionally incomplete outside already genuine
  Chapter 1 proofs.
- [ ] Refresh `docs/import-receipt.md` from `harp repository verify`, rerun its
  verifier, and commit only the receipt:

```sh
git add docs/import-receipt.md
git commit -m "chore(repository): refresh import receipt for textbook publisher"
```

Wave 0 exits only when all publication rules are enforced by the Rust module,
all duplicate contract paths are gone, and the full gate passes without
claiming that content Waves 1–6 are already complete.

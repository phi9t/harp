# Harp Context Control Plane Observation Milestone Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use phi9t_stack parallel-agents
> (recommended) or phi9t_stack plan to implement this plan task-by-task. Steps
> use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first useful Harp context-control milestone: secure local
state, all four workflow packages, immutable baseline releases, deterministic
context selection, provider capability probes, and observation runs through
both Trae CLI and Codex CLI.

**Architecture:** Harp embeds reviewed workflow packages in the Rust binary,
compiles them into content-addressed releases under `HARP_HOME`, resolves a
bounded provider-neutral context bundle, and launches either CLI through the
shared `exec --json` surface. A run publishes its manifest before inference,
tees provider stdout and stderr byte-for-byte into private episode storage, and
seals a separate completion receipt without changing provider configuration,
repository rules, sandbox policy, or approvals.

**Tech Stack:** Rust 1.92, Clap, Serde/Serde JSON, SHA-256, Git subprocesses,
`tempfile`, `tar`, `flate2`, `wait-timeout`, `signal-hook`, `assert_cmd`,
repository JSON contracts, and fake provider executables in integration tests.

---

## Scope

This plan implements tracker items `HCCP-001` through `HCCP-005`:

- confirm the product boundary and public schema vocabulary;
- build secure local state and immutable releases;
- add all four workflow packages and deterministic context resolution;
- implement provider doctor and capability snapshots; and
- implement `harp run` with raw episode capture for both providers.

It does not implement normalized provider events, human outcome labels,
sanitized learning export, ACE suggestions, MCE suggestions, App Server
adapters, resume, promotion, canarying, or rollback. Those remain
`HCCP-006` through `HCCP-011`.

## Execution Setup

Implementation must not begin on `master`. Create an isolated worktree after
the plan commit:

```sh
git worktree add .worktrees/context-control-observation \
  -b feat/context-control-observation master
```

Record this ownership in `tracker.org` before the first code change:

```org
:BRANCH: feat/context-control-observation
:WORKTREE: .worktrees/context-control-observation
:OWNER: primary
```

Use focused commits. Every commit message in this plan ends with:

```text
Co-authored-by: TRAE CLI <noreply@bytedance.com>
```

The standalone payload digest in `docs/import-receipt.md` will be stale between
intermediate commits. Run focused tests for each commit. Refresh the receipt
only after the complete milestone surface is staged, then run `mise run verify`
before the final documentation commit.

## Locked File Structure

Create these focused Rust modules:

```text
crates/harp/src/context_control/
├── mod.rs                 public facade and schema-version constants
├── schema.rs              shared wire enums and receipts
├── canonical.rs           canonical JSON and SHA-256 helpers
├── workflow.rs            embedded package parsing and validation
├── routing.rs             deterministic workflow routing
├── context.rs             bounded item selection and Markdown rendering
├── policy.rs              strict .harp/context-control.json parsing
├── repository_state.rs    Git identity and pre/post state digests
├── state.rs               secure HARP_HOME directories and publication
├── release.rs             immutable baseline release compilation
├── episode.rs             running manifest, raw paths, and completion receipt
├── run.rs                 end-to-end orchestration
└── provider/
    ├── mod.rs             ProviderAdapter domain interface
    ├── probe.rs           bounded version/help capability probes
    ├── invocation.rs      provider-safe argument and prompt construction
    └── process.rs         streaming process execution and signal forwarding
```

Add integration tests without enlarging the existing general CLI test file:

```text
crates/harp/tests/context_control_cli.rs
crates/harp/tests/context_control_run.rs
crates/harp/tests/support/context_control.rs
```

Track source packages here and embed them with `include_bytes!`:

```text
content/context_control/workflows/
├── ci_repair/
├── code_review/
├── dependency_update/
└── general_coding/
```

Each workflow directory contains exactly:

```text
manifest.json
routing.json
context_schema.json
verification.json
outcome_rules.json
playbook.jsonl
```

Target repositories do not need a Harp checkout. The runtime reads embedded
package bytes; the tracked files remain the review and verification authority.

## Public Wire Vocabulary

Use these exact schema identifiers:

| Object | Schema |
|---|---|
| Workflow manifest | `harp-workflow/v1` |
| Routing rules | `harp-routing/v1` |
| Context schema | `harp-context-schema/v1` |
| Verification rules | `harp-verification/v1` |
| Outcome rules | `harp-outcome-rules/v1` |
| Context item | `harp-context-item/v1` |
| Release identity | `harp-context-release-identity/v1` |
| Release manifest | `harp-context-release/v1` |
| Context bundle | `harp-context-bundle/v1` |
| Provider capabilities | `harp-provider-capabilities/v1` |
| Repository policy | `harp-repository-policy/v1` |
| Repository snapshot | `harp-repository-snapshot/v1` |
| Episode manifest | `harp-episode/v1` |
| Provider invocation | `harp-provider-invocation/v1` |
| Completion receipt | `harp-episode-completion/v1` |
| Preflight failure | `harp-preflight-failure/v1` |

Use these exact enum wire values:

```rust
pub enum ProviderId {
    Trae,
    Codex,
}

pub enum WorkflowId {
    CiRepair,
    CodeReview,
    DependencyUpdate,
    GeneralCoding,
}

pub enum WorkflowChoice {
    Auto,
    Workflow(WorkflowId),
}

pub enum ContextItemKind {
    RepositoryInvariant,
    DiagnosticRule,
    WorkflowStep,
    VerificationRecipe,
    AntiPattern,
    ToolUsageRule,
    ArchitectureFact,
    ReviewPreference,
    Example,
    Exception,
}
```

Serde wire values are snake case: `trae`, `codex`, `ci_repair`,
`code_review`, `dependency_update`, and `general_coding`.

---

### Child Session 1: Public Contracts And CLI Skeleton

**Tracker:** `HCCP-001`

**Files:**

- Create: `crates/harp/src/context_control/mod.rs`
- Create: `crates/harp/src/context_control/schema.rs`
- Modify: `crates/harp/src/lib.rs`
- Modify: `crates/harp/src/main.rs`
- Modify: `crates/harp/tests/context_control_cli.rs`
- Modify: `README.md`
- Modify: `docs/product-contract.md`

- [ ] **Step 1: Write failing wire-name and CLI-shape tests**

Add unit tests to `schema.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_and_workflow_ids_have_stable_wire_names() {
        assert_eq!(serde_json::to_string(&ProviderId::Trae).unwrap(), "\"trae\"");
        assert_eq!(serde_json::to_string(&ProviderId::Codex).unwrap(), "\"codex\"");
        assert_eq!(
            serde_json::to_string(&WorkflowId::DependencyUpdate).unwrap(),
            "\"dependency_update\""
        );
        assert_eq!("code_review".parse::<WorkflowId>().unwrap(), WorkflowId::CodeReview);
        assert!("unknown".parse::<WorkflowId>().is_err());
    }
}
```

Add integration assertions to `context_control_cli.rs`:

```rust
#[test]
fn context_control_commands_are_exposed_without_running_a_provider() {
    harp().args(["providers", "--help"]).assert().success();
    harp().args(["releases", "--help"]).assert().success();
    harp().args(["run", "--help"]).assert().success();
}
```

- [ ] **Step 2: Run the narrow tests and confirm failure**

Run:

```sh
cargo test -p harp context_control --lib
cargo test -p harp --test context_control_cli
```

Expected: compilation fails because `context_control`, the enums, and CLI
subcommands do not exist.

- [ ] **Step 3: Add the exact public enums and command skeleton**

In `schema.rs`, derive `Clone`, `Copy`, `Debug`, `Deserialize`, `Serialize`,
`Eq`, `Ord`, `PartialEq`, and `PartialOrd` for `ProviderId`, `WorkflowId`, and
`ContextItemKind`. Implement `Display` and `FromStr` with the exact wire names
from this plan. Keep Clap-specific `ValueEnum` derives in `main.rs`, not the
domain module.

Expose these command shapes in `main.rs`:

```text
harp providers doctor [--provider trae|codex]
harp releases compile
harp releases list
harp releases inspect <release_id>
harp run --provider trae|codex
         [--workflow auto|ci_repair|code_review|dependency_update|general_coding]
         [--model MODEL]
         [--profile PROFILE]
         [--sandbox read-only|workspace-write|danger-full-access]
         [--approval untrusted|on-request|never]
         -- <task words...>
```

Until later sessions supply handlers, return typed errors with these codes:

```text
context_control.not_implemented
provider.not_implemented
run.not_implemented
```

Update `docs/product-contract.md` so local provider observation is inside the
product boundary while hosted model access, provider auth, and provider
security enforcement remain outside it. Register all schema IDs from the
vocabulary table. Update `README.md` with the command shapes and mark them as
the context-control surface.

- [ ] **Step 4: Run focused validation**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp context_control --lib
cargo test -p harp --test context_control_cli
```

Expected: all tests pass; running a skeleton command without `--help` returns
the declared typed error.

- [ ] **Step 5: Commit the contract**

```sh
git add README.md docs/product-contract.md \
  crates/harp/src/lib.rs crates/harp/src/main.rs \
  crates/harp/src/context_control \
  crates/harp/tests/context_control_cli.rs
git commit -m "feat: define context control contracts" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Child Session 2: Embedded Workflow Packages

**Tracker:** `HCCP-003`

**Depends on:** Child Session 1

**Files:**

- Create: `crates/harp/src/context_control/workflow.rs`
- Create: `content/context_control/workflows/ci_repair/*`
- Create: `content/context_control/workflows/code_review/*`
- Create: `content/context_control/workflows/dependency_update/*`
- Create: `content/context_control/workflows/general_coding/*`
- Modify: `crates/harp/src/context_control/mod.rs`

- [ ] **Step 1: Write strict package-loader tests**

Define `WorkflowPackage::builtin(WorkflowId)` and add tests:

```rust
#[test]
fn all_builtin_workflows_parse_and_bind_member_digests() {
    for workflow in WorkflowId::ALL {
        let package = WorkflowPackage::builtin(workflow).unwrap();
        assert_eq!(package.manifest.id, workflow);
        assert_eq!(package.manifest.schema_version, "harp-workflow/v1");
        assert!(!package.playbook.is_empty());
        assert_eq!(package.member_digests.len(), 5);
    }
}

#[test]
fn workflow_parser_rejects_unknown_fields_and_digest_mismatch() {
    let unknown = br#"{"schema_version":"harp-workflow/v1","id":"ci_repair","extra":true}"#;
    assert_eq!(
        WorkflowPackage::parse_manifest(unknown).unwrap_err().code(),
        "workflow.manifest"
    );
    assert_eq!(
        WorkflowPackage::parse_fixture_with_bad_digest().unwrap_err().code(),
        "workflow.digest"
    );
}
```

- [ ] **Step 2: Run the tests and confirm failure**

Run:

```sh
cargo test -p harp context_control::workflow --lib
```

Expected: compilation fails because `WorkflowPackage` and embedded packages do
not exist.

- [ ] **Step 3: Create strict schemas and exact baseline content**

Use `#[serde(deny_unknown_fields)]` on every package type. A manifest has:

```rust
pub struct WorkflowManifest {
    pub schema_version: String,
    pub id: WorkflowId,
    pub title: String,
    pub context_budget_tokens: u32,
    pub selector_version: String,
    pub renderer_version: String,
    pub members: BTreeMap<String, String>,
}
```

`members` binds the SHA-256 of the five non-manifest files. Parse JSONL one
non-empty line at a time and reject duplicate item IDs.

Use these package titles and budgets:

| ID | Title | Budget |
|---|---|---:|
| `ci_repair` | CI repair | 2200 |
| `code_review` | Code review | 1800 |
| `dependency_update` | Dependency update | 2000 |
| `general_coding` | General coding | 1600 |

Use this exact routing vocabulary:

| Workflow | Positive phrases | Negative phrases |
|---|---|---|
| `ci_repair` | `ci failed`, `test failure`, `tests failing`, `lint failed`, `typecheck failed`, `build failed`, `reproduce failure` | `review this diff`, `upgrade dependency` |
| `code_review` | `code review`, `review this diff`, `review this change`, `find bugs`, `audit this patch` | `fix the failing test`, `upgrade dependency` |
| `dependency_update` | `upgrade dependency`, `update dependency`, `bump version`, `refresh lockfile`, `toolchain update` | `review this diff`, `test failure` |
| `general_coding` | no positive phrase requirement | no negative phrases |

Each playbook must contain these exact semantic items:

| Workflow | Required items |
|---|---|
| `ci_repair` | reproduce the narrowest failure; classify before editing; do not weaken tests; rerun the original verification mode |
| `code_review` | inspect the diff before conclusions; report only actionable findings; verify each finding against source; avoid style-only inflation |
| `dependency_update` | identify direct and transitive change; preserve lockfile consistency; avoid unrelated upgrades; run compatibility verification |
| `general_coding` | restate the bounded requirement; inspect local contracts before editing; make the smallest causal change; run repository verification |

Each package must also include one verification recipe and one explicit weak
signal saying that an assistant claim or zero process exit alone is not
verified success.

Embed every file with `include_bytes!` from `workflow.rs`; do not read packages
from the target repository at runtime.

After writing the five non-manifest members, compute their hashes:

```sh
for workflow_dir in content/context_control/workflows/*; do
  for member in routing.json context_schema.json verification.json \
    outcome_rules.json playbook.jsonl; do
    shasum -a 256 "$workflow_dir/$member"
  done
done
```

Copy the resulting lowercase hashes into each `manifest.json` through
`apply_patch`. No manifest member may use a generated timestamp.

- [ ] **Step 4: Run package validation**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp context_control::workflow --lib
```

Expected: all four packages parse from embedded bytes, member hashes match, and
the strict-parser negative tests pass.

- [ ] **Step 5: Commit workflow packages**

```sh
git add content/context_control/workflows \
  crates/harp/src/context_control/workflow.rs \
  crates/harp/src/context_control/mod.rs
git commit -m "feat: add built-in context workflows" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Child Session 3: Secure Local State

**Tracker:** `HCCP-002`

**Depends on:** Child Session 1

**Files:**

- Create: `crates/harp/src/context_control/state.rs`
- Modify: `crates/harp/src/context_control/mod.rs`

- [ ] **Step 1: Write state-root security tests**

Add Unix tests that assert:

```rust
#[test]
fn state_root_precedence_is_explicit_then_xdg_then_home() {
    assert_eq!(
        resolve_state_root(Some(Path::new("/explicit")), Some(Path::new("/xdg")), Path::new("/home")),
        PathBuf::from("/explicit")
    );
    assert_eq!(
        resolve_state_root(None, Some(Path::new("/xdg")), Path::new("/home")),
        PathBuf::from("/xdg/harp")
    );
    assert_eq!(
        resolve_state_root(None, None, Path::new("/home")),
        PathBuf::from("/home/.local/state/harp")
    );
}

#[cfg(unix)]
#[test]
fn state_root_rejects_symlink_and_group_writable_root() {
    let fixture = StateFixture::new();
    assert_eq!(fixture.open_symlink().unwrap_err().code(), "state.symlink");
    assert_eq!(
        fixture.open_with_mode(0o770).unwrap_err().code(),
        "state.permissions"
    );
}
```

Also test that a newly created root, repository directory, release directory,
episode directory, and raw directory all have mode `0700`, while private files
have mode `0600`.

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p harp context_control::state --lib
```

Expected: compilation fails because state resolution and secure publication do
not exist.

- [ ] **Step 3: Implement held secure directories**

Implement:

```rust
pub struct StateRoot {
    root: PathBuf,
}

impl StateRoot {
    pub fn open_from_environment() -> Result<Self, AppError>;
    pub fn open_or_create(path: &Path) -> Result<Self, AppError>;
    pub fn create_private_directory(&self, relative: &Path) -> Result<PathBuf, AppError>;
    pub fn write_private_atomic(
        &self,
        relative: &Path,
        bytes: &[u8],
        replace: ReplacePolicy,
    ) -> Result<(), AppError>;
    pub fn publish_immutable_tree(
        &self,
        relative: &Path,
        members: &BTreeMap<PathBuf, Vec<u8>>,
    ) -> Result<(), AppError>;
}

pub enum ReplacePolicy {
    CreateOnly,
    CompareAndReplace(FileSnapshot),
}
```

On Unix:

- create directories with `DirBuilderExt::mode(0o700)`;
- create files with `OpenOptionsExt::mode(0o600)`;
- compare `MetadataExt::uid()` with `libc::geteuid()`;
- reject root mode bits `0o077`;
- reject symlink roots and symlink path components;
- serialize publication with create-only lock directories beneath
  `${HARP_HOME}/.locks/`; reject symlinked locks and remove a lock only when
  its recorded process identity still matches the holder;
- synchronize file bytes and the containing directory before publication; and
- re-check destination identity after acquiring the publication lock.

Do not change the semantics of the existing public generated-artifact helper in
`fs.rs`; private state gets a separate API.

- [ ] **Step 4: Run security tests**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp context_control::state --lib -- --test-threads=1
```

Expected: precedence, mode, ownership, symlink, traversal, create-only, and
concurrent-replacement tests pass.

- [ ] **Step 5: Commit secure state**

```sh
git add crates/harp/src/context_control/state.rs \
  crates/harp/src/context_control/mod.rs
git commit -m "feat: secure context control state" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Child Session 4: Immutable Baseline Releases

**Tracker:** `HCCP-002`

**Depends on:** Child Sessions 2 and 3

**Files:**

- Create: `crates/harp/src/context_control/canonical.rs`
- Create: `crates/harp/src/context_control/release.rs`
- Modify: `crates/harp/src/context_control/mod.rs`
- Modify: `crates/harp/src/main.rs`
- Modify: `crates/harp/tests/context_control_cli.rs`

- [ ] **Step 1: Write canonical identity and idempotence tests**

Add tests:

```rust
#[test]
fn baseline_release_id_is_stable_and_excludes_time_and_repository_revision() {
    let first = compile_baseline_release().unwrap();
    let second = compile_baseline_release().unwrap();
    assert_eq!(first.release_id, second.release_id);
    assert_eq!(first.identity_bytes, second.identity_bytes);
    assert!(!String::from_utf8(first.identity_bytes).unwrap().contains("created_at"));
}

#[test]
fn publishing_identical_release_is_idempotent_and_conflict_fails() {
    let fixture = ReleaseFixture::new();
    let release = compile_baseline_release().unwrap();
    fixture.publish(&release).unwrap();
    fixture.publish(&release).unwrap();
    assert_eq!(
        fixture.publish_conflicting_member(&release).unwrap_err().code(),
        "release.conflict"
    );
}
```

Add CLI assertions that `releases compile`, `list`, and `inspect` return the
same release ID under an isolated `HARP_HOME`.

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p harp context_control::release --lib
cargo test -p harp --test context_control_cli releases
```

Expected: failure because canonical serialization and release commands are not
implemented.

- [ ] **Step 3: Implement canonical release compilation**

`canonical.rs` exposes:

```rust
pub fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, AppError>;
pub fn json_bytes_with_newline<T: Serialize>(value: &T) -> Result<Vec<u8>, AppError>;
pub fn sha256_hex(bytes: &[u8]) -> String;
pub fn sha256_id(bytes: &[u8]) -> String;
```

Serialize strongly typed structs containing only ordered fields and
`BTreeMap`/`BTreeSet`; do not accept arbitrary `serde_json::Value` in release
identity.

`ReleaseIdentity` includes:

```rust
pub struct ReleaseIdentity {
    pub schema_version: String,
    pub packages: BTreeMap<WorkflowId, String>,
    pub selector_version: String,
    pub renderer_version: String,
    pub context_budgets: BTreeMap<WorkflowId, u32>,
    pub compatible_providers: BTreeSet<ProviderId>,
}
```

The release ID is `sha256-` plus the digest of compact identity bytes.
Publication writes exactly:

```text
identity.json
manifest.json
context_template.json
items.jsonl
```

Aggregate `items.jsonl` in workflow-ID order then item-ID order. `manifest.json`
contains member hashes and no mutable status. `releases inspect` validates all
hashes before returning data.

- [ ] **Step 4: Run release tests**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp context_control::release --lib -- --test-threads=1
cargo test -p harp --test context_control_cli releases
```

Expected: deterministic identity, idempotent publication, conflict rejection,
list, and inspect tests pass.

- [ ] **Step 5: Commit release compilation**

```sh
git add crates/harp/src/context_control/canonical.rs \
  crates/harp/src/context_control/release.rs \
  crates/harp/src/context_control/mod.rs \
  crates/harp/src/main.rs \
  crates/harp/tests/context_control_cli.rs
git commit -m "feat: compile immutable context releases" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Child Session 5: Deterministic Workflow Routing

**Tracker:** `HCCP-003`

**Depends on:** Child Session 2

**Files:**

- Create: `crates/harp/src/context_control/routing.rs`
- Modify: `crates/harp/src/context_control/mod.rs`

- [ ] **Step 1: Write the routing matrix**

Add tests for these exact expectations:

```rust
let cases = [
    ("fix the tests failing in CI", WorkflowId::CiRepair),
    ("review this diff and find bugs", WorkflowId::CodeReview),
    ("upgrade dependency serde and refresh lockfile", WorkflowId::DependencyUpdate),
    ("add a bounded parser for this config", WorkflowId::GeneralCoding),
];
for (task, expected) in cases {
    assert_eq!(route(task, WorkflowChoice::Auto).unwrap().selected, expected);
}
```

Also test:

- explicit `code_review` wins even when the task contains `tests failing`;
- a tie between non-fallback workflows returns `general_coding`;
- the trace contains every considered workflow, score, matched rule IDs, and
  rejection reason; and
- routing performs no subprocess, network, filesystem write, or model call.

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p harp context_control::routing --lib
```

Expected: compilation fails because routing types and logic do not exist.

- [ ] **Step 3: Implement a small rule engine**

Implement:

```rust
pub struct RouteDecision {
    pub selected: WorkflowId,
    pub explicit: bool,
    pub considered: Vec<RouteConsideration>,
}

pub struct RouteConsideration {
    pub workflow: WorkflowId,
    pub score: i32,
    pub matched_rule_ids: Vec<String>,
    pub rejection_reason: Option<String>,
}

pub fn route(task: &str, choice: WorkflowChoice) -> Result<RouteDecision, AppError>;
```

Normalize with Unicode lowercase and collapsed ASCII whitespace. A routing rule
matches when all `all_phrases` occur, at least one `any_phrases` occurs when the
list is non-empty, and no `none_phrases` occur. Add rule scores. Select the
single highest non-fallback score. A zero score, negative score, or top-score
tie selects `general_coding`. Sort trace entries by `WorkflowId`.

- [ ] **Step 4: Run routing tests**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp context_control::routing --lib
```

Expected: the matrix, explicit override, ambiguity fallback, stable trace, and
no-side-effect tests pass.

- [ ] **Step 5: Commit routing**

```sh
git add crates/harp/src/context_control/routing.rs \
  crates/harp/src/context_control/mod.rs
git commit -m "feat: route context workflows" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Child Session 6: Bounded Context Resolution

**Tracker:** `HCCP-003`

**Depends on:** Child Sessions 4 and 5

**Files:**

- Create: `crates/harp/src/context_control/context.rs`
- Modify: `crates/harp/src/context_control/mod.rs`

- [ ] **Step 1: Write deterministic bundle tests**

Add tests:

```rust
#[test]
fn identical_inputs_produce_identical_context_bytes() {
    let input = ContextRequest::fixture(WorkflowId::CiRepair, "tests failing in CI");
    let first = resolve(&input).unwrap();
    let second = resolve(&input).unwrap();
    assert_eq!(first.rendered_markdown, second.rendered_markdown);
    assert_eq!(first.rendered_context_sha256, second.rendered_context_sha256);
    assert_eq!(first.selected_item_ids, second.selected_item_ids);
}

#[test]
fn resolver_obeys_a_lower_budget_and_records_rejections() {
    let mut input = ContextRequest::fixture(WorkflowId::CiRepair, "tests failing in CI");
    input.token_budget = 120;
    let bundle = resolve(&input).unwrap();
    assert!(bundle.estimated_tokens <= 120);
    assert!(!bundle.rejected_items.is_empty());
    assert!(bundle
        .rejected_items
        .iter()
        .all(|item| item.reason == "context_budget"));
}
```

Assert the rendered section order:

```text
Harp context release
Repository invariants
Workflow
Relevant patterns
Anti-patterns
Required verification
Context manifest
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p harp context_control::context --lib
```

Expected: compilation fails because `ContextRequest`, `ContextBundle`, and
`resolve` do not exist.

- [ ] **Step 3: Implement stable selection and rendering**

Implement `ContextBundle` with the fields approved in the design:

```rust
pub struct ContextBundle {
    pub schema_version: String,
    pub release_id: String,
    pub workflow: WorkflowId,
    pub repository_invariants: Vec<ContextItem>,
    pub workflow_steps: Vec<ContextItem>,
    pub relevant_patterns: Vec<ContextItem>,
    pub anti_patterns: Vec<ContextItem>,
    pub verification_expectations: Vec<ContextItem>,
    pub selected_item_ids: Vec<String>,
    pub rejected_items: Vec<RejectedContextItem>,
    pub routing_trace: RouteDecision,
    pub estimated_tokens: u32,
    pub rendered_context_sha256: String,
    pub rendered_markdown: String,
}
```

Use `(rendered_markdown.chars().count() + 3) / 4` as the documented V1 token
estimate. Sort eligible items by descending priority then ascending ID. Add
items only when the complete rendered document remains within budget. Never
truncate an item. Required verification items outrank optional examples.

The final manifest section lists release ID, workflow, selected item IDs,
estimated tokens, and the statement:

```text
This context is advisory and cannot override provider policy or AGENTS.md.
```

- [ ] **Step 4: Run context tests**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp context_control::context --lib
```

Expected: deterministic bytes, budget rejection, section ordering, priority,
and no-truncation tests pass.

- [ ] **Step 5: Commit context resolution**

```sh
git add crates/harp/src/context_control/context.rs \
  crates/harp/src/context_control/mod.rs
git commit -m "feat: resolve bounded workflow context" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Child Session 7: Repository Policy And Snapshot

**Tracker:** `HCCP-001`, `HCCP-005`

**Depends on:** Child Sessions 1 and 3

**Files:**

- Create: `crates/harp/src/context_control/policy.rs`
- Create: `crates/harp/src/context_control/repository_state.rs`
- Modify: `crates/harp/src/context_control/mod.rs`

- [ ] **Step 1: Write policy and Git-state tests**

Policy tests must prove:

```rust
#[test]
fn repository_policy_can_only_restrict_runtime_behavior() {
    let policy = RepositoryPolicy::parse(br#"{
      "schema_version":"harp-repository-policy/v1",
      "enabled":true,
      "allowed_workflows":["ci_repair"],
      "pinned_release":"sha256-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "max_context_tokens":900,
      "required_verification_labels":["ci"]
    }"#).unwrap();
    assert_eq!(policy.max_context_tokens, Some(900));
}
```

Reject unknown fields, free-form prompt fields, executable selector fields,
budgets above the workflow budget, and an allowed-workflow set containing an
unknown ID.

Git tests create a repository and linked worktree, then assert both produce the
same `repository_id`, while dirty content changes `status_sha256` and
`tracked_changes_sha256`. Confirm that serialized snapshots contain no
repository path, remote URL, status path, patch bytes, or untracked file
contents.

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p harp context_control::policy --lib
cargo test -p harp context_control::repository_state --lib -- --test-threads=1
```

Expected: compilation fails because strict policy and Git snapshot modules do
not exist.

- [ ] **Step 3: Implement strict policy and bounded Git commands**

`RepositoryPolicy` is `#[serde(deny_unknown_fields)]` and supports only:

```rust
pub struct RepositoryPolicy {
    pub schema_version: String,
    pub enabled: bool,
    pub allowed_workflows: Option<BTreeSet<WorkflowId>>,
    pub pinned_release: Option<String>,
    pub max_context_tokens: Option<u32>,
    pub required_verification_labels: Vec<String>,
}
```

Read only `.harp/context-control.json`; reject a symlink or file larger than
64 KiB. Missing policy means enabled with no additional restriction.

Run Git with `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`, and
`GIT_NO_REPLACE_OBJECTS=1`. Compute:

- common-dir digest for `repository_id`;
- verified `HEAD`;
- SHA-256 of `git status --porcelain=v2 -z --untracked-files=normal`;
- SHA-256 of `git diff --no-ext-diff --no-textconv --binary HEAD --`;
- tracked and untracked counts without retaining names; and
- a `dirty` boolean.

Cap each captured Git stream at 64 MiB and return `repository.output_limit`
before provider launch when exceeded.

- [ ] **Step 4: Run policy and repository tests**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp context_control::policy --lib
cargo test -p harp context_control::repository_state --lib -- --test-threads=1
```

Expected: strict policy, worktree identity, dirty-state, path non-disclosure,
and output-limit tests pass.

- [ ] **Step 5: Commit repository preflight**

```sh
git add crates/harp/src/context_control/policy.rs \
  crates/harp/src/context_control/repository_state.rs \
  crates/harp/src/context_control/mod.rs
git commit -m "feat: capture repository control state" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Child Session 8: Provider Doctor And Capability Snapshots

**Tracker:** `HCCP-004`

**Depends on:** Child Sessions 1 and 3

**Files:**

- Modify: `Cargo.toml`
- Modify: `crates/harp/Cargo.toml`
- Modify: `Cargo.lock`
- Create: `crates/harp/src/context_control/provider/mod.rs`
- Create: `crates/harp/src/context_control/provider/probe.rs`
- Modify: `crates/harp/src/context_control/mod.rs`
- Modify: `crates/harp/src/main.rs`
- Create: `crates/harp/tests/support/context_control.rs`
- Modify: `crates/harp/tests/context_control_cli.rs`

- [ ] **Step 1: Write fake-provider probe tests**

Create test helpers that write executable `traecli` and `codex` shell fixtures
into a temporary `bin` directory. The supported fixture returns:

```text
<binary> --version
  traecli 0.200.19
  codex-cli 0.144.5

<binary> exec --help
  --json
  --output-last-message
  --cd
  --model
  --profile
  --sandbox
  --config

<binary> exec resume --help
  --json

<binary> app-server generate-json-schema --help
  --out
```

Tests cover supported, missing, malformed, nonzero, output-over-limit, and
timeout probes. CLI tests set `PATH` to the fixture directory and assert
`harp --format json providers doctor` reports both providers and stable
capability digests.

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p harp context_control::provider::probe --lib
cargo test -p harp --test context_control_cli providers
```

Expected: failure because provider probing is not implemented.

- [ ] **Step 3: Add bounded probes**

Add `wait-timeout = "0.2"` to workspace dependencies. Probe commands write
stdout and stderr to private temporary files rather than pipes, wait at most
five seconds, kill on timeout, then read at most 1 MiB per stream.

Implement:

```rust
pub struct ProviderCapabilities {
    pub schema_version: String,
    pub provider: ProviderId,
    pub executable: PathBuf,
    pub version: String,
    pub exec_json: bool,
    pub output_last_message: bool,
    pub working_directory: bool,
    pub model: bool,
    pub profile: bool,
    pub sandbox: bool,
    pub approval_config: bool,
    pub resume_json: bool,
    pub app_server_schema: bool,
    pub capability_sha256: String,
}

pub fn probe(provider: ProviderId, path: &Path) -> Result<ProviderCapabilities, AppError>;
pub fn locate(provider: ProviderId, path_env: &OsStr) -> Result<PathBuf, AppError>;
```

Do not invoke login, auth, models, network, or App Server itself. Doctor fails
when `exec --json`, `--output-last-message`, working-directory selection, or
configuration override is missing. Optional features remain false rather than
failing.

- [ ] **Step 4: Run probe tests**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp context_control::provider::probe --lib
cargo test -p harp --test context_control_cli providers
```

Expected: all fake-provider capability and failure tests pass without a real
provider binary.

- [ ] **Step 5: Commit provider probing**

```sh
git add Cargo.toml Cargo.lock crates/harp/Cargo.toml \
  crates/harp/src/context_control/provider \
  crates/harp/src/context_control/mod.rs \
  crates/harp/src/main.rs \
  crates/harp/tests/context_control_cli.rs \
  crates/harp/tests/support/context_control.rs
git commit -m "feat: probe agent CLI providers" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Child Session 9: Episode Preflight And Manifests

**Tracker:** `HCCP-005`

**Depends on:** Child Sessions 3, 4, 6, 7, and 8

**Files:**

- Create: `crates/harp/src/context_control/episode.rs`
- Modify: `crates/harp/src/context_control/mod.rs`

- [ ] **Step 1: Write manifest-order and partial-run tests**

Add tests that:

- inject fixed nanoseconds, process ID, and counter to obtain a stable episode
  ID;
- create `manifest.json`, `context.json`, and `raw/<provider>/` before a fake
  launch callback is invoked;
- leave the manifest immutable after completion;
- write `completion.json` separately;
- preserve an incomplete episode when no completion is published; and
- write failed preflight records beneath
  `repositories/<repository_id>/preflight_failures/`, not `episodes/`.

Use this assertion:

```rust
let manifest_before = fs::read(episode.path().join("manifest.json")).unwrap();
episode.complete(&completion).unwrap();
let manifest_after = fs::read(episode.path().join("manifest.json")).unwrap();
assert_eq!(manifest_after, manifest_before);
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p harp context_control::episode --lib -- --test-threads=1
```

Expected: compilation fails because episode publication does not exist.

- [ ] **Step 3: Implement immutable identity and separate completion**

Create:

```rust
pub struct EpisodeManifest {
    pub schema_version: String,
    pub episode_id: String,
    pub repository: RepositorySnapshot,
    pub provider: ProviderId,
    pub provider_version: String,
    pub provider_capabilities_sha256: String,
    pub workflow: WorkflowId,
    pub release_id: String,
    pub context_bundle_sha256: String,
    pub context_manifest_completeness: ContextManifestCompleteness,
    pub configuration_mode: ConfigurationMode,
    pub task_sha256: String,
    pub policy_sha256: Option<String>,
    pub invocation: ProviderInvocationManifest,
}

pub struct CompletionReceipt {
    pub schema_version: String,
    pub episode_id: String,
    pub provider_exit_code: Option<i32>,
    pub terminated_by_signal: Option<i32>,
    pub capture_complete: bool,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub final_message_sha256: Option<String>,
    pub raw_archive_sha256: String,
    pub post_repository: RepositorySnapshot,
}
```

Generate episode IDs as:

```text
ep-<16 lowercase hex nanoseconds>-<8 lowercase hex pid>-<8 lowercase hex counter>
```

`EpisodeWriter::begin` publishes canonical manifest and context bytes before it
returns raw file paths. `complete` is create-only and cannot replace an
existing receipt.

- [ ] **Step 4: Run episode tests**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp context_control::episode --lib -- --test-threads=1
```

Expected: stable ID, pre-launch publication, immutable manifest, partial
episode, and preflight separation tests pass.

- [ ] **Step 5: Commit episode publication**

```sh
git add crates/harp/src/context_control/episode.rs \
  crates/harp/src/context_control/mod.rs
git commit -m "feat: publish context episodes" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Child Session 10: Provider-Safe Invocation Builders

**Tracker:** `HCCP-005`

**Depends on:** Child Sessions 6, 8, and 9

**Files:**

- Create: `crates/harp/src/context_control/provider/invocation.rs`
- Modify: `crates/harp/src/context_control/provider/mod.rs`

- [ ] **Step 1: Write exact argument and prompt tests**

For both providers assert the argument vector begins with:

```text
exec
--cd
<repository root>
--json
--output-last-message
<private final-message path>
```

Assert optional fields translate exactly:

```text
--model <model>
--profile <profile>
--sandbox <sandbox>
--config approval_policy="<approval>"
-
```

Assert the vector never contains:

```text
--ephemeral
--ignore-user-config
--ignore-rules
--dangerously-bypass-approvals-and-sandbox
--dangerously-bypass-hook-trust
--search
```

Prompt tests verify stable bytes and separate:

```text
<harp-context release="..." sha256="...">
<rendered context>
</harp-context>
<user-task sha256="...">
<original task>
</user-task>
```

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p harp context_control::provider::invocation --lib
```

Expected: compilation fails because invocation construction does not exist.

- [ ] **Step 3: Implement typed invocation construction**

Implement:

```rust
pub struct ProviderRunOptions {
    pub model: Option<String>,
    pub profile: Option<String>,
    pub sandbox: Option<SandboxMode>,
    pub approval: Option<ApprovalPolicy>,
}

pub struct ProviderInvocation {
    pub schema_version: String,
    pub provider: ProviderId,
    pub executable: PathBuf,
    pub arguments: Vec<OsString>,
    pub working_directory: PathBuf,
    pub stdin_bytes: Vec<u8>,
    pub final_message_path: PathBuf,
}

pub fn build_invocation(
    capabilities: &ProviderCapabilities,
    repository_root: &Path,
    context: &ContextBundle,
    task: &str,
    options: &ProviderRunOptions,
    final_message_path: &Path,
) -> Result<ProviderInvocation, AppError>;
```

Reject NUL bytes, empty tasks, model/profile values over 256 bytes, options not
supported by the capability snapshot, and any final-message path outside the
episode's provider raw directory. Do not accept arbitrary provider arguments.

- [ ] **Step 4: Run invocation tests**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp context_control::provider::invocation --lib
```

Expected: exact argument, parity, prompt digest, unsupported-capability, and
forbidden-flag tests pass.

- [ ] **Step 5: Commit invocation builders**

```sh
git add crates/harp/src/context_control/provider/invocation.rs \
  crates/harp/src/context_control/provider/mod.rs
git commit -m "feat: build provider-safe invocations" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Child Session 11: Streaming Provider Execution

**Tracker:** `HCCP-005`

**Depends on:** Child Sessions 8 through 10

**Files:**

- Modify: `Cargo.toml`
- Modify: `crates/harp/Cargo.toml`
- Modify: `Cargo.lock`
- Create: `crates/harp/src/context_control/provider/process.rs`
- Modify: `crates/harp/src/context_control/provider/mod.rs`
- Modify: `crates/harp/tests/support/context_control.rs`
- Create: `crates/harp/tests/context_control_run.rs`

- [ ] **Step 1: Write byte-exact, nonzero, limit, and signal tests**

The fake provider must:

- validate that stdin contains both Harp context and the original task;
- write two JSONL records with non-UTF-8-safe byte handling to stdout;
- write a diagnostic line to stderr;
- write the requested final-message path;
- exit with a requested code; and
- trap `TERM`, write a marker, and exit 143 in the signal test.

Assert:

```rust
assert_eq!(fs::read(raw_stdout).unwrap(), expected_stdout_bytes);
assert_eq!(fs::read(raw_stderr).unwrap(), expected_stderr_bytes);
assert_eq!(result.exit_code, Some(7));
assert!(result.capture_complete);
```

The output-limit test emits more than the configured 64 MiB cap and expects
`provider.output_limit`. The signal test sends `SIGTERM` through an injected
control channel and expects the child process group to receive it.

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p harp --test context_control_run provider_process
```

Expected: compilation fails because streaming execution does not exist.

- [ ] **Step 3: Implement process-group execution and deterministic archive**

Add `signal-hook = "0.3"` to workspace dependencies.

`execute` must:

1. create raw stdout and stderr as mode `0600` files;
2. spawn the provider in a new Unix process group;
3. write all prompt bytes then close stdin;
4. tee stdout to terminal stdout and raw stdout;
5. tee stderr to terminal stderr and raw stderr;
6. enforce 64 MiB per stream and terminate the process group on overflow;
7. forward `SIGINT`, `SIGTERM`, and `SIGHUP` to the process group;
8. wait for both tee threads and the provider;
9. validate the final-message path as a regular non-symlink file;
10. hash every raw member; and
11. create a deterministic gzip-compressed tar plus `raw_members.tsv`.

Use existing `flate2` and `tar` dependencies. Set archive member mode `0644`,
uid/gid `0`, empty user/group names, empty PAX headers, and mtime `0`, matching
the bounded Meta-Harness archive convention. Archive only provider-produced
members: `stdout.jsonl`, `stderr.bin`, and `final_message.bin` when present.
Keep `raw_traecli_run.tar.gz` and `raw_members.tsv` outside the archive so
publication is non-recursive.

Expose:

```rust
pub struct ProviderProcessResult {
    pub exit_code: Option<i32>,
    pub terminated_by_signal: Option<i32>,
    pub capture_complete: bool,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub final_message_sha256: Option<String>,
    pub raw_archive_sha256: String,
}
```

- [ ] **Step 4: Run process tests**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp --test context_control_run provider_process -- --test-threads=1
```

Expected: byte-exact tee, nonzero exit, final message, deterministic archive,
output cap, and signal-forwarding tests pass.

- [ ] **Step 5: Commit process capture**

```sh
git add Cargo.toml Cargo.lock crates/harp/Cargo.toml \
  crates/harp/src/context_control/provider \
  crates/harp/tests/context_control_run.rs \
  crates/harp/tests/support/context_control.rs
git commit -m "feat: capture provider execution" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Child Session 12: End-To-End `harp run`

**Tracker:** `HCCP-005`

**Depends on:** Child Sessions 1 through 11

**Files:**

- Create: `crates/harp/src/context_control/run.rs`
- Modify: `crates/harp/src/context_control/mod.rs`
- Modify: `crates/harp/src/main.rs`
- Modify: `crates/harp/tests/context_control_run.rs`

- [ ] **Step 1: Write two-provider parity and failure-path tests**

For the same temporary Git repository, task, workflow, baseline release, and
typed run options:

- run once through fake Trae CLI and once through fake Codex CLI;
- assert both manifests pin the same release and context bundle digest;
- assert both provider prompts contain byte-identical rendered context;
- assert provider identity and raw evidence remain distinct;
- assert the manifest exists before the fake provider records launch;
- assert nonzero provider exit returns the provider exit code from `harp run`;
- assert missing capability produces a preflight-failure record and no episode;
- assert a disabled `.harp/context-control.json` fails before launch; and
- assert a policy budget can lower but not raise the workflow budget.

Add a stdout contract assertion: provider JSONL is the only stdout. Harp
lifecycle messages and the final episode ID are on stderr.

- [ ] **Step 2: Run tests and confirm failure**

Run:

```sh
cargo test -p harp --test context_control_run harp_run -- --test-threads=1
```

Expected: failure because orchestration and streaming CLI exit behavior do not
exist.

- [ ] **Step 3: Implement the approved seven-phase run**

Implement:

```rust
pub struct RunRequest {
    pub provider: ProviderId,
    pub workflow: WorkflowChoice,
    pub task: String,
    pub options: ProviderRunOptions,
}

pub struct RunResult {
    pub episode_id: String,
    pub release_id: String,
    pub context_bundle_sha256: String,
    pub provider_exit_code: Option<i32>,
    pub terminated_by_signal: Option<i32>,
}

pub fn run(request: RunRequest, reporter: &mut dyn RunReporter) -> Result<RunResult, AppError>;
```

Execute phases in this exact order:

1. resolve repository root, secure state root, strict repository policy, and
   pre-run Git snapshot;
2. locate and probe the provider;
3. route one workflow and apply policy restrictions;
4. compile or resolve the compatible baseline release;
5. resolve the bounded context and build the invocation;
6. publish manifest and context, then execute and capture the provider; and
7. capture post-run Git state, publish completion, and report episode identity.

`RunReporter` writes text or JSON lifecycle records to stderr. `main.rs` must
special-case the streaming run command so the generic success envelope never
contaminates provider stdout. Return `ExitCode::from(provider_code as u8)` for
codes `1..=255`; signals return `128 + signal` when representable; Harp
preflight errors return `1`.

- [ ] **Step 4: Run all context-control tests**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp context_control --lib -- --test-threads=1
cargo test -p harp --test context_control_cli -- --test-threads=1
cargo test -p harp --test context_control_run -- --test-threads=1
```

Expected: all domain, CLI, provider, parity, policy, raw-capture, and failure
tests pass with fake binaries only.

- [ ] **Step 5: Commit end-to-end observation**

```sh
git add crates/harp/src/context_control/run.rs \
  crates/harp/src/context_control/mod.rs \
  crates/harp/src/main.rs \
  crates/harp/tests/context_control_run.rs
git commit -m "feat: run agent CLIs through Harp" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Child Session 13: Milestone Documentation And Release Gate

**Tracker:** `HCCP-001` through `HCCP-005`

**Depends on:** Child Sessions 1 through 12

**Files:**

- Modify: `README.md`
- Modify: `docs/product-contract.md`
- Modify: `docs/contributing.md`
- Modify: `docs/workstream/harp-context-control-plane/tracker.org`
- Modify: `docs/import-receipt.md`

- [ ] **Step 1: Document only implemented behavior**

Update the CLI section with:

```sh
harp providers doctor
harp releases compile
harp releases list
harp releases inspect <release-id>
harp run --provider trae --workflow auto -- fix the failing CI test
harp run --provider codex --workflow code_review -- review this change
```

Document:

- `HARP_HOME`, `XDG_STATE_HOME`, and fallback precedence;
- `.harp/context-control.json` as a restrictions-only repository policy;
- raw provider JSONL on stdout and Harp lifecycle records on stderr;
- native provider config and rules remain enabled;
- no live provider call occurs in `mise run verify`;
- V1 context-manifest completeness is partial; and
- ACE/MCE suggestions and promotion are not implemented in this milestone.

Do not claim that Harp sees the complete effective prompt or provider skill
selection.

- [ ] **Step 2: Update tracker states and evidence**

Move `HCCP-001` through `HCCP-005` to `REVIEW`. Record every focused test
command and commit hash under `Verification Evidence`. Leave `HCCP-006` through
`HCCP-011` as `BACKLOG`.

- [ ] **Step 3: Refresh the standalone payload digest**

Stage the complete milestone except the receipt:

```sh
git add README.md docs/product-contract.md docs/contributing.md \
  docs/workstream/harp-context-control-plane/tracker.org \
  content/context_control crates/harp Cargo.toml Cargo.lock
```

Run:

```sh
cargo run -p harp -- repository verify
```

Expected: failure naming exactly one new expected SHA-256. Replace only the
digest in `docs/import-receipt.md` with `apply_patch`, stage the receipt, and
rerun repository verification.

- [ ] **Step 4: Run the complete offline release gate**

Run:

```sh
mise run verify
```

Expected:

- Rust formatting, lint, workspace tests, context-control tests, corpus check,
  source verification, search, and Meta-Harness lab tests pass;
- Atlas lint, typecheck, 45-or-more tests, and decoded static export pass;
- Git LFS and repository payload verification pass; and
- neither `traecli` nor `codex` is invoked.

After the gate passes, move `HCCP-001` through `HCCP-005` from `REVIEW` to
`DONE` and add the final `mise run verify` evidence. If that tracker edit
changes the payload digest, refresh the receipt once more and rerun
`cargo run -p harp -- repository verify`.

- [ ] **Step 5: Commit the verified milestone record**

```sh
git add README.md docs/product-contract.md docs/contributing.md \
  docs/workstream/harp-context-control-plane/tracker.org \
  docs/import-receipt.md
git commit -m "docs: complete context control observation milestone" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

Then run:

```sh
git status --short
git log --oneline --decorate -14
```

Expected: only the pre-existing unrelated untracked Obsidian files, images,
`CONTEXT.md`, and Agentic eval/apply drafts remain; the feature branch contains
the 13 focused milestone commits.

---

## Plan Self-Review

### Spec Coverage

| Approved requirement | Implemented by |
|---|---|
| Local-first secure state | Child Session 3 |
| All workflow families | Child Sessions 2 and 5 |
| Immutable pinned release | Child Session 4 |
| Deterministic bounded context | Child Session 6 |
| Restrictions-only repository policy | Child Session 7 |
| Trae CLI and Codex CLI capability probes | Child Session 8 |
| Manifest before inference | Child Session 9 |
| No provider security weakening | Child Session 10 |
| Byte-exact raw observation | Child Session 11 |
| Shared `harp run` interface | Child Session 12 |
| Provider-neutral offline verification | Child Session 13 |
| Observation only; no promotion | Scope and Child Session 13 |

### Deferred Requirements

The following approved design sections intentionally map to later tracker
items and are not silently omitted:

| Deferred capability | Tracker |
|---|---|
| Event normalization and outcome labels | `HCCP-006` |
| Sanitized learning export | `HCCP-007` |
| ACE candidate workshop | `HCCP-008` |
| MCE package and skill candidates | `HCCP-009` |
| App Server, compaction, subagents, resume | `HCCP-010` |
| Shadow, canary, promotion, rollback | `HCCP-011` |

### Type Consistency

- `ProviderId`, `WorkflowId`, and schema IDs are defined once in Child Session
  1 and reused unchanged.
- `WorkflowPackage` owns embedded package bytes; target repositories never
  supply workflow code.
- `ReleaseIdentity` excludes target repository revision and creation time.
- `ContextBundle` contains the release and route but not provider-specific
  arguments.
- `ProviderInvocation` owns native arguments and prompt bytes but not runtime
  results.
- `EpisodeManifest` is immutable; `CompletionReceipt` owns exit and capture
  facts.
- `RunResult` references one episode, release, and context bundle.

### Placeholder Check

The plan contains no unresolved implementation choices, unspecified file
paths, generic “handle errors” steps, or generated code left for the
implementer to invent. Runtime values such as release IDs, episode IDs, package
member digests, commit hashes, and payload digests are deliberately computed
from the exact bytes created during execution.

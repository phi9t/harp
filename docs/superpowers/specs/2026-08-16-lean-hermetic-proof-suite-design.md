# Local Hermetic Lean Proof-Suite Gate

## Objective

Create a local, reproducible, hermetic Lean proof-suite runner that can
load, compile, and run a standard set of Lean proof modules before
Crouzeix/Jin validation or training-data generation depends on those modules.

The gate is a trust boundary for formal artifacts. It answers one question:
given a pinned Lean runtime, a declared proof-suite manifest, and a closed
source/artifact inventory, can this repository reproduce the exact Lean
compile outcome from local bytes?

The first successful gate result does not prove Crouzeix, does not advance a
Jin route, and does not authorize provider execution. It only establishes that
Harp can run a known Lean proof suite under a controlled local boundary and
record the outcome with typed receipts.

## Scope Boundary

This gate is separate from CPFR-R018, CPFR-032, and CPFR-033.

- It does not implement CPFR-R018 context-parity repair.
- It does not prepare, recover, edit, or validate ignored `.runs` trees.
- It does not run live providers or provider wrappers.
- It does not consume root tickets or create proof-agent attempts.
- It does not unblock CPFR-033.
- It does not claim proof progress, proof correctness, Crouzeix progress, Jin
  progress, admissions, mathematical nodes, or independent evaluations.

CPFR-R018 remains the provider-free context digest repair. CPFR-032 remains
blocked until its own separately authorized continuation can prepare and check
a fresh run after the context boundary is repaired. CPFR-033 remains blocked
on CPFR-032.

The sibling `crouzeix-formal-validation` worktree and any Lean reference
material in it are evidence and reference material only. They may help identify
candidate modules, expected command shapes, or known blockers, but they do not
grant landing authority. A future Harp implementation must import only
declared bytes through a manifest, digest them, license-record them when
needed, and make Harp-owned receipts the authority for local status.

## Architecture

### Lean Runtime Lock

The runner consumes a committed Lean runtime lock. The lock records:

- Lean executable identity, version string, platform, byte length, and
  SHA-256 digest;
- Lake identity and invocation mode;
- Mathlib or package revision identities;
- allowed package roots and source roots;
- fixed command templates for build, module check, and optional axiom audit;
- environment allowlist;
- timeout/resource limits; and
- expected runtime inventory digest.

The lock is not an installer recipe. Missing local toolchain bytes are a
`blocked` result, not a prompt to fetch from the network. The runner must never
consult mutable PATH entries, shell startup files, global Lake caches, or
ambient environment variables except for the explicitly allowed variables in
the lock.

### Proof-Suite Manifest

The proof-suite manifest declares the suite as data. Each row identifies:

- stable module ID;
- Lean file path relative to the suite root;
- module name;
- tier;
- expected imports;
- expected declarations when the tier needs declaration-level checks;
- allowed trusted declarations policy;
- input source digests;
- required artifacts;
- command profile; and
- whether the module is smoke-only, local-project, reference-suite, or
  target-route material.

The manifest must be closed. Unknown source files, undeclared generated files,
undeclared package roots, undeclared imports, and digest drift stop the run
before Lean execution with `blocked` when an artifact is unavailable, or
`failed` when local bytes exist but violate the manifest.

### Declared-Write-Surface Runner

The runner creates a fresh temporary execution root for each suite run. It
copies or materializes only manifest-declared regular files, verifies every
copied byte, and invokes Lean/Lake with:

- explicit `argv`;
- explicit current working directory;
- explicit environment allowlist;
- no shell interpolation;
- no network provider calls;
- declared writable roots limited to the run root and receipt root;
- no symlinked source, artifact, generated, or output targets;
- deterministic stdout/stderr capture; and
- deterministic timeout/resource classification.

The runner must reject absolute paths, `..` traversal, symlinked roots,
symlinked files, hardlink surprises where detectable, non-regular inputs,
case-colliding manifest paths, and any output path that escapes its declared
root. It must compute source and artifact inventories before and after Lean
execution; a changed input inventory fails the run.

CPFR-L003 is a local declared-write-surface runner, not an operating-system
filesystem sandbox. Before executing a command, the runner must expand and
inspect declared `argv` and allowed environment values, reject declared absolute
write channels that target paths outside the run root or receipt root, and
record the limitation marker `local_process_no_os_sandbox` when that boundary
is relevant. Its pre/post inventories prove that manifest-declared inputs and
outputs stayed stable, but they do not claim containment against a malicious or
buggy executable that performs hardcoded writes to ambient filesystem paths.

CPFR-L007 is therefore a later required gate for sandboxed real Lean execution.
Before a real Lean run, target-route run, or `mathematical_foundations`
validation is described as hermetic, Harp must add an OS-level sandboxed
execution boundary with explicit write mounts, network denial, and receipts
that distinguish sandbox guarantees from CPFR-L003's local declared-surface
checks.

### Typed Receipts

Every run emits one typed receipt under a declared local receipt root. Receipts
are create-only and contain:

- schema ID and schema version;
- suite manifest digest;
- runtime lock digest;
- runtime inventory digest;
- source inventory digest;
- artifact inventory digest;
- execution root digest;
- command receipt with `argv`, `cwd`, environment, timeout, and exit status;
- stdout/stderr digests and local paths;
- module outcomes;
- trusted-declaration or axiom-audit summary when configured;
- final outcome: `passed`, `failed`, or `blocked`; and
- first blocker or first failure reason.

Typed outcome rules:

- `passed`: every selected module compiled under the locked command, every
  required declaration check and inventory check succeeded, no undeclared
  input/output appeared, and receipt digests recompute.
- `failed`: Lean executed and found an ill-typed proof, forbidden trusted
  declaration, import-policy violation, digest drift after local bytes existed,
  or other local policy violation.
- `blocked`: the runner could not start or complete the closed local check
  because a required toolchain, package artifact, source artifact, permission,
  or resource was missing or unavailable before judging proof correctness.

Missing toolchain or missing artifact is `blocked`. Ill-typed Lean proof code
is `failed`. Exact successful compile plus exact inventory validation is
`passed`.

### Artifact And Source Inventory

The runner records two inventories.

The source inventory covers Lean files, Lake files, lock files, manifest files,
local proof-suite inputs, and any Harp-owned adapters. It uses repository-
relative paths where possible and canonical run-root-relative paths for
materialized copies.

The artifact inventory covers toolchain executables, package caches,
precompiled artifacts when allowed, public-source snapshots, source licenses,
and any reference proof-suite byte set. Every artifact entry records path,
byte length, SHA-256 digest, role, provenance note, and license-status pointer
when the artifact came from upstream.

The inventory is a proof-suite dependency record, not a Crouzeix evidence
publication. Later publication into `evidence/` or `knowledge/` requires the
normal Harp source and license workflow.

## Initial Suite Tiers

The gate starts with small tiers so the hermetic boundary is proven before any
Crouzeix/Jin dependency is added.

| Tier | Purpose | Passing Meaning |
| --- | --- | --- |
| Tiny smoke module | Compile one Harp-owned Lean file with a trivial theorem and no route dependency. | The locked local Lean process can run from the controlled root. |
| Local Lake project module | Compile a minimal local Lake project with declared imports and package metadata. | Lake/project wiring, cwd, package roots, and inventory checks are reproducible. |
| Reference proof-suite modules | Compile selected modules imported from pinned reference bytes. | Harp can reproduce the selected reference suite from declared local artifacts. |
| Later Jin/Crouzeix target | Compile target-route modules only after the runner is stable. | A route-specific formal result can depend on the gate receipt, but the receipt still proves only the declared module set. |

The later Jin/Crouzeix target tier is intentionally last. It must not be added
until smoke, local project, and reference-suite tiers have stable receipts and
clear failure behavior.

## Training Boundary

This design creates no training examples.

The gate may later define a schema for training-data provenance after the
runner is stable. That future schema should reference proof-suite receipts,
module IDs, source digests, command receipts, outcome labels, and normalized
Lean diagnostics. It must not treat failed or blocked runs as positive proof
examples, and it must not include generated candidate text, provider prompts,
or proof-agent transcripts unless a separate data-governance decision allows
that corpus.

Until the runner is stable, the only permitted training-related output is the
receipt shape needed to decide what future examples could safely cite. No
dataset rows, examples, prompt corpora, or model-evaluation inputs are created
by this gate.

## Verification Strategy

Future implementation should be subagent-executed and test-first. This spec is
not an implementation plan; it only defines the contract that such an
implementation must satisfy.

Verification should cover:

- lock parsing rejects unknown fields, duplicate keys, unsupported versions,
  ambient PATH use, and missing digests;
- manifest parsing rejects unknown modules, duplicate IDs, undeclared imports,
  undeclared files, path traversal, absolute paths, symlinks, and digest drift;
- runner tests prove controlled `cwd`, controlled environment, no shell use,
  no network/provider invocation, fresh execution-root materialization,
  declared absolute write-channel rejection, pre/post inventory validation, and
  the `local_process_no_os_sandbox` limitation marker;
- a later CPFR-L007 gate proves OS-level filesystem containment before real
  Lean or `mathematical_foundations` validation can be called hermetic;
- outcome tests distinguish `blocked`, `failed`, and `passed`;
- receipt tests reject missing inventories, changed command receipts, changed
  stdout/stderr digests, changed module outcomes, and schema-version mixing;
- fixture Lean runs cover the tiny smoke tier and local Lake tier before
  reference-suite fixtures are admitted; and
- a final dry gate runs without touching tracker state, ignored `.runs`, live
  providers, or unrelated repository state.

## Non-Goals

- No Lean proof implementation in this design step.
- No Crouzeix or Jin theorem-status claim.
- No live provider execution.
- No mutation of ignored `.runs` evidence.
- No tracker edit.
- No runtime-owned `docs/import-receipt.md` refresh; normal Harp design
  landings still refresh the repository receipt after tracked docs settle.
- No repository-wide license decision.
- No automatic vendoring from a sibling worktree.
- No global Lean or Lake installation management.
- No training-data generation.
- No implementation plan in this document.

## Open Decision Boundaries

The implementation design that follows this spec will still need owner review
for these authority-widening choices:

- where pinned Lean/toolchain bytes live when they are not already committed;
- which reference proof-suite modules are licensed and small enough to admit
  into a Harp-owned manifest;
- whether the axiom/trusted-declaration audit is mandatory for all tiers or
  only for target-route tiers;
- whether receipts stay under a local run root or later become publishable
  evidence; and
- when a route-specific Jin/Crouzeix module is allowed to consume a passed
  proof-suite gate receipt.

Until those choices are made in a reviewed implementation spec or plan, this
gate remains a local hermetic runner contract and not a formal-route landing
authority.

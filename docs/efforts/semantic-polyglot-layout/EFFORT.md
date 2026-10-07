# Semantic polyglot repository layout

Status: specification published; implementation not started.

## Intent

Agree a Corenius-only principle that first-party directories express conceptual ownership, with narrowly documented exceptions. Decide whether Corenius needs Bazel separately from the layout principle. Deliver an agreed policy and one working reference component before specifying broader migrations.

## Authority

The user authorized the design interview and recording its settled decisions. Product implementation, commits, pushes, pull requests and merges are not authorized by this interview.

## Classification

Gated architectural work: Corenius repository layout and build boundaries. Current work is design and documentation only, in an isolated worktree.

## Settled decisions

On 2026-10-05 the user accepted the first-round recommendations:

1. Superseded by the scope correction below.
2. First deliver an agreed policy and one working reference component; broader migrations follow evidence.
3. Allow narrow documented exceptions, including externally mandated, generated and vendored layouts where restructuring creates maintenance costs.

## Design completion

The user proceeded to specification after approving the final naming and Linux-only scope. Independent critique completed and its contracts were reconciled in SPEC.md. Implementation remains a subsequent phase.

## Repository context

Read at Corenius commit 48d71588fff00ac70fa4a29e066a47a6b05f2ebd. Corenius currently has a standalone Rust package, independent nested Rust workspaces and a Python benchmark package; no Bazel build files were found. CONTEXT.md defines hermetic as conformance to declared inputs and execution assumptions, separately from network isolation or reproducibility guarantees.

Dated sibling references describe Milano removing Bazel from its active build path and Torchetta retaining Spack for external libtorch builds. These snapshots are not verification of current sibling state. Inference: a blanket Bazel mandate would require reopening repository-specific build decisions; the accepted separation avoids assuming that mandate.

## Scope correction — 2026-10-05

The user explicitly limited this effort to the current Corenius repository. This supersedes the initial workspace-wide scope. Sibling repositories, shared policy rollout and their build decisions are outside scope. Select reference evidence from actual Corenius components rather than introducing tokenizer or Torch examples merely to demonstrate polyglot builds. Bazel adoption and the extent of source relocation remain open decisions.

## Second-round decisions — 2026-10-05

The user accepted these recommendations: improve existing component boundaries while preserving idiomatic internal layouts; evaluate Bazel against a concrete Corenius problem; prioritize separating product, engineering tooling and experiments, then organize each by responsibility. No source move or Bazel adoption is approved yet.

## Current component evidence

Read at the same Corenius commit: the terminal agent is the root Cargo package; SQLite controller and team control plane are independent nested crates. SQLite already co-locates Python fixtures with Rust implementation. Python controller/operator tooling also lives under tools/agents. Benchmarking and collaboration experiments have existing semantic directories. Inference: the useful first reference is the existing SQLite client integration, rather than adding a synthetic PyO3 or CUDA component. Its Rust test invokes a Python fixture against the actual controller binary; platform and dependency requirements still need verification.

## Third-round decisions — 2026-10-05

The user accepted terminal-agent/, sqlite-controller/ and team-control-plane/ as semantic product boundaries, retaining idiomatic Rust internals. Controller-specific tooling follows its owner; engineering worktree, tracker, evidence and landing tools stay under tools/agents. Existing SQLite client integration is the reference proof. Aim for one root Cargo workspace with the terminal agent as default member, subject to checking dependency/toolchain compatibility. These are design decisions, not authorization to implement source moves.

## Fourth-round decisions — 2026-10-05

Preserve documented root Cargo commands using the default workspace member. Update internal paths and documentation together; retain forwarding scripts only for established consumers. Package independently used Python tools when dependency management warrants it, not every fixture directory. Give controller launchers an explicit binary input and retain an explicit build-and-launch convenience command. Preserve runtime behavior and qualification status in the first migration.

## Compatibility inspection

All three packages declare edition 2024 and minimum Rust 1.95. This does not prove their combined dependency resolution succeeds. Consolidation must preserve locked/offline gate requirements and explicit package features. Existing fixtures use Cargo manifest and binary environment variables, a depth-based repository lookup, absolute Python paths and source-relative scratch outputs; relocating them requires checking these contracts. The selected reference requires Linux process APIs and Unix sockets. Inspection is not execution evidence; no tests have run in this design phase.

## Fifth-round decisions — 2026-10-05

Defer Bazel adoption; first implement semantic layout and evaluate Cargo consolidation. The initial reference-only Linux decision is superseded by the Linux-only scope correction below. Update active entry points and current navigation; preserve historical evidence and commit-specific snapshots, with relocation notes as useful. If consolidation cannot preserve dependency or offline-build behavior without unrelated changes, retain independent Cargo workspaces temporarily and record a follow-up blocker.

## Agreed design for confirmation

- Scope: current Corenius only; no sibling policy edits or rollout.
- Active product directories: terminal-agent/ and controller/, retaining idiomatic package internals and colocated cross-language support. The parked PostgreSQL/Temporal implementation belongs under experiments/team-control-plane/.
- Engineering, benchmarks and exploratory code retain tools/agents/, bench/ and experiments/; controller-specific launchers and views move with their actual owner after dependency inspection.
- Root Cargo workspace and one lockfile are preferred, with terminal agent as default member; concrete compatibility blockers permit independent workspaces temporarily.
- Preserve root user commands; update current paths, gate registrations and documentation. Forward old script entry points only where established consumers require it.
- Python packaging is selective; fixtures need no synthetic uv packages.
- Controller launchers accept an explicit binary; a documented build-and-launch convenience command remains.
- Linux is the sole supported platform for this effort. SQLite client integration is the real reference proof. Preserve its existing public-interface checks, output evidence and process cleanup. Fix relocation-sensitive paths without changing runtime behavior.
- No Bazel, synthetic PyO3/CUDA component, controller redesign, platform expansion or increased live-worker qualification claim in the first migration.
- Acceptance: verify preserved documented commands, relevant Rust/Python component checks, locked/offline gate behavior and the real reference integration after relocation; review active path references and final diff. No success evidence is claimed before execution.

## Ambiguity review

The initial workspace-wide scope is explicitly superseded. Shared Cargo resolution is an aim with a concrete fallback, not a prerequisite for semantic organization. Product layout and build-system adoption are separate. Historical references are preserved rather than silently rewritten. Exact file ownership and compatibility findings belong in the implementation inventory/specification; ambiguous cross-component tools remain in place until ownership is established.

## Platform scope correction — 2026-10-05

The user clarified that only Linux matters. This supersedes the earlier requirement to preserve other components' platform support. Design, dependency choices, launchers and verification may target Linux directly; no portability abstraction, non-Linux compatibility shims or platform matrix is required. Do not add unrelated deletions of dormant platform code to a behavior-preserving layout migration.

## Component naming and status correction — 2026-10-05

The user approved controller/ for the active SQLite-backed implementation and experiments/team-control-plane/ for the retained, parked PostgreSQL/Temporal implementation. This supersedes the earlier three-product directory proposal. Controller names its responsibility: durable state, permissions, work admission and execution reconciliation; SQLite is its storage implementation. Coordinator was considered but understates these authority responsibilities.

Source evidence at the recorded commit: docs/efforts/team-control-plane/EFFORT.md explicitly marks the richer roadmap parked, preserved and unfinished in favor of the SQLite baseline. Treating both as equally active products was incorrect. Preserve the older implementation and its qualification caveats; relocation does not revive or complete that effort. Cargo membership/default gates for the parked implementation must remain explicit in specification rather than silently making it a default product build.

## Specification publication — 2026-10-05

Published SPEC.md to the durable local tracker with Label: ready-for-agent. Existing Linux client integration and component gates were already approved as test seams; no new test API is proposed. A fresh-context reviewer inspected the design against source and reported six executable-contract gaps. The specification resolves them by preserving current experimental gate participation, declaring workspace membership/lock ownership and fallback, defining an adjacent-binary launch-only interface plus existing build-and-launch convenience, preserving relative dataset interpretation, retaining root-local scratch/evidence and assets, and requiring existing integration assertions and process cleanup.

No Rust/Python runtime tests were run or product sources changed. Next phase: blocker-linked tickets after specification review. Git publication and merge authority remain absent.

Independent adversarial specification review found no blocking gaps after reconciliation. This is design/specification evidence only, not proof of migration behavior.

# Corenius semantic repository layout

Status: ready-for-agent
Label: ready-for-agent
Type: specification
Source revision: 48d71588fff00ac70fa4a29e066a47a6b05f2ebd

## Problem Statement

Corenius combines an interactive terminal agent, an active engineering controller, repository engineering tools, benchmarks and retained experiments. The root Rust package and generic crate container obscure these responsibilities. Controller-specific Python launchers and operator views are mixed into the engineering toolbox, separating parts of one product. A parked PostgreSQL/Temporal implementation can appear to be an equally active product.

Developers need to navigate by responsibility without losing working commands, integration checks or retained evidence.

## Solution

Organize the current Corenius repository around its actual components: terminal-agent/ and controller/ are active product directories; tools/agents/ remains the repository engineering toolbox; bench/ remains benchmarking; experiments/ retains exploratory implementations, including the parked team-control-plane/ implementation. Languages may coexist within a component. Preserve idiomatic package internals.

Support Linux only. Prefer a root Cargo workspace with the terminal agent as its default member and a shared lockfile where compatible. Preserve independent workspaces when a demonstrated compatibility blocker requires them. Defer Bazel, and use the existing Rust/Python controller client integration as the working reference.

## User Stories

1. As a developer, I want the terminal agent in a named product component, so that I can find its implementation without assuming the entire repository is one Rust application.
2. As a developer, I want the active controller named for its responsibility, so that its storage choice does not define the architecture.
3. As a controller maintainer, I want Rust implementation and Python support together, so that I can understand one component across languages.
4. As an operator, I want controller launchers and views beside their owner, so that I can find the supported operational interfaces.
5. As a repository maintainer, I want worktree, tracker, evidence and landing tools to remain in the engineering toolbox, so that product and development responsibilities stay distinct.
6. As a benchmark maintainer, I want benchmark harnesses to retain their boundary, so that evaluation infrastructure stays easy to locate.
7. As a researcher, I want collaboration experiments retained separately, so that exploratory code is distinguishable from product implementation.
8. As a developer, I want the old PostgreSQL/Temporal control plane clearly marked parked and unfinished, so that I do not mistake its presence for current product qualification.
9. As a developer, I want existing root Cargo run commands to select the terminal agent, so that routine use remains familiar.
10. As a maintainer, I want package-targeted tests and features to remain explicit, so that controller checks do not silently change meaning.
11. As a developer, I want one workspace and lockfile when compatible, so that package metadata and dependency management are coherent.
12. As a developer, I want a documented fallback when consolidation is incompatible, so that an unrelated dependency problem does not prevent the layout improvement.
13. As an offline developer, I want required locked and offline build behavior preserved, so that the migration does not introduce ambient resolution assumptions.
14. As a Python maintainer, I want packaging only where dependencies and independent use warrant it, so that test fixtures do not become artificial packages.
15. As an operator, I want to supply explicit controller executables to launchers, so that operational startup has an identifiable build input.
16. As a local developer, I want a documented build-and-launch convenience command, so that explicit artifact inputs do not make normal use cumbersome.
17. As an existing tool consumer, I want established entry points preserved where needed, so that relocation does not break automation silently.
18. As a reviewer, I want the real controller client integration to keep exercising Rust and Python together, so that the migration is proven through public behavior.
19. As a reviewer, I want integration output, evidence and process cleanup preserved, so that passing assertions do not conceal leaked or untracked executions.
20. As a terminal-agent user, I want bundled evaluation data and documented dataset behavior to remain available, so that moving the package does not break evaluation setup.
21. As a maintainer, I want current navigation and commands updated while historical records remain intact, so that both present use and past evidence remain understandable.
22. As a Linux developer, I want requirements stated directly for Linux, so that portability scaffolding does not complicate this migration.
23. As an owner, I want no runtime, schema, protocol or qualification expansion hidden in the relocation, so that I can assess the change as a bounded migration.

## Implementation Decisions

- Apply this migration only to Corenius; do not edit sibling repositories or distribute a shared policy change.
- Use the terminal agent and controller as the active product boundaries. Keep controller-owned Rust, Python, tests, fixtures and operational support together. Retain the existing library and executable identities unless an independently approved compatibility decision requires otherwise; directory naming does not itself require renaming binaries or packages.
- Move the older PostgreSQL/Temporal control plane into the experiments boundary, preserve its implementation, and explicitly state its parked, unfinished status. Do not resume its roadmap or import frozen unaccepted candidates.
- Preserve the existing engineering, benchmark and collaboration-experiment boundaries. Inspect dependencies before moving ambiguous shared tools; leave them in place when ownership is not established. Do not create a generic shared module merely because two components use similar code.
- Aim for a root Cargo workspace with the terminal agent as default member. Preserve explicit package features and relevant lockfile semantics. Consolidation must be verified rather than inferred from matching minimum Rust versions. If it requires unrelated dependency changes or breaks required locked/offline behavior, retain independent workspaces temporarily and record the exact blocker, evidence and follow-up work. The layout must still be completed.
- In the consolidated outcome, all three Rust packages are workspace members; only the terminal agent is a default member. The root lockfile is authoritative and redundant member lockfiles are removed only after compatibility verification. Preserve the current Rust package and binary identities. In the fallback outcome, the root workspace contains the terminal agent and any compatible active member; independent packages are explicitly excluded and retain their own workspace declarations and authoritative locks. Record exactly which members consolidated and why each exclusion is necessary. Incorrect workspace wiring is not a compatibility blocker.
- Keep the parked implementation out of default Cargo selection, but preserve its existing repository-gate participation: the engineering gate still invokes its PostgreSQL/Temporal runner. Relocation does not make that check optional. Retain a separately documented direct service-backed invocation and its current failure behavior. Any later change to default gate participation is separate work.
- Preserve documented root terminal-agent commands and their behavior. Account for workspace output locations, manifest-relative asset lookup, fixture imports, repository discovery, executable discovery and scratch/evidence roots when relocating components. Avoid relying on the old directory depth.
- Keep supported component-local build and test entry points usable, updating their documented output locations if a shared target directory changes them. Preserve default evaluation dataset availability and the pre-migration interpretation of relative dataset arguments from documented root commands; absolute dataset arguments remain unchanged. Inventory bundled evaluation data separately from scratch outputs so the package move does not silently redirect inputs or evidence.
- Preserve repository-local scratch/evidence destinations and controller SQL assets, imports and runtime support adjacency. Replace relocation-sensitive depth assumptions where necessary. Produce a current ownership inventory during ticket decomposition, including launchers, relay, views, their tests, fixtures and SQL assets; implementation must not infer ownership solely from a filename.
- Provide a launch-only mode accepting an explicit binary directory containing the existing controller, supervisor and gate executable names. Preserve the adjacent companion arrangement; do not allow the launcher to mix binaries from unrelated build directories. Validate required executable availability before creating a session or services, and fail clearly when incomplete. Launch-only mode does not invoke Cargo. Keep the existing start command as an explicit documented build-and-launch convenience path, preserving its custom target-directory selection and then delegating to the same launch implementation. Keep relay discovery and pinned executable evidence consistent with relocation.
- Move tests with their owning implementation. Preserve established old script entry points through thin forwarding only when current documentation or automation demonstrates a consumer. Forwarding must preserve arguments, exit status and relevant working-directory behavior, and must not duplicate implementation.
- Package independently used Python tools only when their dependency-management needs justify it. Preserve existing Python project metadata and locks where used. Do not require a root uv workspace or synthetic packages for standard-library fixtures.
- Linux is the sole supported platform. No new non-Linux support or portability abstractions are required; unrelated dormant platform code removal is not necessary for this migration.
- Preserve public protocols, schemas, authorization, admission, reconciliation and runtime behavior. Preserve qualification caveats, including the unqualified complete live-worker baseline.
- Update active commands, navigation, gate registrations and current source maps together. Keep historical effort records, immutable decisions, evidence and commit-specific snapshots intact; add relocation guidance where needed. Do not reinterpret historical artifact paths as new locations.
- Keep the local design record and architectural decision authoritative for this effort; do not modify the synchronized workspace-wide policy merely to document the Corenius layout.

## Testing Decisions

- Prefer external behavior through existing seams. Do not add tests that assert source text, directory names alone or implementation-shaped mocks.
- The user approved the existing Linux controller client integration as the primary mixed-language seam. It starts the real Rust service and uses the Python client fixture through its actual Unix socket, dashboard and MCP interfaces. Preserve its pagination, large UTF-8 payload, scope-rejection, idempotency and retained evidence checks; preserve identity-bound process cleanup.
- Run relevant terminal-agent tests and deterministic CLI checks to prove root command selection and packaged evaluation asset discovery without spending real-model calls. Existing provider fixtures and CLI tests are prior art; network-dependent provider qualification is not required by relocation.
- Run existing controller Rust suites with the same feature and concurrency intent, plus the Python launcher, relay and operator-view suites affected by moves. Where explicit binary input is new behavior, verify that path through public launcher behavior with no implicit build and preserve the convenience path.
- Use the existing controller client integration test target under its unchanged Rust package identity, with the locked/offline and fault-injection settings of the controller gate. Require successful socket/CLI/dashboard/MCP checks, pagination and retained hashes, scope rejection and receipt replay, archived evidence and waited owned-process exit. Do not weaken assertions to make moved paths pass.
- Check manifest/package selection and required locked/offline commands from documented invocation locations. A successful metadata query alone is not build or integration proof. Keep baseline failures distinct from migration regressions.
- Preserve the parked control plane's explicit service-backed verification entry point after moving it. Run its required gate when fixtures are available; missing Docker, service images or dependencies must be reported as unverified evidence, never a pass. Do not replace service tests with weaker mocks.
- Run the repository's required engineering gate and relevant benchmark/Insula checks where affected. Separate implementation verification from qualification of existing live-worker scenarios.
- Inspect active path references and the final diff, classifying intentionally retained historical references rather than rewriting them. Exercise forwarding entry points only where retained consumers require them.
- Retain fresh command results, exit statuses and source revision for completion claims. Do not describe hermetic execution, network isolation and reproducibility as interchangeable guarantees.

## Out of Scope

Bazel adoption; sibling migrations or shared policy rollout; synthetic PyO3, C++ or CUDA examples; a mandatory uv workspace; controller redesign; schema/protocol changes; new storage or workflow backends; dependency upgrades unrelated to compatible workspace resolution; revival of the parked roadmap; import or deletion of frozen external worktrees; historical evidence rewriting; expanded live-worker qualification; non-Linux support; real-model trials solely to prove source relocation.

## Further Notes

This specification synthesizes the approved design interview and its scope/name corrections. Inspection was performed against the stated source revision; dependency compatibility and runtime acceptance remain unverified until implementation checks run. The existing controller integration is the working reference, not a new component.

Ready-for-agent denotes specification readiness in the local tracker, not permission to skip blocker-linked tickets, review gates or explicit Git authority. This request authorizes specification writing; it does not authorize product edits, commits, pushes, pull requests or merges. Subsequent execution must use isolated working trees and independent Standards/Spec review, with human-owned merge.

Independent design critique identified six contracts: gate participation, workspace/lock ownership, companion binaries, evaluation assets, root/evidence discovery, and reference evidence. These are reconciled above by preserving current gate behavior, declaring both workspace outcomes, retaining adjacent companions with launch-only validation, preserving dataset interpretation and evidence roots, and requiring the existing public-interface assertions. No review finding is treated as executed test evidence.

A subsequent independent adversarial review of this specification found no blocking gaps. Exact invocation commands and the ownership inventory are delegated to blocker-linked tickets; dependency and runtime verification remain outstanding.

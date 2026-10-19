# Harp ownership and repository layout

Status: implemented and verified; see the repository qualification record.
Baseline: `5342c2e3b08e8671e3ef46aca83bcbbc79b7f5a3`.
Intent and delivery slices: [local specification](../../../.scratch/repo-architecture/spec.md).

## Decisions

Harp has one durable execution control plane for agents. The selected
Python replay runtime, SQLite operational store, guardian protocol and coding
workflow become first-party Harp code. The imported product directory and
`harp_durable/` disappear after their callers migrate. Temporal, PostgreSQL and
retired controller implementations have no execution path in the target tree.

Insula owns isolation and execution-environment construction. Runner transports
admitted work to an SSH host. Workloads live in `lab/`, including maintained
mathematics and SICP. These responsibilities remain separate from durable
execution authority. TLA+ is deferred until those boundaries and their
executable checks are established.

The new Bazel, Triton and training workloads are architecture and follow-up
tickets in this pass. Their implementation and qualification are future work.
The existing-code candidate uses the layout below. The three new workload directories are planned follow-ups.

## Target tree

```text
durable/                    one Python execution control plane
  __main__.py               implementation behind harp durable
  runtime/                  replay, activities, guardian, receipt acceptance
  state/                    SQLite schema, transactions, operational records
  agents/                   admission, Codex adapter, coding activities
  intent/                   Git-managed specs, issues, dependencies and acceptance
  collaboration/            bounded assignments, worktree ownership, integration policy
  git/                      source/candidate/verification/landing domain
  workflows/                coding, verification and workload orchestration
  workloads/                frozen workload manifests, execution and artifacts
  execution.py              bind runtime identity to an Insula execution
insula/                     capsule, rootfs, projection and isolation mechanisms
  owner/                    shared launch gate and capability-limited service owner
  supervisor/               supplied Rust service lifecycle library and owner client
runner/                     SSH transport, preflight, request/result transfer
  hosts.example.toml        fictional example only
lab/
  math/                     Lean developments, mathematical prose and textbook
    textbook/               textbook coverage, exercises and compatibility JSON
  sicp/                     evaluator, course prose and educational exercises
  benchmarks/               retained terminal/build/coding task collections
    legacy_runtime/         isolated historical Rust runtime qualification workspace
  soma/                     Emacs/Doom environment, gym, probes and AI/LSP workloads
  mixed_build/              compact C++/Rust/Python Bazel workload
  triton_compile/           Triton compilation and correctness workload
  training/                 small transformer training workload
cli/                        Harp Rust CLI and supporting product library
sessions/                   rollout indexing and session analysis
evidence/                   captured bytes, licenses, provenance, qualification
tools/                      shared authoring, PDF and repository tools
scripts/                    repository verification and maintenance
docs/                       product/operator documentation and decisions
.scratch/                   Git-managed intent and issues
```

`lab/` describes the role of executable work, not its quality or importance.
Novel Crouzeix proofs remain maintained work. Maintained prose and coverage contracts live beside their workloads;
captured evidence keeps its original location and bytes; each workload manifest
names those inputs explicitly. Moving executable proofs does not require copying
the textbook, changing captured source bytes or creating a second prose authority.

The root Cargo workspace remains the product build. Bazel initially belongs to
`lab/mixed_build/`; it does not replace the Harp build or the Lean toolchain.

## Authority and dependency boundaries

| Component | Owns | Must not own |
| --- | --- | --- |
| `durable/runtime` | Dispatch, replay, attempts, capacity, cancellation, cleanup gating and receipt acceptance | Git intent, proof truth, host discovery |
| `durable/state` | One project workflow SQLite database and its transactions, including approval and landing records | A second scheduler, editable task intent |
| `durable/agents` and `durable/workflows` | Freeze inputs; orchestrate provider, verification and workload activities | Provider-specific approval or recovery policy |
| `durable/intent` | Parse and validate Git-managed specs/issues/tickets, dependencies and acceptance inputs | A second editable backlog in SQLite, implicit intent changes to admitted work |
| `durable/collaboration` | Bounded assignments, isolated worktree ownership and integration policy | Another dispatch/recovery loop, widened worker authority |
| `durable/git` | Exact Git subjects, change-policy validation, verifier inputs, approval binding and physical landing rules | A separate scheduler, database or acceptance authority |
| `insula` | Rootfs/layer identity, mounts, namespaces, network policy and confinement mechanisms | Workflow replay, task status, approval, provider scheduling |
| `insula/supervisor` | Declared service lifecycle, health/readiness, generations and bounded service recovery | Agent jobs, workflow retries, approval, direct service launching/reaping |
| `runner` | Resolve a local host alias; transport pinned requests and artifacts; invoke/query remote execution | Retry an ambiguous effect, infer cleanup from SSH loss, approve or land code |
| `lab` | Workload source, inputs, expected outputs and independent oracles | Credentials, host configuration, controller state or duplicate orchestration |

The Rust CLI invokes the one Python control plane. Durable workflows call
`durable/intent`, `durable/collaboration`, `durable/git` and Insula mechanisms.
Runner submits to that same control plane on the selected host and reconnects by admitted run identity.
Insula does not import durable. Internal Git and intent rules do not call back
into workflow dispatch; workflows compose them through the one runtime/store.
Workloads cannot import controller internals to create or alter accepted state.

Workflow SQLite state stays inside `durable/state/`. Extract the existing store and schema with
their transaction boundaries, rather than introducing a generic database service.
Git approvals and landing intents remain in that same operational database.
Expose the required state transitions through the durable store; relocating
`landing.py` must not leave arbitrary SQL writers scattered across packages.
The engine's own transactional queries are implementation details of this owner.

Guardian launch/release, lock inheritance and acceptance policy stay with durable
execution. Insula may supply cgroup operations, but the controller decides when
an attempt owns capacity and when proven cleanup permits reuse. The execution
binding is `durable/execution.py`; Insula has no durable registry or second CLI
that can submit agent work independently.

The internal `durable/git` boundary represents domain rules and Git operations.
Durable orchestration owns when they run and records their outcome. Verification
remains an independent activity. A passing workload or provider command is not
an approval, and the destination only changes through the owner-approved path.

## Provider and test scope

Codex is the first supported provider for the migrated framework. Qualify the
one/two/four-worker lifecycle using Codex. This supersedes the earlier requirement
for mixed Codex/Trae qualification in this migration. Preserve existing Trae
source and historical evidence in the disposition record; further Trae migration
and qualification are deferred, not silently claimed complete or discarded.
Pi is a possible later agent harness, not a current implementation dependency
or acceptance requirement. Keep a narrow provider boundary grounded in Codex;
do not build speculative multi-provider infrastructure for Pi now.

Tests live beside the source they protect. Use adjacent Python test modules,
Rust in-module or neighboring test modules, and nearby fixtures. Native
integration/acceptance targets should also reference source-adjacent files via
explicit build configuration where needed. Do not add separate component-wide
or repository-wide tests/ trees. During import, preserve the supplied archive's
original test identities and behavior while mapping their new colocated paths.
Colocation changes file placement, not the observable contracts being tested.

## Durable agentic engineering framework

Durable includes the Git-managed engineering workflow as well as execution.
`durable/intent/` supports the existing local-Markdown contract: specs, issues,
tickets, acceptance criteria and dependency links authored in the Matt Pocock
skill style. Those records remain under `.scratch/<feature>/`, outside the code
package. Materializing intent into detailed specs and independently verifiable
implementation tickets follows the Matt Pocock-style authoring workflow and
existing Harp skills. The framework supplies record parsing, validation, revision
binding and execution handoff; it does not replace human design decisions with a
second requirements generator or infer a detailed spec from a task title.
Reuse the [tracker contract](../../agents/issue-tracker.md); do not
invent another tracker format or require a service to use it.

Git owns authored intent and issue state. Admission freezes their exact revisions
and bytes, declared change scope and acceptance inputs for a run. Editing a
spec/ticket afterwards does not mutate that admitted run. SQLite owns execution
attempts, observations and accepted operational results. It may retain immutable
input snapshots and source references; it is not a second editable backlog.
Updating a ticket is a deliberate Git change bound to its evidence, rather than
an implicit consequence of provider exit or a database status update.

`durable/collaboration/` composes bounded assignments, issue dependencies,
non-overlapping worktree ownership, handoff and integration decisions using the
same intent records and runtime. It computes policy and submits admitted work;
`durable/runtime/` alone dispatches, accounts capacity, cancels and recovers it.
Conflicting changes require a fresh integration candidate and verification;
collaboration never grants workers approval or landing authority. Keep the
existing one/two/four-worker scope and explicit reconciliation rules.

`durable/git/` supplies source snapshots, worktree and candidate identities,
change-policy enforcement, independent verification and exact approved landing.
Intent, collaboration, Git operations and execution are internal parts of one
framework with distinct responsibilities. There is no top-level Git-workflow
product or compatibility package to maintain.

## Supplied service supervisor

The [inspected supervisor archive](supervisor-integration.md) adds an existing
Rust component under insula/supervisor/, not another agent controller. Reuse its
service lifecycle implementation. Its private SQLite store owns service intent
and outcomes only; durable/state/ continues to own the one workflow journal,
including candidate acceptance and landing. Neither component writes the
other's database. Service replacement cannot resubmit workflow effects.

The real ExecutionOwner adapter is not supplied. Qualify it against the actual
Insula namespace, launch and cleanup path, including effect-time epoch/generation
fences and root ownership shared with activity admission. The current guardian
and capsule code are ingredients, not demonstrated implementations of that
long-lived-owner protocol. The fixture's direct-process results do not establish
service containment or survival under the current bwrap lifetime. Keep unsupported
capabilities refused and do not create a second launcher/reaper to fill the gap.
Ticket 13 covers acquisition, dependency reconciliation and actual owner tests.

## Migration from current code

| Current source | Target or disposition |
| --- | --- |
| Imported `controller/durable/engine.py` | Replay/runtime in `durable/runtime/`; store/schema in `durable/state/` |
| Imported guardian and receipt helpers | `durable/runtime/`, with the existing launch/cleanup protocol |
| Imported durable Insula binding | `durable/execution.py` |
| `harp_durable/` admission, activities, providers and public CLI | `durable/agents/`, `durable/workflows/`, `durable/__main__.py` |
| Candidate, verifier and landing domain code | `durable/git/`, orchestrated by durable workflows |
| Imported capsule/rootfs/temp-root mechanisms | `insula/`, with all callers and sealed-code manifests migrated |
| Imported benchmark tasks and useful pure validators | `lab/benchmarks/`, using the selected execution path |
| `formalization/` and proof companion programs in `labs/` | `lab/math/`, preserving Lean module names and proof authorities |
| `labs/sicp-evaluator/` and executable SICP exercises | `lab/sicp/`, updating Cargo membership and consumers |
| Root `benchmarks/` | `lab/benchmarks/` where applicable; classify each collection |
| Old controller/Collab/lead/worker/terminal launch implementations | Retire from current source; retain exact source identities and ancestry |
| Imported design/qualification history | Classify useful maintained material, captured evidence and history-only records; no bulk active-code archive |
| Imported source ledger | Move to provenance records with source-to-destination or retirement entries |

The import verifier currently requires an unchanged imported path set. Replace
that assumption with an explicit relocation/disposition ledger before removing
the wrapper directory. Preserve all absorbed branch/worktree ancestry, original
blob identities, modes, source-specific licenses and any captured evidence that
must remain in-tree. Verify mapped retained bytes and reviewed adaptations.
History-only records remain recoverable through exact Git identities. A removed
directory must not be recreated as a compatibility package or vendored archive.

Migrate callers, package names, command arguments, sealed code snapshots,
qualification scripts, docs and the changed-path selector together. No persistent
alias for the old Python packages. Retired controller stubs can disappear once
absence and supported-entrypoint checks replace their refusal tests.

A package move changes the pinned execution identity. Do not rewrite old input
bundles, receipts, execution records or SQLite journals to make them appear to
have run new code. Reconcile old active/uncertain work under its original profile;
prepare a new profile for new work. Specify old-state inspection support before
claiming a migration preserves recovery.

## Structural trust repairs

The preceding audit found observable failures at this baseline. Renaming packages
does not repair them. The migration carries explicit tickets for each contract.

1. **Physical landing.** An ignored destination `.gitattributes` can convert
   `42\n` to `42\r\n`, advance HEAD and index, then return refusal. Qualify the
   effective checkout policy before mutation. A rejected candidate must preserve
   branch, index and bytes. Successful landing must match the verified raw tree.
2. **Independent acceptance.** Frozen argv can run a checker changed by the
   candidate. The audited candidate changed only its checker, left the wrong
   answer, and received a passing verification proof. Admit owner-controlled
   oracle inputs outside the candidate and an explicit change policy. Keep the
   oracle process outside candidate-controlled code when that code could falsify
   the test process's exit status. Tests/config/code-generation changes remain
   expressible through an owner-admitted policy; a universal tests ban is wrong.
3. **Native qualification.** The portable runtime suite exits successfully with
   forty Linux tests skipped on the development host. Keep portable feedback, but
   require a separate native mode for relevant runtime changes. It must fail
   missing capabilities and required skips and bind its result to the actual
   runtime, CLI bridge and backend source bytes. Historical results cannot
   qualify changed code.

Evidence was produced with real Git and disposable SQLite state, plus an actual
Linux public-CLI/namespace run with a deterministic worker. It did not show a
namespace escape or forged receipt. Preserve those bounds when converting the
reproducers into tests.

Owner capability isolation protects against contained workers. A same-user host
operator remains trusted. Cleanup acknowledgment is currently an evidence-bearing
operator assertion. Directory boundaries and a model checker cannot turn that
assumption into mechanical human-only authorization.

## Runner and private host configuration

The first runner handles an explicitly selected SSH host. It checks the declared
OS/architecture, available tools, execution profile and delegated containment
before dispatch. OpenSSH configuration supplies routing and host-key trust.
Do not disable host-key checking, interpolate shell fragments from workload data
or install remote services as a side effect of a run.

The real inventory is local and ignored, for example `runner/hosts.local.toml`,
or lives outside the checkout. Commit a schema and a fictional example only.
Credentials and their private host paths do not enter public workload manifests,
request examples, logs or source archives. Sanitized public results identify a logical host profile
and relevant software/hardware capabilities. An opaque alias can itself disclose
identity; it must be reviewed along with logs and paths.

SSH disconnect is a transport event. Query the admitted remote run before any
further action. Cancellation requires remote acknowledgment and cleanup evidence;
losing the connection proves neither cancellation nor available capacity. Pin
source/profile identity before transfer and verify returned artifact identities.
There is no new fleet scheduler or host migration protocol in this first runner.

Privacy checks cover tracked runner files, generated requests, logs and report
fixtures, including a forced-stage test for a real local inventory. Audit existing
tracked records for exposure separately; adding an ignore rule does not sanitize
already tracked files or Git history. Captured evidence cannot be silently edited.

## Benchmark completeness

The [benchmark inventory](benchmark-inventory.md) expands the earlier directory
map. All existing benchmark payloads are retained under lab entries, including
fixtures/prototypes in effort records, tmux scenarios, catalogs and reports.
Soma's Emacs gym and DevX's AI/LSP workload are separate source acquisitions;
they were referenced by the imported review but are not in the absorbed tree.
Their dirty source snapshots must be captured and their support closure migrated.
Soma comes in as a coherent project under lab/soma/, including its gym, bridge,
workers, probes and maintained source documentation. Shared isolation and service
mechanisms move into Insula; execution/collaboration uses durable. The separately
sourced AI/LSP verifier lives under lab/soma/ai_lsp/ with its own provenance.
Ticket 12 makes that work a prerequisite for final existing-code acceptance.
Captured evidence keeps its canonical bytes and location, referenced by the lab
manifest. Competing scheduler retirement must not discard benchmark scenarios.

## Workload contracts

Each workload declares a versioned ID, source/input identities, required execution
capabilities, an argv entrypoint, output paths and an acceptance oracle. This is
data consumed by the existing runtime, not a new task DSL or scheduler. Required
GPU/native profiles fail preflight when unavailable; a CPU smoke test cannot
qualify a GPU workload. Results record source, environment and oracle identities.

| Workload | Bounded initial operation | Required evidence |
| --- | --- | --- |
| Mathematics | Build an existing Crouzeix route and extract its proof receipt | Same theorem/route authority, module identities, checked receipt and PDF consumers |
| SICP | Build/run the maintained evaluator and its existing behavioral tests | Preserved evaluator semantics and education/source links |
| Existing benchmarks | Materialize and grade a retained representative task | External oracle, clean task workspace, fixture separation, no retired agent launcher |
| Mixed Bazel build | Tokenize a tiny corpus and run a differentiable tensor operation through the Python consumer | Real C++/Rust/Python dependencies, code generation, gradients, tokenizer round-trip and build-impact tests |
| Triton compilation | Compile fixed kernels and execute them against an independent reference | Explicit cold/warm conditions, correctness, toolchain/target identity and measured compilation phases |
| Training | Train a tiny transformer, checkpoint, restart and continue | Loss/gradient sanity, deterministic bounded reference, checkpoint/optimizer continuity and a real supported GPU run |

For the mixed build, use a C++ tensor/autograd core, a Rust BPE tokenizer and a
Python consumer with real native bindings. A small generated ABI/schema artifact
and a pinned build-time generator create meaningful shared build edges. Test a
clean offline build, the no-change rebuild, leaf changes and shared interface
changes. A shell wrapper that independently runs Cargo, a C++ compiler and Python
does not demonstrate a Bazel dependency graph.

The referenced `[vaso] triumvirate bazel graph` discussion, Codex task
`01a14b99-22c3-7472-a077-de10e91a8fa9`, separates rootfs/execution from Bazel builds
inside that environment and checks fresh restoration. Apply those ideas at this
smaller scale. Do not import its private estate, package registry, credentials,
published artifacts or full framework build graph. Choose pins and licensing
from the actual implementation inputs when that ticket starts.

The training job is TorchTitan-like in behavior, not a TorchTitan fork commitment.
Start with a bounded single-device run and checkpoint/resume correctness. Add
multi-device execution only with explicit resource admission and qualification.
Activity replay restarts a process; it does not restore model/optimizer memory.
Training checkpoints and effect reconciliation remain workload responsibilities.

## Verification and completion

Each move updates the relevant path selectors and consumers in the same slice.
Use public CLI lifecycle tests, real Git materialization, namespace cleanup and
the retained Lean/SICP/PDF checks. Enforce import direction and single authority
with executable checks of supported entrypoints and import dependencies. Avoid
tests that freeze every internal helper or merely search for the new spelling.

Runtime qualification progresses through one Codex worker, two Codex workers,
then four Codex workers with actual overlap, restart/cancellation tests,
independent acceptance and serialized exact approval/landing. Reuse current
behavioral tests, but bind fresh results to migrated code. A renamed tree with
only portable tests passing has not completed this migration.

Final existing-code acceptance requires no imported wrapper or `harp_durable/`,
one dispatch/recovery/approval chain, relocated executable workloads, preserved
proof/publication behavior, a working SSH-to-Insula runner, checked provenance,
and no committed private host configuration. New workload tickets remain open
until their implementations have their own evidence. TLA+ can later target the
stable guardian/recovery protocol; it is not a prerequisite for these repairs.

## Verified delivery

The existing-code migration is complete. The [qualification record](../../../evidence/provenance/repository-qualification.json)
identifies the verified body commit, source-bound native checks and complete
changed-surface gate. Tickets 08–10 and 14 remain explicit follow-ups for new
workloads and stronger service-owner capabilities.

## Remaining container cleanup

The CLI and session index are top-level components. The old top-level benchmark
fixtures live under lab/benchmarks; historical Rust controller packages and their
schema receipts form its independent legacy_runtime workspace. The product Cargo
workspace excludes that workload, and the CLI no longer depends on its engine
for schema checks.

The generic content directory is removed. Textbook JSON lives under
lab/math/textbook, SICP records under lab/sicp/contracts, embedded workflow context
under durable/context, and shared source records under evidence/sources. The
compiled corpus is lab/generated/corpus.json. Original evidence and
historical qualification source identities remain distinct from current checks.

## Knowledge consolidation

Maintained mathematical and SICP prose now lives beside its workload under `lab/`.
Benchmark notes live under `lab/benchmarks/`, shared implementation studies under
`lab/agentic_engineering/`, and portable navigation under `lab/obsidian/`. The
headless corpus is generated at `lab/generated/corpus.json`. Registered document
IDs and the `harp-knowledge/v1` schema remain unchanged. Captured upstream sources
stay in `evidence/`. See [ticket 16](../../../.scratch/repo-architecture/issues/16-merge-knowledge-into-lab.md).

## Next durable module consolidation

The agreed follow-up groups durable internals into `runtime/`, `intent/` and
`jobs/`, with coding and declared-command workflows beside their implementations.
A dedicated `estate/` module owns repository identities, managed workspaces,
attempt-bound leases, retention and safe reclamation through the existing SQLite
journal. The runtime owns attempt lifetime; estate owns workspace lifetime;
coding verification and explicit human approval continue to govern landing.
Workers currently use isolated bundle clones; introducing estate must preserve
that isolation. Runner supplies SSH transport and Insula supplies containment.
This is a planned change, tracked in [ticket 17](../../../.scratch/repo-architecture/issues/17-consolidate-durable-modules-and-estate.md).

# Benchmark migration coverage

Status: inventoried for migration; payloads have not moved.
Harp source baseline: `5342c2e3b08e8671e3ef46aca83bcbbc79b7f5a3`.
Parent: [architecture](design.md). Delivery: [benchmark migration](../../../.scratch/repo-architecture/issues/07-relocate-benchmarks-and-provenance.md)
and [Emacs migration](../../../.scratch/repo-architecture/issues/12-preserve-emacs-benchmarks.md).

## Retention contract

All existing benchmark work belongs in the lab migration. The source search must
cover absorbed branches and worktree captures, effort evidence, prototypes,
fixtures, hidden graders, reference solutions, harnesses, inventories, reports
and uncommitted benchmark changes. Limiting the search to the imported bench/
directory is insufficient. A generic history-only retirement must not silently
drop a benchmark collection.

Executable suite code, task fixtures, independent graders, reference solutions,
required support code and suite documentation move under lab/. Historical results
remain labeled with their original source/profile and qualification limits.
Captured evidence and source-specific licenses keep their byte-preserving
authority; a lab manifest must resolve each such artifact to its exact preserved
location and digest, even when canonical evidence remains under evidence/.
Catalog metadata does not become a runnable benchmark by being moved.

Retired schedulers are infrastructure to replace. Their benchmark scenarios and
expected behavior are workload assets to retain. Port execution adapters to the
one durable controller; never restore an old controller to keep a benchmark alive.

## Corenius and existing Harp collections

The imported current tree contains 42 task directories with 588 tracked files and
18 readiness scenarios with 116 tracked files. These are current inventory
counts, not claims that all tasks presently run or have passed. Older reports
mention 41 tasks and must keep their historical meaning.

| Source collection | Contents to account for | Target |
| --- | --- | --- |
| Imported bench/tasks, b-l0 through b-l5 | 11 build/CUDA/proto/wrapper/dependency tasks | lab/benchmarks/coding/tasks/ |
| Imported bench/tasks, c01 through c05 | 5 C++ tasks | lab/benchmarks/coding/tasks/ |
| Imported bench/tasks, p01 through p06 | 6 Python tasks | lab/benchmarks/coding/tasks/ |
| Imported bench/tasks, r01 through r05 | 5 Rust tasks | lab/benchmarks/coding/tasks/ |
| Imported bench/tasks, t01 through t08 | 8 terminal tasks | lab/benchmarks/coding/tasks/ |
| Imported bench/tasks, m01-m03 and mh1-mh3 | 6 multi-ticket and heterogeneous tasks | lab/benchmarks/coding/tasks/ |
| Imported bench/tasks/real_terminal_bench_analyze_access_logs | Additional terminal benchmark task | lab/benchmarks/coding/tasks/ |
| Imported bench/readiness | All 18 scenarios, participants, workspaces and negative acceptance/landing cases | lab/benchmarks/readiness/ |
| Imported bench/harness and bench/tmuxh | Validators, task materialization, reporting and useful terminal observation behavior; replace retired agent launch adapters | lab/benchmarks/support/ |
| Imported bench/insula | Workload-specific capsule requests/layers; shared isolation implementation goes to insula/ | lab/benchmarks/profiles/ |
| Imported bench/inventories/public_agent_benchmarks | Coding/terminal/SWE/program and retrieval/professional catalogs, source lock and materialization scripts | lab/benchmarks/catalogs/ |
| Imported bench/RESULTS.md and results-archive | Measured pilot/fleet results, aggregation inputs and limitations | lab/benchmarks/results/ |
| Imported docs/efforts/multi-agent-bench | Prototype/verified task repos, Blade/CUDA projects, schema, decisions and grader evidence | lab/benchmarks/studies/multi-agent/ |
| Imported docs/efforts/controller-backed-bench and lead-worker-readiness | Benchmark plans, scenarios, scripts, environment snapshots and adjudication evidence | lab/benchmarks/studies/controller-readiness/ |
| Imported docs/efforts/sqlite-controller and team-control-plane evidence | Embedded episode/fixture repos, archived fixtures and benchmark evidence; distinguish core runtime unit tests | lab/benchmarks/studies/controller-history/ |
| Imported docs/efforts/engineering-profile-labs and adoption | Native CLI/protocol probes and learning examples; service-dependent experiments remain historical and excluded from active gates | lab/benchmarks/studies/provider-runtime/ |
| Other imported effort records and branch/worktree snapshots | Any additional benchmark payload found by source-to-destination inventory, including exemplars | Explicit lab destination per payload, never an unexamined retirement |
| Root benchmarks/codex-architecture and dynamic-workflow | Schemas, cases, expected contracts, manifests and retained result records | lab/benchmarks/architecture/ |

No PostgreSQL or Temporal runtime is adopted by retaining historical learning
material. Do not require their services to run the maintained workload gate.
Legacy service-dependent records remain visibly historical with explicit
capability status, rather than being presented as runnable under the new runtime.

## Emacs is a separate source acquisition

Corenius's product review identifies Emacs work in sibling projects. The two
referenced chats were read: `corenius-lite-durable` describes runtime and
qualification tooling; `corenius-agentic-review` describes a session inventory
and failure audit. Neither makes the Emacs payload part of the absorbed tree.
The imported product review points to Soma's gym and DevX's AI/LSP verifier;
read-only inspection confirmed both source checkouts.

| Source | Observed source identity | Required lab coverage |
| --- | --- | --- |
| soma | HEAD `0bfd646a1e2263b53f723796ab2945db325aacbf`, plus uncommitted work | lab/soma/ |
| devx project | HEAD `2724406183236616badef061138208eb20417903`, plus uncommitted work | lab/soma/ai_lsp/ |

The owner selected lab/soma/ as the project home. Preserve the complete gym,
bridge, lifecycle workers, mechanism probes and maintained documentation there;
extract shared isolation/service mechanisms into Insula and use durable for
agent execution. The AI/LSP verifier is a separately sourced companion under
lab/soma/ai_lsp/, not evidence that those sources were already part of Soma.

Soma has 276 tracked files under benchmarks/emacs-gym/ at this HEAD:

- checked Elisp/daemon bridge and its request/response contracts;
- Magit, Dired, Org, comint, Jupyter and LSP scenarios;
- scenario fixtures, grading scripts, traces and summaries.

Associated workers, probes, rootfs/profile scripts and contracts live outside
the benchmark directory and must be included when required by those scenarios.
Do not copy a scenario without the code and environment that make it executable.

The inspected Soma checkout had 57 status entries, including changed bridge
code, replaced trace files and untracked API/report-export work. This is not
57 independent benchmark tasks. Preserve committed, staged, unstaged and
untracked nonignored source identities before selecting the import snapshot.
Do not reset the source or overwrite old traces with the new latest run.
Debugger-context and Org HTML-export tickets are planned work until their actual
implementation and evidence are found; filenames alone do not prove delivery.

DevX's verifier covers actual Rust/Python/C++/CUDA language-server and inline-AI
contracts. Carry the verifier, adapters, independent expectations, fixtures and
required configuration into the lab. Separate offline checks from opt-in live
provider/authentication checks. The inspected checkout had four status entries.
Personal Doom loaders, credentials, host paths and private machine configuration
are not portable benchmark inputs and must not be copied into tracked examples.

No license file was identified by the top-level/source filename inventory for
these two checkouts. Their source-specific terms and ownership still need explicit
inspection during acquisition. Do not infer a license or introduce a repository
license. Their dirty snapshots and untracked files have not yet been captured
or imported into Harp.

## Executable completion check

The migration must produce a per-file source/disposition manifest for the full
benchmark inventory, including the selected dirty source snapshots. It must fail
for an unmapped task/scenario, missing required fixture/support dependency,
unexplained digest/mode change, missing grader/reference, or an unreviewed
benchmark retirement. Check both directions so orphaned destination files and
copy omissions are visible. Preserve independent source and oracle identities.

Exercise representative migrated suites, including the Emacs daemon bridge and
real LSP/Jupyter/process ownership where declared. Prove that unsolved task
fixtures fail, reference solutions pass, and missing capabilities report an
unqualified result. Historical pass counts, synthetic protocol traces and
rootfs wrappers do not establish a fresh live result.

Raw private session corpora referenced by the review chat are audit inputs, not
benchmark assets to publish. Preserve the sanitized failure report and source
references. If an incident becomes a regression benchmark, create a minimal
sanitized reproducer with its own oracle; do not import personal transcripts.

# Harp repository architecture

Status: implemented and verified
Owner: project owner
Branch: codex/repo-architecture
Baseline: 5342c2e3b08e8671e3ef46aca83bcbbc79b7f5a3

## Outcome

Make the selected durable runtime first-party Harp code with one execution control
plane, one SQLite operational owner and internal Git-managed intent, collaboration
and Git workflow modules. Remove the imported product directory and
harp_durable package after migrating callers.
Expose top-level Insula and SSH runner packages and put executable workloads in
lab/. Preserve math, novel Crouzeix proofs, SICP, education and PDF publication.

The [target architecture](../../docs/workstream/repo-architecture/design.md) owns
the tree, dependency graph, migration mapping, trust boundaries and workload
contracts. It supersedes earlier recommendations to retain the imported directory.
The integration candidate implements this ownership graph. Completed checks and
exact source identities are recorded in the repository qualification below.

## Scope and authority

The owner prioritized codebase structure and executable checks ahead of TLA+.
They requested a single durable control plane, top-level Insula/runner and a
workload home, with private host configuration kept out of commits and reports.
Use lab/ as the chosen workload name and correct the education name to SICP.
No Temporal, PostgreSQL, second controller, private source-checkout dependency,
registry publication, persistent remote service or Git history rewrite is part
of this design. Preserve absorbed ancestry and source-specific licensing.

The owner selected architecture and follow-up tickets for the new mixed Bazel,
Triton compilation and transformer-training workloads. Their runnable
implementations are deferred. Existing-code migration is complete and verified.
Tickets 08–10 and 14 retain the agreed follow-up work. Architecture work does not
itself grant new publication, service or resource-allocation authority.

## First useful operation

Run the public Harp CLI on a small deterministic coding task through migrated
durable packages, independently verify its exact candidate and exercise the
existing explicitly approved local landing boundary in a disposable repository.
Use real Git, SQLite state and a native Insula profile. Prove both audit
reproducers are refused safely before calling the integrated result trustworthy.

The package relocation is required to establish the sole code owner. Explicit
state and Git boundaries prevent duplicated transition/approval logic. Native
qualification is required because the portable suite can pass with Linux cases
skipped. No new workload implementation, GPU fleet, model training or TLA+ tool
is a prerequisite for that first operation.

## Delivery slices

- [01: Unify durable runtime, SQLite state and Git workflow ownership](issues/01-unify-durable-control-plane.md)
- [02: Protect owner acceptance inputs and admitted change scope](issues/02-protect-acceptance-oracle.md)
- [03: Qualify Git checkout before physical landing](issues/03-qualify-physical-checkout.md)
- [04: Make Insula a first-party isolation package](issues/04-extract-insula.md)
- [05: Run admitted workloads over SSH under Insula](issues/05-ssh-runner.md)
- [06: Put retained executable mathematics and SICP under lab](issues/06-relocate-math-and-sicp.md)
- [07: Relocate benchmarks and finish imported-tree removal](issues/07-relocate-benchmarks-and-provenance.md)
- [08: Build a compact C++/Rust/Python Bazel workload](issues/08-mixed-bazel-workload.md)
- [09: Add a bounded Triton compilation workload](issues/09-triton-compilation-workload.md)
- [10: Add a checkpointable small transformer training job](issues/10-transformer-training-workload.md)
- [11: Qualify the migrated architecture end to end](issues/11-qualify-migrated-architecture.md)
- [12: Preserve the complete Emacs benchmark work under lab](issues/12-preserve-emacs-benchmarks.md)
- [13: Integrate the supplied Insula service supervisor](issues/13-integrate-insula-supervisor.md)

Runtime consolidation and the two bounded trust repairs can begin independently;
compose them before final qualification. Extract Insula before adding remote
transport. Migrate executable math/SICP and existing benchmarks with their
consumers and provenance checks. Final qualification covers the complete
existing-code architecture. The three new workload tickets are independent
architecture follow-ups, not blockers for removing the imported wrapper.

## Completion evidence

Every ticket names its behavioral acceptance and required evidence. Inspect
`mise run verify -- --plan --base <baseline>` for the whole candidate, update
selectors alongside moved code, run selected checks and affected consumers, and
record the source identity and native qualification limits. A docs-only gate
qualifies this design record, not the target runtime or remote runner.

## Architecture record verification

Passed `mise run verify -- --base 5342c2e3b08e8671e3ef46aca83bcbbc79b7f5a3`
for the fourteen changed/new Markdown files. The selected checks were diff
whitespace and changed local links. An additional inspection validated eleven
ticket structures, an acyclic blocker graph and absence of the private host token
from the new design/tracker files. No runtime, Lean, GPU or remote-runner
qualification was performed for this architecture-only change.

## Benchmark scope clarification

The owner explicitly requires preserving all existing benchmark work, including
Emacs. The [coverage inventory](../../docs/workstream/repo-architecture/benchmark-inventory.md)
records 42 current task directories, 18 readiness scenarios, embedded studies,
fixtures, catalogs, tmux work and result records. Read-only inspection also found
Soma's Emacs/Doom gym and DevX's AI/LSP workload in separate dirty source
checkouts. Ticket 12 acquires those workloads into lab/soma/ and blocks final
migration acceptance. New Bazel/Triton/training implementations remain deferred.
The Emacs sources are now preserved under lab/soma/ with byte-level source maps; the historical inspection evidence below predates that import.

Benchmark clarification checks passed: twelve acyclic tickets, final acceptance
blocked on Emacs acquisition, current 42-task/18-scenario counts checked against
tracked source, changed local links valid, and no private host token in changed
records. `mise run verify -- --base 31e3a076154ef7ec18f176f2628b0c8050195f68`
passed its selected documentation checks. These checks qualify the migration
coverage records, not benchmark execution or import completeness after relocation.

## Existing service supervisor input

The owner supplied an implemented Rust service supervisor. Its [inspection and
integration record](../../docs/workstream/repo-architecture/supervisor-integration.md)
places it under insula/supervisor/ and preserves its reported verification limits.
Reuse it rather than designing another service supervisor. Ticket 13 implements
and qualifies the missing real ExecutionOwner adapter through the authoritative
Insula process/namespace boundary. It blocks final integration and the Emacs
service scenarios. Service state does not become another workflow/approval
journal, and terminal recovery cannot replay expensive workload effects.
The archive is now imported with a real Insula execution owner. The owner exposes only its qualified capabilities; stronger service capabilities refuse admission. No persistent daemon is installed.

Supervisor-record checks passed: archive digest and standalone-document matches,
thirteen-ticket acyclic dependency graph, required supervisor blockers, changed
local links and host-token scan. `mise run verify -- --base ae894136` passed its
selected documentation checks. No supplied Rust tests or real-owner integration
tests were run in this inspection.

## Git workflow belongs inside durable

The owner folded the proposed top-level Git-workflow package into durable. The
framework includes durable/intent/ for the existing Git-managed spec/issue/ticket
contract, durable/collaboration/ for bounded multi-agent policy, and durable/git/
for source/candidate/verification/landing rules. Authored intent remains in Git;
SQLite records execution state and immutable input references. Keep .scratch/
and the existing tracker format. Ticket 01 includes these responsibilities in
the single-runtime migration; this does not add a separate tracker or scheduler.


This clarification passed `mise run verify -- --base 967df339`, selecting diff
whitespace and changed-document local-link checks. Runtime source remains
unmoved; these checks validate the revised design and ticket contracts only.

## Provider, intent authoring and test placement clarification

Codex is the migration's first provider. Qualify one, two and four Codex workers;
mixed-provider qualification is no longer required for this delivery. Existing
Trae source/evidence remains preserved with further integration deferred. Pi may
be considered later and is not a current implementation prerequisite.

The framework-side intent module supports the existing Git-managed records.
Intent materialization into detailed specs and tickets follows the Matt Pocock
style and Harp authoring skills. Colocate new/moved tests with source and preserve
behavioral coverage and import provenance. These decisions supersede the earlier
multi-provider acceptance and the separate tests/ folders shown in the expanded
layout sketch. The source migration is now implemented and verified.

## Completion evidence

Existing-code tickets 01–07 and 11–13 are done. The [repository qualification](../../evidence/provenance/repository-qualification.json)
records body commit, complete-candidate check results, native source identity and
verification limits. Actual Codex runs qualified one, two and four workers; all
eight exact approved qualification candidates landed with matching Git trees and
authority journals. The retained Soma workload completed 139 cases through SSH
and Insula with verified artifact retrieval. New workload implementations and
richer service-owner capabilities remain follow-ups, not claimed deliveries.

## Layout amendments after initial qualification

The owner requested merging maintained knowledge into lab. Ticket
[16](issues/16-merge-knowledge-into-lab.md) places math and SICP prose beside their
workloads, benchmark notes under lab/benchmarks/, shared engineering studies under
lab/agentic_engineering/, and the generated corpus under lab/generated/. Captured
upstream evidence stays in evidence/ and document identities remain unchanged.

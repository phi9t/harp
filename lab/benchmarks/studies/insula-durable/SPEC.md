# Durable execution specification

## Sources and approval
User-supplied insula_durable_reference.zip, DESIGN.md, DBOS.md and VALIDATION.md; user request approves running and implementing that design. Existing Corenius Insula uses tools/insula/capsule.py and has no per-attempt cgroup service.

## Requirements
1. Expose Activity, Context, Store and Runtime under the controller-owned execution-local Python component; deterministic operation identity, spec validation and replay tail checks.
2. Preserve immutable submission/version/contract binding, timers, keyed signals, durable retry budgets and capacity accounting.
3. Preserve persisted claim and release intent, inherited lock ownership, durable guardian receipts and active-attempt acceptance fence.
4. Default recovery to manual. Started interrupted manual activities remain UNCERTAIN/BLOCKED across cancellation or another failure. Cleanup acknowledgment is separate from effect resolution and neither launches work.
5. Honor owned host TMPDIR and private /run/corenius-tmp inside Insula, including subprocess environments; no sys.path mutation or ambient imports.
6. Provide an Insula launcher using the existing capsule builder, immutable code projection, stable persistent-state mount and resolved capsule binding; no extra resident service.
7. Run the original fault cases and integration tests, document disposable SQLite override and trusted process-group backend assumptions.

## Exclusions
DBOS integration, hostile process containment by the trusted backend, automatic backups/migrations/GC, remote scheduling, Rust reimplementation and publication.

## Test seams
Existing reference public Activity/Context/Store/Runtime APIs, CLI subprocesses, real SQLite/filesystem and Linux processes; capsule run_argv planning and live bwrap launch. Cgroup fault tests require real delegated kernel support and must not be reported as passing when unavailable.

## Clarified backend scope
User requested trying both approaches. Implement selectable trusted process groups and fail-closed delegated cgroup v2 cleanup. Real cgroup tests run only when an explicit writable delegated root is supplied; report unavailable otherwise. Backend selection is immutable execution binding. Payload cgroup membership precedes executable release; cgroup control paths are not exposed inside its Insula. Existing Rust controller remains ticket/Git authority.

## Resolved integration details
Activity payloads run in per-attempt bwrap namespaces under the existing admitted capsule. The foreground controller and guardian stay outside those namespaces. Only payload/ is writable inside; controller records, receipts and cgroup membership metadata are host-owned. The fork child retains the inherited ownership descriptor until cgroup membership has been durably published. Missing membership blocks recovery even if a subgroup looks empty. All files excluded by the existing tree digest are forbidden in executable code closure. Canonical cgroup controls are masked even with GPU sysfs projections. JSON control files must be bounded regular files, preventing FIFO waits.

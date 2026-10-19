# 14: Qualify four workers and complete supersession

Status: in-progress
Owner: Codex on `codex/harp-durable-runtime`
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Qualify the unified Harp runtime at a configurable capacity up to four, with a real mixed-provider four-worker run and no supported competing controller path.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] Demonstrate four actually overlapping real coding workers using both Codex and Trae, with separate admitted identities and isolated workspaces. Four queued tasks or a configured limit alone is insufficient.
- [ ] Exercise limits one, two and four, queued admission, cancellation/failure, retained cleanup blocks and safe recovery without exceeding actual admitted capacity.
- [ ] Independently verify candidates, serialize their owner-approved landings and handle stale bases through new checked candidates and approvals, preserving actual primary state.
- [ ] Audit supported entry points together: every retained execution path reaches the selected authority; retired controllers have no hidden launcher, scheduler or recovery fallback.
- [ ] Normal builds/gates and the qualified path require neither Temporal nor PostgreSQL. No other checkout, source archive or private captured session is a dependency.
- [ ] Run required changed-surface and relevant consumer checks for the complete integration candidate. Preserve affected math/proof/SICP/educational/PDF contracts without imposing unrelated cache hydration.
- [ ] Retain exact live/fault-test source, provider/backend/environment and concurrency/landing evidence, including limitations; mark the overall implementation complete only after all approved milestone behavior passes.

## Blocked by

- [10: Qualify two concurrent mixed-provider workers](10-qualify-two-workers.md)
- [11: Migrate or retire Harp’s direct provider workflows](11-migrate-provider-workflows.md)
- [12: Migrate or retire Harp’s older task-graph execution](12-migrate-task-graphs.md)
- [13: Retire the superseded Corenius controller paths](13-retire-corenius-controllers.md)

## Scope

This is the final integration and acceptance gate, not an expansion to fleet scheduling, GPU isolation, automatic conversation resumption or more than four workers. Live runs and setup require explicit authorization.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.

The owner explicitly authorized completion of the four-worker run on actual
Harp tasks, full review, required checks and local landing. No push. Frozen
tasks and independent checks live in `../qualification/`: gate dispatch coverage
(Codex), retained Lean workflow (Trae), runtime architecture (Codex), repository
structure review (Trae). Documentation checks establish structure/link integrity;
semantic review remains separate. Each candidate will have a distinct clean
Harp destination, with serialized reviewed integration into the delivery branch.

### Actual Harp four-worker run

Frozen source: `7db4f06352d32d8517b3ac7e1cd026151ce3d9f0`.
Task-owned run: `/data05/harp-runtime-integration-philip-yang/runs/live-four-01`
on devx. The native Rust bridge was built from identical Rust bytes at
`207b8ce9`, with SHA-256
`60609d812b307a5b9f12da6a5bca73875b5f9d1361afa5b5be140ca8c9700424`.
Python 3.11.2, SQLite 3.51.3, Codex 0.162.1 and Trae 0.210.1 were retained
with exact executable and code hashes in `environment.json` and
`source-hashes.json`. No Temporal/PostgreSQL or installed service was used.

Kernel observations in `overlap.json` establish four simultaneous non-zombie
provider processes, two Codex and two Trae, with distinct admitted cgroups and
PID/start-tick/membership identities confirmed in two passes. The common
observation interval is recorded there. Sampled populated attempt groups never
exceeded four. Each coding result and independent verification completed,
retained one accepted attempt and proved empty cgroup cleanup. A subsequent
completed replay preserved exact results and attempt identities; all source
hashes and untouched destinations were rechecked.

| Task | Provider | Exact independently verified candidate |
| --- | --- | --- |
| Controller verification dispatch | Codex | `386e6f4effc6d99f896cf2d26bd4d2caa7db3155` |
| Lean workflow guide | Trae | `a2e4e82264abd43d77ca381ca010e9e3fb1113ee` |
| Runtime architecture | Codex | `69c17f3aaee312bb48b65b35eb83995c64ef2eaa` |
| Repository structure review | Trae | `d6d7cf73a5acf376ad56a17819f880487aa5975b` |

Each candidate changes only its assigned paths (five files total). Capture
bundle hashes were rechecked before exporting prerequisite-bound delta bundles;
local delta/diff hashes, Git commit/tree identities and exact path ownership
were independently checked. Independent semantic/source review passed all four
candidates, the selector's 16 public CLI-plan cases and all three documentation
link checks. The four original commits are preserved as parents of integrated
candidate `71181925b7ae34041b177a095432203a0a97dc32`.

Whole-candidate review also caught a real Git squash-default partial-mutation
bug. `c242e965` adds explicit `--no-squash`; a real Git regression reproduced
the failure before repair and passed afterward. All 16 portable landing cases
passed, and independent review confirmed the fix. The five imported-controller
retirement tests passed after composition, including the tmux early-refusal
repair. The operator documentation now states the durable Python 3.11 prerequisite.

Exact human approval was requested for the four separate task-owned destination
repositories; no physical landing is claimed before that approval. The owner
also explicitly authorized final local integration on `main` and pushing to the
existing remote. Remote `master` will be preserved.

The first fresh all-module Linux run (`final-suite-01`) is retained as a failed
test-apparatus attempt: its offline rootfs lacked DNS for explicit network-profile
tests, and umask 077 invalidated fixtures intended to create public permissions.
The corrected `final-suite-02` uses the same frozen code, the already-qualified
network rootfs, and conventional fixture creation modes inside a private 0700
run directory. Network authority remains explicit per execution/check profile.
Its outcome and the final complete-candidate gate must be recorded before done.

Corrected native result: `final-suite-02/result.json` reports all 102 tests
passed, zero failures/errors/skips, 475.534 seconds, frozen source unchanged.
It includes the real Git squash-default regression, capacities one/two/four,
queued reuse, mixed-adapter cancellation/recovery, retained uncertainty,
independent verifier isolation and exact approval/landing boundaries. The host
SQLite was 3.51.3 and Python 3.11.2. This supplies all Linux-only cases skipped
by the host gate; the failed apparatus run was not counted as passing.

The real provider observer retained 6,167 samples; 775 samples confirmed all
four active provider identities over a span of 40.412 seconds. The initial
common two-pass kernel observation also records all four exact PID/start-tick
identities. These are actual coding workers, not configured or queued capacity.

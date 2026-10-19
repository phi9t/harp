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

# 04 relocate-parked-control-plane: Relocate the parked control plane

Type: task
Status: open
Owner:
Blocked by:
Claimed:

Triage: ready-for-agent

**Parent:** Corenius semantic repository layout specification (same effort).

**What to build:** Developers can find and explicitly verify the retained PostgreSQL/Temporal experiment without mistaking it for an active product or reviving its roadmap.

**Source baseline:** 48d71588fff00ac70fa4a29e066a47a6b05f2ebd; execute against the integration revision containing completed blockers.

## Acceptance

- [ ] The retained implementation moves into the experimental boundary with its tests, SQL/runtime support and documentation relationships intact.
- [ ] Current navigation clearly states parked, preserved and unfinished status; no frozen unaccepted source is imported and no existing evidence is deleted.
- [ ] Preserve its package/binary identities, current service-backed runner semantics and failure behavior.
- [ ] The engineering repository gate continues invoking the PostgreSQL/Temporal check; relocation does not make it optional.
- [ ] The direct experimental verification entry point works after relocation and relative-root/asset assumptions are corrected.
- [ ] Keep the experiment outside default Cargo product selection while retaining its independent workspace until consolidation.
- [ ] Run service-backed verification when prerequisites are available; record missing Docker, service images or dependencies as unverified, never passing, evidence.

## Exclusions

No roadmap revival, production readiness claim, default gate removal, backend redesign or import from frozen external worktrees.

## Verification evidence

Existing PostgreSQL/Temporal runner and registered repository-gate entry point; record actual command, services used, exit status and cleanup evidence, or exact prerequisite blocker.

## Execution constraints

Linux only. Use an isolated working tree with one writer. Preserve existing work and historical evidence. Implementation requires the subsequent execution phase; ticket publication alone does not authorize commits, pushes, pull requests or merges. Review on repository Standards and parent Spec axes.

## Comments
- 2026-10-07T08:23:48Z codex: Publication normalization preserves ready-for-agent triage, acceptance and blocker graph; implementation remains unstarted.

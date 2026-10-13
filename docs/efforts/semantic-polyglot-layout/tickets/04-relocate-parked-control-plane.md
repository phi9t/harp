# 04 relocate-parked-control-plane: Relocate the parked control plane

Type: task
Status: resolved
Owner: TRAE
Blocked by:
Claimed:

Triage: ready-for-agent

**Parent:** Corenius semantic repository layout specification (same effort).

**What to build:** Developers can find and explicitly verify the retained PostgreSQL/Temporal experiment without mistaking it for an active product or reviving its roadmap.

**Source baseline:** 48d71588fff00ac70fa4a29e066a47a6b05f2ebd; execute against the integration revision containing completed blockers.

## Acceptance

- [x] The retained implementation moves into the experimental boundary with its tests, SQL/runtime support and documentation relationships intact.
- [x] Current navigation clearly states parked, preserved and unfinished status; no frozen unaccepted source is imported and no existing evidence is deleted.
- [x] Preserve its package/binary identities, current service-backed runner semantics and failure behavior.
- [x] The engineering repository gate continues invoking the PostgreSQL/Temporal check; relocation does not make it optional.
- [x] The direct experimental verification entry point works after relocation and relative-root/asset assumptions are corrected.
- [x] Keep the experiment outside default Cargo product selection while retaining its independent workspace until consolidation.
- [x] Run service-backed verification when prerequisites are available; record missing Docker, service images or dependencies as unverified, never passing, evidence.

## Exclusions

No roadmap revival, production readiness claim, default gate removal, backend redesign or import from frozen external worktrees.

## Verification evidence

Existing PostgreSQL/Temporal runner and registered repository-gate entry point; record actual command, services used, exit status and cleanup evidence, or exact prerequisite blocker.

## Execution constraints

Linux only. Use an isolated working tree with one writer. Preserve existing work and historical evidence. Implementation requires the subsequent execution phase; ticket publication alone does not authorize commits, pushes, pull requests or merges. Review on repository Standards and parent Spec axes.

## Comments
- 2026-10-07T08:23:48Z codex: Publication normalization preserves ready-for-agent triage, acceptance and blocker graph; implementation remains unstarted.
- 2026-10-13T00:10:32Z TRAE: moved the parked implementation to `experiments/team_control_plane/`, retained its opt-in repository gate registration and kept it out of default Cargo selection. Verified `cargo test --locked --offline --manifest-path experiments/team_control_plane/Cargo.toml --no-run`, `bash -n experiments/team_control_plane/tests/run-temporal.sh` and `CHECK_TEAM_CONTROL_PLANE=1 sh tools/agents/checks.d/70-team-control-plane.sh`, all under `python3 tools/temp_root.py --`, exit 0.

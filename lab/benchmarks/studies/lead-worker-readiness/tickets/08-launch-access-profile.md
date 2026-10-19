# 08 launch-access-profile: A launch declares what the worker may reach

Type: feature
Status: resolved
Owner: worker-08
Blocked by: 01
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Stories 31, 32, 33, 36 and decision items 5 and 6. Replace the fixed rules in the worker's standing instructions and forbidden-command list with a declared access profile recorded at launch: network on by default, off when the launch says so; the longest a single worker command may run; paths outside the repository the worker may add entries under; and whether container builds and writes to shared caches are allowed (off by default). The standing instructions shown to the worker are generated from the profile, and the supervisor's command findings follow it (for example a download is a finding only when the network is declared off).

## Declared paths
- `tools/agents/worker.py`, `tools/agents/test_worker.py`
- `bench/harness/` only as needed so a scripted participant can be given the network (today it always runs offline)
- `bench/readiness/` (scenarios)

## Acceptance
- [x] A default launch records network on, and the worker's instructions no longer say there is no network; a scripted participant performs a loopback fetch and no finding is raised.
- [x] A launch with the network declared off records it, tells the worker, and raises a finding for a download command; inside a capsule the fetch fails.
- [x] The single-command time limit is a launch option recorded in the run; a scripted command longer than the old two-minute limit completes when the launch allows it.
- [x] Declared external add-paths appear in the worker's instructions and the launch record; container builds and shared-cache writes are recorded as not allowed unless declared.
- [x] Runs recorded before this change still supervise without error.
- [x] All scenarios run in the lead-driven mode and match their expectation.

## Comments
- 2026-10-05T09:06:02Z worker-08: claimed until 2026-10-07T09:06:02Z
- 2026-10-05T10:04:13Z worker-08: resolved

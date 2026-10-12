# Ticket 18: operating documentation and shim retirement

## Built
ADR0002 records consolidation, alternatives, exact verification identity, tracker records and cleanup limits. The CLI guide and retirement checklist accompany the final implementation.

## Remaining shim callers
- tools/agents/lead_launch.py: land module authority/effort helpers.
- tools/agents/tracker.py: effort discovery and accepted ancestry helpers.
- tools/agents/effort.py: effort discovery, worktree and landing helpers.
- tools/agents/worker.py: acceptance lookup through land.
- tools/agents/acceptance.py: effort records and candidate publication helpers.
- bench/harness/lead_driven.py: legacy landing CLI.
- docs/efforts/team-control-plane/research/probes/jj_workspace_evaluation.py: executable research probe invoking both shims.
- tools/agents/checks.d/10-wt.sh and20-land.sh: compatibility suites.
- tools/agents/test_wt.py andtest_land.py: compatibility entry points and exported Python APIs.
- tools/test_temp_root.py: loads the landing shim with `runpy` to check the temporary-worktree export.

Historical preimages and source snapshots under the team-control-plane effort's
evidence also refer to the shims. Preserve these recorded sources with their
original identity when migrating the executable probe.

## Retirement criteria
Keep both shims for one release. Migrate each caller to explicit Collab modules/commands; preserve expected exits and evidence semantics. Run caller suites and qualify the complete disposable ticket lifecycle before removal. Update operator documentation and policy in a separately versioned policy change if synchronized pointers need replacement. Never remove a shim while any listed caller remains.

## Claims
The public CLI is tested in disposable real Git repositories. Integration preserves existing source workers.

## Outside declared paths
Tracker `resolve --mark-reviewed` is the sole ticket mutation seam needed for Corenius closure.

## For later tickets
none

## Not checked
Full repository and independent review acceptance are recorded separately after the final source freeze.

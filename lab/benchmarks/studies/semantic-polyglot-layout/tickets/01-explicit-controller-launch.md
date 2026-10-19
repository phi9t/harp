# 01 explicit-controller-launch: Make controller launch inputs explicit

Type: task
Status: resolved
Owner: TRAE
Blocked by:
Claimed:

Triage: ready-for-agent

**Parent:** Corenius semantic repository layout specification (same effort).

**What to build:** Operators can launch the existing controller system from an explicit, complete binary directory without invoking a build, while local developers retain the documented build-and-launch convenience.

**Source baseline:** 48d71588fff00ac70fa4a29e066a47a6b05f2ebd; execute against the integration revision containing completed blockers.

## Acceptance

- [x] Launch-only mode accepts an explicit binary directory and uses the existing controller, supervisor and gate executable identities from that directory.
- [x] Missing or non-executable required binaries fail clearly before creating sessions or starting services; no mixing with another target directory or implicit Cargo invocation occurs.
- [x] The existing start convenience command preserves target-directory selection, builds explicitly, and delegates to the same launch implementation.
- [x] Pinned executable evidence, relay discovery, argument handling, process ownership and existing operational behavior are preserved.
- [x] Existing launcher and relay suites pass; new launcher behavior has meaningful failing-then-passing evidence through the existing public CLI seam, without real-model calls.
- [x] Current launcher documentation covers both modes and retains live-baseline qualification caveats.

## Exclusions

No source relocation, runtime redesign, new provider qualification or companion binary renaming.

## Verification evidence

Existing launcher/relay suites; focused CLI proof for launch-only, missing companions and convenience behavior; record commands, exit statuses and executable identity handling.

## Execution constraints

Linux only. Use an isolated working tree with one writer. Preserve existing work and historical evidence. Implementation requires the subsequent execution phase; ticket publication alone does not authorize commits, pushes, pull requests or merges. Review on repository Standards and parent Spec axes.

## Comments
- 2026-10-07T08:23:48Z codex: Publication normalization preserves ready-for-agent triage, acceptance and blocker graph; implementation remains unstarted.
- 2026-10-13T00:10:32Z TRAE: implemented explicit `--binary-dir` launch-only mode with required adjacent `corenius-sqlite-controller`, `corenius-attempt-supervisor` and `corenius-attempt-gate` executables. Verified with `python3 tools/temp_root.py -- sh tools/agents/checks.d/75-controller-python.sh` and `python3 tools/temp_root.py -- sh tools/agents/checks.d/70-sqlite-controller.sh`, both exit 0.

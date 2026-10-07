# 01 explicit-controller-launch: Make controller launch inputs explicit

Type: task
Status: open
Owner:
Blocked by:
Claimed:

Triage: ready-for-agent

**Parent:** Corenius semantic repository layout specification (same effort).

**What to build:** Operators can launch the existing controller system from an explicit, complete binary directory without invoking a build, while local developers retain the documented build-and-launch convenience.

**Source baseline:** 48d71588fff00ac70fa4a29e066a47a6b05f2ebd; execute against the integration revision containing completed blockers.

## Acceptance

- [ ] Launch-only mode accepts an explicit binary directory and uses the existing controller, supervisor and gate executable identities from that directory.
- [ ] Missing or non-executable required binaries fail clearly before creating sessions or starting services; no mixing with another target directory or implicit Cargo invocation occurs.
- [ ] The existing start convenience command preserves target-directory selection, builds explicitly, and delegates to the same launch implementation.
- [ ] Pinned executable evidence, relay discovery, argument handling, process ownership and existing operational behavior are preserved.
- [ ] Existing launcher and relay suites pass; new launcher behavior has meaningful failing-then-passing evidence through the existing public CLI seam, without real-model calls.
- [ ] Current launcher documentation covers both modes and retains live-baseline qualification caveats.

## Exclusions

No source relocation, runtime redesign, new provider qualification or companion binary renaming.

## Verification evidence

Existing launcher/relay suites; focused CLI proof for launch-only, missing companions and convenience behavior; record commands, exit statuses and executable identity handling.

## Execution constraints

Linux only. Use an isolated working tree with one writer. Preserve existing work and historical evidence. Implementation requires the subsequent execution phase; ticket publication alone does not authorize commits, pushes, pull requests or merges. Review on repository Standards and parent Spec axes.

## Comments
- 2026-10-07T08:23:48Z codex: Publication normalization preserves ready-for-agent triage, acceptance and blocker graph; implementation remains unstarted.

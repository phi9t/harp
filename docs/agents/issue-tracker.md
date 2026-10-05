<!-- Issue Tracker v2026.10.05.1. Canonical source: corenius/docs/agents/issue-tracker.md. -->
# Issue tracker: Local Markdown

Issues and specs live in tracked `docs/efforts/<effort>/`.
Working detail lives in gitignored `.scratch/<effort>/`.

## Conventions

- Intent and scope: `docs/efforts/<effort>/EFFORT.md`.
- Specification: `docs/efforts/<effort>/SPEC.md`.
- Tickets: `docs/efforts/<effort>/tickets/NN-slug.md`, numbered from `01`.
- Use `tools/agents/tracker.py` to create, claim, list, resolve and lint tickets.
- Ticket lifecycle uses `Status: open`, `claimed`, or `resolved`.
- Record triage separately as `Triage: <role>` in the ticket body,
  using `triage-labels.md`.
- Preserve the tracker’s header order and required ticket sections.
- Append timestamped comments under `## Comments` using the tracker format.

## Publish and fetch

When a skill says “publish to the issue tracker”, create or update the
appropriate durable spec or ticket under `docs/efforts/<effort>/`.

When a skill says “fetch the relevant ticket”, read its referenced path.
A bare ticket number requires an effort name.

## Wayfinding operations

- Map: `docs/efforts/<effort>/map.md`, containing Notes,
  Decisions-so-far and Fog.
- Child: one tracker-compatible ticket per question under `tickets/`.
  Use `Type: task` for research, prototype or grilling work and record
  its activity in the body.
- Blocking: use `Blocked by: NN, NN`. All blockers must be resolved.
- Frontier: open, unblocked, unclaimed tickets, ordered by number.
- Claim: use `tracker.py claim` before starting work.
- Resolve: record the answer and acceptance evidence, then use
  `tracker.py resolve`. Append an answer summary and ticket link
  to the map’s Decisions-so-far.

Follow `agentic-engineering.md` for effort lifecycle and authority.

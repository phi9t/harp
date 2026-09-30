# 03 tracker: tickets and claims (F7)

Type: feature
Status: open
Owner:
Blocked by:
Claimed:

## Description
Build `tools/agents/tracker.py` for the tracked ticket files described in `milestone-b/contracts/ticket-format.md` (read that contract first; it is the format of the files in this directory).
`.scratch` is gone from the design: tickets live under `docs/efforts/<slug>/tickets/` so they survive a fresh clone.

## Interface
`tracker.py new EFFORT SLUG --title T [--type feature|bug|task] [--blocked-by NN,NN]`, `claim EFFORT NN --owner NAME [--ttl-hours H]`, `resolve EFFORT NN --owner NAME [--force]`, `list EFFORT [--json]`, `lint [EFFORT...]` (all efforts when none given), each with `--repo R`.
`claim` and `resolve` edit only the header and append to `## Comments`. `lint` exits 0 when clean and 1 with one line per problem (file, line, message).

## Acceptance
- [ ] Tests for each transition: new, claim, claim refused when claimed or blocked, expired claim taken over, resolve by owner, resolve refused for another owner or unchecked acceptance, resolve --force logged.
- [ ] Tests for each lint error: missing or reordered header line, bad Status or Type, claimed without Owner or Claimed, unknown or self or cyclic blocker, empty Acceptance, malformed comment line, duplicate id.
- [ ] list prints the frontier ordering from the contract and a --json form.
- [ ] Running tracker.py lint on this repository's real ticket files (docs/efforts/agentic-engineering/tickets) exits 0; a test copies them to a temp dir and checks that.
- [ ] Timestamps are injectable for tests (an environment variable or argument), so expiry is tested without sleeping.
- [ ] tools/agents/checks.d/30-tracker.sh runs the suite and lints the repo's real tickets.

## Comments

# 04 msg: messages and handoffs (F8)

Type: feature
Status: resolved
Owner: lead
Blocked by:
Claimed:

## Description
Build `tools/agents/msg.py`. A long-running agent does not re-read an updated handoff file; policy and instruction changes need delivery at a defined point. The lead checks inboxes at each turn start.

## Interface
Append-only files `<root>/.scratch/<effort>/messages/<to>/<ts>-<from>.md` (root = the main worktree; `--root` overrides for tests). Fixed header lines in this order, then a blank line, then the body:
`From:`, `To:`, `Type: info|instruction|question|handoff`, `Re:` (a message file name or empty), `Needs-ack: yes|no`.
`msg.py send --effort E --from A --to B --type T [--re FILE] [--needs-ack] [--body TEXT | --body-file F]`, `inbox --effort E --to B [--json]` (lists unacknowledged messages, oldest first, flagging NEEDS-ACK), `ack --effort E --to B FILE --by NAME` (writes a sidecar `<file>.ack`; the message file is never edited), `lint --effort E` (header check), `show FILE`.
Timestamps are injectable for tests; file names are unique even when two messages are sent in the same second.

## Acceptance
- [x] Test ordering: inbox lists oldest first, including for same-second sends.
- [x] Test ack: an acknowledged message leaves the inbox; the message file is byte-identical before and after ack.
- [x] Test Needs-ack: a message with Needs-ack: yes keeps being reported (flagged) until acknowledged, including across repeated inbox calls.
- [x] Test header lint: missing header, wrong order, bad Type, bad Needs-ack, unknown extra header are each reported.
- [x] Test that send refuses unknown recipients' path traversal (a --to or --effort containing / or ..).
- [x] tools/agents/checks.d/40-msg.sh runs the suite and exits non-zero on failure.

## Comments
- 2026-09-30T08:24:30Z lead: claimed until 2026-10-01T08:24:30Z
- 2026-09-30T08:24:38Z lead: resolved

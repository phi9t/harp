# Contract: ticket file format (F7)

Path: `docs/efforts/<slug>/tickets/NN-slug.md` (tracked). `NN` is a two-digit id (`01`..`99`); ids are unique within an effort.

```
# NN slug: one-line title

Type: feature | bug | task
Status: open | claimed | resolved
Owner: <name or empty>
Blocked by: <comma separated NN ids, or empty>
Claimed: <UTC time YYYY-MM-DDTHH:MM:SSZ> expires <UTC time>    (empty unless Status is claimed)

## Acceptance
- [ ] criterion
- [x] finished criterion

## Comments
- <UTC time> <who>: text
```

Rules the tool enforces:
- The header lines appear in exactly this order, directly after the title line and one blank line. No other header lines.
- `Status: claimed` requires `Owner` and `Claimed`; `open` and `resolved` leave `Claimed` empty (resolved keeps `Owner`).
- `Blocked by` ids must exist in the same directory and must not include the ticket itself or form a cycle.
- `claim` refuses when the ticket is claimed and the claim has not expired, or when any blocker is not `resolved`. An expired claim can be taken over.
- `resolve` requires `Status: claimed` by the same `Owner`, and every `## Acceptance` item checked (`--force` overrides and logs it in `## Comments`).
- Free-form sections (for example `## Description`, `## Interface`) may sit between the header and `## Acceptance`; the tool ignores their content.
- `## Acceptance` has at least one item; `## Comments` is append-only (the tool only appends lines).
- `list` prints the frontier: open, unblocked tickets first, then claimed, then blocked, then resolved.

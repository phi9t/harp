# Contract: ticket file format (F7)

Path: `docs/efforts/<slug>/tickets/NN-slug.md` (tracked). `NN` is a two-digit id (`01`..`99`); ids are unique within an effort.

```
# NN slug: one-line title

Type: feature | bug | task
Status: open | claimed | resolved
Owner: <name or empty>
Blocked by: <comma separated NN ids, or empty>
Claimed: <UTC time YYYY-MM-DDTHH:MM:SSZ> expires <UTC time>    (empty unless Status is claimed)

Triage: needs-triage | needs-info | ready-for-agent | ready-for-human | wontfix

## Acceptance
- [ ] criterion
- [x] finished criterion

## For later tickets
- to NN: finding text

## Comments
- <UTC time> <who>: text
```

Rules the tool enforces:
- The header lines appear in exactly this order, directly after the title line and one blank line. No other header lines.
- `Status: claimed` requires `Owner` and `Claimed`; `open` and `resolved` leave `Claimed` empty (resolved keeps `Owner`).
- `Triage:` is a body field, not a lifecycle status. New tickets start as `needs-triage`; launchable
  tickets use `ready-for-agent`. Legacy tickets without `Triage:` still lint, but cannot be launched by
  the lead-worker readiness launcher until the field is added.
- `Blocked by` ids must exist in the same directory and must not include the ticket itself or form a cycle.
- `claim` refuses when the ticket is claimed and the claim has not expired, or when any blocker is not `resolved`. An expired claim can be taken over.
- `resolve` requires `Status: claimed` by the same `Owner`, and every `## Acceptance` item checked (`--force` overrides and logs it in `## Comments`).
- Free-form sections (for example `## Description`, `## Interface`) may sit between the header and `## Acceptance`; the tool ignores their content.
- `## For later tickets` is optional in ticket files. Each non-empty line must be `- to NN: text`,
  where `NN` exists in the same effort and is not the source ticket.
- The same section may appear in `docs/efforts/<effort>/evidence/NN-report.md`. A report section whose
  only content is `none` is valid and yields no finding.
- `docs/efforts/<effort>/evidence/lead-handoffs.md` may contain `- to NN: text` lines anywhere in the
  file. Other non-bullet prose is ignored.
- `add-finding` appends one line to a ticket file, and `findings` lists every finding addressed to a
  ticket from ticket files, worker reports and lead handoffs. Use `--json` for the structured form.
  Identical finding text for the same target appears once with every source named.
- `## Acceptance` has at least one item; `## Comments` is append-only (the tool only appends lines).
- `list` prints the frontier: open, unblocked tickets first, then claimed, then blocked, then resolved.
- `launchable` prints open tickets whose blockers are resolved and whose triage is `ready-for-agent`,
  ordered by id. Use `--json` for a machine-readable list.

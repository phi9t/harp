# 07 ticket-resolution: Ticket resolution at landing

Type: feature
Status: open
Owner:
Blocked by: 03, 05
Claimed:

Triage: ready-for-agent

## What to build
Landing resolves the ticket through the tracker under the landing lock. Collab never edits ticket files; the Integration commit no longer rewrites ticket status for Corenius efforts.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 33-34 (resolve side)). Use the project glossary terms.

## Acceptance
- [ ] After landing the ticket is resolved with the acceptance evidence recorded.
- [ ] A failure to resolve is reported as landed with records incomplete.
- [ ] A worker candidate that touches ticket files is refused at submit.

## Comments

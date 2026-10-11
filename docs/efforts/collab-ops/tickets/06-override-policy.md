# 06 override-policy: Override policy

Type: feature
Status: open
Owner:
Blocked by: 05
Claimed:

Triage: ready-for-agent

## What to build
A mainline override must name an owner from the configured allowed list. Every override is recorded as an event, including standalone landings, and exported to the effort's evidence when an effort is named.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 19-21). Use the project glossary terms.

## Acceptance
- [ ] An unlisted or whitespace-only owner is refused.
- [ ] A standalone landing with an override records the event.
- [ ] An effort landing also writes the evidence record existing readers expect.

## Comments

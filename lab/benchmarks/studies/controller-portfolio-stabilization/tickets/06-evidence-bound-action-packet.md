# 06 evidence-bound-action-packet: Generate an evidence-bound Action Packet

Type: feature
Status: open
Owner:
Blocked by: 03
Claimed:

## Parent

`docs/efforts/controller-portfolio-stabilization/SPEC.md`

## What to build

Produce and review one schema/fixture-only Action Packet against disposable evidence, then prove that any typed field or normalized body change creates a new packet and that source drift invalidates approval. This ticket cannot target or authorize the real Controller Portfolio.

## Acceptance
- [ ] Closed YAML frontmatter contains target, disposition, evidence IDs, expected generations, body digest, status and required human authority.
- [ ] Packet identity covers canonical frontmatter and the normalized Markdown body; unknown authority fields are rejected.
- [ ] The body states conflicts, rationale, consequences, verification, recovery and unknowns without overriding typed facts.
- [ ] A reviewed approval becomes invalid after source-generation drift, evidence replacement or any body/frontmatter change.
- [ ] Execution cannot begin from a proposed, rejected, invalidated or self-authorized packet.
- [ ] A real pass-two Controller Portfolio packet is refused until ticket 05 closure evidence exists and a later human review approves it.

## Comments

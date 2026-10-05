# 05 lead-control-center: Lead and control center

Type: task
Status: open
Owner:
Blocked by: 01, 02, 03, 04
Claimed:
Spec: ../SPEC.md

## Requirements and evidence

Usable lead tools and dashboard share controller commands/projections. Display objective, two worker assignments, evidence, questions, unknown effects and landing queue. Controls return durable receipts. Prove event replay/reconnect, requested versus observed stop and replacement lead from durable state.

Claim an isolated worktree before implementation. Retain public behavioral red-green evidence, exact source identity and independent Standards/Spec review. No shared branch rewriting.

Owner clarification: implement runtime adapter delivery of inbound questions to the registered Codex lead, durable delivery/ack/resolution states and turn-race recovery. Dashboard shows observed versus reported worker progress, runtime/workspace identity, admitted versus current filesystem/Kata revisions and sync drift. Qualify shared and separate app-server topologies jointly with02/06.

## Acceptance

- [ ] Lead and dashboard use the same API and show objective, both assignments, evidence, questions, uncertainty and landing queue.
- [ ] Required controls return durable receipts; reconnect, requested/observed stop and independent replacement context have public RED/GREEN.
- [ ] Independent Standards/Spec pass.

## Comments

- 2026-10-04T03:53:26Z codex-design: Adopted owner commits9d90222/d18e413/f775734; normalized tracker fields without changing required scope. Controller-first sequencing is binding; Matt route fallback uses the repository crosswalk.

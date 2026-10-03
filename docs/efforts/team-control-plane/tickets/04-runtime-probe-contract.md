# 04 runtime-probe-contract: define the minimum live Trae recovery capability

Type: task
Status: open
Owner:
Blocked by: 01
Claimed:
Label: wayfinder:grilling
Parent: ../map.md

## Question

Which runtime adapter and bounded probes must precede live dispatch? Decide
private supervised app-server versus a transitional exec adapter, explicit
storage/transport ownership and minimum thread/turn reconciliation behavior.
Thread creation with a lost reply may have no known thread ID; treat this
separately from a lost turn-start reply on a known thread.

## Acceptance

- [ ] Owner approves the adapter/probe scope and model/service limits before execution.
- [ ] Specify detach, restart, steering race, interrupt/effect-stop and unknown-start verdicts.
- [ ] Pin runtime/schema versions and distinguish schema availability from exercised behavior.

## Comments

- 2026-10-01T09:34:58Z lead: Preparation: [proposed bounded live contract](../research/live-trae-probe-contract.md) records installed help/version observations and proposed private-entry, recovery matrix, resource envelope and missing launch prerequisites. No service/model launch or scope approval; this ticket remains open.

- 2026-10-02T05:00:00Z owner: “approved for all things” confirms continuation including the proposed finite private runtime probe envelope; recorded in [continuation authority](../decisions/20261002-continuation-authority.md). Approval is no longer the launch blocker. Completed kernel16, executable isolation, selected provider and enforceable spend/token ceiling remain required technical prerequisites. No runtime/model launch or observed recovery result is claimed; ticket remains open until the contract and prerequisites are executable.

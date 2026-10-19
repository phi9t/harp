# 11: Qualify the migrated architecture end to end

Status: ready-for-agent
Owner: unassigned
Parent: [Repository architecture](../spec.md)

## What to build

Review and qualify the complete existing-code migration against the new source identities. Update current operator/ownership docs and instruction/cache/provenance contracts. Keep the three new workload implementations separately open until their own delivery is requested and verified.

## Acceptance criteria

- [ ] Required native lifecycle tests pass without required skips for the migrated runtime/bridge/backend identities.
- [ ] Real Codex and Trae single-worker runs, a mixed pair and four overlapping mixed workers pass independent checks and reviewed serialized exact landings.
- [ ] The two audit reproducers fail safely and ordinary correct candidates succeed.
- [ ] An SSH-dispatched retained workload completes under Insula with reconnect/cancellation evidence and host-neutral public artifacts.
- [ ] Benchmark inventory completeness passes, including acquired dirty-source snapshots and the Emacs gym/AI-LSP suites; historical-only evidence is labeled and no benchmark is silently retired.
- [ ] Relevant math/SICP/PDF consumers and complete-candidate changed-surface gate pass; primary changes are preserved.
- [ ] No imported wrapper directory, harp_durable package, competing controller, Temporal/PostgreSQL execution dependency or committed real host inventory remains.

## Blocked by

- [02-protect-acceptance-oracle](02-protect-acceptance-oracle.md)
- [03-qualify-physical-checkout](03-qualify-physical-checkout.md)
- [05-ssh-runner](05-ssh-runner.md)
- [06-relocate-math-and-sicp](06-relocate-math-and-sicp.md)
- [07-relocate-benchmarks-and-provenance](07-relocate-benchmarks-and-provenance.md)
- [12-preserve-emacs-benchmarks](12-preserve-emacs-benchmarks.md)

- [13-integrate-insula-supervisor](13-integrate-insula-supervisor.md)

## Verification

Freeze the complete candidate, inspect the selected plan with an explicit baseline, run required native/consumer checks and review the resulting diff. Record exact hashes, selected tests, provider overlap and approval/landing evidence. Local landing and any push follow existing human authority; this ticket grants none by itself.

No implementation or qualification result is recorded yet.

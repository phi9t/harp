# 11: Qualify the migrated architecture end to end

Status: in-progress
Owner: Codex on codex/repo-architecture
Parent: [Repository architecture](../spec.md)

## What to build

Review and qualify the complete existing-code migration against the new source identities. Update current operator/ownership docs and instruction/cache/provenance contracts. Keep the three new workload implementations separately open until their own delivery is requested and verified.

## Acceptance criteria

- [ ] Required native lifecycle tests pass without required skips for the migrated runtime/bridge/backend identities.
- [ ] Real Codex runs with one worker, two overlapping workers and four overlapping workers pass independent checks and reviewed serialized exact landings.
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

See the implementation progress below.

## Implementation progress

Migration review found and repaired guardian launch shadowing, frozen-oracle mount isolation, Git replacement-ref identity, missing frozen prompt context, proof-checker execution from inspected repositories, and primary-cache/source coupling. Native runtime matrix: 201/202 initially passed; the exact oracle failure was repaired and all three oracle regressions passed. Native owner: nine passed. Codex one and two workers passed on retained sealed sources; four-way coding overlap is proved, with independent verification rerun after a full-filesystem failure. No failed or old execution has been rebound or relabeled. Final composed source receipts, current candidate review, and local landing remain pending.

Final Codex qualification proved one, two and four actual concurrent workers on
recorded fresh sealed sources. A subsequent real task consumed a newer frozen
Parent while its implementation base retained the older text. All eight exact
candidates passed independent checks, received explicit human approval and landed
serially in their separate repositories; Git heads/trees and journal rows match,
and all destinations are clean. Neutral evidence is in
`evidence/provenance/codex-qualification.json`. Native benchmark84 and supervisor
54Rust+9owner cases pass on the frozen910-file composed source identity. The
full integrated lifecycle receipt and complete-candidate gate remain pending.

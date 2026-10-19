# Use a first-class disposable Trial Repository for Rung 0

Recorded: 2026-10-06

The owner selected a disposable repository for Rung 0 and required Corenius to
become good at constructing such repositories. Rung 0 therefore uses a real
private Git and work-ledger scenario generated from a versioned deterministic
Trial Template, not Corenius, a sibling repository, a mock, or an ad-hoc `/tmp`
checkout.

The template declares the seed tree and history, work items, tests, acceptance
contract, fault hooks and expected identities. Materialization records exact
content and tool identities. Teardown is allowed only after the Candidate,
controller events, verification, runtime observations and cleanup evidence have
been retained independently.

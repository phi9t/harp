# Ticket26 implementation candidate

Fixed base: `720ab7783e1b36e9e8a675f497f9c66ef2b1d87b`. Branch:
`codex-intent26/contracts`. The sole implementing writer has frozen the 17 source
changes recorded in `source-manifest.json`. No commit or integration was performed.
Ticket26 remains claimed pending root-owned independent reviews and integration.

The new public contract service stores immutable namespaced work, exact source
bytes/revisions/digests, intent, acceptance, strategy, delegation, verification and
accepted-subject records. Its transaction checks current authentication, role,
scope, delegation, revisions and exact idempotency arguments. Dependency
supersession changes the dependent revision and suspends fresh grants while
retaining earlier operation receipts. Revalidation requires the new current exact
accepted prerequisite. The existing v1 ArtifactService formats and identities are
preserved.

Production verification requires an independently executed ArtifactService subject
and an explicit v2 verifier binding admitted before execution allocation, with
matching intent, acceptance, policy, environment, suite and commands. The positive
control ran the actual Supervisor gate in its isolated workspace and used the
trusted monitor result through the public ArtifactService verification/read seam.
An already allocated v1 verification episode cannot be rebound. Separate research
verification retains immutable referenced evidence, requires an independent
verifier, and allows negative `investigation_complete` only under the explicit
acceptance contract. Worker and evaluator-change proposals cannot manufacture an
accepted result.

The additive migration is tested against exact legacy specification and original
nullable Attempt/Episode/workspace/execution provenance, repeated migration and
SQL immutability controls. Legacy v2 mapping remains closed until explicit exact
Owner admission; migration does not promote an old v1 proof.

## Executed proof

All behavioral tests used raw JSON public HTTP calls, a real ephemeral PostgreSQL
fixture, and durable reads. No Temporal runner, provider, paid service or GPU was
used. Every attempt, including compilation corrections and a Plan fixture
correction, is retained in `runs.jsonl` with bounded argv, environment, source
hashes, terminal status and raw log hash. The PostgreSQL script adds `--locked`;
the caller adds `--offline`, uses the private target in this worktree, and bounds
build/test concurrency. Every run reached terminal cleanup.

| Retained run | Actual result | Purpose |
| --- | --- | --- |
| `red-admission.log.txt` | compiled; exit101, public HTTP404 versus expected200 | Baseline behavioral RED before implementation |
| `red-authority-plan.log.txt` | exit101, 2 passing / 2 failing | Missing delegation and Plan command behavior |
| `red-dependency-outcomes.log.txt` | exit101, 4 passing / 3 failing | Missing dependency and independent outcome behavior |
| `red-retention-grants-ceilings.log.txt` | exit101, 4 passing / 4 failing | Evidence retention, WorkRef reads, fresh Episode and Plan ceilings |
| `red-replay-ceiling-compatibility.log.txt` | exit101, 8 passing / 1 failing | Current cumulative delegation limits on cached replay |
| `green-api-compatibility.log.txt` | exit0, 75 passing | Library, artifact, kernel, mailbox, delivery, mailbox artifact, supervisor, gateway and contract suites |
| `red-dependency-disposition-revision.log.txt` | compiled; exit101, dependent revision1 versus expected2 | Supersession invalidates stale dependent aggregate revision |
| `green-final-contract-revisions.log.txt` | exit0, 39 passing | Final exact source: library9, gateway4, contracts9, kernel16, mailbox1 |

The compatibility run predates only the dependency revision fix and its public
regression assertion; `source-manifest.json` records both changed hashes. The final
39-test suite exercises those final bytes. The nine contract tests cover exact
admission/idempotency/revisions, retained and unavailable sources, cross-deployment
identity, current bounded delegation and replay, declarative strategy validation
and completed steps, Plan ceiling conservation, independent negative research,
exact supersession/revalidation and old receipts, measured production, and legacy
upgrade provenance. Positive current-authority controls accompany refusals.

`public-records.jsonl` extracts 68 actual public command responses, one actual
measured production subject/result and one original migration provenance snapshot
from the final raw log. Each extraction identifies the raw log line and hash. It
adds no execution claims. The original raw log remains the authoritative proof.
The underlying test databases/workspaces were cleaned; the archive retains the
public proof and measurements, not standalone candidate bundle bytes or a live
deployment.

## Freeze and remaining gates

`source-manifest.json` binds all source postimages, their verified pre-edit/base
preimages, canonical policy/spec/ticket snapshots and the admitted design gate.
`source.patch` contains exactly the source/README changes, including new files;
evidence files are separately sealed by `author-freeze.json`. The normal Git
selection audit includes every durable proof/index/input snapshot and excludes
ignored scratch, target and runtime files. `checks.json` records the formatting,
patch, hash and selection checks. No generated archive rewrites raw log bytes.

Plan execution, durable scope allocation, Kata adapter qualification and v0.2
assembled recovery belong to later tickets. This service admits and bounds
contracts without resetting existing mission epochs or cumulative spend. This
candidate does not claim whole-v2 acceptance. Root retains independent Standards,
Spec, branch and landing gates; they have not been run by this implementing worker.
There is no unresolved authority question from implementation. The shared
Cargo/PostgreSQL resource slot was released after the final terminal GREEN and
cleanup, and remains held for root until explicitly returned.

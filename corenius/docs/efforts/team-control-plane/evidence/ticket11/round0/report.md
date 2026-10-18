# Ticket 11 implementation handoff (review pending)

Baseline commit read: `963f58b31e518a8162dfe719e386ef9d9843321f`.
True source baseline: [admitted ticket 10 manifest](ticket10-accepted-manifest.json).
Worktree: `/data02/home/philip.yang/workspace/corenius-codex-artifact-acceptance`;
branch `codex/artifact-acceptance`. Source frozen 2026-10-01T08:25:20Z.
No commit, push, PR, mainline changes, paid model calls, or production mutations.
Independent Standards/Spec and branch review remain root-owned and pending;
this report does not admit or accept the ticket.

## Implemented boundary

The public typed `artifacts::ArtifactService` provides command envelopes,
scoped snapshots and retained evidence reads. `ArtifactStore` publishes checked
SHA-256 objects and versioned manifests with create-new staging, atomic
non-replacing hard links, file fsync and directory fsync. A fault before manifest
finalization leaves ineligible bytes; a fault before reference rolls back the
broker transaction. Existing corrupt immutable objects cannot be overwritten.

`Configure` is owner-only and explicitly assigns an independent verifier,
mandatory command identities, suite/environment digests, integration base and
whether a scoped non-verifier lead may accept. The verifier uses an existing
scoped lead credential; assignment adds verification authority without widening
parent scope. All service reads/mutations check active recovery epoch and current
parent credentials; worker generation and current attempt principal are checked.
`Submit` is current-implementer-only. Candidate records bind commit/tree/base,
admitted spec digest and immutable admission event revision, attempt generation,
artifact digest, deliverables and risks. Accepted content cannot be silently
reopened. No budget/reservation refund is introduced.

`Verify` retains exact candidate bindings, suite/environment, mandatory command
vectors, exit results, log references and independent verifier principal. `Accept`
requires explicit owner/authorized non-verifier lead action, current exact
candidate/report/policy bindings, successful mandatory checks, resolved decisions
and no unresolved candidate risks. It re-reads candidate and log bytes and
manifests, including on successful acceptance-command replay. Cached command
records retain task scope and argument digests; failed pre-reference transactions
remain retryable. Every successful command advances the task revision and writes
an immutable task event using the existing aggregate namespace.

Owner-only grace collection holds the same PostgreSQL advisory lock as references
and preserves all registered evidence. Positive grace retains fresh orphans;
zero grace is deliberately used only in disposable behavioral fixtures.

Fixture-local `prepare_integration` constructs an actual Git merge tree and a
separate deterministic integration commit with the new base and source as
parents. It checks that commit in a detached disposable worktree, retains each
exit/stdout/stderr result, releases the worktree, and never moves mainline. A
changed base produces a new commit/tree and new checks. Broker base-policy
changes reject the previous candidate/report; submission of the new binding and
fresh verification are required before acceptance.

## Verification

[Retained behavioral proof](ticket11/behavioral-proof.txt) contains:

- Publication RED: incorrect digest was accepted by the initial public stub.
  GREEN: digest rejection, ineligible unfinished manifests, retained byte reads,
  corrupt-object rejection and refusal to overwrite corrupt immutable content.
- Admission/acceptance RED: configuration failed to advance durable task
  revision. GREEN: real PostgreSQL submission, before-reference rollback,
  independent verification, exact candidate matching, explicit acceptance,
  replay, missing referenced logs blocking acceptance and referenced GC safety.
- Git RED: integration preparation unavailable. GREEN: actual changed-base
  integration candidate and checks, failed check exit 7 retained, mainline
  unchanged and temporary worktree released.
- Scoped-read RED: verifier received empty bytes. GREEN: stored candidate bytes,
  unrelated published object denial, unknown credential denial, and epoch
  rotation invalidating read/mutation authority.
- Additional coverage: old generation/spec/admission revision/suite/environment/
  base/commit/candidate/report rejection; grace-retained and collectible fault
  orphans; scope-bound cached commands.

Fresh focused command:
`bash crates/team-control-plane/tests/run-postgres.sh --test artifacts -- --nocapture`
→ six tests passed, exit 0. Uses the admitted pinned loopback disposable real
PostgreSQL runner; every database scenario uses a separate schema.
`cargo fmt --manifest-path crates/team-control-plane/Cargo.toml -- --check`
→ exit 0.
`cargo clippy --manifest-path crates/team-control-plane/Cargo.toml --locked --all-targets -- -D warnings`
→ exit 0.
Repository gate details are recorded in the companion final-gate proof.

Retained run IDs: task `cca94853-2e81-45dd-80d2-e32206fec5f7`, recovery epoch
`5a7a4121-a2d5-4abe-864a-cce01b804060`, candidate
`64ce833e31a140f71686a502d8ef45d8179b9e78c0e41397daf40de8180270bb`, report
`285e6038046753b3134ec66685d0346dd66ef61d71f8eda270c1e59f03b162fd`, candidate
artifact `387b75ea1990ff5f32d97d80a9d6e90de0cb7012a40b9da0404987c1c9794037`.
Git first/changed-base commit/tree/base IDs are retained in behavioral proof.
Fixture databases and temporary repositories/objects are removed after the run;
IDs identify the retained proof, not a live deployment.

## Exact changes and review package

[Baseline-relative patch](ticket11/baseline-relative.patch) SHA-256:
`14596fc90358693091edba2907173800aa7882d1e82871db2cadea87c07995af`.
[Source manifest](ticket11/source-manifest.json) records every source hash.

Source changes: `src/artifacts.rs`, `migrations/0002.sql`, `tests/artifacts.rs`
under `crates/team-control-plane`; shared glue is `src/lib.rs` exporting the
module and `src/storage.rs` making only DSN, active-epoch validation and worker
parent-generation validation crate-visible. All other admitted ticket 10 hashes
remain unchanged. The additive artifact version marker stays separate from core
schema v1 and gateway migrations; migration applies atomically under the existing
lock. The public module shares existing sanitized error codes and excludes DSN
and bearer material from debug/error output.

## Limits

This is the authorized model-free kernel/fixture slice. Temporal, gateway,
production supervised verifier scheduling, protected integration authorization
adapters and production deployment are separate tickets. Integration checks are
trusted finite fixture commands; the helper is local and is not exposed as a
production HTTP mutation. Production process-tree shutdown/deadline guarantees
remain ticket 15 scope. Environment and suite identities are explicit
owner-approved digests, not an inferred host attestation. Report submission is
an authenticated verifier action; the broker does not autonomously execute test
commands or trust implementer prose as proof. The standalone filesystem store
is operator-configured local trusted storage, not an untrusted filesystem or
remote object-store isolation boundary. No machine power-loss experiment was
performed; actual file/directory fsync calls and deterministic interruption
points were exercised. No design/architecture change beyond the approved spec
is claimed.

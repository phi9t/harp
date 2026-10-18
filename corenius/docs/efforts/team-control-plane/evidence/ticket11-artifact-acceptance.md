# Ticket 11 handoff — repair round 1, independent review pending

Baseline commit read: `963f58b31e518a8162dfe719e386ef9d9843321f`.
True source baseline: [admitted ticket 10 manifest](ticket10-accepted-manifest.json).
Worktree: `/data02/home/philip.yang/workspace/corenius-codex-artifact-acceptance`;
branch `codex/artifact-acceptance`. Source frozen 2026-10-01T08:45:15.200656+00:00.
No commit, push, PR, mainline changes, paid models or production mutations.
Root retains ticket admission and independent Standards/Spec/branch review.
This report does not claim ticket acceptance. The [initial package](ticket11/round0/report.md)
is retained as historical evidence and superseded by this round.

## Implemented boundary

Public typed `artifacts::ArtifactService` supplies scoped commands, snapshots and
retained evidence reads. SHA-256 checked immutable objects/manifests use
create-new staging, atomic non-replacing hard links, file fsync and directory
fsync. Pre-finalization bytes remain ineligible; pre-reference faults roll back
the database transaction. Corrupt existing immutable objects cannot be replaced.
Owner-only grace GC shares the reference transaction lock and preserves all
registered candidate/log evidence. Positive grace retains fresh orphans; zero
grace is deliberately used only in disposable behavioral tests.

Owner configuration assigns the independent verifier, mandatory command vectors,
suite/environment identities, integration base and explicit lead acceptance
policy. Current-implementer submission binds exact commit/tree/base, artifact,
admitted specification digest and immutable admission-event revision, attempt
generation, deliverables and risks. Independent verification binds that exact
candidate, suite/environment, executed command results, retained log digests
and verifier principal. Acceptance requires explicit authorized action, current
candidate/spec/base/report/policy, successful mandatory checks, resolved decisions
and no unresolved risks. It re-reads candidate and log bytes, including on
successful acceptance-command replay. Successful commands advance task revision
and append immutable task events. Scoped idempotency records retain argument
bindings. Reservations/budgets are not refunded or silently released.

Fixture-local Git preparation constructs the actual merge tree and deterministic
integration commit, checks it in a detached worktree, retains exit/stdout/stderr,
then releases the worktree. It does not move mainline or push. Changed base policy
invalidates old proof; a new actual candidate and fresh report are required.

## Review blockers repaired

1. The verifier formerly reused a Lead credential, allowing entry into execution
   admission. The visible regression returned `DependencyUnmet` from
   `AssignAttempt` where `Unauthorized` was required. A distinct `Role::Verifier`
   now requires mission, narrow task scope and positive generation; it cannot
   assign attempts, configure policy, accept tasks, admit specifications,
   change dependencies or issue credentials. Authentication recognizes roles
   explicitly and rejects unknown roles. All verifier service/core reads,
   verification mutations and cached report retries recheck parent scope,
   current task generation, expiry and deployment recovery epoch. Tests cover
   missing/zero generation, missing task scope, future generation, expiry,
   cached replay and denial of mission-wide reads. Assignment guards allow
   only Owner/Lead. The artifact migration expands credential constraints
   atomically while leaving accepted core schema version 1 unchanged.

2. The original Git and PostgreSQL fixtures were separate. A new public
   end-to-end fixture publishes actual prepared Git bundle bytes, submits their
   exact digest/commit/tree/base through PostgreSQL, and lets the verifier read
   the retained bytes through scoped evidence access. It imports that bundle
   into an independent repository, measures HEAD, tree and first-parent base,
   executes checks there and publishes the resulting stdout/stderr logs.
   A real base commit change produces a different integration commit/tree,
   bundle digest and executed log digest. Old candidate/report proof is denied;
   the fresh exact candidate/report is accepted and observable in the public
   snapshot. Mainline remains unchanged. The end-to-end regression was recorded
   red before the role repair, at its execution-authority assertion. Subsequent
   Git bundle fixture setup initially failed because Git requires a named tip;
   private temporary fixture refs now export the exact full history and are
   removed immediately. This is fixture setup, not a broker architecture change.

## Fresh evidence

[Behavioral proof](ticket11/behavioral-proof.txt) retains both new red runs and
fresh green results; [round 0](ticket11/round0/behavioral-proof.txt) retains the
original publication/acceptance/Git/scoped-read red-green records.

- `bash crates/team-control-plane/tests/run-postgres.sh --test artifacts -- --nocapture`
  → ten tests passed, exit 0. Uses admitted pinned disposable loopback real
  PostgreSQL; scenarios use independent schemas.
- `cargo fmt --manifest-path crates/team-control-plane/Cargo.toml -- --check`
  → exit 0.
- `cargo clippy --manifest-path crates/team-control-plane/Cargo.toml --locked --all-targets -- -D warnings`
  → exit 0.
- `bash tools/agents/check.sh` → exit 0, `check: all green`.
  [Final gate proof](ticket11/final-gate.json) records command, terminal result,
  retained full-log hash/path and terminal output.

Retained real-Git/PG run: task `9e164fb7-eff5-4a58-917c-164be73eba09`.
Initial commit/tree/base:
`bbd47f0e4c6824ceb5f93bbdc75a177d3bd09e97` /
`d6557d9fe1b9a641b4ee252acab136ca97e6ca5d` /
`f316fcce00b98f0fce2f4760ead36c502af4f40e`.
Changed commit/tree/base:
`adbdcd2c12834f3c51beed119c2a980229646a45` /
`9a28de91917846619ccb976493b56a13e5409793` /
`e17d95c2469143632eaae6b43596c5ebe255d6fb`.
Accepted candidate `471f420057b715392182583ea44f9ea6d87bf8320d025da742581e40d8c3e261`;
report `58c2a20be3c7de2dbc86f32235fec764a591fd28377032df89847ec9d66e53e8`;
bundle `a2508b07e839097446ff60db3c5e77697f651e8d034b8a7dcacbf7569414b6e4`;
executed log `66db851ecaa6ba5912340c5bc837b75f824c0b8149be3328e3169a5b862d26ba`.
Fixture repositories and DB container are removed after runs; these IDs identify
retained proof, not a production deployment.

## Exact changed paths and shared glue

[Baseline-relative patch](ticket11/baseline-relative.patch) SHA-256:
`c8aaa49b0f8ba63dd1f1f93df11fe09898d0a988dca73f9a7784f024aee9d67c`.
[Source manifest](ticket11/source-manifest.json) records all eight source hashes.

Under `crates/team-control-plane/`:

- New `src/artifacts.rs`, `migrations/0002.sql`, `tests/artifacts.rs`.
- `src/lib.rs`: export public artifact module.
- `src/storage.rs`: crate-visible DSN/active-epoch/parent-generation helpers;
  generation checking now covers both Worker and Verifier; authoritative core
  migration invokes additive artifact migration before exposing new grants.
- `src/domain.rs`: add distinct `Verifier` role.
- `src/auth.rs`: explicit verifier role decoding/unknown-role denial;
  execution assignment allows only Owner/Lead.
- `src/admission.rs`: task/generation-scoped Verifier grants and role storage;
  assignment allows only Owner/Lead; verifier does not acquire worker authority.

All other admitted ticket 10 hashes remain unchanged, including migration0001,
immutable specification history, aggregate event namespace and scoped rejects.
The additive artifact migration has its own version marker; gateway migration
ownership remains separate. Credential expansion occurs through the authoritative public core migration
entrypoint, which invokes public `artifacts::migrate` after migration0001.
`ArtifactService::open` delegates to that same additive migration. A separate
Broker-only fixture first failed with `TemporarilyUnavailable` on verifier grant
without a service constructor, then passed after this correction. Repeated public
core migration is also exercised. Root integration orders gateway0003 after
artifact0002. Core schema version remains 1.
No raw DSN or bearer material is added to Debug or error output.

## Limits

This remains the authorized model-free kernel/fixture slice. Temporal, gateway,
production supervised verifier scheduling, protected integration authorization
adapters and production rollout are other tickets. Integration commands are
trusted finite local fixture commands; this helper is not a production HTTP
mutation. Process-tree/deadline guarantees remain ticket 15. Environment/suite
identities are explicit owner-approved digests, not inferred host attestation.
The broker authenticates retained verifier reports and exact content bindings;
it does not autonomously execute tests. The end-to-end fixture independently
executes checks on reconstructed retained candidate bytes. Filesystem storage
is operator-configured trusted local storage, not an untrusted filesystem or
remote-store isolation boundary. No machine power-loss experiment was run;
actual file/directory fsync calls and deterministic interruption points were
exercised. Independent reviews remain pending.

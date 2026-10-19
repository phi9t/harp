# Kata executable evaluation — ticket17 in progress

Corenius commit read: `963f58b31e518a8162dfe719e386ef9d9843321f`.
Kata source built: `b70a4306464e319701df30a3b92860289613b15a`.
Observed behavior below is from an isolated model-free fixture, not production.
No adoption, hooks, tracker migration or broker authority change occurred.

## Reproducible public seam

Built the exact pinned source with Go1.27.1 downloaded from
[official downloads](https://go.dev/dl/?mode=json), archive SHA256
`63d339f0da5ab53635a56f2490a7984dfe12dfcff22ad749f63edaf590168445`.
The existing installed v0.14.3 binary (410ee88) is a different revision and was
not used as evidence for this pinned build. No shared installation changed.

Run [probe](probes/kata_remote_probe.py) with `--binary <pinned-build>` and
`--output <results.json>`. It starts a foreground loopback daemon with a
private temporary KATA_HOME and identity mode, calls real HTTP endpoints,
then stops the daemon and removes its home. It never launches a model.
[Results](../evidence/ticket17-kata-remote-results.json) and
[manifest](../evidence/ticket17-kata-probe-manifest.json) retain hashes and build identity.

## Observed results

| Fixture | Result | Meaning for integration |
| --- | --- | --- |
| Public health | API schema0.24.0 | Pin schema capability separately from app version |
| Worker reads outside issue subtree |404| Scoped remote read boundary enforced |
| Worker edits its issue title |200, changed title| Capture immutable approved content digest; current issue content is mutable |
| Invalid evidence type on close |400| Evidence shape is checked |
| Valid `test` evidence naming `false`, never executed |200, reason done| Closure is an attributable claim, insufficient broker acceptance proof |
| Exact idempotent close replay |200 with original_event| Consume retained receipt, avoid interpreting each delivery as new work |
| Expired scoped token read |403| Remote scope expiration enforced in this fixture |
| Two competing if_unowned claims |200 and409| One issue owner recorded in concurrent claim fixture |
| Stopped-process SQLite copy and restart |Closed issue/owner retained; expired token denied| Cold restore roundtrip works for this fixture only |
| Teardown |Process terminal, loopback listener closed| No retained daemon |

The edit response and subsequent read both showed revision1 despite changed
title in this fixture. This observation alone does not establish the meaning
of Kata's revision field; a broker mapping must hash approved canonical content
and independently bind its own revision. Do not assume source issue revision
is the broker specification digest.

## Boundary and reuse assessment

Kata supplies a substantial existing ledger, CLI/TUI/web/MCP, typed evidence,
claiming, scoped credentials and events. Its pinned
[HTTP contract](https://github.com/kenn-io/kata/blob/b70a4306464e319701df30a3b92860289613b15a/docs/reference/http-api.md)
and [close contract](https://github.com/kenn-io/kata/blob/b70a4306464e319701df30a3b92860289613b15a/docs/reference/cli.md#close)
support those observations. The close probe demonstrates why broker acceptance
must remain separate from issue closure; it is not a defect in a ledger that
records completion claims.

Inference: a Rust deployment using the remote API still needs source-ID binding,
approval digest capture, revision reconciliation, deduplicated projection,
credential lifecycle and a separate daemon/database upgrade/restore procedure.
Go embedding offers transactional host access-controller hooks; those hooks
are not automatically present as callbacks in a Rust remote client. See
[pinned embedding](https://github.com/kenn-io/kata/blob/b70a4306464e319701df30a3b92860289613b15a/docs/development/embedding.md).
Neither ledger claims nor scoped credentials fence an external agent process.

Provisional recommendation: keep Markdown-approved planning and the PG broker
for the first executable kernel; evaluate Kata as a later optional ledger/UI
adapter. This is not a final adopt/defer decision or ticket acceptance.

## Remaining evidence before ticket17 disposition

Forward SQLite upgrade and PostgreSQL logical restore were executed (below).
The subsequent evaluation-scale broker fixture now proves concrete approved
source binding, actual source-event duplicates, guarded broker projection and
ledger-restore replay while preserving one broker authority. See
[mapping](kata-broker-mapping.md), [probe](probes/kata_broker_probe.py) and
[results](../evidence/ticket17-kata-broker-results.json). Production bridge
implementation/operation effort remains unmeasured; this bounded fixture is not
adoption or a production adapter. Independent review of the new seam remains
required before ticket disposition. Conditional ticket18 remains unresolved.

The agent service rejected an additional research context due to its thread
limit. Primary performed this bounded research locally in its own isolated
Kata worktree; independent review is still required.

## Forward upgrade fixture

[Upgrade probe](probes/kata_upgrade_probe.py) ran the installed v0.14.3 binary
(reporting commit410ee88; full source lookup410ee88bca267b117c845a36f8249890c5160e3b)
in an isolated home, created an issue/claim, stopped it, and started the exact
pinned build against that same fixture store. [Results](../evidence/ticket17-kata-upgrade-results.json)
retain both binary hashes and version reports. API0.10.0 became0.24.0; issue UID,
title, winning owner and existing identity credential were preserved. New
if_unowned claim correctly refused the retained assignment with409. Both
processes were terminal and the listener closed. Old binary rejected the new
if_unowned body property and required actor; the probe uses each API's accepted
request shape. This exercises forward SQLite migration only, not rollback,
online backup, production deployment or PostgreSQL migration. The old binary's
source identity is self-reported, not a reproducible build attestation.

## PostgreSQL logical restore fixture

The same pinned public API probe passed against a disposable loopback PG17.6
fixture launched by immutable image digest retained in the manifest. After
stopping Kata, the probe used actual `pg_dump --format=custom`, dropped and
recreated only the fixture database, restored via `pg_restore --exit-on-error`,
and restarted Kata. Closed issue, winning claim and expired-token denial were
preserved. [Results](../evidence/ticket17-kata-postgres-results.json) and
[wrapper](probes/kata_postgres_probe.sh) are retained. The daemon stopped,
listener closed, and the uniquely labeled container was removed. This proves
a logical restore of this small fixture; no online backup/WAL/PITR/remote TLS
or production restore claim follows. Existing scope/expiry/race/close/replay
observations were also reproduced on PostgreSQL.

## Adapter cost and disposition frontier

| Reused capability | Observed source boundary | Remaining broker-owned work |
| --- | --- | --- |
| Existing ledger and human interfaces | CLI/HTTP; browser/TUI/MCP present in pinned source | SourceID binding/projection and operational packaging |
| Scoped worker identity | Actual subtree denial/expiration | Map separate grants and recovery epochs; never elevate ledger credential into runtime authority |
| Claims | Actual concurrent winner | Record broker attempt/generation/reservation independently |
| Closure/evidence | Valid test claim closes without execution | Verify retained bytes/exact candidate and independently executed logs |
| Replay | Exact close returns original receipt | Stable source-event IDs, deduplication and projection reconciliation |
| Mutable source | Actual title changed | Freeze approved canonical content digest and handle amendment explicitly |
| Operations | Actual SQLite forward upgrade and PG logical restore | Extra daemon/schema/API compatibility lifecycle and coordinated broker recovery |

Inference: even without a fork, adopting the ledger requires at least the seven
seams above, a second authorization plane and another service/schema lifecycle.
No custom issue UI is currently being built, so that reuse benefit does not
yet offset proven kernel work. Proposed disposition is **defer Kata integration
for the first persistent kernel**, retaining the tested optional ledger candidate.
This preserves the owner's approved Git-planning/PG-execution baseline. No
new Kata authority, migration or hook installation is proposed.

Unproven: production bridge delivery/reset/outage behavior, credential packaging
and measured production adapter implementation effort. The new executable
fixture proves evaluation-scale source binding and source/broker event replay;
it does not infer a production implementation from those successful cases.
If adoption is later selected, conditional ticket18 must implement and prove
that operational adapter. Ticket17 final acceptance/disposition remains subject
to independent review against the complete contract.

## Evaluation repair1: real Kata → broker command/read seam

The earlier Spec review identified missing concrete binding and broker duplicate
replay evidence. The documented [mapping](kata-broker-mapping.md) and new
[public fixture](probes/kata_broker_probe.py) address those gaps without installing
an adapter. Accepted ticket10 Rust source was copied into the owned scratch
worktree, every file hash verified against the accepted manifest, and compiled
with `cargo build --locked`. The fixture checks those source hashes again and
records the resulting broker binary hash alongside the pinned Kata binary hash.

Meaningful RED: the first event receiver used a fresh command key for each
actual Kata claim-event delivery. Its first broker HTTP request returned200,
while the duplicate returned409 `stale_revision`; the fixture required the
original durable assignment receipt. GREEN uses the stable source-event plus
explicit owner-decision key, including across reconstructed readers and a real
ledger database restore. Source claims without an owner decision still admit
zero attempts. Concurrent duplicate approved delivery returns one exact
attempt/operation/reservation receipt and one outbox row.

Source edit and unexecuted-test close claims are actual Kata events. Replaying
both leaves the broker admitted spec/revision and running status unchanged.
Ledger tokens receive401 from the broker. An actual broker assignment event is
projected through guarded remote metadata. Repeated HTTP projection creates no
second event; replaying the projection event leaves one broker admission.
Kata logical restore while the broker remains active preserves the source
origin/event identity and exact original receipt. The retained JSON contains
all mappings, IDs and public snapshots; no mock/in-memory broker is used.

Measured boundary: one owned PostgreSQL container, two logical databases,
two foreground services, one task/attempt, finite2-capacity/8-episode/3-repair/
30-second-deadline policy, zero model calls. The probe is447 lines and the final
recorded full run is3.224 seconds on this fixture host (including startup and
ledger restore, excluding compilation/teardown). Those are observed fixture
costs, not production adapter engineering estimates. SourceID binding,
approval capture, explicit assignment, projection guards and coordinated restore
are now exercised seams; deployment/delivery/reset/credential packaging costs
remain additional work. With no custom issue UI being built, defer remains the
recommended first-kernel disposition under the approved baseline, not adoption.

Fresh syntax check and public fixture exit0; services terminal, listeners closed,
temporary home removed and container removed. No tracker migration, hooks,
production changes, paid models, shared install changes, commit or push occurred.
The new fixture and mapping require fresh independent review; no acceptance
claim follows from this report.

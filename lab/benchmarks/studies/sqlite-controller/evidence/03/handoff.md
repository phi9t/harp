# Ticket03 implementation handoff

Source is frozen at `d201642b36695dbd0f29a7ac857ee9e8de520728`, branch
`sqlite03/kata`, from exact base
`b55bd9c3e6e44bbb03b8e1ca026e9a3210d8829f`. One author used the exclusive
`corenius-sqlite03-kata` worktree. This report and its archives are an
evidence-only follow-up to that source commit. No push, landing, shared ref
rewrite, other checkout write, production task/token, upstream model call or
tmux change occurred. Root owns canonical coordination, independent Standards,
Spec and branch reviews, final full verification, integration and acceptance.
Final independent gates are unresolved at this handoff; passing author tests is
not acceptance. While preparing this evidence, root reported Standards PASS
with a low nonblocking type-design note and branch643 FAIL with five findings.
Spec review and root's full gate were still running. See the exact retained
[branch report](reviews/branch-643.md). Product bytes remain frozen pending
root's consolidated repair1/3 dispatch.

## Implemented boundary

The controller optionally binds one exact Kata deployment/project and reads
source through its supported HTTP API. Trusted bootstrap selects a private
configuration and token outside worker mounts. Qualification checks API0.24.0,
actual instance UID and authenticated actor, project UID, item UID/project and
the item's creation-origin event before source reads and item effects. HTTP and
filesystem capture run outside SQLite locks/transactions. Workers receive no
Kata credential, credential filename or unrestricted endpoint.

Configured admission uses `create_kata_task`, then `admit`. It refuses
caller-authored `create_task`, changed/closed input and admission of retained
unlinked storage-fixture tasks. The actual Kata WorkRef is globally unique
across local-project aliases; a database uniqueness constraint backs the
transactional command guard. Existing fixture history is retained, not promoted
into qualified source authority.

Source records contain the deployment/project/item identities, observed Kata
revision/content digest, registered storage ID, explicit relative paths,
exact Git commit or explicitly captured dirty/untracked snapshot, every selected
file's original bytes/base64 and SHA256, and an aggregate snapshot digest.
Admission pins its immutable brief/source version. Current observations and
proposed amendments retain their own immutable versions. Dirty snapshot changes
and external Kata intent/status/metadata changes remain proposals, including
changes noticed during a refused admission. No amendment silently changes an
assignment or admitted brief. The selected Git commit remains the source pin;
a later repository HEAD is not automatic authority.

The operator must select every file needed to interpret intent, including
references. Filesystem snapshots refuse symlinks. Git mode retains exact blob
bytes, including symlink blobs without dereferencing them; referenced targets
must be explicitly selected. This is exact source capture, not automatic
discovery of an arbitrary document's reference closure or an amendment-adoption
workflow. Candidate/workspace/adoption execution belongs to the subsequent
ticket04 contract.

Controller events queue separate metadata and genuinely human-visible comment
effects in the same SQL transaction. Effects retain prepared/applying/confirmed/
unknown state, stable identity, immutable payload/controller event/revision,
acknowledged Kata revision and append-only observation history. The supported
`corenius.execution` metadata patch uses both `If-Match` and previous-value/
absence guards. Comments use supported `Idempotency-Key` plus exact body/author
readback. An ambiguous absent comment beyond the server's seven-day key
retention is withheld rather than blindly repeated. Actual HTTP happens after
journaling. Ordered reconciliation stops at the first unknown effect, retaining
conservative uncertainty and withholding later publication.

The visible mapping is created→unassigned, starting→assigned,
observed running→running, unanswered question/unknown/cancel/repair/integration
condition→blocked, and submitted/accepted/landed/aborted→the same named
disposition. The last statuses describe projections of controller state; this
ticket adds no candidate acceptance or landing commands. Broker-owned metadata
revision bumps update current/acknowledged revisions without producing intent
amendments. External execution-summary drift is retained and guarded publication
is withheld. Kata native closure never proves acceptance, landing or process
termination.

Scoped participants can report progress, ask questions, route/acknowledge peer
messages and propose followups. A proposal creates no assignment or Kata work.
Authenticated sender identity is authoritative; caller-supplied sender fields
are refused. Workers cannot claim/close, edit dependencies, choose another
task's authority, issue grants, replace the lead or reconcile Kata.
`participant_context` reconstructs source versions, assignments, decisions,
questions/answers, messages/acknowledgements, followups and effect histories.
Linked-task intent and native lifecycle projections retain cleanup uncertainty
without exposing credentials or raw routing.

The schema is an explicit additive `kata-v1` extension retaining global
`user_version`1/2 for the qualified controller interfaces. Only the exact
qualified controller build/configuration is supported. Startup requires the
retained matching configuration; there is no implicit historical source import.
An old binary is **not** fenced by a new global schema version. No downgrade is
authorized or promised. README and source/schema identity state this boundary.

## Real daemon and public proof

All credited Kata runs use source
`b70a4306464e319701df30a3b92860289613b15a`, API0.24.0, and binary SHA256
`8b0f05b1071f5e7a2b8173c9e5ef358d550a601cebd8c9d9cae6e45975e837f6` at
`corenius-codex-kata-evaluation/.scratch/team-control-plane/kata17/kata-pinned`.
Installed Kata0.14.3 is incompatible. Each fixture has a unique private
KATA_HOME, loopback ports, isolated project/input repository and fixture-only
credentials. The proxy forwards real requests/responses and drops selected
actual HTTP200 acknowledgements; it fabricates no success. The controller is
exercised through its public Unix socket.

| Evidence | Observed result and provenance |
| --- | --- |
| `source/pinned-red-before.json`, `pinned-red-source.tar`, `pinned-red-executable-before.json`, `logs/pinned-red.log.gz`, `pinned-red-confirmation.log.gz` | Sealed exact base source/executable plus frozen public driver. Real daemon setup succeeds; public source capture fails with `unknown command: create_kata_task`, exit1. This reaches intended product behavior, not setup failure. |
| `source/pinned-green-before.json`, `pinned-green-source.tar.gz`, `pinned-public-test.py`, `logs/pinned-green.log.gz` | Same driver SHA as sealed RED; real public test passes1/1 in2.82s. This is a historical source-pinned pair, not the final source claim. |
| `source/alias-red-before.json`, `alias-red-before-source.tar.gz`, `logs/alias-red.log.gz` | Genuine pre-fix public RED: the same actual Kata WorkRef was accepted under a second local-project alias; test exits101. |
| `source/unlinked-regression-red-before.json`, matching source archive, `logs/unlinked-regression-red.log.gz` | Sealed regression variant removes only the unlinked-admission guard in the author's own disposable test state; real public flow reaches `forbidden command succeeded: admit`, exit101. It does not rewrite a shared ref. |
| `source/final-covering2-before.json`, matching archive, `logs/final-covering2.log.gz`, `final-covering2-after.json` | Final public driver passes alias uniqueness, unlinked refusal and all other Kata scenarios; affected inherited checks also pass40/40. Driver SHA is identical to both final boundary RED drivers. All29 source hashes are unchanged across command. |
| `source/clean-parent-before.json`, `logs/clean-parent-green.log.gz`, fixture `k3-wcbhpl3b` | Identical final driver starts with its parent `.scratch` absent, creates it, and passes the full real public flow, exit0. Driver/binary hashes were recorded before command. |

Final driver SHA256 is
`a13da2b0d8e26b4929ff4b11e253801465807065d096036c223ce3183cfde9f3`.
Final archives include every new source/test file. Parent independently checked
the original sealed pinned pair; that does not replace final independent gates.

The final public driver independently decodes both selected source files and
checks their bytes/digests, distinguishes exact Git bytes from dirty snapshot
bytes, and checks immutable admitted/current/proposed identities. It proves
duplicate receipt replay, changed-before-admit refusal with a durable proposal,
worker authorization controls, durable questions/answers/followups/messages,
lost metadata and comment acknowledgements with readback/no duplicate comments,
owned revision changes without amendment loops, real daemon offline/restart and
status recovery, filesystem plus Kata amendments during execution, reconstructed
replacement-lead context, independent external closure, guarded external
namespace drift, offline source uncertainty and changed actual deployment
refusal before item effects. The alternate deployment control deliberately uses
its own fixture bootstrap credential so authentication succeeds far enough to
test actual instance-UID refusal. The earlier403 on a token minted by the first
daemon was a correct authorization control, not a product success or API defect.

Earlier `red*`, `green*` logs and their original fixture DBs/qualifications are
retained as **unsealed source history**. A contemporaneous full source archive
was not captured for those early runs; no retrospective provenance is claimed.
Setup failures (Python helper collision, long socket path, API validation/token
requirements, startup failures), the genuine early `create_task` bypass and
proposal RED remain visible. `unlinked-red.log.gz` and
`final-covering.log.gz` are intermediate oracle failures caused by changed
validation order; neither is credited as final proof. Strict Clippy's original
`cmp_owned` failure is retained in `clippy-initial.log.gz` and was corrected
before final checks.

## Terminal author checks and precise composition

Commands use `--manifest-path crates/sqlite-controller/Cargo.toml`,
`--locked --offline`; test commands use `--features fault-injection` and
`-- --test-threads=2`. They ran standalone; no broad old PostgreSQL/Temporal
suite was added.

| Command/check | Terminal outcome | Source evidence |
| --- | --- | --- |
| `cargo test ... -- --test-threads=2` |69/69 PASS: units4, native25, Kata1, process4, public20, runtime15; native suite877.12s | `pinned-green-before.json` equals `full-tests-after.json` across all29 files; `logs/full-tests.log.gz`. |
| `cargo test ... --test kata --test public --test process --test runtime -- --test-threads=2 --nocapture` |40/40 PASS: Kata1, process4, public20, runtime15 | Final covering2 before/after manifests identical; `logs/final-covering2.log.gz`. |
| Clean-parent direct identical public driver |PASS exit0 | `clean-parent-before.json`, `logs/clean-parent-green.log.gz`. |
| `cargo fmt --manifest-path ... --check` |PASS exit0 | `logs/final-fmt.log.gz`. |
| `cargo clippy ... --all-targets --features fault-injection -- -D warnings` |PASS exit0 | `logs/final-clippy.log.gz`. |
| `cargo build ...` without fault-injection |PASS exit0 | `logs/final-build.log.gz`. |

The69-test full gate predates four final files: README, `src/kata.rs`,
`src/kata_schema.sql`, and `tests/support/kata_protocol.py`. Final changes add
clean-parent fixture setup, global WorkRef uniqueness, unlinked admission
refusal and corresponding documentation. The final40-test gate covers those
changes and affected public/process/runtime behavior. Native/runtime/schema
implementation and native test files are byte-identical to the terminal full
gate. Per root's explicit routing, the author did not rerun unchanged native25
solely for these Kata-specific changes. This is an honest composed author gate,
**not**69 tests on final source. Root runs a fresh bounded full gate on the final
clean commit.

`source/frozen-source.json` pins commit `d201642` and all29 files; its archive
SHA256 is `aae03b0ae4ce664ff10fa452a42ec7b7a0005de65a6e6119069ff6807e94bacc`.
Its29 hashes equal final-covering2-before/after and final-terminal-after, and
were rechecked against the actual worktree during evidence retention. The
evidence-only commit changes no tested product bytes.

## Retention, process disposition and remaining limits

`retention.json` identifies25 original fixture directories, protected local
credential digests, every retained file/archive digest and raw qualification
records. `fixtures/` archives retain stopped controller/Kata DB/WAL/SHM sets,
source Git objects/input bytes, public/API traces, errors and observed
process identities/exits. Private `kata.token`, `controller-kata.json` and
`config.toml` remain local600; archives exclude them. Archive hashes and absence
of those filenames and protected Kata token bytes were independently checked
during handoff preparation. Synthetic fixture controller grants may occur in
private-test receipts; these are not production credentials.

Credited fixtures record observed owned PID/start ticks/boot identity and waited
terminal children, including explicitly stopped/restarted Kata instances. No
unrelated process was signalled. Early `run-cqgu0d7r` has no qualification result
and remains labeled incomplete setup, not credited terminal proof. Raw failures
and original process dispositions were preserved rather than overwritten by a
later success claim.

External namespace tampering intentionally leaves unknown synchronization at
the end of public fixtures; old failures remain in effect history after later
confirmation. Offline and source-drift observations retain uncertainty, not
inferred success. The controller task can remain active/observed running even
after external Kata closure. These final unknown states are evidence, not
cleanup targets.

Ticket01 and qualified02 supervised interfaces are prerequisites. Full02 stays
unaccepted because of the known fail-closed pre-unsubscribe restart cleanup
liveness gap routed to04. This implementation leaves native cleanup, registry
schema and runtime source unchanged and exposes `runtime_cleanup.qualified=false`
plus retained native uncertainty in reconstructed context. It claims no native
retirement, replacement-lead runtime completion, release readiness or live model
qualification. Ticket04 owns candidate execution and the linked runtime
correction. Root reviews and its final gate determine ticket03 conformance.

The received branch findings require repair before acceptance: descriptor-bound
snapshot traversal to close the pathname check/read symlink race; bounded
autonomous Kata work so slow upstream responses cannot starve the sole socket
loop; ordering within each task while allowing publication for unrelated tasks
past an unresolved effect; a bounded visible progress excerpt so a valid maximum
local report cannot produce an oversized permanently unpublishable comment; and
a fault-injection gate for the deterministic public test fixture. Existing
author checks do not establish those missing behaviors. This evidence handoff
preserves the reviewed source and does not start repair independently.

## Consolidated repair1/3 handoff

Root dispatched the six Kata findings and then authorized the independently
adjudicated adoption-fixture scheduling correction. Source is now frozen clean
at `4d5fbff0323f27c6dcb30aec5cf3df7a839511e0`. The original report/evidence above
remains historical; the exact repair source, RED/GREEN, retained resources,
scope and remaining native attribution limits are in
[repair1/handoff.md](repair1/handoff.md).

All six Kata boundaries have real public RED/covering GREEN, including feature-off
fixture gating. Author affected checks pass49/49; the changed exact native case
passes1/1 in both actual shared/separate modes with identical original driver,
deadlines and identity/frame/client/count/peer-creation oracles. Final post-FI
feature-off/fmt/strictClippy/normal build pass. This is an explicitly composed
50-check author result: the49-check source predates only the native FI/fixture
files. Root owns the fresh scoped50 gate and independent rereviews; no ticket03,
full02/04, assembled06 or live qualification acceptance is inferred.

The controlled native RED proves the old global-hold/B-pending circular wait
after real A recovery; the FI-only per-actor freeze lets exact B retry and
preserves both actual projections. It does not reproduce or erase the original
root native `list_turns` refusal. Initial evidence inventory bytes were unchanged
before this expressly requested handoff append; initial40d4805 retains the
original handoff hash. Source archives, failed setup/oracles, unknown states and
prior process dispositions remain immutable in their separately named records.

Root subsequently reported the independent scoped50/default/fmt/strictClippy/normal
build gate PASS. Standards and branch644 rereviews were active; Spec rereview
awaited this author handoff. No acceptance is claimed.

# Ticket01 author handoff

Route: approved gated baseline; Matt owner unavailable, repository crosswalk
applied with TDD and verification skills. Root owns independent Standards/Spec
and branch gates; ticket remains claimed, not resolved. No downstream adapter,
provider, Git landing, Kata, dashboard or historical import was implemented.

Original assignment base: cf52c11055be09298e6ecc1353e5901e45726551.
Integration base: 55bb58b344c44990f527b7bfb61947702fbe5445 (root-authorized doc-only
fast-forward before the first source commit). Worktree: corenius-sqlite01-storage;
branch: sqlite01/storage. Legacy source/evidence is untouched.

Public API and examples: [crate README](../../../../../crates/sqlite-controller/README.md).
Standalone crate owns its lockfile and pins rusqlite0.38.0 with bundled SQLite.
`Controller::handle(Value)` and the Unix socket daemon share authentication,
authority/revision guards, commands, projections, receipts and events. The API
uses JSON envelopes, command project revisions and scoped idempotency; downstream
participants must never open this database. Error descriptions currently use
stable categories in text, not a versioned machine error-code enum.

Storage/authority contract:

- One lifetime OS file lock at the canonical DB pathname plus `.controller-lock`;
  persistent lock file, no stale PID reclamation. Every controller connection
  validates WAL, synchronous FULL, foreign keys; busy timeout is500ms. A mutex
  serializes commands; immediate short transactions end before any external effect.
- Two globally shared slots, unique live task attempts. Admission commits task,
  slot, attempt, immutable brief/base link, prepared start, grant, event and receipt
  together. Task abortion/cancel/unknown outcomes retain unresolved capacity.
  Observed exit is privileged and revokes grant/releases slot atomically.
- Operator is deployment-wide; leads and attempts are project scoped. Lead
  `RoleEpoch` and attempt `ScopeEpoch` are independent. Auth/current epoch precede
  receipt lookup. Idempotency tuple is project/principal/epoch/key; canonical JSON
  arguments include command kind, expected revision and all arguments.
- Canonical WorkRef uniqueness prevents alias tasks. Source WorkRef + exact source revision + computed SHA256 UTF-8 brief digest are
  pinned at task creation. Brief, event, decision and future candidate/evidence/
  acceptance records are append-only. Candidate identity includes commit/tree/base/
  report digest. Future commands must bind verification and acceptance to one
  exact candidate; physical landing must journal expected base and retained lineage.
- Messages retain sender/recipient/ack; questions retain answer/author; decisions
  retain author. Event sequence is global and reconnect cursors may contain gaps.
  Worker projections are scoped; event bodies redact grant tokens. Only operator
  `dispatch_context(attempt)` returns the retained current credential. `pending_operations`
  provides the durable local dispatch/reconciliation queue; notifications are hints.

Ticket02 should extend this deep seam with identity-bound execution and cleanup
records/commands and enforce reconciliation before dispatch. Never infer observed
termination from abort, cancel, failed operation, lease, missing acknowledgement,
or controller restart. `observe_attempt` currently accepts privileged fixture
observations; no supervisor or real execution guarantee is claimed. Ticket03 uses
supported Kata APIs and adds journaled synchronization; ticket04 adds immutable
candidate/evidence/acceptance commands and expected-base landing operations.
Ticket05 consumes the same command/event seam and durable projections. Candidate
storage tables are reserved; their admission and artifact-content validation are
not yet exposed and must be implemented/proven by their owning ticket.

Qualification limits: real on-disk local SQLite/process tests, no paid/live models;
no power-loss/storage-fault proof, network filesystem support, backup rollback
detection or supported restore. An unannounced manual older-DB replacement cannot
be detected here. No automatic schema import or restore is available. Normal
binary excludes fault injection; the explicit test feature alone enables the
pre-COMMIT marker/barrier used by the independently controlling kill test.

Evidence logs include behavioral RED/GREEN, final verification, and honestly
classified setup failures. The initial network Cargo fetch timed out on native
binding downloads and was terminated (status143); direct bounded crate downloads
populated Cargo's normal cache, verified by Cargo lockfile checksums on offline
build. `setup-compile.log` is an implementation compile failure (unsigned SQL
integers corrected), not behavioral RED. Source digests and final command/status
are recorded in `tested-source.json` and `verification.txt`.

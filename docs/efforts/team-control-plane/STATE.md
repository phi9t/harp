# State: durable team control plane

Status: implementing the approved baseline; the full goal remains active.
Repository base: `963f58b31e518a8162dfe719e386ef9d9843321f`.
Root worktree: `/data02/home/philip.yang/workspace/corenius-codex-reference-design`.
Branch: `codex/reference-design`. All changes remain uncommitted.

## Goal and authority

Make Corenius the executable reference for a persistent engineering team with
replaceable Codex/Claude leads and TraeCLI workers. Preserve assignments,
decisions and evidence through failures, prevent unsafe retries, and accept
only verified candidates. The complete scope is in [PLAN.md](PLAN.md) and
[SPEC.md](SPEC.md); kernel acceptance does not replace live demonstrations or
upgrade/restore readiness.

The owner [approved the recommended baseline](decisions/20261001-approved-baseline.md):
Rust broker/gateway and Temporal worker, PostgreSQL execution authority,
Git-approved planning, and supervised environments. Kata is
[deferred for the first kernel](decisions/20261001-kata-deferred-first-kernel.md).
jj remains an optional backend evaluation. No commit, push, PR or deployment
has been authorized. Paid/live probes need a concrete reviewed package and
separate scope/resource approval. No paid model or live Trae server has run.

## Accepted work

Tickets10–15 are accepted with real PostgreSQL behavior, independent Standards
and Spec reviews, and fresh root integration checks. The current exact source
boundary is [ticket14's joint accepted manifest](evidence/ticket14-accepted-manifest.json).
The latest root gate passed252 tests (138 Rust/114 Python) plus one intentionally ignored child
helper exercised by its parent. Formatting and Clippy with warnings denied
passed. This proves the local model-free kernel components, not live-provider
or multi-host recovery.

Ticket17's Kata evaluation is accepted and deferred; its actual API, broker
mapping, upgrade and restore evidence is retained. Ticket18's final conditional
closure waits for kernel16. jj0.44.0 has a bounded executable preflight, not
backend19 acceptance or adoption.

## Work in progress

- Ticket14 accepted: both independent round0 axes, separate shared-glue review,
  and fresh joint gate/fmt/strict Clippy passed. The joint187-file manifest SHA256
  is `ab1370237832f6ac7a105a8c235f556aa3e2b758adab5be1c0e6b8a32306c844`.
  Runtime/Supervisor assembly and validated Execution handles remain16.
- Ticket16 has a completed [read-only integration preflight](research/kernel-integration-preflight.md)
  in its isolated future worktree. All100 accepted baseline files and33 reviewed
  inputs were hash-verified. It maps missing runtime/workflow assembly, operator
  controls and role-neutral physical Execution allocation. Ticket14 is now resolved; ticket16 is claimed by `codex-kernel` and delegated
  to `/root/artifacts11`. It verified the100-file prior baseline and reseeded/hash-verified all187
  joint accepted files before implementation. Focused actualCLI/realPG status and Lead workspace allocation have observed
  red-green evidence. Additive0007 and role-neutral allocation are in progress;
  Actual SDK registered-driver dispatch and a bounded worker/candidate/Verifier
  capsule/Owner-accept fixture pass. Physical pause, uncertain-runtime capacity,
  exact issuing-credential expiry and trusted wait-status observations also have
  retained behavioral red-green evidence. Trusted monitor comparison now rejects
  fabricated success in a focused actual SDK fixture; pre-Verifier report bypass
  and missing-policy allocation negatives pass. Actual SDK Lead planning reads
  sealed captured input; actual Worker continuation imports retained candidate
  bytes, reads new captured context and produces src/resumed. Fresh focused
  lifecycle then submits the post-resume candidate and checks both deliverables
  in independent Verifier capsules through registered SDK episode dispatch before
  explicit Owner acceptance. Focused model-free SDK crash-after-effect takeover
  now proves persisted replacement ownership, same native IDs, actual replacement
  Activity observation and one invocation/effect. Public FIFO inventory rejection
  has retained bounded red/green evidence with unchanged authority. Nine marked
  inventory/process takeover negatives pass, preserving ownership and unknown
  capacity. Configured SDK recovery now passes actual Continue-As-New and
  unchanged-history replay, plus cached-transfer wrong-token/Worker/pause-revision
  negatives. This uses supported500ms sticky timeout; default10s is not a recovery
  SLA claim. Public live native reconciliation now passes missing/corrupt/symlink
  receipt and unauthorized caller controls, holding unknown capacity until the
  original receipt is restored without another Start or ownership transfer.
  All-role cumulative8-episode admissions and shared legacy/episode runtime-store
  exclusion pass focused checks. Genuine repair-budget lifecycle passes with distinct SDK repair candidates
  and measured independent failures; measured/unknown usage remains an active
  implementation gap. Managed sandbox cleanup passed fmt and strict offline
  Clippy for both crates; restored root access now permits sockets/Docker/bwrap.
  Exact13-path cleanup was reconciled only after all1336 pre/post hashes matched.
  Full crash/reconciliation matrix and remaining current-authority/context-integrity
  negatives and frozen independent review remain incomplete. No ticket16 frozen
  proof or independent review is available. The preflight grants no live launch authority.


The accepted supervisor preserves strict controller-loss and32-command concurrency tests.
It verifies original namespace PID1 while the workload is gated, then requires
proven finalized process-group exit or an independently verified namespace
handle through inventory/signaling. Namespace-link disappearance, leader-Z,
unreadable observations and cached booleans do not prove complete shutdown.
Unknown execution retains capacity. The
[PID1 source contract and bounded OS observations](research/pid1-finalization-contract.md)
are retained with hashes; the vendor-kernel distinction and untested failure
modes remain explicit. Raw PID observations also require the original boot and
observer PID frame.

## Next gates

Implement16's complete
assign/start/yield/resume/submit/verify/accept proof and operator controls.
The optional-backend evaluations, live Trae recovery, independent lead demos,
and upgrade/backup/restore tasks remain required by the full plan. The
[restore preflight](research/restore-inventory-preflight.md) identifies missing
independent post-backup inventory and restore-time retention/dispatch gates;
current restart recovery does not prove rollback recovery.

[Private Trae preparation](research/live-trae-private-entry-preflight.md) now
retains complete initialization/config/thread/job schemas. Actual store
isolation, synthetic-provider routing, spend enforcement and runtime recovery
remain unexercised. The [provider documentation review](research/live-trae-provider-configuration.md)
found legacy YAML custom endpoints, but no version-matched TOML routing contract
or documented hard ordinary-turn spend bound. Equivalent source searches stop
until new primary evidence is available. This preparation grants no launch approval.

## Handoff discipline

This is a manual state record, not a committed `effort.py dehydrate` snapshot.
The tool auto-commits, so this manual record preserves reviewed changes explicitly.
The owner has now [authorized gated landing across worktrees](decisions/20261001-landing-authority.md);
commits and landing still require the applicable review and exact-tip checks.
One writer per worktree remains enforced. Reused agent contexts are a documented tool-cap
fallback; reviewers remain separate from each candidate's author.

The latest work accepted supervisor15 with both review axes, separate shared-glue
review and fresh252-test joint integration evidence, and preserved16's read-only
integration preflight. Mailbox14 is accepted with independent reviews and joint evidence. Earlier chronological records are preserved in
[implementation-history.md](implementation-history.md). The
[completion audit](completion-audit.md) records all remaining requirements.

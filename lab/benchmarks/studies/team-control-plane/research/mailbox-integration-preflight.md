# Ticket14 integration preflight

Read Corenius `963f58b31e518a8162dfe719e386ef9d9843321f` plus the accepted
10–12 content manifest. Separate read-only review inspected actual auth/domain,
admission, storage, HTTP, artifact and gateway interfaces. This records gaps
and a handoff, not implementation acceptance; ticket13 remains a prerequisite.

## Existing seams and required additions

- Auth binds principal/role/mission/task/generation/epoch, but Lead has no
  replacement generation. Add mission lead generation/session fencing without
  revoking unrelated workers or giving a new conversation Owner credentials.
- Add durable typed messages and distinct presentation, acknowledgment and
  resolution receipts. Derive sender from authentication. Presentation binds
  recipient, attempt/episode, bounded input digest and exact runtime turn.
  Acknowledgment requires the addressed principal and valid presentation;
  neither transport write nor acknowledgment resolves a question.
- Peer communication grants access to the addressed message and approved
  artifact references, not the peer's full task or history. Current task-scoped
  Actor scope alone cannot represent this disclosure boundary.
- Journal delivery before gateway I/O. Bind episode and expected turn; completion
  races leave pending delivery, lost reply leaves uncertainty. Confirmed runtime
  effect alone cannot manufacture presentation or model acknowledgment.
- Atomically persist checkpoint, questions/dependencies, external Execution
  handles, closed episode and durable wait. An answer racing yield satisfies the
  wait or leaves a wakeup. Yield does not prove background effects stopped.
- Extend accepted13's transition/wakeup seam; do not add another scheduler.
  Finite admission must charge planning, verification, continuation and lead
  replacement episodes. Preserve existing implementer/verifier/acceptance
  separation and avoid broader tool credentials as a scheduling workaround.
- Construct bounded paginated mission context from canonical revisions,
  decisions, pending questions, uncertain/running attempts, candidate/report
  references, constraints and budgets. Retain critical attention and durable
  references when truncating. Existing mission read model is insufficient;
  existing unbounded task history is not a bounded replacement context.
- Expose one authorization boundary through HTTP/MCP, including existing typed
  artifact service. MCP must not impersonate agents with an Owner token or expose
  unrestricted runtime mutations. Authorized decisions precede dependent dispatch;
  approvals bind normalized arguments, principal, scope, expiry and consumption.

These are implementation implications of the existing SPEC, not new execution
or architectural authority. Shared changes necessarily touch auth, additive
schema/read models, transport routing and accepted13 wakeup/transition APIs.
Gateway delivery correlation requires an explicit extension to accepted12;
its current owner-only mutation guard must not silently widen.

## Behavioral handoff

Prove duplicate send/presentation/wakeup, forged sender/role/recipient scope,
stale worker/verifier epoch/generation including replay, surviving old Lead after
replacement, steering-completion/lost-reply races, answers before/during/after
yield, conversation loss with unanswered questions, blocked workers with zero
polling model turns, peer proposals requiring authoritative amendment, and
identical HTTP/MCP submission/verification permissions. No tests or providers
were run for this read-only preparation. Actual behavior remains to implement.

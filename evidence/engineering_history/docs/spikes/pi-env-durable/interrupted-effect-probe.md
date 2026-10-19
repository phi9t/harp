# Pi recovery after a lost controller reply

## Executive summary

The bounded experiment passed. We killed a Pi worker after Corenius recorded
its progress event but before Pi received the reply. After restart, Pi sent
the same saved request. Corenius returned the original receipt. The event count
stayed at one.

We then repeated the test with revoked authority. Corenius refused the saved
request, even though it had a receipt. The event count stayed at one, and the
Attempt still occupied its capacity slot.

This supports a narrow integration path: use Pi to resume reasoning, and keep
effect authority in Corenius. Pi must save the complete request before it sends
the request. Every retry must pass through the controller. A local receipt cache
must not bypass that check.

## Scope and source identities

This technical section is an STE-style draft. Full-rule and dictionary compliance
remain unverified.

- Pi commit: `ae92585d3b3e5f1e4b123d14a34314d826d8d9f5`.
- Corenius baseline: `d7f674829e9844d011869b711d64fcf55313f7f1`.
- Pi uses its real Node SQLite store and a faux model provider.
- Corenius uses its real Rust controller and Unix socket service.
- The effect is a transactional `report_progress` event in a disposable database.

The fixture does not start a supervised workload. It does not use Pi Env,
Insula, a live model, or an external service.

## Test sequence

1. Create a project, task, and admitted Attempt with private fixture credentials.
2. Start a Pi conversation with one controller-backed tool.
3. Save the complete controller command with `api.memo` before the socket call.
4. Send the command through a proxy to the controller.
5. Hold the successful controller reply in the proxy.
6. Send `SIGKILL` to the Pi process.
7. For the revocation case, cancel the Attempt through the controller.
8. Open the same Pi store in a new process.
9. Recover the same tool task and send its saved command to the controller.
10. Compare request identity, receipts, event counts, and occupied slots.

The recovery process proposes a different request key and revision. The stored
memo overrides this proposal. This checks persistence across actual process death.

## Observed results

| Case | Controller response | Progress events | Occupied slots |
| --- | --- | --- | --- |
| Lost reply, authority retained | Original receipt | 1 before and after recovery | 1 |
| Lost reply, authority revoked | Refusal: `authentication: revoked attempt authority` | 1 before and after recovery | 1 |
| Same key, changed payload | Idempotency mismatch | No additional event | 1 |
| Fresh key, current revision | New receipt | 2 after this negative control | 1 |

Both recovery cases retain the original Pi task ID, tool invocation ID, and
complete command. The successful replay returns the same receipt as the intercepted
reply. These comparisons use actual values and hashes in the saved results.

The fresh-key control demonstrates the failure that request persistence prevents.
A new key identifies a new command. The controller can accept that command and
record a second event.

## Adapter requirements

**Inference from this experiment:** A controller-backed Pi tool can use safe replay
for this command if it preserves the original command and checks authority again.

Save the command kind, arguments, revision, and idempotency key before transmission.
Use the saved values on recovery. Do not replace the saved revision with a current
snapshot revision. The controller checks an identical receipt before revision conflict.

Send every recovery request to the controller. Authentication and authorization
must precede receipt reuse. A persisted successful reply does not grant current
authority.

Keep tool failure separate from conversation completion. A faux model can finish
the conversation after the tool reports an error. That completion does not prove
that the controller accepted the effect.

## Evidence and reproduction

- [Runnable probe and prerequisites](../../../experiments/pi-env-durable/README.md).
- [TypeScript worker](../../../experiments/pi-env-durable/effect-worker.ts).
- [Python crash orchestrator](../../../experiments/pi-env-durable/interrupted-effect-probe.py).
- [Independent review](../../efforts/pi-env-durable/evidence/interrupted-effect-probe/review.md).
- [Sanitized results](../../efforts/pi-env-durable/evidence/interrupted-effect-probe/result.json).

The author run and a fresh integration-worktree run passed. Independent source
review passed. All six child processes and both proxy threads stopped. Private
credential-bearing fixture directories were removed.

## Limits and next decision

This experiment tests a ledger mutation. Its event and receipt share one controller
transaction. It does not test a shell command whose external effect can occur
before its receipt is saved.

It also does not test controller death, host power loss, concurrent Pi owners,
upgrades, or changed tool definitions. Revocation prevents replay acceptance; it
does not undo the event recorded before revocation.

The result justifies a subordinate-worker design discussion. A production adapter
still needs independently supervised execution, uncertain-effect reconciliation,
runtime identity checks, and acceptance evidence. The next useful probe would kill
the reasoning worker while a controller-owned execution remains active.

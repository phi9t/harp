# Independent scoped Responses relay review

Two Spec findings remain. The original relay and tests are unchanged. No new scoped Standards finding was identified; this does **not** qualify the existing baseline or resolve its reported Standards/branch654 TEST timing flaw.

## Standards

Reviewed the sealed files against the supplied task, Constitution, workflow and bounded skill policy. No hard Standards breach or useful smell finding was added in this scope. This was a sealed-file review rather than a complete branch/repository gate. The task's earlier C1/C2 scoped PASS and open TEST timing flaw remain outside this review.

## Spec/security

1. **S1 — Important: body/header content enters evidence.** [Relay lines 140–141](inputs/sqlite_responses_relay.py#L140) record caller-controlled `model` and `session_id` directly, despite the privacy comment at lines 165–166. The local UnixHTTP privacy assertion confirmed **2/2** runtime canaries retained: one in an object-valued model body field, one in the session header. Raw values are withheld and not retained in reports/logs. A caller can persist sensitive or arbitrary request content to host evidence. Root should route removal/hashing and model validation to the sole author.
2. **S2 — Moderate: late stream EOF bypasses deadline classification.** [Relay lines 150–155](inputs/sqlite_responses_relay.py#L150) check time before reading and mark EOF complete without another check. With a synthetic 0.02-second limit and 0.06-second EOF delay, the request took **0.060844s** and still recorded `stream_complete=true`, with no failure class. The production 180-second bound is checked between reads; additional waiting is inferred from its unchanged 30-second socket timeout, not measured live. Root may adjudicate this as demo hardening; no strict wall-cap PASS is claimed.

The other **14/15** independent checks passed: fixed TLS constructor/origin/path, routing negatives, caller credential/target stripping, no redirect forwarding, private Unix socket, valid synthetic HTTP/SSE framing, request/header/response limits, six upstream slots, generic error privacy, and read-only synthetic auth/no refresh. Details and limits are in `report.json`.

## Evidence and actual exits

| Check | Actual child exit | Result |
|---|---:|---|
| Sealed author test | 0 | 1 test passed |
| Independent synthetic suite | 1 | 15 tests: 14 passed, S1 failed, 0 errors |
| Focused delayed-EOF check | 1 | S2 assertion failed |

Wrapper exit 0 is only successful evidence capture; the table records the actual test-child exits. Pre-run source/test/harness hashes, initial safe logs, resources and reaped process identities are retained under `evidence/`. All three test children exited and were reaped; each synthetic check ended with zero non-main threads; no sockets or temporary entries remain.

The sealed **actual preflight** records HTTP200, `response.completed`, response status `completed`, and exit 0. This reviewer did not repeat it. It proves the reported observed provider connection only. Later **actual Corenius/team model proof** is absent from these inputs and remains the owner's work. Installed binary hash `8bf204b36a2f6dd0dab73aa2f639892e67ef9ac8befccb4a05b1496ebf25c479` is a pin, not source/ABI assurance; the reported failed official source fetch remains a limitation.

## Pins and scope

- Relay SHA256: `91903578eba38db8ad56ef40e811deebcd511e4017e2a052e4b332b0a69e3a96`
- Author test SHA256: `148377c32ef64de7381c4b73a559595d3424d408254314de1985f7a9a3bcf402`
- Preflight SHA256: `929d00231c0b5d1f68a23a809c459ae2264d69581aacfc3f84c030e8ea1655ed`
- Snapshot source: `a70ddebc27b8971370af93f6be961fcf4f483990`; writer `01a10a2b-1c44-7e11-aed9-11720d14abb0`; input hashes unchanged: `true`.

No real host-auth read, model/provider request, source repair, further agent, timer service, broad repository/trace inspection, or TLS/auth-refresh/catalog/root-Insula expansion occurred. TLS was constructor-verified, not handshake-tested. Header bounds concern parsed fields; strict wire-size/global handler-resource guarantees are unqualified. SSE validity was tested with a fixture, not arbitrary provider output. Root remains sole correction and adjudication owner.

Standards: **0 new scoped findings**; existing baseline remains unqualified. Spec: **2 confirmed findings**, with S1 the higher-priority privacy issue.

# Narrow v2 privacy-repair addendum

**S1 is independently resolved** on source `c0346486b683a043d51028e95900ee6b66b2f065`. No S1 review blocker remains for the ordinary disposable kickoff. Root owns launch and adjudication.

The minimal diff removes all caller `model` and `session_id` values from retained evidence. Fresh local UnixHTTP privacy, routing-negative and credential-stripping checks passed **3/3, actual child exit 0**; **0/2** runtime privacy canaries were retained. The revised author tests passed **2/2, actual child exit 0**. No canary values were printed or retained in reports/logs; no real host-auth read or provider/model request occurred.

**S2 remains OPEN hardening and prototype-nonblocking per root adjudication.** Its original delayed-EOF failure is preserved. The stream implementation is unchanged; the 180-second value is checked between reads, while socket reads remain bounded at 30 seconds. No strict 180-second wall-cap PASS is claimed, and no new stream/fault gate was added.

Standards: 0 new findings for this minimal diff. The previously reported Standards/branch654 TEST timing flaw and broader baseline remain unqualified. Supplied actual preflight remains separate from these synthetic checks; later actual Corenius/team model proof is outside this handoff. Failed official source-fetch/ABI assurance remains a limit.

- Relay SHA256: `539856b74af3ac3ed1d38f3f18f883f6e3864572ddff679a788c172bcb13712c`
- Test SHA256: `c0f22e117fdf40e80c1a00ed885e7be2e46167df2795f6a08fdb7ce3b049d396`
- Author source files were hashed before/after copying and after the checks; copied inputs were hashed before/after execution. All pins match.
- Both v2 test children exited and were reaped; zero synthetic worker threads and zero private temporary entries remain.
- Original `report.md`, `report.json`, initial safe logs, failure results and `evidence-index.json` remain byte-for-byte unchanged. The original index verifies all 23 indexed files.

`addendum-v2.json`, `evidence-v2/`, and `evidence-index-v2.json` hold the exact pins, actual exits/resources and new evidence. The original failures are retained as source-linked v1 observations, not rewritten as a v2 PASS.

# Standards review

Reviewed the separate frozen candidate against supplied source-read commit `acc1c233`; static/hash review only. No shared writes, Git mutations, auditor invocation, or physical launch.

## Documented-standard finding

**Important — rejection behaviors lack visible preimplementation RED.** `docs/agents/skill-orchestration.md:63–66` requires “Every behavior slice must then show red before implementation and green afterward”; `docs/agents/agentic-engineering.md:104–107` repeats this gate. `regression.py:54,60,71,82` skips all four new rejection controls in legacy mode. Bound `red.stderr.log` records one portability failure, one preservation pass, and four skips; `green.stderr.log` records six passes. The positive portability regression therefore has RED/GREEN evidence, but the new expected-origin/candidate input behaviors have GREEN only. `pre-green-freeze.json` asserts RED preceded implementation without supplying failing executions for those controls. Retain this narrow chronological evidence gap; do not rewrite evidence or describe all behavior slices as test-first qualified.

## Nonblocking heuristic

**Possible Speculative Generality:** `regression.py:16,37` creates `CALLS = []` and appends events, but nothing consumes it; command evidence is already persisted. Remove the unused in-memory accumulator in a later behavior-preserving cleanup if this fixture evolves. This is judgement, not policy.

## Verified limits

Hash checks verified all **37 follow-up files** (32 indexed plus five index/seal/final-validation files) and **181 unchanged original files**. Candidate seal/index/script/report hashes match the supplied pins. `portable_audit.py:39–53,68–78,138–155` separates sealed historical origin from retained byte location; the hash-pinned old module has only standard-library imports, function definitions, and guarded main, so helper reuse does not run its main/controller/Git. New result markers (`:181–184`) and report limits grant no acceptance authority or physical admission. Retained legacy failure at old `audit.py:217` remains failed; original PARTIAL, historical budgets, and earlier gaps remain unchanged. Static review cannot independently attest every historical resource absence or prove chronology beyond retained records. Reviewer output stays in its own directory, consistent with `AGENTS.md` one-writer rule.

One documented finding; one nonblocking heuristic. No tooling-enforced details reviewed.

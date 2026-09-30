# Results

Snapshot from 2026-09-30. Every number below was measured by the harness in this repository; nothing is quoted from another source.
Agent under test: `traecli exec -m GPT-5.5` with `model_reasoning_effort=xhigh`, running inside the `agent` capsule (network on,
only its workspace writable, traecli config bound read-only). Verification: pristine tests, `toolchains` capsule, network off.
Raw run directories are not committed (`bench/results/` is gitignored); the aggregated rows are in `bench/results-archive/`.

## 1. Offline validation (the benchmark itself)

`python3 bench/harness/validate.py`: **41 tasks, 0 failing** (301 s). Every task: the start state fails, the reference solution
applies and passes, the verdict is deterministic, deleted or stubbed tests do not help, an empty patch fails, tampering with protected
sources is undone. Multi tasks: every ticket is necessary and all landing orders (up to 24) end in a pass with the declared number of
conflicts. Repeat runs of start and solved states were 100% consistent on the tasks we repeated (see the effort decisions).

## 2. Live pilot: single-ticket tasks, solo (`pilot-1`)

34 tasks x 3 repeats = 102 runs, five in parallel. **102 of 102 passed.** Per task the 95% Wilson interval for 3 of 3 is [0.44, 1.00].
Median agent time 69 s (min 34 s, max 431 s, on `b-l4-wrapper-macro`). Tokens: median 2.5k output and 99k input per run; totals
about 21M input and 0.33M output; 147 agent-minutes. One hard run was diffed against the starting workspace: the agent made the intended
one-line fix.

**Reading:** at this model strength the single-ticket tasks do not discriminate. They are useful as regression checks for the harness
and for weaker agents, not for separating GPT-5.5 configurations. The S/M/L labels are authors' guesses and were not calibrated.

## 3. Solo versus fleet on small multi-ticket tasks (`fleet-1`, 39 of 40 runs when this was written)

Interleaved solo and fleet runs; fleet workers run tickets in parallel; a conflicting ticket is redone from the new main.

| Task | Cond. | n | Pass | Median wall (s) | Conflicts / redos | Median output tokens |
| --- | --- | --- | --- | --- | --- | --- |
| b-l5-fused-op (3 tickets) | solo | 5 | 5/5 | 140 | 0 / 0 | 5.2k |
| b-l5-fused-op | fleet:3 | 5 | 5/5 | 274 | 5 / 5 | 28.2k |
| m01-disjoint-utils (2) | solo | 5 | 5/5 | 73 | 0 / 0 | 2.7k |
| m01-disjoint-utils | fleet:2 | 5 | 5/5 | 58 | 0 / 0 | 2.7k |
| m02-shared-interface (2) | solo | 5 | 5/5 | 73 | 0 / 0 | 3.6k |
| m02-shared-interface | fleet:2 | 5 | 5/5 | 78 | 0 / 0 | 5.2k |
| m03-same-file-append (2) | solo | 5 | 4/5 | 46 | 0 / 0 | 2.3k |
| m03-same-file-append | fleet:2 | 4 | 4/4 | 70 | 4 / 4 | 2.8k |

- Correctness: equal within noise (n = 4 or 5). The single solo failure (`m03`) is one run and is not evidence of a difference.
- Wall time: fleet was not faster except on the fully disjoint `m01` (58 s vs 73 s, one small task, not tested for significance). Per-ticket
  agent startup and reading dominate at this size, and each conflict forces a full redo of one ticket.
- Cost: `b-l5-fused-op` fleet used about 5 times the output tokens of solo. Conflicts were exactly the designed one per run.
- The contamination column in the raw runs is not reported: before it was fixed it counted build artifacts (blade, cmake) as contamination.

## 4. Solo versus fleet on the hard multi-ticket tasks (`fleet-hard`, 4 of 18 runs when this was written)

Three tasks with five tickets each. Only one pair per task was complete at snapshot time:

| Task | Solo wall / out tokens | Fleet:5 wall / out tokens | Fleet conflicts |
| --- | --- | --- | --- |
| mh1-py-inventory | 496 s / 34k | 423 s / 81k | 1 |
| mh2-rs-scheduler | 264 s / 40k | 293 s / 44k | 1 |

One pair each: no conclusion. The final table will be added to this file when the experiment finishes (`bench/results-archive/`).

## What this benchmark can and cannot say today

- It can say whether a change to the harness, capsule or verifiers breaks anything (41 validated tasks, deterministic verdicts).
- It can measure time, tokens, conflicts and redos of solo versus fleet coordination on the multi-ticket tasks.
- It cannot yet say that fleet is better or worse on correctness for GPT-5.5: everything passes. Harder or larger tasks are needed
  for that. Cost in dollars is not measured, only tokens.
- Results depend on one model configuration, few repeats, API latency that varied while two experiments ran at the same time, and a
  capsule whose base is the host `/usr`.

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

## 3. Solo versus fleet on small multi-ticket tasks (`fleet-1`, 40 of 40 runs, complete)

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
| m03-same-file-append | fleet:2 | 5 | 5/5 | 70 | 5 / 5 | 2.5k |

- Correctness: equal within noise (n = 5 per cell). The single solo failure (`m03`) is one run and is not evidence of a difference.
- Wall time: fleet was not faster except on the fully disjoint `m01` (58 s vs 73 s, one small task, not tested for significance). Per-ticket
  agent startup and reading dominate at this size, and each conflict forces a full redo of one ticket.
- Cost: `b-l5-fused-op` fleet used about 5 times the output tokens of solo. Conflicts were exactly the designed one per run.
- The contamination column in the raw runs is not reported: before it was fixed it counted build artifacts (blade, cmake) as contamination, and paths were not recorded then, so the 3 files counted on `m02` fleet cannot be attributed.

## 4. Solo versus fleet on the hard multi-ticket tasks (`fleet-hard`, 18 of 18 runs, complete)

Three tasks with five tickets each, three interleaved rounds (solo, then fleet:5, per task per round). Wall time and output tokens are medians of 3.

| Task | Cond. | Pass | Wall (s), each run | Median out. tokens | Conflicts / redos (3 runs) |
| --- | --- | --- | --- | --- | --- |
| mh1-py-inventory | solo | 3/3 | 496, 469, 435 | 29.6k | 0 / 0 |
| mh1-py-inventory | fleet:5 | 3/3 | 423, 371, 385 | 81.0k | 3 / 3 |
| mh2-rs-scheduler | solo | 3/3 | 264, 429, 229 | 40.0k | 0 / 0 |
| mh2-rs-scheduler | fleet:5 | 3/3 | 293, 307, 290 | 47.1k | 3 / 3 |
| mh3-cxx-engine | solo | **1/3** | 518, 319, 389 | 34.6k | 0 / 0 |
| mh3-cxx-engine | fleet:5 | **0/3** | 275, 461, 226 | 60.7k | 1 / 1 |

- `mh3-cxx-engine` is the first task here that is not saturated: solo passed 1 of 3, fleet 0 of 3. With n = 3 per cell (95% intervals [0.06, 0.79] and [0.00, 0.56]) this shows the task
  discriminates against the agent, not that fleet is worse. The verifier output of these fleet runs was not saved, so **why** they failed is not known from the data; the runner now saves it.
- `mh1`: fleet was faster in all three rounds (371 to 423 s against 435 to 496 s solo) and passed every run, at about 2.7 times the output tokens; one conflict per run, recovered by redo, as designed.
- `mh2`: no wall-time difference worth reporting (solo ranged 229 to 429 s); tokens about 1.2 times.
- Contamination: `mh2` fleet runs each changed `Cargo.lock` (cargo rewriting a tracked lock file) and one `mh3` fleet run changed `CMakeLists.txt`, a file outside its ticket's allowed paths.
  Only the allowed paths are landed, so these did not reach the verified tree; the paths were recorded.
- Timing caveat: the tmux harness tests and other live workloads ran at the same time on this machine, so wall times carry API-latency noise.

## What this benchmark can and cannot say today

- It can say whether a change to the harness, capsule or verifiers breaks anything (41 validated tasks, deterministic verdicts).
- It can measure time, tokens, conflicts and redos of solo versus fleet coordination on the multi-ticket tasks.
- It cannot yet say whether fleet is better or worse on correctness for GPT-5.5: 40 of 40 small multi-ticket runs passed, and on the three hard tasks only
  `mh3-cxx-engine` separated (solo 1/3, fleet 0/3, n = 3). More repeats and more tasks like `mh3` are needed. Cost in dollars is not measured, only tokens.
- Results depend on one model configuration, few repeats, API latency that varied while two experiments ran at the same time, and a
  capsule whose base is the host `/usr`.

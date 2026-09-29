# Collaboration-mechanism simulation: results

Produced by `python3 experiments/collab/sim.py --scenario all --seeds 20` (980 runs, 7 strategies x 7 scenarios x 20 seeds, about 15 s on a 216-core host) and verified by `python3 -m pytest experiments/collab/test_sim.py` (89 passed). Every number below is copied from that sweep's output.

## What was simulated

Each run creates a throwaway git repository. Every merge, rebase step and text-conflict decision is made by real git (`git merge-tree --write-tree --merge-base`, the operation `git rebase` performs per commit; a test cross-checks it against a real `git rebase`). Time is virtual minutes in a discrete-event loop. Agents work through 5 tasks each (edit about 25 min, gate about 8 min on 2 shared gate slots). Strategies: A shared tree, B worktree+rebase+ff under a global lock (milano.scm), C lead/follower landing (serial lead verification, follower ff every 15 min), D merge queue with 3 speculative batches of 3 and bisection, E file leases with optimistic ff-only integration, F patch stream (8-min window, batches of up to 8, 60% of text conflicts auto-resolved, one gate per batch). E2 is an extra variant I added: E plus B's integration lock.

Columns: thr/h = tasks landed per virtual hour (mean over seeds). lat = ready-to-landed minutes, pooled over seeds. cycle p50 = task start-to-landed, which includes lease waits that "lat" does not see. wasted gate-min = slot minutes of runs that did not end in a landing (flakes, failures, cancelled or invalidated speculation), mean per run. text confl = real git conflicts per run. sem->main = hidden semantic pairs that were both present and unreconciled in some mainline tree, summed over seeds (number of seeds affected in brackets); must be 0. red-main min = minutes mainline was red under the pure oracle. blocked = per-agent minutes not editing or reworking. contam = gate runs that saw someone else's uncommitted change. bad runs = seeds violating any of: all tasks landed, no semantic conflict on mainline, only gated tips landed, only fast-forwards, no lost work.

## Tables

### low_contention  (8 agents x 5 tasks, files=60 zipf=0.0 p_sem=0.00 flake=0.00, seeds=20)
| strategy | thr/h | lat p50 | lat p95 | cycle p50 | wasted gate-min | text confl | sem->main (runs) | red-main min | blocked min/agent | contam | bad runs |
|---|---|---|---|---|---|---|---|---|---|---|---|
| A_shared_tree | 9.1 | 8 | 26 | 33 | 298 | 0.0 | 0 (0) | 221 | 75 | 46.4 | 20 |
| B_worktree_ff | 6.8 | 33 | 54 | 58 | 0 | 2.2 | 0 (0) | 0 | 163 | 0.0 | 0 |
| C_lead_follower | 6.4 | 113 | 223 | 138 | 0 | 4.5 | 0 (0) | 0 | 173 | 0.0 | 0 |
| D_merge_queue | 11.7 | 15 | 39 | 39 | 0 | 3.2 | 0 (0) | 0 | 14 | 0.0 | 0 |
| E_leases | 3.0 | 56 | 276 | 107 | 1128 | 0.0 | 0 (0) | 0 | 557 | 0.0 | 0 |
| E2_leases_locked | 6.5 | 24 | 41 | 55 | 0 | 0.0 | 0 (0) | 0 | 167 | 0.0 | 0 |
| F_patch_stream | 11.6 | 17 | 24 | 40 | 0 | 3.1 | 0 (0) | 0 | 16 | 0.0 | 0 |

### hotspot  (8 agents x 5 tasks, files=12 zipf=1.3 p_sem=0.05 flake=0.00, seeds=20)
| strategy | thr/h | lat p50 | lat p95 | cycle p50 | wasted gate-min | text confl | sem->main (runs) | red-main min | blocked min/agent | contam | bad runs |
|---|---|---|---|---|---|---|---|---|---|---|---|
| A_shared_tree | 9.0 | 10 | 30 | 33 | 322 | 0.0 | 2 (1) | 212 | 79 | 47.5 | 20 |
| B_worktree_ff | 6.1 | 28 | 88 | 53 | 14 | 20.4 | 0 (0) | 0 | 152 | 0.0 | 0 |
| C_lead_follower | 5.8 | 108 | 312 | 135 | 14 | 52.5 | 0 (0) | 0 | 160 | 0.0 | 0 |
| D_merge_queue | 9.0 | 36 | 143 | 61 | 25 | 40.4 | 0 (0) | 0 | 31 | 0.0 | 0 |
| E_leases | 2.7 | 9 | 26 | 121 | 114 | 0.0 | 0 (0) | 0 | 580 | 0.0 | 0 |
| E2_leases_locked | 2.8 | 9 | 18 | 115 | 14 | 0.0 | 0 (0) | 0 | 561 | 0.0 | 0 |
| F_patch_stream | 10.5 | 21 | 101 | 51 | 37 | 29.8 | 0 (0) | 0 | 35 | 0.0 | 0 |

### high_semantic  (8 agents x 5 tasks, files=30 zipf=0.5 p_sem=0.35 flake=0.00, seeds=20)
| strategy | thr/h | lat p50 | lat p95 | cycle p50 | wasted gate-min | text confl | sem->main (runs) | red-main min | blocked min/agent | contam | bad runs |
|---|---|---|---|---|---|---|---|---|---|---|---|
| A_shared_tree | 7.8 | 17 | 41 | 40 | 437 | 0.0 | 4 (4) | 200 | 107 | 60.3 | 20 |
| B_worktree_ff | 5.3 | 38 | 109 | 64 | 93 | 6.0 | 0 (0) | 0 | 224 | 0.0 | 0 |
| C_lead_follower | 5.0 | 133 | 372 | 160 | 95 | 13.9 | 0 (0) | 0 | 247 | 0.0 | 0 |
| D_merge_queue | 6.1 | 83 | 333 | 108 | 254 | 17.9 | 0 (0) | 0 | 180 | 0.0 | 0 |
| E_leases | 3.7 | 22 | 101 | 78 | 544 | 0.0 | 0 (0) | 0 | 363 | 0.0 | 0 |
| E2_leases_locked | 4.4 | 19 | 52 | 62 | 92 | 0.0 | 0 (0) | 0 | 276 | 0.0 | 0 |
| F_patch_stream | 4.8 | 128 | 502 | 155 | 311 | 22.2 | 0 (0) | 0 | 303 | 0.0 | 0 |

### flaky_gate  (8 agents x 5 tasks, files=30 zipf=0.8 p_sem=0.05 flake=0.25, seeds=20)
| strategy | thr/h | lat p50 | lat p95 | cycle p50 | wasted gate-min | text confl | sem->main (runs) | red-main min | blocked min/agent | contam | bad runs |
|---|---|---|---|---|---|---|---|---|---|---|---|
| A_shared_tree | 7.4 | 13 | 38 | 39 | 451 | 0.0 | 4 (4) | 244 | 118 | 62.1 | 20 |
| B_worktree_ff | 5.0 | 45 | 118 | 70 | 117 | 9.0 | 0 (0) | 0 | 249 | 0.0 | 0 |
| C_lead_follower | 4.8 | 147 | 380 | 173 | 117 | 21.1 | 0 (0) | 0 | 252 | 0.0 | 0 |
| D_merge_queue | 9.2 | 36 | 139 | 61 | 89 | 17.8 | 0 (0) | 0 | 49 | 0.0 | 0 |
| E_leases | 3.2 | 16 | 91 | 91 | 553 | 0.0 | 0 (0) | 0 | 467 | 0.0 | 0 |
| E2_leases_locked | 3.7 | 15 | 44 | 70 | 118 | 0.0 | 0 (0) | 0 | 377 | 0.0 | 0 |
| F_patch_stream | 9.5 | 27 | 139 | 56 | 80 | 15.4 | 0 (0) | 0 | 56 | 0.0 | 0 |

### agents_2  (2 agents x 5 tasks, files=30 zipf=0.8 p_sem=0.10 flake=0.03, seeds=20)
| strategy | thr/h | lat p50 | lat p95 | cycle p50 | wasted gate-min | text confl | sem->main (runs) | red-main min | blocked min/agent | contam | bad runs |
|---|---|---|---|---|---|---|---|---|---|---|---|
| A_shared_tree | 2.9 | 9 | 27 | 34 | 58 | 0.0 | 0 (0) | 64 | 58 | 11.8 | 20 |
| B_worktree_ff | 3.0 | 9 | 26 | 31 | 7 | 0.1 | 0 (0) | 0 | 50 | 0.0 | 0 |
| C_lead_follower | 3.5 | 19 | 48 | 42 | 7 | 0.4 | 0 (0) | 0 | 20 | 0.0 | 0 |
| D_merge_queue | 3.7 | 10 | 38 | 34 | 7 | 0.4 | 0 (0) | 0 | 12 | 0.0 | 0 |
| E_leases | 2.7 | 9 | 28 | 35 | 28 | 0.0 | 0 (0) | 0 | 70 | 0.0 | 0 |
| E2_leases_locked | 2.8 | 9 | 26 | 34 | 7 | 0.0 | 0 (0) | 0 | 63 | 0.0 | 0 |
| F_patch_stream | 3.5 | 17 | 51 | 39 | 9 | 0.6 | 0 (0) | 0 | 20 | 0.0 | 0 |

### baseline_8  (8 agents x 5 tasks, files=30 zipf=0.8 p_sem=0.10 flake=0.03, seeds=20)
| strategy | thr/h | lat p50 | lat p95 | cycle p50 | wasted gate-min | text confl | sem->main (runs) | red-main min | blocked min/agent | contam | bad runs |
|---|---|---|---|---|---|---|---|---|---|---|---|
| A_shared_tree | 8.3 | 11 | 33 | 37 | 359 | 0.0 | 4 (4) | 224 | 89 | 53.0 | 20 |
| B_worktree_ff | 6.0 | 34 | 91 | 59 | 36 | 7.5 | 0 (0) | 0 | 184 | 0.0 | 0 |
| C_lead_follower | 5.7 | 112 | 291 | 139 | 36 | 17.3 | 0 (0) | 0 | 197 | 0.0 | 0 |
| D_merge_queue | 9.5 | 29 | 124 | 57 | 68 | 15.7 | 0 (0) | 0 | 49 | 0.0 | 0 |
| E_leases | 3.7 | 14 | 65 | 65 | 366 | 0.0 | 0 (0) | 0 | 360 | 0.0 | 0 |
| E2_leases_locked | 4.3 | 11 | 32 | 56 | 36 | 0.0 | 0 (0) | 0 | 287 | 0.0 | 0 |
| F_patch_stream | 9.3 | 26 | 159 | 56 | 81 | 14.8 | 0 (0) | 0 | 73 | 0.0 | 0 |

### agents_32  (32 agents x 5 tasks, files=30 zipf=0.8 p_sem=0.10 flake=0.03, seeds=20)
| strategy | thr/h | lat p50 | lat p95 | cycle p50 | wasted gate-min | text confl | sem->main (runs) | red-main min | blocked min/agent | contam | bad runs |
|---|---|---|---|---|---|---|---|---|---|---|---|
| A_shared_tree | 12.5 | 50 | 123 | 76 | 1439 | 0.0 | 4 (3) | 608 | 492 | 180.8 | 20 |
| B_worktree_ff | 6.4 | 134 | 594 | 161 | 159 | 140.2 | 0 (0) | 0 | 974 | 0.0 | 0 |
| C_lead_follower | 5.6 | 729 | 1544 | 755 | 159 | 324.8 | 0 (0) | 0 | 1216 | 0.0 | 0 |
| D_merge_queue | 9.4 | 334 | 883 | 360 | 297 | 325.2 | 0 (0) | 0 | 600 | 0.0 | 0 |
| E_leases | 3.0 | 16 | 262 | 250 | 2389 | 0.0 | 0 (0) | 0 | 2193 | 0.0 | 0 |
| E2_leases_locked | 4.0 | 12 | 54 | 111 | 161 | 0.0 | 0 (0) | 0 | 1453 | 0.0 | 0 |
| F_patch_stream | 11.4 | 225 | 786 | 247 | 374 | 188.8 | 0 (0) | 0 | 549 | 0.0 | 0 |

[980 runs in 15s]

## Invariant results

B, C, D, E, E2 and F had zero bad runs in every scenario: no semantic conflict ever reached mainline, every mainline tip was a commit that had been gated, all updates were fast-forwards, no work was lost. Only A (shared tree) fails, as expected: semantic conflicts reached mainline in 4 of 20 seeds in the baseline, high-semantic and flaky scenarios (3 of 20 at 32 agents, 1 of 20 in the hotspot scenario), mainline was red for 64 to 608 minutes per run (about 200 to 250 in the 8-agent scenarios), and its "bad runs" column is 20 of 20 everywhere because it commits ungated work by construction. One bug in my own first draft was caught by the completeness invariant and fixed before the numbers above: authors forgot earlier reconciliations when reworking, which livelocked runs under high semantic conflict rates.

## Interpretation

In this model, gate capacity dominates. At 8 or more agents offered load far exceeds the 2 gate slots, so throughput mostly measures how many slots a strategy keeps busy: B and C never use more than one at a time (lock and serial lead), and that alone caps them near 6 tasks/h, whereas D and F batch and speculate (roughly 9 to 12 tasks/h, except in the high-semantic and 2-agent scenarios). The D-versus-F gap is within seed noise (baseline_8: D 9.5 with sd 1.1, F 9.3 with sd 1.5; agents_32: D 9.4 sd 0.8, F 11.4 sd 2.1), so I would not rank them. Batching and speculation pay off most when semantic conflicts are rare and lose much of their edge when they are common: at 35% semantic pairs D falls to 6.1/h (B: 5.3) and F to 4.8/h, below B, with p95 latency of 333 and 502 min against B's 109, because bisection and restacking burn gate minutes (D wasted 254, F 311, B 93). B is the robust default: zero wasted gate time at low contention, the best p95 among the strategies without leases in the high-semantic case (109 min), and equal-best in the 2-agent case, though it is slow at 32 agents (p95 594 min, 140 text conflicts per run) and never exceeds about 7 tasks/h. C is dominated by B here: the serial lead and 15-minute follower add latency (p50 108 to 147 min in the 8-agent scenarios, 729 at 32 agents) for no throughput gain, though agents block less at 2 agents (20 vs 50 minutes). Naive leases (E) perform worst because holding a lease through a gate whose result is invalidated by every other landing produces retry storms (1128 wasted gate-min in low contention); E2 (leases plus the lock) removes the waste and has the best p95 latency and cycle time among the safe strategies at 32 agents (54 min ready-to-landed, 111 min cycle p50), but agents wait on leases (1453 blocked minutes per agent) and its throughput (4.0/h) is below B's (6.4/h), so leases trade throughput for zero text conflicts and short post-edit latency. The shared tree looks fast (12.5 tasks/h at 32 agents) only because it lands unverified work, and its throughput number should be read as invalid, not competitive.

## Where the model is weak

- Gate cost is nearly independent of batch size (only +3% per extra change), which is the assumption most favorable to D and F; a suite whose cost scales with change size would erase much of their advantage.
- Semantic conflicts are hidden random pairs, always fixed by one bounded rework (30% of edit time), always detected by the gate (no false passes), and agents distinguish flakes from real failures for free. Real semantic conflicts cluster by module and are harder to attribute.
- Edits are append-only tokens, so the shared tree can never clobber work and merge conflicts are cheap to resolve; real conflict resolution is harder. F's 60% auto-resolve rate is an invented parameter.
- The lead in C and the coordinators in D and F are dedicated non-agent roles costing only 0.5 min per item; C's 15-minute follower period and D's batch size and depth (K=3, S=3) are untuned guesses. D and F share one batching engine and differ only in parameters and auto-resolve, so they are not independent designs.
- Closed-loop workload of 40 to 160 tasks with a start-up transient and a drain tail; offered load is far above gate capacity at 8+ agents, so results describe a saturated regime. The 2-agent case is the only lightly loaded one, and there the safe strategies span only 2.7 to 3.7 tasks/h.
- No agent-side costs for context switching, no cost of merging conflicts for humans, no failures of the coordinator, and only one repository with 30 to 60 files. Contamination is defined only for a mutable working tree, so it is 0 by construction for B to F. Latencies are pooled across seeds, and the seed count (20) leaves sd around 10 to 20% on D and F throughput.

# SDD ledger - plan: compact-handoff-daemon-bridge

| Check | Finding | Ruling |
|---|---|---|
| Task set | Daemon bridge benchmark and implementation share bridge/server files; docs consume verification evidence | Ruling: implement red benchmark first, then minimal code, then docs - preserves TDD and avoids stale doc claims - cost if wrong is a small rework of benchmark shape |
| Repo shape | /private-source/home/operator/workspace/soma is not a git repo | Ruling: use scratch ledger and direct file review, not git worktree SDD scripts - cost if wrong is weaker branch-level diff packaging |

Task red: daemon bridge benchmark created and run.
- Command: benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh .scratch/emacs-gym/red/bridge-daemon
- Result: exit 1 as expected; daemon external observe and direct-call rejection passed; request.status failed with unsupported_operation.

Task review: blocking findings addressed.
- Finding 1: daemon transport exposed generic command.dispatch surface. Fixed with `soma-gym-bridge-operation-allowlist`, daemon profile allows only observe/status/direct-call, and daemon benchmark now includes command-dispatch rejection.
- Finding 2: cleanup could wait forever after failed emacsclient shutdown. Fixed with bounded shutdown polling plus TERM/KILL fallback.
- Earlier green verification before the async-status slice: daemon benchmark
  passed with 4/4 scenarios; aggregate selected-rootfs benchmark passed with
  6/6 families and 35/35 scenarios.

## 2026-09-28T06:41:19Z daemon async status/await slice

Task red: added `daemon-async-status-await` to `benchmarks/emacs-gym/bridge/scripts/daemon-client.py`.
- Command: `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh .scratch/emacs-gym/red/bridge-daemon-async`
- Result: exit 1 as expected; existing four daemon scenarios passed, async scenario failed because `bridge.defer_echo` and `request.await` were unauthorized/unsupported.

Task green: added minimal timer-backed daemon async transport support.
- Files: `benchmarks/emacs-gym/elisp/soma-gym-bridge.el`, `benchmarks/emacs-gym/bridge/elisp/daemon-start.el`, `benchmarks/emacs-gym/bridge/scripts/daemon-client.py`
- Behavior: `bridge.defer_echo` stores a running request and completes it via daemon timer; `request.status` observes in-flight and terminal records; `request.await` waits boundedly for terminal state.
- Focused verification: `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh benchmarks/emacs-gym/bridge/traces/latest-daemon` passed 5/5 scenarios with Insula checks.
- Ruling: keep this as a synthetic bridge lifecycle probe instead of routing real Magit/Jupyter work through the daemon in this slice - it proves request lifecycle without changing package benchmark authority - cost if wrong is one extra follow-up to replace the synthetic operation with package-backed async tasks.

Final verification for async-status slice:
- Focused daemon command: `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh benchmarks/emacs-gym/bridge/traces/latest-daemon`
- Focused result: run id `20260928T064253Z-2372309`, 5/5 daemon scenarios as expected, Insula checks passed.
- Aggregate command: `scripts/insula/run-emacs-gym-benchmarks-rootfs.sh benchmarks/emacs-gym/traces/latest`
- Aggregate result: run id `20260928T064313Z-2374738`, 6/6 benchmark families passed, 36/36 scenarios as expected.

## 2026-09-28T06:55:19Z checked buffer read/edit/save slice

Task red: added `buffer-read-purity`, `checked-edit-save`, and `stale-edit-rejected` to `benchmarks/emacs-gym/bridge/elisp/run-benchmark.el`.
- Initial red attempts caught test syntax/shape mistakes; after fixing the scenario harness, the red command `benchmarks/emacs-gym/bridge/scripts/run-benchmark.sh .scratch/emacs-gym/red/bridge-buffer-edit` exited nonzero with 8 scenarios executed, 5 as expected, and the three new scenarios failing on unsupported `buffer.read`, `buffer.replace_range`, and `buffer.save`.

Task green: added minimal checked buffer operations to `benchmarks/emacs-gym/elisp/soma-gym-bridge.el`.
- `buffer.read` returns a one-based half-open text range without changing point, mark-active, selected window, modified state, or `chars_tick`.
- `buffer.replace_range` validates live buffer, buffer name, text owner, visited file, `chars_tick`, point/mark/narrowing/minibuffer if supplied, range bounds, and expected range text before mutating under `atomic-change-group`. Failed preconditions return `conflict` and do not mutate.
- `buffer.save` validates the same buffer identity/revision/file preconditions before saving the visited file.
- Focused bridge verification: `benchmarks/emacs-gym/bridge/scripts/run-benchmark.sh benchmarks/emacs-gym/bridge/traces/latest` passed 8/8 scenarios with Insula checks.
- Regression daemon verification: `benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh benchmarks/emacs-gym/bridge/traces/latest-daemon` passed 5/5 scenarios with Insula checks.
- Aggregate verification: `scripts/insula/run-emacs-gym-benchmarks-rootfs.sh benchmarks/emacs-gym/traces/latest` passed 6/6 benchmark families and 39/39 scenarios, run id `20260928T065147Z-2464669`.

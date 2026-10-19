# 0005: Implement The Selective Magit Staging Benchmark

Status: Batch checked-bridge, terminal, and daemon stale-recovery evidence captured
Type: Follow-up task
Depends on: 0001 Emacs/Doom Internals Atlas; 0002 Soma Insula Contract Spike; 0004 Checked Elisp Bridge

## Intent

Create the first externally meaningful benchmark task: selective Magit staging
inside the pinned Soma Insula.

## Requirements

- Generate disposable Git fixtures with multiple hunks, unrelated staged and
  unstaged changes, stale views, and active-region variants.
- Exercise Magit section selection, hunk/file/list staging branches,
  `this-command`, refresh, and post-stage hooks when operational fidelity is in
  scope.
- Grade exact index contents, worktree preservation, unrelated staged state,
  refresh/view generation, and command-context evidence.

## Verification

- A scripted correct policy repeatedly passes.
- A deliberately incorrect policy that stages too much or bypasses required
  command context fails even if it reports success.

## Captured Evidence

Batch-safe selected-rootfs benchmark:

```sh
benchmarks/emacs-gym/magit/scripts/run-benchmark.sh
```

Latest trace:

```text
benchmarks/emacs-gym/magit/traces/latest/summary.md
```

Result: 7 scenarios total, 7 scenarios as expected, and Insula runtime checks
passing under `selected-container-rootfs`. The three correct policies now enter
Magit through the first synchronous checked bridge command-dispatch slice.

Covered correct policies:

- Selected Magit hunk staging.
- Selected Magit file-section staging.
- Active-region Magit file-section staging.

Covered bad policies:

- Raw `git add -u` bypasses Magit and stages too much.
- Direct bridge-wrapper call to `magit-stage-1` produces the requested index
  effect while missing Magit command-context/post-stage evidence.
- Selecting the whole unstaged list stages unrelated changes.
- Acting on a stale Magit view after an external edit stages a change that was
  not part of the observed target state.

Live daemon selected-rootfs benchmark:

```sh
benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh
```

Latest aggregate trace:

```text
benchmarks/emacs-gym/traces/latest/bridge-daemon/19-daemon-magit-stale-view-recovery.json
```

Result: 28 daemon bridge scenarios total, 28 scenarios as expected, and Insula
runtime checks passing in aggregate run `20260929T040533Z-3771476`. The Magit
stale-recovery scenario reads model-facing `magit.observe_sections` evidence,
mutates the worktree externally, refreshes the Magit status buffer, rejects the
old repository-fingerprint preconditions before mutation, re-observes sections,
reselects the refreshed intended hunk, and stages it through checked
`command.dispatch` while preserving the external hunk, unrelated unstaged debug
change, and unrelated staged file.

## Remaining Work

- Add richer daemon package-operation variants beyond the first hunk, file,
  Transient suffix, prompt, and stale-recovery paths.
- Add repeated-run stress and randomized fixture variants before treating this
  as a sealed model benchmark.

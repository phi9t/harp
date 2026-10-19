# 0004: Implement The Checked Elisp Bridge With Command-Context Tests

Status: Batch plus live daemon transport, first async await, and episode finish captured; richer async and PTY remain
Type: Follow-up task
Depends on: 0001 Emacs/Doom Internals Atlas; 0002 Soma Insula Contract Spike

## Intent

Implement the first Elisp bridge around the Atlas command-dispatch and
state-ownership contracts.

## Requirements

- Expose separate operations for passive observation, derived inspection, active
  diagnostic, buffer-local edit, visible display/select, command dispatch, and
  terminal input.
- Enforce buffer identity, text-owner identity, relevant revisions, narrowing,
  selected window, point/region, minibuffer/transient state, and package handle
  preconditions as appropriate for each operation.
- Return operation states: `accepted`, `running`, `succeeded`, `failed`,
  `waiting_for_input`, and `outcome_unknown`.
- Include direct-call, `call-interactively`, command-dispatch, and PTY tests.

## Verification

- A bridge wrapper cannot claim Magit staging fidelity unless command identity
  and post-stage hook behavior are proven.
- Stale FIM/edit acceptance is rejected after human edits.

## Captured Evidence

First synchronous batch-safe bridge slice:

```text
benchmarks/emacs-gym/elisp/soma-gym-bridge.el
```

Selected-rootfs benchmark using that slice:

```sh
benchmarks/emacs-gym/magit/scripts/run-benchmark.sh
```

Latest Magit run records `command.dispatch` bridge results for each action.
Correct hunk, file, and active-region file staging scenarios report bridge
state `succeeded`, state sequence `accepted -> running -> succeeded`, route
`command-execute`, `this_command = magit-stage`, and passed precondition checks
for live target buffer, allowlisted command, selected-window buffer, character
revision, point, and minibuffer state.

Raw `git add -u` and direct `magit-stage-1` wrapper scenarios report bridge
state `failed` because they bypass checked command dispatch. List-scope and
stale-view scenarios can still enter the bridge successfully, but fail the
semantic grader for staging too much or acting on stale observed state.

First model-facing request-envelope smoke:

```sh
benchmarks/emacs-gym/bridge/scripts/run-benchmark.sh
```

The selected-rootfs bridge benchmark reports 16 scenarios total and 16 as
expected. It exercises request IDs, terminal request status storage,
`emacs.observe_buffer`, `command.dispatch`, duplicate request rejection,
unchecked direct-call rejection, checked `buffer.read`, checked
`buffer.replace_range`, checked `buffer.save`, stale-edit conflict rejection,
selected-window command drift rejection, point edit drift rejection,
narrowed/mark-active/read-only checked-edit rejection, checked
`completion.accept`, stale completion rejection, `episode.finish` final
evidence validation, and a JSONL Unix-domain socket
round trip inside the selected `source-built-emacs-v8` Insula. The
command-dispatch scenario still routes through the existing checked
`command-execute` bridge and records the expected `accepted -> running ->
succeeded` state sequence.

First live daemon/external-client bridge smoke:

```sh
benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark.sh
```

The selected-rootfs daemon bridge benchmark reports 28 scenarios total and 28 as
expected. It starts source-built Emacs as a named daemon inside the Insula,
starts the JSONL bridge socket in that daemon at
`/project/run/soma-gym-bridge-daemon.sock`, then drives the bridge from a
separate rootfs-owned Python client process. It verifies external
`emacs.observe_buffer`, rootfs-owned `emacsclient --tty` terminal-frame
attachment through rootfs-owned `tmux`, `emacs.observe_frames`, stored terminal
status retrieval through
`request.status`, command-dispatch rejection under the daemon transport
profile, direct-call rejection, real tmux-delivered terminal key dispatch that
switches the client frame to the target buffer, and one timer-backed
daemon-owned in-flight request whose initial submission returns `running`,
whose status is observable while in flight, and whose terminal success is
observed through `request.await` without resubmission. It also verifies
`request.await` timeout as `outcome_unknown` followed by later reconciliation of
the original request, an explicit `waiting_for_input` request/respond lifecycle,
checked point navigation in the selected terminal frame, a real terminal
minibuffer prompt opened by queued daemon `command.dispatch` and completed by
tmux input, real terminal-key Magit hunk staging from the daemon-owned terminal
frame, queued Magit hunk staging through daemon `command.dispatch`, Magit
Transient prompt-state observation/cancel from real terminal input, Magit
Transient Stage suffix execution with observed target-file scope, a
package-owned Magit `magit-stage-files` prompt opened by queued
`command.dispatch` and completed by tmux input with preserved `this-command` and
post-stage hook evidence, stale Magit view recovery through model-facing
`magit.observe_sections`, stale repository-fingerprint rejection, refreshed
section re-selection, and checked `magit-stage`, plus `episode.finish` failure
with `incomplete_episode` while a required request is still running and success after
all required request IDs reach the expected terminal states. It also verifies a
live Comint terminal-input scenario in which the daemon terminal frame submits
through `comint-send-input` to a rootfs-owned shell, preserves process identity,
updates Comint input markers, carries the Insula run id into subprocess output,
and performs the intended side effect exactly once. It also verifies a daemon
existing-kernel Jupyter eval-region path in which the daemon terminal frame is
switched to an associated source buffer, `jupyter.observe_source` captures
association and selected-region identity, `jupyter.eval_region` enters the
stored async request lifecycle, `request.status`/`request.await` observe
completion, `jupyter.query_state` records rootfs-owned runtime state, and the
grader checks preserved kernel token plus exactly-once selected-region
execution. It also verifies UI-rendered Jupyter display-update presentation
through `jupyter.eval_region_via_repl`: the source range is submitted through
the associated REPL client with `jupyter-repl-execute-cell`,
`jupyter.observe_display` finds the updated display-id range in
`*jupyter-repl[soma-daemon-display-update]*`, checked terminal point navigation
moves to that range, and tmux capture contains `SOMA_DISPLAY_STAGE:updated`.
It also verifies daemon stale `completion.accept` rejection after
terminal-frame human editing, with conflict evidence and preserved final buffer
text. It also verifies daemon Python LSP rename through real `lsp-rename`, with
`textDocument/rename` request evidence, lsp-mode workspace edit application,
rootfs-owned `pylsp`/Python identity, definition/caller/test-file updates, and
rootfs Python tests after the rename. It also verifies terminal/minibuffer
`command.dispatch -> lsp-rename`, including prompt observation, tmux-supplied
rename input, original-request terminal status polling, `command-execute` route
evidence, passive post-command source observation, and the same multi-file
Python test evidence. It also verifies `daemon-multi-async-status-await`, where
two daemon-owned async requests keep independent status/result identity and can
be awaited out of submission order. It also verifies
`daemon-package-async-jupyter-magit`, where a delayed existing-kernel Jupyter
eval-region request remains independently awaitable while queued Magit staging
runs and completes first, with no request/result swap and no duplicate kernel
execution. It also verifies `daemon-comint-terminal-interrupt-reconcile`, where
terminal-frame `C-c C-c` invokes `comint-interrupt-subjob`, preserves the
rootfs-owned shell process, and prevents the interrupted command's side effect.

The aggregate selected-rootfs runner now includes both bridge families plus
Org and Dired, and reports 8 benchmark families, 90 scenarios total, and 90
scenarios as expected under
`benchmarks/emacs-gym/traces/latest/summary.json`.

## Remaining Work

- Extend the live JSONL daemon bridge beyond the first queued terminal
  minibuffer, Magit, Transient prompt-state, Magit Stage suffix, Magit package
  prompt, Magit stale-view recovery, first Comint terminal input, first Jupyter
  eval-region request, first stale completion rejection, and first
  terminal/minibuffer LSP rename into richer package-specific prompt/infix flows
  and multi-operation async cases.
- Add full FIM model serving/ranking and richer package-specific prompt handling,
  then extend checked edits into package-specific operations.
- Add direct-call, `call-interactively`, command-dispatch, and PTY comparison
  tests through the bridge harness, not only Atlas probes.
- Add richer prompt/transient interaction tests and Doom/Evil-specific terminal
  bindings on top of the existing `emacsclient --tty` frame attachment and
  baseline key-dispatch proof.

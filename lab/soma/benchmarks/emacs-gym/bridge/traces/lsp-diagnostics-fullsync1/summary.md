# Bridge Daemon Transport Benchmark Summary

- Generated: 2026-09-29T04:04:52Z
- Benchmark: `bridge-daemon-transport`
- Run id: `20260929T040414Z-3718539`
- Daemon: `soma-gym-bridge-daemon`
- Socket: `/project/run/soma-gym-bridge-daemon.sock`
- Total scenarios: 28
- Scenarios as expected: 28
- Insula checks passed: `True`
- Passed: `True`

## daemon-external-observe

- Title: A process outside Emacs observes a buffer through the live daemon JSONL bridge
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-terminal-frame

- Title: A rootfs-owned emacsclient --tty creates a terminal frame attached to the live daemon
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-terminal-key-dispatch

- Title: Real terminal key input switches the daemon-owned emacsclient frame to the target buffer
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-request-status

- Title: A process outside Emacs can retrieve stored terminal request status from the live daemon
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-command-dispatch-rejected

- Title: The live daemon transport rejects command.dispatch for a non-allowlisted command
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-direct-call-rejected

- Title: The live daemon transport rejects unchecked direct-call operations
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-async-status-await

- Title: The live daemon transport exposes an in-flight request, then awaits its terminal status without resubmission
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-await-timeout-outcome-unknown

- Title: request.await reports outcome_unknown on timeout, then reconciles the original request without resubmission
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-waiting-for-input-response

- Title: A daemon request enters waiting_for_input and completes after an explicit external response
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-terminal-minibuffer-command

- Title: A queued daemon command opens a real terminal minibuffer and completes from tmux input
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-terminal-magit-key-dispatch

- Title: Real terminal key input stages one Magit hunk in the daemon terminal frame
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-magit-command-dispatch

- Title: A live daemon JSONL request stages one Magit hunk through checked command.dispatch
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-terminal-transient-magit-dispatch

- Title: Real terminal input opens and cancels Magit's Transient dispatch without changing fixture Git state
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-terminal-transient-magit-stage-suffix

- Title: Real terminal input opens Magit's Transient dispatch and executes its Stage suffix with the observed target-file scope
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-terminal-transient-magit-diff-infix

- Title: A queued daemon Magit diff Transient observes, toggles, and applies the --stat infix state from terminal input without mutating Git state
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-package-prompt-magit-stage-files

- Title: A queued daemon Magit command opens a package minibuffer prompt and stages the selected file from tmux input
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-episode-finish-lifecycle

- Title: episode.finish rejects a running dependency and succeeds after required daemon requests are terminal
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-comint-terminal-send-input

- Title: A live daemon terminal frame submits input through Comint to a rootfs-owned subprocess exactly once
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-magit-stale-view-recovery

- Title: The daemon bridge rejects stale Magit view preconditions, refreshes, re-observes sections, and stages only the refreshed intended hunk
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-jupyter-existing-kernel-eval-region

- Title: The live daemon bridge evaluates only the selected region on an existing rootfs-owned Jupyter kernel
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-jupyter-display-update

- Title: The live daemon bridge captures display_data and update_display_data from an existing rootfs-owned Jupyter kernel
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-stale-completion-rejected

- Title: The live daemon bridge rejects stale completion.accept after intervening terminal editing
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-lsp-python-rename-symbol

- Title: The live daemon bridge renames a Python symbol through lsp-mode and verifies the workspace edit
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-lsp-terminal-rename-symbol

- Title: A queued daemon command runs interactive lsp-rename and receives the new name from the terminal minibuffer
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-lsp-diagnostics-after-edit

- Title: A guarded daemon buffer edit creates live pyright diagnostics, repairs the source, saves it, and observes diagnostics clearing
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-multi-async-status-await

- Title: Two daemon-owned async requests keep independent status and can be awaited out of submission order
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-package-async-jupyter-magit

- Title: A daemon-owned Jupyter eval request keeps independent status while a queued Magit package command completes first
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

## daemon-comint-terminal-interrupt-reconcile

- Title: A live daemon terminal frame interrupts a partial Comint command and reconciles process state without the side effect
- Expected policy pass: `True`
- Policy passed grader: `True`
- As expected: `True`

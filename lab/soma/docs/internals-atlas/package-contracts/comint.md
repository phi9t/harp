# Comint Contract

## Owned State

- Process object and process buffer.
- Process mark separating historical output/input from pending input.
- Point, current input region, old input region, and history.
- `comint-last-input-start`, `comint-last-input-end`, and output markers.
- Completion state around input submission.

Primary local sources:

- `/opt/soma/build/emacs-30.2/lisp/comint.el:452-457`
- `/opt/soma/build/emacs-30.2/lisp/comint.el:655-656`
- `/opt/soma/build/emacs-30.2/lisp/comint.el:1886-2035`
- `/opt/soma/build/emacs-30.2/lisp/comint.el:2421-2451`
- `/opt/soma/build/emacs-30.2/lisp/comint.el:2593-2601`

## Required Session Context

- Current buffer must be the intended comint buffer.
- Process must be live and associated with the buffer.
- Process executable, cwd, environment, PATH, and tool resolution must match the
  selected Soma rootfs contract when the process is part of a benchmark task.
- Point relative to process mark determines whether current or old input is
  submitted.
- `comint-get-old-input`, `comint-input-autoexpand`,
  `comint-input-filter-functions`, and `comint-input-filter` can change the
  submitted payload and history behavior.
- Completion-in-region state may be torn down by send.

## Faithful Command Paths

- `comint-send-input` is the faithful submit path.
- Raw `process-send-string` skips history, markers, fields, and process-mark
  bookkeeping.

## Async Work

- Process filters insert output and may run while other Lisp is waiting.
- Output can move process mark and window point.
- Shell commands may launch additional tools whose rootfs identity also matters
  for benchmark evidence.

## Stale Handles

- Process object after restart.
- Process mark after output.
- Pending input after user edits.
- Current buffer if filters run in a different dynamic context.

## Completion Evidence

- Input bytes submitted once to the intended process.
- `comint-last-input-*` markers updated.
- Process mark advanced correctly.
- Output collected from the intended process and working directory.
- Process-side rootfs evidence includes cwd, run id, PATH, selected manifest,
  and required tool resolution.

## Negative Cases

- Sending old history when pending bottom input was intended.
- Raw process send without comint bookkeeping.
- Duplicate send after timeout.
- Output read from the wrong process buffer.
- Correct output from a shell or tool that resolved from host state rather than
  the selected rootfs.

Probe `comint-process-mark-input-boundary` runs a real local subprocess under
the Soma Insula and shows that the submitted command changes when point is
before versus after the process mark. This is now a first-class deep-dive
surface, not just a generic "process buffer" case.

Probe `comint-subprocess-rootfs-identity` records cwd, run id, PATH, and Python
resolution for a Comint-launched subprocess. Probe
`comint-long-lived-shell-cwd-tool-drift` keeps a shell alive across commands,
changes cwd to `/project/tmp`, and verifies that rootfs tool resolution and run
id survive after the state change. Probe
`comint-timeout-outcome-unknown-no-retry` submits one slow non-idempotent command,
classifies the short wait as `outcome_unknown`, then verifies the side effect
happened exactly once.

The first selected-rootfs Comint benchmark now lives at
`benchmarks/emacs-gym/comint/`. Its public runner is:

```sh
benchmarks/emacs-gym/comint/scripts/run-benchmark.sh
```

The current selected-rootfs aggregate trace lives under
`benchmarks/emacs-gym/traces/latest/comint/`.

The current trace summary reports 11 scenarios and 11 as expected under
`selected-container-rootfs`. The positive cases prove live input submission
through `comint-send-input`, process-mark reobservation after asynchronous
output, partial-output completion waiting, interrupt reconciliation, and
conservative timeout handling. The negative cases reject raw
`process-send-string`, submission from old history when bottom input was
intended, output ownership from a similarly named process buffer, stale
process-mark submission after asynchronous output, premature success on partial
output, and retrying a non-idempotent command after an unknown timeout outcome.

The daemon bridge now also covers two live terminal-frame Comint paths under the
same selected rootfs. `daemon-comint-terminal-send-input` submits shell input via
real tmux keys and verifies `comint-send-input`, process identity, input-marker,
run-id, and exactly-once side-effect evidence. The newer
`daemon-comint-terminal-interrupt-reconcile` scenario submits a long-running
command, waits for partial output, sends terminal `C-c C-c`, records
`comint-interrupt-subjob` through the buffer-local post-command hook, verifies
the rootfs-owned shell process survives, and checks that the interrupted
command's completion marker and side effect do not occur.

Remaining benchmark work should cover:

- Rootfs/tool resolution drift after PATH changes or process restarts.
- Richer PTY-backed prompts and additional terminal-frame process-control
  variants beyond submit and interrupt.

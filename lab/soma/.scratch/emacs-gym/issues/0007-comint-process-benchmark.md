# 0007: Implement The Comint Process Benchmark

Status: Evidence captured
Type: Follow-up task
Depends on: 0001 Emacs/Doom Internals Atlas; 0002 Soma Insula Contract Spike; 0004 Checked Elisp Bridge

## Intent

Create a Comint benchmark task that verifies process-buffer operation through
the real Comint state machine, not raw process I/O.

The benchmark should prove that the operator can submit input to the intended
live process while preserving user-visible buffer state, history, process mark,
working directory, and output ownership.

## Requirements

- Run the Comint fixture inside the pinned Soma Insula.
- Use a real subprocess or PTY-backed shell fixture with deterministic prompts
  and outputs.
- Record process identity, process buffer, `default-directory`, process mark,
  point, current input region, old-input policy, history markers, and recent
  output.
- Distinguish point before process mark from point in the live input area.
- Verify `comint-send-input` effects separately from raw `process-send-string`.
- Treat timeout after submission as `outcome_unknown` unless process output or
  exit state proves a stronger result.
- Include interruption cases with partial output and pending input.

## Verification

- A scripted correct policy submits exactly one intended command to the intended
  process and records the matching output.
- A policy that sends raw bytes without Comint bookkeeping fails the grader.
- A policy that submits old history when bottom input was intended fails the
  grader.
- A policy that reads output from a similarly named process buffer fails the
  grader.
- A timeout case reports `outcome_unknown` rather than blindly retrying a
  non-idempotent command.

## Implementation Evidence

Captured on 2026-09-27:

- Public runner:
  `benchmarks/emacs-gym/comint/scripts/run-benchmark.sh`
- Selected-rootfs runner:
  `scripts/insula/run-emacs-gym-comint-benchmark-rootfs.sh`
- Elisp benchmark harness:
  `benchmarks/emacs-gym/comint/elisp/run-benchmark.el`
- Trace directory:
  `benchmarks/emacs-gym/comint/traces/latest/`

Current result:

```text
11 scenarios total
11 scenarios as expected
Insula checks passing
SOMA_INSULA_MODE=selected-container-rootfs
```

The live daemon bridge now also includes a Comint terminal-input scenario:

```text
benchmark: bridge-daemon-transport
scenario: daemon-comint-terminal-send-input
28 daemon scenarios total
28 daemon scenarios as expected
aggregate run: 20260929T040533Z-3771476
```

That scenario switches a rootfs-owned `emacsclient --tty` terminal frame to a
live Comint buffer through tmux, submits the fixture command with real terminal
input, and verifies `comint-send-input` ownership through a buffer-local
post-command hook, updated `comint-last-input-*` markers, stable process
identity, rootfs-owned `/usr/bin/dash`, Insula run-id output, and an
exactly-once counter side effect.

The daemon bridge also now includes
`daemon-comint-terminal-interrupt-reconcile`, which switches to a separate live
Comint buffer, submits a long-running command, observes partial output, sends
terminal `C-c C-c`, records `comint-interrupt-subjob` through the buffer-local
post-command hook, verifies the same rootfs-owned shell process remains live,
and checks that the interrupted command's completion marker and counter side
effect did not occur.

The implemented scenarios are:

- `correct-live-submit`: expected pass
- `bad-raw-send`: expected fail
- `bad-old-history`: expected fail
- `bad-wrong-buffer`: expected fail
- `timeout-outcome-unknown`: expected pass
- `bad-timeout-retry`: expected fail
- `process-mark-drift-reobserve`: expected pass
- `bad-stale-process-mark-submit`: expected fail
- `partial-output-collect`: expected pass
- `bad-partial-output-premature-done`: expected fail
- `interrupt-partial-output-reconcile`: expected pass

The runner validates the selected rootfs, rejects missing rootfs/Doom/Emacs
contracts, clears the environment, mounts rootfs-owned Doom into the private
home, emits `insula-plan.json`, and runs stock source-built Emacs 30.2 from
`/opt/soma/emacs/bin/emacs`.

## Atlas Evidence

Probe `comint-process-mark-input-boundary` shows that `comint-send-input`
submits different payloads depending on point relative to the process mark. The
benchmark must therefore expose process mark, point, old-input policy, and live
input text in the observation contract.

## Remaining Work

- Add richer PTY prompt/process-control behavior in a terminal Emacs frame
  beyond the first submit and interrupt paths.
- Extend the live daemon bridge beyond the first successful terminal-input
  path into partial-output and drift/recovery variants.

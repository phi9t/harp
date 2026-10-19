# Command Dispatch And Operation Lifecycle

Emacs command execution is an interaction path, not just a function call.

## Dispatch Routes

Direct function call:

- Exercises explicit arguments and the body of a function.
- Does not necessarily set `this-command`, `last-command`,
  `current-prefix-arg`, command history, keymap context, minibuffer state, or
  post-command behavior.
- Safe candidates: passive buffer reads and small internal helpers whose contract
  is explicitly argument-driven.

`call-interactively`:

- Exercises the command's interactive argument acquisition.
- In batch probes, it acquired prefix arguments but still did not reproduce full
  command-loop state.
- Use only when the relevant behavior is interactive argument parsing and no
  command-loop hooks/keymaps/package command identity are part of the contract.

`command-execute` or keymap dispatch:

- `simple.el.gz:2753-2805` shows that `command-execute` normalizes prefix args,
  autoloads commands, and then calls `call-interactively` for ordinary commands.
- Batch keymap dispatch in probe `command-routes` set `this-command`, unlike the
  direct and non-key routes.
- Use for actions whose fidelity depends on command identity but not terminal
  byte decoding.

Real terminal input through a PTY:

- Required for Ghostty/SSH/tmux input decoding, terminal local maps, full
  pre-command/post-command behavior, recursive minibuffer interactions, and
  Transient prefix/infix/suffix lifetime.
- Required before freezing actions that claim keyboard fidelity.

## Effective Function

The function in the source file is not the whole operational unit. A faithful
operation is:

```text
upstream function definition
  + installed advice
  + effective hooks
  + buffer-local variables
  + dynamic caller context
  + package view state
  + command-loop/minibuffer/transient state
```

The Atlas requires command-sensitive adapters to capture at least:

- `this-command`
- `last-command`
- `real-this-command` when available
- `current-prefix-arg`
- selected window and current buffer
- active minibuffer/transient/completion state
- relevant buffer-local variables
- installed advice on the invoked command
- pre-command and post-command hook effects for terminal-path tests

## Which Bridge Operations May Use Which Route

Direct call is acceptable for:

- passive state reads using explicit buffer context
- non-display inspection that does not depend on interactive arguments
- pure helper functions used inside a larger checked action

`call-interactively` is acceptable for:

- commands whose only missing input is the interactive spec
- commands where batch probes show the same required effects as the desired
  route

Command dispatch is required for:

- actions depending on `this-command`, command remapping, keymaps, or command
  history
- Magit staging variants that participate in post-stage hooks
- completion accept paths whose provider exit function depends on command state

PTY/terminal tests are required for:

- minibuffer cancellation and recovery
- Transient prefix/infix/suffix handling
- Doom popup/window visible behavior
- real user key sequences in the target terminal profile
- any action profile that claims "operate Doom as the user would"

## Operation States

Every action result must be one of:

- `accepted`: the adapter accepted the request and recorded preconditions.
- `running`: work is still in progress.
- `succeeded`: completion evidence proves the intended state exists.
- `failed`: evidence proves the operation did not complete or violated a
  precondition.
- `waiting_for_input`: Emacs is at a prompt, transient, completion menu, or
  other user-visible continuation state.
- `outcome_unknown`: the adapter stopped waiting or lost the handle after a
  consequential request may have been submitted.

Timeout semantics:

- Timeout before an accepted request may be `failed`.
- Timeout after an LSP request, Jupyter execution, Git command, subprocess
  input, or filesystem operation is usually `outcome_unknown` until independent
  evidence proves success or failure.
- Efficiency rewards can apply only after policy/invariant checks and task
  completion evidence. Fewer actions cannot compensate for lost state.

## Probe Evidence

Probe `command-routes` recorded four routes under the development Insula:

- direct call: `this_command=nil`, `called_interactively=false`
- `call-interactively`: prefix argument acquired, command-loop state still nil
- `command-execute`: route recorded separately but batch did not emulate a real
  command loop
- keymap dispatch: `this_command=soma-probe-command`

Probe `magit-wrapper-this-command` showed that
`magit-run-post-stage-hook` responds to recognized Magit command identity and
does not run for a bridge wrapper command. This directly constrains Magit
adapter design.

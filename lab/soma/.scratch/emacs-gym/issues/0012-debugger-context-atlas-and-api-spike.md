# 0012: Add A Debugger Context Atlas And Safe API Spike

Status: Open
Type: Follow-up task
Depends on: 0001 Emacs/Doom Internals Atlas; 0011 Safe Emacs Agent API Profile

## Intent

Design the first debugger-context API for coding agents without giving them
unsafe execution-control authority by default.

The user-facing goal is to let an agent understand debugger context associated
with source buffers: active session, target process, stack, current frame,
locals, watches, breakpoints, and source mapping. Continue, step, restart, kill,
breakpoint mutation, and arbitrary expression evaluation require additional
backend-specific contracts.

## Requirements

- Choose the first backend to study, such as `dap-mode`, `gud`, or a narrower
  project-specific debugger path.
- Build an Atlas contract for debugger-owned state:
  - Emacs source buffer and display state
  - debugger frontend state
  - adapter/backend process
  - target process
  - thread/frame ids
  - breakpoint identity
  - source mapping
  - variable/object lifetimes
- Add executable probes for observation-only debugger state.
- Define `debug-inspect-v0` operations:
  - `debug.observe_session`
  - `debug.observe_stack`
  - `debug.select_frame`
  - `debug.observe_locals`
  - `debug.observe_breakpoints`
  - `debug.evaluate_watch_preview`
- Keep execution-control operations out of the first safe subset.

## Verification

- A debugger fixture runs entirely inside the selected rootfs and records
  adapter, target process, language runtime, source file, and ABI/toolchain
  identity.
- The observation API reports the same current source/frame/locals that the
  debugger frontend owns.
- Stale frame ids, restarted target processes, mismatched source mappings, and
  host-runtime fallback are rejected.
- The safe profile cannot continue, step, restart, kill, mutate breakpoints, or
  evaluate arbitrary expressions unless an explicit stronger capability is
  enabled.

## Notes

This issue exists because debugger context belongs in the long-term Emacs agent
API, but Soma does not yet have the source-grounded package contract needed to
expose debugger execution controls safely.

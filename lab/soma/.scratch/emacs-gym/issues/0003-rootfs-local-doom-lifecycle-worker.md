# 0003: Implement The Rootfs-Local Doom Lifecycle Worker

Status: Batch and repeated-teardown evidence captured; terminal-frame lifecycle probes remain
Type: Follow-up task
Depends on: 0001 Emacs/Doom Internals Atlas; 0002 Soma Insula Contract Spike

## Intent

Launch and inspect Doom Emacs inside the pinned Soma Insula with a lifecycle
worker that records cold, first-use, and warmed states.

The worker must prove rootfs-owned Emacs/Doom execution, not merely a host Doom
session entered through `bwrap`.

## Requirements

- Record Emacs binary/version/features, Doom core revision, user config hash,
  generated profile/module files, Straight package revisions, and
  profile-specific extra package/config files.
- Record selected rootfs manifest, dependency-closure id, ABI manifest id,
  Emacs dynamic-library surface, native-comp/libgccjit identity, tree-sitter
  grammar paths, package roots, and cache/runtime paths.
- Ensure Emacs, Doom package code, native-comp outputs, package caches, and
  helper executables resolve from the selected rootfs or declared sandbox-owned
  paths.
- Distinguish module declared, package installed, autoload available, feature
  loaded, mode active, and external service ready.
- Use the inspected non-Evil profile as the default unless another profile is
  explicitly selected.
- Run both batch-safe lifecycle probes and terminal-frame probes where command
  loop or display behavior is in scope.

## Verification

- The worker emits cold, warmed, cold-repeat-1, and warmed-repeat-1 lifecycle
  traces.
- The trace shows Corfu/Vertico, popup, TTY, Magit, Org+Jupyter, and LSP
  readiness or absence separately.
- The trace fails if Emacs or Doom package/native-comp/cache paths resolve from
  undeclared host state.

## Captured Evidence

Captured on 2026-09-28:

- `workers/emacs-doom-lifecycle/scripts/run-lifecycle.sh` delegates to the
  selected-rootfs runner and writes:
  - `workers/emacs-doom-lifecycle/traces/latest/insula-plan.json`
  - `workers/emacs-doom-lifecycle/traces/latest/cold.json`
  - `workers/emacs-doom-lifecycle/traces/latest/warmed.json`
  - `workers/emacs-doom-lifecycle/traces/latest/cold-repeat-1.json`
  - `workers/emacs-doom-lifecycle/traces/latest/warmed-repeat-1.json`
  - `workers/emacs-doom-lifecycle/traces/latest/summary.json`
  - `workers/emacs-doom-lifecycle/traces/latest/summary.md`
- The latest summary reports status `pass`; cold, warmed, cold-repeat-1, and
  warmed-repeat-1 child Emacs processes exit `0`, emit traces, and pass their
  checks.
- The focused repeated-teardown trace
  `workers/emacs-doom-lifecycle/traces/lifecycle-teardown-green2/summary.json`
  records `child_count=4`, `teardown_probe.status=pass`, and zero preexisting
  or remaining `SOMA_LIFECYCLE_CHILD_TOKEN` processes for each child launch.
- The worker launches source-built GNU Emacs `30.2` from
  `/opt/soma/emacs/bin/emacs` inside the selected CUDA-image-derived rootfs.
- Doom core and user config are rootfs-owned and mounted at
  `/project/home/.emacs.d` and `/project/home/.doom.d`.
- The runner seeds per-run writable Doom cache/state directories from the
  selected rootfs, then binds those over
  `/project/home/.emacs.d/.local/cache` and
  `/project/home/.emacs.d/.local/state`.
- The warmed child loads the generated profile through Doom, runs Doom startup,
  requires `magit`, `org`, `jupyter`, `lsp-mode`, `python`, `rust-mode`,
  `cc-mode`, and `sh-script`, activates Elisp, Org, Python, Rust, C++, and
  shell major modes, and inspects Jupyter kernelspecs through rootfs Python.
- Capability readiness is split across user-init declaration, loaded-profile
  Doom module state, package/autoload state, feature load, mode availability,
  and external tool readiness.
- Module activation is now read from Doom's loaded `doom-modules` table via
  `doom-module-active-p`, not from generated-profile text. This matters because
  the generated profile contains disabled-module documentation text such as
  `(:ui . popup)` even when Popup is not active.
- The selected rootfs profile enables `:tools magit lsp`,
  `:lang (org +jupyter) python rust cc sh`, and the listed Emacs/config
  modules. It does not enable Evil, Corfu, Vertico, Popup, or Doom's TTY module.

## Remaining Work

- Add terminal-frame lifecycle probes for real key dispatch, minibuffer,
  Transient, popup/display behavior, and Ghostty/SSH/tmux-facing UI effects.
- Select or materialize a fuller pinned Doom profile before claiming
  Corfu/Vertico/Popup/TTY readiness.

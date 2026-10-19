# Probe Contract

Each mechanism probe records:

- mechanism and hypothesis
- exact source revision and entry points
- minimal fixture
- predicted behavior
- observed trace
- independent state checks
- counterexample
- derived gym requirement

The current traces live under:

```text
probes/emacs-internals/traces/latest/
```

The authoritative runner is:

```sh
probes/emacs-internals/scripts/run-probes.sh
```

It enters the Soma Insula wrapper before running Emacs. The inner runner
`run-probes-batch.sh` refuses host execution unless
`SOMA_ALLOW_NON_INSULA_PROBES=1` is explicitly set.

Selected-rootfs runners first call
`scripts/insula/validate-emacs-rootfs-contract.py`. That preflight makes the
source-built Emacs claim part of the runtime gate: `/opt/soma/emacs/bin/emacs`,
the matching GNU source archive/tree, required configure flags, rootfs-owned
Doom, required Jupyter/LSP/Python/C++/Rust tools, dynamic loader, linked DSOs,
and forbidden host/Nix ABI prefixes must all check out before probes or
benchmarks run.

## Current Probe Suite

Current run: 36 probes, 36 passing, with Insula runtime checks passing under
`selected-container-rootfs` Insula. The probe rootfs wrapper now delegates plan
construction and execution to `scripts/insula/soma_insula.py run-command`; the
validated module-authority trace is
`probes/emacs-internals/traces/soma-insula-module-command-green1/`.

The rootfs-local Doom lifecycle worker is a separate gate:

```sh
workers/emacs-doom-lifecycle/scripts/run-lifecycle.sh
```

Current run: cold, warmed, cold-repeat-1, and warmed-repeat-1 child Emacs
processes pass under the same selected rootfs. The lifecycle wrapper now
delegates Doom env/profile state, writable Doom local cache/state mounts, and
plan construction to `scripts/insula/soma_insula.py run-command`; the validated
module-authority trace is
`workers/emacs-doom-lifecycle/traces/lifecycle-teardown-green2/`. The worker
records source-built Emacs identity, rootfs-owned Doom core and user config,
generated profile/cache files, loaded Doom module state, package/autoload state,
first-use feature loads, major-mode activation, Jupyter kernelspec discovery,
and per-child teardown probes based on unique
`SOMA_LIFECYCLE_CHILD_TOKEN` process-environment markers. Module activation is
read from Doom's loaded `doom-modules` state; generated-profile text is advisory
only because disabled module documentation can contain module-looking tokens.

The probes are:

1. `insula-rootfs-dependency-closure-contract`
2. `insula-plan-runtime-consistency`
3. `rootfs-mount-boundary-evidence`
4. `toolchain-resolution-closure-classification`
5. `toolchain-executable-provenance-matrix`
6. `toolchain-smoke-build-rootfs-identity`
7. `subprocess-environment-closure`
8. `native-comp-cache-closure`
9. `abi-surface-requires-rootfs-manifest`
10. `emacs-patch-provenance-contract`
11. `jupyter-kernel-rootfs-identity-proxy`
12. `jupyter-kernelspec-rootfs-contract`
13. `jupyter-real-kernel-rootfs-identity`
14. `lsp-server-rootfs-resolution-contract`
15. `lsp-stdio-initialize-rootfs-identity`
16. `comint-subprocess-rootfs-identity`
17. `comint-long-lived-shell-cwd-tool-drift`
18. `comint-timeout-outcome-unknown-no-retry`
19. `current-buffer-selected-window`
20. `command-routes`
21. `magit-wrapper-this-command`
22. `base-indirect-buffers`
23. `marker-movement`
24. `narrowing-region-without-text-change`
25. `minibuffer-transient-interruption`
26. `reentrant-process-output`
27. `doom-cold-warmed-state`
28. `org-source-edit-propagation`
29. `jupyter-delayed-reply-duplicate-retry`
30. `fim-stale-acceptance`
31. `observation-fontification-side-effect`
32. `capf-provider-exclusivity`
33. `capf-exit-function-acceptance`
34. `comint-process-mark-input-boundary`
35. `dired-undo-does-not-restore-files`
36. `with-timeout-wait-boundary`

## Interpretation Rules

- A passing batch-safe probe proves the mechanism under the inspected Emacs
  runtime and Insula contract.
- The rootfs/toolchain closure probes now prove a selected container-rootfs
  execution path. Development-minimal traces remain useful only when explicitly
  requested with `SOMA_USE_DEVELOPMENT_INSULA=1`.
- A batch-safe probe that mentions terminal limitations does not prove terminal
  fidelity. It creates a follow-up terminal-frame gate.
- `jupyter-real-kernel-rootfs-identity` proves one real Python kernel request in
  the selected rootfs. It does not replace existing-kernel attachment,
  incarnation/restart, package-level display-update presentation, or
  `emacs-jupyter` source-buffer probes.
- `lsp-stdio-initialize-rootfs-identity` proves language-server process startup
  and protocol initialization. It does not replace `lsp-mode` document sync,
  version checks, workspace edits, or diagnostics.
- Comint probes now cover process-mark input boundary, rootfs-owned subprocess
  identity, long-lived cwd/tool drift, and timeout-as-unknown behavior. PTY
  prompts and additional terminal-frame process-control variants remain separate
  gates; the daemon benchmark now covers first terminal-frame submit and
  interrupt paths.
- Probe traces are benchmark-design evidence. They are not learner-visible
  graders.
- The Doom lifecycle worker is construction evidence for cold/warmed profile
  behavior. It does not replace terminal-frame probes for key dispatch,
  minibuffers, Transient, or display behavior.
- The Comint benchmark is the next evidence layer, not another mechanism probe:
  `benchmarks/emacs-gym/comint/scripts/run-benchmark.sh` currently reports 11
  scenarios as expected under the selected rootfs.
- The Magit benchmark is the first package-specific benchmark layer:
  `benchmarks/emacs-gym/magit/scripts/run-benchmark.sh` currently reports 7
  scenarios as expected under the selected rootfs, including checked-bridge
  command dispatch for selective hunk, file, active-region file staging, and
  targeted bad-policy rejection.
- The Dired benchmark now covers stale same-path marks after external
  replacement: `benchmarks/emacs-gym/dired/scripts/run-benchmark.sh` reports 8
  scenarios as expected under the selected rootfs, including filesystem-signature
  revalidation before destructive operations.

## Next Probe Candidates

- Terminal-frame command dispatch: compare real PTY input through the selected
  rootfs Emacs against direct, interactive, and command-dispatch routes.
- `lsp-mode` rename/workspace-edit: start a real Emacs LSP client, rename across
  a fixture project, and verify document versions, workspace edits, diagnostics,
  and toolchain identity.
- Existing-kernel `emacs-jupyter` attach: associate a source buffer with an
  already-running kernel, verify kernel incarnation, execute one selected
  region, and reject wrong-kernel results.
- Additional Comint prompt/process-control variants beyond the first daemon
  terminal-frame submit and interrupt paths.
- Dired partial-failure, prompt, and terminal-frame variants beyond the current
  batch stale-mark and filesystem-ownership checks.
- Stock instrumentation coverage: compare command hooks, advice, change hooks,
  process filters, timers, redisplay probes, and module-level observations
  against independent outcomes to identify evidence that stock Emacs cannot
  expose faithfully.
- Patched Emacs observability delta: if stock coverage is insufficient, run the
  same fixtures under a source-pinned patched build and prove that added trace
  points improve evidence without changing command, package, process, display,
  or input semantics.

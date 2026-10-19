# Source Map

This source map records the local code and generated state inspected for the
current Internals Atlas pass. The authoritative runtime is the selected Soma
Insula rootfs, not the host Emacs or host Doom checkout.

## Environment Identity

- Insula mode: `selected-container-rootfs`
- Selected rootfs:
  `.scratch/emacs-gym/insula/rootfs-store/content/soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34`
- Base image:
  `ghcr.io/pytorch/pytorch:2.13.0-cuda13.2-cudnn9-runtime@sha256:7492928e093d67276716440161f694a0e4ea27796d599d64b23cb76cdd665e71`
- Materialization kind: `source-built-emacs-v8` / `docker-cuda-source-build`
- Rootfs recipe SHA256:
  `0bddf41e6539849c1d8e0a44954ebb3293f6eaa08081741c4adbf3c777fe808c`
- Docker image id:
  `sha256:9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d349d7bd6b6bb80`
- Docker image tag:
  `soma-emacs-rootfs:0bddf41e6539849c`
- Rootfs tree hash SHA256:
  `931b5d4338e0c0eb876f5b14f7babb71bc89b20e5b6a625d26799d16c0cb41dd`
- Emacs executable: `/opt/soma/emacs/bin/emacs`
- Emacsclient executable: `/opt/soma/emacs/bin/emacsclient`
- Emacs version: `30.2`
- System configuration: `x86_64-pc-linux-gnu`
- Build features:
  `ACL GMP GNUTLS LIBSELINUX LIBXML2 MODULES NATIVE_COMP NOTIFY INOTIFY PDUMPER SECCOMP SQLITE3 THREADS TREE_SITTER ZLIB`
- Emacs source archive: `/opt/soma/src/emacs-30.2.tar.xz`
- Emacs source archive SHA256:
  `b3f36f18a6dd2715713370166257de2fae01f9d38cfe878ced9b1e6ded5befd9`
- Emacs source SHA policy:
  checked against the validator-pinned GNU Emacs `30.2` release hash before
  extraction.
- Emacs source directory: `/opt/soma/build/emacs-30.2/`
- Emacs source directory exists: `t`
- Emacs executable SHA256:
  `824c1318464a98778d9f1677508e05af3dc5e94c34c63653145e079e5d26f6c2`
- Emacs configure flags:
  `--prefix=/opt/soma/emacs --with-native-compilation --with-tree-sitter --with-sqlite3 --without-x --without-sound --without-dbus --without-gconf --without-gsettings --without-toolkit-scroll-bars --without-xwidgets --without-mailutils`
- Emacs capability checks:
  `json-parse-string=t`, `native-comp-available-p=t`, `treesit-available-p=t`
- Emacs dynamic loader: `/lib64/ld-linux-x86-64.so.2`
- Required native tool ABI evidence:
  rootfs contract records ELF status, interpreter, and `ldd` output for native
  tools in the production tool inventory.
- Glibc owner: `container-image-rootfs`
- Installed Lisp directory: `/opt/soma/emacs/share/emacs/30.2/lisp/`
- Doom core: `/opt/soma/doom-home/.emacs.d`
- Doom core revision: `8b475fcb2574286d7f4413c22cc7902d80d21ced`
- Doom user config: `/opt/soma/doom-home/.doom.d`
- Doom profile cache:
  `/opt/soma/doom-home/.emacs.d/.local/cache/profiles.@.el`
- Generated module state:
  `/opt/soma/doom-home/.emacs.d/.local/etc/@/init.30.d/95-modules.auto.el`
- Rootfs contract validation:
  `scripts/insula/validate-emacs-rootfs-contract.py <selected-rootfs>` exits
  `0` with `"ok": true`, no errors, and no warnings.
- Rust toolchain:
  rootfs-owned `1.81.0` with `rust-src` installed for `rust-analyzer` sysroot
  discovery.
- Emacs source-build provenance note:
  `docs/insula/emacs-source-build.md`

The earlier host-Spack Atlas pass is superseded for production evidence. The
selected rootfs source-builds stock GNU Emacs and retains the matching source
tree, so C-level Atlas claims should cite `/opt/soma/build/emacs-30.2/` and the
rootfs contract rather than a host stage directory. Spack remains a possible
future materialization/provenance layer inside the same container-rootfs ABI
boundary; it is not required for the current source-built Emacs baseline.

## Package Revisions

The package source inspected under
`/opt/soma/doom-home/.emacs.d/.local/straight/repos/` in the selected rootfs:

- `magit`: `c800f79c2061621fde847f6a53129eca0e8da728`
- `org`: `89df5bf46ba214db44eea898cc7cacc0b27fd760`
- `lsp-mode`: `197f56bf7a38b564cc5dec26d66b5f494da39783`
- `jupyter`: `242fdc709ce0faa3b9ee81dcc48cfd791898e6b8`
- `transient`: `1f7039ef8d548d6fe858084fcbeae7588eba4190`
- `corfu`: not installed in the selected minimal profile
- `cape`: not installed in the selected minimal profile
- `tabbymacs`: not installed in the selected minimal profile
- `evil`: not installed in the selected minimal profile
- `forge`: not installed in the selected minimal profile

The selected rootfs uses a minimal rootfs-owned Doom profile:

```elisp
(doom!
 :emacs dired electric undo vc
 :tools eval lookup lsp magit
 :lang emacs-lisp (org +jupyter) python rust cc sh
 :config (default +bindings))
```

Host user Doom files and host `devx` package overlays are not part of
production benchmark evidence.

## Source Entry Points

Installed Emacs Lisp:

- `simple.el.gz:2753-2805`: `command-execute` dispatches commands and calls
  `call-interactively` for ordinary commands.
- `simple.el.gz:5529-5566`: `filter-buffer-substring` and
  `buffer-substring--filter` show that text extraction can be mediated by
  filtering hooks.
- `simple.el.gz:7242-7264`: redisplay hooks run with per-window buffer context.
- `subr.el.gz:4849`: `with-current-buffer` is the normal dynamic current-buffer
  binding primitive.
- `window.el.gz:7908-7927` and `window.el.gz:9086-9203`: `display-buffer` and
  `switch-to-buffer` are display/window operations, not pure buffer reads.
- `minibuffer.el.gz:2762-2781`: minibuffer cancellation unwinds recursive edit.
- `minibuffer.el.gz:2918-3003`: CAPF providers return ranges, collections, and
  properties; active completion keeps state by marker/range checks.
- `minibuffer.el.gz:2918-2979`: `:exclusive no` allows CAPF fallback after a
  failed prefix match; an exclusive provider can own the attempt.
- `comint.el.gz:1886-2035`: `comint-send-input` owns process-mark, history, and
  last-input bookkeeping.
- `dired.el.gz:2823-2832`, `dired.el.gz:3959-4059`,
  `dired.el.gz:4339-4422`: Dired marks, deletion flags, filesystem deletion,
  and Dired undo are distinct.
- `emacs-lisp/timer.el.gz:478-484`: `with-timeout` only checks while Emacs is
  waiting for external events; CPU-bound loops can run past the deadline.
- `buffer-substring` and `buffer-substring-no-properties`: installed primitive
  metadata plus probe evidence show that property-preserving reads can invoke
  `buffer-access-fontify-functions` while no-properties reads avoid that hook.

Doom:

- `/opt/soma/doom-home/.emacs.d/lisp/doom-lib.el:880-923`: `after!`
  expands around feature loading and disabled-package checks.
- `/opt/soma/doom-home/.emacs.d/lisp/doom-lib.el:1528-1547`: `doom!`
  processes modules in noninteractive evaluation and is not an interactive
  startup executor.
- `/opt/soma/doom-home/.emacs.d/lisp/doom-lib.el:1601-1605`: `package!` is
  package metadata, not package readiness.
- `/opt/soma/doom-home/.emacs.d/modules/config/use-package/init.el:132-163`:
  `use-package!` wraps `use-package` and respects disabled/missing packages.
- `/opt/soma/doom-home/.emacs.d/lisp/doom-keybinds.el:439-486`: `map!`
  depends on active keybinding machinery and ignores Evil-specific bindings if
  Evil is not loaded.

Package entry points are recorded in the package contract files.

## Observation Classes

`passive text/state read`:

- Reads that avoid display changes, package activation, prompt entry, process
  waits, and external service calls.
- Examples: `buffer-substring-no-properties`, buffer names, point/mark/narrowing
  values, and modification ticks.
- Gym rule: passive reads may be used in normal observation, but must still
  record which buffer and window context was bound while reading.

`derived inspection`:

- Reads that may compute or consult derived package state without changing the
  external world.
- Examples: Magit section inspection, Org element parse, LSP local workspace
  metadata, CAPF query without acceptance, and property-preserving text reads
  that may run access fontification hooks.
- Gym rule: record derived cost and package feature loading. A derived
  inspection is not silently equivalent to a passive read.

`active diagnostic`:

- An observation-like action that may load packages, query a server, refresh a
  Magit status buffer, run redisplay, or ask a kernel/process for state.
- Gym rule: require explicit permission in the action profile and record the
  asynchronous lifecycle and side effects.

## RFC 0002 Assumptions

Confirmed:

- A trace/checker layer is necessary. The probe traces record behavior that
  final filesystem state would miss.
- A rootfs/Insula-style launcher is necessary. The public probe runner now enters
  bwrap and records `insula-plan.json`.
- A typed action profile is needed. The source/probe evidence shows direct calls,
  command dispatch, terminal input, and package-owned operations have different
  contracts.
- The benchmark and learner-facing bridge must remain separate. Several probes
  require hidden independent checks, such as parent Org buffer text or stale
  completion acceptance rejection.
- The selected source-built rootfs can support a rootfs-owned Emacs+LSP source
  benchmark for Python, C++, and Rust. The LSP benchmark reports 6 scenarios
  total and 6 as expected, including direct-edit rejection and stale
  document/workspace guards.
- The selected source-built rootfs can run the current benchmark family set as
  one aggregate gate: Magit, Comint, Jupyter, LSP, Org, Dired, batch bridge,
  and daemon bridge report 90 scenarios total and 90 as expected with Insula
  checks passing, including the daemon bridge prompt-response lifecycle, checked
  point navigation, a real terminal minibuffer prompt completed by tmux input,
  real terminal-key Magit hunk staging, queued Magit hunk staging through daemon
  `command.dispatch`, Magit Transient prompt-state observation/cancel from real
  terminal input, Magit Transient Stage suffix execution with observed
  target-file scope, Magit Diff Transient `--stat` infix state transition with
  diff-buffer evidence and unchanged Git state, a package-owned Magit
  `magit-stage-files` prompt completed by tmux input with preserved command
  identity and hook evidence,
  stale/refreshed Magit view recovery through daemon `magit.observe_sections`,
  repository-fingerprint precondition rejection, and refreshed checked
  `magit-stage`, one live
  Comint terminal-input scenario through `comint-send-input` to a rootfs-owned
  subprocess, one daemon existing-kernel Jupyter eval-region scenario through
  `jupyter.observe_source`, `jupyter.eval_region`, `request.status`,
  `request.await`, and `jupyter.query_state` with preserved kernel token,
  rootfs-owned runtime paths, and exactly-once selected-region execution, one
  daemon Jupyter `display_data`/`update_display_data` capture path with shared
  parent request id, stable display id, ordered initial/update payloads, and
  UI-rendered REPL display-update presentation, one
  daemon stale `completion.accept` rejection after terminal-frame human editing,
  daemon Python LSP rename through both semantic `lsp.rename_symbol` and
  terminal/minibuffer `command.dispatch -> lsp-rename` paths, real
  `textDocument/rename`, lsp-mode workspace edit application, rootfs-owned
  `pylsp`/Python identity, multi-file update checks, and rootfs Python tests
  after the rename, daemon pyright diagnostics-after-edit evidence for
  full-document LSP sync, a coalesced live-buffer replacement, save,
  `Expected expression` diagnostic publication, repair/save, and diagnostic
  cleanup, daemon-owned multi-request async status/await evidence for
  two independent in-flight requests awaited out of submission order without
  result swapping, a cross-package async case where delayed existing-kernel
  Jupyter eval remains independently observable while queued Magit staging
  completes first, a live terminal-frame Comint interrupt reconciliation case
  that preserves rootfs-owned shell process identity and prevents the
  interrupted side effect, and batch bridge selected-window command drift, point
  edit drift,
  narrowed/mark-active/read-only edit-state guards, checked completion
  acceptance, and stale completion rejection.
- The generic selected-rootfs batch runner now delegates rootfs selection,
  network authorization, bwrap mount construction, environment allowlist
  construction, and plan emission to `scripts/insula/soma_insula.py`. Current
  aggregate plans show this module as the authority for Magit, Comint, Jupyter,
  LSP, Org, Dired, the batch bridge, and the live daemon bridge. Separate
  module-authority traces show the Internals Atlas probe wrapper and Doom
  lifecycle worker also route through `soma_insula.py run-command`.
- The selected source-built rootfs can support batch-safe Org state-machine
  checks. The Org benchmark reports 6 scenarios total and 6 as expected,
  including source-edit parent-buffer writeback, refile metadata preservation,
  duplicate-ID rejection, visible-text-only move rejection, and Babel
  parent-buffer result insertion.
- The selected source-built rootfs can support batch-safe Dired filesystem
  checks. The Dired benchmark reports 8 scenarios total and 8 as expected,
  including marked-delete behavior, mark/flag distinction, rename collision
  handling, overwrite rejection, and Dired-undo-as-filesystem-recovery
  rejection. It also covers external same-path replacement after a Dired mark:
  Dired may preserve the same visible mark after refresh, so the policy must
  compare filesystem signatures, clear/abort stale marked handles, and preserve
  the replacement file.
- The batch bridge now proves passive `buffer.read` does not disturb point,
  mark-active, `chars_tick`, or selected-window state; checked
  `buffer.replace_range` advances the character revision; checked
  `buffer.save` writes the intended visited file; and stale edit preconditions
  reject without an additional mutation.
- The selected source-built rootfs can start a live Emacs daemon and expose the
  JSONL bridge socket from inside that daemon to a separate rootfs-owned Python
  client process. That daemon bridge now exposes one waitable in-flight request
  through `request.status` and `request.await`, and a model-facing
  `magit.observe_sections` operation that returns section identity plus Git
  repository fingerprints for stale-view guards.
- Existing-kernel `emacs-jupyter` association is now covered by the batch-safe
  Jupyter benchmark. It validates the same-rootfs Emacs/Jupyter/kernel path,
  wrong-buffer rejection, restart/continuity rejection, timeout as
  `outcome_unknown`, duplicate retry rejection, and protocol-level
  `display_data`/`update_display_data` parent-id, display-id, and payload
  ordering. The daemon bridge adds the first model-facing existing-kernel
  eval-region paths with asynchronous status/await evidence, kernel-state
  query evidence, and UI-rendered REPL display-update presentation.

Revised:

- The gym interface cannot be frozen before terminal-frame probes. Batch-safe
  probes are useful but explicitly do not prove Ghostty/SSH/tmux input semantics.
- The current host profile is non-Evil. The default gym keyboard assumptions
  should model the inspected profile unless another profile is explicitly
  selected.
- Doom generated state is part of the environment identity. The gym must record
  generated module/profile files, not only `~/.doom.d/init.el`.
- The current production-style Insula baseline is a selected
  CUDA-image-derived rootfs with source-built stock GNU Emacs and a rootfs
  contract. The development minimal-bind namespace is still available only when
  explicitly requested with `SOMA_USE_DEVELOPMENT_INSULA=1`.
- Rootfs evidence cannot rely on environment variables alone. The production
  contract must also verify working-directory sentinels, rootfs/entrypoint
  identity, and the emitted bwrap plan before launching Emacs, kernels, or
  benchmark subprocesses.
- The production Insula contract is stronger than the current development
  bwrap namespace. Hermeticity requires a selected rootfs that owns Emacs, Doom
  package state, LSP servers, Jupyter server/kernels, Python/C++/Rust
  toolchains, build/test tools, and ABI-relevant shared libraries.
- Source-built Emacs is now a runtime contract: production selected-rootfs
  runners validate `/opt/soma/emacs/bin/emacs`, the GNU source archive, the
  retained source tree, configure flags, dynamic loader, linked DSOs, Doom
  source, and required tool inventory before entering benchmark work.
- Network disablement is not the definition of hermeticity. Network is a
  declared phase/task capability; the reproducibility claim comes from the
  rootfs dependency closure and ABI manifest.
- A patched Emacs build would be an environment-identity change, not a hidden
  instrumentation detail. The Atlas should keep stock Emacs as the baseline and
  require stock-vs-patched evidence before adopting patches for operability or
  observability.
- LSP source evidence has moved beyond raw stdio initialization. The daemon
  bridge now covers model-facing Python rename through both semantic
  `lsp.rename_symbol` and terminal/minibuffer `command.dispatch -> lsp-rename`,
  with real `textDocument/rename` and lsp-mode workspace edit application.
  Remaining LSP gaps are broader workspace-edit/code-action cases, multi-root
  stress cases, and richer async model-facing bridge behavior.

Rejected:

- Batch Emacs alone is a faithful model of interactive command-loop behavior.
- Entering `bwrap` with host `/usr` binds is enough to make benchmark execution
  hermetic.
- A single `buffer-chars-modified-tick` guard is sufficient for edits.
- A buffer can be modeled as one string per visited filename.
- A Magit task can be graded solely by final Git index contents.
- A Jupyter timeout can be treated as proof that no execution occurred.
- Jupyter kernel execution is not hermetic merely because Emacs is running in
  an Insula. The kernel process and Python environment need their own rootfs
  evidence; the current Jupyter benchmark records that evidence for its
  batch-safe existing-kernel and display/update slice.
- Emacs+LSP behavior is not rootfs-faithful merely because the Emacs client runs
  in Insula. The language server process and relevant language toolchain need
  the same rootfs evidence; the current LSP source benchmark records that
  evidence for Python, C++, and Rust fixtures.
- A locally patched Emacs binary can be treated as equivalent to stock Emacs
  without recording source revision, configure flags, patch set, build
  provenance, native-comp closure, ABI manifest, and stock-vs-patched behavior
  deltas.

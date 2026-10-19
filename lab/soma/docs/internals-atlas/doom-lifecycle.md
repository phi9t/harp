# Doom Lifecycle

The inspected production-style setup is Doom v3-style core and user config
inside the selected Soma Insula rootfs. It is not an Evil-enabled profile, not a
plain `emacs -Q` environment, and not the user's host Doom profile.

Current lifecycle evidence comes from:

```sh
workers/emacs-doom-lifecycle/scripts/run-lifecycle.sh
```

Latest result: `workers/emacs-doom-lifecycle/traces/latest/summary.md` reports
status `pass` for cold, warmed, cold-repeat-1, and warmed-repeat-1 child Emacs
processes, with the aggregate teardown probe passing.

## Inspected Topology

- Insula mode: `selected-container-rootfs`
- Rootfs:
  `.scratch/emacs-gym/insula/rootfs-store/content/soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34`
- Materialization kind: `source-built-emacs-v8`
- Rootfs recipe SHA256:
  `0bddf41e6539849c1d8e0a44954ebb3293f6eaa08081741c4adbf3c777fe808c`
- Base image:
  `ghcr.io/pytorch/pytorch:2.13.0-cuda13.2-cudnn9-runtime@sha256:7492928e093d67276716440161f694a0e4ea27796d599d64b23cb76cdd665e71`
- Rootfs tree hash SHA256:
  `931b5d4338e0c0eb876f5b14f7babb71bc89b20e5b6a625d26799d16c0cb41dd`
- Emacs: source-built `/opt/soma/emacs/bin/emacs`, version `30.2`
- Emacs executable SHA256:
  `824c1318464a98778d9f1677508e05af3dc5e94c34c63653145e079e5d26f6c2`
- Emacs source directory: `/opt/soma/build/emacs-30.2/`
- Emacs configured with native-comp, JSON, tree-sitter, and sqlite3, without X.
- Doom core inside the sandbox: `/project/home/.emacs.d`
- Doom core rootfs source: `/opt/soma/doom-home/.emacs.d`
- Doom core revision: `8b475fcb2574286d7f4413c22cc7902d80d21ced`
- User config inside the sandbox: `/project/home/.doom.d`
- User config rootfs source: `/opt/soma/doom-home/.doom.d`
- Generated profile entrypoint:
  `/project/home/.emacs.d/.local/etc/@/init.30.2.el`
- Generated module fragments:
  `/project/home/.emacs.d/.local/etc/@/init.30.d/`
- Profile cache:
  `/project/home/.emacs.d/.local/cache/profiles.@.el`
- Package build tree:
  `/project/home/.emacs.d/.local/straight/build-30.2/`
- Package repos:
  `/project/home/.emacs.d/.local/straight/repos/`

The lifecycle runner mounts the rootfs-owned Doom core and user config into a
private `HOME`. The code is read-only, while per-run Doom cache/state
directories are copied from the selected rootfs and rebound writable under
`/project/home/.emacs.d/.local/cache` and
`/project/home/.emacs.d/.local/state`. This preserves rootfs ownership of code
and package state without pretending runtime caches are immutable.

## Enabled Modules

The selected rootfs `.doom.d/init.el` enables:

- Emacs: `dired`, `electric`, `undo`, `vc`
- Tools: `eval`, `lookup`, `lsp`, `magit`
- Languages: `emacs-lisp`, `(org +jupyter)`, `python`, `rust`, `cc`, `sh`
- Config: `(default +bindings)`

The `evil` module is absent. Corfu, Vertico, Popup, and Doom's TTY module are
also absent from the loaded profile. The lifecycle trace records these as
capability states rather than assuming readiness from host configuration.

Important trace detail: the generated profile text contains the token
`(:ui . popup)` inside disabled-module explanatory text. That is not module
declaration evidence. The lifecycle worker therefore treats Doom's loaded
`doom-modules` table and `doom-module-active-p` as authoritative for
`module_active_in_loaded_profile`.

The warmed lifecycle phases currently prove:

- `magit`, `org`, `jupyter`, `lsp-mode`, `python`, `rust-mode`, `cc-mode`, and
  `sh-script` can be required inside the selected rootfs.
- `emacs-lisp-mode`, `org-mode`, `python-mode`, `rust-mode`, `c++-mode`, and
  `sh-mode` activate in temporary buffers.
- Jupyter kernelspec discovery succeeds through rootfs Python
  `/usr/bin/python3`.

The lifecycle worker now also records per-child teardown evidence. Each child
launch receives a unique `SOMA_LIFECYCLE_CHILD_TOKEN`; the parent reads
`/proc/<pid>/environ` before and after the child launch and checks that no
preexisting process had the token and no token-marked process remains after the
child exits. The focused trace
`workers/emacs-doom-lifecycle/traces/lifecycle-teardown-green2/summary.json`
records `child_count=4` and `teardown_probe.status=pass`.

## Startup Lifecycle

The lifecycle to model is:

```text
configuration declarations
-> package/module resolution and generated artifacts
-> early bootstrap
-> profile initialization
-> module initialization/configuration
-> user configuration
-> deferred first-use activation
```

Important source points:

- `doom!` at `/project/home/.emacs.d/lisp/doom-lib.el:1528-1547`
  mutates module state in noninteractive evaluation.
- `package!` at `/project/home/.emacs.d/lisp/doom-lib.el:1601-1605`
  declares package metadata.
- `use-package!` at
  `/project/home/.emacs.d/modules/config/use-package/init.el:132-163`
  wraps `use-package`.
- `after!` at `/project/home/.emacs.d/lisp/doom-lib.el:880-923`
  delays code until feature load and skips disabled packages.
- `map!` at `/project/home/.emacs.d/lisp/doom-keybinds.el:439-486`
  expands key declarations through Doom/general keybinding machinery.

## Six Meanings Of Enabled

The gym must record these separately:

- Module declared: present in `/project/home/.doom.d/init.el` and Doom's loaded
  module state after the generated profile is loaded.
- Package installed: source/build artifact exists under Straight.
- Autoload available: generated autoload file exists in `build-30.2`.
- Feature loaded: `(featurep 'package)` in the live Emacs process.
- Mode active: major/minor mode active in a specific buffer.
- External service ready: language server, Jupyter kernel, process, tmux, or
  Git repository is alive and associated with the right buffer/session.

Probe `doom-cold-warmed-state` and the lifecycle worker demonstrate this split:
the cold child records generated profile/package state before first-use
requires, while the warmed child loads selected packages and records live
feature and mode activation. Capability readiness is a tuple, not one bit.

## Environment Identity Fields

Every gym episode must record:

- Emacs executable and version
- Doom core path and revision
- user config path and content hash
- generated profile/module file paths and hashes
- Straight build directory and relevant package revisions
- profile-specific extra package/config files, including AI/completion config
  only when the selected profile declares it
- action profile
- terminal mode and whether a terminal frame was used
- Insula rootfs or development-minimal-binds contract
- HOME, TMPDIR, XDG cache/data/runtime dirs inside Insula
- Kernel/subprocess rootfs identity for workflows that launch Jupyter kernels,
  language servers, shells, or other long-lived children

## Constraints

- Do not infer readiness from `doom!` alone.
- Do not assume Evil keybindings or the user's host Doom module set for this
  selected rootfs.
- Do not bypass generated Doom state when launching a faithful gym session.
- Do not treat `after!` as readiness for LSP, Jupyter, Magit refresh, or popup
  display completion.
- Terminal-frame probes are required before any final keyboard-fidelity claim.

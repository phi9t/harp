# LSP Contract

## Owned State

- Workspace roots and clients.
- Document synchronization state and version.
- Position encoding/decoding functions.
- Pending request ids.
- Workspace edits and diagnostics.

Primary local sources:

- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/lsp-mode/lsp-mode.el:4417-4423`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/lsp-mode/lsp-mode.el:4431-4501`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/lsp-mode/lsp-mode.el:4503-4514`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/lsp-mode/lsp-mode.el:4532-4558`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/lsp-mode/lsp-mode.el:4583-4664`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/lsp-mode/lsp-mode.el:6819-6826`
- `/opt/soma/doom-home/.emacs.d/.local/straight/repos/lsp-mode/lsp-mode.el:7201-7208`

## Required Session Context

- Current buffer must be an LSP-managed document.
- Workspace and client must match the intended project.
- Buffer point must map through negotiated position encoding.
- Document version must match the edit response.
- Language-server process must resolve from the selected Soma rootfs unless the
  task explicitly studies a cross-environment failure.
- Python, C/C++/CUDA, and Rust tasks must also record the relevant rootfs-owned
  language toolchain: Python executable/prefix/packages for Python; compiler,
  linker, headers, libc/libstdc++ identity, and `clangd` for C/C++; `rustc`,
  `cargo`, crate/cache paths, and `rust-analyzer` for Rust.
- Network access, if needed by a server, is a declared task capability and does
  not replace dependency-closure or ABI evidence.

## Faithful Command Paths

- `lsp-rename` reads interactive rename text, sends `textDocument/rename`, and
  applies a workspace edit.
- A direct text replacement is not an LSP rename.
- Workspace edit application must respect document-version checks.

## Async Work

- Requests to language servers.
- Diagnostics after edit application.
- Server-initiated `workspace/applyEdit`.
- Server subprocess startup and toolchain/build-system probing.

## Stale Handles

- Workspace/client after server restart.
- Document version after user edits.
- Position encoders after client/workspace changes.
- Diagnostics after edits but before publish.

## Completion Evidence

- Request result received for the intended client/workspace.
- Language-server executable, process cwd, environment, and rootfs manifest match
  the Emacs-side plan.
- Relevant Python/C++/Rust toolchain identity is captured from inside the
  rootfs.
- Workspace edit applied without document-version mismatch.
- Affected files match expected LSP result.
- Diagnostics and project checks do not show introduced errors where covered.

## Negative Cases

- Same-spelled identifiers changed outside LSP reference set.
- Rename applied despite stale document version.
- Position interpreted as raw byte/character offset without encoding conversion.
- Text replacement reported as an LSP refactor.
- Emacs client inside Insula but `pyright`, `clangd`, `rust-analyzer`, compiler,
  or build tool resolved from host state.

## Current Probe Evidence

Probe `lsp-server-rootfs-resolution-contract` records server command resolution
for Python, C/C++, and Rust language servers. Probe
`lsp-stdio-initialize-rootfs-identity` starts `pyright-langserver`, `clangd`,
and `rust-analyzer` through stdio inside the selected rootfs, sends
`initialize`, waits for an initialize result, sends `shutdown`/`exit`, and
records executable realpath, pid, workspace root, and shutdown status.

That proves rootfs-owned server startup and protocol readiness.

## Current Benchmark Evidence

The selected-rootfs LSP source benchmark is implemented in
`benchmarks/emacs-gym/lsp/elisp/run-benchmark.el` and run through
`benchmarks/emacs-gym/lsp/scripts/run-benchmark.sh`. The current selected-rootfs
aggregate trace lives under `benchmarks/emacs-gym/traces/latest/lsp/`.

The current summary reports 6 scenarios total and 6 as expected with Insula
checks passing:

- `correct-python-lsp-source`
- `correct-cpp-lsp-source`
- `correct-rust-lsp-source`
- `bad-direct-text-edit-as-refactor`
- `stale-document-version-rejected`
- `stale-workspace-root-rejected`

This proves a real Emacs `lsp-mode` client can obtain source/symbol evidence
from rootfs-owned Python, C++, and Rust fixtures under the selected
source-built Emacs rootfs. It also proves that the grader can reject direct text
replacement masquerading as an LSP refactor and stale document/workspace
handles.

The live daemon benchmark adds two model-facing LSP rename paths:
`benchmarks/emacs-gym/traces/latest/bridge-daemon/23-daemon-lsp-python-rename-symbol.json`.
That scenario switches the daemon terminal frame to the Python source buffer,
observes the initialized `pylsp` workspace, queues `lsp.rename_symbol`, tracks
it through `request.status` and `request.await`, verifies that lsp-mode sent
`textDocument/rename`, verifies workspace edit application, checks the
definition, caller, and test files, and runs rootfs Python tests after the
rename. The second path is
`benchmarks/emacs-gym/traces/latest/bridge-daemon/24-daemon-lsp-terminal-rename-symbol.json`;
it queues `command.dispatch -> lsp-rename`, observes the real minibuffer prompt,
supplies the new name through tmux, polls the original request to terminal
success, confirms `command-execute`, `this-command`, `textDocument/rename`, and
workspace edit evidence, then verifies the multi-file update and rootfs Python
tests. It also adds
`benchmarks/emacs-gym/traces/latest/bridge-daemon/25-daemon-lsp-diagnostics-after-edit.json`,
which uses a separate rootfs-owned `pyright-langserver` fixture for live
diagnostics-after-edit evidence. That path records rootfs-owned
`pyright-langserver`, Node, and Python executable paths, uses full-document LSP
sync with debounce disabled, edits the live daemon buffer through guarded
`buffer.replace_range`, saves the broken buffer through the bridge before
requiring diagnostics containing `Expected expression`, repairs and saves the
source through the bridge, observes diagnostics clearing under explicit
`lsp-diagnostic-clean-after-change` fixture policy, and runs rootfs Python tests
on the repaired source.

The remaining LSP contract work is additional workspace-edit/code-action cases,
multi-root projects, and richer terminal-frame/async bridge variants.

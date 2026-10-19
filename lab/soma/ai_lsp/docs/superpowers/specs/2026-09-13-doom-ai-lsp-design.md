# Doom AI Completion and LSP Verification Design

## Purpose

`devx` is a standalone, version-controlled home for the Doom Emacs AI
completion configuration and its integration verifier.  It owns the Copilot
and Tabbymacs behavior added during this work while leaving unrelated personal
Doom configuration in `~/.doom.d`.

The implementation must fix incorrect Copilot language identifiers and verify
that Copilot can operate concurrently with the configured language server for
Rust, Python, C++, and CUDA.

## Repository Layout

```text
devx/
├── README.md
├── bin/
│   └── verify-ai-lsp
├── doom/
│   ├── ai-completion.el
│   └── packages.el
├── test/
│   └── ai-lsp-verifier.el
└── docs/superpowers/
    ├── plans/
    └── specs/
```

Each file has one responsibility:

- `doom/packages.el` declares `copilot.el` and Tabbymacs for Doom's package
  manager.
- `doom/ai-completion.el` configures key bindings, backend selection, automatic
  activation, and Copilot compatibility mappings.
- `test/ai-lsp-verifier.el` owns fixture creation, assertions, timeouts, and
  integration-test reporting inside Emacs.
- `bin/verify-ai-lsp` launches the verifier in an isolated Doom daemon and
  guarantees process cleanup.
- `README.md` documents installation, commands, expected results, and failure
  interpretation.

## Doom Integration Boundary

The existing `~/.doom.d/packages.el` will replace its inline AI package
declarations with one absolute `load` of `~/workspace/devx/doom/packages.el`.
The existing `~/.doom.d/config.el` will replace its inline AI completion block
with one absolute `load` of `~/workspace/devx/doom/ai-completion.el`.

Absolute paths are intentional: the repository location was explicitly chosen
and this avoids symlink lifecycle problems.  Both loader entries will check
that the target exists and raise an actionable error if the repository is
missing, rather than silently starting Doom without the requested behavior.

No other Doom settings move into this repository.

## Runtime Behavior

Copilot remains the default inline completion backend.  Tabbymacs remains a
manually selectable alternative.  The two modes are mutually exclusive in each
buffer.

The existing keys remain stable:

- `C-f` accepts a full inline suggestion.
- `M-f` accepts one Copilot word.
- `C-e` accepts one Copilot line.
- `C-c a c` selects Copilot.
- `C-c a t` selects Tabbymacs.
- `C-c a 0` disables inline AI completion.

Automatic Copilot activation is limited to writable, local, file-visiting
programming buffers.  It must not start for `*scratch*`, minibuffers, hidden
internal buffers, read-only buffers, or remote files.  Manual selection remains
available when the user explicitly wants Copilot elsewhere.

## Copilot Language-ID Compatibility

The installed and current upstream `copilot.el` stores language mapping keys as
strings but looks them up using major-mode symbols.  Until upstream corrects
that mismatch, the configuration will add symbol-key aliases without replacing
the upstream string entries.  This makes the workaround effective now and
harmless after an upstream fix.

The required identifiers are:

| Emacs mode | Copilot language ID |
| --- | --- |
| `rustic-mode`, `rust-mode`, `rust-ts-mode` | `rust` |
| `python-mode`, `python-ts-mode` | `python` |
| `c++-mode`, `c++-ts-mode` | `cpp` |
| `cuda-mode`, `cuda-ts-mode` | `cuda-cpp` |

The configuration will keep Doom's `lsp-copilot-enabled` disabled.  Copilot's
own `copilot.el` JSON-RPC connection supplies inline completions; `lsp-mode`
continues to manage only the language-specific analysis server.  This prevents
two Emacs clients from starting competing Copilot language-server sessions for
the same buffer.

## Verifier Architecture

`bin/verify-ai-lsp` will start a uniquely named Doom daemon, run the Emacs Lisp
verifier through `emacsclient`, collect its exit result, and stop only the
daemon it created.  A shell trap will perform cleanup after success, failure,
or interruption.  The verifier will not connect to or alter an existing Emacs
session.

The Emacs verifier will use a temporary root for source fixtures and a temporary
`lsp-session-file`.  It will enable `lsp-auto-guess-root` so a headless run never
blocks on project-import prompts.  Each language fixture will be a minimal,
independent project or translation unit.

For every language, the verifier must establish all of these facts in the same
buffer:

1. Doom selected the expected major mode.
2. Copilot mode is enabled and Tabbymacs mode is disabled.
3. Copilot reports the expected language identifier.
4. The expected language-server process starts and reaches LSP's `initialized`
   state.
5. Copilot's server connection is alive and authenticated.
6. A real `textDocument/inlineCompletion` request completes before the timeout
   with a protocol-valid response.  An empty completion list is valid because
   model output is nondeterministic; authentication, transport, or timeout
   errors are failures.

The expected language servers are:

| Language | Emacs mode | LSP server |
| --- | --- | --- |
| Rust | `rustic-mode` | `rust-analyzer` |
| Python | `python-mode` | `pyright` |
| C++ | `c++-mode` | `clangd` |
| CUDA | `cuda-mode` | `clangd` |

Rust Analyzer is provided by the Rust toolchain.  Clangd is already installed
in Doom's local LSP server directory.  Pyright will be installed under the
user's home directory and must be discoverable through Doom's saved
environment.  CUDA verification checks the Clangd protocol integration and
mode selection; it does not require `nvcc` or assert that a CUDA toolkit is
installed.

## Failure Handling

Every wait has a bounded timeout.  Failure output names the language, failed
boundary, expected value, observed value, relevant executable, and the tail of
the LSP or Copilot log when available.  One language failure does not suppress
results for the remaining languages.

Temporary source files, the temporary LSP session, the isolated Emacs daemon,
and child language-server processes are removed or terminated by the verifier.
The verifier never deletes user projects, modifies source files outside its
temporary root, logs authentication tokens, or prints the Copilot auth database.

## Test Strategy

Implementation follows red-green verification:

1. Add mapping and activation tests that fail against the current inline Doom
   configuration and current `copilot.el` behavior.
2. Move the configuration into `devx` and add the compatibility mappings until
   those tests pass.
3. Run a fixture-level verifier with deliberately wrong expected server IDs to
   prove its assertions detect mismatches, then restore the correct matrix.
4. Run the complete isolated verifier for Rust, Python, C++, and CUDA.
5. Run `doom sync` and `doom doctor` using the configured Emacs environment.
6. Start a fresh Doom daemon and confirm that Copilot no longer opens
   `*scratch*` automatically.

Completion is reached only when the four-language verifier exits zero and the
fresh Doom checks pass.  If an external Copilot service outage prevents a real
completion request, the result will be reported as an external blocker rather
than converted into a passing test.

## Migration and Rollback

Migration copies the existing AI package declarations and configuration into
the repository before replacing the original blocks with loaders.  A syntax or
test failure stops the migration before any completion claim.

Rollback consists of restoring the original inline blocks in the two Doom files
from their pre-migration content.  The standalone repository can then remain as
documentation or be disconnected without affecting unrelated Doom settings.

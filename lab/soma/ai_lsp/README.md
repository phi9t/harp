# DevX Doom AI/LSP configuration

This standalone repository owns the Doom Emacs inline-AI configuration and its
real language-server verifier.  It keeps the AI-specific package declarations,
runtime configuration, and tests version controlled without moving unrelated
settings out of `~/.doom.d`.

## Layout

| Path | Responsibility |
| --- | --- |
| `doom/packages.el` | Doom package declarations for `copilot` and `tabbymacs` |
| `doom/ai-completion.el` | Inline-completion behavior, bindings, and Copilot language IDs |
| `test/ai-lsp-verifier.el` | Unit and real LSP/Copilot integration checks |
| `bin/verify-ai-lsp` | Isolated Doom-daemon verifier launcher |

## Install in Doom

Keep the rest of your personal Doom configuration in `~/.doom.d`.  Add these
checked loaders to the corresponding files; the absolute paths are intentional.

In `~/.doom.d/packages.el`:

```elisp
(let ((devx-ai-packages
       (expand-file-name "~/workspace/devx/doom/packages.el")))
  (unless (file-readable-p devx-ai-packages)
    (error "DevX AI package declarations are missing: %s" devx-ai-packages))
  (load devx-ai-packages nil 'nomessage))
```

In `~/.doom.d/config.el`:

```elisp
(let ((devx-ai-config
       (expand-file-name "~/workspace/devx/doom/ai-completion.el")))
  (unless (file-readable-p devx-ai-config)
    (error "DevX AI completion configuration is missing: %s" devx-ai-config))
  (load devx-ai-config nil 'nomessage))
```

After changing package declarations, run:

```bash
~/.emacs.d/bin/doom sync
```

Then restart Emacs so it loads the updated configuration.  Run `doom sync`
after a pull that changes `doom/packages.el`; a configuration-only change
needs only an Emacs restart.

## Inline completion

Copilot is selected automatically only for writable, local, file-visiting
programming buffers.  It is the default backend; Tabbymacs is a manual,
buffer-local alternative, and the two are kept mutually exclusive.  Doom's
`lsp-copilot-enabled` remains disabled: Copilot uses its own connection while
`lsp-mode` manages the language-specific server.

| Key | Action |
| --- | --- |
| `C-c a c` | Select Copilot for the current buffer |
| `C-c a t` | Select Tabbymacs for the current buffer |
| `C-c a 0` | Disable inline AI completion for the current buffer |
| `TAB` | Accept the complete inline suggestion |
| `C-f` | Accept the complete inline suggestion |
| `M-f` | Accept one Copilot word |
| `C-e` | Accept one Copilot line |

## Verified language matrix

The integration verifier opens a fixture for every row, checks the mode,
Copilot language ID, initialized LSP server, Copilot authentication, and a real
`textDocument/inlineCompletion` request.

| Language | Emacs mode | LSP server | Copilot language ID |
| --- | --- | --- | --- |
| Rust | `rustic-mode` | `rust-analyzer` | `rust` |
| Python | `python-mode` | `pyright` | `python` |
| C++ | `c++-mode` | `clangd` | `cpp` |
| CUDA | `cuda-mode` | `clangd` | `cuda-cpp` |

CUDA verification covers CUDA mode selection and Clangd's protocol integration;
it does not require `nvcc` or a CUDA toolkit.

## Verify

The launcher starts a Doom daemon whose name includes both the launcher PID and
a random nonce.  Before using or stopping it, the launcher verifies a private
ownership marker and the daemon PID.  Startup, client calls, and cleanup are
bounded, and cleanup removes only that verified daemon and the launcher's own
temporary root.  It accepts exactly these modes:

```bash
bin/verify-ai-lsp unit
bin/verify-ai-lsp integration
bin/verify-ai-lsp all
```

`unit` runs the configuration contracts.  `integration` runs the four real
language-server plus Copilot checks.  `all` runs unit first and then
integration.  Successful integration output has one `PASS` line per matrix row.
The default outer bounds are 30 seconds for startup, 300 seconds for a client
evaluation, and 5 seconds for cleanup.  For a deliberately slow environment,
set positive integer-second overrides with `DEVX_AI_LSP_STARTUP_TIMEOUT`,
`DEVX_AI_LSP_CLIENT_TIMEOUT`, or `DEVX_AI_LSP_CLEANUP_TIMEOUT`.

Pyright is installed through lsp-mode's package provider and is discovered by
`(lsp-package-path 'pyright)` at
`~/.emacs.d/.local/etc/lsp/npm/pyright/bin/pyright-langserver` on this
installation.  The other prerequisites are `rust-analyzer` on `PATH` and the
LSP-managed `clangd` package.

## Troubleshooting

- **Emacs is not found:** set `DEVX_EMACS` to the absolute path of the desired
  executable, for example `DEVX_EMACS=/path/to/emacs bin/verify-ai-lsp all`.
  The launcher otherwise tries `emacs` on `PATH`, then Doom's saved environment.
- **A server binary is missing:** install `rust-analyzer` so it is on `PATH`.
  For `pyright` or `clangd`, use Doom/lsp-mode's package management and rerun
  the verifier; do not substitute an unrelated global binary for the
  LSP-managed package.
- **Copilot authentication fails:** complete Copilot's normal Emacs sign-in
  flow, restart Emacs, and rerun the verifier.  Never print, copy, or include
  the Copilot auth database or access tokens in diagnostics.
- **A network request times out:** check network/proxy connectivity, retry the
  bounded verifier, and preserve only its sanitized error context.  An empty
  completion list is acceptable; authentication, transport, and timeout errors
  are not.  Increase `DEVX_AI_LSP_CLIENT_TIMEOUT` only when the outer launcher
  bound, rather than a reported LSP or Copilot request bound, is the limiting
  timeout.
- **Daemon startup fails or times out:** the launcher reports a bounded,
  credential-sanitized tail of its private daemon log.  It refuses to use or
  stop a same-named daemon unless both its ownership marker and PID match.
- **An LSP server does not initialize:** inspect the relevant tail of `*lsp-log*`
  reported by the verifier and confirm the server named in the matrix is
  installed.  Do not add authentication records or token values to verifier
  output, bug reports, or logs.

Verifier output must never contain authentication-database contents or access
tokens.

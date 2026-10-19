# Package Contracts Probe Notes

## Environment identity inspected

- Repo issue: `.scratch/emacs-gym/issues/0001-emacs-doom-internals-atlas.md`
- Emacs: `30.2`, `x86_64-pc-linux-gnu`
- Magit checkout: `~/.emacs.d/.local/straight/repos/magit` at `c800f79c2061621fde847f6a53129eca0e8da728`
- Org checkout: `~/.emacs.d/.local/straight/repos/org` at `89df5bf46ba214db44eea898cc7cacc0b27fd760`
- lsp-mode checkout: `~/.emacs.d/.local/straight/repos/lsp-mode` at `197f56bf7a38b564cc5dec26d66b5f494da39783`
- Jupyter checkout: `~/.emacs.d/.local/straight/repos/jupyter` at `242fdc709ce0faa3b9ee81dcc48cfd791898e6b8`
- Core Emacs libs inspected from:
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/comint.el.gz`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/dired.el.gz`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/minibuffer.el.gz`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/completion-preview.el.gz`

## Source paths and entry points

- Magit: `magit-section.el`, `magit-apply.el`, `magit-mode.el`
- Org: `org-src.el`, `org-capture.el`, `org-refile.el`, `org-id.el`, `ob-core.el`
- LSP: `lsp-mode.el`
- Jupyter: `jupyter-client.el`, `jupyter-base.el`, `jupyter-messages.el`, `jupyter-repl.el`, `jupyter-server.el`, `jupyter-server-kernel.el`
- Comint: `comint.el.gz`
- Dired: `dired.el.gz`
- Completion/CAPF/ghost text: `minibuffer.el.gz`, `completion-preview.el.gz`

## Findings

### Magit

- Section state is object-backed, buffer-local, and reconstructed on refresh. `magit-section-mode` installs command hooks and strips `magit-section` text properties from copied text, while `magit-root-section` is buffer-local state. Section objects are recreated by `magit-insert-section--create`, with markers at start/end and optional reuse of old visibility via `magit-get-section` and `magit-section-ident`; this means old section objects are stale after refresh even when their logical identity survives. Refs: `~/.emacs.d/.local/straight/repos/magit/lisp/magit-section.el:445-491`, `:552-610`, `:1429-1494`.
- Region staging is not a single primitive. `magit-stage` branches on diff type and scope: region, hunk, hunks, file, files, list, untracked, and falls back to interactive file selection. Faithful probes need the current section/diff scope, not just a pathname. Refs: `~/.emacs.d/.local/straight/repos/magit/lisp/magit-apply.el:282-306`, `:309-347`.
- Post-stage behavior is gated by `this-command`, not by observed Git side effects. `magit-run-post-stage-hook` only runs when `this-command` is in `magit-post-stage-hook-commands`, and the hook is invoked from `magit-post-refresh-hook`. A wrapper command that calls `magit-stage` directly can mutate the index while missing the post-stage hook path. Refs: `~/.emacs.d/.local/straight/repos/magit/lisp/magit-apply.el:395-402`, `~/.emacs.d/.local/straight/repos/magit/lisp/magit-mode.el:94-115`.

### Org

- Source edit buffers are marker-coupled views into a parent Org buffer, not separate ownership domains. `org-edit-src-code` captures block metadata, opens a dedicated edit buffer, and stores Babel info buffer-locally. Write-back uses `org-src--contents-for-write-back`, then `org-edit-src-exit` compares the original region against rewritten contents and replaces the parent region through markers. If the source buffer disappears, exit aborts. Refs: `~/.emacs.d/.local/straight/repos/org/lisp/org-src.el:528-546`, `:1321-1371`, `:1482-1518`.
- Capture owns a transient indirect buffer plus a plist snapshot of origin, target buffer, window config, and insertion metadata. `org-capture` stores `:original-buffer`, `:return-to-wconf`, target position, etc.; `org-capture-finalize` explicitly handles concurrent captures by restoring `org-capture-plist` from the buffer-local current plist before finalize hooks. `org-capture-refile` records the captured headline with a marker in the base buffer before killing the indirect buffer, then calls `org-refile` from the base buffer. Refs: `~/.emacs.d/.local/straight/repos/org/lisp/org-capture.el:701-807`, `:836-988`, `:990-1022`, `:1043-1065`, `:1702-1715`.
- Refile target identity is file plus position, but IDs can duplicate on copy paths. `org-refile` rejects targets inside the moving subtree/region, opens the destination file, and warns in the docstring that `org-refile-keep`/copy can duplicate `ID` properties. `org-id-get-create` delegates to `org-id-get`, and `org-id-add-location` only persists locations for file-visiting buffers, keyed by ID to abbreviated filename. Refs: `~/.emacs.d/.local/straight/repos/org/lisp/org-refile.el:433-560`, `~/.emacs.d/.local/straight/repos/org/lisp/org-id.el:309-345`, `:653-663`.
- Babel execution state is anchored by a marker to the outermost executing block. `org-babel-current-src-block-location` is a marker, and `org-babel-execute-src-block` reuses or computes it before execution, performs cache checks, may borrow `default-directory` from a session buffer, and dispatches to a language-specific executor. This is stateful beyond plain text replacement. Refs: `~/.emacs.d/.local/straight/repos/org/lisp/ob-core.el:673-678`, `:814-940`, `:1993-1997`.

### lsp-mode

- Rename is a request on the current text document and current cursor position, then a workspace edit apply. `lsp-rename` sends `textDocument/rename` with `lsp--text-document-identifier` and `lsp--cur-position`, then feeds the response to `lsp--apply-workspace-edit`. Refs: `~/.emacs.d/.local/straight/repos/lsp-mode/lsp-mode.el:4417-4423`, `:4503-4514`, `:6819-6826`.
- Position conversion is encoding-dependent and buffer-local. lsp-mode negotiates per-buffer position encoders and decoders for UTF-16, UTF-8, or UTF-32; all rename/diagnostic/edit positioning depends on these functions rather than raw Emacs point offsets. Refs: `~/.emacs.d/.local/straight/repos/lsp-mode/lsp-mode.el:4431-4501`, `:1880-1882`.
- Workspace edits are version-checked before application. `lsp--check-document-changes-version` rejects document changes when the versioned identifier does not match `lsp--cur-version` for the visited buffer. `lsp--apply-workspace-edit` runs this check before applying `create`, `rename`, `delete`, or text edits. Refs: `~/.emacs.d/.local/straight/repos/lsp-mode/lsp-mode.el:4532-4547`, `:4548-4558`, `:4583-4628`.
- Text edits apply by translated ranges, descending order, and direct buffer mutation. `lsp--apply-text-edit` converts the range to points, deletes that region, and inserts `new-text`. Probe code should treat stale ranges or wrong document versions as hard failures, not soft warnings. Refs: `~/.emacs.d/.local/straight/repos/lsp-mode/lsp-mode.el:4652-4664`.
- Server-initiated `workspace/applyEdit` goes through the same local edit path and only reports `:applied` if no error was signaled. Refs: `~/.emacs.d/.local/straight/repos/lsp-mode/lsp-mode.el:7201-7208`.

### Jupyter

- Source/repl association is buffer-local via `jupyter-current-client`. The variable is permanent-local and also dynamically rebound during message handling. `jupyter-repl-associate-buffer` sets the current buffer's `jupyter-current-client` only if the major mode matches the REPL client's kernel language mode. Refs: `~/.emacs.d/.local/straight/repos/jupyter/jupyter-client.el:148-154`, `:239-243`, `~/.emacs.d/.local/straight/repos/jupyter/jupyter-repl.el:2006-2032`.
- REPL persistence propagates client association across compatible buffers and server-opened files, which means association can move outside the original REPL buffer. `jupyter-repl-persistent-mode` advises buffer switching/display and `jupyter-server-mode-set-client` temporarily sets `jupyter-current-client` in `server-buffer`; the file comment explicitly says this is imperfect for overlapping requests/clients. Refs: `~/.emacs.d/.local/straight/repos/jupyter/jupyter-repl.el:2033-2085`, `:2121-2141`, `~/.emacs.d/.local/straight/repos/jupyter/jupyter-base.el:656-697`.
- Kernel identity is not just kernelspec name. Server models expose `:id`, `:name`, `:last_activity`, `:execution_state`, and connections. `jupyter-launch` on a `jupyter-server-kernel` distinguishes connecting to an already-running kernel by existing `id` versus starting a new kernel and replacing the kernel ID; the object also gains a fresh session afterward. Refs: `~/.emacs.d/.local/straight/repos/jupyter/jupyter-server.el:224-233`, `:370-400`, `~/.emacs.d/.local/straight/repos/jupyter/jupyter-server-kernel.el:335-370`.
- Request lifecycle is explicit state, not inferred from output buffers. `jupyter-request` stores a generated request ID, type, content, client, sent time, idle flag, last message, and accumulated messages. Idle completion is defined by receipt of a status: idle message, and helper APIs block on that state. Refs: `~/.emacs.d/.local/straight/repos/jupyter/jupyter-base.el:467-491`, `:503-524`, `~/.emacs.d/.local/straight/repos/jupyter/jupyter-client.el:542-575`.
- Message causality is carried by headers and parent headers. Message encoding sets `:msg_id`, `:msg_type`, `:session`, and decode eagerly exposes `:parent_header`; helpers provide `jupyter-message-id`, `jupyter-message-parent-id`, and `jupyter-message-status-idle-p`. Faithful probes should match outputs/replies to parent IDs, not just arrival order. Refs: `~/.emacs.d/.local/straight/repos/jupyter/jupyter-messages.el:127-140`, `:352-366`, `:621-669`.
- Synchronous eval deliberately inhibits normal handlers and history. `jupyter-eval` sends `execute_request` with `:store-history nil` and `:handlers nil`, then only waits for subscribed messages such as `execute_reply`; this is a different contract from interactive REPL execution. Refs: `~/.emacs.d/.local/straight/repos/jupyter/jupyter-client.el:760-783`.

### Comint

- The process mark is the ownership boundary for pending input versus historical text. `comint-send-input` sends text from process mark to point when point is after the mark; otherwise it calls `comint-get-old-input`, reinserts that text at the process mark, and sends the copy. Refs: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/comint.el.gz:1886-1956`, `:2421-2451`.
- Sending input updates history, field properties, last-input markers, and the process mark before the transport send. This bookkeeping is part of the contract; raw `process-send-string` is insufficient. Refs: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/comint.el.gz:1986-2035`, `:2593-2601`.
- `completion-in-region-mode` is force-disabled before send. If a probe accepts inline completion and immediately submits, it must preserve the same completion teardown path as `comint-send-input`. Ref: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/comint.el.gz:1936-1941`.

### Dired

- Marks and deletion flags are buffer text state, not filesystem state. `dired-mark` and `dired-unmark` literally replace the first character on each file line with `dired-marker-char` or space. Refs: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/dired.el.gz:4339-4422`.
- Deletion acts in two phases: gather marked entries and markers, then perform filesystem deletion, then delete Dired entries in all relevant Dired buffers. `dired-do-flagged-delete` and `dired-do-delete` both pass an alist of absolute filenames plus point markers into `dired-internal-do-deletions`. Refs: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/dired.el.gz:3959-4009`.
- Undo only restores Dired buffer presentation, not lost files. `dired-undo` explicitly says it can recover marks, killed lines, or subdirs, but cannot undo actual file changes. Refs: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/dired.el.gz:2823-2832`, `:4014-4059`.

### Completion-at-point and ghost-text

- CAPF providers return `(START END COLLECTION . PROPS)` and must preserve point/current buffer. `:exclusive no` means a failing provider should yield to later providers, but Emacs approximates failure with `try-completion` on the current prefix, which can mis-model non-prefix backends. Refs: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/minibuffer.el.gz:2918-2979`.
- CAPF state validity is anchored to the provider returning the same start boundary on subsequent calls. `completion-at-point` stores the original `start` as a marker and keeps completion active only while the provider still reports that same field start. Refs: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/minibuffer.el.gz:2986-3003`, `:2856-2904`.
- Completion metadata owns ordering, annotation, grouping, and exit callbacks, and should remain constant within a boundary interval. Refs: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/minibuffer.el.gz:94-170`, `:2451-2504`.
- Stock ghost text in Emacs 30 is `completion-preview-mode`, implemented as an overlay that is driven by `completion-at-point-functions`. It stores CAPF-derived bounds, suffix list, common prefix, and props on the overlay itself. Refs: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/completion-preview.el.gz:221-236`, `:270-366`.
- Preview freshness is explicitly guarded. Before showing a new preview, `completion-preview--show` checks whether buffer text is still a prefix of the previously suggested candidate; if not, it hides the overlay so stale ghost text is not shown. Refs: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/completion-preview.el.gz:380-450`.
- Accept paths can still run arbitrary provider exit logic. `completion-preview-insert` calls the CAPF `:exit-function` with status `finished`; `completion-preview-complete` hides the preview before invoking a possibly slow or side-effecting exit function and carries a bug note about stale previews after arbitrary text/point modification. Refs: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/completion-preview.el.gz:452-469`, `:471-539`.

## Probe implications

- Magit probes should compare direct function calls versus genuine command-loop invocation, with `this-command` and post-refresh hook execution captured, not just Git index deltas.
- Org probes need dual-buffer assertions: parent buffer region contents, marker survival, capture/refile target markers, and ID database effects.
- LSP rename probes should record negotiated position encoding, pre/post document versions, and whether edits were rejected due to version mismatch.
- Jupyter probes should log request ID, parent header ID, kernel ID, session/client identity, idle transition, and whether evaluation used interactive REPL or synchronous helper paths.
- Comint probes should distinguish submission from process mark versus historical resend, and verify marker/history bookkeeping rather than only subprocess bytes.
- Dired probes should separately grade buffer flags/marks, filesystem mutations, and the non-recoverability of deleted files through `dired-undo`.
- Completion probes should mutate text between preview/update/accept steps and verify whether the same CAPF start boundary still holds before accepting ghost text.

## Gaps and uncertainties

- I did not inspect Doom-specific advice, Evil remappings, or live buffer-local hook state in an interactive session; this report is package-source grounded, not live-session traced.
- I did not inspect non-stock ghost-text frontends such as Corfu/Company/Copilot overlays; the ghost-text findings above are for stock Emacs 30 `completion-preview`.
- I did not run Jupyter kernels or LSP servers here, so asynchronous ordering claims come from local source semantics rather than recorded live traces.

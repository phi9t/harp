# Emacs Core Session Semantics

## Environment identity inspected

- Binary: `/private-source/home/operator/devx/envs/doom-emacs/.spack-env/view/bin/emacs`
- Version: GNU Emacs 30.2, `system-configuration=x86_64-pc-linux-gnu`
- Installed tree:
  - Lisp: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/`
  - Info manuals: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/info/`
  - Exec dir: `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/libexec/emacs/30.2/x86_64-pc-linux-gnu/`
- `source-directory` is stale on this host:
  - `/private-source/home/operator/devx/stage/spack-stage-emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/spack-src/`
  - That path no longer exists, so local C-source citations are not available from this installation.

## Source paths and entry points

- Command dispatch:
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/simple.el.gz:2596-2613`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/simple.el.gz:2753-2805`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/emacs-lisp/nadvice.el.gz:170-181`
- Buffers, windows, redisplay:
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/info/elisp.info.gz:911-926`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/info/elisp.info.gz:3613-3617`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/window.el.gz:7908-7927`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/window.el.gz:9086-9203`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/simple.el.gz:7242-7264`
- Indirect buffers, undo/property handling:
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/simple.el.gz:10622-10706`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/simple.el.gz:3648-3686`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/info/elisp.info.gz:1053-1074`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/info/elisp.info.gz:1371-1376`
- Timers, timeout, async:
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/emacs-lisp/timer.el.gz:475-505`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/emacs/30.2/lisp/minibuffer.el.gz:2762-2781`
  - `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/share/info/elisp.info.gz:1306-1311`

## Findings with citations

### 1. Lisp runtime implicit context and dynamic variables

- Emacs explicitly distinguishes the current buffer from buffers shown in windows. The Elisp manual frames `Current Buffer` as the object primitives implicitly act on, while windows separately track what is displayed. See `elisp.info.gz:911-926` and `elisp.info.gz:3613-3617`.
- `switch-to-buffer` warns not to use it for temporary Lisp work, and says to use `set-buffer` instead so code does not perturb `window-buffer` correspondences. It ends by both updating window display state and calling `set-buffer`, so it is a UI/state transition, not just a dynamic rebinding. See `window.el.gz:9086-9203`.
- `with-temp-buffer` and related macros rely on `with-current-buffer`, confirming that a large part of the Lisp runtime contract is implicit dynamic buffer context rather than explicit buffer parameters. See `subr.el.gz:5091-5104`.
- `with-silent-modifications` binds `buffer-undo-list`, `inhibit-read-only`, and `inhibit-modification-hooks`, which is a concrete example of the dynamic context surface the gym must preserve when it claims to be “silent” or “non-observing”. See `subr.el.gz:5120-5128`.

### 2. Current buffer vs selected window vs displayed buffer

- The manual states that the selected window “usually displays the current buffer, but this is not necessarily the case.” See `elisp.info.gz:3613-3617`.
- `display-buffer` composes policy from `display-buffer-overriding-action`, user actions, special actions, and fallback actions. Showing a buffer is therefore policyful and may select or reuse windows according to dynamic ambient state. See `window.el.gz:7908-7927`.
- `pre-redisplay-functions` runs with `current-buffer` rebound to the buffer shown in each window being redisplayed. That means observation hooks can execute in per-window buffer context even if the initiating code started elsewhere. See `simple.el.gz:7242-7264`.
- Live batch probe on this install showed the split directly:
  - `current-buffer` was `" *probe-base*"`
  - `(window-buffer (selected-window))` was `" *probe-other*"`
  - This was produced by `emacs -Q --batch` after `set-window-buffer` followed by `set-buffer`, so the divergence is real even without Doom loaded.

### 3. Buffer identity, text identity, and indirect buffers

- The manual says indirect buffers “share text with some other buffer.” See `elisp.info.gz:925-926` and `elisp.info.gz:3570-3571`.
- `clone-buffer` copies text and then restores narrowing, point, mark, buffer-local variables, and mode state into a fresh buffer. See `simple.el.gz:10622-10669`.
- `clone-indirect-buffer` instead calls `make-indirect-buffer`, which preserves shared text identity while creating a distinct buffer object with its own window/buffer-local session state. See `simple.el.gz:10672-10706`.
- Live probe on this install:
  - base buffer had `buffer-base-buffer => nil`
  - indirect buffer had `buffer-base-buffer => " *probe-text*"`
  - narrowing applied in the indirect buffer did not destroy the base buffer object
- Implication: the gym cannot treat “buffer state” as one string per file. It needs at least:
  - buffer object identity
  - base text identity
  - visited-file identity
  - per-buffer narrowing and locals

### 4. Markers, narrowing, modified ticks, text properties, overlays

- The Elisp manual exposes distinct subsystems for `Narrowing`, `Markers`, and `Overlays`; those are not reducible to raw buffer text. See `elisp.info.gz:1053-1074` and `elisp.info.gz:1371-1376`.
- Undo records property changes separately from text insertion/deletion, via `(nil PROP VAL BEG . END)` entries that replay `put-text-property`. It also guards against undo helpers switching buffers mid-flight. See `simple.el.gz:3650-3686`.
- Live probes on this install showed:
  - Inserting `"Z"` before a marker moved marker position from `3` to `4`.
  - `put-text-property` changed `buffer-modified-tick` from `4` to `5` while leaving `buffer-chars-modified-tick` at `4`.
  - Subsequent character insertion raised both ticks to `6`.
  - `narrow-to-region` and `widen` changed `point-min`/`point-max` but left both ticks unchanged.
  - Creating an overlay added overlay state without changing text content.
- Implication: a probe layer that only snapshots characters will miss:
  - property-only mutations
  - restriction changes
  - overlay state
  - marker relocation

### 5. Command dispatch routes, command loop state, and `this-command`/`last-command`

- `execute-extended-command` explicitly seeds `this-command`, `real-this-command`, and synthetic `this-command-keys` before calling `command-execute`. See `simple.el.gz:2596-2613`.
- `command-execute` normalizes prefix arguments into `current-prefix-arg`, clears `prefix-arg` for ordinary commands, resolves autoloads, and for ordinary commands ends up in `call-interactively`. See `simple.el.gz:2753-2805`.
- `nadvice` documents that re-evaluating interactive specs in Lisp is not fully faithful to the C path in `call-interactively`; it specifically calls out subtle differences in `command-history` and possibly other details. See `nadvice.el.gz:170-181`.
- `undo-auto-amalgamate` keys off `(eq this-command last-command)` when deciding whether to merge undo boundaries, so command-loop identity is not cosmetic metadata. See `simple.el.gz:4103-4127`.
- Important caveat from live probing:
  - In `emacs -Q --batch`, direct `funcall`, `call-interactively`, and `command-execute` all left `this-command`, `real-this-command`, and `called-interactively-p` effectively nil in the probe body.
  - That is evidence that batch mode is not a faithful substitute for the PTY-driven command loop when this ticket needs real session semantics.

### 6. Reentrancy, process filters, timers, `with-timeout`, recursive edit

- `with-timeout` is timer-backed and only notices timeouts when Emacs waits for an external event; a busy loop with no wait point will not time out. See `timer.el.gz:478-505`.
- `minibuffer-quit-recursive-edit` quits by throwing an exit continuation, which is an explicit reentrancy/stack-unwinding path rather than a simple return. See `minibuffer.el.gz:2762-2773`.
- The manual separates process filters from `accept-process-output`, reinforcing that async process delivery is a first-class subsystem. See `elisp.info.gz:1306-1311`.
- Live probes on this install showed:
  - timer callback ran with `current-buffer="*scratch*"` and selected-window buffer `"*scratch*"`
  - process filter ran with `current-buffer="*scratch*"` while `(process-buffer p)` was `" *probe-proc*"`
  - inside the process filter, `inhibit-quit` was `t`
- Implication: async callbacks cannot assume they inherit the caller’s buffer/window context. Probe code must bind the target buffer/window explicitly before touching session-sensitive state.

### 7. Observation effects and redisplay

- `pre-redisplay-functions` are per-window and rebind `current-buffer` to the displayed buffer. See `simple.el.gz:7242-7264`.
- `switch-to-buffer` and `display-buffer` are display actions, not pure reads. Even inspecting or presenting a buffer can change selection, recent-buffer history, window state, or which buffer subsequent hooks run in. See `window.el.gz:7908-7927` and `window.el.gz:9086-9203`.
- Implication: “observe without changing session state” is only credible if the bridge stays off display-changing APIs or accounts for their side effects explicitly.

## Probe implications

- The gym state model should track at minimum:
  - current buffer
  - selected window
  - displayed buffers per window
  - base-buffer versus indirect-buffer identity
  - visited file identity
  - point, mark, and marker-bearing state
  - narrowing
  - modified ticks and chars-modified ticks
  - text properties and overlays
  - dynamic variables relevant to dispatch such as `current-prefix-arg`, `this-command`, `last-command`, `inhibit-read-only`, and `inhibit-modification-hooks`
- Bridge operation tiers:
  - Pure internal reads can often use `with-current-buffer`.
  - Session/UI moves must distinguish `set-buffer` from `switch-to-buffer` or `display-buffer`.
  - Command-sensitive package actions should not be validated only with batch `funcall` or even batch `command-execute`; they need real command-loop or PTY-path probes.
- Async bridge code should treat timers and process filters as reentrant entry points with unspecified current buffer/window unless it binds them deliberately.

## Gaps and uncertainties

- No local Emacs C source checkout was available. The installation’s `source-directory` points to a removed Spack stage, so I could not cite `src/eval.c`, `src/keyboard.c`, `src/lread.c`, `src/buffer.h`, `src/insdel.c`, or `src/xdisp.c` from local files.
- Because this was a subagent run in batch mode, I did not validate true PTY input decoding, pre-command/post-command hook timing, key remapping precedence, recursive minibuffer interaction, Evil state transitions, or Doom advice layers.
- The command-loop findings above are therefore split into:
  - source-grounded dispatch structure from installed Lisp/manuals
  - batch-mode observations proving that batch is not the same semantic environment as an interactive terminal session

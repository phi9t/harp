# Doom Lifecycle Notes For Emacs Gym

## Environment identity inspected

- Host `emacs` resolves to `/private-source/home/operator/devx/envs/doom-emacs/.spack-env/view/bin/emacs`, which resolves to Spack Emacs 30.2 at `/private-source/home/operator/devx/spack-v1.2.2/opt/spack/linux-icelake/emacs-30.2-pktkld2qki65epklgjqyxfuh4iawlxs5/bin/emacs-30.2`.
- Doom core is a git checkout at `~/.emacs.d` on branch `master` (`~/.emacs.d/.git/HEAD:1`), with local `HEAD` observed as `8b475fcb2574286d7f4413c22cc7902d80d21ced`.
- Doom metadata reports `3.0.0-pre` and pins straight itself in `~/.emacs.d/.doom:1-3` and `~/.emacs.d/.doom:110-116`.
- User config is `~/.doom.d`; no explicit `~/.doom.d/profiles.el` was present. Profile discovery falls back to Doom’s configured search path in `~/.emacs.d/lisp/lib/profiles.el:12-21`.

## Source paths and entry points

- Bootstrap starts in `~/.emacs.d/early-init.el`, which handles `--profile`, `--init-directory`, GC deferral, and `doom-initialize` for normal startup (`~/.emacs.d/early-init.el:29-44`, `~/.emacs.d/early-init.el:60-100`, `~/.emacs.d/early-init.el:142-149`).
- Profile generation logic lives in `~/.emacs.d/lisp/lib/profiles.el`; generated profile init files are written by `doom-profile-generate` (`~/.emacs.d/lisp/lib/profiles.el:255-313`).
- The current generated global-profile init is `~/.emacs.d/.local/etc/@/init.30.2.el`, with a sync/version guard at `~/.emacs.d/.local/etc/@/init.30.2.el:1-5`.
- The generated profile cache file is `~/.emacs.d/.local/cache/profiles.@.el`; current contents show only an implicit `safe-mode` profile entry, not an active named user profile (`~/.emacs.d/.local/cache/profiles.@.el:1`).
- User module selection is in `~/.doom.d/init.el:17-198`. User package/config entrypoints are `~/.doom.d/packages.el:56-60` and `~/.doom.d/config.el:40-56`.

## Findings

### 1. Core topology is “global Doom install + private user dir”, not a named profile workflow

- Doom’s profile loader searches `~/.doom.d/profiles.el`, `~/.emacs.d/profiles.el`, `~/.config/doom-profiles.el`, `~/.doom-profiles.el`, and implicit profile directories under both Doom dirs (`~/.emacs.d/lisp/lib/profiles.el:12-21`).
- The generated profile cache only records an implicit `safe-mode` profile and does not indicate an active named profile (`~/.emacs.d/.local/cache/profiles.@.el:1`).
- The generated init under `~/.emacs.d/.local/etc/@/init.30.2.el` is therefore the active “global” profile init for this checkout, matching Doom’s profile README fallback semantics (`~/.emacs.d/profiles/README.org:48-58`).

### 2. User module set is non-Evil, Corfu+Vertico, popup/workspaces, Magit+Forge, Org+Jupyter, TTY

- Enabled modules/flags in the user `doom!` form:
  - Completion: `(corfu +orderless)` and `vertico` (`~/.doom.d/init.el:23-30`)
  - UI: `doom`, `dashboard`, `hl-todo`, `modeline`, `ophints`, `(popup +defaults)`, `(vc-gutter +pretty)`, `vi-tilde-fringe`, `workspaces` (`~/.doom.d/init.el:31-54`)
  - Editor: `file-templates`, `fold`, `snippets`, `(whitespace +guess +trim)`; `evil` is commented out (`~/.doom.d/init.el:56-69`)
  - Tools: `ein`, `(eval +overlay)`, `lookup`, `lsp`, `(magit +forge)`, `tmux` (`~/.doom.d/init.el:91-110`)
  - OS/lang: `tty`, `(org +jupyter)`, `(python +lsp +pyright +uv)`, `(cc +lsp)`, `(go +lsp)`, `(rust +lsp)`, `(sh +lsp)` (`~/.doom.d/init.el:113-198`)
- The generated init snapshot also records `:ui popup (+defaults)`, `:os tty`, `:tools magit (+forge)`, `:lang org (+jupyter)`, and no `:editor evil` in its serialized `doom-modules` table (`~/.emacs.d/.local/etc/@/init.30.2.el:5`).

### 3. Package state is straight-based, shallow-cloned, and locally warmed; no local lockfile snapshot was found

- Doom uses straight, sets `straight-base-dir` to `doom-local-dir`, uses per-Emacs-version build dirs, disables modification checks, and shallow clones by default (`~/.emacs.d/lisp/doom-straight.el:39-58`).
- Doom overlays package `:pin` declarations on top of straight lockfiles by advising `straight--lockfile-read-all` (`~/.emacs.d/lisp/doom-straight.el:74-78`).
- Core package metadata pins `straight.el` itself in `~/.emacs.d/.doom:110-116`.
- User package declarations are not inline; `~/.doom.d/packages.el` loads `~/workspace/devx/doom/packages.el` and hard-errors if it is missing (`~/.doom.d/packages.el:56-60`).
- The external package file pins `copilot.el` and `tabbymacs` (`~/workspace/devx/doom/packages.el:4-13`).
- I did not find any local `versions/*.el`, `versions.el`, or other straight lock snapshot under `~/.emacs.d/.local/straight`; the current machine state is therefore best described by the live repo checkouts plus `build-30.2-cache.el`, not by a checked-in lockfile.
- Selected installed package heads observed directly from local repos:
  - `magit` `c800f79c2061621fde847f6a53129eca0e8da728`
  - `forge` `315e8e9a2b45d050ca7fc717595cc698e175b140`
  - `transient` `1f7039ef8d548d6fe858084fcbeae7588eba4190`
  - `vertico` `93f15873d7d6244d72202c5dd7724a030a2d5b9a`
  - `corfu` `abfe0003d71b61ffdcf23fc6e546643486daeb69`
  - `lsp-mode` `197f56bf7a38b564cc5dec26d66b5f494da39783`
  - `jupyter` `242fdc709ce0faa3b9ee81dcc48cfd791898e6b8`
  - `copilot.el` `90f429b100418d897b673d6c36c2817364b28e3e`
  - `tabbymacs` `32b8c40774515c940a02c78874cf928b356478de`
  - `evil` `729d9a58b387704011a115c9200614e32da3cefc`
  - `evil-collection` `d052ad2ec1f6a4b101f873f01517b295cd7dc4a9`

### 4. `doom!`, `package!`, `use-package!`, `after!`, and `map!` expansion points are straightforward and important for the gym

- `doom!` only mutates module state in noninteractive evaluation; it does not itself run full startup in interactive sessions (`~/.emacs.d/lisp/doom-lib.el:1528-1547`).
- `package!` is declarative metadata, not an installer/loader (`~/.emacs.d/lisp/doom-lib.el:1601-1605`).
- `use-package!` is a thin wrapper over `use-package` and is skipped for disabled or compile-time-missing packages (`~/.emacs.d/modules/config/use-package/init.el:132-163`).
- `after!` expands to `with-eval-after-load` and explicitly no-ops for disabled packages (`~/.emacs.d/lisp/doom-lib.el:880-923`).
- `map!` is a `general`-powered convenience macro; its own docstring says Evil-specific bindings are ignored when Evil is not loaded (`~/.emacs.d/lisp/doom-keybinds.el:439-476`).

### 5. Cold-start versus warmed/first-use behavior is visible in both source and local cache state

- Early startup defers GC aggressively, with different thresholds for noninteractive versus interactive startup (`~/.emacs.d/early-init.el:29-44`).
- The generated init is rebuilt by `doom sync`/`doom profile sync`; it inlines startup vars, autoload/loaddefs, and a generated `doom-startup` function (`~/.emacs.d/lisp/lib/profiles.el:255-313`).
- Corfu is not globally enabled at process start; it is enabled from `doom-first-input` via `global-corfu-mode` (`~/.emacs.d/modules/completion/corfu/config.el:40-42`).
- Magit is explicitly `:defer-incrementally` on several dependencies, and Forge is loaded only after `magit-status` is called (`~/.emacs.d/modules/tools/magit/config.el:41-49`, `~/.emacs.d/modules/tools/magit/config.el:195-203`).
- Local warmed-state evidence:
  - `~/.emacs.d/.local/etc/sync` mtime `2026-03-19`
  - `~/.emacs.d/.local/cache/profiles.@.el` mtime `2026-09-13`
  - `~/.emacs.d/.local/etc/@/init.30.2.el` mtime `2026-09-16`
  - `~/.emacs.d/.local/straight/build-30.2-cache.el` mtime `2026-09-16`
  - `~/.emacs.d/.local/env` was regenerated during this investigation on `2026-09-27` by `doom env`
- That split suggests the machine is not at “fresh install” state. The gym should expect precomputed generated init/load-path/build caches, plus first-use lazy activation of packages like Corfu, Magit, Forge, and language integrations.

### 6. Popup, TTY, Magit, and Org behavior materially affect command-path fidelity

- Popup mode installs its own `display-buffer-alist` rules and window parameters when `+popup-mode` turns on (`~/.emacs.d/modules/ui/popup/config.el:51-70`).
- Default popup rules route completions, messages, doom transient buffers, `*doom:*`, man/help/info, etc. into specific popup slots/sizes (`~/.emacs.d/modules/ui/popup/config.el:129-167`).
- In popup buffers, `[escape]` is only bound to `doom/escape` when Evil is present, and the comment explicitly says tty Emacs should use `C-g` instead (`~/.emacs.d/modules/ui/popup/config.el:42-49`).
- TTY mode enables mouse support, clipboard integration (`clipetty` or `xclip`), terminal cursor-shape updates, and Kitty keyboard protocol support (`~/.emacs.d/modules/os/tty/config.el:13-19`, `~/.emacs.d/modules/os/tty/config.el:24-36`, `~/.emacs.d/modules/os/tty/config.el:44-59`).
- Corfu’s tty fallback is `corfu-terminal` when `:os tty` is enabled and `tty-child-frames` is unavailable (`~/.emacs.d/modules/completion/corfu/config.el:220-223`).
- Magit relocates transient files into Doom profile data dir, overrides Magit display routing, and ignores popup handling for Magit/transient buffers so Magit uses its own display logic (`~/.emacs.d/modules/tools/magit/config.el:46-49`, `~/.emacs.d/modules/tools/magit/config.el:121-140`).
- Org adds popup rules for source, Babel, and capture buffers (`~/.emacs.d/modules/lang/org/config.el:991-1000`).

### 7. Evil-specific code exists in installed packages, but current user config disables the Evil module

- The user `doom!` form comments out `evil`, so Evil-dependent module code should not be active in the current config (`~/.doom.d/init.el:56-58`).
- Non-Evil defaults set `doom-leader-alt-key` to `C-c` and `doom-localleader-alt-key` to `C-c l` (`~/.emacs.d/modules/config/default/+emacs-bindings.el:3-5`).
- Doom still ships rich Evil overlays for smart TAB, minibuffer bindings, leader maps, Org, and Magit, but they are gated on `(modulep! :editor evil +everywhere)` or related checks (`~/.emacs.d/modules/config/default/+evil-bindings.el:3-32`, `~/.emacs.d/modules/config/default/+evil-bindings.el:41-120`, `~/.emacs.d/modules/tools/magit/config.el:241-314`, `~/.emacs.d/modules/lang/org/config.el:1084-1125`).
- Implication: the gym should model this host as non-Evil by default unless it intentionally launches another profile/config. Using Evil-centric assumptions for baseline keyboard dispatch would be inaccurate for the inspected setup.

## Probe implications

- Use real Doom startup, not naked `emacs --batch`, when probing or grading. Live probe result: plain `emacs --batch` on this host did not populate Doom variables, while forcing `-l ~/.emacs.d/early-init.el` did. For the gym, the faithful startup path is normal Emacs startup or an explicit Doom bootstrap path.
- Treat generated files under `~/.emacs.d/.local/etc/@/` and `~/.emacs.d/.local/cache/` as part of the runtime contract. They are not optional implementation details; they materially define load-paths, module graph, autoloads, and startup functions.
- Model lazy activation points, not just final loaded features. Corfu turns on at first input; Magit and Forge are deferred; Org and popup behavior add mode-specific window routing; TTY extras change key decoding and clipboard behavior.
- Model popup/transient completion behavior as part of the environment. Magit and popup windows do not follow a simple “open buffer in current window” contract, and tty `escape` behavior differs from GUI assumptions.
- Baseline gym keybinding expectations should be non-Evil for this inspected config, with `C-c`/`C-c l` style leader alternatives and Vertico/Corfu completing the dominant interaction path.

## Gaps and uncertainties

- I did not attach to a live interactive user session; conclusions are from local source, generated state, filesystem evidence, and batch probes.
- Evaluating `~/.emacs.d/.local/etc/@/init.30.2.el` standalone in batch hit a `wrong-type-argument (hash-table-p nil)` path during user init loading, so I treated the generated file primarily as source evidence rather than a fully replayable standalone bootstrap.
- No local straight lock snapshot (`versions/*.el`) was found, so package “lock state” here is best described as the current repo heads plus explicit `:pin` declarations, not as a persisted lockfile.
- I did not exhaustively expand every `map!` use site or trace all `use-package!` expansions at runtime; the cited macro definitions establish the expansion points the gym should target.

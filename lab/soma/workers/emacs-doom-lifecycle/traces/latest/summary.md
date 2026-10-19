# Doom Lifecycle Worker Summary

- Generated: 2026-09-29T03:52:42Z
- Status: `pass`
- Runtime: selected CUDA-image-derived Soma Insula rootfs
- Emacs: source-built `/opt/soma/emacs/bin/emacs`
- Doom core: rootfs-owned `/project/home/.emacs.d`
- Doom user config: rootfs-owned `/project/home/.doom.d`

- Children: `4`
- Teardown probe: `pass`

## Checks

- `cold child exited successfully`: `t` 0
- `warmed child exited successfully`: `t` 0
- `cold-repeat-1 child exited successfully`: `t` 0
- `warmed-repeat-1 child exited successfully`: `t` 0
- `cold trace was emitted`: `t` /workspace/soma/workers/emacs-doom-lifecycle/traces/latest/cold.json
- `warmed trace was emitted`: `t` /workspace/soma/workers/emacs-doom-lifecycle/traces/latest/warmed.json
- `cold-repeat-1 trace was emitted`: `t` /workspace/soma/workers/emacs-doom-lifecycle/traces/latest/cold-repeat-1.json
- `warmed-repeat-1 trace was emitted`: `t` /workspace/soma/workers/emacs-doom-lifecycle/traces/latest/warmed-repeat-1.json
- `cold trace checks passed`: `t` pass
- `warmed trace checks passed`: `t` pass
- `cold-repeat-1 trace checks passed`: `t` pass
- `warmed-repeat-1 trace checks passed`: `t` pass
- `cold child token was unique before launch`: `t` nil
- `cold child left no token-marked processes after exit`: `t` nil
- `warmed child token was unique before launch`: `t` nil
- `warmed child left no token-marked processes after exit`: `t` nil
- `cold-repeat-1 child token was unique before launch`: `t` nil
- `cold-repeat-1 child left no token-marked processes after exit`: `t` nil
- `warmed-repeat-1 child token was unique before launch`: `t` nil
- `warmed-repeat-1 child left no token-marked processes after exit`: `t` nil
- `child teardown tokens are pairwise unique`: `t` ((("phase" . "cold") ("token" . "8277edde2bea64e53c4b0ce42d05fccb380f7c0dcde4bf4bb140a4310eda4e4a") ("status" . "pass") ("preexisting_count" . 0) ("remaining_count" . 0)) (("phase" . "warmed") ("token" . "6e007e7eda0d5db34b08d13726408fc3068d5ba8573930ae265ae700c0fcfa95") ("status" . "pass") ("preexisting_count" . 0) ("remaining_count" . 0)) (("phase" . "cold-repeat-1") ("token" . "48fcbe587f3082db56c3ca249ff8c2536d7349e3582782be3523e77ba4149e3a") ("status" . "pass") ("preexisting_count" . 0) ("remaining_count" . 0)) (("phase" . "warmed-repeat-1") ("token" . "6c45a4102b4fe0cf9205434a40864cc9c60ad5699c6680d8901f817e649843f3") ("status" . "pass") ("preexisting_count" . 0) ("remaining_count" . 0)))

## Warmed First Use

- require `magit`: `loaded`
- require `org`: `loaded`
- require `jupyter`: `loaded`
- require `lsp-mode`: `loaded`
- require `python`: `loaded`
- require `rust-mode`: `loaded`
- require `cc-mode`: `loaded`
- require `sh-script`: `loaded`
- mode `emacs-lisp-mode`: `activated` -> `emacs-lisp-mode`
- mode `org-mode`: `activated` -> `org-mode`
- mode `python-mode`: `activated` -> `python-mode`
- mode `rust-mode`: `activated` -> `rust-mode`
- mode `c++-mode`: `activated` -> `c++-mode`
- mode `sh-mode`: `activated` -> `sh-mode`

## Capability Module State

- `corfu`: user-init `:json-false`, loaded-profile `:json-false`, text-token `:json-false`
- `vertico`: user-init `:json-false`, loaded-profile `:json-false`, text-token `:json-false`
- `popup`: user-init `:json-false`, loaded-profile `:json-false`, text-token `t`
- `tty`: user-init `:json-false`, loaded-profile `:json-false`, text-token `:json-false`
- `magit`: user-init `t`, loaded-profile `t`, text-token `t`
- `org+jupyter`: user-init `t`, loaded-profile `t`, text-token `t`
- `lsp`: user-init `t`, loaded-profile `t`, text-token `t`
- `python`: user-init `t`, loaded-profile `t`, text-token `t`
- `rust`: user-init `t`, loaded-profile `t`, text-token `t`
- `cc`: user-init `t`, loaded-profile `t`, text-token `t`
- `sh`: user-init `t`, loaded-profile `t`, text-token `t`

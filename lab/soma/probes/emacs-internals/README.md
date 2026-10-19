# Emacs Internals Probes

This directory contains executable mechanism probes for the Emacs/Doom
Internals Atlas.

Run the current batch-safe probe suite with:

```sh
probes/emacs-internals/scripts/run-probes.sh
```

This public entrypoint uses the selected rootfs Insula by default. Set
`SOMA_USE_DEVELOPMENT_INSULA=1` only when deliberately regenerating the
development-minimal host-bind classification.

The runner writes one JSON trace per probe plus `summary.json` and
`summary.md` under `probes/emacs-internals/traces/latest/` by default.

The current suite uses `emacs --quick --batch` so it can run without touching
the user's live Doom session. Records that involve terminal-only behavior state
that limit explicitly in their trace and derive a later terminal-frame gate.

Current authoritative trace: 36 probes, 36 passing, with Insula runtime checks
passing under `selected-container-rootfs`.

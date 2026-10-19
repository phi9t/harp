# 0010: Evaluate The Emacs Patch Observability Spike

Status: Open
Type: Follow-up task
Depends on: 0001 Emacs/Doom Internals Atlas; 0002 Soma Insula Contract Spike; 0009 Spack Rootfs Materialization Spike

## Intent

Decide whether Soma needs a source-pinned patched Emacs build to improve
operator observability or operability.

The default answer is stock Emacs plus Lisp, advice, hooks, process logs,
protocol logs, native modules where appropriate, PTY capture, and independent
checkers. This issue exists to test the cases where that stack may not expose
enough evidence without perturbing the behavior we need to measure.

## Requirements

- Pin a matching Emacs source tree for the selected runtime before making
  C-level claims or patches.
- Build a stock Emacs baseline inside the selected Soma rootfs or an explicitly
  labeled development rootfs.
- Identify observability gaps with executable probes before proposing patches.
  Candidate gaps include:
  - command-loop state and `this-command`/`last-command` transitions
  - key dispatch, remapping, minibuffer, and recursive-edit state
  - buffer text identity, modification counters, markers, overlays, and
    narrowing
  - process filter reentry, timers, sentinels, and async request lifecycle
  - redisplay attempt/completion evidence and terminal-frame behavior
  - native-comp and package-load events relevant to Doom activation
- Prefer non-invasive instrumentation first: Elisp hooks/advice, narrow source
  debugging, protocol logs, module-level probes, PTY logs, and supervisor-owned
  state checks.
- If a patch is proposed, keep it minimal and observational unless a separate
  operability patch is explicitly justified. The patch must not encode task
  answers or benchmark-specific shortcuts.
- Treat a patched Emacs as a distinct Insula runtime. Record source revision,
  patch files, configure flags, compiler/toolchain identity, native-comp and
  libgccjit closure, shared-library ABI surface, tree hash, and executable hash.
- Run stock and patched builds against the same fixtures and compare behavior,
  trace completeness, latency, allocation overhead, and failure modes.
- Define rollback policy: every benchmark should be able to say whether it used
  stock Emacs or a named patched Emacs variant.

## Verification

- A stock-instrumentation coverage report lists which required events are
  captured, which are inferred, and which are missing.
- At least one insufficient-stock probe is preserved before any patch is
  accepted.
- A patched build, if produced, is materialized through the rootfs build path and
  appears in the selected rootfs manifest and ABI manifest.
- Stock and patched runs execute the same probe fixtures with identical initial
  state.
- The patched run improves evidence for a named gap and does not change the
  externally checked editor/package behavior for that fixture.
- The probe suite fails if a patched Emacs binary is used without the matching
  patch provenance and rootfs manifest entries.

## Atlas Evidence

The Atlas already shows that final state alone is not enough: command context,
hooks, async state, process identity, kernel identity, and display behavior can
matter. This issue decides whether those facts can be captured faithfully above
stock Emacs, or whether a small source patch is required for trustworthy gym
operation.

## Current Baseline

The prerequisite stock source-build baseline is now available in the selected
Soma Insula rootfs:

- Materialization kind: `source-built-emacs-v8` / `docker-cuda-source-build`
- Selected rootfs store id:
  `soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34`
- Rootfs tree hash SHA256:
  `931b5d4338e0c0eb876f5b14f7babb71bc89b20e5b6a625d26799d16c0cb41dd`
- Emacs: stock GNU Emacs `30.2` built from the verified GNU release tarball
  inside the CUDA-image-derived rootfs
- Emacs source tree: `/opt/soma/build/emacs-30.2/`
- Emacs executable: `/opt/soma/emacs/bin/emacs`
- Emacsclient executable: `/opt/soma/emacs/bin/emacsclient`
- Emacs executable SHA256:
  `824c1318464a98778d9f1677508e05af3dc5e94c34c63653145e079e5d26f6c2`
- Source-build provenance note: `docs/insula/emacs-source-build.md`

That closes the "build a stock Emacs baseline" prerequisite for this issue. The
patch decision remains open until executable probes show a named observability
gap that Lisp/advice/hooks/protocol logs/PTY capture and independent checkers
cannot cover faithfully.

# research-workers-verified-and-corrected

Date: 20260929
Supersedes:

## Decision

The three research reports under `evidence/` are accepted as leads with the corrections below. They are inputs
to design, not facts: only what is marked "lead re-ran" was checked by the lead.

## Why (runs, verdicts, checks)

Workers: `traecli` GPT-5.5 xhigh, one worktree each, read-only investigators, Haiku supervisors relaying the
deterministic verdict. Run dirs are under `~/.cache/corenius/workers/research-{blade,insula,tasks}-20260929T2220*`.
- Original verdicts were VIOLATED for all three, solely from the `rm -rf` text rule. After the rule change
  (`20260929-rm-judged-by-target-not-text.md`): blade CLEAN, insula DEGRADED (stderr: one upstream model stream
  error, one sandbox-blocked command), tasks DEGRADED (same). Old verdicts kept as `verdict.prev.json`.
- Blade isolation audit: every blade invocation went through `research-blade/run_blade_build.sh`, which uses
  `bwrap --unshare-net --clearenv`. No unsandboxed blade run was found in the event log.
- The insula worker stat'ed traecli's `auth.json` (metadata only, no contents) and ran keyword `rg` over traecli's
  own memory file. It made no model call (only `--help`). No credential text is in the landed files (scanned).
- Landed files exclude build outputs, scratch runs, binaries, and a 375 KB raw grep of erdos source; the reports keep
  file:line citations instead.

Lead re-ran (results match the reports unless stated):
- Blade in a fresh HOME offline: l0-flat builds, rc 0, 1.9 s, using the worker's pip-marker trick (touch
  `~/.blade/blade_status/pip_checking.<size of /opt/tiger/typhoon-blade/executor>` and `pip3_checking.<size>`).
  Without the markers blade retries an internal PyPI mirror and times out (insula worker's exit 124).
- Settler: `library_settler(prefer_dynamic=["cpp3rdlib/dual"])` fails ("soname can't find in dual"); the
  `prefer_static` fix builds.
- Six tasks (t01, t02, p01, p02, r01, c01), start and fixed, under `bwrap --unshare-net`: all 12 as reported
  (start fails; fixed passes; 0.02 to 0.44 s each).
- The pristine-tests verifier: deleted tests, a planted always-pass tests dir, and a wrong value all fail; the
  correct value passes.

## Corrections to the reports

1. **GPU.** The insula report says the GPU is unusable. That holds only inside the worker's own sandbox. On the
   host, `/dev/nvidia*` exist and `nvidia-smi -L` lists the B200s. The lead compiled a kernel with
   `/usr/local/cuda/bin/nvcc -arch=sm_100` and ran it on the host and inside
   `bwrap --ro-bind / / --dev-bind /dev /dev` (result correct, 8 devices). With `--dev /dev` only it fails.
   Capsules need `/dev/nvidia*` bound for GPU tasks.
2. **Blade "needs preseeded python deps".** True only until the pip-marker files exist; the markers suffice.
3. **Blade CUDA is not solved.** `cu_library` reads the `NVCC` env var (`env.get('NVCC','nvcc')`,
   `target_cu_library.py:98`); with `NVCC=/usr/local/cuda/bin/nvcc` blade issues that path, but ninja reports
   `/usr/local/cuda/bin/nvcc: not found`. Adding cuda to `PATH` does not help either. Blade runs steps in its own
   "sailfish" environment, which probably does not see `/usr/local/cuda`. Open; a follow-up worker should investigate.
4. Report claims not re-run by the lead (still `[R]`/`[X]` from the worker only): rule option lists, most blade
   probe logs, the 20-target cost numbers (1.1 s cold), Harbor schema readings, the sizing calculation (about 93
   observations per arm for 0.5 vs 0.7, an assumption, not a measurement).

## Consequences

- Capsule design needs: a `/dev/nvidia*` bind option; blade home with the pip markers; a fix for nvcc visibility.
- A follow-up research worker on "blade sailfish environment and cu_library" is warranted before M3.
- Landing lead-authored exceptions to supervisor verdicts is recorded here, not implied.

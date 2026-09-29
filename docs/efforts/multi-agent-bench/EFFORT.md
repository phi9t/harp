# Effort: multi-agent-bench

Status: active
Base: main
Lead: claude (session 2026-09-29), owner phi9t

## Intent

Measure multi-agent work on real tasks. Tasks are real repos with real verifiers (terminal work, and
python/rust/c++ tasks with uv, cargo, cmake+ninja and blade+CUDA builds). The same task runs solo and as a
lead/worker fleet, and the harness records pass rate, wall time, conflicts and contamination. Blade tasks come
in a complexity ladder (L0-L5) modelled on how lgtorch and erdos use blade. Every step that runs task code runs
inside an insula capsule (layered bwrap bundle built from a request).

## Scope and non-goals

In: `bench/` (tasks, harness, validate), `tools/insula/capsule.py` (layered capsule builder), `worker.py --insula`.
Out for this milestone: any model spend (offline validation only), bazel, calibrating the simulator, unifying the
capsule builder with `src/insula.rs`, editing `src/cli.rs` or the other agent's AIME files.

Plan: `~/.claude/plans/tidy-questing-alpaca.md` (approved 2026-09-29, amended for erdos and capsules).

## Milestones and exit criteria

- [ ] M1 capsule builder: `python3 tools/insula/capsule.py build` twice gives one digest; probes pass.
- [ ] M2 harness and T/P/R/C tracks: `python3 bench/harness/validate.py --all` passes items 1-7 of the plan.
- [ ] M3 blade ladder L0-L3 with real blade, offline, inside a capsule.
- [ ] M4 L4-L5 and multi tasks; fleet:2 with a fake agent lands in every allowed order.
- [ ] M5 (only on the owner's request) a live traecli pilot on a subset.

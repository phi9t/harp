# Effort: agentic-engineering

Status: active
Base: main
Lead: interactive session (Claude)

## Intent

Make Corenius the owner of agentic-engineering practice for the workspace: one
policy, a lead/supervisor/worker mechanism that works with `traecli` workers,
and evidence that work is done. Sibling repos will copy what works here.

## Scope and non-goals

In scope: policy and its sync tool, the worker/supervisor mechanism, worktree,
landing, tracker, message and evidence tooling, and the references that record
how sibling repos solve these problems. Non-goals for now: adopting the
mechanism in sibling repos, and the provider sandbox unless Milestone B needs it.

## Milestones and exit criteria

- [x] A: concrete feature set, `docs/superpowers/specs/2026-09-29-agentic-engineering-features.md`
- [ ] B: implement and verify Tier 1 (F1-F10) behind `tools/agents/check.sh`, in
  the order the spec gives. Each feature is done when its checks pass, `AGENTS.md`
  documents it, and the lead has used it on a real task.
- [ ] Later phases: `docs/superpowers/plans/2026-09-29-agentic-engineering-roadmap.md`

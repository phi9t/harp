# 07 jj-workspace-qualification: Optional jj workspace backend

Type: task
Status: open
Owner:
Blocked by: 01, 04
Claimed:
Spec: ../SPEC.md

## Requirements and evidence

Qualify pinned jj against the same private workspace/candidate/retention contract and Git landing interoperability; use existing evaluation evidence where applicable. Git-backed baseline remains runnable and supported. This ticket is optional for initial baseline acceptance, mandatory before jj is advertised as supported.

Claim an isolated worktree before implementation. Retain public behavioral red-green evidence, exact source identity and independent Standards/Spec review. No shared branch rewriting.

## Acceptance

- [ ] Pinned jj satisfies the common private workspace/candidate/retention seam with actual Git protected-landing interoperability.
- [ ] Git remains supported; public RED/GREEN and independent Standards/Spec pass before advertising jj support.

## Comments

- 2026-10-04T03:53:26Z codex-design: Adopted owner commits9d90222/d18e413/f775734; normalized tracker fields without changing required scope. Controller-first sequencing is binding; Matt route fallback uses the repository crosswalk.

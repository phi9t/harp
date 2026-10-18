# 05 landing-v2: Landing v2: safe push order and recovery

Type: feature
Status: claimed
Owner: collab-ops-05
Blocked by: 04
Claimed: 2026-10-12T01:49:53Z expires 2026-10-14T01:49:53Z

Triage: ready-for-agent

## What to build
land verifies the local fast-forward is possible, records push intent, pushes, then fast-forwards locally. recover finishes or rolls back an interrupted landing. land takes an expected tip, rechecks overlap just before acting, reports real push-failure causes, supports --no-push, and reports post-landing record failures as landed with records incomplete.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 11-18, 22, 23). Use the project glossary terms.

## Declared paths
- `tools/collab/`

## Acceptance
- [ ] A crash between push and local fast-forward is recovered by recover and never leaves the remote ahead without a record.
- [ ] A moved Candidate is refused against an expected tip.
- [ ] Authentication, network and non-fast-forward push failures are reported distinctly.
- [ ] An ignored or untracked collision that appears during a long check is caught at landing.
- [ ] --no-push lands locally only.
- [ ] A failed post-landing record write is reported as landed, records incomplete.

## Comments
- 2026-10-12T01:49:53Z collab-ops-05: claimed until 2026-10-14T01:49:53Z

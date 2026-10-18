# 01 dashboard: Team dashboard

Type: task
Status: resolved
Owner:
Blocked by:
Claimed:
Spec: ../EFFORT.md

## Requirements

Own controller/operator/mac_dashboard.py and controller/operator/mac_dashboard_test.py only.
Implement the dashboard sections and config/manifest read interface in the spec.
Keep the current simple happy-path view usable while adding section navigation.
Use standard-library Python and curses, no product dependencies or model calls.

## Acceptance


- [x] Structured source data, freshness and missing fields display honestly.
- [x] Tickets, current/historical agents, progress/questions, evidence/gates, VCS and infrastructure/context are reachable with keyboard navigation.
- [x] Small terminals scroll without crashing. No native worker input is sent.
- [x] Focused behavioral tests and an actual PTY check pass.
## Retained discussion

Existing operator layout and source separation are owner-approved. The lead
owns tool integration/deployment; the controller worker remains untouched.

## Comments

- 2026-10-05T05:54:54Z main-lead: Normalize tracker metadata for landing; retained prior disposition: reviewed and deployed (source e6ae1ad; STATE.md records evidence). Product baseline acceptance and runtime readiness are unchanged.

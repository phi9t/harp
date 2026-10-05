# 01 — Team dashboard

Status: claimed
Blocked by: none
Spec: ../EFFORT.md

## Requirements

Own tools/agents/mac_dashboard.py and tools/agents/test_mac_dashboard.py only.
Implement the dashboard sections and config/manifest read interface in the spec.
Keep the current simple happy-path view usable while adding section navigation.
Use standard-library Python and curses, no product dependencies or model calls.

## Acceptance

- Structured source data, freshness and missing fields display honestly.
- Tickets, current/historical agents, progress/questions, evidence/gates, VCS
  and infrastructure/context are reachable with keyboard navigation.
- Small terminals scroll without crashing. No native worker input is sent.
- Focused behavioral tests and an actual PTY check pass.

## Comments

Existing operator layout and source separation are owner-approved. The lead
owns tool integration/deployment; the controller worker remains untouched.

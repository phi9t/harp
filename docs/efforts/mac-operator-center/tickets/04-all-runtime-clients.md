# 04 — Maintain every published runtime client

Status: implemented and independently reviewed; live product recovery pending
Blocked by: none

## Owner request

Keep a Codex client open for the lead and all workers, including both live
product worker slots once their runtime bindings exist.

## Requirements

Own tools/agents/mac_tmux.py and tools/agents/test_mac_tmux.py only. Add optional
runtime_bindings_path to config version1. Read the explicitly configured public
JSON roster on each reconciliation: {observed_at, bindings:[{role,thread,cwd,
endpoint,current}]}. Each binding includes its concrete endpoint and canonical
thread UUID. Current is a boolean; no private grant or auth fields are supported.
Keep implementation and product roles distinct in the published role string.

Add current exact bindings using the existing checked native resume command.
Deduplicate across coordinator, implementation, configured extra and public
runtime roster bindings by endpoint/thread. Retain prior clients as historical
when valid new roster state supersedes them. Missing, malformed or stale (>5min)
rosters cannot admit new clients; retain their prior views with unknown state.
Record source timestamp/digest and missing/degraded status in the manifest.
Only use explicitly configured source data; do not infer runtime identities
from transcripts, an arbitrary default daemon, SQLite or bare thread IDs.
No task mutations, new runtime servers, prompts, controls or client deletion.

## Verification

Focused RED/GREEN tests: two simultaneous published workers and lead; endpoint
identity/dedup; replacement and valid empty roster; malformed/missing/stale
source retaining views without admission. Preserve original focused suite.
Separate review and root-owned deployment; no live session mutations by author.

## Scoped disposition — 2026-10-05

Source15ba811: focused suite43/43 PASS. Independent Standards, Spec and branch
reviews resolved the scoped findings. Dedicated operator views were installed.
Codex0.160.0 exact native implementation/lead clients are verified. Product
controller/API and UI gateways became unavailable; their reattachment remains
pending core-coordinator reconciliation. No full product qualification credit.
See ../STATE.md and ../evidence/client-update-20261005.json.

## Public export follow-up

The runtime roster may also carry strictly allowlisted status/reason fields.
Unavailable or pending publication admits no clients, even when bindings are
present. Preserve upstream timestamps/digests and prior unknown membership.
Legacy shape and explicit observed/available/ok publication remain supported.
Unknown status, auth fields, extra fields and credential endpoints fail closed.
No raw reason body enters errors. Keep native observer launch grants private.

This adaptation changes only the view manager and focused tests. It grants no
runtime recovery, worker admission or model-turn authority. Existing owner-created
clients can be adopted by exact binding after authenticated transport readiness.

## Existing scoped observer references

Accept the bounded public roster projection_kind descriptor and finite numeric
source timestamp. The operator may configure external_views_path containing
explicit endpoint/thread/cwd/window/pane references to already-existing scoped
clients. Observe these windows only; never rename, tag, respawn, kill or select
them. Keep private launch grants outside viewer sources. Exact public identity
and actual pane/session membership must match before reference admission.

When external mode is configured, missing/stale/malformed references must not
fall back to launching another runtime client. Owned operator clients retain
precedence. Missing/dead external panes remain explicit without replacement.
Prior external membership becomes unknown when runtime source is unavailable
or stale. Original source times/digests and static-source limitations remain visible.

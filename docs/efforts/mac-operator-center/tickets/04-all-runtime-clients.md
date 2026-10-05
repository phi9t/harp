# 04 — Maintain every published runtime client

Status: claimed
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

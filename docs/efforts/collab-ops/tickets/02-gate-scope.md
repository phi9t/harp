# 02 gate-scope: Gate runs collab tests only when the toolset changes

Type: task
Status: claimed
Owner: collab-ops-02
Blocked by: 
Claimed: 2026-10-11T23:45:55Z expires 2026-10-13T23:45:55Z

Triage: ready-for-agent

## What to build
The collab gate check runs its suite when the change set touches the toolset, and also runs it when the change set cannot be determined.

Spec: docs/efforts/collab-ops/SPEC.md (user stories 53). Use the project glossary terms.

## Declared paths
- `tools/agents/checks.d/65-collab.sh`
- `tools/agents/test_check.py`

## Acceptance
- [x] A change outside the toolset skips the suite and says so.
- [x] A change inside the toolset runs it.
- [x] An undeterminable change set runs it.
- [x] Shellcheck is clean for the check script.

## Comments
- 2026-10-11T23:37:11Z collab-ops-02: claimed until 2026-10-13T23:37:11Z
- 2026-10-11T23:59:00Z claude: released; first launch had no declared paths
- 2026-10-11T23:45:55Z collab-ops-02: claimed until 2026-10-13T23:45:55Z

# milestone-b-fleet

Date: 20260930
Supersedes:

## Decision

Milestone B Tier 1 features F5 to F10 are built by a fleet of six traecli workers (GPT-5.5, xhigh, inside the `agent` capsule, one worktree each),
specified in `docs/efforts/agentic-engineering/milestone-b/SPEC.md` with three contracts and six tracked ticket files. The lead owns contracts, tickets, shared
files (`check.sh`, `AGENTS.md`, the feature register), verification and landing. Scope: Tier 1 only. Landing: the lead verifies, then lands locally by
fast-forward; nothing is pushed (owner decision, 2026-09-30).

## Why this shape

- The couplings are real but contract-shaped: `wt.py` loads `land` through a fixed plug-in hook; `claims.py` writes against the evidence record API. Shared files are
  lead-owned and every feature adds one `checks.d/NN-name.sh`, so workers never edit the same file (conflicts are designed out, not manufactured).
- Workers cannot run git on their own tree or on the real repo inside the capsule, so their tests use throwaway repos; real-repo acceptance is run by the lead.
- `tools/agents/check.sh` (new, green on the existing suites before dispatch) is the gate; it runs the existing suites, the tracker lint once it exists, and every
  `checks.d` script. Cargo runs only with `CHECK_CARGO=1` (the main checkout still carries uncommitted AIME work; the feature spec's Decision 2 stays open).

## Consequences

- Each worker's verdict is judged by the deterministic supervisor; a CLEAN verdict means the boundary held, not that the work is right. Verification notes and outcomes are
  appended to this record.
- After landing, F5 to F10 are used on this run (land T3-T6 through `wt.py`/`land.py`, tracker list/lint over these tickets, evidence records of the six runs, claim sampling of the
  lead's notes, handoff messages), and the feature register is updated.

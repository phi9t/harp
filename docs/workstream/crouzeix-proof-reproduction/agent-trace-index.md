# Crouzeix Proof Agent Trace Index

## Claim Ceiling

This index captures execution guidance and trace-derived evidence from the
proof-agent amendment chain. It is not a proof of Crouzeix's conjecture and
does not override run artifacts, tracker state, source code, or committed
specifications.

## Captured Amendments

| Amendment | Source path | Lines | Capture mode | Summary | Removal status |
|-|-:|-:|-|-|-|
| 001 | `.worktrees/crouzeix-proof-reproduction/docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-001.md` | 188 | summarized | Establishes run artifacts as primary evidence, separates control/resource failures from proof failure, records flat-baseline route-family collapse pressure, and is superseded where it recommends immediate root recovery/resume. | not re-landed as raw amendment |
| 002 | `.worktrees/crouzeix-proof-reproduction/docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-002.md` | 125 | summarized | Records `expert-g0-function-theory` as recovered terminal `failed` with `provider_invoked=false`, one failed attempt, no admissions, and no mathematical nodes; later resume instructions are superseded. | not re-landed as raw amendment |
| 003 | `.worktrees/crouzeix-proof-reproduction/docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-003.md` | 188 | summarized | Identifies `expert-g0-operator-dilation` as a second nonterminal root with no per-ticket call, attempt, admission, mathematical node, frontier event, or attempt-ledger row; aggregate receipts cannot decide per-ticket orphan classification. | not re-landed as raw amendment |
| 004 | `.worktrees/crouzeix-proof-reproduction/docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-004.md` | 180 | summarized | Narrows the blocker to a local pre-provider ticket-scope validation failure and identifies `_run_expert_attempt()` / `expert_runner.py` as the repair seam. | not re-landed as raw amendment |
| 005 | `.worktrees/crouzeix-proof-reproduction/docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-005.md` | 102 | summarized | Converts the blocked `expert-frontier-001` state into a provider-free repair protocol and blocks live CPFR-032 action from unchanged evidence. | removed from old proof-reproduction worktree after capture commit |
| 006 | `.worktrees/crouzeix-proof-reproduction/docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-006.md` | 85 | summarized | Turns the no-diagnosis-churn rule into a repair-only gate for the unchanged `operator-dilation` nonterminal state. | removed from old proof-reproduction worktree after capture commit |
| 007 | `.worktrees/crouzeix-proof-reproduction/docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-007.md` | 347 | excerpted | Defines the actionable CPFR-R014 provider-free repair contract, red-test matrix, forbidden commands, allowed files, and completion evidence. | removed from old proof-reproduction worktree after capture commit |
| 008 | `.worktrees/crouzeix-proof-reproduction/docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-008.md` | 195 | summarized | Adds the stale-trace contradiction guard: fresh validators override prior “No blockers / continue roots” agent output. | removed from old proof-reproduction worktree after capture commit |
| 009 | `.worktrees/crouzeix-proof-reproduction/docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-009.md` | 148 | summarized | Adds worktree and run-ID admissibility requirements before local traces can be used as proof-run evidence. | removed from old proof-reproduction worktree after capture commit |
| 010 | `.worktrees/crouzeix-proof-reproduction/docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-010.md` | 176 | summarized | Adds bounded metadata-first trace search and rejects broad tool-result archaeology without a known relevant call. | removed from old proof-reproduction worktree after capture commit |
| 011 | `.worktrees/crouzeix-proof-reproduction/docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-011.md` | 156 | summarized | Adds candidate trace-set construction and current-rollout exclusion to avoid self-ingesting the active review transcript. | removed from old proof-reproduction worktree after capture commit |
| 012 | `.worktrees/crouzeix-proof-reproduction/docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-012.md` | 258 | excerpted | Adds the active reachability guard: tracker-linked amendment guidance must be tracked or staged before handoff; identifies the pre-`running` provider-boundary validation seam. | removed from old proof-reproduction worktree after capture commit |

## Durable Capture

The durable normalized capture is
`docs/workstream/crouzeix-proof-reproduction/proof-agent-amendment-capture-001.md`.
That file preserves the active handoff rule, unchanged proof state, CPFR-R014
repair contract, trace evidence protocols, and amendment reachability protocol.

## Raw-File Removal Rule

The raw amendment files may be removed from the old
`crouzeix-proof-reproduction` worktree only after this index and the normalized
capture are committed or otherwise included in a coherent staged landing.

# CPFR-032 Gate Report 001

## Claim Ceiling

This report is read-only evidence for CPFR-032 gating. It is not a provider
call, not a proof attempt, not a proof claim, and not authorization to continue
without owner approval.

## Scope

This report was run from the clean integration worktree
`integrate/crouzeix-baseline-reconciliation`. It deliberately did not copy
ignored `.runs` evidence from the proof source worktree.

The source proof worktree contains ignored run trees for `expert-frontier-001`
and `expert-frontier-002`, but those are not present in this clean integration
worktree. The dirty source tracker reports a fresh `expert-frontier-002`
pre-provider digest-boundary blocker. That evidence remains unvalidated here.

## Commands

| Command | Run ID | Exit status | Output artifact |
|-|-:|-:|-|
| `tickets.py validate-runtime ...` | `expert-frontier-001` | `2` | `/tmp/cpfr-032-validate-runtime-001.txt` |
| `prepare_frontier.py --scan-contexts ...` | `expert-frontier-001` | `2` | `/tmp/cpfr-032-scan-contexts-001.txt` |
| `run_frontier.py report-roots ...` | `expert-frontier-001` | `1` | `/tmp/cpfr-032-report-roots-001.txt` |
| `tickets.py validate-runtime ...` | `expert-frontier-002` | `2` | `/tmp/cpfr-032-validate-runtime-002.txt` |
| `prepare_frontier.py --scan-contexts ...` | `expert-frontier-002` | `2` | `/tmp/cpfr-032-scan-contexts-002.txt` |
| `run_frontier.py report-roots ...` | `expert-frontier-002` | `1` | `/tmp/cpfr-032-report-roots-002.txt` |

No before/after run-tree digest comparison was possible in the integration
worktree because both run trees were absent before the read-only commands ran
and remained absent.

## Result

- Runtime validation: `blocked: runtime root does not exist`.
- Context scan: `blocked: current prepare_frontier.py has no --scan-contexts option`.
- Root readiness: `blocked: run_spec.json is absent for the requested run`.
- Provider call allowed by this report: `no`.

## Decision

CPFR-032 is `blocked`.

The blocker is absence of validated local run evidence in the clean integration
worktree. The source worktree's dirty `expert-frontier-002` evidence may be
useful, but it must be validated or deliberately quarantined by a later plan
before it can authorize any provider execution.

## Evidence Excerpts

```text
tickets: runtime root does not exist: labs/crouzeix_proof_reproduction/.runs
```

```text
prepare_frontier.py: error: unrecognized arguments: --scan-contexts
```

```text
protocol.ValidationError: cannot inspect run specification: [Errno 2] No such file or directory:
labs/crouzeix_proof_reproduction/.runs/expert-frontier-002/run_spec.json
```

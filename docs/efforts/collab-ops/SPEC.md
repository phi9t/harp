# Collab operations specification

Base commit: c0a2fc0370f18ff28009dbe9b62a59916de0e582
Status: ready-for-agent
Triage: ready-for-agent

## Problem Statement

Multi-agent work in Corenius runs on three overlapping tools: the collab CLI (attempts,
receipts, recovery), `wt.py` (worktrees) and `land.py` (gated landing). They use
different branch names, different ticket homes and different safety rules. An audit
found real defects in the older two: garbage collection deletes a brand-new worktree,
landing pushes before the local fast-forward can fail, push errors are reported as
stale candidates, gate results are not recorded, mainline overrides are unverified and
are not recorded for standalone landings, and the accepted tip is not pinned. Stale
worktrees and branches pile up, and nothing sweeps them safely. The operator cannot
trust any of them as production infrastructure, and cannot run tickets through one
system end to end.

## Solution

One modular collab toolset runs the whole ticket lifecycle: it reads the tickets that
the Matt skill flow publishes, opens an Attempt in an isolated worktree, checks the
exact Integration commit, records a Check receipt, lands by fast-forward only and
resolves the ticket. A manually triggered sweep keeps worktrees and branches clean
without losing work. `wt.py` and `land.py` remain as compatible shims for one release.

## User Stories

1. As an operator, I want one tool to take a ticket from claim to landing, so that I do not coordinate three tools by hand.
2. As an operator, I want `open-attempt` to claim the ticket, create the worktree and record the Attempt together, so that the three never disagree.
3. As an operator, I want a failed `open-attempt` to leave a recoverable state, so that a crash never strands a claim.
4. As a worker, I want an isolated worktree on a branch named for me and the ticket, so that I can work without touching anyone else's checkout.
5. As a worker, I want to submit my work and have the exact Candidate sealed, so that what I submitted is what gets checked.
6. As an operator, I want the check to run on a fresh Integration commit built from the Candidate and the ticket closure, so that I verify exactly what will land.
7. As an operator, I want a Check receipt recording subject, command, outcome and log identity, so that landing relies on evidence rather than memory.
8. As an operator, I want the check to refuse a dirty workspace, a changed HEAD or a moved trunk, so that a green result cannot be stale.
9. As an operator, I want a check timeout and streamed output, so that a hung gate neither blocks silently nor holds the lock forever.
10. As an operator, I want the landing lock not to be held across the whole gate run, so that other work is not starved.
11. As an operator, I want landing to fast-forward only the exact checked commit, so that nothing unreviewed lands.
12. As an operator, I want landing to refuse if the Check receipt no longer matches, so that a tampered log or moved subject cannot land.
13. As an operator, I want the local fast-forward to be verified possible before anything is pushed, so that a push never leaves the remote ahead of local.
14. As an operator, I want push intent recorded before the push, so that a crash between push and fast-forward is recoverable.
15. As an operator, I want `recover` to finish or safely roll back an interrupted landing, so that I never reconcile by hand from Git internals.
16. As an operator, I want push failures reported by their real cause (authentication, network, non-fast-forward), so that I do not mistake them for a stale Candidate.
17. As an operator, I want `--no-push` to land locally only, so that offline work is possible.
18. As an operator, I want overlap with ignored or untracked files checked at landing time, not only before the gates, so that a long gate cannot hide a collision.
19. As an operator, I want a mainline override to name an allowed owner from configuration, so that anyone cannot self-authorize.
20. As an operator, I want every override recorded as an event, including standalone landings, so that there is always an audit trail.
21. As an operator, I want the override also exported to the effort's evidence when an effort is named, so that existing readers keep working.
22. As an operator, I want the landing to take an expected tip, so that the accepted commit is pinned and a moved branch is refused.
23. As an operator, I want a failure to write post-landing records to be reported as a recorded partial success, not a failed landing, so that I know the code did land.
24. As an operator, I want a moved trunk to offer `refresh`, so that an Attempt is carried forward without losing the worker's commits.
25. As an operator, I want `refresh` to apply only clean cherry-picks and stop at the first conflict, so that nothing is resolved silently.
26. As an operator, I want direct landings by other tools to count as normal trunk movement, so that they do not break open Attempts unexpectedly.
27. As an operator, I want workers unable to change ticket files, the gate scripts, the gate entry point, the constitution or policy documents, so that a worker cannot weaken the check that judges its own work.
28. As an operator, I want the protected-path list set at initialization with Corenius defaults, so that other repositories can choose their own.
29. As a lead agent, I want to pass the worker's process id or run directory when opening an Attempt, so that collab can refuse recovery while that worker is alive.
30. As an operator, I want `recover` and `abandon` to refuse while a recorded worker process is alive, so that `--confirm-stopped` is no longer only an honor system.
31. As an operator, I want an `inspect` command that lists exactly what blocks a state change, so that I do not diagnose hidden index flags and ignored files by hand.
32. As an operator, I want one ticket source adapter for Corenius efforts, so that tickets published by the Matt skill flow are the tickets collab runs.
33. As an operator, I want `tracker.py` to remain the only writer of ticket files, so that claims, blockers and resolution stay consistent.
34. As an operator, I want the claim to appear in the tracker when an Attempt opens, and the ticket to be resolved inside the locked landing, so that ticket state and Git state agree.
35. As a maintainer of another repository, I want a standalone ticket source for the simple committed-ticket layout, so that collab still works outside Corenius.
36. As an operator, I want a sweep that lists everything it would clean and removes nothing by default, so that I can review before acting.
37. As an operator, I want the sweep to remove worktrees and branches that are merged, clean and belong to a terminal Attempt, so that routine hygiene is automatic to request.
38. As an operator, I want a fresh or in-progress worktree never to be swept, so that new work cannot be deleted.
39. As an operator, I want clean worktrees with unmerged commits archived to a recoverable ref before removal, so that no commit is lost.
40. As an operator, I want dirty worktrees archived with their patch and untracked files only when I ask, so that uncommitted work is preserved first.
41. As an operator, I want stale collab refs, old check logs and orphaned check worktrees pruned past a retention age, so that state does not grow without bound.
42. As an operator, I want the sweep to skip locked worktrees with a clear reason, not fail with a usage error, so that its report is trustworthy.
43. As an operator, I want branches and worktrees not created by collab touched only when fully merged, so that another session's work is safe.
44. As an operator, I want unmerged foreign branches listed with their owner and skipped unless I pass an explicit flag, and archived first when I do, so that I stay in control.
45. As an operator, I want to trigger the sweep manually from a repository skill, so that hygiene is one request away.
46. As an operator, I want the mainline reference to be configurable and to consider the remote when reporting how far behind a branch is, so that status is not misleading.
47. As an operator, I want worktree creation locked and its metadata validated, so that concurrent creation or corrupt metadata cannot produce silent half-state.
48. As a maintainer, I want the old `wt.py` and `land.py` commands to keep their flags and exit codes for one release, so that launchers and gates keep working.
49. As a maintainer, I want the shims to print a deprecation notice in their help, so that callers move to collab.
50. As a maintainer, I want the toolset split into modules with narrow interfaces, so that each can change without touching the others.
51. As a maintainer, I want effort record-branch landing kept as an optional module, so that existing efforts keep working until they close or move.
52. As a maintainer, I want new efforts to land on `main` with in-tree records, so that the default flow is trunk-based.
53. As a reviewer, I want the gate to run the collab tests only when the toolset changed, so that routine gate runs stay fast.
54. As a reviewer, I want the toolset readable at 120 columns with no packed statements, so that it meets the repository review standard.

## Implementation Decisions

- Modules: ticket source (Corenius effort adapter and standalone adapter behind one interface), workspace (create, inspect, safe removal), state store (the only module touching SQLite), check runner (isolated check producing a Check receipt), landing (the push and fast-forward protocol), policy (protected paths, override owners), sweep (cleanup tiers) and a thin CLI. No module reaches into another's storage.
- The tracker remains the sole writer of ticket files. Opening an Attempt claims through it; landing resolves through it under the landing lock. Collab never edits ticket files.
- The Integration commit is built from the Candidate and the ticket closure; the closure no longer rewrites ticket status inside the commit for Corenius efforts.
- Branches follow the repository agent/topic convention; the Attempt identity is stored in the state store, not encoded in the branch name. The state store is authoritative; worktree metadata only carries the Attempt identity for display.
- Command renames: the collab claim becomes `open-attempt`, avoiding the tracker's different meaning of claim. Existing collab command names otherwise remain.
- Landing order: verify the local fast-forward is possible, record push intent, push, then fast-forward locally, all under the landing protocol with durable states so `recover` can finish or roll back each step. Push errors carry their real cause.
- The landing lock is not held across the gate run; the check runs under its own lane protection and landing revalidates subject, trunk and overlap immediately before acting.
- Check has a configurable timeout and streams to its log; the receipt records timeout as a distinct outcome.
- Mainline override: a configured list of allowed owners; every override is an event in the state store, always; effort evidence export is additional.
- Landing accepts an expected tip and refuses a moved Candidate.
- Post-landing record failures end in a distinct "landed, records incomplete" outcome with a recorded repair step.
- `refresh` abandons an Attempt and opens a new one on current trunk, cherry-picking only when every pick applies cleanly.
- Worker process liveness: `open-attempt` accepts a process id or run directory from the launcher; recover and abandon refuse while it is alive.
- Protected paths: set at initialization; Corenius defaults add the gate scripts, the gate entry point and the constitution to the existing coordination and policy paths.
- Sweep tiers: (1) merged, clean, terminal Attempt: remove worktree and branch; (2) clean with unmerged commits: archive to a recoverable ref, then remove; (3) dirty: archive patch and untracked files, only with an explicit flag; (4) prune stale collab refs, old logs, orphaned check worktrees and dangling worktree registrations past retention. Dry run is the default and `--apply` acts. Non-terminal and fresh worktrees are never swept. Foreign branches only when fully merged, otherwise listed with owner; an explicit flag archives them first.
- Mainline resolution is configurable; status reports divergence against the remote as well as local mainline when a remote exists.
- Shims keep current flags and exit codes for one release; they translate to collab and add a deprecation notice. Existing tests remain unchanged except expectations that encode the audited defects: a fresh worktree must survive garbage collection, and a mainline override requires configured authority. Reports identify each corrected expectation and its regression evidence.
- Effort record-branch support moves into an optional landing module; new efforts do not use it.
- The gate runs the collab tests when files under the toolset changed, and runs them when the change set cannot be determined.
- A repository skill triggers the sweep manually.
- Glossary terms: Attempt, Candidate, Integration commit and Check receipt, as defined in the project glossary.

### Completion clarifications

The user invoked `/implement` after reviewing the staged completion recommendation.
The public CLI and existing shim seams remain the approved behavioral interfaces.
Implementation and commits occur on `codex/collab-ops-complete` and isolated ticket
branches; existing source worktrees and active workers retain their owners.

Corenius effort ticket closure is a tracker-owned post-landing record operation.
The checked Integration commit contains the contribution without rewriting the
Corenius ticket. Resolution and its acceptance evidence become a separate,
attributed bookkeeping commit under the same landing lane. A failure after code
landing is recorded as `landed_records_incomplete`; recovery repairs only the
bound record operation and cannot repeat the code landing or overwrite operator
changes. Standalone tickets retain closure inside the checked Integration commit.

Dirty cleanup archives preserve tracked edits, staged state, untracked and ignored
files, executable modes and symlink identity without following external symlinks.
Git objects needed to recover removed refs remain pinned. Unknown/live writers,
substituted repositories and externally locked workspaces remain retained.
Retention age alone never establishes safe removal. Restore behavior is tested
before an archive can authorize deletion in disposable qualification repositories.

Legacy override names are validated against trusted configuration, including
repeatable `collab.overrideOwners` Git configuration for repositories that have
not initialized Collab. A caller-supplied name is not an authority grant. Legacy
support does not silently initialize a Collab ticket ledger.

## Testing Decisions

- A good test drives the public collab CLI in a disposable real Git repository and asserts observable results only: refs, state rows, receipts, exit codes, files on disk. No test reaches into a module's internals.
- Seam one: the collab CLI. Every behavior above is proven here, including each audit defect as a regression: a fresh worktree survives the sweep (W1), locked worktrees are reported not failed (W2), a crash between push and local fast-forward is recovered (L1), push causes are distinguished (L2), a collision appearing during a long gate is caught (L3), receipts record argv, outcome and timeout (L4), overrides are validated and always recorded (L5), partial post-landing failure is reported distinctly (L6), a moved branch is refused against an expected tip (L7).
- Seam two: the shim contract. Existing worktree and landing suites pass against the shims, with only the explicitly documented unsafe expectations corrected as described above.
- Process death uses the existing failpoint mechanism; liveness uses a real child process.
- The sweep skill is a thin wrapper over the CLI and has no tests of its own.
- Prior art: the existing public-CLI suite for collab, and the worktree and landing suites.

## Out of Scope

- Process supervision beyond liveness refusal, scheduling, automatic leases or retries, distributed or network-filesystem execution, authenticated review, and agent launching.
- Replacing the retained controller or any controller state.
- Migrating existing efforts' tickets or record branches.
- Changing synchronized policy documents (a separate policy change with version and manifest bump).
- The engineering-profile design document.
- Touching other sessions' branches and worktrees.

## Further Notes

- Audit source, W1 reproduced: the sweep in the old worktree tool deletes a clean worktree with no commits because a branch with no commits ahead counts as merged.
- The native controller tests currently fail on `main` for an unrelated environmental reason (a missing runtime path); landing from this effort must use the strict gate once that is fixed. Until then, landings state the exclusion explicitly.
- Tickets follow the tracker format and are produced by the next step of the skill flow.

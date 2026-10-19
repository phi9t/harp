# Independent critique of SPEC.md draft 1 (2026-10-05)

Reviewer: a clean-context subagent of the same model family as the lead, read-only, given the spec, the
charter, the first decision file and the repository, and not the design conversation.
Verdict: not ready for tickets. Fifteen findings, eight blocking. The lead spot-checked the citations
behind findings 2, 3, 4, 6 and 9 against the files; they held.

| # | Severity | Finding | Disposition in draft 2 |
|---|---|---|---|
| 1 | blocking | Building four groups in the toolbox does not make the controller pass them; no one substrate could run the trial | Superseded by the owner's three instantiations: each is complete in itself |
| 2 | blocking | Network-on-by-default contradicts the controller's root Insula design; scripted participants run offline | Since decided (network default applies to I1); harness work listed |
| 3 | blocking | "Worker commits" was filed as settled but reverses the owner's 2026-09-29 decision | Put to the owner and since decided |
| 4 | blocking | The harness cannot serve as the seam today; draft 1 presented new work as existing | Testing Decisions now lists the missing harness work and rejects the fleet runner as the seam |
| 5 | blocking | Several stories cannot be judged from a disposable repository | Stories tagged D or L with an observation surface; a run observer added |
| 6 | blocking | The live trial contradicted the limits it cited | Separate live trials per instantiation; I2's limits stated as limits |
| 7 | blocking | The toolbox lander rebases in the worker's worktree; integration-to-mainline undefined; only a human merges | Recorded in the I1 column; integration landing since decided |
| 8 | blocking | Tracker tickets as source conflict with Kata in the controller; triage is not read by the tracker | Ticket source is per instantiation; triage gap recorded for I1 |
| 9 | should-fix | Coverage table partly wrong (hand-offs, process and disk accounting, integration branch) | Table rewritten per instantiation |
| 10 | should-fix | Conflict with the instruction to park unrelated work | Exemption since decided |
| 11 | should-fix | Contribute-versus-build boundary not reproducible | I1 work is built here if exempted; for I2 the contribution is scenarios and results only |
| 12 | should-fix | Owner confirmation unverifiable beyond his phrases | Questions recorded verbatim in the decision file; elaborations labelled |
| 13 | should-fix | Untestable, compound or conflicting stories | Stories rewritten: split, given observables, duplicates merged |
| 14 | should-fix | Case-study failures with no story; some stories without basis | Added: overlap, independent review, re-review after rework, brief pinning, malformed blockers, long commands. Removed: lead continuity, context-low, bounded disk |
| 15 | minor | Triage label, glossary narrowing, empty state sections | Triage set to needs-info; Candidate wording widened; state filled |

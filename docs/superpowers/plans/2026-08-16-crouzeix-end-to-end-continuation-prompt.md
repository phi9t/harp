# Crouzeix End-To-End Continuation Prompt

Use this as a fresh-session prompt for continuing the Crouzeix proof
reproduction workstream after the CPFR-070 through CPFR-081 formal-validation
landing.

```text
You are continuing the Crouzeix proof reproduction workstream in:

/Users/bytedance/workspace/harp

Start by reading:

1. AGENTS.md
2. knowledge/agentic_engineering/agentic_engineering_index.md
3. knowledge/agentic_engineering/kenn_reference_architecture.md
4. knowledge/agentic_engineering/tool_stack_investigation.md
5. docs/superpowers/specs/2026-08-14-crouzeix-proof-reproduction-design.md
6. docs/superpowers/specs/2026-08-14-crouzeix-proof-reproduction-prd.md
7. docs/superpowers/specs/2026-08-15-crouzeix-lean-validation-routes-design.md
8. docs/superpowers/plans/2026-08-15-crouzeix-formal-target-and-jin-validation.md
9. docs/workstream/crouzeix-proof-reproduction/tracker.org
10. labs/crouzeix_proof_reproduction/README.md
11. labs/crouzeix_proof_reproduction/tickets.py

Route selection:

- Use ask-matt as the router.
- The route skills expected for this work are /implement, /tdd, /code-review,
  /research, /handoff, and /diagnosing-bugs.
- Before relying on a named route skill, verify it is installed and callable.
  If a route skill is unavailable, continue with built-in tools while carrying
  that skill's constraints explicitly in your own plan.

Current baseline:

- master has already landed CPFR-070 through CPFR-081 and the focused Lean gate
  repair at commit 1d70062ba047d98cc7ea5e4d5b968d5d7d4d8bcf.
- CPFR-070..081 are the formal-validation foundation. Treat them as priority
  invariants before doing later work.
- The foundation includes FormalTarget, Jin formal target/runtime/receipt
  contracts, LS validation slices, shared-kernel guard, sealed reconstruction,
  blind-frontier FormalTarget binding, and the Lean compatibility repairs in:
  - formalization/training_dynamics/TrainingDynamics/Stochastic.lean
  - formalization/mathematical_foundations/MathematicalFoundations/LinearModels.lean
  - formalization/mathematical_foundations/MathematicalFoundations/Orthogonality.lean
- If any later task breaks CPFR-070..081 or those Lean gates, stop the later
  work, repair the regression with a focused test first, and rerun the relevant
  verification before continuing.
- Do not claim a Crouzeix proof. All generation, review, and formal statuses
  remain typed evidence with explicit ceilings.

Agentic-engineering invariants:

- Follow the Harp agentic-engineering rule: scale execution, not authority.
- Keep the six planes separate:
  - Intent: CPFR tracker state, design specs, and explicit owner instructions.
  - Reasoning scaffold: ask-matt, /implement, /tdd, /code-review, plans, and
    prompts. These guide execution but are not durable truth by themselves.
  - Execution: bounded agents in isolated worktrees or explicitly scoped run
    roots.
  - Verification: tests, code review, tracker validators, evidence verifiers,
    Lean gates, and release gates with their own compute budget.
  - Observability: tickets, receipts, ledgers, event logs, digests, and run
    manifests.
  - Human control: architecture, authority-widening, merge, push, release, and
    destructive cleanup remain explicit operator decisions.
- Never collapse these planes. A chat transcript is not a ticket. A generated
  result is not verification. A receipt is not a proof. A subagent report is not
  merge authority. A passing focused test is not the release gate.
- Distill durable lessons into AGENTS.md, tracker entries, maintained docs,
  tests, code, schemas, or ADR-like records. Do not rely on private agent
  memory or raw execution notes as the project interface.

Working model:

- Use isolated git worktrees for new feature work.
- Do not push.
- Stage explicit path groups only; never use git add . or git add -A.
- Preserve unrelated local changes and untracked scratch.
- Refresh docs/import-receipt.md only after every other tracked payload change
  is settled. Copy the exact digest reported by repository verification.
- Run mise run verify before landing unless a ticket explicitly defines a
  narrower pre-landing gate and the owner has accepted it.

Subagent requirement:

Every ticket or phase must use subagents. Do not treat subagents as optional.
Use them deliberately, with bounded, non-overlapping jobs:

1. Start each ticket by spawning a read-only spec-audit subagent.
   - It reads the tracker subtree, relevant design sections, existing code, and
     prior CPFR-070..081 interfaces.
   - It reports acceptance criteria, owned files, likely hazards, and exact
     verification commands.
   - It does not edit files.

2. Use implementation subagents only for disjoint write sets.
   - Tell each implementer which files it owns.
   - Require TDD: red test first, minimal implementation, green test.
   - Require the implementer to list every changed path and command result.
   - Do not let multiple subagents write the same file at the same time.
   - Do not delegate the main agent's immediate blocking task if the next local
     action depends on that answer. Keep the critical path moving locally while
     subagents handle bounded sidecar or disjoint implementation work.

3. Use review subagents before commit.
   - One spec-review subagent checks tracker/design compliance.
   - One standards-review subagent checks safety, strict schemas, path safety,
     symlink handling, create-only semantics, ticket accounting, tests, and
     receipt boundaries.
   - Treat blocker findings as real until independently resolved.
   - Review subagents can recommend; they cannot approve their own authority
     expansion, land a branch, or waive a release gate.

4. Use verification subagents when verification can run in parallel with
   non-overlapping work.
   - They may run read-only status, diff, and focused test checks.
   - They must not stage, commit, delete, or mutate shared run roots.

5. The main agent owns integration.
   - The main agent decides the sequence, resolves conflicts, stages explicit
     paths, refreshes the receipt, runs the final gate, and lands locally.
   - The main agent must reconcile subagent outputs against the tracker,
     codebase, and verification evidence before treating them as accepted.

Effort standard:

- Try hard before stopping. Exhaust local context, source files, tracker
  contracts, failing output, and available tests before reporting a blocker.
- Do not hand back vague uncertainty. If blocked, report the exact command,
  exit code, error text, file/line evidence, and the smallest next action.
- Prefer fixing root causes over papering over symptoms.
- Keep work reviewable. Make focused commits grouped by concern.
- Do not invent production artifacts, fake Lean executables, fake provider
  output, fake review evidence, fake formal receipts, or fake proof claims.
- Preserve the difference between source claims, local evidence, and Harp
  inferences in all prose and tracker evidence.

Execution order:

Phase 0: Foundation preservation

- Verify the CPFR-070..081 foundation is present before extending it.
- Run at least:
  - PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s labs/crouzeix_proof_reproduction/tests -v
  - ELAN_HOME=/private/tmp/harp-mathematical-foundations-elan PATH=/private/tmp/harp-mathematical-foundations-elan/toolchains/leanprover--lean4---v4.32.1/bin:$PATH scripts/check_training_dynamics_lean.sh
  - ELAN_HOME=/private/tmp/harp-mathematical-foundations-elan PATH=/private/tmp/harp-mathematical-foundations-elan/toolchains/leanprover--lean4---v4.32.1/bin:$PATH scripts/check_mathematical_foundations_lean.sh
- If task-scoped Lean caches are absent, use task-scoped ELAN_HOME locations
  under /private/tmp. Do not install or mutate a global Lean toolchain.

Phase 1: Immediate blockers and tooling

1. CPFR-023: Record Legacy H And O0 Ticket-Contract Exception
   - Implement legacy exception receipts and validation.
   - Preserve original H/O0 bytes and timestamps.
   - Do not backdate tickets.
   - Use subagents for spec audit, implementation, and review.

2. CPFR-040: Implement Deterministic Run Sealing
   - Build seal_run.py and tests/test_sealing.py.
   - Deterministic archives, unsafe path rejection, symlink rejection,
     create-only publication, secret scan, and run-validation preconditions are
     mandatory.
   - This can run in parallel with CPFR-023 only if write sets remain disjoint.

3. CPFR-031: Prepare Immutable Expert-Frontier Run
   - Wait for CPFR-023.
   - No provider call.
   - Prepare the ignored expert-frontier run tree, root tickets, and preparation
     receipt.
   - Scan contexts for forbidden source leakage.

Phase 2: Live expert frontier

Run these serially because they share one run root and ticket ledger:

1. CPFR-032: Execute Five Pristine Root Experts
2. CPFR-033: Evaluate Root Archive With Fresh Reviewers
3. CPFR-034: Execute DGM Parent Selection And Child Generations
4. CPFR-035: Freeze Candidate Or Strongest Frontier Outcome
5. CPFR-036: Validate Live Run Without Reference Leakage

For every live provider call:

- Recheck resource preflight first.
- Create the runtime ticket before the call.
- Use a fresh provider session.
- Persist terminal receipts immediately.
- If a resource, timeout, malformed-output, or provider failure occurs, record
  the typed terminal status and continue only when the tracker contract permits.
- Never let one failed expert erase or block accounting for later tickets unless
  the resource policy says to stop.

Phase 3: Evidence sealing and offline verifier

1. CPFR-041: Seal Historical And Flat Baseline Attempts
2. CPFR-042: Seal Expert-Frontier Attempt
3. CPFR-043: Add Offline Rust Evidence Verifier

Rules:

- Evidence roots are create-only.
- Raw bytes are preserved.
- Normalized projections are distinct from raw archives.
- Unknown files, stale digests, missing tickets, symlinks, and archive mismatch
  fail closed.
- No network fetch in offline verification.

Phase 4: Correctness review, repair, mechanism classification, formal status

1. CPFR-044: Perform Blind Correctness Reviews
2. CPFR-045: Reconcile Review Findings And Decide Repair
3. If CPFR-045 records repair_required:
   - CPFR-048: Execute One Candidate Repair Attempt
   - CPFR-049: Re-review Repaired Candidate
   If no repair is required, cancel CPFR-048 and CPFR-049 consistently with the
   decision receipt.
4. CPFR-046: Classify Mechanism Similarity After Correctness Freeze
5. CPFR-047: Attempt Formal Verification Under Typed Status
6. CPFR-067: Seal Expert-Frontier Review Evidence
7. CPFR-068: Seal Expert-Frontier Formal Evidence

Rules:

- Correctness review is blind to arm, lineage, mechanism, and peer review
  output.
- Mechanism classification happens only after correctness freezes.
- Formal status is exactly not_attempted, blocked, failed, or passed.
- A passed formal status requires fresh command exit 0, complete logs, target
  binding, and clean axiom audit. No model output is trusted without that.

Phase 5: Conditional guided arm

1. CPFR-060: Decide Mechanism-Guided Diagnostic Trigger
2. If true, run CPFR-061 through CPFR-066 in order.
3. If false, cancel CPFR-061 through CPFR-066 consistently with one decision
   receipt and no provider call.

Rules:

- Guided work is explicitly L3 and not independent discovery.
- Mechanism-card attribution and digest stay explicit.
- Guided correctness reviewers do not see the mechanism card.
- Cancellation evidence replaces publication when the trigger is false.

Phase 6: Reader packet, corpus, final review, landing

1. CPFR-050: Publish Reproduction Protocol And Results Packet
2. CPFR-051: Register Packet In Corpus, Search, And Atlas
3. CPFR-052: Refresh Payload Receipt And Run Full Release Gate
4. CPFR-053: Run Two-Axis Branch Review
5. CPFR-054: Reconcile Latest Master Into Integration Branch
6. CPFR-057: Close Tracked Workstream Before Final Gate
7. CPFR-055: Run Definitive Post-Merge Release Gate
8. CPFR-056: Fast-Forward Local Master Without Push

Rules:

- Technical prose authority belongs under knowledge/ and registered packet
  paths, not frontend code.
- Every result claim needs a local evidence route and claim ceiling.
- Atlas/search/generated artifacts must be regenerated together.
- CPFR-055 is the final fresh gate on the exact integration commit.
- CPFR-056 fast-forwards local master only after CPFR-055 passes.

Per-ticket fresh-session template:

Use this for each CPFR ticket:

```text
You are implementing exactly CPFR-XXX in /Users/bytedance/workspace/harp.

Read first:
1. AGENTS.md
2. knowledge/agentic_engineering/agentic_engineering_index.md
3. knowledge/agentic_engineering/kenn_reference_architecture.md
4. docs/workstream/crouzeix-proof-reproduction/tracker.org CPFR-XXX subtree
5. The tracker subtrees of all direct dependencies
6. The relevant design sections named in CPFR-XXX DESIGN_SECTION
7. Any modules and tests named in Owned Files

Agentic-engineering constraints:
- Keep intent, reasoning scaffold, execution, verification, observability, and
  human control separate.
- Use subagents to scale bounded execution, not to transfer merge, release,
  proof, or authority-widening decisions.
- Treat subagent reports as evidence to reconcile, not as authoritative state.
- Make every side effect observable through tickets, receipts, ledgers, tests,
  or committed docs.

Use subagents:
- Spawn one read-only spec-audit subagent before editing.
- If implementation can be split into disjoint write sets, spawn worker
  subagents with explicit file ownership. Otherwise implement locally.
- Spawn one spec-review subagent and one standards-review subagent before
  committing.
- Do not let subagents stage, commit, delete, or write outside their assigned
  files unless explicitly directed.

Scope:
- Implement only CPFR-XXX.
- Preserve CPFR-070..081 and the Lean gate repairs.
- Do not push.
- Do not make proof claims beyond the ticket status.
- Do not use fake provider, Lean, review, or evidence values except in clearly
  named test fixtures.

TDD:
- Write failing tests first.
- Run the focused red command and record the expected failure.
- Implement minimally.
- Run focused green tests.
- Run tracker validation if the tracker changed.

Verification:
- Run every command in CPFR-XXX Verification Plan.
- Run git diff --check.
- If tracked payload changed, refresh docs/import-receipt.md last using the
  repository verifier's expected digest.
- Run mise run verify before landing unless the owner explicitly accepts a
  narrower gate for this ticket.
- Record command outputs and digests as evidence; do not turn raw logs into
  durable project guidance unless a lesson is distilled into maintained files.

Commit:
- Stage explicit paths only.
- Commit one focused concern.
- Commit message must include exactly one final trailer:
  Co-authored-by: TRAE CLI <noreply@bytedance.com>

Finish with:
- files changed
- tests run and results
- commit SHA
- tracker status
- subagents used and their findings
- blockers or residual risk
```

Final closeout expectations:

- master remains clean.
- Feature work is locally landed by fast-forward only.
- No push occurs.
- Generated caches may be cleaned only after verifying no relevant process is
  running.
- Preserve unrelated untracked scratch unless the owner explicitly asks to
  remove it.
```


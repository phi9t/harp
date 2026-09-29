# Agent guidance and workflow inventory

Snapshot date: 2026-09-29. Read-only survey of `/data02/home/philip.yang/workspace`.
Purpose: record the rules agents are actually told to follow, per repo, so Corenius can own the practice.

Conventions:
- Paths are relative to `/data02/home/philip.yang/workspace`. `path:N` is a line number in the file as read on the snapshot date.
- Quotes are verbatim, trimmed. `[INFERENCE]` marks a conclusion I drew; everything else is stated in the cited file.
- Skipped on purpose: worktree copies (`milano.worktrees`, `*.worktrees`), `arnold-doc-lark-deep-dive-20260924`, product-only docs, most of the ~140 bytedcli per-domain `AGENTS.md` (only the root, `src/auth`, `src/cli/commands`, and the mr-review-loop skill were read), harbor `docs`/`apps/viewer`/`rewardkit` (code-style only), sglang `.claude/rules` (not in scope), `corenius/.scratch/research/*` (research copies).
- Not read: files Arnold's `AGENTS.md` delegates to (`docs/index.yaml`, `docs/agent-instructions/core/policies/*.md`, arnold/AGENTS.md:17-35). Arnold's real policy is therefore under-represented here.

## 0. What exists (guidance surface map)

| Layer | Where | Repos |
|---|---|---|
| Clanker Constitution v2026.08.11 (identical text, 7 sections) | `CONSTITUTION.md` | cutlass, ds4, ferric_continuum, megatronlm, miles_diffusion, monarch, nccl, torchtitan, ultron, vaso (identical md5); lagrange-torch-mlu-insula and torchetta (reflowed copies, same rules); verl (differs, not diffed) |
| Ultron "Agentic Engineering Workflow" (gated/short/bug path, roborev gate, human merge) | `docs/agents/agentic-engineering.md`, pulled in by an `ultron-agentic-workflow` block in `AGENTS.md` | ultron (canonical), cutlass, ds4, ferric_continuum, megatronlm, miles_diffusion, monarch, nccl, torchtitan, vaso |
| Repo-specific AGENTS.md rules | `AGENTS.md` / `CLAUDE.md` | all others |
| Machine-readable policy | `verl/agentic.toml` (outside the scoped list; found while checking verl) | verl |
| SDD ledgers | `.superpowers/sdd/` (git-ignored) | corenius, devx, distributed_timeline, monarch, modular, ferric_continuum (milano has an empty `.superpowers/`) |

- No `.github/copilot-instructions.md` found in any scoped repo. flashinfer says so explicitly (flashinfer/AGENTS.md:5).
- One `.cursorrules`, a pointer: `megatronlm/.cursorrules:1` "See CLAUDE.md for all repository guidelines." Also `modular/.cursor/rules/*.mdc`.
- Ultron fleet manifest lists 9 repos: ultron, torchtitan, monarch, megatronlm, nccl, ferric_continuum, vaso, miles_diffusion, cutlass (`ultron/docs/agents/agentic-workflow-repos.json`). ds4 carries the block and an extended copy but is not in that list. [INFERENCE: ds4 drifted out of, or was never added to, the manifest.]
- CLAUDE.md conventions vary: pure import (`arnold/CLAUDE.md:3` `@AGENTS.md`; `torchetta/CLAUDE.md:5`), constitution import (`ds4/CLAUDE.md:3`, `monarch/CLAUDE.md:3`), identical copy (bytedcli, harbor, megatronlm, modular root, stratum, torchtitan `.claude/CLAUDE.md`), or deliberately separate (`ferric_continuum/CLAUDE.md:5-7` "intentionally keeps AGENTS.md and CLAUDE.md separate").

### Policy version drift (Ultron workflow copies)

| Version stamp (line 1) | Repos |
|---|---|
| `corenius/docs/agents/...` v2026.09.29.1 ("managed by Corenius"; only the owner name changes) | corenius-collab-spike |
| v2026.09.02.1 | ultron, cutlass, ds4 (+ evidence-bound/route-recipe/context-boundary additions) |
| v2026.08.15.2 | torchtitan (+ clean-context-subagent default, `ultron/mainline` authority section) |
| v2026.08.14.1 | ferric_continuum (+ evidence-bound/task-runs pointers), megatronlm, miles_diffusion, monarch, nccl, vaso |
| unversioned rewrite | torchetta (`torchetta/docs/agents/agentic-engineering.md:1-5`, own six-phase adaptation) |

The 09.02 change over 08.14 is only the `ultron/mainline` pointer paragraph (ultron/docs/agents/agentic-engineering.md:37-43). Six repos lack it.

## 1. Per-repo distinctive rules

Each entry lists only rules that differ from, or sharpen, the Constitution and the Ultron workflow. Constitution and Ultron content is summarized once in section 1.0.

### 1.0 Shared baseline

Constitution (cutlass/CONSTITUTION.md, same in 10 repos):
- Honor the request; mode matters: ":11 explain, review, and diagnose are read-only; change, build, and fix include implementation and verification."
- Escalation: ":15 Ask only when a missing decision materially changes the result, required authority is absent, or an action is destructive, irreversible, or outside the requested scope."
- Merge: ":18 Do not merge a pull request without user authorization. Plan or specification documents do not grant merge authority."
- Destructive: ":28 Do not reset, discard, stash, overwrite, or rewrite existing work without explicit authorization." ":29 Never amend a commit unless explicitly requested." ":31 When corrected or told to stop, stop mutating state."
- Verification: ":34 Test behavior and contracts, not source text, configuration tautologies, or mocked versions of the same logic." ":37 Never claim success without fresh evidence. Distinguish verified facts, inferences, and unverified assumptions."
- Memory: ":47-48 Put durable project guidance in AGENTS.md ... Do not create agent-private memories instead of updating shared instructions."
- Precedence: ":4 Direct user instructions and more specific repository instructions override these defaults."

Ultron workflow (ultron/docs/agents/agentic-engineering.md):
- Thesis: ":4 Scale agent execution, not agent authority." ":6-8 The human owns architectural authority and merge."
- Path choice: short path only if one repo, one session, reversible, narrow, and no API/schema/migration/security/destructive/concurrency/prod/architecture (:45-48); otherwise gated path; "Record the classification and its evidence in the durable ticket before implementation begins" (:50-51).
- Gated path gates, in order (:72-115): Intent, Design (human-approved), Independent critique ("separate context that did not inherit the design", prefer another model family, :81-83), Specification (tracker spec, blocker-linked tracer-bullet tickets, no duplicate `docs/superpowers/specs|plans`, :84-89), Isolated execution (":90 each ticket in its own worktree"; ":91 never share a mutable checkout"), Behavioral proof (red-green), Ticket conformance (Standards + Spec axes), Independent verification (":99 Run `roborev review --branch`"; ":101 Passing tests do not replace this gate"), Repair (max 3 iterations, escalate non-convergence, :102-106), Human control, Distillation (:111-115).
- Bug path: fast red-capable command first, 3-5 ranked hypotheses, one variable at a time, ":65 After three failed fixes, stop patching and escalate to architectural review."
- Git authority: ":108-110 Commit, push, and pull-request creation each require explicit authorization. Only a human performs the merge; agents stop at the reviewed branch even when otherwise authorized." ":127 Skill text never grants Git authority."
- Boundaries (:117-130): durable intent lives outside prompts/conversations/memory; verification "has its own budget"; extending the 3-iteration budget "requires a human decision".
- Skill orchestration (ultron/docs/agents/skill-orchestration.md): precedence is user > repo policy > phase owner > supporting skill (:6-11); "Agents never perform merges" (:76); durable vs ephemeral artifact table (:80-86); fallback when a skill is not invocable: "do not skip the phase or pretend the skill ran" (:21-25).

### 1.1 Repos with rich, distinctive rules

**milano** (milano/AGENTS.md; no Constitution, no Ultron block)
- Verification/evidence: gates run in the Insula against the staged index only ("`:27` archives the **staged Git index** ... unstaged edits are silently absent"). ":88 Local macOS checks do not authorize a runtime or release claim. The mandatory publication gate is `b200-release` on the exact packaged artifact." Evidence "content-addressed and append-only" (:242); receipts re-validated at publication (`milano/AGENTS.md` architecture section).
- Git: one worktree per task via `python3 -m milano.scm new` (:192); main checkout "never commit or stage in it" (:196); "One task per commit, staging only that task's files" (:199); commit message includes `Verified in the Insula: ...` with gates and evidence run id (:200); "Agents add their Co-Authored-By: trailer" (:202); tick roadmap boxes in the same commit (:203).
- Merge: agents integrate by tool, not by hand: "`:205` Integration, never by hand" via `milano.scm integrate --gate ...` (rebase, run gates on the exact commit, fast-forward, push, retry). ":218 'Merge' means rebase plus fast-forward. Mainline is linear: no merge commits, no force pushes, no deletion." ":220 Force-push (with --force-with-lease) only your own topic branch."
- Gates by change type: anything under `torch_accel/` needs `runtime-test`; distributed also needs the NCCL collectives gate; milestone exits need `package` then `b200-release` (:214-216).
- Resources: ":224 GPUs are leased, never assumed" (flock leases, 900 s wait, shared across sandboxes). ":231 Never delete another run's state by hand, and run cache-gc --apply only after reviewing its dry run." Evidence is append-only.
- Forbidden: blanket fallback ("`:240` Reject unsupported operations explicitly"); reintroducing legacy backend names (audit script); generated state in the checkout (:243).
- Roles: multiple peer agents, each in its own worktree; no lead/follower.

**torchetta** (torchetta/AGENTS.md, CLAUDE.md, docs/agents/agentic-engineering.md)
- Law: "Scale execution. Bound authority. Verify independently. Preserve intent." (AGENTS.md:38)
- Authority: "Merge requests, branch merges, artifact publication, destructive cleanup, and high-cost live GPU launches require explicit user authorization for that specific action." (docs/agents/agentic-engineering.md:130-131)
- Destructive: "Avoid destructive cleanup unless the user explicitly asks" (AGENTS.md:167); dry-run first and confirm latest successful build of both targets is retained (docs/agents/agentic-engineering.md:122-125); on failure "record the blocker instead of deleting state to make the next attempt cleaner" (:136).
- Evidence: production evidence only from inside the rootfs ("Treat build and verifier execution outside that rootfs as diagnostic only, not production evidence", AGENTS.md:144-146); verifier levels are not interchangeable ("Passing artifact checks do not imply compiler, C++ consumer, distributed runtime ... success", agentic-engineering.md:159-162).
- Secrets/privacy: never paste credentials or private repository output (AGENTS.md:166); pre-commit hook rewrites staged files through `tools/desensitize_source.py` (CLAUDE.md:80-83); TOS publish only to the verified US bucket (AGENTS.md:171-174).
- Publish is dry-run by default; real publication needs explicit execution flags (agentic-engineering.md:110-119).
- Tracker: gitignored `.scratch/` with `open -> claimed -> resolved` (AGENTS.md:47-49; CLAUDE.md:84-85).

**lagrange-torch-mlu-insula** (AGENTS.md; own CONSTITUTION copy; no Ultron block)
- Phase order is mandatory ("`:46` Follow these phases in order"); workflow order is test-enforced (`tests/test_workflow.py`, :171).
- Destructive: ":196 Do not delete cache, source, stage, install, rootfs, or evidence directories to recover from failures unless explicitly authorized." ":198 Record failures with command, exit status, log path, image digest, source commits, and relevant environment variables."
- Claim scope: ":182 Off-device validation proves build provenance ... It does not prove MLU execution correctness."
- Forbidden: image-preinstalled torch as release output (:189); silently replacing NeuWare packages with pip/uv (:194); publishing a package that only references an external Insula (:105).

**soma** (soma/AGENTS.md)
- "Insula" means rootfs-owned execution, not merely bwrap (:9-11). Fail closed on network: `share-net` requires a matching per-task declaration (:22-24). "Do not call a run hermetic when Emacs is inside Insula but Jupyter kernels, LSP servers ... resolve from undeclared host state." (:160)
- Patched Emacs only when probes prove Lisp-level instrumentation cannot work, and every patched claim needs a stock-vs-patched comparison (:43-52). Development wrapper output must stay labelled `not_rootfs_owned` (:26-28).

**robust-training-observatory** (AGENTS.md)
- Terminology contract shared "with the sibling repos ... (milano, lagrange-torch-mlu-insula, distributed_timeline)" (:16-17): hermetic, isolated, offline, reproducible are four separate properties; "Conflating them ... is the confusion this table exists to prevent" (:20-21). "sandbox" is reserved for TraeCode CLI (:38).
- Provenance: every rendered claim must resolve to a pinned anchor; `npm test` enforces it (:115-116). "Arnold is the product authority. We do not push to `mlsys/arnold`" (:112). "Never commit `.cache`, `.venv`, `out/`, or py-spy output; a dump on a shared host contains other people's processes." (:119) "Research notes from parallel sessions may be untracked ... leave them alone." (:123)
- Duty: rebuild and republish `dist/` to the Goofy Preview alias after any page change (:121).

**stratum-observatory** (AGENTS.md)
- Evidence integrity: ":17 A new fingerprint is not an approval. Do not overwrite accepted source bytes, reviewed subjects, runtime receipts or trust roots to silence drift." ":19 Every figure written there must come from a report actually generated on the current bytes -- never copied from a task description, a commit message or a previous run."
- Reporting: "`:21` report BLOCKED/NOT_RUN faithfully"; exit codes 0 PASS, 1 contradiction, 2 BLOCKED/NOT_RUN (:48). "No `continue-on-error`, fake raw hashes, invented reviewer keys, copied signatures or benchmark claims from payload/rate calculations." (:21)
- Test-first: "Write a failing behavioral/negative test, implement locally, run `npm run verify:engineering`." (:21). Receipts go stale on any edit under `src/`, `tools/`, `verification/` (:78).
- Forbidden: no weights, credentials, telemetry, bundled fonts, external runtime scripts (:23); never `pip install` into an ambient interpreter (:51); never hand-edit generated `src/pytorch-source.mjs` or `src/worked-results.mjs` (:147, :155).

**distributed_timeline** (AGENTS.md)
- Branching: `byted` is local mainline; feature branches `phi9t/<agent>/<feature_name>`, short, lowercase (:54-64).
- Secrets: "Do not add new secrets to source files, tests, docs, or generated traces." (:283); named landmines: `config_cn.yaml` (credentials packaged in artifacts), `test_conn.sh` ("do not use it as a routine validation command or paste its output", :291-293); avoid logging full BMQ payloads (:294).
- Tests: prefer synthetic fixtures; credentials must come through the existing config mechanism (:304-306); fragile FSDP patches have AST tests that must be updated with the implementation (:228-230).
- Import-boundary rule with a test (`bytedance.ndtools` must not import the runtime, :33-35).
- Experiment workflow is delegated to `docs/superpowers/workflows/minimal-parallel-ndtimeline-experiments.md` (:132-133). GPU: "auto-selects idle GPUs, refuses busy GPUs by default" (:135).

**ds4** (AGENTS.md, CLAUDE.md, docs/agents/evidence-bound-method.md, task-runs.md)
- Correctness: "Preserve correctness before speed. Do not keep a faster path with unexplained attention, KV cache, or logits drift." (AGENTS.md:26-27); "Do not add permanent semantic variants behind flags" (:44); "Do not introduce C++" (:46).
- Safety: "Do not run multiple huge model processes concurrently. The instance lock is intentional." (:52); avoid large CPU inference runs (:50).
- Test triggers: at every major change run `scripts/ds4 verify`, SSD-streaming test, keep CPU oracle building; distributed/tensor-parallel test only after "ask the user" (:99-106). "`scripts/ds4 verify` ... the repeatable proof ... before claiming the engine works." (:83-86)
- Do not trust a filename to identify a build; use `--build-id` (:79-82). Constitution pinned to a reviewed tag; "Update it through normal review, not by fetching a newer version at startup" (CLAUDE.md:8-9).
- Evidence-bound method (docs/agents/evidence-bound-method.md): roles are responsibilities, not agents (:47); "An agent produces a candidate. Verification produces evidence. An authorized decision accepts the outcome. None substitutes for another." (:76-77); implementer "cannot declare its own evidence sufficient by assertion" (:54-55); verifier "does not modify the candidate while reviewing it" (:56-57); "Human authority covers merge, push, deployment, publication, security-policy changes, external effects, and destructive cleanup." (:25-26). Outcome Packet template (:98-117) includes escalation conditions and budgets. Uncertainty ledger statuses: observed / inferred / unknown (:122-135). Record baseline failures separately from regressions (:167-169).
- Task runs (docs/agents/task-runs.md): Engineering Task -> committed Task Contract -> Task Run in one dedicated worktree -> replaceable Agent Attempt (:9-30); four modes delivery/remediation/scout/experiment (:95-110); `task-start` admission verifies clean dedicated worktree, contract as a Git blob, no live owner (:114-131); checkpoint policy includes `before-external-side-effect` and `before-handoff` (:78-88).

**monarch** (AGENTS.md, CLAUDE.md, Constitution + Ultron block)
- Default execution: "`:55` Clean-context subagent execution is the default execution rule for non-trivial work" with synthesis, sequencing, integration in the main thread (:55-62).
- Execution boundary: "`:82` Every Linux-local Monarch command runs through `scripts/run`, the sole gateway into the hermetic bwrap rootfs", with an enumerated exemption list (:115-119).
- Code: "Never commit code that fails the type checkers" (:169); "For large changes, include a 'walkthrough' in the commit message" (:170); refactor in-repo without back-compat because upstream is imported internally, GitHub PRs are not merged directly (:41-44); flaky CI tests disabled by filing a `DISABLED <test>` issue (:277-279).
- 8-GPU verifier: acceptance ladder and "Failure classification" - Python failures acceptable only if each failed node passes in isolation (:226-229); acceptance artifacts are named (:233-238).
- Ownership: "scripts own executable behavior, AGENTS.md owns repo-level policy, and repo-local skills under .agents/skills/ own agent procedure" (:240-242).

**torchtitan** (AGENTS.md = .claude/CLAUDE.md; docs/agents/*; .claude/rules/*)
- Tooling override: do not use `search_files`/`meta:code_search`; inspect with `rg` (:5-8).
- Rootfs discipline: "Host shell is only for orchestration" (:277); "Do not invoke host-side Python for setup, preflight, data preparation, training, evaluation, parsing, or summarization of real GPU runs" (:287-288); "Full-job preflights must not skip distributed or CUDA gates unless the run is explicitly diagnostic and the result artifact says so" (:298-299).
- Claim classification: "Smoke runs prove plumbing only" (:320); "Never use `deterministic_warn_only`; a warning-only run is not numerical proof." (:264-265); performance tests at least 10 steps (:385); loss/grad norm bitwise identical when identity is promised (:379).
- Keep generated datasets, checkpoints, rollouts out of git (:322).
- Completion criteria: focused + broader tests, evidence at stated standard, compatibility preserved, `pre-commit` passes, PR explains why the change belongs at this seam (:462-472). "Root cause before patch" (:397).
- Orchestration: "the main agent should behave as an orchestrator: dispatch a fresh task subagent ..., dispatch a separate review subagent ..., then dispatch a finalizer subagent" to inspect both traces and propose improvements (:487-492).
- Branch authority (docs/agents/ultron-mainline.md): "Agents still do not commit on, rebase, reset, or force-update `ultron/mainline`. A human authorizes each fast-forward and each push." (:23-24); published-trunk exception ADR 0006 (:16-28); integration worktree is exactly `<repo>/.worktrees/ultron-mainline` and ticket worktrees sit beside it (:38); "Every worktree stays within its own repository" (Ultron workflow copy).
- Stop rule: "Repeated audit or launch-boundary checks stop when a clean-context verifier confirms the same blocker and no state has changed" (docs/agents/agentic-engineering.md:138-140).
- Path rules: `.claude/rules/experiments.md` - experiments must lint but core must not change for them.

**vaso** (AGENTS.md, docs/agents/cross-agent-collaboration.md, experiments/AGENTS.md)
- The only written lead/follower protocol. Roles (cross-agent-collaboration.md:19-29): Lead works in its own worktrees, owns specs/tickets/guards/landing branch; Follower works in the primary checkout and long-running env, owns builds and insula proofs. "The human decides direction, authorizes landing, relays the first message."
- Invariants (:31-55): one writer per working tree; "The target branch moves only by fast-forward"; durable intent in tracker, inter-agent messages are files; guards are the contract ("Neither agent negotiates them in prose"); only discard what you have proven superseded (content-hash manifest); "The landing ref only ever points at verified commits"; "Green means the lead ran it on the exact tip."
- Failure-mode table with rule per incident (:165-183), e.g. a rebase moved the landing ref to a known-red commit and the follower landed it within 30 seconds -> rebase detached, verify, then move the ref.
- I/O hygiene: every agent has a declared disk-backed root; tmpfs roots refused; one reused Bazel output base per role (:146-163).
- experiments/AGENTS.md:8-12 - "land each verified native package capture as its own package-scoped change before starting the next package"; not done while work exists only as a dirty worktree.

**roborev** (AGENTS.md, CLAUDE.md, docs/plans/2026-08-agentic-engineering-law.md)
- Git prohibitions: "Never amend commits" (AGENTS.md:209; CLAUDE.md:401); "Never push/pull unless explicitly asked" (:210; CLAUDE.md:402); "NEVER merge pull requests" (:211; "non-negotiable", CLAUDE.md:403); "NEVER change git branches without explicit user confirmation" (:212; CLAUDE.md:404); never `make install` or `go install` of the daemon binary (:213).
- Review: "Never invoke the `roborev review` CLI command in any form unless the user explicitly asks" and never invoke roborev skills unasked (:240-244). Pasted findings are direct fix input (:253).
- Daemon rule: background jobs never edit tracked source in the user's checked-out tree (:33); isolated fixes use temp worktrees with patches stored in the DB (:35).
- Privacy: no hostnames, usernames, job IDs, review IDs, log excerpts, or machine-specific behavior in tests, commits, PRs, issues, docs (:201-208).
- Release testing must not publish anything (:215-222).
- Tests: testify only, no `assert.True(x == y)`, no bash tests that grep scripts for expected text ("usually tautological", :179-184).
- PR body: "Do not add navel-gazing PR sections for validation, testing, checks run" (:232-236); "do NOT include a 'Test plan' section" (CLAUDE.md:393).
- Commits: "Commit after completing each piece of work; do not wait to be asked." "stage ALL modified files" (:227-228; CLAUDE.md:21).
- Session end: "Landing the Plane ... Work is NOT complete until `git push` succeeds ... NEVER say 'ready to push when you are'" (:256-279). This contradicts :210 in the same file (see conflict C2).
- Law doc (docs/plans/2026-08-agentic-engineering-law.md:100-132): seven supporting laws (bounded agency, independent verification budget, durable intent, isolated execution, human-owned merge, observable sessions, distilled architecture memory). Gaps named: verification budget not modeled, authority boundaries "enforced by convention", Loop B (harness learning) absent (:172-201).

**megatronlm** (AGENTS.md; third_party/verl, Megatron-Bridge, local/verl copies)
- Skills are mandatory context, with a fixed order: pull artifact, select skill, then answer ("Never skip or reorder these steps", AGENTS.md:10-19).
- PRs: "All PRs must be created as **drafts**" (:25); "Never push branches directly to NVIDIA/Megatron-LM ... push to a personal fork" (:26).
- Paths: "Never use `/tmp` for repo workflows, tests, launchers, caches, or generated artifacts" (:35); "Never use absolute filesystem paths in docs, examples, tests, launchers, or source code" (:37).
- Megatron-Bridge (third_party/Megatron-Bridge/AGENTS.md:27-44): NEVER modify `3rdparty/Megatron-LM`, run the full test suite, add required dependencies, commit secrets or env-specific paths; ASK FIRST before new dependency, CI workflow edits, public API changes in `models/conversion/`; ALWAYS run pre-commit, sign off commits (`git commit -s`), use `uv run`, use the current year.
- verl copy (third_party/verl/AGENTS.md): "Breaching these guidelines can result in automatic banning." (:4) Duplicate-work `gh` checks before a PR (:8-19); no busywork PRs (:21-23); "Pure code-agent PRs are **not allowed**. A human submitter must understand and defend the change" (:27); PR text must state AI assistance and tests run (:29-32); attribution trailers `Co-authored-by:` (:56-67); "verify their suggestions against the current state of the repo" for bot reviews (:71); fail closed on duplicate work (:34-36).

**bytedcli** (AGENTS.md = CLAUDE.md, Chinese; src/auth, src/cli/commands sub-files)
- Scope/routing: "修改某 domain 前，必须确认是否存在对应子模块 AGENTS.md 并优先以其约定为准" (:20).
- Branch/MR: "默认走分支开发与 MR 流程，不要直接 commit + push 到主分支" (:269); only version-release commits may go to the main branch (:270); MR description defaults to a `## Summary` only (:279).
- Review gate: before creating an MR, pushing to an existing MR, or pinging the review bot, "Agent 必须先对当前 MR 相对目标分支的最终净 diff 独立跑一次 /mr-reviewer" (:301).
- Release: do not run `npm publish` manually (release script does it, :306 "发版时不要手动执行 npm publish").
- Verification honesty: if the requirement says verify with the local build, do not substitute the `ts-node` source path and still claim compiled-artifact verification (:258).
- Secrets: browser-captured write endpoints must never store Cookie/JWT; "默认 dry-run 预览完整 payload，必须显式 --yes 才提交" (:164); credential commands mask output in text mode (:182); "SK 不得作为 CLI option、位置参数、日志或输出字段" (src/auth/AGENTS.md:32); `x-zti-token` and `x-jwt-token` never in logs or error details (src/auth/AGENTS.md:59); fail closed on unknown host/site (src/auth/AGENTS.md:73).
- Fixtures: no real online services, tokens, cookies, JWTs in docs, fixtures, mocks (:43).
- Repo-level bans: platform ban list (:91-97) and proxy-binary registry with mandatory public source URL (:99-107).
- Documentation duty: sync `skills/*/SKILL.md`, README, website only on defined triggers (:49-89); never hand-edit generated skill mirrors.

**modular** (AGENTS.md = CLAUDE.md; max/mojo/kernels CLAUDE.md; .cursor/rules)
- "Sign commits with `git commit -s`" (AGENTS.md:165; mojo/CLAUDE.md:138 "Always sign commits with Signed-off-by"); "Never commit secrets or large binary files" (:202); commit titles need a component tag, checked by CI (:293-296); work from `main`, feature branches for significant changes (:142-145).
- "Performance improvements must include benchmarks" (:194; mojo/CLAUDE.md:143).
- Hermetic: "All kernel development, Bazel builds, and Mojo GPU commands must run inside the bwrap rootfs" (:236-238).
- `.cursor/rules/git.mdc` (generic): commit frequently, "Avoid Direct Commits to Main Branch", "Sign Your Commits (Optional but Recommended)" (:11) - softer than AGENTS.md, an internal inconsistency on signing.

**arnold** (AGENTS.md, 35 lines)
- "Always respond in Chinese-simplified" (:1). Intent-based authorization: "'能帮我修复吗'是修复请求；解释、评审和状态查询只授权读取与回答" (:7). "共享工作副本不切分支、不改 Git 暂存区；新修改任务使用独立 worktree" (:9). "不输出凭据" (:10). "代码通过验证、部署稳态、线上行为生效是不同结论" (:11) - three separate claim levels.
- "语义修改前读取 design-first policy" (:33). Full policy is external (see not-read note above).

**miles_diffusion / cutlass** (AGENTS.md + Ultron block)
- "Land on local `main` only; do not push unless explicitly asked." (miles_diffusion/AGENTS.md:46-47; cutlass/AGENTS.md:40 "'Land on mainline' means commit to the local main branch only.")
- cutlass: "CUDA builds MUST run inside the rootfs ... Never work around it by building CUDA on the host" (:43) with a named expert escape hatch; "Do not use automatic formatters like clang-format" (:30); "Keep formatting-only changes separate from functional edits" (:31); upstream sync by rebasing fork commits onto a release tag (:44).
- miles_diffusion: canonical policy files "are synchronized from Ultron ... and should be edited there, not here" (:5-6).

**ferric_continuum, nccl, ultron, vaso (root), ds4 skills**: local Markdown/org tracker under `.scratch/` (ferric_continuum/AGENTS.md:59; nccl/AGENTS.md:5; ultron/AGENTS.md:5), triage labels, domain docs. ultron scope: "Park adjacent model, kernel, release, or generic agent-workflow material unless it answers a specific robust-training controller question." (ultron/AGENTS.md:19-21). [INFERENCE: ultron's own guidance says generic agent-workflow material is out of scope, yet ultron is the canonical source of the fleet workflow; ownership is now moving to Corenius, which removes that tension.]

### 1.2 Repos with mostly build/style/product content (few agent rules)

- **flash_attn** (AGENTS.md, CLAUDE.md): architecture summary and test recipes; only rule-like lines are "If you get OOM errors ... select [a free GPU] with `CUDA_VISIBLE_DEVICES`" (CLAUDE.md:50) and a security section noting build-time submodule fetch and wheel download (AGENTS.md:123-126).
- **flashinfer** (AGENTS.md, CLAUDE.md): "NEVER write to package directories" (CLAUDE.md:195); "Torch headers MUST NOT be included in `include/` directory files" (CLAUDE.md:327); pre-commit; security notes on checksum-disable flags and `FLASHINFER_LOGLEVEL=10` dumping all inputs/outputs (AGENTS.md:130-137). No git, review, or escalation rules.
- **harbor** (AGENTS.md = CLAUDE.md): "When verifying changes, only run `uv run pytest tests/unit/`" (:218); "Do not test CLI help panels or command console output" (:220); "Always run `uv run ruff check --fix .`, `uv run ruff format .`, and `uv run ty check` after making any code changes" (:260); Supabase CLI over MCP for remote DB mutation (:376); paginate queries over 1,000 rows (:377); CHANGELOG only for breaking changes (:3). No git rules, refers to CONTRIBUTING.md (:7).
- **monarch_tech_review** (AGENTS.md, CLAUDE.md): run everything through the Zephyr container, "Do not install upstream PyTorch/CUDA wheel packages in `.venv`" (AGENTS.md:32); short lower-case commit messages; PRs list "exact verification commands you ran" (:59) - opposite of roborev's no-validation-list rule.
- **sglang/docs** (AGENTS.md): docs-writing rules. "Do not guess flags, defaults, or behavior" - verify against upstream code or vendor docs (:18-22); use `docs.json` not `mint.json`.
- **ui-ux-pro-max-skill** (CLAUDE.md): "Never push directly to `main`. Always: create branch, commit, push branch, `gh pr create`" (:113-118) - instructs the agent to push and open PRs unprompted.

### 1.3 Repos with no agent-facing rules beyond the Constitution and the Ultron block

nccl (AGENTS.md:1-22), ultron (:1-30), vaso root (:1-17 plus the collaboration pointer), ds4 skills section, ferric_continuum (mostly a planned-agents roadmap, not rules; ferric_continuum/AGENTS.md:24-52 describes agents that "do not exist today").

## 2. SDD / .superpowers workflow (characterization)

Sampled: `corenius/.superpowers/sdd/corenius-agentic-execution-core-ink-tui/{progress.md,reviews/*}`, `devx/.superpowers/sdd/2026-09-13-doom-ai-lsp/*` and `2026-09-15-ai-lsp-certification/progress.md`, `distributed_timeline/.superpowers/sdd/2026-08-26-minimal-parallel-ndtimeline/progress.md`, `monarch/.superpowers/sdd/01-inventory-and-disposition/*`, `modular/.superpowers/sdd/2026-08-22-gpu-kernel-optimization/*`, `ferric_continuum/.superpowers/sdd/spec.org/progress.md`. Roughly 150 files read by name, ~15 opened.

Structure (per plan directory `.superpowers/sdd/<plan-slug>/`):
- `progress.md` - the SDD ledger. Header `# SDD ledger - plan: <plan path or inline>`; base commit; branch; a "Preflight scan" table (`Tasks | Producer -> consumer | Finding`) checking cross-task interface consistency before dispatch (devx progress.md:6-24); "Ruling:" lines that record a decision as `X - because Y - cost if wrong Z` (devx progress.md:26; distributed_timeline progress.md:8); then a running log per task: `dispatched (base <sha>, implementer <id>)`, `reviewer could not verify ...; controller resolved from ...`, `fix round n/5 (a addressed, b open; commits x..y)`, `complete (commits x..y, review clean)`.
- `task-N-brief.md` - the implementer's unit: Files (Create/Modify), Interfaces (Consumes/Produces), checkbox Steps with the failing test first (devx task-1-brief.md:1-14; modular task-1-brief.md:8-30).
- `task-N-report.md` - implementer report with status (e.g. `DONE_WITH_CONCERNS`, monarch task-1-report.md:5), changed paths, commands run and results, RED/GREEN evidence (devx final-fix-report.md:15-20).
- `review-<sha>..<sha>.diff` - the exact range handed to the reviewer.
- Review outputs: two independent axes, Standards and Spec, each with a verdict and severity counts (Critical / Important / Minor or Hard / Judgment) - monarch `task-1-standards-review.md`, `task-1-spec-review.md`; re-reviews are numbered (`-rereview-1..4`). A "final review package" for the whole branch (corenius `reviews/final-review-package.md`).
- `final-fix-report.md` for post-final-review fixes.
- `.superpowers/sdd/.gitignore` is `*` (devx), so all of this is untracked and ephemeral. Durable output goes to tracker/docs. monarch's Standards review flagged a hard violation when durable evidence depended on `.superpowers` traces (monarch task-1-standards-review.md:15).

Flow (inferred from the ledgers, [INFERENCE] where noted): plan approved -> controller writes preflight scan and rulings -> per task: brief -> implementer subagent (own id) works red-green and commits -> controller reviews the commit range -> Standards + Spec reviewers -> fix rounds up to 5 -> "review clean" -> next task -> final whole-branch review with a critical/important triage (corenius progress.md:28-40: reviewer found one Critical, fix round 1/1, re-verified) -> human acceptance (`ready-for-human`, monarch progress.md:57). Roles: controller (lead, sequencing, rulings, resolves reviewer gaps), implementer (fresh subagent per task), reviewer(s) (separate, per axis), human (acceptance). [INFERENCE: role names come from ledger wording, not from a written policy in these repos; the skill itself lives outside the workspace.]

Observed variations and drift:
- Isolation: the Ultron rule is a worktree per ticket, but three ledgers ruled otherwise: devx (existing checkout required by path-bound loaders, progress.md:26), distributed_timeline ("using current checkout because branch is not main/master, tree is clean", progress.md:7-8), monarch (a dedicated worktree "would create a ref without the separately required Git authorization", progress.md:11). Each records the cost-if-wrong. So the worktree rule is routinely overridden by ruling, with the reason documented.
- Authority: monarch ledger states "Authority: local file edits and read-only verification only. No commit, push, merge, deletion, publication, remote mutation, or ref creation." (progress.md:7), while devx and distributed_timeline implementers commit per task. Authority is set per run, not by repo policy.
- Fix budget: ledgers say `n/5`; Ultron says max 3 (`agentic-engineering.md:102`); `verl/agentic.toml` says `max_automatic_fix_rounds = 3`.
- Plan artifact location: devx and distributed_timeline hold plans/specs under `docs/superpowers/plans|specs` (devx progress.md:1, distributed_timeline progress.md:1-6); Ultron says do not duplicate them (`agentic-engineering.md:87-89`); roborev requires removing `docs/superpowers/` from the branch before a PR (roborev/AGENTS.md:231); ferric_continuum uses `.scratch/*.org` as the plan (ferric progress.md:1).
- Ledger discipline is uneven: modular and ferric ledgers are sparse (no fix rounds, no reviews directory); milano has no ledger.
- The devx ledger notes a fallback when a tool is unavailable ("controller resolved from the report's captured non-zero command evidence") - reviewers sometimes cannot re-run RED; the controller substitutes captured evidence. That is a weaker standard than "fresh evidence" (Constitution :37). [INFERENCE]

## 3. Synthesis

### 3.1 Rule taxonomy (rule -> repos of origin -> strongest wording)

**A. Authority and Git**
| Rule | Repos | Strongest wording |
|---|---|---|
| Human performs every merge; agents stop at reviewed branch | Ultron fleet (10), torchetta, torchtitan, roborev, verl toml, Constitution | "agents stop at the reviewed branch even when otherwise authorized" (ultron/docs/agents/agentic-engineering.md:109-110); "NEVER merge pull requests ... non-negotiable" (roborev/CLAUDE.md:403) |
| Commit/push/PR each need explicit per-action authorization | Ultron fleet, torchetta, roborev (push), cutlass/miles (push), verl toml | "Skill text never grants Git authority" (ultron/.../agentic-engineering.md:127) |
| Never amend | Constitution (10 repos), roborev | "Never amend a commit unless explicitly requested" (cutlass/CONSTITUTION.md:29); "Never amend commits; fixes should be new commits" (roborev/AGENTS.md:209) |
| No reset/stash/discard/rewrite of existing work | Constitution, milano (no force-push to shared), torchtitan ultron/mainline | cutlass/CONSTITUTION.md:28 |
| Mainline is never rewritten / linear | milano, torchtitan, vaso | "no merge commits, no force pushes, no deletion" (milano/AGENTS.md:218); "moves only by fast-forward" (vaso cross-agent:36) |
| Do not switch branches without confirmation | roborev, arnold (shared working copy) | "NEVER change git branches without explicit user confirmation" (roborev/AGENTS.md:212) |
| Draft PRs; push only to a personal fork | megatronlm | megatronlm/AGENTS.md:25-26 |
| DCO sign-off | modular, Megatron-Bridge, verl copy | "Sign off commits: `git commit -s`" (Megatron-Bridge AGENTS.md:42) |
| Pure agent PRs banned; disclose AI use | verl copy | "Pure code-agent PRs are **not allowed**" (megatronlm/third_party/verl/AGENTS.md:27) |
| Feature branch + MR; no direct push to default branch | bytedcli, ui-ux, modular | "不要直接 commit + push 到主分支" (bytedcli/AGENTS.md:269) |

**B. Worktrees and shared state**
| Rule | Repos | Strongest wording |
|---|---|---|
| One worktree per task/ticket; never share a mutable checkout | Ultron fleet, milano, arnold, verl toml | ultron/.../agentic-engineering.md:90-91; "never commit or stage in [the main checkout]" (milano/AGENTS.md:196) |
| One writer per working tree | vaso, verl toml | "No agent edits, restores, stashes or moves files in a tree another agent is using" (vaso cross-agent:33-35) |
| Never touch another agent's worktree/state | milano, vaso, robust (parallel notes) | "Never delete another run's state by hand" (milano/AGENTS.md:231) |
| Leases for scarce hardware | milano; ds4 instance lock; distributed_timeline idle-GPU refusal | "GPUs are leased, never assumed" (milano/AGENTS.md:224) |
| Declared disk-backed I/O roots; no /tmp | vaso, megatronlm | megatronlm/AGENTS.md:35; vaso cross-agent:146-163 |

**C. Verification and evidence**
| Rule | Repos | Strongest wording |
|---|---|---|
| Fresh evidence before any success claim | Constitution, torchetta | cutlass/CONSTITUTION.md:37 |
| Independent verification gate distinct from tests | Ultron fleet, roborev, bytedcli, torchtitan | "Passing tests do not replace this gate" (ultron/.../agentic-engineering.md:101) |
| Red-green with visible failure first | Ultron fleet, stratum, devx SDD | ultron/.../agentic-engineering.md:92-95; stratum/AGENTS.md:21 |
| Candidate-bound, content-addressed, append-only evidence | milano, ds4, vaso | "Make evidence content-addressed and append-only" (milano/AGENTS.md:242) |
| Claim-scope taxonomy (what a check does and does not prove) | milano, lagrange, torchtitan, torchetta, arnold, soma | "Local macOS checks do not authorize a runtime or release claim" (milano/AGENTS.md:88); "Smoke runs prove plumbing only" (torchtitan/AGENTS.md:320) |
| Never overwrite receipts/baselines to silence drift; report BLOCKED/NOT_RUN | stratum | stratum/AGENTS.md:17,21 |
| Tests must exercise behavior, not grep text | Constitution, roborev, lagrange, torchetta | roborev/AGENTS.md:179-184 |
| Execute only inside the hermetic rootfs; host is diagnostic | milano, monarch, torchtitan, torchetta, cutlass, modular, ds4, soma, lagrange, robust | "sole gateway" (monarch/AGENTS.md:82); "diagnostic only, not production evidence" (torchetta/AGENTS.md:145) |
| Root cause before patch; 3 failed fixes -> architecture review | Ultron fleet, torchtitan | ultron/.../agentic-engineering.md:65 |
| Determinism proof requirements | torchtitan | "Never use `deterministic_warn_only`" (:264) |

**D. Destructive actions**
| Rule | Repos | Strongest wording |
|---|---|---|
| Do not delete state to recover from failure; record it | lagrange, torchetta, milano | "Do not delete cache, source, stage, install, rootfs, or evidence directories to recover from failures unless explicitly authorized" (lagrange/AGENTS.md:196) |
| Dry-run before cleanup; `--execute`/`--apply` needs authorization | torchetta, milano | torchetta docs/agents/agentic-engineering.md:122-125 |
| Writes default to dry-run; explicit `--yes` | bytedcli | bytedcli/AGENTS.md:164 |
| Do not install a dev build over the user's production binary | roborev | roborev/AGENTS.md:213 |
| Do not run huge concurrent processes | ds4 | ds4/AGENTS.md:52 |
| Release tests must not publish | roborev, torchetta (publish dry-run default), lagrange | roborev/AGENTS.md:215-222 |

**E. Escalation triggers**
| Trigger | Repos |
|---|---|
| Missing decision changes result / destructive / out of scope | Constitution |
| Three failed fixes; repair non-convergence; budget exhausted | Ultron fleet ("attention-needed", :105-106) |
| Ask before distributed test run | ds4 (:103-104) |
| ASK FIRST: new dependency, CI workflow change, public API change | Megatron-Bridge (:34-37) |
| Same blocker confirmed by clean verifier: stop re-auditing, report needed authority | torchtitan (docs/agents/agentic-engineering.md:138-140) |
| Release tool cannot validate without publishing: stop and ask | roborev (:221-222) |
| Guideline conflicts with requested change: refuse and explain | verl copy (:77-79) |

**F. Documentation duties**
| Rule | Repos |
|---|---|
| Distill lasting knowledge into ADRs/glossary/architecture docs; keep micro-plans ephemeral | Ultron fleet (:111-115) |
| Update user-facing docs before opening a PR; drop `docs/superpowers` working docs | roborev (:231) |
| Update SKILL.md/README/website on defined triggers; generated mirrors never hand-edited | bytedcli (:49-89) |
| Comment where model mechanics or cache lifetime are non-obvious; prefer comments over design docs | ds4 (:38-42) |
| Record baseline failures separately from regressions | ds4 EBM (:167-169) |
| Every rendered claim cites a pinned source anchor | robust-training-observatory (:115) |
| CHANGELOG only for breaking changes | harbor (:3) |

**G. Secrets and privacy**
| Rule | Repos |
|---|---|
| Never commit secrets | modular (:202), Megatron-Bridge (:31), distributed_timeline (:283), torchetta (:166), bytedcli (:43, :164) |
| Mask credentials in output; never in options/logs | bytedcli (:182, auth:32, auth:59) |
| No host paths / private review data in committed text | roborev (:201-208), torchetta (desensitize hook, CLAUDE.md:80-83), megatronlm (:37) |
| Do not read or print credentials | arnold (:10) |
| Shared-host dumps contain other people's processes | robust (:119) |

**H. How work is tracked**
| Mechanism | Repos |
|---|---|
| `.scratch/` local Markdown tracker + triage labels | ds4, nccl, ultron, monarch, torchtitan, vaso, torchetta (gitignored) |
| `.scratch/*.org` | ferric_continuum |
| Kata ledger (required/optional/projection) | verl toml (required), torchtitan (projection), torchetta (Kata-shaped), roborev integration |
| GitHub issues/PRs | harbor (`DISABLED <test>`), verl, megatronlm |
| MR + Lark thread | bytedcli |
| Committed task contract + run journal | ds4 |
| Local `.superpowers/sdd` ledger (ephemeral) | corenius, devx, distributed_timeline, monarch, modular, ferric_continuum |

**I. Roles**
| Roles | Repos |
|---|---|
| Human owns architecture and merge; agents bounded | Ultron fleet, roborev, torchetta, ds4 |
| Lead / follower with landing protocol | vaso |
| Controller / implementer / reviewers (Standards, Spec) | SDD ledgers |
| Orchestrator + task subagent + review subagent + finalizer subagent | torchtitan (:487-492) |
| Clean-context subagent by default | monarch (:55), torchtitan |
| Planner/scout, implementer, verifier, integration path | ds4 EBM (:47-62) |
| Peer agents each with own worktree, integrated by tool | milano |

**J. Forbidden things (distinct)**
Reintroducing prior-backend names (milano); C++ (ds4); network-sharing without declaration (soma); host CUDA builds (cutlass); host-side Python for GPU runs (torchtitan); `tmp`/absolute paths (megatronlm); `merge`/`branch switch`/`push`/`install` (roborev); coupling `experiments` to core (torchtitan); direct `main` push (bytedcli, ui-ux); `continue-on-error` and fabricated hashes/keys (stratum); busywork or duplicate PRs (verl); mocking the same logic as the test target (Constitution).

### 3.2 Conflicts between repos

C1. Who may commit, and how much to stage.
- Ultron fleet: commit needs explicit authorization (ultron/.../agentic-engineering.md:108-110).
- roborev: "Commit after completing each piece of work; do not wait to be asked" and "stage ALL modified files" (roborev/AGENTS.md:227-228).
- milano: agents commit, "One task per commit, staging only that task's files" (milano/AGENTS.md:199) - opposite staging rule to roborev.
- cutlass/miles_diffusion: "Land on mainline" = agent commits to local `main` (cutlass/AGENTS.md:40) - conflicts with milano "never commit or stage in [the main checkout]" (:196) and with Ultron's "Only a human performs the merge" if landing on main counts as a merge. [INFERENCE: landing on local main is a merge into the integration branch.] Note cutlass and miles_diffusion also carry the Ultron block, so both rules bind them with no stated precedence beyond "more specific repository guidance".
- verl/agentic.toml: `commit_per_task = true` with `push_requires_human`, `merge_requires_human` - a third combination.

C2. Push.
- roborev is self-contradictory: "Never push/pull unless explicitly asked" (AGENTS.md:210; CLAUDE.md:402) vs "Work is NOT complete until `git push` succeeds ... NEVER say 'ready to push when you are'" (AGENTS.md:258, 277-278). The Landing the Plane section appears to be imported boilerplate. [INFERENCE]
- ui-ux says create branch, push, `gh pr create` as the routine (ui-ux-pro-max-skill/CLAUDE.md:113-118); bytedcli expects a pushed branch then `bytedcli codebase mr create` (bytedcli/AGENTS.md:269-276); milano's `integrate` pushes mainline as part of the tool (milano/AGENTS.md:205-212). Versus cutlass/miles ("do not push"), torchtitan ("human authorizes ... each push"), Ultron (explicit authorization).

C3. Merge.
- Human-only: Ultron, roborev, torchtitan, verl toml, Constitution (with authorization escape).
- Agent-executed via tool: milano `integrate` fast-forwards and pushes `diziet/mainline` (milano/AGENTS.md:205-220); vaso follower "Fast-forwarding the landing branch into the target at turn boundaries" (vaso cross-agent:25); bytedcli mr-review-loop "跟到 merge queue" and hand off to the merge queue (bytedcli/AGENTS.md:276). In milano and vaso a human authorization exists at the goal level ("The human ... authorizes landing", vaso :28) but not per merge. The Constitution's escape hatch ("without user authorization") permits these; Ultron's text ("agents stop at the reviewed branch even when otherwise authorized") does not. Corenius cannot adopt both.

C4. Force-push and rebase.
- milano permits `--force-with-lease` on own topic branches (:220) and rebase-integrate; Ultron's `ultron/mainline` is never rebased/force-updated (torchtitan/docs/agents/ultron-mainline.md:23, fleet default :13); roborev forbids amend and branch change but is silent on rebase; Constitution forbids "rewrite existing work" without authorization (:28) which a topic-branch force-push technically is; cutlass rebases fork commits onto upstream tags (:44).

C5. Amend. Constitution and roborev: never. milano, vaso, verl: silent. Only "unless explicitly requested" is a shared wording.

C6. Worktrees: whether and where.
- Location: sibling `../milano.worktrees/<agent>-<topic>` (milano/AGENTS.md:194), in-repo `.worktrees/<name>` (torchtitan ultron-mainline :38, vaso cross-agent:23, verl toml, modular `.worktrees`), unspecified (Ultron). Gitignore consequences differ.
- Necessity: Ultron requires a worktree per gated ticket; roborev forbids switching branches at all; arnold forbids branching a shared copy but requires a worktree for new modifications; three SDD runs overrode with rulings (section 2).
- Naming: `<you>/<task>` (milano), `phi9t/<agent>/<feature>` (distributed_timeline:60), `vaso/<effort>` (vaso), `feat/`/`fix/` (ui-ux:115), plain `feature/<topic>` (SDD ledgers). No fleet-wide branch name scheme.

C7. Mainline naming. `main`, `master` (nccl, vaso), `byted` (distributed_timeline), `diziet/mainline` (milano), `ultron/mainline` (fleet), `origin/master` (bytedcli, pre-push hook). The Ultron pointer branch resolves the fleet case only; milano and lagrange are outside the fleet.

C8. Generated / derived files in git.
- roborev: stage "generated files" too (CLAUDE.md:21). milano, torchetta, lagrange, torchtitan: generated state stays out of the repo (milano/AGENTS.md:243; torchetta/AGENTS.md:160-161; lagrange/AGENTS.md:159-161; torchtitan:322).
- Yet other repos commit generated artifacts as part of the contract: monarch_tech_review `docs/` HTML exports ("treat as build artifacts", AGENTS.md:11), bytedcli skill mirrors (bytedcli/AGENTS.md:60-61), stratum `src/pytorch-source.mjs`.
- Distinction (committed reproducible derivative vs generated runtime state) is implicit. [INFERENCE]

C9. Co-author / attribution trailers.
- Required: milano ("Agents add their Co-Authored-By: trailer", :202), verl ("Add attribution using commit trailers", verl AGENTS.md:56-58).
- DCO Signed-off-by required: modular, Megatron-Bridge, verl copy.
- Not mentioned anywhere else, including the Constitution and the Ultron workflow. roborev asks for privacy scrubbing of commit text (:201). The harness in this session supplies its own trailer instruction. [INFERENCE: no repo forbids a trailer, but only two require one, so trailer policy is currently decided by the harness, not by the repos.]

C10. Repair budget: Ultron 3 (:102-106) vs SDD ledgers 5 (`fix round n/5`) vs verl toml 3 (`max_automatic_fix_rounds`).

C11. Where design/plan artifacts live: tracker only (Ultron :87-89) vs committed `docs/superpowers/plans` (devx, distributed_timeline; torchetta AGENTS "research specs and `docs/superpowers/plans/` hold temporary design", docs/agents/agentic-engineering.md:15-16) vs must be stripped before PR (roborev:231).

C12. Evidence in PR text: roborev bans validation/testing sections (roborev/AGENTS.md:232-236); bytedcli puts verification in the group message and only `## Summary` in the MR (:279, :272); verl requires test commands and results plus AI disclosure (verl AGENTS.md:29-32); cutlass wants test evidence (cutlass/AGENTS.md:49); monarch_tech_review wants the exact verification commands (AGENTS.md:59); milano puts `Verified in the Insula:` in the commit (:200).

C13. Test scope: harbor "only run unit tests" (:218); Megatron-Bridge "NEVER run the full test suite" (:29); Ultron/milano "then the repository's broader required checks" (Ultron :94-95); ds4 "at every major change" run full verify (:99-106); torchtitan requires "the relevant broader suite passes" (:465).

C14. Formatting: cutlass bans auto-formatters (:30) while harbor, Megatron-Bridge, roborev, flashinfer, and distributed_timeline require or run them.

C15. Subagent default: monarch and torchtitan default to clean-context subagents (monarch:55, torchtitan Ultron block); Ultron's own short path says "Keep the primary context" (:55); roborev forbids invoking review/skills unasked; megatronlm mandates a skill read before any plan. Skill-invocation posture ranges from "mandatory" to "never unasked".

C16. Terminology collision: "Insula" - milano/robust define it as the host-side bwrap boundary plus driver interface (robust AGENTS.md:25); soma defines it as a rootfs whose whole closure is owned, "not just a namespace ... or bwrap invocation" (soma/AGENTS.md:9-14); robust forbids "sandbox" for it (:38) while monarch/ds4/torchtitan say "bwrap sandbox"/"hermetic bwrap rootfs". "Hermetic" is defined as provenance (robust:20-21, soma:20-22) but used loosely as "sandboxed" elsewhere.

C17. Tmp/cache: megatronlm forbids `/tmp` (:35); flash_attn defaults its cache to `/tmp/${USER}` (flash_attn/CLAUDE.md:45); vaso refuses tmpfs roots (cross-agent:152-155); this session's harness also prescribes a scratchpad.

C18. Language: arnold "Always respond in Chinese-simplified" (:1); bytedcli documentation in Chinese; everything else English. No fleet rule.

C19. PR draft policy: megatronlm drafts only (:25); others silent; ui-ux and bytedcli create ready PRs/MRs.

C20. Unit vs bug taxonomy: Ultron's bug path requires a failing regression test before the fix; stratum requires a failing behavioral test for all work; most repos (flash_attn, flashinfer, harbor) state no test-first rule.

### 3.3 Rules that exist in only one repo and should probably be universal

[INFERENCE for the "should"; each citation is the one repo that states it.]

1. Never overwrite evidence, baselines, or receipts to make drift pass; a new fingerprint is not an approval (stratum/AGENTS.md:17).
2. Every number in a report comes from a report generated on the current bytes, never from a task description, commit message, or earlier run (stratum:19).
3. BLOCKED / NOT_RUN as first-class results with a distinct exit code, never coerced to pass or zero (stratum:21, :48).
4. A candidate-bound evidence model: agent produces a candidate, verification produces evidence, an authorized decision accepts (ds4/docs/agents/evidence-bound-method.md:76-77); implementer cannot self-certify; verifier does not modify the candidate (:54-57).
5. Separate baseline failures from regressions; a missing prerequisite is not a successful check (ds4 EBM :167-169).
6. Claim-level separation: code verified vs deployed vs behaving online (arnold:11) and "smoke proves plumbing" (torchtitan:320), "off-device validation does not prove MLU execution" (lagrange:182).
7. Do not delete or clean state to recover from a failure; record command, exit status, log path, identity (lagrange:196-199; also torchetta docs :136).
8. Dry-run first for cleanup and default writes to dry-run with an explicit confirm flag (torchetta docs :122-125; bytedcli:164).
9. Background/daemon agent work never edits the human's checked-out tree (roborev:33, :251).
10. No private review data, hostnames, usernames, job IDs in committed text (roborev:201-208), enforced mechanically by a desensitizing pre-commit hook (torchetta CLAUDE.md:80-83).
11. Stop re-auditing when a clean verifier confirms the same blocker and nothing changed (torchtitan docs/agents/agentic-engineering.md:138-140).
12. ASK FIRST list: new dependencies, CI workflow edits, public API signature changes (Megatron-Bridge:34-37).
13. Single-writer-per-tree plus landing ref only points at verified commits, with "green means run on the exact tip" (vaso cross-agent:33-55). Milano's "gates run on the exact commit it will push" (milano/AGENTS.md:205-212) is the same idea; generalize.
14. Content-addressed, append-only evidence (milano:242).
15. Fail closed on ambiguous identity/network/authority (soma:22-24; bytedcli auth:73).
16. Pin a constitution version and update it only via reviewed change (ds4/CLAUDE.md:8-9; monarch/CLAUDE.md:8-10).
17. Session-end handoff: record open items that live only in context (vaso "Lead handover" :185-200; roborev "Hand off - Provide context for next session", :273) - though roborev couples it to a push.
18. Do not hand-edit generated files; regenerate (stratum:147,:155; bytedcli mirror rule).
19. Skills-first ordering for artifacts (megatronlm AGENTS.md:10-19) if Corenius adopts a skill catalog.
20. Refuse to proceed on duplicate/trivial work and say what is missing (verl copy :34-36).

### 3.4 Rules repos assume but never state

[INFERENCE throughout; each is a behavior the text depends on without saying it.]

1. Host identity and hardware: `n116-077-207` and 8xB200 are assumed (milano/AGENTS.md:9; robust:104; miles_diffusion/AGENTS.md:27-33 "Only /data02 is user-writable"). No repo says what an agent should do when running elsewhere except milano's hostname check.
2. The user/owner: branch prefix `phi9t/`, "the user" and "the human" are never defined; multi-human is unaddressed.
3. Who counts as "explicit authorization": one message in-session vs standing policy. Ultron says "for the specific action"; cutlass says "unless the user explicitly asks". No repo says whether a prior message covers later pushes.
4. Skill/tool availability: `ask-matt`, Matt and Superpowers skills, `roborev`, Kata. Ultron has a fallback clause for a missing skill (skill-orchestration.md:21-25) but assumes `roborev review --branch` exists (Ultron :99) while roborev's own guide forbids invoking `roborev review` unasked (roborev/AGENTS.md:240). Two rules assume opposite defaults for the same command.
5. That `.scratch/` and `.superpowers/` are gitignored and per-checkout; nothing says how they are shared across worktrees (SDD ledgers in one checkout are invisible to a sibling worktree). Torchetta says `.scratch` is gitignored (CLAUDE.md:84-85); Ultron mandates cross-repo work "starts in Ultron at `.scratch/<effort>/`" (:33).
6. How agents identify themselves: milano requires a `Co-Authored-By` trailer without naming the format or the model string.
7. A secrets baseline for GPU/LLM repos: HF_TOKEN, ANTHROPIC_API_KEY, DAYTONA_API_KEY appear in harbor (:328-331), sglang docs (:55), flashinfer, with no rule on where they may be stored or logged, in contrast to bytedcli.
8. Cost/budget of GPU time: only milano (leases), ds4 (instance lock), monarch (8-GPU only), modded_nanogpt rules in torchtitan (:299-305) address it; Ultron's "verification has its own budget" (:122) has no numeric unit.
9. Language and tone: arnold alone specifies response language; Constitution :39-44 governs style otherwise.
10. Precedence between a repo's local guidance and a stale synced Ultron copy: each says "more specific repository guidance take precedence", but the fleet copies have five different versions (section 0).
11. What "verified" means when tests cannot run (no GPU, no rootfs): milano and torchetta distinguish levels; flash_attn, flashinfer, harbor do not.
12. Timeout/wait policy for locked resources beyond milano's `MILANO_GPU_LEASE_WAIT` (:225-227).
13. That commits are local-only until pushed is assumed in cutlass/miles ("Land ... local main only") but not for other repos.
14. That the reader can read Chinese: arnold and bytedcli are Chinese-only, with English elsewhere.

### 3.5 Thin or absent guidance

- Nearly nothing agent-specific: nccl (Ultron block + skills stub only), vaso root (block + collaboration pointer; the real rules are in linked docs), ultron root (block + scope note), cutlass beyond Git Workflow and the rootfs guard, flash_attn (architecture doc, no process), flashinfer (architecture + style, no process), harbor (style + test scope; no git/escalation/evidence rules, contributes to CONTRIBUTING.md instead), monarch_tech_review (Zephyr execution + commit style only), ui-ux-pro-max-skill (one Git Workflow paragraph), sglang/docs (writing standards), ferric_continuum (planned agent roadmap; the working rules are the Ultron block and Bazel commands), soma (rich on Insula, silent on git/merge/secrets/review).
- Unreviewed depth: arnold (policy lives in unread `docs/agent-instructions/**`), bytedcli per-domain files (~130 unread).
- No guidance file at all in the scoped list: none found missing entirely, but several repos in the workspace outside the list were not surveyed (grk, jj, kube, lancelot, linux_kernel, myriatron, ndtl, skaffen, surflo, waystone, rto-film, torchetta-release-builds, corenius-aime-hardening).
- Fleet coverage gap: Ultron fleet manifest has 9 repos; ds4 is off-manifest; milano, lagrange, torchetta, soma, robust, stratum, distributed_timeline, devx, corenius have their own or no fleet workflow; only milano+torchetta+lagrange have serious hermetic-verification doctrine, so "Insula" practice is not part of the fleet policy. [INFERENCE: the Ultron workflow is silent on hermetic execution, GPU leases, evidence receipts, and claim-scope, which are the strongest rules in the GPU repos.]

## 4. Suggested reading order for Corenius

1. Constitution v2026.08.11 (one page, already common).
2. ultron/docs/agents/agentic-engineering.md and skill-orchestration.md (fleet workflow).
3. ds4/docs/agents/evidence-bound-method.md and task-runs.md (evidence model; unmerged into the fleet copy).
4. vaso/docs/agents/cross-agent-collaboration.md (multi-agent landing).
5. milano/AGENTS.md (hermetic execution, leases, scm integrate).
6. roborev/AGENTS.md + docs/plans/2026-08-agentic-engineering-law.md (the "why" and the git prohibitions; note the internal contradiction).
7. stratum-observatory/AGENTS.md (evidence-integrity rules).

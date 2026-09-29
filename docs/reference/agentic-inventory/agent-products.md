# Agent products, tools and research in sibling repos

Date: 2026-09-29. Scope: read-only survey of `/data02/home/philip.yang/workspace/*`.

Conventions:
- Paths are relative to `/data02/home/philip.yang/workspace/` unless absolute.
- "(inference)" marks a judgment that goes beyond what the cited file states.
- Depth varies by item. Each entry says what was read. Large source trees (roborev `internal/prompt`, harbor `agents/installed/*`, jj `lib/`, bytedcli domains, grk `source/`) were sampled, not audited.
- Nothing was executed. Claims about behavior come from docs and source headers, not from running the tools.

Per-item format: problem, key design decisions, what it shows about agent workflows, reusable for Corenius, hazards.

---

## 1. Baseline: Corenius (three checkouts)

Checkouts:
- `corenius` (main, `a8e7e30`, dirty).
- `corenius-aime-hardening` (branch `aime-harness-hardening`, clean, same commit).
- `corenius-collab-spike` (branch `claude/collab-spike`, 3 commits on the same base).

**Problem.** A standalone Rust terminal agent (`corenius/README.md`, about 5.3k lines in `src/`). The spike's `docs/practice/README.md` also designates it the home for agentic-engineering practice.

**Design.**
- Turn-scoped runtime: `src/core/{types,runner,policy,transcript}.rs`. `ProviderSession` is a trait with `run_turn` and `stream_turn`. Events are `AssistantDelta | ToolResult | Done`.
- Providers:
  - `src/provider/codex.rs` is the direct Responses transport.
  - `src/provider/acp.rs` speaks JSON-RPC to `traecli acp serve`. It is prompt-only, one process per prompt, with no Corenius-side tool loop (inference from `run_acp_prompt_inner`).
- `PolicyGate` (`src/core/policy.rs`, 47 lines) auto-allows read-only tools and returns `Ask` for writes and shell. It has no scopes, no remembered decisions and no audit log.
- `src/insula.rs`: bubblewrap rootfs sandbox with a digest-enforced selection file.
- Also present: TUI (`src/tui.rs`), supervisor (`src/supervisor.rs`), `docs/observability/durable-execution.md` (design only).

**AIME harness** (`src/eval.rs`, 1934 lines; `src/cli.rs`; `evals/aime_sanity.jsonl`).
- Commands: `eval aime`, `aime-sanity`, `aime-replay`, `aime-regrade`.
- Data: GAIR AIME 2024/2025-I/2025-II (60 problems), fetched into gitignored `.scratch/aime-benchmark/data/` and count-validated. Tracked sanity set has 6 lines (5 pass cases plus a skipped `timeout-probe`).
- Prompt forbids tools and skills and requires a final `FINAL: <integer>`. The scorer takes the last `FINAL:` line and its first integer of at most 3 digits. There is no last-number fallback. Score is binary.
- Failure kinds: `TimeoutNoResponse`, `StallNoProgress`, `ProviderError`, `NoFinal`, `WrongFinal`, `PromptContamination`.
- Artifacts per run: `samples.jsonl` (appended per sample), `manifest.json` (argv, git commit and `git status --short`, dataset sha256, model, timeout, mode, provider profile), `summary.json`, `run_metrics.json`, `triage.md`.
- Offline regrade writes independent artifacts plus a scorer-delta report.
- Uncommitted change in `corenius` (7 files, about +686/-74): `AcpSessionProfile` disables 8 features and 20 tools, sends empty skills/MCP in `session/new`, pins reasoning effort, adds idle-stall detection with retries, raises the default timeout from 300 s to 600 s.
- Scripted fake ACP server: `tests/support/scripted_acp.rs`. Steps are `Success`, `ProviderError`, `Stall`, `PartialThenStall`, `MalformedLine`. It validates argv and prompt shape, and has `assert_consumed`.
- Lineage: DeepSeek Harness (fresh child processes, scripted provider, replay) and DS4 `ds4-eval` regrade (from the AIME spec under `docs/superpowers/`).

**SDD workflow artifacts.**
- Live ledger: `corenius-aime-hardening/.superpowers/sdd/2026-09-25-*/`: `progress.md` (preflight producer/consumer table with "Ruling: ... cost if wrong", per-task reviewer verdicts, "fix round N/5" counters with commit ranges), `task-N-brief.md`, `task-N-report.md`, `review-<sha>..<sha>.diff`.
- Ends with verification on `a8e7e30`: `cargo fmt --check && cargo test && cargo clippy -D warnings`.
- Older ledger `corenius/.superpowers/sdd/corenius-agentic-execution-core-ink-tui/` describes a TypeScript/Ink codebase that no longer exists. It is stale.
- Commit history ("chore: untrack SDD Task 5 report") shows reports are committed, then untracked.

**Spike additions** (`corenius-collab-spike`): `CONSTITUTION.md`, `docs/agents/*` (workflow, skill-orchestration crosswalk, `agentic-workflow-repos.json`), `docs/practice/imported/` (verbatim milano `scm.py`, vaso `relay.py`, hooks), `docs/reference/*`, `docs/spikes/collab/design-space.md`, `experiments/collab/` (discrete-event simulator of 7 merge strategies, 980 runs, 89 tests), `tools/sync_agentic_workflow.py`.

**Shows.** Corenius already does manifests with git state and hashes, offline regrade, fixtures that reject protocol mismatch, and rulings that state the cost of being wrong.

**Gaps against a mature product.**
- Runtime: no ACP tool loop, no session resume, no durable replay, coarse policy, no token/cost budgeting, single eval task type, 5-problem sanity set, single-shot samples with no variance.
- Process: none of the three checkouts has `AGENTS.md`, `CLAUDE.md` or `.claude`, although the spike's `CONSTITUTION.md` section 7 says durable guidance belongs in `AGENTS.md`. There is no tracked evidence ledger and no baseline or regression thresholds.

**Hazards.**
1. Evidence is ephemeral: `.superpowers/`, `.scratch/`, `.corenius/` and `target/` are gitignored.
2. Dirty-tree provenance: the isolated-check manifests record commit `a8e7e30`, but the code that ran was uncommitted (`git_status_short` records file names, not content).
3. Split state: the isolation change exists only in the `corenius` working tree, and the SDD reports only in `corenius-aime-hardening`. Do not reset either.
4. Scores are not comparable across the isolation and timeout change. The 56/60 and 53/60 runs used old settings. Regrade hardcodes `AimeProviderProfile::default()`.
5. `classify_failure` matches error text ("stalled", "timed out"). `AcpFailureKind` exists but is not carried into samples.
6. `SharedProvider` mode is not really shared (new process per prompt). Regrade reports `timeout_ms=0`. `schema_version` stays 1 while fields are added.
7. README examples hardcode `/data02/home/philip.yang/...`.
8. The spike is a divergent fork, and its `docs/practice/imported/` files are verbatim snapshots that go stale.

---

## 2. roborev (`roborev/`)

Read: `README.md`, `AGENTS.md`, `CLAUDE.md`, `docs/plans/2026-08-agentic-engineering-law.md`, `docs/agent-hook.md`, `docs/advanced/*`, and key files under `internal/`, `cmd/roborev/`. The `go.kenn.io/kit` dependency (worktree, daemon runtime, agenthook profiles) is not in this repo and was not read.

**Problem.** Local daemon plus CLI. A post-commit hook enqueues a background AI review of each commit, so agent-written code is verified while context is fresh. Guiding law: "scale agent execution, not agent authority". The human owns the merge, and background work never mutates the tracked working tree (`AGENTS.md`).

**Architecture.**
- CLI, then HTTP API on 127.0.0.1:7373, then daemon, then worker pool (default 4), then agent adapters. Files: `internal/daemon/server.go`, `worker.go` (retry, failover, per-agent quota cooldown), `outputbuffer.go`, `broadcaster.go`.
- Storage: SQLite in WAL mode with inline `ALTER TABLE` migrations (`internal/storage/db.go`). Optional PostgreSQL sync by UUID (`sync.go`, `docs/advanced/postgres-sync.md`). Tables `repos`, `commits`, `review_jobs` (about 48 columns) and `reviews`.
- Queue is a SQLite table. `ClaimJob` (`internal/storage/jobs.go`) is an atomic `UPDATE ... WHERE id=(SELECT ... LIMIT 1)`. It honours `retry_not_before` and `claim_blocked`, and refuses to claim while a pause or drain flag is set.
- States: queued, running, done, failed, canceled, applied, rebased, skipped (`internal/storage/models.go`).
- Updates are fenced on `status='running' AND worker_id=?`, so a zombie worker cannot overwrite a reclaimed job.
- Verdict: `ParseVerdict` (`internal/storage/verdict.go`) is deterministic and stored as `verdict_bool`. The code comment says to fix the prompt, not add heuristics.

**Git hooks.**
- `internal/githook/githook.go` generates a versioned POSIX `post-commit` and `post-rewrite` hook (marker "post-commit hook v4", driving `NeedsUpgrade`). It falls back to `command -v` and exits 0 if the binary is missing. An embeddable function-wrapped variant is inserted into an existing hook, and uninstall removes only roborev lines.
- `cmd/roborev/postcommit.go` never fails a commit. It logs JSONL to `post-commit.log`, skips during a rebase, and normalizes relative `core.hooksPath`. The timeout comes from `config.ResolveHookTimeout` (no git spawn).
- `post-rewrite` runs `remap` (`cmd/roborev/remap.go`), moving reviews from old SHAs to new.
- Self-healing: the daemon repairs registered repos' hooks at startup and skips ephemeral binaries (`internal/daemon/hookrepair.go`).
- Second layer: `agent-hook` (`internal/agenthook/`, `docs/agent-hook.md`) installs into Claude, Codex, Copilot, Cursor and other agents' hook systems, and reminds the agent to fix open failed reviews.

**Backends** (`internal/agent/`). `Agent` interface with `Review`, `WithReasoning`, `WithAgentic`, `WithModel`. Optional `SessionAgent` (resume) and `SynthesisAgent`. Backends: codex, claude-code, gemini, copilot, opencode, cursor, kiro, kilo, droid, pi, grok, `test`, and ACP (`acp_agent.go`). Discovery is a PATH cascade with backup agent and model on failover.
- Review mode: Claude uses `--allowedTools Read,Glob,Grep`. Codex uses `--sandbox read-only`, with a switch for hosts where bwrap is broken.
- Agentic mode adds Edit, Write and Bash and passes `--dangerously-skip-permissions` (`claude.go`).
- The commit classifier runs Claude with `--tools ""`, a JSON schema and never the dangerous flag (`classifyArgs`).

**Loops.**
- `roborev fix`: one foreground pass, commit, close review.
- `roborev refine` (`cmd/roborev/refine.go`): oldest failed unclosed review, temp worktree, agent run (1 h timeout), submodule-tamper check, patch capture, apply, commit, wait for re-review. Default `--max-iterations 10`. Then a branch-level review. Needs a clean tree and a feature branch.
- Background fix jobs run in a worktree at the reviewed commit. The patch is stored in the DB and applied later by a human (`docs/advanced/background-tasks.md`).
- Panels: named reviewer subagents fan out and synthesize (`docs/advanced/subagent-review-panels.md`, `internal/daemon/panel_*.go`, `internal/review/synthesis.go`).
- `compact` re-verifies open findings against current code (`cmd/roborev/compact.go`).

**Evidence.**
- Each job stores prompt, diff, command_line, session_id, model, patch and token usage. Review output is stored separately with `closed` and `verdict_bool`. Cost tracking in `internal/storage/cost.go`.
- CI rule: PR review config comes from the trusted default branch, not the PR head (`internal/prompt/prompt.go`, `internal/daemon/ci_worktree.go`). Kata integration is disabled for CI PR reviews (`docs/integrations/kata.md`).
- The plan doc admits gaps: no verification budget, no authority-boundary policy object, refine is "trusted code only", no harness-learning loop.
- Test isolation: `internal/testenv/testenv.go` (`RunIsolatedMain`) redirects `ROBOREV_DATA_DIR`, git config and XDG. `ProdLogBarrier` fails if production state is touched. `git_isolation_guard_test.go` uses the AST to fail any test package that uses git without isolation.

**Reusable for Corenius** (inference unless noted).
1. Always-exit-0, silent-failure hook that posts one JSON enqueue request with a short timeout. Copy the versioned marker and self-repair.
2. SQLite queue with atomic claim, `retry_not_before`, pause and drain, worker-id fencing (rusqlite, WAL).
3. Agent trait with capability builders and read-only-by-default tool lists. Separate zero-tool mode for untrusted input.
4. Worktree per agent, patch captured as data, applied by a human.
5. Stored boolean verdict plus deterministic parser. Bounded loop (max iterations, then branch-level review, then escalate).
6. A `test` agent for hermetic tests, and the isolation guard.
7. Hash-chain the per-job records. roborev has no tamper evidence.

**Hazards.**
- Agentic mode depends on `--dangerously-skip-permissions`. roborev has no sandbox of its own, and Codex falls back to `--full-auto` where bwrap fails.
- Prompts include diffs and commit text from possibly hostile authors. Mitigations (XML escaping, trusted-branch config, zero-tool classifier) are not hard guarantees.
- Heuristic verdict parsing can flip on chatty output.
- Absolute binary path in the hook. Version managers and ephemeral builds break it.
- Large schema with inline migrations and a PostgreSQL mirror. `AGENTS.md` forbids `make install` because a dev daemon can migrate the live `~/.roborev/reviews.db`.

---

## 3. harbor (`harbor/`)

Read: `README.md`, `AGENTS.md`/`CLAUDE.md`, `rfcs/0001-trajectory-format.md`, `skills/create-task`, `skills/rewardkit`, `adapters/aime/`, and core files in `src/harbor` (agents, environments, verifier, trial, job). Nothing was run. `agents/installed/*` (for example `claude_code.py`, about 2k lines) was not read in depth.

**Problem.** Python 3.12+ framework, from the Terminal-Bench authors, for evaluating agents and models in container sandboxes, also for RL rollouts. One CLI runs a dataset x agent x model matrix in parallel.

**Task format** (`AGENTS.md`, `skills/create-task/SKILL.md`, `src/harbor/models/task/config.py`). A directory with `task.toml` (schema 1.4), `instruction.md`, `environment/Dockerfile` (or compose), `tests/test.sh`, optional `solution/solve.sh`.
- `task.toml` sections: `[task]`, `[metadata]`, `[verifier]` (timeout, env, user, network), `[agent]`, `[environment]` (cpus, memory, gpus, image, healthcheck, skills_dir, mcp_servers), `[solution.env]`.
- Multi-step tasks under `steps/<name>/` with reward aggregation.
- Fixed container paths (`src/harbor/models/trial/paths.py`): `/logs/agent`, `/logs/verifier`, `/logs/artifacts` mounted from the host trial dir. `/tests` uploaded after the agent finishes. `/solution` copied only by the oracle agent.

**AIME adapter** (`adapters/aime/src/aime/task-template/`, `adapters/aime/aime.yaml`). 60 problems from GAIR-NLP/AIME-Preview. Instruction: write an integer in 0 to 999 to `/app/answer.txt`. `tests/test_outputs.py` bakes in `EXPECTED_ANSWER` and does an exact string match after `.strip()`. `test.sh` writes `1` or `0` to `/logs/verifier/reward.txt`. 1 CPU, 2 GB, 3000 s agent timeout.

**Environments** (`src/harbor/environments/`). `BaseEnvironment` ABC: `start`, `stop`, `exec`, upload and download, capabilities, network-policy validation, `preflight()`. Lazy imports via `factory.py`. Providers: docker (default, with egress-control sidecar), podman, daytona, e2b, modal, runloop, gke, ack, openshift, ec2, apple-container, singularity, kata, ssh, and others. Network policy per phase: `public`, `no-network`, allowlist. Optional separate verifier environment (`skills/rewardkit/SKILL.md`).

**Agents** (`src/harbor/agents/`).
- `BaseAgent`: `name`, `version`, `setup(env)`, `run(instruction, env, context)`. The agent fills `AgentContext` (tokens, cost, rollout details).
- `BaseInstalledAgent` (`agents/installed/base.py`): `install(env)`, then runs a CLI inside the container. Has secret redaction, API-error classification (`ApiRateLimitError`, `ContextWindowExceededError`), and `convert_trajectory` to ATIF. About 40 ship (claude_code, codex, aider, openhands, mini_swe_agent, goose, gemini_cli, opencode).
- Custom: `--agent module:Class` (`examples/agents/marker_agent.py`). `acp:<id>@ver` runs any ACP registry agent (`docs/content/docs/agents/acp.mdx`).
- Internal: terminus-2 (tmux, JSON/XML parsing). Utility: `oracle` (runs `solve.sh`) and `nop`.
- ATIF v1.8 (`rfcs/0001-trajectory-format.md`, `src/harbor/models/trajectories/`): root fields `schema_version`, `session_id`, `agent`, `steps`, `final_metrics`, `subagent_trajectories`. Steps carry `source`, `message`, `reasoning_content`, `tool_calls`, `observation.results[]`, `metrics`.

**Verifier** (`src/harbor/verifier/verifier.py`). Uploads `tests/`, runs `test.sh`, reads `reward.json` (finite floats) else `reward.txt`. Typed errors: `RewardFileNotFoundError`, `RewardFileEmptyError`, `VerifierOutputParseError`. NaN and Inf rejected. `packages/rewardkit` is a standalone verifier with criterion files and TOML LLM or agent judges. pass@k in `utils/pass_at_k.py` (single-key 0/1 rewards only).

**Trials and jobs** (`trial/trial.py`, `trial/queue.py`, `job.py`). Steps: build and start environment, healthcheck, agent setup, agent run under timeout, collect artifacts, verify, stop. Events emitted at each stage. Exceptions land in `result.exception_info`. Each trial directory holds `config.json`, `lock.json`, `results.json`, `trial.log`, `agent/`, `verifier/`, `artifacts/`. `TrialQueue` bounds concurrency with per-agent pools and exponential-backoff retry, excluding non-retryable errors (`AgentTimeoutError`, `ApiUsageLimitError`, reward-file errors). Resume: requires an identical `config.json`, otherwise `FileExistsError`. It deletes trial dirs without `result.json` and skips completed ones.

**Registry.** `registry.json` pins datasets by git URL, commit and path. 70+ adapters (list in `AGENTS.md`), parity notes in `adapters/parity_summary.csv`.

**Reusable for Corenius** (inference).
- Run Corenius as an installed agent: `install()` uploads the binary, `run()` execs it with the model and key from `extra_env` (pattern in `agents/installed/aider.py`). Reuse the AIME adapter as is.
- Emit ATIF v1.x JSON to `/logs/agent/trajectory.json` (Rust structs mirroring `models/trajectories/*.py`, validated with `utils/trajectory_validator.py`).
- Copy the reward contract, the trial directory layout, typed error names, resume-by-config-equality, and the `oracle`/`nop` self-test agents.
- The minimum viable Rust-native harness: task loader, docker exec wrapper, `test.sh` runner with reward parsing.

**Hazards.**
- AIME `test.sh` always exits 0. A missing or empty answer scores 0 exactly like a wrong answer. Missing reward file is non-retryable.
- Exact match with no normalization: `042` and `42.0` fail.
- Agent and verifier both have `network_mode=public` in the AIME template, so a contamination risk exists for a public benchmark (inference).
- The verifier runs in the same container by default, so the agent could tamper before grading. `[verifier] environment_mode = "separate"` avoids this at the cost of a task change.
- `aime.yaml` sets `force_build: true` (rebuild every trial). The base image is external (ghcr.io).
- `AGENTS.md` lists an `orchestrators/` directory that does not exist in `src/harbor/`. Docs are partly stale.
- Any config change breaks resume. Partial trial directories are deleted.
- `AGENTS.md` conventions (TaskGroup, ty, Ruff) are Python-specific.

---

## 4. bytedcli (`bytedcli/`)

Read: `README.md`, `AGENTS.md` (`CLAUDE.md` is a symlink), `CONTEXT.md`, `docs/cli-analysis/AGENT_FRIENDLY_GUIDELINES.md`, `docs/CONTRIBUTOR_PATTERNS.md`, `.agents/skills/*`, `.agents/commands/develop-to-merge.md`, `dev-cli/src/tasks/*`, `plugins/`, `schemas/`. Not read: individual domain implementations.

**Problem.** One CLI for ByteDance internal R&D platforms (about 250 domains) with stable `--json` output, plus about 260 agent skills that route tasks to commands.

**Agent-friendly CLI principles.**
- Any rule that makes the agent guess or retry is a defect.
- Command shape `domain [resource] [subresource] action`, at most 4 levels, kebab-case, verb last. Closed verb set (`get`, `list`, `search`, `query`, `create`, `update`, `delete`, `deploy`, `execute`). Synonyms banned; old names kept as hidden aliases (`docs/cli-analysis/HIDDEN_ALIASES.md`).
- Flag-first, no positionals, global flag dictionary, numbers parsed at option definition.
- `requiredOption` banned. Use `softRequiredOption` plus handler validation so every error is one JSON envelope.
- Errors go through `AppError` as `{status, error:{message, code, hint}}`. `hint` must be a copy-pasteable fix and tests assert it is non-empty. Silent returns are forbidden.
- Output only through `src/utils/output.ts`. Text default, `--json` global. Truncation is never silent (`truncated`, `has_more`, `page`, `page_size`). Secrets redacted in text mode.
- Layering: api, service, handler, command. Zod parsing.
- Progressive help layers `@USAGE`, `@HELP`, `@MAN` (`.agents/skills/agent-friendly-review/references/cli-interface-guide.md`).
- Safety: writes default to dry-run and need `--yes`. `self safety-policy` yields `allow`, `deny` or `confirm`. `confirm` returns `SAFETY_POLICY_CONFIRMATION_REQUIRED` to the agent, which must get user approval and retry. `deny` cannot be bypassed (`skills/bytedcli/references/safety-policy.md`).
- Mechanical enforcement: `dev-cli/src/tasks/validate/agent-friendly.ts` at pre-commit. The MCP server (`src/mcp/`) reuses the command tree.

**Skill generation.**
- Each domain has `skills/bytedance-<domain>/SKILL.md` plus `references/`. Router skill `skills/bytedcli/SKILL.md` with `references/command-surface.md` and `subskills-index.md`. A filtered variant `skills/doubao-bytedcli`.
- `dev-cli/src/tasks/sync/subskills.ts` mirrors sibling skills into `skills/bytedcli/references/subskills/<skill>/GUIDE.md` during `build:assets`. Idempotent. Hand-editing a mirror is a blocking review finding. `shared-refs.ts` copies shared `invocation.md` and `troubleshooting.md` into every skill.
- `validate/skills.ts`: name equals directory (1 to 64 characters), description at most 1024, banned and retired skill list.
- Distribution: `release/publish-skills.ts`; `self skill list|install|update|remove|get` with `-g` for `~/.agents/skills`; lock file `~/.local/share/bytedcli/data/skill-lock.json`.
- Plugins: `plugins/registry.json` (trusted list) and `schemas/bytedcli-plugin.schema.json` (command paths, `overrideBuiltIn`, credentials, capabilities; forbids `..` and absolute paths; repos must be on code.byted.org).
- Dev skills in `.agents/skills/*`, symlinked from `.claude/skills/*`: agent-friendly-review, mr-reviewer, mr-review-loop, fix-ci, api-to-cli, domain-scaffolder, skill-creator. `skills-lock.json` pins skill-creator by hash.
- `agent-friendly-review` has three layers (API naming and return types; code structure with nesting at most 3, functions at most 20 lines; CLI help docs) and ends in a graded report (`.agents/skills/agent-friendly-review/SKILL.md`).

**Workflows.**
- `/develop-to-merge` (`.agents/commands/develop-to-merge.md`): ten steps (goal, fresh worktree from `origin/master`, install, smallest change, local validate, local `mr-reviewer` gate, commit and push, create MR, `mr-review-loop`, remove worktree). Each step has a completion criterion. Forbids self-enqueueing to the merge queue and destructive git.
- `mr-review-loop` (`.agents/skills/mr-review-loop/SKILL.md`, `constants.yaml`, `templates/`, `scripts/`):
  - One Lark root card per MR, all later messages in that thread. The root message id is persisted in the MR body under `## Agent Review Thread`, so another session can resume.
  - Disposition Ledger: every finding at every severity gets exactly one evidence-backed disposition, Fix, Rebut or Defer.
  - Polls every 5 minutes and classifies: new finding, waiting, clean for bot handoff, bot queued, bot unresponsive, terminal.
  - Only the bot or a human enqueues the merge. A launchd reminder follows up at 7, 14 and 30 days.
- `fix-ci` must read the CI step log and grep `not ok`, never infer from a local run.

**Reusable for Corenius** (inference).
- Skill validation gate; single source of truth with generated mirrors and a "do not edit mirrors" rule; router skill with progressive loading.
- `skill install` with global and project scope and a lock file; plugin manifest with trust registry.
- Agent-friendliness lint at pre-commit; `hint` in every error; `confirm`/`--yes` policy with machine-readable code.
- Disposition ledger; state persisted in an external artifact so loops resume; no self-merge; explicit terminal states.

**Hazards.**
- `AGENTS.md` is about 46 KB with per-domain trivia, which bloats agent context. The mr-reviewer skill itself blocks domain content in the root file.
- Tightly coupled to internal Lark and Codebase tooling, with hard-coded chat and bot ids. Hidden dependency on a local `lark-cli`.
- The reminder job is macOS launchd only, and a bot outage blocks the loop.
- Mirror drift if `build:assets` is skipped.

---

## 5. ui-ux-pro-max-skill (`ui-ux-pro-max-skill/`)

Read: `CLAUDE.md`, `skill.json`, `src/ui-ux-pro-max/templates/*`, `cli/src/utils/*`, `cli/scripts/sync-assets.mjs`, `.releaserc.json`.

**Problem.** A design-knowledge skill: CSV databases of styles, palettes, typography, charts, UX rules and 22 framework stacks, searched by a dependency-free Python script and installed into 20 AI assistants.

**Design.**
- Source of truth `src/ui-ux-pro-max/{data,scripts,templates}`. One shared `templates/base/skill-content.md` (with `{{TITLE}}`, `{{SCRIPT_PATH}}` and similar placeholders) plus per-platform `templates/platforms/*.json` (root, skillPath, filename, frontmatter, Skill-vs-Workflow). Paths differ per platform (for example `.github/prompts/*.prompt.md` for Copilot, `.kiro/steering/` for Kiro).
- Installer (`cli/src/utils/template.ts`): `resolveInstallPaths` is shared by the real install and `--dry-run`, so they cannot disagree. Existing files are skipped unless `--force`. Detection by dot-directory (`cli/src/utils/detect.ts`), avoiding a blanket install for the shared `.agents`.
- `cli/scripts/sync-assets.mjs --check` compares SHA-256 hashes after CRLF-to-LF normalization as a CI gate. No symlinks, because Windows checks them out as text files.
- Query contract in the skill: smallest mode, one intent, 2 to 5 terms, verify the returned domain, retry once on empty, never persist unverified output. Zero results are labeled as no match. The skill says dataset text is data, never instructions, and tells the agent to ask the user rather than install Python.
- `npm run verify:data` chains CSV and semantic validation, `scripts/validate-agent-guide.py` (executes the commands quoted in the guide), and relevance evaluation with calibration and held-out splits.
- Release: semantic-release with npm package `uipro` (`.releaserc.json`, `.github/workflows/release.yml`). Claude plugin manifest in `.claude-plugin/`.
- No review or merge workflow beyond branch, PR via `gh`, and CI.

**Reusable for Corenius** (inference). One JSON per target agent plus a shared renderer; `--dry-run` that shares code with the real install; hash-based asset drift gate; guide validator that runs its own commands; data-is-not-instructions stance; held-out eval for retrieval.

**Hazards.** Data and scripts are duplicated three ways (drift caught only by CI). `.claude/skills/ui-ux-pro-max/SKILL.md` is hand-authored while other platforms are templated. Counts such as "79 styles" are duplicated across about 20 JSON files. A legacy `init --legacy` path hits GitHub releases (rate limits).

---

## 6. jj (`jj/`), agent-relevant subset only

Read: `docs/glossary.md`, `docs/operation-log.md`, `docs/conflicts.md`, `docs/technical/conflicts.md`, `docs/technical/concurrency.md`, `docs/working-copy.md`, and headers of `lib/src/{repo,rewrite,conflicts,operation,op_store,op_heads_store,transaction,workspace,local_working_copy}.rs`. Not read: `docs/design/jj-converge-command.md`, `docs/design/run.md`.

**Problem.** History editing, concurrent tooling and recovery without lock files or manual bookkeeping. This is the substrate a multi-agent VCS needs.

**Key decisions.**
- **Change ids** are stable across rewrites, unlike commit ids. Divergent changes (one change id, several visible commits) are flagged, with `xyz/0`-style offsets (`docs/glossary.md`; `lib/src/evolution.rs`).
- **Automatic descendant rebase**: `MutableRepo::rebase_descendants` runs in the same transaction (`lib/src/repo.rs`, `lib/src/rewrite.rs`).
- **First-class conflicts** are stored in the tree as merge objects. Rebase always succeeds, and resolution is deferred (`docs/conflicts.md`, `lib/src/conflicts.rs`, `merged_tree.rs`).
- **Operation log**: every command is an immutable content-addressed operation holding a view (heads, bookmarks, per-workspace working-copy commit). `jj undo`, `jj op revert`, `jj op restore` roll back the whole repo. `--at-op` gives read-only time travel without snapshotting (`docs/operation-log.md`, `lib/src/operation.rs`, `op_store.rs`).
- **Lock-free concurrency** (`docs/technical/concurrency.md`): concurrent writers create divergent operation heads. The next load three-way-merges views, and conflicted bookmarks are recorded as data (`lib/src/op_heads_store.rs`, `Transaction::merge_operation`). A narrow lock fallback exists in `lib/src/lock/`.
- **Workspaces**: several working copies share one repo, each with its own `@` and a recorded last-synced operation. Staleness is detected and `jj workspace update-stale` repairs it (`docs/working-copy.md`, `lib/src/workspace.rs`). Snapshotting the working copy is itself an operation.

**Shows.** Per-agent isolated workspaces, an auditable operation history, undo for any mistake, and conflicts and divergence recorded as data rather than errors. (inference) The operation log is close to an audit trail per agent action.

**Reusable for Corenius** (inference).
- Record the operation id before and after each agent turn so a turn can be reverted with `op restore`. Give reviewers `--at-op` snapshots.
- One jj workspace per sub-agent.
- Use the stable change id as the handle for a unit of agent work across rebases.
- Copy the immutable-operation plus merge-on-read pattern for Corenius's own session store.
- Surface conflicted commits to a resolver agent instead of aborting.

**Hazards.**
- Automatic snapshotting can silently commit secrets or stray files, so ignore rules must be set.
- Concurrent edits to one change yield divergent changes and bookmark conflicts, and agents must handle both.
- A stale working copy is a real state.
- The atomicity guarantee assumes ordered filesystem writes (`docs/operation-log.md`).
- An agent may push conflicted commits.
- This conflicts with the Milano rule that mainline is git-linear with rebase-plus-fast-forward (`milano/AGENTS.md`). Adopting jj means deciding how it co-exists with that policy (inference).

---

## 7. devx (`devx/`)

Read: `README.md`, `doom/ai-completion.el`, `doom/packages.el`, `docs/superpowers/specs/2026-09-14-ai-lsp-certification-design.md` and `plans/`, `bin/verify-ai-lsp`. Not read: the roughly 2,000 lines under `test/` in full, except by summary.

**Problem.** Doom Emacs inline-AI setup (Copilot default, Tabbymacs manual, `lsp-mode`), paired with a verifier that proves it works against real language servers for Rust, Python, C++ and CUDA.

**Key decisions.**
- Fail-closed certification: pass needs positive evidence plus negative properties. Failures are aggregated across languages into one report.
- Private-API isolation: only `test/ai-lsp-adapter.el` may call `copilot--*` or `lsp--*`. A missing symbol is reported as an incompatible dependency, never as a pass or a language failure.
- Independent expectations: a separate literal mode-to-language-id table so one bad shared value cannot make production and tests wrong together.
- Owned process lifecycle in `bin/verify-ai-lsp`: `setsid` with a nonce ownership marker, TERM then KILL deadlines, timeout floor computed from sub-budgets (360 s), and it kills only the captured, validated process group.
- Credential-safe diagnostics: stream-filter redaction before disk, modes 0700/0600, line-boundary truncation, tests with fake credentials placed across truncation boundaries.
- Certificate tied to pinned versions and hashes. Live Copilot output is declared non-deterministic.

**Shows.** Eval and review discipline applied to an AI tool. It uses real dependencies, checks failure modes, and produces a machine-readable certificate.

**Reusable for Corenius** (inference). Process-group ownership with a nonce and bounded TERM/KILL; timeout floor derived from sub-budgets; redaction before persistence in the transcript writer; a compatibility adapter around provider quirks; independent expected values in eval tests; aggregate-all-failures reporting.

**Hazards.** The spec names `bin/certify-ai-lsp` and `certificates/`, but `bin/` shows only `verify-ai-lsp`, so the spec may run ahead of the implementation (not verified). Absolute `~/workspace/devx` paths in loaders. Depends on live Copilot network and auth, and on private Copilot and lsp-mode APIs.

---

## 8. monarch (`monarch/`)

Read: `docs/agentic-ai/*` (notably `glm52-local-serving.md`, `glm53-flash-deep-dive.md`), `ginkgo/README.md`, `ginkgo/docs/{verification-ladder,evidence-boundary}.md`, `ginkgo/eval/{manifest,orchestrator}.py`, result-directory manifests, `AGENTS.md`, `CLAUDE.md`, `CONSTITUTION.md`, `CONTEXT.md`, `docs/agents/*`. `control-plane-results/` is actor-isolation test output with little agent content.

**Problem.** Serve GLM-5.2 locally (SGLang, Dynamo, a Responses-API adapter) so it can drive Codex-style coding agents, and prove that it can. This is not an agent framework. The other half is agent-process rules for coding agents.

**Eval platform.**
- `ginkgo/README.md`: "No fallback. Fail fast, fail loud." No default ports, ambient services or alternate URLs. `scripts/run` is the only gateway into the hermetic bwrap rootfs (`AGENTS.md`).
- `ginkgo/docs/verification-ladder.md`: 13 rungs, passing a lower rung never implies a higher one.
- `ginkgo/docs/evidence-boundary.md`: readiness, blocker and completion evidence are separate, with an explicit non-evidence list (`/v1/models` without an inference probe, fixture scores, default-port probes).
- Serving verifier: a synthetic agent-loop probe with tool calls (`list_project_files`, `read_project_file`) that asserts `FINAL_SCORE: 19`. A terminal bench uses `terminal_run`, `terminal_read_file`, `terminal_write_file`. GLM-5.3 forces thinking mode, so `reasoning_content` mapping needs care (`glm53-flash-deep-dive.md`, web-sourced and time-stamped).
- `ginkgo/eval/manifest.py`: suites pin dataset and harness revisions and an execution backend (`bwrap_rootfs`, `harbor_local_docker`). Endpoints carry roles (`model_under_test`, `judge`).
- `ginkgo/eval/orchestrator.py`: file-backed resumable state machine in `evalrun-state.json`. Stages: campaign materialization, suite prep, trial generation, grading, suite summary, campaign summary. Each stage is `running` or `completed` and lists durable artifacts. Resume is refused if `manifest_sha256`, `run_id`, `mode` or `suite_ids` differ. State is set to "completed" only after artifacts exist. Writes are plain `write_text`, not atomic.
- Results (`glm52-benchmark-results/benchmark-*`): `archive-manifest.json`, `contract_artifact_sha256`, `environment.json`, `run.json`, `summary.json`. Failed attempts are kept, for example a loop-diagnosis run with `status: environment_setup_failed` classed as infrastructure with `model_failures: 0`. Directory names (promptfix, timeout, loop-diagnosis) show an iterative failure-driven loop was run (inference).

**Agent rules** (`AGENTS.md`, `CLAUDE.md`, `CONSTITUTION.md`, `docs/agents/agentic-engineering.md`, `docs/agents/skill-orchestration.md`).
- Principle: "Scale agent execution, not agent authority".
- Constitution: match the requested mode (explain, review and diagnose are read-only), never claim success without fresh evidence, stop mutating after a correction, no merge without authorization, put learning in shared docs.
- Clean-context subagents by default for non-trivial work. Gated path (11 steps): intent, design, independent critique from a context that did not inherit the design, spec and tickets, worktree per ticket, red/green proof, two-axis review (standards and spec), `roborev review --branch`, at most 3 repair iterations then escalate, human-only merge, distill into glossary and ADRs.
- Short path allowed only if every condition holds (reversible, one repo and session, no schema change). Bug path: red-capable command first, rank 3 to 5 hypotheses, escalate after 3 failed fixes.
- Skill precedence: user, repo, phase owner, supporting skill.

**Reusable for Corenius** (inference). `EvalRun` state file with per-stage status, artifact lists and manifest-hash resume guard; failure classification into infrastructure, model and environment-setup; non-evidence list; a tool-loop conformance probe per provider; per-run archive manifest with SHA-256; path classification; three-iteration repair budget; separate verifier context; pinned constitution.

**Hazards.** Non-atomic, unlocked state writes. Result directories are committed at repo root and contain host paths and PATH. The Harbor path runs on the host outside the sandbox. The Responses adapter is single-process. Rules are prose only, with nothing mechanical enforcing them, and depend on roborev, Superpowers skills and a Kata tracker. Files are "managed by Ultron" copies and can drift.

---

## 9. kube/investigations (`kube/`)

Read: the directory structure, `investigations/*/analysis/report.md`, `live_nccl_probe_summary_20260703T0738Z.md`, the collection and probe scripts, and `docs/superpowers/specs/2026-07-03-staged-live-nccl-probe-design.md` with its plan. Per-pod raw evidence was not read.

**Problem.** Forensics on a hung 32-rank training job (`aj-66f9dc22-...`, namespace `norbert`), staged and read-only against live GPU pods.

**Design.**
- Layout: `investigations/<job>-<UTC>/{cluster,pods,trainer,coordinator,analysis}/` plus `collection.log`. Follow-up runs named `-deep-traces`, `-live-failover`, `-current-state`, `-live-nccl-probe-<ts>`. Many are `.tgz`-archived.
- Hypothesis to evidence: the first report localized the hang to `huggingface_mixin.py:74` (`torch.distributed.barrier()`), seen in every sampled rank's `py-spy` stack.
- Topology ladder p1, p2, p4, p8 (world sizes 4, 8, 16, 32), stopping at the first failure. p4 failed with a local CUDA OOM on one rank, which corrected the earlier "silent fabric hang" theory.
- Safety model: never signal, attach to or ptrace live training PIDs; pick GPUs by free memory and refuse below 4 GiB; timeout on every kubectl and torchrun call; `DRY_RUN`; overrides logged to the stage manifest; kill only the probe process tree.
- Scripts: `collect_deep_worker_traces.sh`, `collect_hang_forensics.sh`, `run_staged_live_nccl_probe.sh`, `minimal_nccl_probe.py`, `run_nccl_probe_on_trainers.sh`.

**Shows.** Evidence-first investigation with a paper trail and negative results kept. No orchestration framework was found; orchestration is script-level. (inference) The many per-stage directories suggest an agent iterating experiment, artifact, revised hypothesis.

**Reusable for Corenius** (inference). Timestamped investigation bundle with raw evidence, analysis report and evidence index; staged escalation that stops at first failure; read-only-by-default with a command allowlist for live systems; pre-flight guard, dry-run, per-command timeouts, stage manifest; report template with "Bottom line" and cited evidence.

**Hazards.** `.tgz` archives duplicate directories. `/proc` environment dumps (`per_pid_forensics.txt`) may contain secrets. A "non-destructive" probe still consumes GPU memory on shared hardware and OOMed. The first report's root cause was partly overturned by later runs, so conclusions need dates and versions.

---

## 10. ferric_continuum (`ferric_continuum/.scratch/evidence-bound-agentic-engineering/`)

Read: `spec.org`, `pending-workstreams.org`, `issues/01-10`, `receipts/deepseek-v41-reconciliation.org`, `docs/agents/evidence-bound-method.md`, `docs/agents/task-runs.md`, `AGENTS.md`, `CLAUDE.md`, `CONSTITUTION.md`. `learning-records/` is empty. `lessons/0001` is the only agent-relevant lesson (separate evidence layers).

**Problem.** Migrate about 13 `.scratch` efforts into an evidence-bound model and specify the tooling that makes it enforceable. The docs say the automation does not exist yet.

**Design.**
- Model: Engineering Task, Task Contract, Task Run (one dedicated worktree), replaceable Agent Attempts. A candidate, an evidence record and a promotion are three separate things: an agent produces a candidate, verification produces evidence, a human decision promotes. Modes: delivery, remediation, scout, experiment. An experiment can end rejected or inconclusive, and a scout's recommendation does not authorize implementing it.
- Ticket 01: TOML contract with a digest over the contract bytes and the Git blobs it references. Authoritative context must be committed.
- Ticket 02: run journal at `<git-common-dir>/devx/runs/<id>/` (modes 0700/0600) with append-only `events.jsonl`, checkpoints, writer leases, trajectory hashes.
- Ticket 03: admission gate (clean linked worktree, no merge or rebase in progress, prerequisite receipts).
- Ticket 04: dirty-worktree recovery. Previous writer must be proven gone. Immutable snapshot (patch, index copy, no-follow untracked archive, hashes). Ambiguity goes to `needs-human`. Uncertain external effects are reconciled before retry.
- Ticket 05: evidence records and finish gate. Evidence is invalidated when the candidate changes, and every review finding is disposed of. Completion is separate from merge, push and deploy.
- Ticket 06: one runtime module, `scripts/agentic/task {context,validate,start,checkpoint,status,resume,rebind,finish}`, JSON for machines and text for humans.
- Ticket 07: supervision packets, where a human answers a decision rather than acknowledging status.
- Tickets 08 to 10: classify the existing estate. Claimed tickets are never taken over, and tracker history is preserved.
- The receipt (`receipts/deepseek-v41-reconciliation.org`) separates promotion status from completion status. The threat model is explicitly not cryptographic attestation against a malicious local writer.

**Reusable for Corenius** (inference). JSONL run journal with private permissions; checkpoint phases; contract digest from Git blobs; candidate-bound evidence with invalidation; finish gate separate from promotion; recovery snapshot; `needs-human` state; writer lease with nonce and heartbeat; machine-readable CLI over one module.

**Hazards.** Specification only (no implementation found in the files read, inference). Depends on Kata and the Git-common-dir layout. Ceremony cost scales, and evidence invalidation and recovery are hard to test (tickets call for crash-injection tests). `.scratch` is uncommitted, so authoritative context may be missing. `pending-workstreams.org` records stale tracker statuses.

---

## 11. Smaller repos

Read for these: README, AGENTS/CONTEXT, doc indexes and grep hits. Depth is lighter, and mechanism claims come from docs.

**skaffen** (`skaffen/README.md`, `CONTEXT.md`, `docs/adr/0001-observe-execution-without-owning-it.md`, `contracts/model-mastery/v1/*.schema.v1.json`, `missions/wave-1-canonical-model-mastery.json`). A "Chief of Staff" that tracks a multi-repo model/MLSys campaign without owning execution. Loop: orient, prioritize, brief, delegate, verify, debrief. Monitoring is read-only (ADR 0001). Observations stay in ignored local state, and a sanitized receipt becomes tracked evidence only after explicit review and promotion. Non-waivable promotion gates. Schema fixtures, source audits, runtime evidence and quality claims are kept separate, and an unexecuted plan is never evidence. Reusable: evidence-bundle schemas, mission file format, promotion-gate primitive, read-only watch versus mutating action split. Hazards: Python, one mission so far, possibly over-fitted to model mastery (inference); no runtime enforcement.

**stratum-observatory** (`AGENTS.md`, `VERIFICATION.md`, `docs/AGENT_VERIFIER.md`, `verification/tests/test_reported_numbers.py`). Deterministic model-architecture visualization with a heavy verification regime. Separate engineering, science and runtime gates, each with a receipt bound to source, verifier, manifest and artifact hashes. `VERIFICATION.md` is generated, and a test fails if it drifts from the reports. The science gate is currently BLOCKED (exit 2), showing the gate can fail honestly. Prose may not contain quantities. Coverage is never inferred from names. Reusable: hash-bound receipts, docs-versus-report drift tests, tri-state results (pass, blocked, not-applicable), coding-agent contract file. Hazards: large slow suite (1483 Chromium states), Node and browser toolchain.

**soma** (`AGENTS.md`, `CONTEXT.md`, `docs/rfcs/0002-doom-emacs-gym-insula.md`, `tmp/jupyter-smoke-probes/`). Emacs/Doom gym with hermetic execution. An Insula is a bwrap rootfs whose tool and ABI closure is proven by contracts, and a namespace alone does not count. Fail-closed network policy (`SOMA_INSULA_NETWORK=share-net` also needs `SOMA_INSULA_ALLOW_NETWORK_FOR=<task-id>`). The dev wrapper is labeled `not_rootfs_owned` with `abi_guarantee=none`. Rootfs contract validated before any episode, provenance emitted at materialization. Reusable: plan/receipt consistency check, fail-closed network, explicit labels for weaker modes. Hazards: `soma/tmp` is committed probe output, heavy CUDA-image design.

**myriatron** (`experiments/trae_bwrap_broker/README.md`; `tools/anchor`, `tools/sandbox`, `rust/nova_anchor` not read). Throwaway prototype of a broker that lets an agent CLI delegate work into a sandbox without exposing credentials. One schema-versioned JSON request (`read_file`, `apply_patch`, `bazel_test`), read-only rootfs, network unshared, bounded output, digest-bound receipt per request. `bazel_test` runs on a deterministic-commit snapshot. Reusable: request/receipt schema, snapshot-based testing, network and hidden-path probes. Hazard: README says "not a production agent interface". The rest of the repo is a PyTorch ranking reference and not agent-relevant.

**waystone** (`CONTEXT.md`, `docs/verification/live-hdfs-io.md`, `rust/src/transfer.rs`). HDFS client stack. Only the verification docs are relevant: opt-in live I/O verification against a caller-owned test root, and a pinned-native-build contract. Possible Rust reference for CLI transfer verification (inference).

**grk** (`README.md`, `PROVENANCE.md`, `NOTICE.md`, `docs/ARCHITECTURE.md`, `docs/DEEP-DIVE.md`; source not read). Unofficial reconstruction of a Cursor-derived desktop agent app, with host, coordinator, local-exec daemon, cloud-agents and transcript-mirror boundaries (from doc and file names only). Useful for architecture reading only. Hazard: legal and provenance risk from reverse-engineered proprietary code. `research-archives/README.md` is a SHA-256-pinned installer archive (provenance pattern only).

**lancelot** (`README.md`): FineWeb-Edu loader, no agent content.

**surflo** (`tests/test_publication_audit.py`, `verify_recipe.py`): 3D reconstruction research fork, not agent-relevant.

---

## 12. Cross-cut

### Ideas that recur

1. **Evidence-bound claims.** Nothing counts as done without a fresh, recorded, checkable artifact. Corenius manifests; monarch `evidence-boundary.md` and non-evidence list; ferric candidate/evidence/promotion split; skaffen promotion gates; stratum hash-bound receipts and tri-state results; devx certificate; roborev per-job stored prompt, diff and verdict; harbor `reward.txt` plus trial `results.json`; milano's `AGENTS.md` (`b200-release` gate, receipts under `evidence/`).
2. **Isolated execution.** Worktree per task (roborev refine, bytedcli develop-to-merge, monarch gated path, ferric Task Run, milano `scm new`). Namespace sandboxes: bwrap in corenius `src/insula.rs`, soma, myriatron, monarch, milano. Container sandboxes: harbor. Workspaces: jj.
3. **Review loops with a bound.** roborev `refine --max-iterations 10`, monarch 3-iteration repair budget, Corenius SDD "fix round N/5", bytedcli Disposition Ledger with terminal states. All end in human-only merge.
4. **Ledgers.** roborev SQLite, jj operation log, ferric `events.jsonl`, bytedcli MR body and Lark thread, Corenius SDD `progress.md`, monarch `evalrun-state.json`, kube stage manifests.
5. **Fail closed, no silent fallback.** monarch "No fallback", soma network policy, devx aggregate failures, bytedcli banned silent returns and required `hint`, Corenius no last-number fallback.
6. **Separation of roles and authority.** Author versus verifier context (monarch, ferric), read-only observers (skaffen ADR 0001, kube), no self-merge (bytedcli, roborev, monarch).
7. **Redaction and permissions on evidence.** devx (0700/0600 and stream redaction), ferric (0700/0600), bytedcli (redacted text output), harbor (secret redaction in installed agents).
8. **Generated artifacts are checked for drift.** bytedcli mirrors, ui-ux `check:assets`, stratum `VERIFICATION.md` test, harbor adapter parity.

### Where the repos disagree

| Topic | Positions |
|---|---|
| Who is the verifier | roborev: a background daemon agent on every commit. monarch and ferric: a clean-context human-in-loop critic on a gated path. bytedcli: local `mr-reviewer` plus a remote bot. Harbor: a shell script with no LLM (rewardkit adds judges). |
| Sandbox strength | roborev: none of its own, delegates to agent CLIs, uses `--dangerously-skip-permissions` in agentic mode. soma, myriatron, monarch: contract-proven bwrap that fails closed. Harbor: container with opt-in network policy. Corenius: bwrap insula, but the ACP path delegates tool policy to `traecli`. |
| Where evidence lives | roborev: SQLite in `~/.roborev`. jj: repo op-store. ferric: `<git-common-dir>/devx/runs`. monarch: committed result directories. Corenius and skaffen: gitignored, promoted by explicit act. |
| Merge model | milano and corenius-spike: linear history via rebase and fast-forward. jj: change ids with automatic rebase and first-class conflicts. bytedcli: MR plus merge queue owned by a bot. |
| Status of the process | ferric and monarch rules are prose or spec (partly unimplemented). bytedcli and milano enforce mechanically (pre-commit lint, hooks, gates). |
| Determinism of scoring | Corenius and harbor AIME: exact integer match. roborev: string-parsed verdict, explicitly not heuristic. monarch: fixture scores are non-evidence. devx: live Copilot output non-deterministic and certified only structurally. |
| Agent-context size | bytedcli `AGENTS.md` is about 46 KB. Skill-router pattern (bytedcli, ui-ux) pushes detail into references. Corenius has no `AGENTS.md` at all. |

### Gaps nobody covers (inference unless noted)

1. **Tamper-evident ledgers.** Ferric states the limit ("not cryptographic attestation"), Milano says "Receipts are not yet signed", and roborev notes no tamper evidence. No repo has hash-chained or signed run records.
2. **Cost and token budgets as a first-class gate.** roborev's plan doc lists "no verification budget" as a gap. Corenius records `usage` but never aggregates it. Harbor records cost per step but has no budget gate.
3. **Variance and statistics for agent evals.** Corenius samples are single-shot. Harbor has pass@k but the AIME adapter does not use repeated attempts by default (inference). No repo has baselines with confidence intervals and regression thresholds.
4. **Contamination and answer-leak controls for public benchmarks.** Only Corenius's substring `PromptContamination` check exists. Harbor's AIME template has network on and the answer in the tests directory.
5. **Prompt-injection defense across the review path.** roborev escapes and uses trusted-branch config. ui-ux states data-is-not-instructions. Nobody tests it with adversarial fixtures.
6. **A tool-using agent loop conformance suite that spans providers.** Monarch's probe is the only one, and it targets a Responses adapter. Corenius's ACP path has no tool loop.
7. **Durable resume of an agent session, not just an eval run.** Corenius `durable-execution.md` is design only, and ferric's journal is spec only. Harbor and monarch resume at trial or stage level only.
8. **Rollback of agent turns.** Only jj has operation-level undo. Nobody wires it to an agent turn boundary (inference).
9. **Cross-repo evidence index.** Evidence lives in five incompatible places. No repo indexes runs across them.
10. **Enforcement of the prose rules.** Monarch and ferric describe gates that no code checks. Bytedcli-style pre-commit lints are the model, but not applied to agent-process rules.
11. **A tested agent-instruction file for Corenius itself.** No `AGENTS.md`, `CLAUDE.md` or `.claude` exists in any of the three Corenius checkouts.

### Suggested first uses for Corenius (inference, ranked by leverage and cost)

1. Adopt Harbor's task directory format and ATIF output for the AIME harness, so results become comparable and cross-checkable (harbor section).
2. Add a candidate-bound, append-only run journal with a content digest of the dirty tree, closing Corenius hazard 2 (ferric tickets 01, 02, 04; monarch `evalrun-state.json`).
3. Add a failure classification that separates infrastructure from model failures, and carry `AcpFailureKind` into samples (monarch, harbor typed errors).
4. Add roborev-style bounded review loop and stored boolean verdicts to the SDD workflow, with worktree-per-task and human-only merge.
5. Write a mechanically checked `AGENTS.md` (bytedcli pre-commit lint pattern), and keep it short (bytedcli hazard).
6. Prototype jj workspaces with operation ids recorded per agent turn, as the rollback primitive, after resolving the git-linear-history policy conflict.

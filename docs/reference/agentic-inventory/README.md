# Agentic-engineering inventory (2026-09-29)

Four read-only sweeps of every workspace repo. Each file labels what was read
versus inferred and cites paths. Depth differs: skills and products were
skimmed and sampled, and Arnold's policy files and about 130 bytedcli domain
guides were not read. Nothing here was independently re-verified beyond spot
checks; treat details as leads.

| File | Covers |
| --- | --- |
| [policy-and-efforts.md](policy-and-efforts.md) | Ultron-distributed policy copies and their drift, repo-local `docs/agents`, cross-repo trackers, stale worktrees |
| [skills-and-commands.md](skills-and-commands.md) | About 100 hand-written skills, 258 generated, commands, hooks, workflows; duplicates; proposed shared layout |
| [agent-products.md](agent-products.md) | roborev, harbor, bytedcli, jj, monarch's agentic work, devx and the rest: what each teaches and offers corenius |
| [guidance-and-workflows.md](guidance-and-workflows.md) | Rules every repo tells its agents, the SDD ledger workflow, 20 conflicts (C1-C20), gaps |

## Prioritized absorption list

Merged from the four sweeps. Order is by leverage and by what blocks the rest.

1. **Reconcile the policy fleet.** No repo matches canonical 2026.09.29.1; fix
   before any `sync --apply`. Hand-merge the diverged forks (torchtitan, ds4,
   ferric_continuum) first, then decide the lagrange, torchetta and verl v08.11
   constitutions.
2. **Settle the rule conflicts.** Who merges, force-push, worktrees, commit
   staging, trailers, fix-round cap (3 versus 5). See conflicts C1-C20; the
   milano-versus-vaso merge-authority question is one of them.
3. **Give corenius its own house in order.** Add an `AGENTS.md`, commit the
   isolation work, and stop gitignoring evidence that manifests refer to.
4. **Skills.** A frontmatter schema and validator first, then a shared review
   layer, a merged develop-to-merge and land-on-mainline flow, verification
   skills, and one sandbox-run skill.
5. **Absorb the missing mechanisms.** Ultron's branch-authority text, Kata
   `active-work.md`, ferric's evidence-bound-method and task-runs docs with its
   ready-for-agent issues, vaso's relay ideas (already imported as snapshots).
6. **Fill gaps nobody covers.** Tamper-evident ledgers, cost and token gates,
   eval variance statistics, and enforcement hooks (only one exists today).

## Report-only findings

Prunable `/tmp` and `~/.cache` worktrees and stale Ultron kernel, release and
active-work worktrees are listed in `policy-and-efforts.md`. Nothing was
removed.

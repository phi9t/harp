# Agentic engineering practice

Corenius is the home for how agents are engineered and how they collaborate.
This directory collects the practice that exists in sibling repositories and
is where new practice is designed. Direction from the owner (2026-09-29):
anything relevant to agentic engineering should come here.

## Status of the imports

`imported/` holds **reference snapshots**, copied verbatim, not forks. Each
source repository still runs its own copy, and its own copy is what its hooks
and scripts execute. Nothing was moved or deleted at the source. A snapshot is
current only as of the commit below; when corenius supersedes one, the source
repo adopts the corenius version and the snapshot is removed.

| Snapshot | Source | Commit | What it is |
| --- | --- | --- | --- |
| `imported/milano/scm.py`, `test_scm.py` | `milano/milano/`, `milano/tests/` | milano `ed218ae` | worktree-per-task, gated rebase + fast-forward integrate, pre-push rules |
| `imported/milano/gpu_lease.sh` | `milano/scripts/lib/` | milano `ed218ae` | flock GPU leases |
| `imported/milano/pre-commit`, `pre-push` | `milano/.githooks/` | milano `ed218ae` | hooks |
| `imported/milano/multi-agent-section.md` | `milano/AGENTS.md` | milano `ed218ae` | the written rules |
| `imported/vaso/cross-agent-collaboration.md`, `templates/` | `vaso/docs/agents/` | vaso `03f93d9` | lead/follower protocol, failure-mode table |
| `imported/vaso/relay.py`, `test_agent_relay.py` | `vaso/scripts/agents/`, `vaso/tests/` | vaso `03f93d9` | landing-branch relay, manifest, watch |
| `imported/robust-training-observatory/terminology.md` | observatory `AGENTS.md` | rto `584d9b8` | shared vocabulary (Insula, capsule, hermetic, ...) |

## Absorbed from Ultron (decision 2026-09-29)

The owner decided that Ultron does not own agentic-engineering policy;
corenius does. The canonical files now live here, at the paths sibling repos
expect: `CONSTITUTION.md`, `docs/agents/` (`agentic-engineering.md`,
`skill-orchestration.md`, the manifest `agentic-workflow-repos.json`, and the
tracker templates), plus `tools/sync_agentic_workflow.py` and its test
(21 pass). The managed docs now name corenius as the canonical source, and the
manifest registers corenius as a managed repo. Cross-repository efforts start
in corenius `.scratch/<effort>/`.

Sync semantics: managed policy files and the managed AGENTS block are enforced
(DRIFT, repaired by `--apply`). The local tracker files (`issue-tracker.md`,
`triage-labels.md`, `domain.md`) are install-once: created from the template
when missing (reported as MISSING), and never reported as drift or overwritten
once they exist, so repos keep their own tracker text.

Not done yet, deliberately, because each touches other repos:

1. **Ultron still has its copies.** They are now stale relative to corenius.
   Ultron should become a consumer, and its copies removed once the sync has run.
2. **`sync --apply` has not been run.** Run `--check` first from the landed
   corenius checkout (the tool locates its source at `<workspace>/corenius`).
   Applying rewrites the managed block and policy files in about ten repos.
3. **Names still say Ultron.** The `ultron-agentic-workflow:start` marker in
   every repo's `AGENTS.md`, and the `ultron/mainline` branch pointer, are
   kept for compatibility. Renaming needs a migration in the tool.
4. **`CONSTITUTION.md`** is upstream (`kenn-io/constitution`), copied as is.

## Not imported

- **Product code** (`torch_accel`, Bazel/Spack graphs) and Ultron's knowledge
  and research content. Only engineering-process artifacts belong here.
- **Evidence and receipts** (`milano` cache) stay with the repo that produced them.

## Open decisions

1. **How the mechanism reaches the repos.** Vendor a copy per repo, depend on
   a corenius binary (`coord`), or a thin per-repo adapter. The design-space
   document recommends the runtime approach once the interception question
   (`spikes/collab/design-space.md`) is answered.
2. **Merge authority.** Vaso and the constitution keep merge with the human;
   milano lets an agent integrate to its mainline after gates. A shared
   mechanism needs one configurable rule.

## Where the work is

- `docs/spikes/collab/design-space.md`: mechanism survey and recommended design.
- `experiments/collab/`: the simulator that measures the candidates.

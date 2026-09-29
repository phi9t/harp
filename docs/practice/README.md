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

## Deliberately not imported

- **Ultron-managed policy** (`CONSTITUTION.md`, `docs/agents/agentic-engineering.md`,
  `skill-orchestration.md`). Ultron declares itself the canonical source and
  distributes these to sibling repos through
  `ultron/docs/agents/agentic-workflow-repos.json`. Copying them here would
  create a second, drifting source. Whether corenius replaces Ultron as that
  source is an open decision, listed below.
- **Product code** (`torch_accel`, Bazel/Spack graphs). Only the collaboration
  and engineering-process artifacts belong here.
- **Evidence and receipts** (`milano` cache). These stay with the repo that
  produced them.

## Open decisions

1. **Ultron versus corenius as the canonical policy source.** Ultron currently
   owns the managed files and the distribution manifest. Options: keep Ultron
   for policy and make corenius the home for *mechanisms* (recommended for
   now); or move the manifest and policy to corenius and have Ultron consume it.
2. **How the mechanism reaches the repos.** Vendor a copy per repo, depend on
   a corenius binary (`coord`), or a thin per-repo adapter. The design-space
   document recommends the runtime approach once the interception question
   (`spikes/collab/design-space.md`) is answered.
3. **Merge authority.** Vaso and the Ultron constitution keep merge with the
   human; milano lets an agent integrate to its mainline after gates. A shared
   mechanism needs one configurable rule.

## Where the work is

- `docs/spikes/collab/design-space.md`: mechanism survey and recommended design.
- `experiments/collab/`: the simulator that measures the candidates.

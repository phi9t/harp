# Sibling repository references

What each repo is for, how it is designed, and what it teaches an agent
working across the workspace. Written 2026-09-29 by reading the repos at the
commits recorded at the top of each file; each claim cites a path or is marked
inferred. These are snapshots. Re-read the repo before relying on a detail.

| Repo | One line | Commit read |
| --- | --- | --- |
| [milano](milano.md) | Hermetic build and qualification harness for the `torch_accel` PyTorch device on B200; gated, evidence-bound releases | `ed218ae` |
| [torchetta](torchetta.md) | Builds the ByteDance libtorch forks and native CUDA consumers in an Insula rootfs, records release evidence | `9322aa1` |
| [ultron](ultron.md) | Knowledge, research-journal and claim-tracking repo, plus a fixture-only recovery-policy model | `5061e89` |
| [ndtl](ndtl.md) | Experimental append-only per-rank JSONL observation of training runs, plus the `synd` sidecar | `8bc8ba7` |
| [arnold](arnold.md) | Product monorepo for the GPU training platform: Eurus collectors, Housekeeper failover, ETTR and traces | `756a165` |

## Cross-cutting findings

Stated by the reference authors from their reading; verify before acting.

- **No code-level links between them.** Arnold references none of the others,
  and nothing references ndtl or ultron's controller. The couplings are
  vocabulary, data schemas and human process. Any coordination mechanism
  corenius builds must supply the missing contract, not assume one.
- **Evidence claims are weaker than they look, in more than one repo.**
  Milano receipts copied some fields from the target file and recorded a wrong
  NCCL version (finding N2, since fixed); torchetta's validator hashes the
  verifier report but not the wheel or rootfs, and uses substring checks;
  milano receipts are unsigned and failed gates leave no receipt.
- **Drift hazards.** Ultron's documented test command runs a fraction of its
  tests; ultron's adapter rejects real ndtimeline rows; torchetta's staged
  manifest is a schema its own validator rejects; MLU targets were removed from
  torchetta but still described in `distributed_timeline`.
- **Agents already work in these repos** (torchetta had a live agent editing
  while it was read). Guidance files, ticket ledgers and reviewer protocols are
  gitignored in places (`.scratch/`), so they do not survive a fresh clone.

## Known defect found while writing these (milano)

`milano.scm integrate` cannot retry after a push is rejected: it fast-forwards
the local mainline to the gated tip before pushing, so when another agent wins
the race, the next attempt rebases onto the new origin, the old local tip is no
longer an ancestor, and it fails with "local ... has commits not on the rebased
branch". Confirmed by reading `milano/scm.py` `integrate` and
`_fast_forward_mainline`. The retry loop was never exercised by a test. Tracked
in the milano reference, section 8 (S1).

## How these relate to the rest of corenius

- Practice and policy: `../practice/README.md`, `../agents/`.
- Collaboration design and evidence: `../spikes/collab/design-space.md`,
  `../../experiments/collab/RESULTS.md`.

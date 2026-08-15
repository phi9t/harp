---
id: crouzeix-proof-reproduction-research
title: Crouzeix proof-reproduction research
type: research
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [crouzeix-conjecture, proof-reproduction, ai-assistance, provenance]
confidence: medium
canonical: reproduction_research.md
---

# Crouzeix proof-reproduction research

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

## Scope and method

This research note is bounded to:

1. `jinshanmu/CrouzeixConjecture` at Git commits
   [`565b6a3e0659b6e0785f783b016c3f6d9f171fa5`](https://github.com/jinshanmu/CrouzeixConjecture/tree/565b6a3e0659b6e0785f783b016c3f6d9f171fa5)
   and
   [`9df07838327b988e3924453daa29c8cd726d34b0`](https://github.com/jinshanmu/CrouzeixConjecture/tree/9df07838327b988e3924453daa29c8cd726d34b0),
   including their reachable Git history;
2. Lorist-Schwenninger arXiv
   [`2608.03841v1`](https://arxiv.org/abs/2608.03841v1), only for its proof
   mechanism comparison and AI disclosure; and
3. Harp's existing Crouzeix evidence receipts and canonical packet.

No secondary commentary or unrelated paper is used. Immutable upstream bytes
were inspected in temporary storage and were not added to the repository.
Source claims below use commit-pinned GitHub locators or arXiv v1 line
locators. Harp receipts establish measured identity, not mathematical truth
([source manifest](../../evidence/crouzeix_conjecture/source_manifest.tsv#L1),
[source registry](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)).

## Source-inventory checkpoint

This table is the provisional inventory recorded before synthesis. `Observed
role` describes what the artifact can establish; it does not infer hidden
worker behavior.

| Source | Immutable identity | Measured identity | Observed role |
|---|---|---|---|
| Jin root prompt | Git blob reachable at both bounded commits and first public commit `df001051d58e2e897038ad849ef0cc8a7b88b349` | 4,106 bytes; SHA-256 `0a0c3000b81efc4d9edc65ec3cd1d53df0d4e69b24bfee9fe0860301d853d6fc` | Root task contract and requested orchestration; not a run transcript ([prompt lines 28-47](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L28-L47)) |
| Jin first public commit | `df001051d58e2e897038ad849ef0cc8a7b88b349`, 2026-07-18T16:15:19+08:00 | Git commit and tree metadata | Adds the prompt and a candidate proof in the same commit; does not establish their private temporal order ([commit](https://github.com/jinshanmu/CrouzeixConjecture/commit/df001051d58e2e897038ad849ef0cc8a7b88b349)) |
| Jin audited v4 | Git commit `565b6a3e0659b6e0785f783b016c3f6d9f171fa5`, tree `40aafa503bd32762dbf6d1a67ddef3e2b067f0e1` | v4 TeX: 39,727 bytes; SHA-256 `5713de029c4a7486e25e86d16e6413d04929bdf5f92439c3237d4a930b1c9242` | Formalization-matched proof, formalization map, manuscript audit, and specific AI disclosure ([receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L11)) |
| Jin bounded head | Git commit `9df07838327b988e3924453daa29c8cd726d34b0`, tree `ff9ff787a91707ddf747d2c670bf9e729e1c0cab` | archive: 2,905,847 bytes; SHA-256 `6fd68bb302a3bdbe68d8a4e540e6a2b4b973359f88a12bab4d4b0a2aab47a270` | Later repository state, unchanged root prompt, README, commit history, and expanded manuscript ([receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L14)) |
| Jin formalization records | Files at `565b6a3` | map SHA-256 `2f2acda668efcb3866a034e3cea3c87261a9fb5b4ee83bc5ec9519855b051b3e`; audit SHA-256 `e49ec74f445ddfbfdc5275d04c8e2e340f64c1780af53e46c4ecfc273bc07826` | Author-maintained theorem map and manuscript correspondence record ([map receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L7), [audit receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L10)) |
| Lorist-Schwenninger source | arXiv `2608.03841v1` | source archive: 7,330 bytes, SHA-256 `b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9`; TeX: 18,783 bytes, SHA-256 `20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a` | Independent mechanism comparison and AI disclosure ([source receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L27), [TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)) |
| Harp packet | Current canonical Crouzeix packet and receipts | Revision-local Markdown plus registered manifests | Existing claim boundaries, proof reconstruction, and verification observations; secondary synthesis rather than upstream process evidence ([packet index](crouzeix_conjecture_index.md), [claim ledger](claim_evidence_ledger.md)) |

### Prompt identity check

Fresh extraction of
[`crouzeix_conjecture_prompt.txt`](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt)
from the bounded Git repository produced:

```text
byte count  4106
SHA-256     0a0c3000b81efc4d9edc65ec3cd1d53df0d4e69b24bfee9fe0860301d853d6fc
```

These values exactly match Harp receipt
[`JIN-HEAD-PROMPT`](../../evidence/crouzeix_conjecture/source_manifest.tsv#L20).
The same path is present at `565b6a3`, and `git log --follow` attributes its
addition to the first public commit
[`df00105`](https://github.com/jinshanmu/CrouzeixConjecture/commit/df001051d58e2e897038ad849ef0cc8a7b88b349).
This establishes public byte continuity within the bounded history, not that
the prompt was the exact private invocation or that its instructions were
executed.

## Recoverable historical protocol

### Root prompt: recovered bytes

The recoverable root contract is the 47-line prompt blob
`5b2705db56787157fadbfd9416522feb69b4ad95`, identical at the first public
commit and both bounded revisions. It contains two layers:

- lines 1-25 state the finite-dimensional polynomial conjecture; and
- lines 28-47 specify the research task and orchestration policy
  ([pinned prompt](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L1-L47)).

The task layer asks for a standalone compilable LaTeX proof, assumes an
affirmative proof exists, excludes theorem-strength reductions and finite
computational checks as terminal results, prohibits use of public or connected
sources, and requires iteration until a complete proof survives adversarial
audit
([task and success contract](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L28-L47)).
Its output path is user-specific:
`/Users/shanmujin/Documents/CrouzeixConjecture/LaTeX`
([prompt line 30](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L30)).

### Requested root and worker behavior

The prompt requests, but does not prove execution of, this control loop:

1. launch a diverse portfolio of mathematically distinct approaches;
2. preserve early independence by withholding the favored route from most
   workers;
3. maintain an explicit registry of approach families;
4. block routes that end at theorem-strength missing lemmas unless a new
   mechanism appears;
5. keep incompatible routes alive before cross-pollination;
6. use adversarial workers to inspect gaps, circularity, and handwaving;
7. demand concrete lemmas, constructions, equations, or counterexamples; and
8. have a root repeatedly synthesize, challenge, redirect, and launch new
   rounds
   ([prompt lines 33-45](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L33-L45)).

These are **root-prompt instructions**. The bounded corpus contains no worker
transcript or registry showing their realization
([MISSING - CC-044](claim_evidence_ledger.md#cc-044-underlying-agent-transcripts-are-absent)).
Consequently, statements such as “workers independently explored multiple
families” or “the root reopened a blocked route” are not recoverable facts.

### Public artifact sequence

The reachable Git history gives an artifact sequence, not a private discovery
trace:

| Public point | Commit time | What the commit records |
|---|---:|---|
| [`df00105`](https://github.com/jinshanmu/CrouzeixConjecture/commit/df001051d58e2e897038ad849ef0cc8a7b88b349) | 2026-07-18T16:15:19+08:00 | First public commit adds the prompt, candidate proof, README, and PDFs together |
| [`627bb2a`](https://github.com/jinshanmu/CrouzeixConjecture/commit/627bb2ab311146ed770a80fea72107c20bea857d) | 2026-07-19T09:20:51+08:00 | Adds the Lean formalization and audit records |
| [`4910388`](https://github.com/jinshanmu/CrouzeixConjecture/commit/4910388e4aa501fef3031019156a8172832c9ca0) | 2026-07-31T00:11:15+08:00 | Records an auxiliary-basis proof simplification and corresponding formalization changes |
| [`8096084`](https://github.com/jinshanmu/CrouzeixConjecture/commit/809608471754712fc6e68b9e149bf89a5f8598cc) | 2026-08-03T23:01:32+08:00 | Aligns Lean with the simplified v3 proof |
| [`595f6fc`](https://github.com/jinshanmu/CrouzeixConjecture/commit/595f6fc702a77978c589e4f02fdc324996f9d39c) | 2026-08-05T16:13:36+08:00 | Adds the v4 proof and a large formalization update |
| [`565b6a3`](https://github.com/jinshanmu/CrouzeixConjecture/commit/565b6a3e0659b6e0785f783b016c3f6d9f171fa5) | 2026-08-05T16:32:05+08:00 | Binds the formalization records to the 39,727-byte v4 manuscript |
| [`c724883`](https://github.com/jinshanmu/CrouzeixConjecture/commit/c7248839e773856e3fe8a9cd26f056dada47dacb) | 2026-08-06T10:17:01+08:00 | Expands the v4 source with a mass-parameterized completion principle |
| [`9df0783`](https://github.com/jinshanmu/CrouzeixConjecture/commit/9df07838327b988e3924453daa29c8cd726d34b0) | 2026-08-06T23:43:43+08:00 | Final bounded state, with later finite-dimensional and operator formulations |

Commit metadata can establish ordering and changed paths. Because the root
prompt and first proof arrive in one commit, it cannot establish when the
prompt was run, what preceded it, or whether the committed proof was its
direct output
([first commit](https://github.com/jinshanmu/CrouzeixConjecture/commit/df001051d58e2e897038ad849ef0cc8a7b88b349)).

## Publicly documented proof-discovery actions

The public sources document the following actions. “Documented” means an
author statement or observable repository change, not an independently
reconstructed model trajectory.

| Action | Evidence and locator | Supported conclusion | Claim ceiling |
|---|---|---|---|
| Frame a complete-proof search with independent families, blocking rules, and adversarial audit | Root prompt, [lines 28-47](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L28-L47) | This was the public task contract | Does not show the orchestration ran |
| Contribute proof development, manuscript preparation, and adversarial checking | Bounded README, [lines 45-47](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/README.md#L45-L47) | The repository author attributes these broad activities to OpenAI ChatGPT | No transcript, model version, scope allocation, or audit output is identified |
| Propose scaled conjugate-eigenvalue samples and a compensating origin sample | Audited v4, [lines 1085-1095](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L1085-L1095) | The author specifically attributes the decisive sampling/cancellation idea to OpenAI ChatGPT | Does not establish the exact prompt, intermediate reasoning, novelty allocation, or whether every algebraic step came from the model |
| Implement the cancellation as `w_i = conjugate(lambda_i)/2` and `v = -G^{-1}Pu` | Audited v4, [lines 387-423](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L387-L423) | The published mechanism makes the attributed idea mathematically concrete: `Gv + Pu = 0` removes the unknown correction | Final source is outcome evidence, not evidence of how the idea was found |
| Formalize the sampling, ordered Gramian inequality, and norm endpoint | Formalization map, [lines 28-38](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/Lean/FORMALIZATION_MAP.md#L28-L38) | Author-maintained records map the mechanism to named Lean declarations | The map is not a build receipt; Harp's builds remain blocked |
| Explore proof strategies for the independent perturbation route | Lorist-Schwenninger source, lines 153-154, pinned by the [TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28) | The authors disclose use of ChatGPT 5.6 Pro for strategy exploration and retain responsibility | No prompts, rejected routes, or mapping from model output to Lemma 1 is supplied |
| Publish a distinct power-iterate mechanism | Lorist-Schwenninger source, lines 67-99 and 125-128, pinned by the [TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28) | Uniformly bounded commuting perturbations `E_n` yield a recurrence that rules out norm above two; applying it to every `f^n` closes the double-layer argument | This comparison does not prove that AI produced the lemma |

There is an attribution drift inside the bounded history. The first public
README says “OpenAI Codex” contributed to development, preparation, and
checking
([`df00105` README lines 34-39](https://github.com/jinshanmu/CrouzeixConjecture/blob/df001051d58e2e897038ad849ef0cc8a7b88b349/README.md#L34-L39));
the bounded revisions say “OpenAI ChatGPT”
([`565b6a3` README lines 45-47](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/README.md#L45-L47)).
The root prompt names neither product nor model. The evidence therefore does
not identify one reproducible historical runtime.

## Missing process evidence

The following entries are `MISSING` from the bounded public corpus. They are
requirements for historical process reproduction, not assertions that the
records do not exist privately.

| Missing item | Why it matters | Resolution evidence |
|---|---|---|
| Exact root invocation, system/developer instructions, provider, model build, sampler settings, and seed | The same user prompt can induce materially different search behavior across runtimes | An immutable root request/response envelope with model and sampling metadata |
| Worker prompts, worker outputs, launch topology, and context supplied to each worker | The prompt prescribes independence and dynamic delegation, but neither can be audited from the root text | A per-worker event log binding parent, prompt, context, model, output, and timestamps |
| Approach-family registry and route-state transitions | The central anti-collapse machinery cannot be measured without family labels and blocked/reopened decisions | An append-only registry with stable route IDs, evidence for each state transition, and parent-child provenance |
| Rejected routes, adversarial findings, and dispositions | Final manuscripts preferentially preserve successful reasoning and cannot reveal selection pressure or defect yield | Candidate snapshots plus finding, severity, owner, disposition, and revised artifact |
| Tool calls, local files, computational checks, and source-access logs | The historical prompt prohibited external and local context, but compliance is not observable | Sandboxed filesystem/network receipts and content-addressed tool I/O |
| Token, wall-clock, worker-count, and retry accounting | Efficiency and the effect of aggressive multi-agent search cannot be compared | Scheduler and provider usage logs with a declared accounting boundary |
| Edit-level lineage from generated candidate to committed TeX and Lean | Git proves public revisions, not which generated text or idea caused each edit | A provenance graph joining run outputs, human edits, commits, manuscript equations, and Lean declarations |
| Pre-publication timing | The first commit adds prompt and proof together, so public Git cannot order private prompt execution and proof creation | Signed or otherwise integrity-protected records predating the first commit |
| Exact identity behind “Codex,” “ChatGPT,” and later “ChatGPT 5.6 Pro” | Product names and attribution changed or differ across sources | Provider-issued immutable model identifiers bound to each run |

The corpus-level absence is already registered as
[`CC-044`](claim_evidence_ledger.md#cc-044-underlying-agent-transcripts-are-absent).
The first commit's co-arrival of prompt and outcome is visible in its
[file-level diff](https://github.com/jinshanmu/CrouzeixConjecture/commit/df001051d58e2e897038ad849ef0cc8a7b88b349).

## Reproduction anchor {#reproduction-anchor}

The defensible anchor is a **prospective clean-room rerun of the public task
contract**, not a replay of the unobserved historical run.

### Fixed historical input

- Preserve the exact 4,106-byte prompt blob and its SHA-256 as the historical
  treatment
  ([prompt receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L20)).
- Treat lines 1-25 as the theorem statement and lines 28-47 as the orchestration
  contract
  ([pinned prompt](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L1-L47)).
- Run it in a clean workspace with public network and preexisting project
  context disabled, matching the prompt's declared constraints
  ([prompt lines 28-32](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L28-L32)).
- Record the user-specific output path as a historical nonportable parameter,
  rather than silently changing the prompt bytes
  ([prompt line 30](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L30)).

### Evaluation anchor

The candidate output must be evaluated without requiring textual or structural
similarity to either published proof. A valid new route is allowed. For a
Jin-like route, the source-backed checkpoint is the chain from positive-real
completion through exact correction cancellation, ordered Gramian inequality,
first-term norm endpoint, fixed-domain simple-spectrum limit, and outer-domain
limit
([formalization map lines 28-38](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/Lean/FORMALIZATION_MAP.md#L28-L38),
[lines 64-80](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/Lean/FORMALIZATION_MAP.md#L64-L80)).
For a Lorist-Schwenninger-like route, the checkpoint is its uniformly bounded
commuting `E_n` family, scalar recurrence, terminal-term bound, and
double-layer instantiation
([arXiv v1 TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28),
[Harp reconstruction](05_lorist_schwenninger_proof.md)).

The historical Lean anchor is source-level only: Lean 4.28.0, Mathlib
`8f9d9cff6bd728b17a24e163c9402775d9e6a365`, `lake build`, then
`lake env lean AxiomAudit.lean`
([toolchain receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L9),
[verify receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L12)).
Harp's two clean-room build observations are `blocked`, neither passing nor
failing
([verification manifest](../../evidence/crouzeix_conjecture/verification_manifest.tsv#L2)).
They must not be used as a successful historical build result.

## Leakage tiers

Leakage must be an explicit experimental condition because the decisive Jin
mechanism and the independent Lorist-Schwenninger mechanism are now public.

| Tier | Material visible to the candidate-generating system | Permitted claim |
|---|---|---|
| `L0: theorem only` | Prompt lines 1-25 only; no orchestration text, repository, packet, manuscripts, Lean names, or arXiv source | Tests unaided proof search under the fixed theorem statement |
| `L1: historical prompt` | Exact 4,106-byte root prompt, including orchestration, but no Jin/Lorist-Schwenninger repository, packet, disclosures, or mechanism vocabulary | Tests the recoverable public task contract; this is the primary reproduction tier |
| `L2: attribution leak` | README and AI disclosures, including “scaled conjugate eigenvalues” and “origin sample,” but no derivation | Tests completion from a decisive mechanism hint; cannot support an independent-rediscovery claim |
| `L3: mechanism leak` | Jin or Lorist-Schwenninger manuscript, formalization map, theorem names, or Harp mechanism chapters | Tests reconstruction, debugging, or formalization, not discovery |
| `L4: full outcome` | Both proofs, Lean tree, Harp claim ledger, critical review, and evaluator notes | Tests exposition or verification only |

The tier boundary follows the sources themselves: the root prompt contains
process instructions but no Herglotz sampling idea
([prompt lines 28-47](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L28-L47)),
whereas the audited disclosure names both the scaled samples and the origin
sample
([v4 lines 1085-1095](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L1085-L1095)).
Training-data contamination is separate from supplied-context leakage and must
be reported even at `L0` and `L1`.

## Variables/controls

This is an experimental design inventory, not an improved prompt.

### Independent variables

| Variable | Levels or accounting rule | Source basis |
|---|---|---|
| Prompt condition | `L0` theorem-only versus `L1` exact historical prompt; future prompt variants must be separate treatments | The historical prompt's theorem/task split ([lines 1-47](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L1-L47)) |
| Runtime identity | Provider, immutable model build, sampling configuration, seed | `MISSING` from the Jin corpus; LS only names ChatGPT 5.6 Pro ([LS receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)) |
| Search topology | Single agent versus root/worker orchestration; fixed versus dynamic concurrency | Root prompt requests dynamic multi-agent search ([lines 33-42](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L33-L42)) |
| Independence policy | Favored-route visibility, family-registry enforcement, cross-pollination threshold | Root prompt requests early independence and delayed cross-pollination ([lines 35-39](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L35-L39)) |
| Audit policy | No adversary, end-only adversary, or continuous adversarial review | Root prompt requests adversarial workers throughout ([line 40](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L40)) |
| Tool access | Text reasoning only; sandboxed computation; Lean available to verifier only; Lean available during search | Prompt permits computation but excludes fixed-parameter verification as a terminal result ([lines 30-32](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L30-L32)); the public formalization has a pinned toolchain ([Lean README lines 14-17](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/Lean/README.md#L14-L17)) |
| Budget | Root tokens, aggregate worker tokens, wall time, maximum live workers, and retry ceiling reported separately | Historical values are `MISSING` ([CC-044](claim_evidence_ledger.md#cc-044-underlying-agent-transcripts-are-absent)) |
| Leakage | `L0` through `L4`, plus a separate training-contamination declaration | Decisive disclosure and proof sources are public ([v4 disclosure](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L1085-L1095), [arXiv v1](https://arxiv.org/abs/2608.03841v1)) |

### Controls

- Use the same theorem statement bytes, success definition, runtime batch,
  seed schedule, wall-clock and token ceilings, and sandbox policy across
  prompt treatments.
- Keep candidate generation blind to manuscripts, Lean declaration names,
  Harp chapters, and evaluator rubrics at `L0` and `L1`.
- Freeze an append-only event schema before runs: run ID, parent ID, route ID,
  prompt digest, model ID, tool input/output digests, start/end timestamps,
  token counts, state transition, and selected descendants.
- Have evaluators inspect anonymized candidates without knowing prompt
  treatment, cost, or whether the route resembles Jin or
  Lorist-Schwenninger.
- Separate generation from verification. A verifier may use the pinned
  manuscripts and Lean records after candidate freeze, but those artifacts
  must not flow back into a clean-room generation run.
- Preserve all failed and rejected candidates, because the public repository
  retains outcomes but not the search denominator
  ([first public commit](https://github.com/jinshanmu/CrouzeixConjecture/commit/df001051d58e2e897038ad849ef0cc8a7b88b349)).

## Candidate metrics

No metric may substitute stylistic similarity to a published proof for
correctness.

| Metric | Definition | Interpretation |
|---|---|---|
| Complete-proof rate | Fraction of frozen candidates whose full implication chain survives independent mathematical review, with no theorem-strength missing lemma | Primary endpoint; matches the root prompt's completion criterion ([lines 31-32](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L31-L32)) |
| Audit-survival rate | Fraction of promoted candidates remaining valid after adversarial findings are resolved and re-reviewed | Measures the prompt's “survives adversarial audit” endpoint ([lines 40-45](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L40-L45)) |
| Unproved-obligation count | Number and strength class of unsupported lemmas at freeze time; theorem-equivalent obligations receive the highest severity | Detects elegant reductions that do not close the conjecture ([prompt lines 37-38](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L37-L38)) |
| Independent-family coverage | Number of substantively distinct proof families reaching a concrete lemma before cross-pollination, classified blind from the registry | Tests whether orchestration prevents early route collapse ([prompt lines 34-39](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L34-L39)) |
| Defect yield and closure | Adversarial findings per candidate, stratified by severity, plus the fraction fixed, rejected, or unresolved | Distinguishes active falsification from unrecorded reassurance; the README only reports no specific error ([README lines 7-10](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/README.md#L7-L10)) |
| Time and cost to first valid mechanism | Wall time and aggregate tokens until the first candidate component later retained in a complete proof | Enables efficiency comparison without conflating root and worker budgets |
| Provenance completeness | Fraction of candidate claims linked to route, prompt, model, tool receipts, parent outputs, review findings, and final disposition | Measures whether a future history can support stronger process claims than `CC-044` |
| Mechanism rediscovery, secondary | Whether a clean-room run independently reaches Jin's sample/origin cancellation or Lorist-Schwenninger's bounded commuting perturbation recurrence | Diagnostic only. Absence does not invalidate a different complete proof; presence at `L2+` is not independent rediscovery |
| Formal verification status | `not attempted`, `blocked`, `failed`, or `passed`, with toolchain, source digest, axioms, and logs | Prevents a blocked build from being reported as either success or failure ([verification manifest](../../evidence/crouzeix_conjecture/verification_manifest.tsv#L2)) |

For quantitative reporting, the denominator must be all launched candidates,
not only promoted manuscripts. Token cost must report root, worker, verifier,
and failed-retry subtotals separately before giving an aggregate.

## Threats to validity

1. **Training contamination.** A model can know one or both public 2026 proofs
   without receiving them in context. Context tier `L0` or `L1` therefore does
   not prove historical independence
   ([Jin bounded repository](https://github.com/jinshanmu/CrouzeixConjecture/tree/9df07838327b988e3924453daa29c8cd726d34b0),
   [Lorist-Schwenninger arXiv v1](https://arxiv.org/abs/2608.03841v1)).
2. **Runtime nonidentity.** The first README names Codex, later README text
   names ChatGPT, the Jin prompt names no runtime, and Lorist-Schwenninger name
   ChatGPT 5.6 Pro. A modern rerun cannot be called an exact model replay
   ([first README](https://github.com/jinshanmu/CrouzeixConjecture/blob/df001051d58e2e897038ad849ef0cc8a7b88b349/README.md#L34-L39),
   [bounded README](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/README.md#L45-L47),
   [LS TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)).
3. **Prompt-outcome co-publication.** The first commit adds both prompt and
   proof, so Git cannot establish causation or private order
   ([commit `df00105`](https://github.com/jinshanmu/CrouzeixConjecture/commit/df001051d58e2e897038ad849ef0cc8a7b88b349)).
4. **Survivorship and selection bias.** Public Git records selected artifacts,
   while failed agents, rejected routes, and human selection decisions are
   absent
   ([CC-044](claim_evidence_ledger.md#cc-044-underlying-agent-transcripts-are-absent)).
5. **Instruction compliance is not observable.** The prompt prohibits public
   and local sources, but no sandbox receipt proves that the historical run
   complied
   ([prompt line 30](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L30)).
6. **Evaluator anchoring.** Reviewers familiar with the two public routes may
   overvalue matching vocabulary or undervalue a valid third route. Blind
   correctness review and mechanism-similarity scoring must remain separate.
7. **Formalization is not a historical process transcript.** The Lean map is
   strong outcome structure but does not reveal model contribution or search
   chronology
   ([formalization map lines 1-10](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/Lean/FORMALIZATION_MAP.md#L1-L10)).
8. **Revision drift.** The audited map binds the 39,727-byte v4 at `565b6a3`;
   the bounded head v4 is 70,088 bytes with a different digest, while the
   author-maintained map digest is unchanged
   ([audited receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L11),
   [head v4 receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L22),
   [head map receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L16)).
9. **Verification censoring.** Harp's clean-room Lean builds stopped at a disk
   preflight. Treating them as pass or fail would bias the outcome
   ([audited build log](../../evidence/crouzeix_conjecture/verification/jin-565b6a3-build.log#L1),
   [head build log](../../evidence/crouzeix_conjecture/verification/jin-9df0783-build.log#L1)).
10. **Shared prerequisite risk.** Mechanistically distinct endpoints can still
    share double-layer or reduction assumptions; agreement is not independent
    line-by-line validation
    ([LS source lines 119-128, TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28),
    [Harp proof comparison](06_proof_comparison.md)).
11. **Disclosure granularity.** Author disclosures support attribution at the
    stated level, not chain-of-thought reconstruction or quantitative credit
    allocation
    ([Jin v4 lines 1085-1095](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L1085-L1095),
    [LS source receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)).

## Source table

| ID | Class | Immutable locator | Digest or identity | Used for | Cannot establish |
|---|---|---|---|---|---|
| `JIN-PROMPT` | Public primary Git artifact | [`9df0783:crouzeix_conjecture_prompt.txt`](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt) | Git blob `5b2705db56787157fadbfd9416522feb69b4ad95`; 4,106 bytes; SHA-256 `0a0c3000b81efc4d9edc65ec3cd1d53df0d4e69b24bfee9fe0860301d853d6fc` | Theorem, task, and requested orchestration | Execution, runtime, compliance, or success |
| `JIN-FIRST` | Public primary Git metadata | [commit `df00105`](https://github.com/jinshanmu/CrouzeixConjecture/commit/df001051d58e2e897038ad849ef0cc8a7b88b349) | Commit SHA; 2026-07-18T16:15:19+08:00 | First public co-arrival of prompt, proof, and README | Private temporal order or causation |
| `JIN-V4-AUDITED` | Public primary Git manuscript | [`565b6a3` v4 TeX](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex) | 39,727 bytes; SHA-256 `5713de029c4a7486e25e86d16e6413d04929bdf5f92439c3237d4a930b1c9242` | Proof mechanism and specific ChatGPT attribution | Underlying transcript or peer review |
| `JIN-MAP` | Public primary author-maintained record | [`565b6a3:Lean/FORMALIZATION_MAP.md`](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/Lean/FORMALIZATION_MAP.md) | 21,057 bytes; SHA-256 `2f2acda668efcb3866a034e3cea3c87261a9fb5b4ee83bc5ec9519855b051b3e` | Manuscript-to-declaration map and active endpoint chain | Successful local build or historical AI process |
| `JIN-AUDIT` | Public primary author-maintained record | [`565b6a3:Lean/MANUSCRIPT_AUDIT.md`](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/Lean/MANUSCRIPT_AUDIT.md) | 14,966 bytes; SHA-256 `e49ec74f445ddfbfdc5275d04c8e2e340f64c1780af53e46c4ecfc273bc07826` | Exact audited manuscript/toolchain binding and author audit | Independent review or head-v4 correspondence |
| `JIN-HEAD` | Public primary Git snapshot | [tree `9df0783`](https://github.com/jinshanmu/CrouzeixConjecture/tree/9df07838327b988e3924453daa29c8cd726d34b0) | Tree `ff9ff787a91707ddf747d2c670bf9e729e1c0cab`; archive SHA-256 `6fd68bb302a3bdbe68d8a4e540e6a2b4b973359f88a12bab4d4b0a2aab47a270` | Final bounded repository state and history | Identity with the audited v4 or Preprints.org bytes |
| `LS-ARXIV-V1` | Public primary arXiv source | [arXiv `2608.03841v1`](https://arxiv.org/abs/2608.03841v1) | TeX SHA-256 `20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a`; source archive SHA-256 `b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9` | Independent proof mechanism, historical characterization, and ChatGPT 5.6 Pro disclosure | Prompt, transcript, private chronology, or peer review |
| `HARP-EVIDENCE` | Local receipt and verification layer | [source manifest](../../evidence/crouzeix_conjecture/source_manifest.tsv#L1), [verification manifest](../../evidence/crouzeix_conjecture/verification_manifest.tsv#L1) | Receipt schemas bind each listed artifact and observation | Fresh byte measurements and typed local build/scan outcomes | Upstream mathematical truth |
| `HARP-PACKET` | Harp synthesis | [packet index](crouzeix_conjecture_index.md), [claim ledger](claim_evidence_ledger.md) | Current repository revision | Mechanism reconstruction and claim ceilings | Independent primary evidence about discovery |

## Findings checkpoint

1. **The public root prompt is reproducible as bytes.** It is 4,106 bytes,
   SHA-256 `0a0c3000b81efc4d9edc65ec3cd1d53df0d4e69b24bfee9fe0860301d853d6fc`,
   and the same Git blob is reachable at the first public commit and both
   bounded revisions
   ([receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L20),
   [first commit](https://github.com/jinshanmu/CrouzeixConjecture/commit/df001051d58e2e897038ad849ef0cc8a7b88b349)).
2. **The historical run is not reproducible from the public corpus.** The
   requested multi-agent registry, worker interactions, route transitions,
   tool calls, budgets, model identity, and selection lineage are absent
   ([CC-044](claim_evidence_ledger.md#cc-044-underlying-agent-transcripts-are-absent)).
3. **The strongest Jin AI attribution is specific.** The audited manuscript
   attributes scaled conjugate-eigenvalue sampling and the compensating origin
   sample to OpenAI ChatGPT; the equations implement exact cancellation
   ([disclosure](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L1085-L1095),
   [cancellation](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L387-L423)).
4. **The public task contract and the decisive idea are separate evidence.**
   The prompt prescribes search governance but does not contain the sampling
   mechanism. The mechanism first appears in outcome artifacts, so it must be
   withheld from `L0`/`L1` generation
   ([prompt](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/crouzeix_conjecture_prompt.txt#L28-L47),
   [v4 disclosure](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L1085-L1095)).
5. **Lorist-Schwenninger provide a useful mechanism comparator, not a replay
   trace.** Their arXiv v1 uses bounded commuting perturbations across all
   powers and discloses only that ChatGPT 5.6 Pro explored proof strategies
   ([arXiv v1](https://arxiv.org/abs/2608.03841v1),
   [TeX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28)).
6. **A defensible reproduction is prospective and tiered.** Its primary
   treatment is the exact root prompt under `L1`, with source isolation,
   append-only process receipts, blind correctness evaluation, and all failed
   routes retained. It cannot be described as an exact historical replay
   because runtime and process evidence are missing.
7. **Formal verification status must stay typed.** The source-level theorem
   map is substantial, but Harp's clean-room builds are blocked rather than
   passed or failed
   ([formalization map](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/Lean/FORMALIZATION_MAP.md),
   [verification manifest](../../evidence/crouzeix_conjecture/verification_manifest.tsv#L2)).

The bounded corpus is exhausted for this research question. Each requested
section now contains either source-backed findings or an explicit `MISSING`
entry. This note does not propose a replacement prompt or implement the
experiment.

Back to the [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Crouzeix conjecture index]].

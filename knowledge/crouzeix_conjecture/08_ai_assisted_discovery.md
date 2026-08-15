---
id: crouzeix-ai-assisted-discovery
title: AI-assisted discovery and documented actions
type: deep-dive
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [crouzeix-conjecture, ai-assistance, discovery, provenance]
confidence: medium
canonical: 08_ai_assisted_discovery.md
---

# AI-assisted discovery and documented actions

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

The public artifacts document two kinds of AI involvement: a specific
sampling/cancellation idea in Jin's manuscript and proof-strategy exploration
in Lorist-Schwenninger. They do not expose the complete interaction history
needed to reconstruct discovery or allocate credit at the level of individual
reasoning steps.

## Jin task contract {#jin-task-contract}

The pinned Jin prompt asks for a complete standalone proof, explicitly rejects
reductions that stop at theorem-strength missing lemmas, requests a diverse
portfolio of approaches, and directs the root agent to:

- preserve independence between early routes;
- maintain an approach-family registry;
- reopen blocked routes only for materially new mechanisms;
- use adversarial agents to find gaps and circularity;
- require equations, constructions, or counterexamples rather than status
  reports; and
- continue until a complete proof survives audit.

This establishes the intended search protocol. It does not prove which parts
were actually executed, because run transcripts are not in the repository.

## Documented action matrix {#documented-action-matrix}

| Documented action | Source | Context/tool | Supported conclusion |
|---|---|---|---|
| Search for a complete standalone proof | Jin prompt receipt | Multi-agent instruction | The task contract asked for diverse approaches, approach-family tracking, and adversarial checks |
| Sample at half conjugate eigenvalues and add an origin sample | Jin disclosure | OpenAI ChatGPT | The manuscript attributes the cancellation design to ChatGPT |
| Explore proof strategies | Lorist-Schwenninger disclosure | ChatGPT 5.6 Pro | The authors report strategy exploration |
| Prepare exposition, bibliography, and typesetting | Jin disclosure | OpenAI ChatGPT | Assistance extended beyond the central sampling idea |

The matrix records actions that the authors disclose. It does not infer model
intent, hidden chain-of-thought, autonomous authorship, or the fraction of total
research labor performed by a model.

## Jin's specific attribution {#jin-specific-attribution}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-042: Jin attributes sampling and cancellation to ChatGPT|EVIDENCE - CC-042]].**
Jin's formalization-matched v4 disclosure attributes two linked choices to
OpenAI ChatGPT:

1. sample the matrix Herglotz kernel at scaled conjugate eigenvalues, using
   scale `1/2` in the mass-two application; and
2. add the origin sample that cancels the nonconstant adjoint-algebra
   correction.

Those choices correspond to the displayed samples
`w_i = conjugate(lambda_i)/2`, the vector `v = -G^{-1}Pu`, and the exact
`Gv + Pu = 0` cancellation. The disclosure also reports help with exposition,
bibliography, and typesetting. The human author accepts responsibility for the
mathematics and citations.

## Lorist-Schwenninger's disclosure {#lorist-schwenninger-disclosure}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-043: Lorist-Schwenninger disclose ChatGPT strategy exploration|EVIDENCE - CC-043]].**
Lorist and Schwenninger state that ChatGPT 5.6 Pro was used to explore proof
strategies for their note. They state that the authors wrote the note and take
responsibility for it.

The disclosure does not identify whether the final perturbation lemma, the
recurrence, the double-layer application, or rejected candidate routes arose
from any particular model output. Harp therefore records “strategy
exploration,” not invention of the final lemma.

## Artifact timeline {#artifact-timeline}

The inspected Jin Git history records:

```text
2026-07-18  candidate proof project status
2026-07-19  Annals-format and Lean artifacts
2026-07-31  auxiliary-basis simplification and standalone v2
2026-08-03  simplified v3 formalization alignment
2026-08-05  finalized v4 proof and formalization
2026-08-06  expanded completion and operator-level formulations
```

This is repository history, not a transcript history. Commit dates can order
public artifacts but cannot show when a private idea was first generated,
tested, rejected, or independently rediscovered.

Lorist-Schwenninger arXiv v1 is pinned as `2608.03841v1`. Its introduction
states that Jin's proof appeared independently during preparation and uses a
different route. The arXiv artifact does not expose the private sequence of
strategy-exploration conversations.

## Missing process evidence {#missing-process-evidence}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-044: Underlying agent transcripts are absent|MISSING - CC-044]].**
The inspected public corpus does not contain:

- underlying ChatGPT transcripts;
- per-agent prompts or model settings;
- the complete approach-family registry;
- rejected proof attempts and adversarial findings;
- timestamps tying outputs to manuscript edits;
- token, tool, and elapsed-time accounting; or
- a provenance graph from candidate ideas to accepted equations.

Without these artifacts, the defensible claims are the disclosed actions and
the public prompt's intended protocol. It would be overreach to reconstruct a
fine-grained discovery narrative from the final manuscripts alone.

## Reproducible action ledger {#reproducible-action-ledger}

For future AI-assisted theorem work, preserve this minimum record:

| Resolving action | Enabling context | Tool or skill | Required receipt |
|---|---|---|---|
| Generate independent proof families | Problem statement and forbidden shortcuts | Parallel reasoning agents | Prompt, model identity, output, and family label |
| Reject theorem-strength reductions | Explicit completeness criterion | Adversarial proof review | Blocking lemma and rejection rationale |
| Cross-pollinate viable routes | Mature independent derivations | Synthesis pass | Parent outputs and accepted transfer |
| Test algebraic cancellation | Exact formulas and symbolic examples | CAS or formal proof assistant | Input, output, assumptions, and digest |
| Promote a final proof | Source-bound claim graph | Human plus formal/adversarial review | Review findings, disposition, and final source identity |

This ledger would not decide mathematical correctness by itself. It would make
the process auditable and let later reviewers distinguish search, synthesis,
verification, and exposition.

Back to the [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Crouzeix conjecture index]].

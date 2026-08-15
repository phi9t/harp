---
id: verified-coevolution-source-registry
title: Verified coevolution agenda - source registry
type: source-registry
mode: SOURCE REGISTRY
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [recursive-self-improvement, coevolution, sources, provenance]
confidence: medium
---

# Verified coevolution agenda source registry

Mode: `SOURCE REGISTRY`.

The authoritative structured rows are in `content/sources/source_registry.tsv`.
This page gives packet-local reading guidance for the agenda-specific sources.

## Public source rows

| Source ID | Identity | Local artifact | Claim ceiling |
|---|---|---|---|
| `LADDER` | arXiv `2503.00735v3`, `LADDER: Self-Improving LLMs Through Recursive Problem Decomposition` | `evidence/verified_coevolution_agenda/artifacts/ladder/` | Author-reported recursive variant generation, verifier-guided GRPO, LADDER-TTRL setup, results, and limitations; no reproduction. |
| `PRIME-TTRL` | arXiv `2504.16084v3`, `TTRL: Test-Time Reinforcement Learning` | `artifacts/prime_ttrl/` | Author-reported majority-vote pseudo-label reward and test-distribution RL method; no reproduction. |
| `NSRSA` | arXiv `2603.21558v1`, `Stabilizing Iterative Self-Training with Verified Reasoning via Symbolic Recursive Self-Alignment` | `artifacts/nsrsa/` | Reasoning-trace quality filters and reported recursive self-training experiment; not broad alignment evidence. |
| `SAHOO` | arXiv `2603.06333v1`, `SAHOO: Safeguarded Alignment for High-Order Optimization Objectives in Recursive Self-Improvement` | `artifacts/sahoo/` | Goal Drift Index, constraint checks, regression-risk monitoring, and reported control results; not proof of invariant goals. |
| `MODEL-COLLAPSE` | Nature `s41586-024-07566-y`, `AI models collapse when trained on recursively generated data` | `artifacts/model_collapse/` | Recursive generated-data collapse mechanism and reported experiments; no local reproduction. |
| `SCRIVENS-VERIFICATION` | arXiv `2603.28650v1`, `Information-Theoretic Limits of Safety Verification for Self-Improving Systems` | `artifacts/scrivens_verification/` | Conditional theory of statistical gates and property verification under stated sequential-model assumptions. |
| `GODEL-AGENT` | arXiv `2410.04444v4`, `Godel Agent: A Self-Referential Agent Framework for Recursive Self-Improvement` | `artifacts/godel_agent/metadata/` | Metadata and abstract-page identity only; full text intentionally not vendored here. |
| `MENDEL-GODEL-MACHINE` | arXiv `2608.07645v1`, `Mendel Godel Machine: Recursive Self-Improving Coding Agents via Comparative Evolution` | `artifacts/mendel_godel_machine/metadata/` | Metadata and abstract-page identity only; full text intentionally not vendored here. |
| `GODEL-MACHINE` | Existing Springer summary plus arXiv `cs/0309048v5` identity | `artifacts/godel_machine_arxiv/metadata/` and existing Springer evidence | Publisher-summary claim ceiling is preserved; arXiv abstract identity is added without vendoring full text. |

## Bundle contract

`evidence/verified_coevolution_agenda/manifest.tsv` records 34 source records:
33 public source records plus the supplied research agenda. Five CC-BY arXiv
works have Atom metadata, abstract page, semantic HTML, PDF, and deterministic
`pdftotext -raw` text. The Nature model-collapse article has first-party HTML
and deterministic plain text. Godel Agent, Mendel Godel Machine, and the
historical Godel Machine arXiv identity are metadata-only records.

`artifact_inventory.tsv` records every file in the bundle, including
acquisition, provenance, manifest, and receipt files. `capture_receipt.tsv`
records the acquisition-script and manifest digests.

## Reused Harp evidence

This packet reuses, rather than duplicates, Harp evidence for STOP,
Meta-Harness, AHE, DGM, ADAS, and the Springer Godel Machine record. When a
claim depends on those systems, cite the existing Harp evidence row and this
packet's claim ledger boundary rather than recapturing the same source.

## Missing and rights-restricted surfaces

`GODEL-AGENT`, `MENDEL-GODEL-MACHINE`, and the arXiv `GODEL-MACHINE` record are
metadata-only in this packet. Do not cite local full text for them. If a later
change clears redistribution rights or chooses a different evidence policy, it
must update this file, the claim ledger, `manifest.tsv`, and the source
registry row before reader prose changes.

---
id: crouzeix-source-registry
title: Crouzeix conjecture source registry
type: source-registry
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [crouzeix-conjecture, numerical-range, provenance, source-registry]
confidence: high
canonical: source_registry.md
---

# Crouzeix conjecture source registry

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

This registry fixes the identities and claim ceilings used throughout the
packet. Upstream proof bytes remain remote-only. The local
[source manifest](../../evidence/crouzeix_conjecture/source_manifest.tsv#L1)
binds remote artifacts to immutable identities and digests; it does not make
their mathematics correct.

## JIN-V4-AUDITED: Formalization-matched Git manuscript {#jin-v4-audited-formalization-matched-git-manuscript}

- Class: pinned Git manuscript and formalization record.
- Identity: commit `565b6a3e0659b6e0785f783b016c3f6d9f171fa5`, tree
  `40aafa503bd32762dbf6d1a67ddef3e2b067f0e1`.
- Local receipt: [JIN-565-V4-TEX](../../evidence/crouzeix_conjecture/source_manifest.tsv#L11).
- Semantic locator: commit-pinned
  [v4 TeX](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex).
- Can support: the proof interface, formulas, limit order, AI disclosure, and
  manuscript-to-Lean map explicitly tied to this revision.
- Cannot support: peer review, a successful Harp-local Lean build, identity
  with the Preprints.org posting, or claims about later repository prose.

## JIN-REPO-HEAD: Pinned repository head {#jin-repo-head-pinned-repository-head}

- Class: pinned Git repository snapshot.
- Identity: commit `9df07838327b988e3924453daa29c8cd726d34b0`, tree
  `ff9ff787a91707ddf747d2c670bf9e729e1c0cab`.
- Local receipt: [JIN-HEAD-ARCHIVE](../../evidence/crouzeix_conjecture/source_manifest.tsv#L14).
- Semantic locator: commit-pinned
  [repository tree](https://github.com/jinshanmu/CrouzeixConjecture/tree/9df07838327b988e3924453daa29c8cd726d34b0).
- Can support: the later repository layout, README status language, prompt,
  Lean declarations, and revision history.
- Cannot support: byte identity between its manuscripts and earlier audited
  v4, a successful Harp-local Lean build, or journal submission status.

## JIN-V4-HEAD: Later v4 manuscript {#jin-v4-head-later-v4-manuscript}

- Class: pinned Git manuscript.
- Identity: repository-head commit
  `9df07838327b988e3924453daa29c8cd726d34b0`.
- Local receipt: [JIN-HEAD-V4-TEX](../../evidence/crouzeix_conjecture/source_manifest.tsv#L22).
- Semantic locator: commit-pinned
  [later v4 TeX](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex).
- Can support: the later manuscript's wording and its distinct digest.
- Cannot support: the assertion that the Lean manuscript audit was refreshed
  for these bytes or that this is the Preprints.org file.

## JIN-ANNMATH: Annals-formatted manuscript {#jin-annmath-annals-formatted-manuscript}

- Class: pinned Git manuscript.
- Identity: repository-head commit
  `9df07838327b988e3924453daa29c8cd726d34b0`.
- Local receipt: [JIN-HEAD-ANNMATH-TEX](../../evidence/crouzeix_conjecture/source_manifest.tsv#L13).
- Semantic locator: commit-pinned
  [Annals-formatted TeX](https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/AnnMath/the_numerical_range_is_a_2_spectral_set.tex).
- Can support: existence and byte identity of the formatted artifact.
- Cannot support: receipt, review, acceptance, or publication by the Annals of
  Mathematics.

## JIN-PREPRINTS-V1: Preprints.org metadata record {#jin-preprints-v1-preprints-org-metadata-record}

- Class: dated metadata record.
- Identity: URL observation on `2026-08-14`; acquisition returned HTTP `403`.
- Local receipt: [JIN-PREPRINTS-V1-METADATA](../../evidence/crouzeix_conjecture/source_manifest.tsv#L24).
- Semantic locator:
  [Preprints.org record](https://www.preprints.org/manuscript/202607.1919/v1).
- Can support: existence of the named metadata route at the observation date.
- Cannot support: manuscript bytes, a Git-revision mapping, peer review, or
  mathematical correctness.

## LS-ARXIV-V1: Lorist-Schwenninger arXiv v1 {#ls-arxiv-v1-lorist-schwenninger-arxiv-v1}

- Class: pinned arXiv source, PDF, and metadata.
- Identity: `arxiv:2608.03841v1`.
- Local receipts: [TeX](../../evidence/crouzeix_conjecture/source_manifest.tsv#L28),
  [PDF](../../evidence/crouzeix_conjecture/source_manifest.tsv#L26), and
  [source archive](../../evidence/crouzeix_conjecture/source_manifest.tsv#L27).
- Semantic locator: versioned
  [arXiv abstract](https://arxiv.org/abs/2608.03841v1).
- Can support: the perturbation lemma, recurrence, double-layer application,
  abstract uniform-algebra variant, disclosures, and stated limitations.
- Cannot support: peer review, independent reproduction, or complete
  boundedness.

## CROUZEIX-2007: Earlier universal numerical-range bound {#crouzeix-2007-earlier-universal-numerical-range-bound}

- Class: publisher metadata record.
- Identity: DOI `10.1016/j.jfa.2006.10.013`.
- Local receipt: [CROUZEIX-2007](../../evidence/crouzeix_conjecture/source_manifest.tsv#L2).
- Semantic locator: [publisher DOI](https://doi.org/10.1016/j.jfa.2006.10.013).
- Can support: bibliographic identity for the earlier universal-bound route.
- Cannot support: full-text theorem details not independently captured here.

## CROUZEIX-PALENCIA-2017: One-plus-square-root-two result {#crouzeix-palencia-2017-one-plus-square-root-two-result}

- Class: publisher metadata record.
- Identity: DOI `10.1137/17M1116672`.
- Local receipt: [CROUZEIX-PALENCIA-2017](../../evidence/crouzeix_conjecture/source_manifest.tsv#L3).
- Semantic locator: [publisher DOI](https://doi.org/10.1137/17M1116672).
- Can support: bibliographic identity for the $1+\sqrt2$ result.
- Cannot support: an independent derivation or a claim that the constant is
  attained.

## DELYON-DELYON-1999: Double-layer calculus {#delyon-delyon-1999-double-layer-calculus}

- Class: publisher metadata record.
- Identity: DOI `10.24033/bsmf.2340`.
- Local receipt: [DELYON-DELYON-1999](../../evidence/crouzeix_conjecture/source_manifest.tsv#L4).
- Semantic locator: [publisher DOI](https://doi.org/10.24033/bsmf.2340).
- Can support: bibliographic identity for the double-layer framework.
- Cannot support: every specialized identity used by the two new preprints
  without their own derivations.

## RANSFORD-SCHWENNINGER-2018: Prior proof analysis {#ransford-schwenninger-2018-prior-proof-analysis}

- Class: publisher metadata record.
- Identity: DOI `10.1137/17M1143757`.
- Local receipt: [RANSFORD-SCHWENNINGER-2018](../../evidence/crouzeix_conjecture/source_manifest.tsv#L29).
- Semantic locator: [publisher DOI](https://doi.org/10.1137/17M1143757).
- Can support: bibliographic identity for prior analysis of the
  Crouzeix-Palencia route.
- Cannot support: uncaptured theorem text or a constant-two conclusion.

## SCHWENNINGER-DEVRIES-2025: Double-layer review {#schwenninger-devries-2025-double-layer-review}

- Class: publisher metadata record.
- Identity: DOI `10.1007/s00020-025-02800-2`.
- Local receipt: [SCHWENNINGER-DEVRIES-2025](../../evidence/crouzeix_conjecture/source_manifest.tsv#L30).
- Semantic locator: [publisher DOI](https://doi.org/10.1007/s00020-025-02800-2).
- Can support: bibliographic identity for the modern double-layer review.
- Cannot support: uncaptured proof details or validation of either candidate
  proof.

## HARP-LOCAL-VERIFY: Local build and scan observations {#harp-local-verify-local-build-and-scan-observations}

- Class: Harp-generated, revision-bound verification receipts.
- Identity: normalization `crouzeix-log-normalization/v1`, Lean
  `leanprover/lean4:v4.28.0`, Mathlib
  `8f9d9cff6bd728b17a24e163c9402775d9e6a365`.
- Local receipt: [verification manifest](../../evidence/crouzeix_conjecture/verification_manifest.tsv#L1).
- Can support: two zero-finding prohibited-token scans and two typed build
  blocks caused by the recorded disk preflight.
- Cannot support: a successful Lean build, absence of all proof gaps,
  manuscript correspondence beyond named records, or theorem truth.

## Quotation audit

The packet uses paraphrase for mathematical claims. Its only short source
phrases are theorem/declaration names and the disclosures needed to distinguish
reported AI actions. No quotation is treated as proof evidence, and no quoted
passage exceeds 50 words.

Back to the [Crouzeix conjecture index](crouzeix_conjecture_index.md).

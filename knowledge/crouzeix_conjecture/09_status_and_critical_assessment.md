---
id: crouzeix-status-and-critical-assessment
title: Crouzeix proof status and critical assessment
type: deep-dive
status: active
created: 2026-08-14
updated: 2026-08-20
tags: [crouzeix-conjecture, publication-status, critical-review, missing-evidence]
confidence: medium
canonical: 09_status_and_critical_assessment.md
---

# Crouzeix proof status and critical assessment

> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.

## Strongest justified conclusion {#strongest-justified-conclusion}

The pinned Jin and Lorist-Schwenninger preprints present two detailed,
mechanistically distinct routes to the constant-two Crouzeix conclusion. Harp
can reproduce their finite-dimensional cores from the inspected sources:

- Jin's exact correction cancellation, ordered Gramian inequality, and norm
  endpoint; and
- Lorist-Schwenninger's perturbation recurrence, terminal bound, and
  contradiction above two.

Jin's repository also contains a broad Lean development whose source-level
declaration graph matches the advertised route. Harp's scans found no
prohibited tokens. Harp did not complete either clean-room build of Jin's
upstream repository revision because the disk policy blocked cache
materialization. Separately, Harp now has a local Lean port whose terminal
polynomial assembly compiles under the shared root with proof-slice receipts.

The strongest justified status is therefore:

> two source-backed candidate proofs; one Harp-local source-mapped Lean port
> with passing terminal proof-slice receipts for the polynomial route; upstream
> clean-room Jin repository builds, publication status, Preprints.org byte
> identity, and independent mathematical review remain open evidence channels.

## Jin Git manuscript versus Preprints.org metadata {#jin-git-manuscript-versus-preprints-metadata}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-045: Preprints manuscript bytes are not mapped to Git|MISSING - CC-045]].**
The Preprints.org route returned HTTP `403` during headless acquisition and is
registered as metadata only. Harp does not possess the posted manuscript bytes,
so it cannot identify the posting with:

- the formalization-matched v4 at `565b6a3`;
- the later v4 at `9df0783`;
- the Annals-formatted manuscript; or
- any other Git artifact.

Title or author similarity is not byte identity. The packet therefore cites
Git manuscripts for equations and uses Preprints.org only for the dated
metadata boundary.

## Manuscript versus Lean revision identity {#manuscript-versus-lean-revision-identity}

The formalization map and manuscript audit are strongest at the earlier
formalization-matched revision. At repository head, the v4 manuscript digest
has changed while the manuscript-audit artifact retains its earlier identity.

This creates three distinct questions:

1. Does the Lean source build at each revision?
2. Do the exported formal statements imply the intended constant-two theorem?
3. Does the author-maintained manuscript map describe the exact TeX bytes being
   presented?

The source graph provides evidence for question 2. The blocked local builds
leave question 1 unresolved for the upstream Jin repositories in Harp. The
Harp-owned proof port is a separate local formalization channel with its own
receipts. The digest mismatch limits question 3 at repository head.

## Lorist-Schwenninger publication status {#lorist-schwenninger-publication-status}

The registered artifact is arXiv `2608.03841v1`, with versioned TeX, PDF, Atom
metadata, and source-archive digests. Harp has no peer-review, acceptance, or
journal-publication receipt for this proof. Its mathematical claims remain
preprint claims even though the packet reproduces the core derivation.

The note itself reports that Jin's proof appeared independently and uses a
different approach. That statement is preserved as the authors' report, not as
an independently verified priority determination.

## Annals-formatted artifact versus submission status {#annals-formatted-artifact-versus-submission-status}

**[[knowledge/crouzeix_conjecture/claim_evidence_ledger#CC-046: Annals submission status is not established|MISSING - CC-046]].**
Jin's repository contains an `AnnMath/` manuscript and README language calling
it a submission manuscript. The local receipt establishes that the TeX artifact
exists at repository head. It does not establish:

- that the manuscript was transmitted to the Annals of Mathematics;
- a submission date or manuscript number;
- an editor or reviewer assignment;
- review outcome; or
- acceptance or publication.

Venue formatting and venue workflow are separate facts.

## What the two proof artifacts do not validate about each other {#what-the-two-proof-artifacts-do-not-validate-about-each-other}

Agreement on the final constant and shared prerequisites is not independent
line-by-line verification:

- Jin's Lean development does not formalize the Lorist-Schwenninger
  perturbation lemma.
- Lorist-Schwenninger do not validate Jin's sampled-kernel cancellation or
  manuscript-to-Lean correspondence.
- Both rely on inherited double-layer and reduction machinery, packaged
  differently.
- Neither direct route establishes the completely bounded case.
- Both AI disclosures lack underlying interaction transcripts.

The two mechanisms are independent enough to be valuable cross-checks at the
architecture level, but they can still share an error in a common prerequisite
or reduction.

## Adversarial review checklist {#adversarial-review-checklist}

For Jin:

1. Check that simple spectrum is required only for the auxiliary matrix.
2. Permit repeated target values and repeated kernel samples.
3. Verify the correction cancellation on the prescribed graph vector only.
4. Preserve the order in `YG^{-1}P` and `PG^{-1}Y`.
5. Check norm convergence of both weighted Gramian series.
6. Keep the simple-spectrum limit inside a fixed outer domain.
7. Separate similarity from the final unitary conjugation.

For Lorist-Schwenninger:

1. Derive the recurrence using `E_nT = TE_n`.
2. Bound `T^n` directly from Equation 1 and bounded `E_n`.
3. Confirm `kappa^{-N}m_{N+1}` vanishes for `kappa > 1`.
4. Track the sign of `r_n`.
5. Verify the coefficient
   `1 - 1/(kappa - 1)^2` is positive only in the contradiction regime
   `kappa > 2`.
6. Distinguish the abstract lemma bound from the separate
   functional-calculus product bound.
7. Do not infer complete boundedness from complete positivity of the
   symmetrized map.

## Evidence that would raise confidence {#evidence-that-would-raise-confidence}

The most valuable next evidence is:

1. a clean build of both upstream Jin revisions under their pinned Lean and
   Mathlib identities;
2. machine-captured axiom output for the upstream exported polynomial and
   rational theorems;
3. a refreshed manuscript audit bound to the exact current TeX digest;
4. independent expert review of both finite-dimensional cores and inherited
   reduction steps;
5. a separate formalization of the Lorist-Schwenninger perturbation lemma;
6. lawful acquisition and hashing of the Preprints.org manuscript;
7. venue-issued status evidence for any submission claim; and
8. provenance-preserving AI interaction archives if fine-grained discovery
   claims are desired.

Until then, the packet should be used as a rigorous mechanism reconstruction
and evidence map, not as an announcement that external review is complete.

Back to the [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Crouzeix conjecture index]].

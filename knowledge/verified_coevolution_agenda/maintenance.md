---
id: verified-coevolution-maintenance
title: Verified coevolution agenda - maintenance rules
type: maintenance
mode: MAINTENANCE
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [recursive-self-improvement, maintenance, evidence, source-rights]
confidence: medium
---

# Verified coevolution agenda maintenance rules

Mode: `MAINTENANCE`.

This packet is intentionally separate from the nine-chapter RSI teaching spine.
Do not migrate or rewrite the teaching spine as part of this packet without a
separately reviewed change.

## Source revisions

- Preserve the source revisions named in `source_registry.md` and
  `evidence/verified_coevolution_agenda/manifest.tsv`.
- Record newer arXiv revisions as revision-watch metadata until separately
  audited. Do not silently update claims to a newer revision.
- Re-run `evidence/verified_coevolution_agenda/acquire.sh --refresh` only when
  intentionally refreshing the whole evidence bundle. Review changed bytes and
  update claim ceilings before changing reader prose.

## Claim IDs

- Keep `VCA-*` headings stable. Add new IDs for new material claims rather than
  renumbering existing IDs.
- Update `claim_evidence_ledger.md` before changing
  `verified_coevolution_agenda.md`.
- Every non-trivial reader-facing mechanism, result, comparison, hypothesis, or
  missing-evidence statement needs a ledger anchor and local evidence locator.

## Supplied input

`evidence/verified_coevolution_agenda/artifacts/supplied_research_agenda.txt`
is immutable design input. Do not edit it for spelling, formatting, or Harp
branding. If the user supplies a replacement agenda, preserve it as a new
artifact and update the receipt instead of rewriting the old one.

## Source-rights boundary

- Full local text is available for LADDER, PRIME-RL TTRL, NSRSA, SAHOO,
  Scrivens verification, and the Nature model-collapse article.
- Godel Agent, Mendel Godel Machine, and the historical Godel Machine arXiv
  record are metadata-only in this packet. Do not add full-text claims unless a
  later change clears the rights boundary and updates evidence, registry, and
  ledger files together.
- Existing Harp evidence for STOP, Meta-Harness, AHE, ADAS, DGM, and the
  Springer Godel Machine row should be reused rather than duplicated.

## Atlas and generated files

When changing canonical Markdown or route registration, regenerate both:

- `atlas/src/content/generated/corpus.json`
- `atlas/dist/harp-atlas.html`

Then run `cargo run -p harp -- repository verify`, copy the reported digest into
`docs/import-receipt.md`, and rerun the release gate.

## Deferred migration rule

This packet may link to RSI chapters, but it does not relocate or rewrite them.
Any future migration from this standalone agenda into the teaching spine must
name the source packet, preserve `VCA-*` claim anchors, update Atlas/search
contracts, and pass `mise run verify`.

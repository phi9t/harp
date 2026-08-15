---
id: darwinx-maintenance
title: DarwinX packet maintenance
type: maintenance
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [darwinx, maintenance, provenance]
confidence: high
---

# DarwinX packet maintenance

This file defines how to update the DarwinX packet without weakening its
evidence boundary.

## Owned files

```text
knowledge/darwinx/
├── darwinx_index.md
├── 01_mechanism_and_selection.md
├── 02_evaluation_audit.md
├── 03_critical_review.md
├── 04_comparative_synthesis.md
├── 05_successor_experiment.md
├── claim_evidence_ledger.md
├── source_registry.md
└── maintenance.md

evidence/darwinx/
├── PROVENANCE.md
├── acquire.sh
├── artifact_inventory.tsv
├── capture_receipt.tsv
├── manifest.tsv
├── artifacts/
├── metadata/
└── text/
```

`knowledge/darwinx/` owns maintained interpretation. `evidence/darwinx/`
owns captured upstream bytes, derived text, manifests, and the verbatim
provisional review.

## Updating the paper

When arXiv publishes a new revision:

1. do not overwrite the v1 artifacts;
2. add revision-specific PDF, HTML, metadata, API response, and text sidecar;
3. add new manifest and inventory rows;
4. record the new revision in the capture receipt;
5. diff the paper's method, tables, appendices, and limitations;
6. add or revise claim-ledger entries;
7. preserve old claim IDs unless their statements become false;
8. link superseding or contradicting claims explicitly; and
9. update reader prose only after the ledger is correct.

Do not cite an unversioned arXiv URL when the revision changes a material
claim.

## Updating public implementation status

The v1 paper does not link an official DarwinX optimizer implementation. If a
release appears:

1. verify that it is first-party;
2. pin a full Git commit;
3. record remote, revision, and license status;
4. capture only files needed for the source audit;
5. keep implementation behavior separate from paper intent;
6. run provider-free tests where possible;
7. record environment failures precisely; and
8. revise `DX-009` and `DX-010` rather than deleting their historical state.

BrowserCode is a browser action runtime. Do not classify it as the DarwinX
optimizer.

## Supplied review

`evidence/darwinx/artifacts/supplied_review.md` is immutable. It must continue
to match the captured SHA-256 in `manifest.tsv`.

Corrections belong in:

- `knowledge/darwinx/03_critical_review.md`;
- `knowledge/darwinx/claim_evidence_ledger.md`; and
- new versioned review notes if the user supplies a later analysis.

Do not edit the captured review to match the maintained packet.

## Claim IDs

- Keep every published `DX-*` ID stable.
- Add a new ID when evidence class, scope, or statement changes materially.
- Use `supersedes`, `contradicts`, `narrows`, or `unresolved-with` relationships
  in the ledger.
- Never reuse an ID for an unrelated claim.
- Every material claim in reader prose must link to one exact ledger heading.

## Quantitative updates

For every result, record:

- task count;
- metric;
- sampling count;
- frozen model;
- base and evolved harness identity;
- evolution and report split;
- retry and timeout policy;
- action and access policy;
- reproduction status; and
- source table, figure, or line locator.

Never average heterogeneous percentage-point changes into a scientific effect
unless the source defines a common estimand.

## Comparison systems

Meta-Harness and DGM use their existing Harp evidence. Do not duplicate their
paper or repository captures inside `evidence/darwinx/`.

HarnessX v1 is captured here because this packet audits a material comparison
that had no prior Harp evidence. If Harp later creates a full HarnessX packet,
move authority through an explicit registry update. Do not silently leave two
canonical copies.

## Prose audit

Before landing a change:

1. scan for `proves`, `guarantees`, `causes`, `validates`, and `reproduces`;
2. verify every use against the ledger;
3. distinguish archive admission from steering eligibility;
4. distinguish bounded regression from strict monotonicity;
5. distinguish task transfer from environment transfer;
6. distinguish a proprietary base rescue from a strong same-model comparison;
7. distinguish action-policy change from reasoning-only improvement; and
8. distinguish a positive merge from an isolated recombination effect.

## Regeneration and verification

After changing managed Markdown:

```sh
cargo run -p harp -- build
cd atlas
corepack pnpm run export:html
cd ..
mise run verify
```

The build must regenerate both:

- `atlas/src/content/generated/corpus.json`; and
- `atlas/dist/harp-atlas.html`.

If `harp repository verify` reports a stale payload digest, update
`docs/import-receipt.md` only after every other tracked file is settled. Rerun
the full verification gate afterward.

Back to the [DarwinX index](darwinx_index.md).

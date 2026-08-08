# Harp provenance: Weng RSI citation closure

## Scope and stopping rule

This bundle is the external-source portion of the bounded citation closure
anchored by Lilian Weng's *Harness Engineering for Self-Improvement*. It
contains 47 nodes:

- the anchor article at depth zero;
- all 39 numbered anchor references and three decision-relevant body links at
  depth one; and
- four retained foundations at depth two: `CONCRETE-SAFETY`, `FUNSEARCH`,
  `GODEL-MACHINE`, and `QD-2016`.

Depth-three references are parsed where the source representation permits it,
but are not fetched. This is a deliberate stop rule, not a claim that the
literal transitive closure is exhausted. The rationale and retention predicate
live in `content/bounded_transitive_closure.md`.

The three implementation overlays in the RSI knowledge topic—Pi, Hermes Agent,
and OpenAI Codex—are not part of this citation bundle. The body-linked Karpathy
autoresearch project is represented by immutable repository metadata and its
README rather than a source-tree copy.

## Capture identity

- Capture completed: `2026-08-01T05:27:46Z`
  (`2026-07-31T22:27:46-07:00`).
- Source declarations: `sources.tsv`.
- Typed retained edges: `closure_edges.tsv`.
- Per-source acquisition and parse results: `manifest.tsv`.
- Every captured artifact and derivative digest: `artifact_inventory.tsv`.
- Tool and workflow receipt: `run-receipt.tsv`.

The acquisition workflow is `acquire.sh`. It records the digest of itself and
of `sources.tsv` in the run receipt. A normal run is idempotent for existing
non-empty captures; `--refresh` deliberately re-fetches mutable upstream
representations and therefore produces new digests.

## Representations and parsing

For sources with an arXiv identity, the preferred capture is the arXiv PDF,
with the arXiv abstract/license page retained as metadata. When arXiv exposes
an HTML paper representation, the workflow also captures it and uses the
semantic `ltx_bibitem` elements to produce structured citation records. PDF
text is extracted independently with Poppler in reading order, and the final
`References` or `Bibliography` section is retained as a raw fallback.

`outgoing_citations.tsv` is derived, not an authority-resolution result. It
contains `arxiv-html:ltx_bibitem` records when semantic arXiv HTML exists and
`plain-numbered` records when only an unambiguous numbered bibliography can be
split safely. Unnumbered PDF bibliographies remain available as raw reference
blocks rather than being guessed into inaccurate records.

The current capture contains:

- 47 successfully fetched primary nodes;
- 43 extracted raw bibliography sections;
- 1,905 structured outgoing citation records from 29 nodes;
- 231 captured or derived files totaling 184,793,419 bytes; and
- 55 retained typed edges in the bounded closure.

The four nodes without a conventional bibliography section are the Anthropic
position page, the Virginia Tech three-page Good reprint excerpt, the Karpathy
README, and the mirrored LessWrong essay.

## Source substitutions

When a locator named by the anchor was unavailable to the non-interactive
capture, the source identity was preserved and a documented representation was
used:

- AFlow and Anchored Self-Play use their arXiv versions after OpenReview denied
  direct PDF acquisition.
- PaperBench and RE-Bench use the immutable assets published by PMLR's official
  proceedings repository after stale proceedings PDF paths returned `404`.
- Good's 1965 article uses Virginia Tech's institutional three-page reprint.
- Yudkowsky's essay uses a GreaterWrong rendering because the original
  LessWrong page returned an automated-access challenge. The original locator
  remains recorded as the work's identity.
- Quality Diversity uses Frontiers' first-party PDF rather than its challenged
  full-HTML representation.

These substitutions change the captured representation, not the claimed work
identity. `sources.tsv` retains both the original locator and the acquisition
URL.

## Copyright and license boundary

There is no bundle-wide content license. Every paper, article, page, and README
remains subject to its upstream copyright and license. `manifest.tsv` records a
license-status value and a license-evidence locator for every node. The current
metadata identifies 36 captures with an explicit Creative Commons or arXiv
distribution notice and leaves 11 as `unknown`; `unknown` means no license was
inferred, not that reuse is unrestricted.

`licenses/` captures one canonical legal-code or distribution-policy document
for each detected license family: CC BY 4.0, CC BY-SA 4.0, CC BY-NC-SA 4.0,
CC BY-NC-ND 4.0, and arXiv's non-exclusive distribution license.
`license_assignments.tsv` maps each node to its detected status and evidence
locator. These reference texts do not assign a license where the upstream work
does not state one.

Virginia Tech identifies the Good reprint as `In Copyright`. The Karpathy
repository did not expose a conventional root `LICENSE` file at the pinned
commit, so its README capture remains license-unknown. Harp-authored manifests,
scripts, edge records, and provenance prose do not relicense the captured
upstream works.

## Integrity and use

Use `manifest.tsv` to select source nodes and `artifact_inventory.tsv` to
verify exact bytes before relying on a capture. Use `closure_edges.tsv` for the
bounded traversal; do not treat every row in `outgoing_citations.tsv` as an
admitted node. Extending the closure requires an explicit new depth or
retention rule, new source declarations, and a fresh receipt.

Harp's SQLite search also indexes the three text representations in
`evidence/rlm/` and the maintained RSI notes. Searchability makes the RLM
lineage available as a peer RSI anchor; it does not change the membership or
accounting of this Weng-rooted citation bundle.

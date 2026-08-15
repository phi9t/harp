# Harp provenance: DarwinX

## Identity and scope

This bundle supports Harp's technical review of *DarwinX: Evolving Agent
Harnesses Through Natural Selection*. The primary source is arXiv revision
`2608.07545v1`. The packet also captures *HarnessX: A Composable, Adaptive, and
Evolvable Agent Harness Foundry* at arXiv revision `2606.14249v1` because the
supplied review makes a material cross-system comparison that was not already
covered by Harp's evidence collection.

The user's 2026-08-14 review is preserved verbatim as
`artifacts/supplied_review.md`. It is provisional analysis and a research
agenda, not evidence for DarwinX's method, results, implementation, or
reproducibility. The maintained packet independently confirms, narrows,
contradicts, or leaves unresolved its material claims.

Meta-Harness and Darwin Gödel Machine evidence is not duplicated here. Harp
already has revision-pinned paper and implementation evidence for those
systems under `evidence/weng/`, `evidence/meta_harness/`, and
`evidence/implementations/`.

## Captured representations

For DarwinX and HarnessX, the bundle retains:

- the revision-pinned arXiv PDF;
- the revision-pinned semantic HTML;
- the revision-pinned abstract and license page;
- a dated arXiv Atom metadata response; and
- a deterministic `pdftotext -raw` sidecar.

The PDF is the source of record. Text sidecars provide stable local line
locators but do not supersede the PDF when extraction loses layout or equation
structure.

`manifest.tsv` records source identity, representation, URL, resolved version,
byte count, SHA-256 digest, license status, and claim-boundary note.
`artifact_inventory.tsv` records every captured or derived artifact.
`capture_receipt.tsv` records capture time, Harp base commit, tool identity,
and the acquisition-script digest. `acquire.sh` is the idempotent external
capture workflow; the supplied review remains a checked-in local input rather
than something the script reconstructs from private session state.

## Public implementation search

On 2026-08-14 America/Los_Angeles, the revision-pinned DarwinX semantic HTML
contained one external GitHub link:
`https://github.com/browser-use/browsercode`. The paper identifies BrowserCode
as the browser substrate used for WebArena-Infinity, not as the DarwinX
optimizer implementation. Searches for the exact paper title plus `GitHub` and
for the paper title plus `Monet` returned the arXiv paper and index pages but no
official DarwinX source repository or project page.

This is a dated negative finding, not proof that no non-public or later
implementation exists. The packet therefore makes paper-level method claims
but no released-code claims for DarwinX.

## Evidence boundary

The DarwinX paper supports its stated algorithm, experiment protocols,
author-reported results, appendices, and limitations. It does not independently
establish:

- the behavior of an unreleased optimizer implementation;
- the historical identity of the proprietary Monet harness;
- exact values or schedules for `β` and `δ`;
- proposer, analyzer, or reasoned-verifier prompts and complete model settings;
- the total evolution token, trajectory, API-cost, or wall-clock bill;
- causal effects of archive retention, parent selection, recombination, or any
  individual evolved skill; or
- benchmark reproduction by Harp.

Harp did not run Terminal-Bench 2.1, TerminalWorld, WebArena-Infinity, or
SWE-bench Verified for this packet. Quantitative statements remain
author-reported unless explicitly labeled as arithmetic recomputed from a
reported table.

## Copyright and licenses

Both arXiv abstract pages identify their papers as
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). The supplied review
is user-provided material; Harp records no separate upstream license for it.
There is no bundle-wide license, and this provenance record does not relicense
any artifact.

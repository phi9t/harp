# Harp provenance: EnvHarness

## Identity and scope

This bundle captures the primary preprint *EnvHarness: Awakening Static Worlds
for Agent Learning* at arXiv revision `2608.19880v1`. It supports a bounded
Harp system reading of learner-conditioned interventions over existing agent
environments.

The raw PDF is the source of record. The Atom response and abstract page bind
the title, revision, submission date, and CC BY 4.0 license. The text sidecar
is a deterministic reading aid; it does not supersede the PDF where extraction
loses layout, tables, or equations.

## Captured representations

- revision-pinned arXiv PDF;
- arXiv Atom metadata response;
- revision-pinned arXiv abstract and license page; and
- a `pdftotext -layout` sidecar.

`manifest.tsv` records source identity and public retrieval metadata.
`artifact_inventory.tsv` records the byte count and SHA-256 digest of every
captured representation. `run-receipt.tsv` binds the capture script, source
table, manifest, and inventory to the observed paper revision.

## Claim boundary

The paper supports its own method description, protocol, author-reported
measurements, and stated limitations. It does not independently establish:

- that its interface-preserving wrappers retain the semantic meaning or
  transfer value of each transformed task;
- that equal environment count implies matched rollout, token, designer-call,
  wall-clock, or environment-execution cost;
- that the scaling curve isolates environment wrapping from active
  learner-conditioned task selection;
- that five-rollout adaptive candidate screening estimates reliable causal
  effects or generalizes beyond the selected tasks;
- that the separate fixed-seed GRPO study establishes robust weight-level
  co-evolution; or
- that the system demonstrates recursive successor improvement.

The canonical system reading preserves those distinctions. It classifies the
main experiment as skill extraction and retrieval around a frozen policy and
the reinforcement-learning result as a separate, bounded experiment.

## Copyright and license

The captured arXiv abstract page identifies revision `2608.19880v1` as
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Harp-authored
provenance and analysis do not relicense any upstream artifact.

# Meta-Harness evidence provenance

This bundle separates four evidence identities:

- `META-HARNESS`: arXiv `2603.28052v1`, already captured under
  `evidence/weng/`;
- `META-HARNESS-SITE`: the first-party project page and its local asset
  closure as fetched on 2026-08-08;
- `META-HARNESS-REPO` and `META-HARNESS-TB2-ARTIFACT`: narrow snapshots of
  two pinned public Git repositories; and
- `META-HARNESS-TRAE-RUN`: one local proposal-interface experiment.

`site_manifest.tsv` records the URL, local path, capture timestamp, HTTP
`Last-Modified`, byte count, and SHA-256 of every page asset. The capture is
offline-complete for the page's CSS, data, JavaScript, images, and referenced
font files. Captured page bytes have not been rewritten.

The page metadata names “Conference on Language Modeling (COLM) 2026.” No
official venue or OpenReview record was located during this capture, so Harp
treats that wording as a dated first-party site claim, not independently
verified acceptance evidence.

The main repository is pinned at
`44b9942127847f7421db70d8c7e48407f09a3c70`. Its README describes the release
as cleaned paper code and says it was not tested beyond checking that it runs;
the snapshot is therefore implementation evidence, not an independent
reproduction. Its MIT license is preserved beside the snapshot.

The TerminalBench-2 artifact repository is pinned at
`57fefdb2ff84af3fd81b69d67814acbe69bd0743`. All five tracked files are
captured. No license file was present at that revision, which is recorded as
`not-present-at-pinned-revision	MISSING` rather than inferred.

The TRAE run preserves a normalized review surface plus one deterministic raw
archive. Its receipt and validation report state the claim ceiling: proposal
schema and filesystem boundaries were validated, all three candidates were
retained as interface-invalid, and no benchmark, paid model evaluation, or
held-out result was reproduced.

`source_test_findings.tsv` records the provider-free text-classification result
and the bounded dependency failures for TerminalBench-2 and experimental
Harbor. It is an environment receipt, not an upstream test artifact.

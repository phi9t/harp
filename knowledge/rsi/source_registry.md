---
id: recursive-self-improvement-source-registry
title: Recursive self-improvement - source registry guide
type: source-registry-guide
mode: DOMAIN ORIENTATION
status: active
created: 2026-07-31
updated: 2026-08-01
tags: [recursive-self-improvement, sources, provenance]
confidence: medium
---

# Source registry guide

Mode: `DOMAIN ORIENTATION`.

**EVIDENCE — local registry.** The authoritative claim registry is `sources/source_registry.tsv`. `vendored-inspected` means a checked-in capture was read for the stated claim ceiling; `fetched-local-inspected` means a pinned public-source snapshot was inspected and its immutable revision is registered in `evidence/implementations/manifest.tsv`; `vendored-uninspected` means the full capture is locally available but only source identity and graph connectivity have entered the synthesis; `vendored-partial` means the best captured representation is still incomplete, as with a publisher summary. Exact artifact paths, resolved versions, hashes, and parse status live in `../evidence/weng/manifest.tsv` and `../evidence/rlm/manifest.tsv`.

**INFERENCE — cohort policy.** `established` contains work published before 2026 in a checked venue or durable book/journal record. `accepted-frontier` requires an official 2026 venue page. `preprint-watchlist` contains arXiv work or venue claims not independently verified through official proceedings. Venue prestige is not used as evidence of impact, and this packet records no citation-count claims.

**EVIDENCE — anchor coverage.** This is a multi-anchor RSI topic. [WENG-HARNESS] owns the broad harness/self-improvement bibliography topology. The [RLM-PAPER] → [RLM-REPO] → [A1ZHANG-HARNESS-BLOG] lineage anchors recursive inference, context externalization, harness-level inductive bias, and post-training transfer. Each anchor owns only its own claims and citation lineage.

**INFERENCE — claim ceiling.** A `vendored-uninspected` row supports source identity and graph connectivity only; vendoring does not silently promote it into interpreted evidence. A fetched abstract supports author claims and abstract measurements, not implementation correctness, independent replication, or generalization beyond the named experiment.

**INFERENCE — implementation overlay.** Rows with depth `NA` and cohort `implementation-snapshot` are local source checkouts or pinned repository metadata added to make harness mechanisms concrete. They are excluded from the Weng-rooted closure, and their claim ceiling is present-day behavior at the recorded commit rather than historical capability, benchmark quality, or RSI efficacy.

**EVIDENCE. Meta-Harness tiers.** [META-HARNESS-SITE] is a dated first-party
page capture, [META-HARNESS-REPO] and [META-HARNESS-TB2-ARTIFACT] are pinned
implementation snapshots, and [META-HARNESS-TRAE-RUN] is a local
proposal-interface experiment. None inherits the paper's result authority.
“COLM 2026” remains a site claim because no official venue record was located;
the artifact's missing license remains explicit.

**MISSING.** Most 2026 preprints in Weng's bibliography have not been interpreted beyond registration even though their source representations are now vendored. The RLM paper's 51 parsed citations and the blog's 11-entry bibliography likewise have not been admitted automatically. They remain available for bounded later expansion without inventing their contents.

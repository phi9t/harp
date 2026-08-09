---
id: recursive-self-improvement-evidence-graph
title: Recursive self-improvement - evidence graph guide
type: evidence-graph-guide
mode: DOMAIN ORIENTATION
status: active
created: 2026-07-31
updated: 2026-08-01
tags: [recursive-self-improvement, evidence-graph, provenance]
confidence: medium
---

# Evidence graph guide

Mode: `DOMAIN ORIENTATION`.

**EVIDENCE — local graph.** The authoritative edge list is `sources/evidence_graph.tsv`. A `cites` edge proves that the source names the target. It does not prove endorsement, implementation reuse, or independent confirmation.

**INFERENCE — main lineages.** The graph has these useful paths:

```text
GOOD-1965 → WENG-HARNESS → practical full-stack RSI
GODEL-MACHINE → DGM → empirical, archive-based self-modification
QD-2016 → DGM ┐
FUNSEARCH → ALPHAEVOLVE ┴→ executable evaluator + diverse search
CONCRETE-SAFETY → WENG-REWARD → evaluator-integrity boundary
RLM-PAPER → RLM-REPO
          └→ A1ZHANG-HARNESS-BLOG → harness-induced transfer evidence
WENG-HARNESS ⇄ A1ZHANG-HARNESS-BLOG → complementary RSI anchors
META-HARNESS-SITE → META-HARNESS
META-HARNESS-REPO → META-HARNESS
META-HARNESS-TB2-ARTIFACT → META-HARNESS-REPO
META-HARNESS-TRAE-RUN → META-HARNESS-REPO
```

**INFERENCE — historical edge boundary.** The last relationship is conceptual rather than bibliographic. The RLM blog post-dates Weng's article and is represented by the typed `parallel-anchor` edge, not by a fabricated citation edge.

**INFERENCE. Graph reading rule.** `extends` means a source presents itself as advancing a predecessor's mechanism; `implements` means it realizes a named loop or representation; `evaluates` means it supplies an evaluation surface; `critiques` means it exposes a limitation; and `requires` means the target is a prerequisite for understanding the source's role in this map.

`reports-on` keeps a dated project page subordinate to the paper it discusses.
`instantiates` links a released artifact to the implementation it specializes.
`evaluates-interface` records a local compatibility experiment without
promoting it to benchmark or reproduction evidence.

**MISSING.** No graph edge demonstrates that the current public systems form one integrated, repeatedly self-improving successor pipeline.

---
id: deepseek-harness-source-registry
title: DeepSeek Harness source registry
type: source-registry
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [deepseek-harness, sources, provenance, cordis]
confidence: high
source_ids: [DEEPSEEK-HARNESS, CORDIS-PAPER, DSH-SMOKE]
---

# DeepSeek Harness source registry

Mode: `SOURCE REGISTRY`.

| Source ID | Source class | Local artifact | Upstream identity | Stability | Claim ceiling |
|-|-|-|-|-|-|
| `DEEPSEEK-HARNESS` | pinned public implementation snapshot | `evidence/implementations/deepseek_harness/snapshot/` plus revision files under `evidence/implementations/deepseek_harness/` | `https://github.com/deepseek-ai/deepseek-harness.git` at `47f943859bef60e4160492346772ded9b24f765a` | Git commit pinned; MIT license captured at `snapshot/LICENSE`; expanded narrow snapshot, not a full mirror | Present-day DSH docs, package surfaces, selected source behavior, config model, tool/session/sandbox/subagent/persistence design at the pinned commit; no benchmark reproduction or model-provider result |
| `CORDIS-PAPER` | pinned public paper artifact | `evidence/cordis_paper/artifacts/cordis-paper-v8.pdf`; text extraction at `evidence/cordis_paper/text/cordis-paper-v8.txt`; metadata at `evidence/cordis_paper/metadata/README.md` | `https://github.com/cordiverse/paper.git` at `948a07b369c62adb3b12e102458be5c18dfb69b9`, tag `v8` | Git tag/commit pinned; upstream license not found at captured revision and remains explicit in provenance | The paper's formal definitions and author claims about spatiotemporal composability, revertible effects, reactive coeffects, dynamic composition, and Cordis as implementation; no endorsement of DSH-specific product behavior unless DSH source also supports it |
| `DSH-SMOKE` | local keyless verification receipt | `evidence/deepseek_harness_study/README.md` | Local commands against `/tmp/harp-deepseek-harness` at `47f943859bef60e4160492346772ded9b24f765a`; recorded 2026-08-15 | Re-runnable only while dependencies and temp checkout remain available; receipt is local observation | Source identity, package-manager/runtime versions, script and package-surface observations, and no-key tool/composition checks; no paid model call, UI, benchmark, or platform sandbox guarantee |

## Acquisition notes

The DSH snapshot is treated as implementation evidence under Harp's public
implementation workflow. Captured files under
`evidence/implementations/deepseek_harness/snapshot/` are upstream bytes and
must not be edited for Harp prose.

The Cordis paper is a separate evidence packet rather than an implementation
snapshot. Its `artifact_inventory.tsv` records the PDF, extracted text, and
README sizes and SHA-256 digests. Harp uses the extracted text for citations,
but the PDF remains the captured paper artifact.

The local smoke receipt is intentionally narrow. It verifies mechanics that do
not require provider credentials: repository identity, package surfaces,
scripts, static config/examples, and local keyless execution or inspection
where available.

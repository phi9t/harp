---
id: self-improving-agents-survey-source-registry
title: Self-improving agents survey - source registry
type: source-registry
mode: SOURCE REGISTRY
status: active
created: 2026-08-09
updated: 2026-08-09
tags: [recursive-self-improvement, sources, provenance, claim-ceilings]
confidence: medium
---

# Self-improving agents survey source registry

Mode: `SOURCE REGISTRY`.

The authoritative structured rows are in `content/sources/source_registry.tsv`.
This page gives packet-local reading guidance for the four new source IDs.

| Source ID | Identity | Stability | Local artifacts | Claim ceiling |
|---|---|---|---|---|
| `SIMAS-PAPER` | arXiv `2607.13104v1`, `Self-Improvements in Modern Agentic Systems: A Survey` | Immutable arXiv version for the captured paper; arXiv record may later gain newer versions. | `evidence/self_improving_agents_survey/survey/metadata/`, `survey/artifacts/pdf/`, `survey/text/` | Survey taxonomy, author framing, abstract, formal model, and bibliography references. No independent result verification. |
| `SIMAS-SITE` | `https://selfimproving-agent.github.io/` captured on 2026-08-09 | Mutable web page, captured as dated bytes; same bytes match the pinned site source commit in this bundle. | `evidence/self_improving_agents_survey/site/index.html`, `site/static/images/fig-si-main-001.png`, `site/index.txt` | Dated hub claims, paper-library counts, one-hop link topology, and first-party overview figure. |
| `SIMAS-SITE-REPO` | GitHub Pages source repo at `d8af6607ced118351108670f823cd106649cb757` | Immutable Git commit. | `evidence/self_improving_agents_survey/repos/site_repo/` | Pinned source bytes and commit metadata for the hub page. |
| `SIMAS-AWESOME-REPO` | `selfimproving-agent/Awesome-Self-Improving-Agents` default branch observed at `57a1d89e5bafcd65db7feb51e809422700aeb48a` | Repository is mutable; captured README/LICENSE use the observed commit. | `evidence/self_improving_agents_survey/repos/awesome_repo/` | Repository identity, README framing, MIT license text, and update-list metadata. No full tree audit. |

## Capture inventory

`capture_manifest.tsv` records every fetched and derived artifact with URL,
path, observation time, bytes, status, SHA-256, and claim ceiling.
`artifact_inventory.tsv` records the digest of every file in the evidence
bundle. `link_inventory.tsv` records the one-hop hub links.

The one-hop link summary is:

| Link kind | Count |
|---|---:|
| arXiv | 245 |
| GitHub | 183 |
| OpenReview | 23 |
| publisher | 10 |
| project/community page | 4 |
| Hugging Face | 1 |
| same-site | 1 |

These are hub-observed links, not endorsed source identities. A linked paper or
repository becomes inspected Harp evidence only when promoted into a separate
packet or explicitly cited from a local artifact with its own claim ceiling.

## Cannot-prove boundaries

- The hub cannot prove that a linked method works as described.
- GitHub stars, issue counts, and default-branch state are dated observations.
- The survey's bibliography is not a license grant for linked PDFs, code, or
  dataset artifacts.
- The survey taxonomy does not decide whether a system satisfies Harp's
  recursive-improvement standard; it only classifies update target and signal.
- The extracted PDF text is a local reading aid. Raw PDF and HTML artifacts own
  source identity.

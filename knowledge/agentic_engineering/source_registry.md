---
id: agentic-engineering-source-registry
title: Agentic engineering source registry
type: source-registry
status: draft
created: 2026-08-14
updated: 2026-08-14
tags: [agentic-engineering, sources, provenance]
---

# Agentic engineering source registry

Mode: `SOURCE REGISTRY`.

| Source ID | Source class | Local artifact | Upstream identity | Stability | Claim ceiling |
|-|-|-|-|-|-|
| `AE-001` | dated public primary source | `evidence/agentic_engineering/wes_mckinney_agentic_engineering/article.html`; derived text at `article.txt` | `https://wesmckinney.com/blog/agentic-engineering-aug-2026/`; captured with HTTP 200 after redirect | Mutable web page, dated capture. HTML SHA-256 `e0f53a5d1d3400735583507f0f9fee6cfa8650dc8a30bc57b28bdd6a1b8ff4d3` | What the post says about Kenn's workflow, tools, constitution summary, and author-reported metrics |
| `AE-002` | dated public primary source | `evidence/agentic_engineering/clanker_constitution/page.html`; derived text at `page.txt` | `https://wesmckinney.com/blog/clanker-constitution/`; captured with HTTP 200 | Mutable web page, dated capture. HTML SHA-256 `bc40e370db5f72b04f204473cdd8c2ef7d4fff4daf7e6dba764a52c918d8481f` | Wes-hosted constitution text and stated CC BY 4.0 license notice |
| `AE-003` | dated public source mirror | `evidence/agentic_engineering/clanker_constitution/github.html`; derived text at `github.txt` | `https://github.com/kenn-io/constitution`; captured with HTTP 200 | Mutable GitHub page, dated capture. HTML SHA-256 `3cd39b7c258a16f9e573dafe54cbb942a862145bd5eb0a474f3b96edd04286e8` | Repository page visibility and public mirror identity only; not a pinned commit |

## Acquisition notes

Artifacts were fetched with `curl -L --fail` and HTTP headers were retained
next to each capture. Text files are derived for local reading and citation;
HTML files remain the captured upstream bytes.

The constitution source includes the statement: "Clanker Constitution (c) 2026
Kenn Software LLC. Licensed under CC BY 4.0. Canonical source:
https://github.com/kenn-io/constitution." Harp preserves that notice and does
not relicense captured upstream material.

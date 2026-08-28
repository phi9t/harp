---
id: rsi-standalone-consolidation
title: RSI knowledge migration standalone
type: migration-record
mode: MAINTENANCE
status: active
created: 2026-08-28
tags: [recursive-self-improvement, harness, migration, provenance]
confidence: high
---

# RSI knowledge migration standalone

Mode: `MAINTENANCE`.

Harp is the only maintained authoring location for recursive-self-improvement and
Harness knowledge. The frozen upstream source commit is
`861232a70beed5792c7f73ea00ee4cf4faebea7b`. This Harp consolidation began
from `022daa5a3471ffa85f64b2eb64c0304053e7c48d`.

The first standalone import already carried the source RSI product from commit
`d14d0c5d2f07bfcecebe300d047fbb99e8ee36db`. It deliberately rewrote the
embedded Atlas into the root `atlas/` application, moved structured contracts
into `content/`, and moved captured evidence into `evidence/`. Therefore,
this reconciliation imports the later knowledge changes rather than copying the
old directory layout back into Harp.

The adjacent `source_reconciliation.tsv` file records one row for every
frozen upstream `knowledge/rsi/` path. It distinguishes content
that Harp already carries, portable later changes that Harp merges, and
source-specific runtime material that needs a separate public-source packet.

Raw Lark captures, internal media, local investigation bundles, generated
build outputs, and source materializations remain outside this corpus.
When an internal source informs a retained claim, Harp records the source
identity, access boundary, revision, and claim ceiling without publishing its
raw body.

For the maintained reader entry point, start at
[[knowledge/rsi/rsi_index|Recursive self-improvement]].

## Reconciliation outcome

The frozen source contains 172 RSI paths. Nine are byte-identical with Harp.
The standalone import already relocated 81 paths into root Atlas, content, or
evidence locations. The remaining later source changes are reviewed at claim
level: portable additions strengthen the Harp technical spine, while
source-only execution and source-derived statements remain excluded rather than
becoming undeclared Harp dependencies.

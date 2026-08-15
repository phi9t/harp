---
id: agentic-engineering-index
title: Agentic engineering reference architecture
type: reference-architecture
status: draft
created: 2026-08-14
updated: 2026-08-14
tags: [agentic-engineering, harnesses, human-control, verification, constitution]
---

# Agentic engineering reference architecture

Mode: `REFERENCE ARCHITECTURE`.

This packet uses Wes McKinney's August 2026 Kenn posts as dated source
material for Harp's harness design work. It treats Kenn as a reference
architecture for production agentic engineering, not as a reproduced benchmark.

## Reader routes

- [Kenn reference architecture](kenn_reference_architecture.md)
- [Kenn tool stack investigation](tool_stack_investigation.md)
- [Source registry](source_registry.md)
- [Claim evidence ledger](claim_evidence_ledger.md)
- [Missing evidence](missing_evidence.md)

## Core framing

**INFERENCE — [AE-001], [AE-002].** The useful Harp design lesson is: scale
agent execution, not agent authority. Human operators own intent, architecture,
merge, and production authority; agents supply bounded execution bandwidth;
independent verification and durable knowledge keep that bandwidth from turning
into unreviewed code churn.

## Claim ceiling

This packet can support:

- source-backed summaries of what Wes McKinney says Kenn does;
- Harp-authored interpretation of those practices as harness design patterns;
- contributor guidance aligned with bounded agent authority; and
- comparison against Harp's local-first execution and verification design.

This packet cannot support:

- reproduced Kenn throughput, bug-rate, or cost claims;
- claims about Kenn's private implementation beyond the public posts;
- endorsement of external tools as dependencies for Harp; or
- a claim that human approval can be removed from production authority.

---
id: agentic-engineering-missing-evidence
title: Agentic engineering missing evidence
type: missing-evidence
status: draft
created: 2026-08-14
updated: 2026-08-14
tags: [agentic-engineering, missing-evidence]
---

# Agentic engineering missing evidence

Mode: `MISSING EVIDENCE`.

This packet is a reference architecture, not a reproduced study. These evidence
gaps bound its claims:

## Kenn production claims

The packet does not reproduce Kenn's reported throughput, bug rate, token
spend, or production quality. The public post is evidence that Wes McKinney
reported those claims, not independent evidence that they occurred.

Missing:

- raw PR counts;
- defect accounting;
- roborev findings and closure logs;
- token billing records;
- codebase sizes and change distributions; and
- independent audit of merged changes.

## Tool behavior

The packet captures public pages and GitHub API metadata for Forge, Kata,
Ghosthub, AgentsView, roborev, and Superpowers. It does not inspect or execute
their implementations.

Missing:

- source snapshots for each tool;
- command/interface contracts beyond public docs;
- trace examples;
- failure modes; and
- permission or authority models.

## Tool deployment

The packet does not prove how Kenn deploys the tools internally or how the
tools interact under real workload.

Missing:

- configured Kata federation topology;
- Forge workspace launch traces;
- Ghosthub session fleet traces;
- AgentsView session ingestion corpus;
- roborev review findings and closure logs; and
- integration data showing cross-tool causality.

## Constitution execution

The packet captures the public constitution text but does not prove how it is
loaded, enforced, or measured inside Kenn sessions.

Missing:

- agent launch configuration;
- per-repository instruction layering;
- model-family differences;
- violation examples; and
- evaluation showing behavioral lift from the constitution.

## Harp transfer

The packet proposes Harp guidance but does not prove that every Kenn practice
should transfer directly.

Missing:

- Harp-local trace analysis for repeated agent failures;
- A/B evaluation of instruction changes;
- measured impact on review burden;
- issue throughput under the Simplicity Gate; and
- evidence that durable docs improve future Harp agent behavior.

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

The packet does not inspect the implementation behavior of Forge, Kata,
Ghosthub, AgentsView, roborev, or Superpowers.

Missing:

- source snapshots for each tool;
- command/interface contracts;
- trace examples;
- failure modes; and
- permission or authority models.

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

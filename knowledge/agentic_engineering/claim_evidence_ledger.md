---
id: agentic-engineering-claim-ledger
title: Agentic engineering claim evidence ledger
type: claim-ledger
status: draft
created: 2026-08-14
updated: 2026-08-14
tags: [agentic-engineering, claims, evidence]
---

# Agentic engineering claim evidence ledger

Mode: `CLAIM LEDGER`.

## AE-C001: Kenn workflow is human-operator centered

- Class: `SOURCE CLAIM`
- Source: `AE-001`
- Locator: `evidence/agentic_engineering/wes_mckinney_agentic_engineering/article.txt`, lines containing "human-operator loops" and "the clankers ... are not in charge"
- Claim: McKinney rejects fully autonomous no-human-in-the-loop coding pipelines and says Kenn's loops are human-operator loops.
- Reproduction status: public source captured; private Kenn workflow not reproduced.

## AE-C002: Kenn workflow includes design, spec, review, implementation, verification, durable docs, and human merge

- Class: `SOURCE CLAIM`
- Source: `AE-001`
- Locator: `evidence/agentic_engineering/wes_mckinney_agentic_engineering/article.txt`, workflow bullet list under "Planning, Architecture, and... Caring about the Output"
- Claim: McKinney describes a process of human-involved design, second opinion, Superpowers spec, adversarial spec review, implementation planning, roborev verification, durable architecture documents, PR explanation, and human-owned merge.
- Reproduction status: public source captured; workflow execution not reproduced.

## AE-C003: Kenn treats verification as a significant compute budget

- Class: `SOURCE CLAIM`
- Source: `AE-001`
- Locator: `evidence/agentic_engineering/wes_mckinney_agentic_engineering/article.txt`, lines containing "hundreds of dollars in tokens bug-bashing"
- Claim: McKinney says Kenn may spend hundreds of dollars in tokens bug-bashing large changesets with roborev.
- Reproduction status: public source captured; token accounting not reproduced.

## AE-C004: Constitution has seven operating principles

- Class: `SOURCE CLAIM`
- Source: `AE-002`
- Locator: `evidence/agentic_engineering/clanker_constitution/page.txt`, section headings 1 through 7
- Claim: The constitution lists seven principles: honor the request, act with judgment, finish the job, protect existing work, verify reality, communicate for humans, and learn in the right place.
- Reproduction status: public source captured.

## AE-C005: Constitution is harness policy

- Class: `INFERENCE`
- Sources: `AE-001`, `AE-002`
- Claim: The constitution is best modeled as an agent operating contract that defines authority, completion, verification, communication, and memory policy, not merely as a prompt.
- Evidence that would weaken: Kenn treating the constitution only as optional prose with no operational effect.
- Evidence that would falsify: a source showing Kenn agents do not load or follow constitution-style instructions.

## AE-C006: Scale execution, not authority

- Class: `INFERENCE`
- Sources: `AE-001`, `AE-002`
- Claim: The strongest Harp design lesson is to scale agent execution while keeping human-owned intent, architecture, merge, and production authority explicit.
- Evidence that would weaken: production data showing unattended autonomous loops achieve comparable quality without human authority controls.
- Evidence that would falsify: a Kenn source saying agents own merge/release authority in the described workflow.

## AE-C007: Durable knowledge should be distilled from execution traces

- Class: `INFERENCE`
- Source: `AE-001`
- Claim: Specs and plans are execution scaffolding; lasting lessons should be distilled into living architecture documents, shared instructions, tests, code, and ADRs.
- Evidence that would weaken: repositories where retained raw plans are the primary successful long-term agent context.
- Evidence that would falsify: a source showing Kenn's production code depends on retained Superpowers specs/plans as durable authority.

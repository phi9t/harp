# Worked example: readable prose with an auditable contradiction

This example shows the complete route:

```text
main prose
  -> exact ledger claim
  -> exact source record and locator
```

Production documents keep the ledger in a separate file. Main prose and ledger
entries are shown together only to make the teaching route visible on one page.

## Main prose

**[EVIDENCE - EX-101](#ex-101-figure-label-reports-590).**
The figure labels the transferred result as 59.0%. This result has not been
independently reproduced here.

**[EVIDENCE - EX-102](#ex-102-nearby-prose-reports-595).**
The nearby prose reports 59.5%. This result has not been independently
reproduced here.

**[MISSING - EX-103](#ex-103-the-exact-value-is-unresolved).**
The exact value remains unresolved without an authoritative result artifact or
author correction.

**[INFERENCE - EX-104](#ex-104-rounding-could-explain-the-discrepancy).**
Rounding could explain the discrepancy, but the inspected source does not
establish that explanation.

## Ledger entries

## EX-101: Figure label reports 59.0

- Class: `EVIDENCE`
- Statement: Figure 4 labels the transferred result as 59.0%.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Example paper](source-registry-template.md#example-paper-darwin-godel-machine-paper)
- Locator: [Figure 4 extraction, line 461](../../evidence/weng/text/dgm.txt#L461)
- Scope: Figure 4 label.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: Direct visible figure label in the inspected source.
- Caveat: Nearby prose reports a different value.
- Relationship: `unresolved-with EX-102`

## EX-102: Nearby prose reports 59.5

- Class: `EVIDENCE`
- Statement: The prose near Figure 4 reports 59.5%.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [Example paper](source-registry-template.md#example-paper-darwin-godel-machine-paper)
- Locator: [Figure 4 discussion, line 517](../../evidence/weng/text/dgm.txt#L517)
- Scope: Prose adjacent to Figure 4.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: Direct source text.
- Caveat: Figure 4 labels the value as 59.0%.
- Relationship: `unresolved-with EX-101`

## EX-103: The exact value is unresolved

- Class: `MISSING`
- Statement: Available source materials do not establish whether 59.0% or
  59.5% is the authoritative result.
- Source: [EX-101](#ex-101-figure-label-reports-590) and
  [EX-102](#ex-102-nearby-prose-reports-595)
- Locator: [Figure 4 value, line 461](../../evidence/weng/text/dgm.txt#L461) and
  [nearby prose value, line 517](../../evidence/weng/text/dgm.txt#L517)
- Scope: Exact transferred result.
- Reproduction: Not independently reproduced here.
- Confidence: `high`
- Confidence basis: The inspected source contains both values.
- Caveat: Rounding may explain the difference, but the source does not say so.
- Resolves when: An authoritative result artifact or author correction settles
  the value.

## EX-104: Rounding could explain the discrepancy

- Class: `INFERENCE`
- Statement: Rounding could explain the 59.0% versus 59.5% discrepancy.
- Source: [EX-101](#ex-101-figure-label-reports-590) and
  [EX-102](#ex-102-nearby-prose-reports-595)
- Locator: [Figure 4 value, line 461](../../evidence/weng/text/dgm.txt#L461) and
  [nearby prose value, line 517](../../evidence/weng/text/dgm.txt#L517)
- Scope: The discrepancy between these two source statements only.
- Reproduction: Not applicable.
- Confidence: `low`
- Confidence basis: Plausible numerical explanation with no source support.
- Caveat: The values could instead reflect different runs, subsets, or an
  editorial error.
- Weakens if: The two values use different hidden accounting boundaries.
- Falsified by: An authoritative artifact shows both values are exact results
  from different runs or evaluation scopes.

## Dated observation pattern

For a mutable page without an immutable revision:

```markdown
- Class: `SOURCE CLAIM`
- Mode: `paraphrase`
- Source stability: `dated observation`
- Observed: `2026-08-08`
- Source: [Local dated capture](../../evidence/weng/metadata/dgm-arxiv.html)
- Locator: [Captured abstract description](../../evidence/weng/metadata/dgm-arxiv.html)
- Caveat: Refresh before presenting this as current.
```

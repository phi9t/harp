# Claim ledger template

Each heading is a stable claim target. Replace examples with your own inspected evidence.

## EX-001: License title identifies the version

- Class: `EVIDENCE`
- Statement: The captured license is titled Attribution-ShareAlike 4.0 International.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [[source-registry-template#EXAMPLE-LICENSE: SICP captured license|License registry]] and [[source-registry-template#EXAMPLE-CAPTURE: SICP capture record|Capture registry]]
- Locator: [[evidence/sicp/LICENSE.txt|Captured license]] ([exact line 1](../../evidence/sicp/LICENSE.txt#L1))
- Scope: The retained SICP capture only.
- Reproduction: Source wording inspected; no fresh remote verification or result reproduction.
- Confidence: `high`
- Confidence basis: Direct captured text for observations; the stated boundary for gaps.
- Caveat: This is an evidence-writing example, not a new source acquisition.

## EX-002: Capture record describes the edition

- Class: `SOURCE CLAIM`
- Statement: The provenance record describes the captured PDF as an unofficial Texinfo edition.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [[source-registry-template#EXAMPLE-LICENSE: SICP captured license|License registry]] and [[source-registry-template#EXAMPLE-CAPTURE: SICP capture record|Capture registry]]
- Locator: [[evidence/sicp/PROVENANCE.md|Capture provenance]] ([artifact identity](../../evidence/sicp/PROVENANCE.md#L3))
- Scope: The retained SICP capture only.
- Reproduction: Source wording inspected; no fresh remote verification or result reproduction.
- Confidence: `high`
- Confidence basis: Direct captured text for observations; the stated boundary for gaps.
- Caveat: This is an evidence-writing example, not a new source acquisition.

## EX-003: Separate capture metadata from interpretation

- Class: `INFERENCE`
- Statement: Keeping capture metadata separate makes later evidence review easier.
- Source: [[source-registry-template#EXAMPLE-LICENSE: SICP captured license|License registry]] and [[source-registry-template#EXAMPLE-CAPTURE: SICP capture record|Capture registry]]
- Locator: [[evidence/sicp/PROVENANCE.md|Capture provenance]] ([artifact identity](../../evidence/sicp/PROVENANCE.md#L3))
- Scope: The retained SICP capture only.
- Reproduction: Source wording inspected; no fresh remote verification or result reproduction.
- Confidence: `high`
- Confidence basis: Direct captured text for observations; the stated boundary for gaps.
- Caveat: This is an evidence-writing example, not a new source acquisition.
- Weakens if: Separate records make the source route harder to follow.
- Falsified by: Reader checks show the split prevents reliable source identification.

## EX-004: Current remote availability is unverified

- Class: `MISSING`
- Statement: This example does not establish current availability of the remote PDF.
- Source: [[source-registry-template#EXAMPLE-LICENSE: SICP captured license|License registry]] and [[source-registry-template#EXAMPLE-CAPTURE: SICP capture record|Capture registry]]
- Locator: [[evidence/sicp/PROVENANCE.md|Capture provenance]] ([artifact identity](../../evidence/sicp/PROVENANCE.md#L3))
- Scope: The retained SICP capture only.
- Reproduction: Source wording inspected; no fresh remote verification or result reproduction.
- Confidence: `high`
- Confidence basis: Direct captured text for observations; the stated boundary for gaps.
- Caveat: This is an evidence-writing example, not a new source acquisition.
- Resolves when: An explicitly requested fresh source check establishes availability.

## EX-005: License title quoted exactly

- Class: `EVIDENCE`
- Statement: The license title reads "Attribution-ShareAlike 4.0 International".
- Mode: `quote`
- Source stability: `pinned`
- Source: [[source-registry-template#EXAMPLE-LICENSE: SICP captured license|License registry]] and [[source-registry-template#EXAMPLE-CAPTURE: SICP capture record|Capture registry]]
- Locator: [[evidence/sicp/LICENSE.txt|Captured license]] ([exact line 1](../../evidence/sicp/LICENSE.txt#L1))
- Scope: The retained SICP capture only.
- Reproduction: Source wording inspected; no fresh remote verification or result reproduction.
- Confidence: `high`
- Confidence basis: Direct captured text for observations; the stated boundary for gaps.
- Caveat: This is an evidence-writing example, not a new source acquisition.

## EX-006: Two records name the same license

- Class: `EVIDENCE`
- Statement: The license and provenance record both name Attribution-ShareAlike 4.0 International.
- Mode: `paraphrase`
- Source stability: `pinned`
- Source: [License registry](source-registry-template.md#example-license-sicp-captured-license) and [capture registry](source-registry-template.md#example-capture-sicp-capture-record)
- Locator: [[evidence/sicp/LICENSE.txt|Captured license]] ([exact line 1](../../evidence/sicp/LICENSE.txt#L1)) and [[evidence/sicp/PROVENANCE.md|Capture provenance]] ([artifact identity](../../evidence/sicp/PROVENANCE.md#L3))
- Scope: The retained SICP capture only.
- Reproduction: Source wording inspected; no fresh remote verification or result reproduction.
- Confidence: `high`
- Confidence basis: Direct captured text for observations; the stated boundary for gaps.
- Caveat: This is an evidence-writing example, not a new source acquisition.

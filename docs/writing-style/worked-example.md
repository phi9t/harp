# Worked example: educational capture metadata

Main prose and ledger entries are shown together only to make the teaching route
visible on one page. Production documents keep their ledger separately.

## Main prose

**[[#EX-101: License title identifies the version|EVIDENCE - EX-101]].**
The captured license is titled Attribution-ShareAlike 4.0 International. No new result reproduction is claimed.

**[[#EX-102: Capture record describes the edition|SOURCE CLAIM - EX-102]].**
The provenance record describes the captured PDF as an unofficial Texinfo edition. No new result reproduction is claimed.

**[[#EX-103: Current remote availability is unverified|MISSING - EX-103]].**
This example does not establish current availability of the remote PDF. No new result reproduction is claimed.

**[[#EX-104: Separate capture metadata from interpretation|INFERENCE - EX-104]].**
Keeping capture metadata separate makes later evidence review easier. No new result reproduction is claimed.

## Ledger entries

## EX-101: License title identifies the version

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

## EX-102: Capture record describes the edition

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

## EX-103: Current remote availability is unverified

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

## EX-104: Separate capture metadata from interpretation

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

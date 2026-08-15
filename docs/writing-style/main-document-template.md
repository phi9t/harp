# Main technical document template

Use this file as a starting point. Replace `EX-*` IDs with packet-specific
stable IDs whose headings exist in the packet claim ledger.

## Purpose

State the reader's decision or understanding target. Do not make a material
technical claim in this section unless it has a claim block.

## Mechanism

**[[claim-ledger-template#EX-001: Released selector reads the shallow score|EVIDENCE - EX-001]].**
State one bounded source-backed mechanism. Keep all sentences in this block
under the same source route and accounting boundary.

**[[claim-ledger-template#EX-003: The result supports harness improvement|INFERENCE - EX-003]].**
State the interpretation separately. Include the concrete uncertainty boundary
when it affects the reader's conclusion.

## Reported results

**[[claim-ledger-template#EX-002: Authors report the benchmark result|SOURCE CLAIM - EX-002]].**
State the author's reported result with benchmark, task count, metric, model,
and reproduction status.

## Direct quotation

**[[claim-ledger-template#EX-005: Repository describes empirical validation|EVIDENCE - EX-005]].**
The repository calls DGM a system that "empirically validates each change
using coding benchmarks." The wording was inspected; the behavior was not
independently reproduced here.

## Evidence gap

**[[claim-ledger-template#EX-004: Next-cycle evidence is missing|MISSING - EX-004]].**
Name the missing experiment, artifact, or source needed for the stronger
conclusion.

## Legacy compatibility

New managed prose uses the native wiki markers above. This transitional
Markdown marker remains only while the style validator accepts both forms:

**[EVIDENCE - EX-001](claim-ledger-template.md#ex-001-released-selector-reads-the-shallow-score).**
Use the native marker when authoring a new claim block.

## Conclusion

State the strongest justified conclusion. Link any material conclusion to its
ledger entry rather than relying on citations attached only to earlier
premises.

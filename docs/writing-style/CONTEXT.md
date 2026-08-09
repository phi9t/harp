# Credible technical documentation glossary

This context defines the language used to write readable technical prose whose
material claims remain independently auditable.

## Language

**Material claim**:
A falsifiable statement whose error would change a reader's technical
conclusion, decision, implementation, or quantitative understanding.

A material claim requires an explicit evidence class and a source route. It
belongs in the claim ledger when a reviewer may need to audit its exact
wording, locator, scope, or caveat.

Examples:

- "The paper reports 50.0% on its 200-task SWE-bench subset."
- "The released selector reads the shallow Polyglot aggregate."
- "A matched next-cycle experiment is required to establish successor
  improvement."

Non-examples:

- section introductions;
- navigation instructions;
- statements about what the current document will explain.

**Source-derived statement**:
A quote, paraphrase, numerical result, mechanism description, or author
interpretation whose authority comes from an identified source.

A source-derived statement must preserve:

- source identity;
- locator;
- whether the wording is quoted or paraphrased;
- the source's accounting boundary;
- any material caveat or contradiction; and
- whether the result was independently reproduced.

Writers must not silently convert a source-derived statement into an
independently established fact.

**Evidence class**:
The relationship between a statement and its support. The core classes are
`EVIDENCE`, `SOURCE CLAIM`, `INFERENCE`, and `MISSING`.

**EVIDENCE**:
A bounded statement about inspectable source content or directly observed
behavior that is accurately supported by an inspected source. `EVIDENCE` does
not imply that the behavior or result described by the source was independently
reproduced.

Use `EVIDENCE` when the statement's subject is what the artifact contains or
what an inspection directly establishes:

- "Figure 4 contains the label 59.0%."
- "`DGM_outer.py` reads `overall_performance` during parent selection."
- "The paper's method section describes an archive of generated agents."

_Avoid_: using `EVIDENCE` to mean independently reproduced.

**SOURCE CLAIM**:
An author's reported result, interpretation, intended behavior, or design
assertion. The documentation preserves the attribution rather than adopting
the statement as independently established fact.

Use `SOURCE CLAIM` when the statement's subject is the system, result, or world
described by the source:

- "The authors report that the transferred agent achieves 59.5%."
- "The repository authors assert that DGM is a self-improving system."
- "The paper's authors argue that the improvements generalize across models."

The same source passage can support either class depending on the bounded
statement. "The results section reports 50.0%" is `EVIDENCE` about source
content. "The authors report that DGM achieves 50.0%" is a `SOURCE CLAIM` about
the reported result.

_Avoid_: `CLAIM`, which is overloaded with material claim and claim entry.

**Rendering mode**:
Whether a source-derived statement is presented as a direct `quote` or a
`paraphrase`.

Rendering mode is separate from evidence class and source stability. Both
quoted and paraphrased author assertions use `SOURCE CLAIM`; both quoted and
paraphrased observations about source content may use `EVIDENCE`.

Every canonical claim entry records `Mode: quote` or `Mode: paraphrase` for a
source-derived statement. Direct quotations preserve the source wording and
quotation boundary. Paraphrases preserve meaning, scope, attribution, and
caveats without quotation marks.

**Faithful paraphrase**:
A paraphrase that a reasonable reviewer can map to the cited source without the
documentation adding a new causal link, generalization, comparison, judgment,
prediction, recommendation, certainty level, or reconciliation.

A faithful paraphrase keeps its source-derived evidence class. A paraphrase
becomes `INFERENCE` when it adds any of the following:

- a causal consequence not stated by the source;
- a generalization beyond the named setup;
- a comparison or synthesis across sources;
- an explanation of why a result occurred;
- a prediction about another environment;
- a normative recommendation;
- stronger certainty than the source uses; or
- a reconciliation of conflicting source statements.

Source-backed premises and the inference derived from them belong in separate
claim blocks with separate ledger entries.

**Multi-source evidence**:
One `EVIDENCE` claim block supported by two or more sources that each
independently establish the same bounded statement.

The canonical claim entry lists every source and locator separately. Removing
any one source must not change the claim's meaning.

If the statement exists only after combining facts from multiple sources, the
statement is `INFERENCE`, not multi-source `EVIDENCE`.

_Avoid_: attaching several citations to a synthesis and labeling the synthesis
as direct evidence.

**Source contradiction**:
Two or more source-derived statements that cannot all be true under the same
scope, version, metric, or accounting boundary.

Contradicting statements remain separate `EVIDENCE` or `SOURCE CLAIM` entries.
The ledger links them with an explicit relationship such as:

- `contradicts`;
- `unresolved-with`;
- `supersedes`; or
- `narrows`.

Documentation must not silently select, average, or normalize conflicting
values.

**Reconciliation**:
A documentation-authored explanation that resolves or bounds a source
contradiction. Reconciliation is `INFERENCE` unless a new source directly
settles the conflict.

When available evidence cannot settle the conflict, add a `MISSING` entry that
names the artifact or correction required.

Example:

```markdown
**[EVIDENCE - DGM-029A](claim-ledger.md#dgm-029a).**
Figure 4 labels the transferred result as 59.0%.

**[EVIDENCE - DGM-029B](claim-ledger.md#dgm-029b).**
The nearby prose reports 59.5%.

**[MISSING - DGM-029C](claim-ledger.md#dgm-029c).**
The exact result remains unresolved without an authoritative result artifact or
author correction.
```

**Source locator**:
The most precise stable address that lets a reviewer find the support for one
claim without searching the whole source.

Use the first available locator in this hierarchy:

1. exact local file and line anchor for source code;
2. table, figure, algorithm, section, or appendix for a paper;
3. exact local artifact plus field, key, event ID, or record identity for
   structured data and logs; or
4. document-level citation only when the source has no finer stable structure.

Examples:

```markdown
- Locator: [`DGM_outer.py:63`](../../evidence/implementations/dgm/snapshot/DGM_outer.py#L63)
- Locator: [DGM Figure 4 value](../../evidence/weng/text/dgm.txt#L461)
- Locator: [`README.md:14`](../../evidence/implementations/dgm/snapshot/README.md#L14)
```

_Avoid_: a paper or repository root link when a stable local passage, table,
field, or code line is available.

**Pinned source**:
A source whose identity is fixed by version, commit, publication revision,
capture date plus digest, or another immutable identifier.

When a source does not support stable line anchors:

- link the local captured artifact;
- record its immutable identity;
- use a semantic locator such as section, heading, table, figure, page, JSON
  key, or event ID; and
- preserve the capture digest when available.

Do not fabricate line precision from unstable extraction.

**Dated observation**:
A source-stability status for a bounded statement read from a mutable source
that has no immutable identity. The claim entry still records `Mode: quote` or
`Mode: paraphrase`; it separately records `Source stability: dated observation`,
the observation date, source URL or local capture, and semantic locator.

A dated observation is weaker than a pinned source and must not be presented as
current fact after its observation date without refresh.

Example:

```markdown
- Mode: `paraphrase`
- Source stability: `dated observation`
- Observed: `2026-08-08`
- Locator: [Captured abstract description](../../evidence/weng/metadata/dgm-arxiv.html)
```

**INFERENCE**:
A conclusion derived by the documentation author from one or more source
materials. An inference names its supporting evidence and states the evidence
that would weaken or falsify it when that boundary is not obvious.

A material inference always records:

- `Weakens if`: evidence that would reduce confidence without fully overturning
  the claim; and
- `Falsified by`: an observation or experiment that would make the claim false.

Minor local interpretations do not require formal weakening or falsifying
conditions unless they drive a technical decision or conclusion.

_Avoid_: an inference written as an inevitable conclusion with no stated path
for correction.

**MISSING**:
Evidence required for a stronger conclusion but absent from the inspected
corpus. `MISSING` records the gap instead of filling it with a favorable
assumption.

**Reproduction status**:
The independent axis that records whether a result or behavior was rerun,
recomputed, replayed, or otherwise verified outside the source author's report.

Reproduction status never changes a statement's evidence class silently. A
paper result can be `SOURCE CLAIM` and independently reproduced; a code reading
can be `EVIDENCE` and not executed.

Reproduction status always appears in the canonical claim entry. It also
appears in main prose when the claim is:

- quantitative;
- benchmark-based;
- safety-critical; or
- likely to be mistaken for documentation-authored verification.

Ordinary source-code observations may omit inline reproduction wording when the
evidence label and clickable source route already make the boundary clear.

Example:

```markdown
**[SOURCE CLAIM - DGM-020](claim-ledger.md#dgm-020).**
The authors report 50.0% on the 200-task SWE-bench subset. This result has not
been independently reproduced here.
```

**Source route**:
The shortest auditable path from reader-facing prose to the evidence that owns
the statement.

A source route consists of:

1. a compact claim-ID link in the main document;
2. the exact stable claim entry in the separate ledger; and
3. a clickable source locator owned by that claim entry.

Every step in the source route must be clickable from the document that names
it. A reader should be able to open the claim entry from the main prose, then
open the cited source artifact from the claim entry.

A main document may also provide a direct source link when it materially helps
code or artifact navigation. The direct link supplements the ledger route; it
does not replace the claim entry.

**Claim ledger**:
A separate audit document with one heading-based canonical entry per material
claim. It records stable claim ID, evidence class, claim wording, source
identity, locator, accounting boundary, reproduction status, confidence, and
caveat.

The ledger audits the main document. It does not replace the explanation.

**Citation entry**:
The stable, directly linkable ledger record for one material claim. It owns the
claim ID and source route used by the main document.

_Avoid_: citations that name a source ID but cannot open either the exact claim
entry or the underlying local source.

**Confidence**:
The documentation author's assessment of how strongly the available evidence
supports the bounded claim.

Confidence always appears in the canonical claim entry with a short confidence
basis. Main prose includes confidence only when uncertainty materially affects
interpretation, and then states the concrete boundary rather than a bare
`high`, `medium`, or `low` badge.

Example ledger fields:

```markdown
- Confidence: `medium`
- Confidence basis: Direct mechanism evidence; missing next-cycle experiment.
```

Example main prose:

```markdown
**[INFERENCE - DGM-050](claim-ledger.md#dgm-050).**
The result supports harness improvement, but not successor improvement because
the paper does not compare parent and child as producers of later accepted
children.
```

**Canonical claim entry**:
A heading-based ledger section with a stable claim ID and short descriptive
title.

Each canonical claim entry records:

- evidence class;
- exact claim statement;
- clickable source identity;
- clickable locator;
- accounting or version scope;
- reproduction status;
- confidence;
- caveat or contradiction; and
- weakening and falsifying evidence for every material inference.

Example:

See the complete, validated
[EX-001 canonical entry](claim-ledger-template.md#ex-001-released-selector-reads-the-shallow-score).

The heading is the canonical link target. A generated summary table may provide
a scan-friendly index, but the table is not the authoritative claim record.

**Claim block**:
One paragraph or tightly related bullet group governed by one evidence class,
claim ID, source route, and accounting boundary.

A paragraph claim block begins with a clickable label:

```markdown
**[EVIDENCE - DGM-017](claim-evidence-crosswalk.md#dgm-017).**
The released Polyglot path stores the expanded score separately. Parent
selection continues to read the shallow aggregate.
```

For a bullet-group claim, place the label in its own paragraph immediately
before one Markdown list. The label and list form one claim block:

```markdown
**[EVIDENCE - DGM-017](claim-evidence-crosswalk.md#dgm-017).**
- Parent selection reads the shallow aggregate.
- Expanded evaluation is stored separately.
```

Do not label every sentence when the sentences form one claim. Split the block
when:

- the evidence class changes;
- the source route changes;
- the accounting or version boundary changes;
- the prose adds an inference; or
- a caveat can change the reader's interpretation.

_Avoid_: one long paragraph that begins as source-backed evidence and ends with
an unlabeled interpretation.

Each claim block has exactly one evidence class. Source-backed facts and
documentation-authored interpretation belong in separate blocks, even when the
inference immediately follows the evidence.

Example:

```markdown
**[EVIDENCE - DGM-017](claim-evidence-crosswalk.md#dgm-017).**
The released selector reads `overall_performance`. The expanded result is
stored as `overall_performance_deep`.

**[INFERENCE - DGM-018](claim-evidence-crosswalk.md#dgm-018).**
The released search therefore allocates later work using the shallow score,
which differs from the paper's described 50-task search pressure.
```

Unlabeled transition prose is allowed only when it makes no falsifiable
technical assertion.

# Credible technical documentation style guide

Use this style for research summaries, technical articles, implementation deep
dives, design reviews, and other documents whose material claims must remain
auditable.

The writing system has three layers:

1. the **main document** teaches and argues;
2. the **claim ledger** audits material claims; and
3. the **source registry** fixes source identity and claim ceiling.

The reader-facing document should stay readable. Audit detail belongs in the
ledger, but every material claim must have a clickable route to it.

## Core rule

A material claim is a falsifiable statement whose error would change a
reader's technical conclusion, decision, implementation, or quantitative
understanding.

Every material claim must have:

- one evidence class;
- one stable claim ID;
- one clickable claim-ledger entry;
- at least one precise source locator or an explicit evidence gap; and
- an accounting, version, or applicability boundary when the boundary affects
  interpretation.

## Evidence classes

Use exactly these classes:

### `EVIDENCE`

A bounded statement about inspectable source content or directly observed
behavior that is supported by an inspected source.

`EVIDENCE` does not mean independently reproduced. It means the documentation
accurately reports what an inspection establishes within the stated scope.

```markdown
**[EVIDENCE - EX-001](claim-ledger-template.md#ex-001-released-selector-reads-the-shallow-score).**
The released selector reads `overall_performance`.
```

Choose `EVIDENCE` when the statement's subject is the artifact or direct
observation: "Figure 4 contains 59.0%" or "`DGM_outer.py` reads
`overall_performance`."

### `SOURCE CLAIM`

An author's reported result, interpretation, intended behavior, or design
assertion.

```markdown
**[SOURCE CLAIM - EX-002](claim-ledger-template.md#ex-002-authors-report-the-benchmark-result).**
The authors report 50.0% on their 200-task benchmark subset. This result has
not been independently reproduced here.
```

Choose `SOURCE CLAIM` when the statement's subject is the reported system,
result, or world: "The authors report that the system achieves 50.0%." The
source-content statement "the results section reports 50.0%" is instead
`EVIDENCE`.

Do not use the shorter class `CLAIM`. It is overloaded with material claim and
claim entry.

### `INFERENCE`

A conclusion derived by the documentation author from one or more sources.

```markdown
**[INFERENCE - EX-003](claim-ledger-template.md#ex-003-the-result-supports-harness-improvement).**
The result supports harness improvement, but not successor improvement because
the study does not compare parent and child as producers of later accepted
children.
```

Material inferences must record what would weaken and falsify them.

### `MISSING`

Evidence required for a stronger conclusion but absent from the inspected
corpus.

```markdown
**[MISSING - EX-004](claim-ledger-template.md#ex-004-next-cycle-evidence-is-missing).**
The corpus contains no matched next-cycle experiment.
```

`MISSING` prevents a favorable assumption from filling an evidence gap.

## Claim blocks

Label coherent claim blocks, not every sentence.

A claim block is one paragraph or tightly related bullet group with:

- one evidence class;
- one claim ID;
- one source route; and
- one accounting boundary.

For a bullet group, put the claim label in its own paragraph immediately before
one Markdown list. The label and adjacent list form one claim block.

Split the block when:

- the evidence class changes;
- the source route changes;
- the scope, version, or metric changes;
- the prose adds interpretation; or
- a caveat changes the conclusion.

Do not start with source-backed evidence and end with an unlabeled inference.

## Main document

The main document should:

- explain mechanisms before significance;
- use claim blocks only for material assertions;
- keep navigation and non-falsifiable transitions unlabeled;
- link every material claim ID to the exact ledger heading;
- state inline reproduction status for quantitative, benchmark, safety-critical,
  or easily misattributed claims;
- state concrete uncertainty when it affects interpretation; and
- avoid duplicating full provenance metadata.

Use the [main-document template](main-document-template.md).

## Claim ledger

The claim ledger owns the canonical audit record.

Use one `##` heading per material claim:

```markdown
## EX-001: Released selector reads the shallow score
```

The heading is the stable link target. Each entry records:

- class;
- statement;
- rendering mode;
- source stability and observation date when applicable;
- source;
- locator;
- scope;
- reproduction status;
- confidence and confidence basis;
- caveat or contradiction;
- relationships to other claims; and
- weakening and falsifying evidence for material inferences.

A generated summary table may help scanning, but it does not own claim detail.

Use the [claim-ledger template](claim-ledger-template.md).

## Source registry

The source registry owns source identity and claim ceiling.

For each source, record:

- stable source ID;
- source class;
- clickable local artifact;
- immutable identity, or observation date for mutable sources;
- what the source can prove; and
- what it cannot prove.

Use the [source-registry template](source-registry-template.md).

## Clickable source route

The required path is:

```text
main claim block
  -> exact claim-ledger heading
  -> exact local source artifact and locator
```

Every step must be clickable.

A direct source link may supplement the ledger link when it helps code or
artifact navigation. It does not replace the ledger entry.

## Quotation and paraphrase

`quote` and `paraphrase` are rendering modes, not evidence classes.

A paraphrase remains source-derived only when it preserves:

- meaning;
- scope;
- attribution;
- certainty;
- accounting boundary; and
- caveats.

It becomes `INFERENCE` when it adds:

- a new causal link;
- a generalization;
- a cross-source synthesis;
- an explanation of why;
- a prediction;
- a recommendation;
- stronger certainty; or
- a reconciliation.

Source stability is independent of rendering mode. A mutable unpinned source
uses `Source stability: dated observation` while its mode remains `quote` or
`paraphrase`.

## Multiple sources

One `EVIDENCE` block may cite several sources only when each source
independently supports the same bounded statement.

If the statement exists only after combining sources, it is `INFERENCE`.

## Contradictions

Keep conflicting source statements as separate claim entries.

Link them using:

- `contradicts`;
- `unresolved-with`;
- `supersedes`; or
- `narrows`.

Use one `Relationship` field per edge. Repeat the field when one claim relates
to several claims. `contradicts` and `unresolved-with` must appear on both
linked entries, and no claim may relate to itself.

Do not silently choose, average, or normalize conflicting values.

A documentation-authored reconciliation is `INFERENCE`. An unresolved conflict
is `MISSING`.

See the [worked example](worked-example.md).

## Source locators

Use the most precise stable locator available:

1. local file and line anchor for code;
2. table, figure, algorithm, section, or appendix for papers;
3. artifact field, key, event ID, or record identity for structured data; or
4. document-level citation only when no finer locator exists.

Sources without stable line anchors require:

- a local captured artifact;
- immutable identity such as version, commit, or digest; and
- a semantic locator.

Mutable unpinned sources have `dated observation` source stability and an
observation date. Do not invent line precision from unstable extraction.

## Reproduction status

Reproduction status is separate from evidence class.

It always appears in the ledger. It appears inline for:

- quantitative claims;
- benchmark results;
- safety-critical claims; and
- claims likely to be mistaken for our own verification.

## Confidence

Confidence is ledger-first.

Record:

```markdown
- Confidence: `medium`
- Confidence basis: Direct mechanism evidence; missing next-cycle experiment.
```

In main prose, state the concrete uncertainty boundary instead of showing a
bare confidence badge.

## Review

Before publishing, use the [review checklist](review-checklist.md).

The glossary and decision rationale are in:

- [credible documentation glossary](CONTEXT.md);
- [ADR 0001](../adr/0001-source-backed-evidence-and-clickable-claim-routes.md).

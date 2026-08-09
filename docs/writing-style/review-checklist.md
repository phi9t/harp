# Credible documentation review checklist

## Main document

- Does each material claim have a stable claim ID?
- Does each claim ID link to the exact ledger heading?
- Does each claim block contain exactly one evidence class?
- Are `EVIDENCE`, `SOURCE CLAIM`, `INFERENCE`, and `MISSING` used correctly?
- Are source-backed premises separated from interpretation?
- Are quantitative, benchmark, and safety-critical claims accompanied by inline
  reproduction status?
- Are accounting, version, and applicability boundaries visible where needed?
- Does the conclusion have its own claim route instead of borrowing citations
  from earlier premises?

## Claim ledger

- Does every material claim have one heading-based canonical entry?
- Are claim IDs unique and stable?
- Is rendering mode recorded for each source-derived statement as exactly
  `quote` or `paraphrase`?
- Is source stability recorded separately as `pinned` or `dated observation`?
- Does each dated observation include an observation date?
- Is every source identity clickable?
- Is every locator clickable and semantically precise?
- Is scope explicit?
- Is reproduction status explicit?
- Does confidence include a concrete basis?
- Does every material `INFERENCE` include `Weakens if` and `Falsified by`?
- Are contradictions separate claims with explicit relationships?
- Are symmetric contradiction relationships recorded on both entries?
- Does each relationship target another existing claim?
- Are unresolved contradictions represented as `MISSING`?

## Source registry

- Does every cited source have one stable source ID?
- Is the local artifact clickable?
- Is immutable identity recorded?
- If the source is mutable, is it a dated observation?
- Does the entry state what the source can prove?
- Does the entry state what the source cannot prove?
- Is the locator as precise as the source permits?

## Paraphrase audit

- Does the paraphrase preserve attribution, scope, certainty, and caveats?
- Did it add a causal link?
- Did it generalize beyond the source?
- Did it synthesize multiple sources?
- Did it explain why, predict, recommend, or reconcile?
- If yes to any of the last five questions, is the added statement a separate
  `INFERENCE` block?

## Readability

- Does the document explain the mechanism before significance?
- Can the main prose be read without opening the ledger?
- Can a skeptical reviewer audit any material claim in two clicks?
- Are dense provenance details kept out of the main narrative?
- Are headings, tables, equations, and examples used only when they reduce
  reader effort?
- Is promotional or vague language removed?

## Final verification

- Open every main claim link.
- Open every source link from the ledger.
- Verify local anchors resolve.
- Check claim IDs for duplicates.
- Check ledger entries for required fields.
- Check that generated summary tables, if any, match heading-based entries.
- Run the repository's documentation and link validators.

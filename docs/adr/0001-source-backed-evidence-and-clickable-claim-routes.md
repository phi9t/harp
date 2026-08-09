# Use source-backed evidence labels and clickable claim routes

Technical documents use `EVIDENCE` for bounded statements supported by an
inspected source, not only for independently reproduced results. Reproduction
status remains a separate field. Every material claim in reader-facing prose
links to a stable claim-ledger entry, and that entry links to the exact local
source artifact or source record. A direct source link may supplement this
route when it helps navigation, but it does not replace the ledger entry. This
keeps the main document readable while making the complete provenance route
clickable and auditable. Canonical claim entries use stable claim-ID headings;
summary tables may be generated for scanning but do not own claim detail.
Reader-facing prose labels coherent claim blocks rather than every sentence;
blocks split whenever evidence class, source route, accounting boundary, or
interpretation changes. Each block has exactly one evidence class, so an
evidence citation never appears to certify a later inference.

The evidence class for an author's reported result, interpretation, intended
behavior, or design assertion is `SOURCE CLAIM`. The shorter `CLAIM` label is
not used because it conflicts with the broader terms material claim and claim
entry.

The statement's subject distinguishes these two classes. A statement about
inspectable artifact content, such as "Figure 4 contains 59.0%," is
`EVIDENCE`. A statement about the reported system or result, such as "the
authors report that the system achieves 59.0%," is `SOURCE CLAIM`.

Direct quotation and paraphrase are rendering modes rather than evidence
classes. The claim ledger records which mode was used, so readers can
distinguish source wording from documentation wording without expanding the
class vocabulary. Source stability is a separate field: mutable unpinned
sources are `dated observation`s while their rendering mode remains `quote` or
`paraphrase`.

A paraphrase keeps a source-derived class only when it adds no new causal link,
scope, comparison, judgment, prediction, recommendation, certainty, or
reconciliation. Any such addition is a separate `INFERENCE` claim block and
ledger entry.

One `EVIDENCE` block may cite several sources only when each source
independently supports the same bounded statement. A statement created by
combining sources is `INFERENCE`.

Conflicting source statements remain separate linked claims. Their ledger
entries record relationships such as `contradicts`, `unresolved-with`,
`supersedes`, or `narrows`. Documentation-authored reconciliation is
`INFERENCE`; an unresolved conflict is `MISSING`.

Each claim uses the most precise stable locator available: code line, paper
table/figure/algorithm/section, or structured artifact field. Document-level
links are a fallback, not the default.

Sources without stable line anchors are pinned by version, commit, date, or
digest and use semantic locators. Mutable unpinned sources are recorded as
dated observations. Documentation does not invent line precision from unstable
extraction.

Reproduction status is mandatory in the ledger and appears inline for
quantitative, benchmark, safety-critical, or easily misattributed claims.

Confidence is ledger-first and includes a concrete confidence basis. Main prose
states the uncertainty boundary only when it changes interpretation; it does
not show bare confidence badges by default.

Every material `INFERENCE` claim entry records evidence that would weaken it and
evidence that would falsify it. Minor local interpretations need those fields
only when they drive a technical decision.

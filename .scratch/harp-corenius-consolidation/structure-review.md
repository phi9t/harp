# Consolidation structure review

Reviewed integration: 9ac9ce75 on local master, based on 84fc9bec.
This review proposes follow-up organization; it does not authorize or perform
additional moves.

## Findings

The Atlas and RSI article archive are removed. The remaining top-level layout
still reflects two product histories: Harp's runtime and study tooling share
crates/harp, while the imported runtime/controller retain their own workspace
under corenius. The complete import is intentional: its manifest and Git
ancestry preserve every branch and dirty capture before further restructuring.

| Surface | Current ownership | Recommended follow-up |
| --- | --- | --- |
| Agent runtime and control | crates/harp-*, crates/harp/src/context_control, corenius/terminal_agent and controller | Compare the actual runtime/controller contracts and select one maintained entry point per operation before combining implementations. |
| Math, proofs and education | knowledge topic packets, formalization/lean, content contracts, labs and tools/textbook_pdf | Consider experimental/ as a study home, but migrate a complete useful slice with its registrations and checks. Preserve Crouzeix as maintained proof work. |
| Generated publication | content/generated/corpus.json and ignored publications/crouzeix_textbook | Keep structured corpus output separate from publication Markdown; neither is a prose authority. |
| Evidence | evidence plus captured Corenius artifacts | Preserve exact bytes, source locators and license records. Directory tidiness does not justify rewriting captures. |
| Current and historical engineering guidance | root AGENTS, maintained docs and imported effort records | Keep one current contributor policy and label inherited proposals as history. |

## Separate dependencies before bulk moves

The main Harp library exports context_control alongside corpus, sources and
crouzeix_textbook. Moving only Markdown into experimental/ would leave the
runtime coupled to proof/publication code and test prerequisites. A useful
first structural slice is to separate the study compiler/publication API and its
verification selection from runtime work, retaining the current CLI contracts.
The root verification selector already supports changed-surface checks; preserve
that behavior rather than making every runtime change require Lean or a browser.

The imported workspace pins Rust 1.95 and its SQLite binding differs from the
root workspace. Keep its locked build boundary until a focused unification
change proves terminal-agent and controller behavior. PostgreSQL/Temporal remain
parked; restructuring does not activate their services.

## A bounded experimental directory

The owner's proposed experimental/ directory can group math, proof, SICP and
other educational work. Its name should not imply that preserved proofs or
education may be discarded. Start with one topic, update the canonical packet
registration and root-qualified links, and prove its reader, source verification,
Lean receipt and PDF publication still work before moving other topics.

Crouzeix paths are validated in Rust publication/receipt code and the PDF input
collector. Evidence locators, Lean module routes, publication pointers and the
import map must move together where applicable. Keep captured evidence immutable
and regenerate derived artifacts; do not treat a directory rename as sufficient
verification.

## Landing boundary

The consolidation preserves the original Corenius work and the existing dirty
Harp work. All 63 primary paths are recoverable in snapshot 85fdb248. After
reconciliation and verification, local master fast-forwarded to 9ac9ce75.
The primary working tree is clean. No push or source-checkout retirement occurred.
The proposed structural changes above remain follow-up design work.

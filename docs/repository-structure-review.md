# Repository structure review

This note reviews the repository as it exists today. It separates current facts
from recommendations, and it treats any future move as optional follow-up rather
than a required migration. The goal is to give a realistic recap of where Harp's
runtime, imported Corenius source, preserved mathematics, education, evidence,
and local planning records currently live.

## Current facts

### 1. The repository has two active product stories at the top level

The root [README](../README.md), [docs/README.md](README.md), and
[knowledge/harp_knowledge_home.md](../knowledge/harp_knowledge_home.md) all
describe the same repository from different entry points:

- [README.md](../README.md) is the product overview and the fastest way to find
  the CLI, durable runtime, knowledge library, and retained study areas.
- [docs/README.md](README.md) is the software and contributor manual.
- [knowledge/harp_knowledge_home.md](../knowledge/harp_knowledge_home.md) is the
  maintained prose entry point for readers using the repository as an Obsidian
  vault or Markdown library.

That split is intentional, but it means a new reader has to infer which top
level is for "product runtime", which is for "maintained prose", and which is
for "historical design and integration context".

### 2. The Rust CLI is current and lives in the root workspace

The root Cargo workspace in [Cargo.toml](../Cargo.toml) owns the current Harp
CLI and supporting libraries:

- [crates/harp/](../crates/harp) is the main CLI and owns corpus validation,
  search, source verification, durable CLI entry points, and Crouzeix textbook
  publication logic.
- [crates/harp-runtime/](../crates/harp-runtime) and the other root crates are
  current Rust libraries, not imported historical source.
- [labs/sicp-evaluator/](../labs/sicp-evaluator) is also part of the Rust
  workspace, so the "labs" label does not mean "throwaway" by itself.

There is no single `src/` directory at the repository root. The maintained Rust
surface is intentionally crate-based.

### 3. Runtime source spans the integration package and imported tree

The current supported coding runtime is the Python + SQLite path described in
[docs/durable-runtime.md](durable-runtime.md) and
[docs/corenius-integration.md](corenius-integration.md). In the filesystem,
that surface is split:

- [harp_durable/](../harp_durable) is the Harp-facing package and CLI wrapper.
- [harp_durable/workflows.py](../harp_durable/workflows.py) imports
  `controller.durable` from the imported Corenius tree.
- [corenius/controller/durable/README.md](../corenius/controller/durable/README.md)
  documents the underlying replay runtime and Insula/cgroup machinery.

So the repository does not have one isolated "Python runtime" directory. The
selected runtime spans Harp integration code and the retained imported engine.
Both follow Harp governance and serve the same durable authority.

### 4. Imported Corenius source, history, and provenance are intentionally retained in place

The imported source lives under [corenius/](../corenius). Its provenance is not
implicit:

- [corenius/HARP_IMPORT.json](../corenius/HARP_IMPORT.json) records the imported
  source commit, branch tips, worktree captures, and retained file identities.
- [docs/corenius-integration.md](corenius-integration.md) explains what was kept
  active, what was retired, and what remains as preserved reference material.
- [scripts/verify_corenius_import.py](../scripts/verify_corenius_import.py)
  checks the preserved import against retained Git objects.
- [docs/import-map.tsv](import-map.tsv) records retired Atlas and RSI surfaces
  alongside their disposition.

This means `corenius/` is not just "old code" and not just "vendor". It is a
preserved imported subsystem with active runtime dependencies, retired surfaces,
and explicit provenance.

### 5. The current canonical boundaries are real and mostly consistent

The best current statement of ownership is still
[docs/product-contract.md](product-contract.md) together with the root
[AGENTS.md](../AGENTS.md):

- [knowledge/](../knowledge) is the authority for maintained technical prose and
  educational text.
- [content/](../content) is structured machine-readable data and derived
  contracts, including [content/generated/corpus.json](../content/generated/corpus.json).
- [evidence/](../evidence) holds captured source bytes, manifests, provenance,
  licenses, and narrow snapshots.
- [formalization/lean/](../formalization/lean) holds the shared Lean libraries.
- [tools/textbook_pdf/](../tools/textbook_pdf) owns the PDF-building toolchain
  for the Crouzeix publication workflow.

This boundary is coherent in policy even though the implementation spans several
directories.

### 6. Mathematics, Crouzeix, and proofs are retained as maintained work, not archive material

The repository still treats the mathematics and proof program as active:

- [knowledge/crouzeix_conjecture/](../knowledge/crouzeix_conjecture) and
  [knowledge/crouzeix_textbook/](../knowledge/crouzeix_textbook) are maintained
  prose packets.
- [formalization/lean/Crouzeix/](../formalization/lean/Crouzeix),
  [formalization/lean/CrouzeixConjecture/](../formalization/lean/CrouzeixConjecture),
  and [formalization/lean/CrouzeixTextbook/](../formalization/lean/CrouzeixTextbook)
  hold the corresponding Lean developments.
- [content/crouzeix_textbook/](../content/crouzeix_textbook) contains the
  textbook's machine-readable coverage and exercise contracts.
- [labs/crouzeix_proof_reproduction/](../labs/crouzeix_proof_reproduction)
  contains proof-reproduction and validation tooling that still supports the
  surrounding program.
- [evidence/crouzeix_conjecture/](../evidence/crouzeix_conjecture) contains the
  source receipts and preserved evidence bundle.

The naming is uneven, but the actual ownership is not ambiguous: this material
is retained and maintained.

### 7. SICP and education remain first-class, but they are spread across prose, evidence, labs, and assets

The education surface is also distributed:

- [knowledge/sicp/](../knowledge/sicp) is the maintained SICP prose packet.
- [evidence/sicp/](../evidence/sicp) preserves the PDF and provenance bundle.
- [content/sicp/](../content/sicp) contains machine-readable source and coverage
  ledgers.
- [labs/sicp-evaluator/](../labs/sicp-evaluator) is the executable evaluator
  companion.
- [assets/](../assets) includes shared course assets referenced by some teaching
  material.

This is a legitimate split by role, but it makes "where SICP lives" harder to
answer in one sentence.

### 8. PDF publication is present even though no tracked `publications/` tree exists today

There is no tracked `publications/` directory in this checkout.
Instead:

- [.gitignore](../.gitignore) explicitly ignores `/publications/crouzeix_textbook/`.
- [tools/textbook_pdf/README.md](../tools/textbook_pdf/README.md) documents the
  working-draft PDF build flow.
- [content/crouzeix_textbook/](../content/crouzeix_textbook) and
  [formalization/lean/CrouzeixTextbook/](../formalization/lean/CrouzeixTextbook)
  are part of that publication pipeline.

So publication is retained, but its outputs are intentionally not part of the
tracked repository surface.

### 9. Atlas is removed, and the RSI article-reproduction product is retired

The current repo state matches the retirement language in the main docs:

- [README.md](../README.md) says Atlas and the offline HTML export are retired.
- [knowledge/article_survey.md](../knowledge/article_survey.md) is the brief
  maintained survey that remains.
- [docs/import-map.tsv](import-map.tsv) records retired Atlas and RSI article
  surfaces rather than pretending they still exist in-tree.

That is a substantive simplification. The older article-reproduction product is
not part of the current maintained layout.

### 10. The local tracker is real repository state

The current issue and planning tracker is local Markdown, not an external
service:

- [docs/agents/issue-tracker.md](agents/issue-tracker.md) defines the contract.
- [.scratch/](../.scratch) contains feature specs, tickets, and review records.
- Existing examples include
  [.scratch/harp-corenius-consolidation/](../.scratch/harp-corenius-consolidation)
  and [.scratch/harp-runtime-integration/](../.scratch/harp-runtime-integration).

That makes `.scratch/` operationally important even though its name still reads
like temporary workspace.

## Actual navigation and duplication problems

### 1. Runtime ownership is harder to read than runtime behavior

The current runtime story spans [crates/harp/](../crates/harp),
[harp_durable/](../harp_durable), and [corenius/](../corenius). The behavior is
documented, but the ownership boundary is not obvious from the tree alone. A
reader can easily mistake `corenius/` for a frozen import even though Harp's
selected Python runtime still depends on it.

### 2. "Maintained" and "experimental" names do not line up with actual importance

[labs/](../labs) contains active companions such as
[labs/crouzeix_proof_reproduction/](../labs/crouzeix_proof_reproduction) and the
workspace member [labs/sicp-evaluator/](../labs/sicp-evaluator). Meanwhile the
authoritative proof and education material is split across [knowledge/](../knowledge),
[formalization/](../formalization), [content/](../content), and [evidence/](../evidence).
The result is accurate by function, but not intuitive by name.

### 3. Readers have several entry points and no single repository map

[README.md](../README.md), [docs/README.md](README.md),
[knowledge/harp_knowledge_home.md](../knowledge/harp_knowledge_home.md), and
[docs/corenius-integration.md](corenius-integration.md) are all useful, but they
serve different audiences. There is not yet one concise, current "if you want X,
go to Y" map for repository structure itself.

### 4. Historical planning material sits near current guidance

[docs/superpowers/](../docs/superpowers) and [corenius/docs/](../corenius/docs)
contain valuable history, but they add visual noise when someone is trying to
identify the current supported surface. The repo does document that these are
historical or imported contexts, yet the tree still presents them alongside live
guidance.

### 5. Publication and proof dependencies cross directory boundaries

The Crouzeix textbook flow depends on prose, Lean, content contracts, evidence,
and PDF tooling at once. A directory move that looks cosmetic would affect
root-qualified wikilinks, Lean module discovery, generated corpus checks, PDF
input discovery, and ignored publication outputs. The layout is therefore more
coupled than the top-level names suggest.

## Recommendations

These are proposals only. None of them should be read as a required migration.

### 1. Keep the current canonical ownership model

The existing policy boundary is good enough to preserve:

- `knowledge/` for maintained prose
- `content/` for machine-readable contracts and derived output
- `evidence/` for captured bytes and licenses
- `formalization/lean/` for shared Lean libraries
- `corenius/` for imported source and provenance

That boundary already matches the repository's actual contracts better than a
large rename would.

### 2. Prefer a small navigation fix before any directory move

The smallest coherent next step would be a maintained repository map that points
people to the current surfaces without moving them. This review can serve as the
source inventory for that future map. A navigation document is lower risk than a
tree-wide rename because it does not disturb links, receipts, verification, Lean
cache expectations, or ignored publication outputs.

### 3. Treat `experimental/` as optional and narrow, not as a blanket new home

The repository currently has no `experimental/` directory. If
the project wants one, it should be treated as a proposal to test on one bounded
slice, not as an umbrella move for all math, proof, and SICP material.

A full move of the maintained math and education program into `experimental/`
would be risky for several reasons:

- root-qualified knowledge links would need coordinated updates
- Lean module routes and cache expectations under [formalization/lean/](../formalization/lean)
  would need review
- textbook PDF source discovery under [tools/textbook_pdf/](../tools/textbook_pdf)
  would need coordinated changes
- corpus and content checks tied to [content/generated/corpus.json](../content/generated/corpus.json)
  and registered knowledge paths would need regeneration and verification
- the name `experimental` could falsely imply that retained proofs, SICP, or
  textbook materials are provisional or expendable

If `experimental/` is pursued at all, the safest pilot would be a companion-code
surface such as selected material from [labs/](../labs), not the canonical
knowledge, evidence, or Lean authorities.

### 4. Keep preserved imports and preserved evidence visibly separate from current product code

The current review suggests preserving two distinctions that are already mostly
working:

- keep [corenius/](../corenius) visibly identifiable as imported preserved source
  with active selected dependencies
- keep [evidence/](../evidence) visibly identifiable as byte-preserving source
  capture rather than "content to tidy up"

Those separations reduce accidental damage to provenance.

Keep the current authority boundaries and make the runtime split easier to
navigate. Consider an `experimental/` move only for a bounded companion surface,
with coordinated checks for links, Lean routes, cache discipline, corpus
generation and PDF publication.

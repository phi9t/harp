# Shared mathematical reading editions

Approved direction: the owner's request to spec and execute follows the proposed shared PDF/web design. This specification uses the existing reader, renderer, publication CLI and PDF build interfaces as the test boundaries.

## Problem statement

The committed textbook is absent from mainline. Its feature branch has diverged, and its PDF exporter describes a particular dirty checkout. Readers need an attractive, truthful book in print and in the browser, with usable links to Lean evidence.

## Solution

Integrate the completed Wave 4 baseline and PDF tooling onto current mainline, preserving newer platform behavior. Both formats use canonical Markdown, stable chapter/theorem identifiers, and the same formal correspondence. The website is the browsing edition; PDFs are the sustained-reading edition. Neither is a separate prose authority.

## User stories

1. As an ML researcher, I can start at linear algebra and follow prerequisites to the proof chapters.
2. As a reader, I see complete mathematical statements and hypotheses without opening a disclosure.
3. As a reader, I distinguish proof from motivation, history and ML analogy.
4. As a Lean user, I can open the exact theorem/provider source and see the correspondence boundary.
5. As a desktop reader, I can navigate a chapter without losing the place in its argument.
6. As a mobile reader, I can read prose without sideways page scrolling.
7. As a keyboard user, I can reach chapter navigation, section links and evidence disclosures with visible focus.
8. As a reader following a link, I can address a theorem and return using browser history.
9. As a print reader, I get clear chapter openings, balanced equations and legible source notes.
10. As a PDF reader, I can use contents links, bookmarks and accompanying relative source links.
11. As an auditor, I see the actual edition inputs and never mistake a draft for whole-book verification.
12. As a contributor, I can build/test from a clean checkout without another local repository or private chapter edits.
13. As the maintainer, I keep existing Atlas, proof evidence, releases and Pages behavior.

## Editorial design

White paper #ffffff, dark ink #202b38, links #285ea8, secondary text #586575, rules #dde2e8 and evidence accents #376b59. Color never carries status alone. Use existing Charter/Iowan/Georgia reading faces with Avenir/Segoe/Arial navigation and KaTeX math; record actual PDF font environment. Serif chapter titles and restrained utility text belong to the book. Avoid dashboard cards, oversized status counters, gradients, decorative animation or a new global theme.

Web prose targets 64-72 characters per line, 18-20px body text and 1.65-1.75 line height. Desktop chapter/section navigation sits outside the reading measure. On narrow screens it becomes an accessible disclosure. The theorem statement and proof remain expanded; source metadata may collapse. A compact proof-reference area carries Lean source and prerequisite links using existing canonical data, without inventing statuses or duplicating theorem text.

PDFs retain the 7.5 by 10 inch page, serif text, quiet chapter openings, unboxed mathematical calculations, separate Lean code styling and artifact-tagged running headers. Matrix displays use square brackets. Do not globally restyle set braces or function arguments.

## Implementation decisions

Preserve current Atlas routing and canonical reader. Add textbook-specific presentation using the existing document identity, not text-sniffed proof assertions. Canonical prose renders once. Show Lean/evidence links already supplied by validated content. Do not promise PDF downloads until verified distributable assets exist.

PDF source discovery must be explicit and independent of import-time Git HEAD discovery. Validate actual input files and the staged candidate when preparing a commit. Record revision and per-file hashes; release mode rejects missing sources, symlinks, fallback and unresolved references. Pure rendering tests must not load the real book.

Pin runtime dependencies and document supported build commands. Local rendering need not be byte-identical across machines. Never claim reproducibility beyond the recorded environment. Public artifact distribution requires a separate license/notice audit.

## Testing decisions

Reuse the public publication CLI, canonical renderer, React reader interactions, offline PDF build and artifact checker. Test behavior rather than CSS source strings. Add regression cases before fixes for provenance, input discovery and source containment. Test navigation, disclosures, accessible labels and preserved theorem text through the rendered website.

Inspect 390px, 768px and 1440px viewports, 200% zoom, keyboard traversal and long formulas. Normal text contrast must meet 4.5:1; focus must be visible; only individual long equations may scroll. Inspect representative final PDF pages and run all-page geometry/link/hash checks. Review the final candidate independently along Standards and Spec axes. Full repository verification remains required before landing.

## Out of scope

Completing unfinished Chapter 1/13 work or Wave 5, changing terminal proof claims, replacing current Atlas, installing background services, hydrating Lean caches, adding a repository-wide license, publishing PDF release assets, or force-pushing.

## Execution contract

Follow tracker.org and the classified committed input inventory. The mainline landing and ordinary GitHub push were requested; execute them only after fresh final-candidate checks. Stop for missing authority, cache preconditions, protected-branch restrictions or overlap with preserved user edits.

# Atlas redesign implementation plan

## Approved direction

Build a research-first Atlas for a new technical reader, with deeper reading
available through topic navigation and search. The owner approved the four
areas and delegated aesthetic decisions: RSI research, durable execution,
mathematics and Lean, and evaluation and evidence. Supporting study includes
SICP, agentic engineering, and implementation studies.

Use white #FFFFFF, slate #F5F7FA, ink #202B38, secondary text #586575,
blue #285EA8, and evidence green #376B59. Use Avenir Next for navigation and
headings, Charter for articles, and system monospace for code, with local
fallbacks. No external fonts, decorative counters, or animated backgrounds.
The signature is a small evidence margin with provenance and explicit limits,
not a confidence score presented as a proof certificate.

## Architecture and scope

Keep canonical technical prose in knowledge/harp_knowledge_home.md and
knowledge/rsi/concepts/durable-execution.md. Derive the home and topic pages
from the compiled orientation document. TypeScript owns navigation and display
only. Existing hash routes, diagnostics, lessons, system comparisons, source
folds, and the operational snapshot remain reachable. Do not alter theorem
statements, captured artifacts, the corpus schema, or execution behavior.

## Implementation

- [ ] Add home and four topic routes in atlas/src/app/routes.ts. Test empty
  routes, unknown routes, topic round trips, and preserved legacy links in
  routes.test.ts. The new default is #home, not a Weng chapter.
- [ ] Reshape knowledge/harp_knowledge_home.md into four explicit reading
  areas with source-backed introductions and links. Explain Harp's executor
  in the existing durable-execution concept, bounded by ADR 0002 and current
  agent workflow documentation. Regenerate the corpus with mise run build
  followed by .build/harp-target/size/harp build.
- [ ] Add ResearchHome.tsx using the canonical orientation sections. Replace
  the eleven-way header in AtlasApp.tsx with topic navigation and a secondary
  library/tools menu. Add a local title/tag search to KnowledgeHome.tsx.
  Test actual clicks, search results, empty results, and preserved routes.
- [ ] Restyle tokens.css and the reader rules in layout.css. Add a scoped
  research.css for home, topic navigation, and evidence presentation. Give
  articles readable measures, modest headings, scrollable code/tables/math,
  preserved diagram whitespace, visible focus, and mobile layouts.
- [ ] Improve ChapterReader.tsx contents navigation and provenance disclosure.
  Test section focus, original HTML receipts, MathML, and code whitespace.
- [ ] Inspect the rendered home, RSI chapter, execution reading, Crouzeix
  status, and mathematical foundations on desktop and mobile. Check keyboard
  navigation, reload, overflow, and browser accessibility diagnostics.
- [ ] Run focused Atlas lint/types/tests and static export checks. Review the
  diff independently for standards and technical scope. Regenerate corpus and
  export together, refresh import receipt last, then freeze the candidate.
- [ ] Verify the warm Lean cache without hydration, run mise run verify, and
  commit the coherent redesign. Preserve unrelated primary-checkout changes.
  Do not push or change the deployed GitHub Page without an explicit push.

## Verification boundaries

Tests use existing public interfaces: route parse/format, the rendered React
reader, and the decoded standalone export. No source-text or CSS snapshot
assertions substitute for rendering. Browser checks cover wide equations and
schematics, while theorem claims continue to rely on the canonical status
documents and their revision-bound receipts. A local Lean build is a local
compile result, not a fresh upstream or hermetic certification.

## Follow-ups

Live execution monitoring, new proof results, new research claims, and public
hosting of captured evidence are outside this presentation change.

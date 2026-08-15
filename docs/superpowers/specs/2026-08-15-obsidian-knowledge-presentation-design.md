# Obsidian-native Harp knowledge presentation design

**Status:** Approved for implementation planning
**Date:** 2026-08-15
**Audience:** Harp readers and maintainers using Obsidian and the offline Atlas
**Primary surface:** repository-root Obsidian vault; reader-facing prose remains
under `knowledge/`

## 1. Objective

Make Harp a first-class Obsidian knowledge base without creating a second
documentation authority or weakening the repository's evidence discipline.

The implementation will:

1. vendor and pin the full MIT-licensed
   [`kepano/obsidian-skills`](https://github.com/kepano/obsidian-skills)
   repository;
2. install its five skills into the local TRAE skill runtime:
   `obsidian-markdown`, `obsidian-bases`, `json-canvas`, `obsidian-cli`, and
   `defuddle`;
3. make every managed internal knowledge reference an Obsidian wikilink;
4. render and validate wikilinks, heading links, embeds, and callouts in the
   Rust corpus compiler and offline Atlas;
5. add a human-oriented vault home, packet/task navigation, a Base, and a
   Canvas; and
6. standardize the frontmatter that makes managed knowledge discoverable by
   Obsidian properties and Bases.

The desired result is not a prettier file tree. A reader must be able to begin
with a question, follow human-readable links to a responsible document, open
the exact claim and its evidence boundary, and continue into related packets
through backlinks, tags, Bases, and the Canvas.

## 2. Authority model and non-goals

### 2.1 Repository-root vault

The Harp repository root is the Obsidian vault root. `knowledge/` remains the
only authority for technical prose. The root-vault choice permits native,
portable wikilinks to:

- managed prose in `knowledge/`;
- immutable captured evidence in `evidence/`;
- structured registries and contracts in `content/`;
- public source snapshots in `evidence/implementations/`; and
- relevant local runnable supplements in `labs/`.

The implementation must not create a `knowledge/`-only vault that needs
symlinks, copied evidence, or relative-path exceptions to reach the evidence
layer.

### 2.2 Canonical versus machine-local state

The committed repository will contain only portable vault assets:

```text
knowledge/
  harp_knowledge_home.md
  obsidian/
    harp_knowledge.base
    harp_knowledge_map.canvas
    README.md
tools/
  obsidian/
    profile/
    apply_profile.py
```

The user-specific `.obsidian/` directory remains ignored. It may contain
workspace tabs, recent notes, device geometry, installed plugins, and other
machine-local state. `tools/obsidian/profile/` is a reviewed, sanitized source
profile; `apply_profile.py` copies only its declared portable files into a
chosen vault's `.obsidian/` directory. It never reads, commits, or overwrites
personal workspace state unless passed an explicit replacement flag.

### 2.3 Non-goals

This work does not:

- change the technical conclusions, evidence classes, or claim IDs in Harp;
- rewrite captured upstream artifacts for Obsidian syntax or styling;
- install community Obsidian plugins, themes, sync, publish, or remote
  services;
- make Obsidian the source of truth for generated Atlas output;
- require the Obsidian desktop application or its CLI in CI;
- add Markdown beneath `content/`;
- index ignored research investigations or personal notes; or
- claim that an Obsidian link, graph edge, Base row, or Canvas edge is
  evidence.

## 3. Upstream skill and vendor contract

### 3.1 Pinned upstream snapshot

Vendor the complete upstream repository at:

```text
remote:   https://github.com/kepano/obsidian-skills.git
revision: a1dc48e68138490d522c04cbf5822214c6eb1202
license:  MIT
```

Use the source ID `OBSIDIAN-SKILLS` and preserve every repository file,
including `.claude-plugin/`, `skills/`, `README.md`, and `LICENSE`, under:

```text
evidence/implementations/obsidian_skills/
  REMOTE
  REVISION
  LICENSE
  LICENSE_STATUS
  snapshot/
```

Add a digest row for every tracked snapshot file to
`evidence/implementations/manifest.tsv`. This follows the existing pinned
implementation-snapshot verifier rather than introducing an unverified
vendor format. The source snapshot documents the syntax and user-facing
workflows adopted by Harp; it is not an authority for Harp's evidence claims.

### 3.2 Local TRAE installation

Install each upstream directory through the supported skill installer at the
pinned revision:

```text
skills/obsidian-markdown
skills/obsidian-bases
skills/json-canvas
skills/obsidian-cli
skills/defuddle
```

The installation destination is `~/.trae/skills/`. It is a user-machine setup
step, is not committed to Harp, and must be verified by checking that each
destination contains `SKILL.md`. The final handoff tells the operator to
restart TRAE CLI before relying on the installed skills.

## 4. Obsidian document contract

### 4.1 Metadata baseline

Every tracked Markdown document under a managed `knowledge/` root will have
YAML frontmatter with these required properties:

```yaml
---
id: stable-kebab-case-id
title: Human-readable title
type: index | chapter | concept | system-reading | technical-deep-dive | \
  learning-lesson | source-card | source-registry | claim-evidence-ledger | \
  missing-evidence-ledger | benchmark-guide | reference-architecture | \
  research-atlas | glossary | maintenance | template
status: active | reference | archived | draft
tags: [lowercase, hierarchical-tags-allowed]
confidence: high | medium | low
---
```

Existing metadata survives when compatible. The migration adds optional fields
only when the existing file already has a clear owner:

```yaml
mode: DOMAIN ORIENTATION | TECHNICAL REVIEW | TECHNICAL DEEP DIVE
source_ids: [SOURCE-ID]
coverage_keys: [coverage-key]
canonical: knowledge/path/to/canonical.md
```

The migration does not invent publication dates, source identities, claim
confidence, or local deployment status. Existing `created` and `updated`
fields are retained; dates are optional for older documents. New presentation
assets use their actual creation date.

The Rust corpus exposes metadata for every compiled canonical document. Atlas
and the Base consume that projection; they do not scrape frontmatter
independently. Search indexes the original Markdown text as today, including
frontmatter only if it is intentionally useful to retrieval.

### 4.2 Human prose and evidence construct

Adopt the Nova Vision robust-training split:

1. reader-facing indexes, chapters, system readings, and deep dives explain
   the question, mechanism, decision, caveat, and next route;
2. source registries own source identity and the claim ceiling;
3. claim ledgers own stable IDs, exact evidence locators, confidence, and
   contradictions; and
4. missing-evidence ledgers own the requirements for stronger claims.

Harp retains its existing vocabulary exactly:

- `EVIDENCE`;
- `SOURCE CLAIM`;
- `INFERENCE`; and
- `MISSING`.

Do not replace `SOURCE CLAIM` with Nova's shorter `CLAIM`, and do not add
`SPECULATION` as a Harp claim class. Each index and long-form reader route
will make the evidence layer visible through an Obsidian callout, rather than
repeating provenance tables in the main exposition.

The standard reader pattern is:

```markdown
> [!tip] Evidence-aware reading
> Follow a claim label to its ledger entry before carrying a result into a
> design or implementation decision. The [[knowledge/.../missing_evidence|
> missing-evidence ledger]] names the evidence required for a stronger claim.
```

The user-facing prose should use short sections and nested bullets where
possible. Avoid wide tables with long prose cells in new home and navigation
surfaces.

## 5. Internal-link grammar and migration

### 5.1 Canonical syntax

After migration, managed prose uses vault-root-qualified Obsidian links for all
internal references:

```markdown
[[knowledge/darwinx/darwinx_index|DarwinX]]
[[knowledge/darwinx/claim_evidence_ledger#DX-024: The reported studies support durable harness capability|INFERENCE - DX-024]]
[[evidence/darwinx/artifacts/darwinx-2608.07545v1.pdf|DarwinX v1 PDF]]
![[evidence/darwinx/artifacts/darwinx-2608.07545v1.pdf#page=1|DarwinX paper]]
[[#Evidence boundary]]
```

Use paths without the `.md` suffix for Markdown notes and include the exact
suffix for non-Markdown files. Every link is vault-root-qualified except a
same-note heading reference. This removes basename ambiguity and makes the
serialized source readable outside the current directory.

External HTTP(S), DOI, mail, and other web links remain ordinary Markdown
links. Captured evidence remains untouched. Raw Markdown links in captured
artifacts are not migrated.

### 5.2 Safe resolution rules

Introduce one typed wikilink parser/resolver shared by:

- canonical corpus validation;
- Atlas/offline rendering;
- credible-documentation claim validation;
- DGM and Crouzeix packet contract tests; and
- the migration/audit script.

The resolver accepts:

- `[[path|alias]]`;
- `[[path#Heading|alias]]`;
- `[[#Heading|alias]]`; and
- `![[path#subpath|alias]]`.

It rejects:

- absolute paths;
- `..` traversal;
- empty paths except same-note headings;
- malformed aliases or unclosed delimiters;
- targets outside `knowledge/`, `evidence/`, `content/`, `labs/`, or `crates/`;
- ambiguous omitted-extension note targets;
- nonexistent target files; and
- nonexistent or ambiguous target headings.

Heading links name the visible heading text in vault Markdown. The resolver
maps that heading deterministically to Harp's route/fragment identifier.
Existing Crouzeix explicit heading IDs stay supported; all other targets use
the shared slug algorithm. A link to a non-Markdown evidence artifact may not
name a heading except the supported PDF `#page=N` form.

### 5.3 Mechanical migration boundary

The migration tool rewrites all internal Markdown links in tracked Markdown
below managed `knowledge/` roots and the writer-facing documentation templates
that define future canonical prose. It leaves:

- external links;
- intra-page anchors;
- code fences and inline code;
- raw evidence;
- ignored `knowledge/investigations/local/`;
- `.obsidian/` state; and
- generated Atlas artifacts

unchanged.

For every source Markdown link, the tool resolves the old link before writing
the new one, verifies the target after conversion, preserves the displayed
text as a wikilink alias, and emits a deterministic migration report. It is
idempotent: a second run produces no diff. There will be no mixed internal
Markdown/wikilink syntax in the managed prose after the migration, except
where a literal syntax example must be fenced and labelled as an example.

### 5.4 Claim-link syntax

The main-document claim marker changes from Markdown-link syntax to:

```markdown
**[[knowledge/packet/claim_evidence_ledger#CLAIM-ID: Claim title|EVIDENCE - CLAIM-ID]].**
```

The credible-docs parser validates that:

1. the displayed `CLASS - CLAIM-ID` matches the ledger class and ID;
2. the target resolves to exactly one ledger H2;
3. the target H2 owns that claim ID; and
4. quantitative claims still state reproduction status inline.

Source and locator fields in claim ledgers use the same wikilink grammar for
local artifacts. Existing external URLs remain Markdown links. This preserves
the current claim route:

```text
reader claim -> exact ledger H2 -> exact evidence artifact
```

while making each leg native to Obsidian backlinks and hover previews.

## 6. Rust compiler and Atlas projection

### 6.1 Parser and renderer

Add a small `corpus::obsidian` module that scans authored Markdown while
protecting code fences and inline code. It produces typed text, wikilink, embed,
and callout segments instead of using regular-expression replacement.

`corpus::render` uses that module before its Pulldown-Cmark event transform:

- a wikilink becomes a normal HTML anchor with the resolved offline Atlas
  route;
- an embed becomes a safe Atlas fallback link marked with
  `data-obsidian-embed`, rather than inlining arbitrary local bytes;
- PDF embeds preserve their page subpath in the fallback URL;
- a supported Obsidian callout becomes semantic, escaped Atlas HTML with
  `obsidian-callout` classes and an accessible title; and
- untrusted raw HTML remains escaped exactly as today.

Obsidian itself reads the original wiki syntax and performs native embeds,
previews, backlinks, tags, Base views, and Canvas navigation. Atlas consumes
only the deterministic projection. The compiler never depends on a running
Obsidian process.

### 6.2 Corpus metadata and discoverability

Extend `CanonicalDocument` with validated frontmatter properties. Atlas strict
TypeScript contracts parse these fields from `unknown`. The corpus registration
adds:

- one first-class `knowledge` reader route targeting
  `knowledge/harp_knowledge_home.md`;
- every DarwinX packet document as an auxiliary document;
- DarwinX as a FTS search root; and
- a route from the new home to all existing reader routes and registered
  packets.

The DarwinX packet remains an attached research packet. It does not become a
seventeenth canonical RSI system.

### 6.3 Atlas presentation

Atlas adds a compact knowledge-home reader view driven entirely by corpus
metadata:

- packet cards grouped by `type`, `status`, and tags;
- direct routes for the knowledge home, DarwinX, and existing packets;
- visible claim/evidence/missing-evidence callouts;
- an accessible document metadata strip; and
- stable handling for `data-obsidian-embed` fallback links.

No technical prose, claim text, or packet registry is duplicated in
TypeScript. Atlas remains a renderer of canonical Markdown and validated
metadata.

## 7. Vault presentation assets

### 7.1 Vault home

`knowledge/harp_knowledge_home.md` is the entrypoint for Obsidian readers. It
contains:

- an explicit “open the repository root as the vault” instruction;
- reader routes by question: orient, compare systems, audit a claim, inspect
  evidence, learn through SICP, and browse theorem/research packets;
- links to each registered packet index;
- an evidence-aware reading callout;
- links to the Base and Canvas;
- a note that `evidence/` provides source artifacts and does not become prose
  authority; and
- a bounded maintenance section that points to the writing style guide and
  claim-ledger contract.

### 7.2 Base

`knowledge/obsidian/harp_knowledge.base` has:

- a **Reader routes** card/list view for active `index`,
  `research-index`, `benchmark-guide`, and `reference-architecture` notes;
- a **Technical deep dives** table filtered by appropriate `type`;
- an **Evidence ledgers** list for registry, claim ledger, and missing-evidence
  types; and
- a **Recently updated** view using `file.mtime`, not invented historical
  dates.

It filters to `file.inFolder("knowledge")`, excludes ignored/local material,
and displays title, type, status, confidence, tags, source IDs, and file
links. It uses no third-party Obsidian plugin.

### 7.3 Canvas

`knowledge/obsidian/harp_knowledge_map.canvas` is a small navigational map, not a
claim graph. It contains grouped file nodes for:

- RSI orientation and chapters;
- system research packets;
- evaluation and benchmark surfaces;
- evidence-layer documents;
- the SICP learning route; and
- Atlas/verification boundaries.

Edges mean “read next” or “audit through,” never causal or evidentiary
support. The Canvas file is validated as JSON Canvas: unique IDs, existing
file paths, and edge endpoints.

## 8. Shared profile and documentation

`tools/obsidian/profile/` includes only portable, reviewed configuration:

- core plugins needed for the experience: file explorer, search, graph,
  backlinks, outgoing links, properties, page preview, tags, outline, Bases,
  Canvas, and bookmarks;
- a starter bookmarks file pointing to the vault home, Base, Canvas, source
  registry, claim ledger, and missing evidence;
- a starter workspace with the vault home, backlinks, outline, properties,
  and file explorer; and
- an optional CSS snippet that improves evidence callouts, metadata chips, and
  wide tables without changing claim meaning.

`tools/obsidian/apply_profile.py` validates the source profile, creates
`.obsidian/` safely, copies only allowlisted files, refuses symlinks, and
defaults to create-only behavior. Its `--replace` mode is explicit. It writes
a local receipt naming the profile version and content digest.

Update `README.md`, `RESOURCES.md`, `docs/product-contract.md`,
`docs/contributing.md`, and `AGENTS.md` to state:

- the repository-root vault boundary;
- the difference between committed profile and ignored personal state;
- the wiki-link/evidence-link rules;
- the full upstream skill vendor and local installation process; and
- the canonical registration work required for a new packet.

## 9. Verification contract

### 9.1 Rust tests

Add focused tests for:

- wikilink tokenization, aliases, same-note headings, cross-note headings,
  embeds, and invalid forms;
- deterministic path/heading resolution and repository-root containment;
- code-fence and inline-code immunity;
- callout projection with HTML escaping;
- PDF embed fallback URL preservation;
- source/locator enforcement beneath `evidence/`;
- native claim-label parsing against ledger H2 targets;
- idempotent migration output and a zero-unmigrated-link audit;
- frontmatter minimum schema and stable metadata projection;
- Base YAML schema and Canvas JSON/edge/file validation; and
- DarwinX completeness: all nine files compile, are searchable, and are
  Atlas-routable.

Existing DGM and Crouzeix packet tests are updated to validate wiki links
without weakening their source, roster, or claim coverage checks.

### 9.2 Atlas tests

Add TypeScript tests for:

- strict metadata parsing;
- the knowledge route and all existing route IDs;
- rendering internal wiki links as offline routes;
- callout semantics and embed fallback behavior;
- metadata strip accessibility; and
- the preserved 16-system invariant.

### 9.3 Command gates

During implementation, run focused gates after each slice:

```sh
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo test -p harp --test credible_docs_style -- --test-threads=1
cargo test -p harp --test dgm_knowledge_packet -- --test-threads=1
cargo test -p harp --test crouzeix_conjecture_knowledge_packet -- --test-threads=1
cd atlas && corepack pnpm run test
```

Before landing, run:

```sh
mise run verify
```

Use `harp search query` assertions for DarwinX and the vault home. Do not call
the Obsidian desktop CLI from CI. If a local Obsidian instance is available,
the final manual check opens the repository root, Base, Canvas, home note, a
claim ledger target, and a PDF embed.

## 10. Delivery sequence

Land independently reviewable commits in this order:

1. `build: vendor pinned obsidian skills evidence`
   Vendor snapshot, license status, manifest records, source verification, and
   local TRAE installation receipt outside the repository.
2. `feat: parse and render Obsidian knowledge links`
   Typed wikilink/callout resolver, compiler/Atlas projection, and tests while
   legacy Markdown links still pass.
3. `docs: migrate managed knowledge to Obsidian links`
   Frontmatter normalization, mechanical internal-link conversion, claim-marker
   conversion, migration audit, and writing-style template updates.
4. `feat: add Obsidian knowledge home and navigation`
   Home note, Base, Canvas, profile/bootstrap tool, DarwinX registration,
   search/Atlas integration, and user documentation.
5. `build: regenerate and seal Obsidian knowledge presentation`
   Regenerate `atlas/src/content/generated/corpus.json`,
   `atlas/dist/harp-atlas.html`, and `atlas/dist/harp-atlas.receipt.json`; then
   refresh `docs/import-receipt.md` with `harp repository verify`.

Every commit message ends with:

```text
Co-authored-by: TRAE CLI <noreply@bytedance.com>
```

## 11. Acceptance criteria

The work is complete only when:

1. the complete pinned upstream `obsidian-skills` repository is vendored,
   license-preserving, and source-verified;
2. all five skills are installed locally and named in the operator handoff;
3. repository-root Obsidian opens the same managed technical prose as Harp;
4. every managed internal prose reference uses validated wikilink syntax;
5. claim labels still reach exact ledger H2s and local evidence artifacts;
6. no captured upstream evidence changed for presentation reasons;
7. the vault home, Base, Canvas, and profile are valid without community
   plugins;
8. DarwinX is compiled, searchable, and visible in Atlas;
9. offline Atlas links remain functional and do not expose arbitrary local
   filesystem paths;
10. no personal `.obsidian/` state is tracked;
11. the 16 canonical-system contract remains unchanged; and
12. `mise run verify` passes on the settled, staged repository payload.

# Obsidian-native Harp Knowledge Presentation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` (recommended) or
> `superpowers:executing-plans` to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Harp’s repository-root vault, canonical knowledge corpus, and
offline Atlas provide the same evidence-aware, Obsidian-native navigation
without creating a second prose authority.

**Architecture:** Vendor the full pinned Obsidian skills repository as
license-preserved implementation evidence, add one typed Obsidian syntax
resolver shared by the corpus compiler and documentation validators, then
mechanically migrate authored note links and project the results into Atlas.
The vault home, Base, Canvas, and portable profile are canonical presentation
assets; ignored `.obsidian/` state remains personal machine configuration.

**Tech Stack:** Rust 1.92 and `pulldown-cmark` 0.13, strict TypeScript/React
19, Python 3 standard library, YAML/JSON Canvas text files, `mise`, Git LFS,
and the pinned `kepano/obsidian-skills` repository.

---

## Invariant Map

| Boundary | Enforced invariant | Structural enforcement |
|---|---|---|
| Prose authority | Technical prose remains under managed `knowledge/` roots; `content/` remains structured-only. | Existing corpus content-root rejection plus managed-root migration audit. |
| Evidence fidelity | Captured files under `evidence/*/artifacts/` are never reformatted or decorated with block IDs. | Migration scanner excludes `evidence/`; source manifests hash vendored bytes. |
| Local links | Wiki targets cannot escape the repository or pass through symlinks. | One typed resolver rejects absolute paths, traversal, nonregular targets, and ambiguous names. |
| Claim routes | Reader claim → exact ledger H2 → exact evidence locator remains intact. | Credible-docs parser validates the wiki claim target and exact locator separately. |
| Offline Atlas | Every wiki note link becomes an internal Atlas route; arbitrary local paths never leak into the export. | Renderer maps only resolved registered targets; all other safe artifact links remain repository-relative. |
| Canonical systems | The RSI system registry stays exactly 16 identities. | Preserve existing registry test; DarwinX registers only as a packet/auxiliary-document route. |
| Personal state | No `.obsidian/` runtime state or user workspace layout becomes tracked. | Ignore root `.obsidian/`; profile tool writes only allowlisted files on explicit invocation. |
| Source vendor | Full upstream snapshot, license, revision, and digest are verifiable offline. | Existing implementation manifest and `harp sources verify`. |

## Exact Link Policy

Internal **note navigation** is fully migrated to vault-root-qualified
Obsidian wikilinks:

```markdown
[[knowledge/darwinx/darwinx_index|DarwinX]]
[[knowledge/darwinx/claim_evidence_ledger#DX-024: The reported studies support durable harness capability|INFERENCE - DX-024]]
[[evidence/darwinx/artifacts/darwinx-2608.07545v1.pdf|DarwinX v1 PDF]]
![[evidence/darwinx/artifacts/darwinx-2608.07545v1.pdf#page=1|DarwinX paper]]
```

Exact line locators into immutable raw evidence have a narrow, intentional
dual form:

```markdown
[[evidence/weng/text/mce.txt|MCE extracted text]]
([exact lines 103–865](../../evidence/weng/text/mce.txt#L103-L865))
```

Obsidian has no native immutable line-anchor syntax, and adding block IDs to
captured evidence would violate Harp’s evidence contract. The first link is
native vault navigation; the second retains the current exact, verifier-checked
line locator. This is the only retained internal Markdown-link class. External
URLs remain standard Markdown links.

## File Structure

| Path | Responsibility |
|---|---|
| `evidence/implementations/obsidian_skills/{REMOTE,REVISION,LICENSE,LICENSE_STATUS,snapshot/**}` | Complete MIT-licensed upstream snapshot. |
| `evidence/implementations/manifest.tsv` | Digest records for each vendored upstream file. |
| `crates/harp/src/corpus/obsidian.rs` | Typed scanner/parser/resolver for wiki links, embeds, callouts, headings, and safe paths. |
| `crates/harp/src/corpus/{contracts.rs,render.rs,mod.rs,tests.rs}` | Corpus validation, safe Atlas projection, metadata projection, route registration. |
| `crates/harp/src/search.rs` | DarwinX and vault-home search-root registration. |
| `crates/harp/tests/{credible_docs_style.rs,dgm_knowledge_packet.rs,crouzeix_conjecture_knowledge_packet.rs,obsidian_knowledge_packet.rs,cli.rs}` | Shared claim/link contracts and packet/integration coverage. |
| `scripts/migrate_obsidian_links.py` | Deterministic, idempotent, checkable migration and audit tool. |
| `scripts/validate_obsidian_assets.py` | Standard-library validation for frontmatter, Base YAML shape, Canvas graph, and profile allowlist. |
| `knowledge/harp_knowledge_home.md` | Reader-first vault entrypoint. |
| `knowledge/obsidian/{README.md,harp_knowledge.base,harp_knowledge_map.canvas}` | Portable vault navigation assets. |
| `tools/obsidian/profile/**` | Sanitized portable `.obsidian/` profile source. |
| `tools/obsidian/apply_profile.py` | Explicit, symlink-safe, create-only profile installer. |
| `atlas/src/content/{types.ts,contracts.ts,canonical.ts}` | Strict projection and metadata access. |
| `atlas/src/app/{AtlasApp.tsx,ChapterReader.tsx,KnowledgeHome.tsx}` | Atlas knowledge route, callout styling, metadata strip, safe embeds. |
| `atlas/src/**/*.{test.ts,test.tsx,css}` | Atlas route/rendering/accessibility tests and style. |
| `README.md`, `RESOURCES.md`, `AGENTS.md`, `docs/{product-contract.md,contributing.md,writing-style/*.md}` | Operator, authoring, evidence, and route documentation. |

### Task 1: Vendor the upstream suite and install local skills

**Files:**
- Create: `evidence/implementations/obsidian_skills/REMOTE`
- Create: `evidence/implementations/obsidian_skills/REVISION`
- Create: `evidence/implementations/obsidian_skills/LICENSE`
- Create: `evidence/implementations/obsidian_skills/LICENSE_STATUS`
- Create: `evidence/implementations/obsidian_skills/snapshot/**`
- Modify: `evidence/implementations/manifest.tsv`
- Modify: `crates/harp/tests/cli.rs`
- Modify: `docs/product-contract.md`
- Modify: `AGENTS.md`
- Test: `crates/harp/tests/cli.rs`

- [ ] **Step 1: Add the failing vendor-manifest expectation**

  In `crates/harp/tests/cli.rs`, add assertions in
  `sources_verify_accepts_the_tracked_offline_evidence` for an
  `OBSIDIAN-SKILLS` implementation source and its expected snapshot-file count.
  Use a count derived from the actual vendored manifest rows, not a guessed
  literal. Add a focused test that reads `REMOTE`, `REVISION`, and
  `LICENSE_STATUS` and requires:

  ```rust
  assert_eq!(
      fs::read_to_string(root.join("REMOTE")).unwrap(),
      "https://github.com/kepano/obsidian-skills.git\n"
  );
  assert_eq!(
      fs::read_to_string(root.join("REVISION")).unwrap(),
      "a1dc48e68138490d522c04cbf5822214c6eb1202\n"
  );
  assert_eq!(
      fs::read_to_string(root.join("LICENSE_STATUS")).unwrap(),
      "MIT\n"
  );
  ```

- [ ] **Step 2: Run the test to prove the vendor does not exist**

  Run:

  ```sh
  cargo test -p harp --test cli sources_verify_accepts_the_tracked_offline_evidence -- --test-threads=1
  ```

  Expected: failure because `OBSIDIAN-SKILLS` is absent from the implementation
  manifest and snapshot root.

- [ ] **Step 3: Create a source-faithful upstream snapshot**

  From a clean temporary clone at
  `a1dc48e68138490d522c04cbf5822214c6eb1202`, copy every tracked upstream
  file beneath:

  ```text
  evidence/implementations/obsidian_skills/snapshot/
    .claude-plugin/marketplace.json
    .claude-plugin/plugin.json
    LICENSE
    README.md
    skills/defuddle/SKILL.md
    skills/json-canvas/SKILL.md
    skills/json-canvas/references/EXAMPLES.md
    skills/obsidian-bases/SKILL.md
    skills/obsidian-bases/references/FUNCTIONS_REFERENCE.md
    skills/obsidian-cli/SKILL.md
    skills/obsidian-markdown/SKILL.md
    skills/obsidian-markdown/references/CALLOUTS.md
    skills/obsidian-markdown/references/EMBEDS.md
    skills/obsidian-markdown/references/PROPERTIES.md
  ```

  Do not add Harp branding or normalize upstream formatting. Copy the upstream
  MIT license into the top-level vendor `LICENSE`, and write:

  ```text
  # REMOTE
  https://github.com/kepano/obsidian-skills.git

  # REVISION
  a1dc48e68138490d522c04cbf5822214c6eb1202

  # LICENSE_STATUS
  MIT
  ```

  Add one `OBSIDIAN-SKILLS` manifest row per `snapshot/` file. Every row uses
  the exact remote, 40-character revision, repository-relative snapshot path,
  byte length, and SHA-256. Keep `manifest.tsv` lexicographically sorted by
  source ID then snapshot path.

- [ ] **Step 4: Extend product documentation**

  Add one short product-contract evidence bullet saying that Harp carries a
  pinned MIT Obsidian-syntax skill snapshot. Add an AGENTS rule that source
  changes to the snapshot use the implementation-manifest update workflow and
  that the snapshot is not a Harp prose authority.

- [ ] **Step 5: Run source verification**

  Run:

  ```sh
  cargo test -p harp --test cli sources_verify_accepts_the_tracked_offline_evidence -- --test-threads=1
  cargo run -p harp -- sources verify
  ```

  Expected: both pass, the report includes one additional implementation
  source, and no binary/LFS count changes occur.

- [ ] **Step 6: Install all five skills locally**

  Run the supported installer at the pinned revision:

  ```sh
  python3 /Users/bytedance/.trae/skills/.system/skill-installer/scripts/install-skill-from-github.py \
    --repo kepano/obsidian-skills \
    --ref a1dc48e68138490d522c04cbf5822214c6eb1202 \
    --path skills/obsidian-markdown \
           skills/obsidian-bases \
           skills/json-canvas \
           skills/obsidian-cli \
           skills/defuddle
  ```

  Verify each local path:

  ```sh
  for skill in obsidian-markdown obsidian-bases json-canvas obsidian-cli defuddle; do
    test -f "$HOME/.trae/skills/$skill/SKILL.md"
  done
  ```

  This is machine-local setup, not a tracked receipt. Do not add its output to
  `evidence/`.

- [ ] **Step 7: Commit the vendor slice**

  ```sh
  git add \
    evidence/implementations/obsidian_skills \
    evidence/implementations/manifest.tsv \
    crates/harp/tests/cli.rs \
    docs/product-contract.md \
    AGENTS.md
  git diff --cached --check
  git commit -m "build: vendor pinned obsidian skills evidence" \
    -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
  ```

### Task 2: Add a typed Obsidian syntax and link-resolution boundary

**Files:**
- Create: `crates/harp/src/corpus/obsidian.rs`
- Modify: `crates/harp/src/corpus/mod.rs`
- Modify: `crates/harp/src/corpus/contracts.rs`
- Modify: `crates/harp/src/corpus/render.rs`
- Modify: `crates/harp/src/corpus/tests.rs`
- Test: `crates/harp/src/corpus/obsidian.rs`

- [ ] **Step 1: Write parser/resolver tests first**

  Add unit tests in `corpus::obsidian::tests` for these exact cases:

  ```rust
  assert_eq!(
      parse_inline("[[knowledge/darwinx/darwinx_index|DarwinX]]").unwrap(),
      vec![Inline::WikiLink(WikiLink {
          embed: false,
          path: Some("knowledge/darwinx/darwinx_index".into()),
          subpath: None,
          alias: Some("DarwinX".into()),
      })]
  );
  assert_eq!(
      parse_inline("[[knowledge/darwinx/claim_evidence_ledger#DX-024: Durable capability|DX-024]]").unwrap(),
      vec![Inline::WikiLink(WikiLink {
          embed: false,
          path: Some("knowledge/darwinx/claim_evidence_ledger".into()),
          subpath: Some(WikiSubpath::Heading("DX-024: Durable capability".into())),
          alias: Some("DX-024".into()),
      })]
  );
  assert!(parse_inline("[[../../etc/passwd]]").is_err());
  assert!(parse_inline("[[knowledge/darwinx/darwinx_index|]]").is_err());
  assert_eq!(
      parse_inline("`[[knowledge/rsi/rsi_index]]`\n```md\n[[knowledge/rsi/rsi_index]]\n```").unwrap(),
      vec![Inline::Text("`[[knowledge/rsi/rsi_index]]`\n```md\n[[knowledge/rsi/rsi_index]]\n```".into())]
  );
  ```

  Add resolution tests for omitted `.md` suffixes, explicit non-Markdown
  suffixes, aliases, same-note headings, source-heading duplicates, PDF
  `#page=1`, traversal rejection, a symlink escape fixture, and an ambiguous
  omitted-extension target.

- [ ] **Step 2: Run the focused tests to verify the module is missing**

  Run:

  ```sh
  cargo test -p harp corpus::obsidian::tests --lib -- --test-threads=1
  ```

  Expected: compilation failure because `corpus::obsidian` is not defined.

- [ ] **Step 3: Implement the typed parser**

  Add `mod obsidian;` in `crates/harp/src/corpus/mod.rs`. In
  `obsidian.rs`, define:

  ```rust
  #[derive(Clone, Debug, Eq, PartialEq)]
  pub(super) enum WikiSubpath {
      Heading(String),
      Block(String),
      PdfPage(u32),
  }

  #[derive(Clone, Debug, Eq, PartialEq)]
  pub(super) struct WikiLink {
      pub(super) embed: bool,
      pub(super) path: Option<String>,
      pub(super) subpath: Option<WikiSubpath>,
      pub(super) alias: Option<String>,
  }

  #[derive(Clone, Debug, Eq, PartialEq)]
  pub(super) enum Inline {
      Text(String),
      WikiLink(WikiLink),
      Callout(Callout),
  }
  ```

  Scan bytes rather than applying global regular expressions. Track fenced code
  blocks and inline code spans. Parse `[[...]]`, `![[...]]`, and Obsidian
  callout leaders only outside code. Reject empty path/alias fields, repeated
  `|`, invalid block IDs, invalid page numbers, and unclosed delimiters with
  `AppError::invalid_input` codes prefixed `knowledge.obsidian.`.

  Add a resolver taking `&HeldDirectory`, the source path, the parsed link,
  and a `BTreeMap<String, HeadingIndex>`. It normalizes vault-root paths,
  permits only `knowledge`, `evidence`, `content`, `labs`, and `crates`,
  uses a deterministic `.md` candidate only for omitted extensions, calls the
  held-directory regular-file check, and returns:

  ```rust
  pub(super) struct ResolvedWikiLink {
      pub(super) target: PathBuf,
      pub(super) heading_id: Option<String>,
      pub(super) pdf_page: Option<u32>,
      pub(super) embed: bool,
      pub(super) display: String,
  }
  ```

- [ ] **Step 4: Use the resolver for canonical local-link validation**

  Change `contracts::validate_local_links` to visit both normal Markdown
  links and parsed wiki links. Keep existing Markdown links valid during this
  implementation slice so that the link engine can land before the bulk
  migration. For wiki links:

  - normal note/evidence targets must exist and remain under allowed roots;
  - heading links must identify exactly one target heading;
  - PDF embeds may use `#page=N` with `N >= 1`;
  - embeds of non-PDF binary artifacts must fail closed; and
  - block links are rejected until the repository owns an immutable
    block-ID-bearing target contract.

- [ ] **Step 5: Project wiki links and callouts into the offline renderer**

  In `render.rs`, preprocess only authored Markdown events with the typed
  inline sequence:

  - `[[note|alias]]` renders as `<a href="...">alias</a>` using the existing
    `RouteTarget` projection;
  - `![[note]]` renders as an accessible fallback link with
    `class="obsidian-embed-fallback"` and `data-obsidian-embed="true"`;
  - PDF pages preserve `#page=N` in the fallback `href`;
  - callouts render to escaped semantic HTML:

    ```html
    <aside class="obsidian-callout obsidian-callout-tip" role="note">
      <p class="obsidian-callout-title">Evidence-aware reading</p>
      ...
    </aside>
    ```

  Keep unknown raw HTML escaped and retain all current Markdown rendering
  behavior.

- [ ] **Step 6: Run the focused compiler tests**

  Run:

  ```sh
  cargo test -p harp corpus::obsidian::tests --lib -- --test-threads=1
  cargo test -p harp corpus::tests::resolves_parent_links_without_allowing_repository_escape --lib -- --test-threads=1
  cargo test -p harp corpus::tests::reader_route_targets_precede_document_routes_and_fragments_survive --lib -- --test-threads=1
  ```

  Expected: wiki aliases and headings route correctly, PDF pages survive,
  unsafe targets fail, and legacy Markdown routes remain unchanged.

- [ ] **Step 7: Commit the parser/render slice**

  ```sh
  git add crates/harp/src/corpus/{mod.rs,obsidian.rs,contracts.rs,render.rs,tests.rs}
  git diff --cached --check
  git commit -m "feat: parse and render Obsidian knowledge links" \
    -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
  ```

### Task 3: Make claim and packet contracts wiki-link aware

**Files:**
- Modify: `crates/harp/tests/credible_docs_style.rs`
- Modify: `crates/harp/tests/dgm_knowledge_packet.rs`
- Modify: `crates/harp/tests/crouzeix_conjecture_knowledge_packet.rs`
- Create: `crates/harp/tests/obsidian_knowledge_packet.rs`
- Test: the four files above

- [ ] **Step 1: Add failing wiki-claim fixtures**

  In `credible_docs_style.rs`, add these fixtures:

  ```markdown
  **[[knowledge/example/claim_evidence_ledger#EX-002: Authors report the benchmark result|SOURCE CLAIM - EX-002]].**
  The authors report 50.0%; this result has not been independently reproduced.
  ```

  Require it to validate when the ledger has the matching H2 and source
  fields. Add failing fixtures for:

  ```markdown
  **[[knowledge/example/claim_evidence_ledger#EX-002: Wrong title|SOURCE CLAIM - EX-002]].**
  Body.
  ```

  and a missing ledger target. Keep the existing Markdown-marker fixtures
  valid until Task 4 converts canonical prose.

- [ ] **Step 2: Run the style suite to prove native claim markers fail**

  Run:

  ```sh
  cargo test -p harp --test credible_docs_style main_claims_must_match_ledger_class_and_id -- --test-threads=1
  ```

  Expected: current parser does not recognize the native wikilink marker.

- [ ] **Step 3: Add shared wiki-link extraction to style and packet tests**

  Replace test-local assumptions that all links originate from
  `pulldown_cmark::Tag::Link` with a small test helper that merges normal
  Markdown links and `corpus::obsidian` wiki links. Implement
  `parse_main_claims` to accept either exact marker:

  ```text
  **[CLASS - ID](relative-ledger.md#slug).**
  **[[knowledge/packet/ledger#ID: Heading|CLASS - ID]].**
  ```

  For a native marker, call the same resolver as the compiler and compare the
  resolved ledger H2 title/slug to the actual `ClaimEntry`. Continue requiring
  conventional Markdown links for line-precise raw evidence locators.

- [ ] **Step 4: Extend DGM and Crouzeix packet contract tests**

  Update their local-link walkers to validate wiki links, aliases, heading
  references, and exact backlinks. Preserve:

  - DGM file roster, source IDs, frontmatter, and authority notice checks;
  - Crouzeix explicit heading-ID checks and local evidence-root restriction;
  - all claim rosters and reciprocal relationship assertions.

  Add one test in each packet suite that confirms a wiki link escaping via
  `[[../../outside]]` fails without reading outside the fixture repository.

- [ ] **Step 5: Add a DarwinX packet contract**

  Create `obsidian_knowledge_packet.rs` with a test named:

  ```rust
  #[test]
  fn darwinx_packet_is_complete_searchable_and_atlas_routable()
  ```

  It must assert:

  - exactly these nine packet files exist:
    `darwinx_index.md`, `01_mechanism_and_selection.md`,
    `02_evaluation_audit.md`, `03_critical_review.md`,
    `04_comparative_synthesis.md`, `05_successor_experiment.md`,
    `claim_evidence_ledger.md`, `source_registry.md`, `maintenance.md`;
  - every file has valid frontmatter and resolves all Markdown/wiki links;
  - `darwinx_index.md` links to every sibling document;
  - every `DX-*` ledger entry is referenced by reader-facing prose;
  - the corpus includes every packet document after registration;
  - `search::load_documents` includes `knowledge/darwinx`; and
  - no assertion treats DarwinX as a canonical RSI system.

- [ ] **Step 6: Run packet and style regressions**

  Run:

  ```sh
  cargo test -p harp --test credible_docs_style -- --test-threads=1
  cargo test -p harp --test dgm_knowledge_packet -- --test-threads=1
  cargo test -p harp --test crouzeix_conjecture_knowledge_packet -- --test-threads=1
  cargo test -p harp --test obsidian_knowledge_packet -- --test-threads=1
  ```

  Expected: all legacy source/claim contracts remain green while native wiki
  syntax is accepted.

- [ ] **Step 7: Commit the validation slice**

  ```sh
  git add \
    crates/harp/tests/credible_docs_style.rs \
    crates/harp/tests/dgm_knowledge_packet.rs \
    crates/harp/tests/crouzeix_conjecture_knowledge_packet.rs \
    crates/harp/tests/obsidian_knowledge_packet.rs
  git diff --cached --check
  git commit -m "test: validate Obsidian claim and packet contracts" \
    -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
  ```

### Task 4: Build and test the deterministic migration tool

**Files:**
- Create: `scripts/migrate_obsidian_links.py`
- Create: `scripts/tests/test_migrate_obsidian_links.py`
- Modify: `docs/writing-style/{STYLE_GUIDE.md,main-document-template.md,claim-ledger-template.md,source-registry-template.md,worked-example.md,review-checklist.md}`
- Modify: `docs/contributing.md`
- Test: `scripts/tests/test_migrate_obsidian_links.py`

- [ ] **Step 1: Write migration fixtures**

  In `scripts/tests/test_migrate_obsidian_links.py`, create a temporary
  repository fixture with:

  ```text
  knowledge/a.md                    # links to b.md and an evidence text file
  knowledge/b.md                    # H2 heading "Result boundary"
  evidence/source.txt               # immutable example source
  ```

  Test exact conversions:

  ```python
  assert migrated == (
      "[[knowledge/b#Result boundary|Result]]\n"
      "[[evidence/source.txt|source]] ([exact lines 2–3](../evidence/source.txt#L2-L3))\n"
      "`[Code](b.md)`\n"
      "```md\n[Code](b.md)\n```\n"
  )
  ```

  Add tests for:

  - relative paths resolving to vault-root `knowledge/...`;
  - same-note anchors;
  - external URLs unchanged;
  - raw evidence roots skipped;
  - links to PDFs retaining `.pdf`;
  - missing source link failure before output write;
  - ambiguous target heading failure;
  - idempotence (`--write` then `--check`); and
  - a refusal when a file outside managed `knowledge/` roots is supplied.

- [ ] **Step 2: Run the fixture test to prove the tool is absent**

  Run:

  ```sh
  python3 -m unittest scripts.tests.test_migrate_obsidian_links
  ```

  Expected: import failure because `migrate_obsidian_links` does not exist.

- [ ] **Step 3: Implement the migration engine**

  Implement only with Python standard library:

  ```python
  def discover_managed_notes(repo_root: Path) -> list[Path]: ...
  def scan_markdown_links(text: str) -> list[LinkSpan]: ...
  def resolve_markdown_target(source: Path, destination: str, repo_root: Path) -> ResolvedTarget: ...
  def heading_for_fragment(note: Path, fragment: str) -> str: ...
  def wiki_destination(target: ResolvedTarget, label: str) -> str: ...
  def migrate_text(source: Path, text: str, repo_root: Path) -> str: ...
  ```

  `LinkSpan` carries byte offsets, display text, destination, and code-region
  state. The scanner skips inline code, fenced code, HTML source folds, and
  already-native wiki links. It resolves the old Markdown destination before
  writing. For a note fragment, it reads the target heading and emits the
  visible heading title. For an immutable evidence `#L...` locator, it emits a
  wiki artifact link plus the original exact Markdown line link. It refuses to
  modify a file that has an unresolved local link.

  CLI contract:

  ```text
  python3 scripts/migrate_obsidian_links.py --check
  python3 scripts/migrate_obsidian_links.py --write
  python3 scripts/migrate_obsidian_links.py --report target/obsidian-link-migration.json
  ```

  The report is deterministic JSON with input count, converted count,
  retained-line-locator count, skipped-captured count, and remaining managed
  Markdown-link count.

- [ ] **Step 4: Update writer-facing documentation**

  Update the style guide and templates so new managed prose uses:

  ```markdown
  **[[knowledge/packet/claim_evidence_ledger#EX-002: Authors report the benchmark result|SOURCE CLAIM - EX-002]].**
  ```

  Document the narrow exact-line-locator dual-link exception. Update
  `docs/contributing.md` to say canonical prose is under `knowledge/`, use
  vault-root wiki links for notes/artifacts, and run the migration check before
  committing a prose change.

- [ ] **Step 5: Run migration and style checks**

  Run:

  ```sh
  python3 -m unittest scripts.tests.test_migrate_obsidian_links -v
  python3 scripts/migrate_obsidian_links.py --check
  cargo test -p harp --test credible_docs_style -- --test-threads=1
  ```

  Expected before bulk conversion: fixture tests pass; repository `--check`
  reports managed Markdown links requiring conversion; style tests pass because
  compatibility is still dual syntax.

- [ ] **Step 6: Commit the migration tooling slice**

  ```sh
  git add \
    scripts/migrate_obsidian_links.py \
    scripts/tests/test_migrate_obsidian_links.py \
    docs/writing-style \
    docs/contributing.md
  git diff --cached --check
  git commit -m "feat: add deterministic Obsidian link migration" \
    -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
  ```

### Task 5: Migrate managed knowledge links and normalize frontmatter

**Files:**
- Modify: every tracked `knowledge/**/*.md` in the managed packet roots
- Modify: `scripts/migrate_obsidian_links.py`
- Modify: `crates/harp/tests/obsidian_knowledge_packet.rs`
- Test: all managed Markdown through migration `--check`

- [ ] **Step 1: Add a failing zero-legacy-link assertion**

  Extend `obsidian_knowledge_packet.rs` with a recursive managed-note audit:

  ```rust
  assert_eq!(
      audit.remaining_note_navigation_markdown_links,
      0,
      "managed knowledge must use wikilinks for note/artifact navigation"
  );
  ```

  Permit only the recorded evidence-line-locator pattern:

  ```text
  [[evidence/...|...]] ([exact lines ...](...#L...))
  ```

  Reject relative `.md` Markdown links, relative PDF links, and
  `[label](../knowledge/...)` navigation in managed prose.

- [ ] **Step 2: Run the assertion to verify it fails**

  Run:

  ```sh
  cargo test -p harp --test obsidian_knowledge_packet -- --test-threads=1
  ```

  Expected: failure reports existing Markdown navigation links.

- [ ] **Step 3: Add frontmatter only where it is missing**

  Add the baseline contract to every managed note lacking frontmatter:

  ```yaml
  ---
  id: derived-from-stable-vault-path
  title: Existing H1 title
  type: inferred-from-owning-directory
  status: active
  tags: [existing-topic-tags]
  confidence: medium
  ---
  ```

  Use these deterministic type mappings:

  | Path | `type` |
  |---|---|
  | `knowledge/rsi/chapters/**` | `chapter` |
  | `knowledge/rsi/concepts/**` | `concept` |
  | `knowledge/rsi/systems/**` | `system-reading` |
  | `knowledge/rsi/lessons/**` | `learning-lesson` |
  | `**/claim_evidence_ledger.md` or `claim_evidence_crosswalk.md` | `claim-evidence-ledger` |
  | `**/source_registry.md` | `source-registry` |
  | `**/missing_evidence.md` or `gap_map.md` | `missing-evidence-ledger` |
  | `**/*index.md` | `research-index` |
  | `knowledge/harness_benchmarks/**` | `benchmark-guide` |
  | `knowledge/evaluator_integrity/**` | `benchmark-guide` |
  | `knowledge/agentic_engineering/**` | `reference-architecture` |
  | `knowledge/crouzeix_conjecture/**` | `technical-deep-dive` |
  | `knowledge/darwinx/**` and `knowledge/darwin_godel_machine/**` | `technical-deep-dive` |

  Preserve existing `id`, `title`, `type`, `mode`, `status`, `created`,
  `updated`, tags, confidence, and packet-specific fields. Never create source
  IDs, coverage keys, dates, or evidence status by inference.

- [ ] **Step 4: Apply the mechanical link migration**

  Run:

  ```sh
  python3 scripts/migrate_obsidian_links.py --write \
    --report target/obsidian-link-migration.json
  python3 scripts/migrate_obsidian_links.py --check \
    --report target/obsidian-link-migration-check.json
  cmp target/obsidian-link-migration.json target/obsidian-link-migration-check.json
  ```

  Review the two reports and the diff. Confirm that:

  - links in `knowledge/investigations/local/` were untouched;
  - no file under `evidence/*/artifacts/` changed;
  - code examples were untouched;
  - external URLs remain Markdown;
  - every migrated internal note link is vault-root-qualified;
  - exact raw-evidence line locators use the dual form; and
  - a second `--write` produces no diff.

- [ ] **Step 5: Add index callouts and prose routes**

  For `knowledge/rsi/rsi_index.md`, `knowledge/darwinx/darwinx_index.md`,
  `knowledge/darwin_godel_machine/darwin_godel_machine_index.md`,
  `knowledge/agentic_engineering/agentic_engineering_index.md`,
  `knowledge/crouzeix_conjecture/crouzeix_conjecture_index.md`, and
  `knowledge/self_improving_agents_survey/synthesis.md`, add a short
  `> [!tip] Evidence-aware reading` callout that links to the responsible
  claim ledger, source registry, and missing-evidence document when present.
  Do not introduce wide tables or repeat the ledger’s provenance fields.

- [ ] **Step 6: Run managed-prose validation**

  Run:

  ```sh
  python3 scripts/migrate_obsidian_links.py --check
  cargo test -p harp --test credible_docs_style -- --test-threads=1
  cargo test -p harp --test dgm_knowledge_packet -- --test-threads=1
  cargo test -p harp --test crouzeix_conjecture_knowledge_packet -- --test-threads=1
  cargo test -p harp --test obsidian_knowledge_packet -- --test-threads=1
  cargo run -p harp -- check
  ```

  Expected: zero managed navigation Markdown links, no modified captured
  artifacts, valid ledgers, and valid corpus compilation.

- [ ] **Step 7: Commit the prose migration**

  Stage explicit path groups rather than `git add .`:

  ```sh
  git add knowledge/rsi knowledge/darwinx knowledge/darwin_godel_machine
  git add knowledge/meta_harness knowledge/harness_benchmarks
  git add knowledge/evaluator_integrity knowledge/self_improving_agents_survey
  git add knowledge/agentic_engineering knowledge/crouzeix_conjecture
  git add scripts/migrate_obsidian_links.py crates/harp/tests/obsidian_knowledge_packet.rs
  git diff --cached --check
  git commit -m "docs: migrate managed knowledge to Obsidian links" \
    -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
  ```

### Task 6: Add the portable vault home, Base, Canvas, and profile installer

**Files:**
- Create: `knowledge/harp_knowledge_home.md`
- Create: `knowledge/obsidian/README.md`
- Create: `knowledge/obsidian/harp_knowledge.base`
- Create: `knowledge/obsidian/harp_knowledge_map.canvas`
- Create: `tools/obsidian/profile/{app.json,core-plugins.json,bookmarks.json,workspace.json,snippets/harp-knowledge.css}`
- Create: `tools/obsidian/apply_profile.py`
- Create: `scripts/validate_obsidian_assets.py`
- Modify: `.gitignore`
- Test: `scripts/validate_obsidian_assets.py`

- [ ] **Step 1: Write asset validation tests**

  In `scripts/validate_obsidian_assets.py`, validate:

  ```python
  REQUIRED_PROFILE_FILES = {
      "app.json",
      "core-plugins.json",
      "bookmarks.json",
      "workspace.json",
      "snippets/harp-knowledge.css",
  }
  REQUIRED_CORE_PLUGINS = {
      "file-explorer", "global-search", "graph", "backlink", "outgoing-link",
      "properties", "page-preview", "tag-pane", "outline", "bookmarks",
      "bases", "canvas",
  }
  ```

  Validate Base top-level keys are only `filters`, `formulas`, `properties`,
  `summaries`, and `views`; each view has a supported type. Validate Canvas
  is an object with `nodes` and `edges`, unique 16-hex IDs, file nodes whose
  paths exist under the repository root, and edges whose endpoints exist.

- [ ] **Step 2: Run the validator before assets exist**

  Run:

  ```sh
  python3 scripts/validate_obsidian_assets.py
  ```

  Expected: failure describing the first missing portable asset.

- [ ] **Step 3: Create the human-facing vault home**

  Create `knowledge/harp_knowledge_home.md` with valid metadata:

  ```yaml
  ---
  id: harp-knowledge-home
  title: Harp knowledge home
  type: research-index
  status: active
  tags: [harp, knowledge, navigation, obsidian]
  confidence: high
  ---
  ```

  The prose must:

  - state “Open the Harp repository root as the Obsidian vault”;
  - route by question: orient RSI, compare systems, inspect evidence, learn
    SICP, review harness benchmarks, inspect DarwinX, and explore Crouzeix;
  - link to `[[knowledge/obsidian/harp_knowledge.base|Knowledge Base]]` and
    `[[knowledge/obsidian/harp_knowledge_map.canvas|Knowledge Canvas]]`;
  - include an `Evidence-aware reading` callout explaining claim ledger,
    source registry, and missing-evidence responsibilities;
  - state that `knowledge/` is technical-prose authority, `evidence/` is
    captured source material, and graph/Base membership is not evidence; and
  - link back to `[[docs/writing-style/STYLE_GUIDE|credible documentation style]]`.

- [ ] **Step 4: Create the Base and Canvas**

  `harp_knowledge.base` uses:

  ```yaml
  filters: 'file.inFolder("knowledge") && file.ext == "md"'
  properties:
    file.link:
      displayName: "Document"
    title:
      displayName: "Title"
    type:
      displayName: "Kind"
    status:
      displayName: "Status"
    confidence:
      displayName: "Confidence"
    tags:
      displayName: "Tags"
  views:
    - type: cards
      name: Reader routes
      filters: 'type == "research-index" || type == "benchmark-guide" || type == "reference-architecture"'
      order: [file.link, title, status, confidence, tags]
    - type: table
      name: Technical deep dives
      filters: 'type == "technical-deep-dive" || type == "system-reading"'
      order: [file.link, title, status, confidence, tags, source_ids]
    - type: list
      name: Evidence ledgers
      filters: 'type == "source-registry" || type == "claim-evidence-ledger" || type == "missing-evidence-ledger"'
      order: [file.link, title, status, confidence]
    - type: table
      name: Recently updated
      order: [file.link, title, type, file.mtime]
  ```

  `harp_knowledge_map.canvas` contains text group nodes and file nodes for the vault home,
  RSI index, DarwinX index, DGM index, Meta-Harness deep dive, benchmark
  guide, claim/source/missing evidence surfaces, and Crouzeix index. Label
  edges only `read next` or `audit through`.

- [ ] **Step 5: Create profile and safe installer**

  Add root `.obsidian/` ignore while retaining `/knowledge/.obsidian/` to
  avoid tracking legacy local state:

  ```gitignore
  /.obsidian/
  /knowledge/.obsidian/
  ```

  Implement `apply_profile.py` with:

  ```text
  usage: apply_profile.py --vault /absolute/repository/path [--replace]
  ```

  It must:

  - require an absolute real vault directory;
  - reject a symlinked vault, `.obsidian`, destination component, and profile
    source;
  - create `<vault>/.obsidian` with owner-only permissions;
  - copy only `REQUIRED_PROFILE_FILES`;
  - refuse existing destination files by default;
  - replace only the same allowlisted destination files with `--replace`;
  - write `.obsidian/harp-profile-receipt.json` atomically with profile version,
    file digests, and UTC application time; and
  - never inspect arbitrary personal workspace files.

  Configure only core plugins; no community plugin ID or network service is
  added. The CSS snippet styles `.obsidian-callout`, metadata chips, and
  `.canonical-table-scroll` without changing text or evidence classes.

- [ ] **Step 6: Run portable asset validation**

  Run:

  ```sh
  python3 scripts/validate_obsidian_assets.py
  tmp_vault="$(mktemp -d)"
  cp -R knowledge "$tmp_vault/knowledge"
  cp -R evidence "$tmp_vault/evidence"
  cp -R content "$tmp_vault/content"
  python3 tools/obsidian/apply_profile.py --vault "$tmp_vault"
  test -f "$tmp_vault/.obsidian/harp-profile-receipt.json"
  ! python3 tools/obsidian/apply_profile.py --vault "$tmp_vault"
  python3 tools/obsidian/apply_profile.py --vault "$tmp_vault" --replace
  ```

  Expected: profile applies create-only, the repeated default invocation
  fails, and explicit replacement passes.

- [ ] **Step 7: Commit navigation assets**

  ```sh
  git add \
    .gitignore \
    knowledge/harp_knowledge_home.md \
    knowledge/obsidian \
    tools/obsidian \
    scripts/validate_obsidian_assets.py
  git diff --cached --check
  git commit -m "feat: add Obsidian knowledge home and navigation" \
    -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
  ```

### Task 7: Register metadata, DarwinX, and the knowledge home in corpus, search, and Atlas

**Files:**
- Modify: `crates/harp/src/corpus/{mod.rs,contracts.rs,render.rs,tests.rs}`
- Modify: `crates/harp/src/search.rs`
- Modify: `crates/harp/tests/{cli.rs,obsidian_knowledge_packet.rs}`
- Modify: `atlas/src/content/{types.ts,contracts.ts,canonical.ts,canonical.test.ts}`
- Create: `atlas/src/app/KnowledgeHome.tsx`
- Modify: `atlas/src/app/{AtlasApp.tsx,ChapterReader.tsx,routes.ts,routes.test.ts,ReaderApp.test.tsx}`
- Create/Modify: focused `atlas/src/app/**/*.test.tsx`
- Modify: Atlas CSS files that own reader panels

- [ ] **Step 1: Add failing Rust corpus/search registration tests**

  In corpus tests, require:

  ```rust
  assert!(corpus.reader_routes.iter().any(|route| {
      route.route_id == "knowledge"
          && route.canonical_markdown_path == "knowledge/harp_knowledge_home.md"
  }));
  ```

  Require every DarwinX file to be present in `corpus.documents` and assert
  `search query "population selection"` returns
  `knowledge/darwinx/darwinx_index.md`. Update the CLI search fixture to create
  a DarwinX index and knowledge home fixture, then assert both route paths.

- [ ] **Step 2: Run focused tests to prove registration is missing**

  Run:

  ```sh
  cargo test -p harp corpus::tests::compiles_reader_routes_from_canonical_markdown --lib -- --test-threads=1
  cargo test -p harp --test cli search_refresh_status_and_query_share_a_digest_receipt -- --test-threads=1
  ```

  Expected: failure because neither the new reader route nor DarwinX search
  root exists.

- [ ] **Step 3: Extend canonical document metadata**

  In `mod.rs`, add:

  ```rust
  #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
  pub(super) struct DocumentMetadata {
      pub(super) id: String,
      pub(super) kind: String,
      pub(super) status: String,
      pub(super) tags: Vec<String>,
      pub(super) confidence: String,
      pub(super) mode: Option<String>,
      pub(super) source_ids: Vec<String>,
      pub(super) coverage_keys: Vec<String>,
  }
  ```

  Add `metadata: DocumentMetadata` to `CanonicalDocument`. In
  `contracts.rs`, parse YAML frontmatter through a strict, bounded parser:
  reject duplicate keys, nested values outside one-line lists, unknown
  property types, invalid IDs, invalid confidence, and missing required
  baseline keys for managed documents. Canonical packet-specific keys listed
  in the design remain optional. Frontmatter is never trusted to route outside
  the source path chosen by the corpus registry.

- [ ] **Step 4: Register all discoverable surfaces**

  Update `READER_ROUTES` to add:

  ```rust
  (
      "knowledge",
      "Knowledge",
      "knowledge/harp_knowledge_home.md",
  ),
  ```

  Add all nine DarwinX Markdown files and the vault home to
  `AUXILIARY_DOCUMENTS` with stable IDs. Add
  `("knowledge/darwinx", "canonical-markdown", "md")` to search roots. Update
  source/corpus fixture setup so tests include the new route. Do not modify
  `REQUIRED_CHAPTERS`, retained concepts, system registry, or the 16-system
  count.

- [ ] **Step 5: Add strict Atlas metadata contracts**

  In `atlas/src/content/types.ts`, add the TypeScript equivalent:

  ```ts
  export type DocumentMetadata = {
    id: string;
    kind: string;
    status: string;
    tags: string[];
    confidence: "low" | "medium" | "high";
    mode: string | null;
    source_ids: string[];
    coverage_keys: string[];
  };
  ```

  Add `metadata: DocumentMetadata` to `CanonicalDocument`. In
  `contracts.ts`, parse every metadata field from `unknown`, reject absent or
  invalid values, and keep the existing schema version only if this is
  backward-compatible within the repository. If a schema version bump is
  needed, update Rust, TypeScript, fixtures, static export receipt, and product
  contract together.

- [ ] **Step 6: Implement the Atlas knowledge route**

  Add `KnowledgeHome.tsx` that derives cards from `canonicalCorpus.documents`
  and `metadata`; it must not contain a hardcoded packet list. It groups:

  - `research-index`, `benchmark-guide`, and `reference-architecture` as
    reader routes;
  - `technical-deep-dive` and `system-reading` as deep dives;
  - source/claim/missing ledgers as evidence surfaces.

  Each card opens the existing document route, shows title, status, confidence,
  and tags, and is keyboard accessible. In `ChapterReader.tsx`, add a metadata
  strip using the parsed fields and retain the existing receipt digest.
  Style callouts and embed fallbacks in the owning Atlas CSS module. Do not
  make embed fallbacks inline arbitrary files.

- [ ] **Step 7: Write and run Atlas tests**

  Add tests that:

  ```ts
  expect(parseCanonicalCorpus(corpusWithBadMetadata)).toThrow(
    "Document 0 metadata confidence",
  );
  expect(renderKnowledgeHome()).toHaveTextContent("DarwinX");
  expect(renderKnowledgeHome()).toHaveTextContent("Claim and evidence");
  expect(routeFor({ kind: "reader", routeId: "knowledge" })).toBe("#knowledge");
  ```

  Add one renderer fixture proving a wiki link becomes
  `href="#documents/darwinx-index"` and a callout is accessible as `role="note"`.
  Preserve current route tests and assert the 16-system invariant unchanged.

  Run:

  ```sh
  cargo test -p harp corpus::tests --lib -- --test-threads=1
  cargo test -p harp --test cli -- --test-threads=1
  cd atlas && corepack pnpm run lint && corepack pnpm run typecheck && corepack pnpm run test
  ```

- [ ] **Step 8: Commit corpus/Atlas registration**

  ```sh
  git add crates/harp/src/corpus crates/harp/src/search.rs crates/harp/tests
  git add atlas/src/app atlas/src/content atlas/src/**/*.css
  git diff --cached --check
  git commit -m "feat: register Obsidian knowledge in Atlas and search" \
    -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
  ```

### Task 8: Document, regenerate, seal, and verify the complete presentation

**Files:**
- Modify: `README.md`
- Modify: `RESOURCES.md`
- Modify: `AGENTS.md`
- Modify: `docs/{product-contract.md,contributing.md}`
- Modify: `atlas/src/content/generated/corpus.json`
- Modify: `atlas/dist/harp-atlas.html`
- Modify: `atlas/dist/harp-atlas.receipt.json`
- Modify: `docs/import-receipt.md`
- Test: full `mise run verify`

- [ ] **Step 1: Update reader and contributor documentation**

  Add concise instructions:

  ```text
  Open `/Users/.../harp` (the repository root) as the Obsidian vault.
  Start at `knowledge/harp_knowledge_home.md`.
  Apply the portable profile with:
  python3 tools/obsidian/apply_profile.py --vault "$(git rev-parse --show-toplevel)"
  ```

  Explain that:

  - `knowledge/` owns technical prose;
  - `.obsidian/` is ignored personal state;
  - `tools/obsidian/profile/` is portable, reviewed configuration;
  - note/artifact navigation uses root-qualified wikilinks;
  - exact immutable raw-evidence line locators retain their dual Markdown
    locator suffix;
  - new packets require corpus route, auxiliary documents, search root,
    metadata, and packet tests; and
  - the five locally installed skills are available after a TRAE CLI restart.

- [ ] **Step 2: Regenerate derived outputs**

  Run:

  ```sh
  cargo run -p harp -- build
  cd atlas
  corepack pnpm run test:export
  ```

  Confirm that only:

  ```text
  atlas/src/content/generated/corpus.json
  atlas/dist/harp-atlas.html
  atlas/dist/harp-atlas.receipt.json
  ```

  change as generated products, plus expected authored/documentation changes.

- [ ] **Step 3: Run staged-surface verification**

  Stage only the intended payload, shelter unrelated work, then verify:

  ```sh
  git add \
    README.md RESOURCES.md AGENTS.md docs \
    evidence/implementations/obsidian_skills \
    evidence/implementations/manifest.tsv \
    crates/harp atlas knowledge tools scripts .gitignore
  git stash push --keep-index -u -m "obsidian-presentation-unrelated-work"
  mise run verify
  git stash pop
  ```

  If another active worktree has changed a shared file, do not overwrite it.
  Rebase or merge the finished worktree only after reviewing the conflict.

- [ ] **Step 4: Refresh the import receipt last**

  After every generated byte and documentation change is settled, run:

  ```sh
  cargo run -p harp -- repository verify
  ```

  Copy the reported payload SHA-256 into `docs/import-receipt.md`, then rerun:

  ```sh
  cargo run -p harp -- repository verify
  mise run verify
  ```

  Expected: repository verification and complete release gate pass with the
  refreshed receipt.

- [ ] **Step 5: Perform local Obsidian smoke check when available**

  If `obsidian` exists and a local app instance is running, open the repository
  root and verify manually:

  1. `knowledge/harp_knowledge_home.md` renders with callouts and routes;
  2. `knowledge/obsidian/harp_knowledge.base` shows reader routes;
  3. `knowledge/obsidian/harp_knowledge_map.canvas` has no missing file nodes;
  4. a DarwinX claim opens its ledger H2;
  5. a wiki source artifact opens from the vault; and
  6. a PDF page embed opens as expected.

  If no Obsidian CLI/application is available, record “not available; CI
  remains parser/asset validated” and do not treat that absence as a test pass.

- [ ] **Step 6: Commit the sealed integration**

  ```sh
  git add \
    README.md RESOURCES.md AGENTS.md docs \
    atlas/src/content/generated/corpus.json \
    atlas/dist/harp-atlas.html \
    atlas/dist/harp-atlas.receipt.json
  git diff --cached --check
  git commit -m "build: seal Obsidian knowledge presentation" \
    -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
  ```

## Plan Self-Review

### Spec coverage

| Approved design requirement | Plan task |
|---|---|
| Full pinned MIT vendor and all five local skills | Task 1 |
| Root vault and ignored personal state | Tasks 6 and 8 |
| Typed safe wiki parser/resolver | Task 2 |
| Claims, evidence locators, and packet contracts | Task 3 |
| Full managed-note link migration and metadata | Tasks 4 and 5 |
| Nova-style reader/evidence split and human prose | Tasks 5 and 6 |
| Vault home, Base, Canvas, portable profile | Task 6 |
| DarwinX corpus/search/Atlas discoverability | Tasks 3 and 7 |
| Atlas metadata/callout/embed rendering | Tasks 2 and 7 |
| Regeneration, import receipt, and complete release gate | Task 8 |

The only intentional exception to “all internal links are wikilinks” is the
exact-line locator suffix for immutable raw evidence, which cannot receive an
Obsidian block ID without violating source fidelity.

### Placeholder scan

The plan contains no unresolved feature placeholders. Every implementation
slice names paths, tests, API shapes, expected behavior, and commands.

### Consistency review

The shared type names are `WikiLink`, `WikiSubpath`, `ResolvedWikiLink`, and
`DocumentMetadata` across all tasks. The accepted claim terminology remains
`SOURCE CLAIM`, not `CLAIM`. The vault root is always the repository root;
`knowledge/` remains the prose root.

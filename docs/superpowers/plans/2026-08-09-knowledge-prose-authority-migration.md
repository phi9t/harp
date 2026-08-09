# Knowledge Prose Authority Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `knowledge/` the sole authority for Harp technical Markdown while retaining `content/` solely for structured contracts and diagnostics.

**Architecture:** Move the 135 tracked `content/**/*.md` files to the path-preserving managed tree `knowledge/rsi/`, plus move the Weng field note into that tree. Keep path-bearing JSON and TSV inputs under `content/`, but rewrite their canonical Markdown values to `knowledge/rsi/...`. The Rust corpus compiler and FTS search use explicit managed knowledge roots, reject Markdown below `content/`, and continue to emit the existing Atlas generated paths.

**Tech Stack:** Rust 1.92 (`harp` corpus/search/repository verifier), Python 3 standard library (Weng projector and one-shot link rewrite), TypeScript/React 19 Atlas, Vite/Vitest, Git LFS, mise.

---

## Scope and file map

### Canonical prose moves

Move every tracked Markdown file selected by:

```sh
git ls-files -z -- 'content/*.md' 'content/**/*.md'
```

from `content/<relative>.md` to `knowledge/rsi/<relative>.md`. At the time of
planning this is 135 files, covering root RSI material, `chapters/`,
`concepts/`, `systems/`, `weng/`, `weng-sources/`, `lessons/`, `sicp/`, and
`sources/`.

Also move:

```text
knowledge/lw_rsi_harness.md
-> knowledge/rsi/lw_rsi_harness.md
```

Keep these established packets at their current roots:

```text
knowledge/darwin_godel_machine/
knowledge/meta_harness/
knowledge/harness_benchmarks/
```

Do not add the AHE packet in this migration.

### Structured inputs that stay in `content/`

Do not move these files or the diagnostics subtree:

```text
content/coverage-map.tsv
content/harness_deep_dive_manifest.tsv
content/lessons.json
content/retained-concepts.tsv
content/systems/system_readings.json
content/weng-claim-ladder.json
content/weng-comparison-matrix.tsv
content/weng-course-status.json
content/weng-reading-map.json
content/weng-source-cards.tsv
content/sources/*.tsv
content/diagnostics/**/*.json
```

Rewrite only their values that designate canonical Markdown or source-card
paths. Preserve paths for JSON/TSV contracts, diagnostics, and evidence.

### Code, tests, projections, and docs

| Surface | Files | Required result |
| --- | --- | --- |
| Filesystem boundary | `crates/harp/src/fs.rs` | A symlink-rejecting recursive regular-file extension listing API used by corpus validation. |
| Corpus contract | `crates/harp/src/corpus/{mod.rs,contracts.rs,lessons.rs,render.rs}` | `knowledge/rsi/` is the RSI Markdown root; `content/**/*.md` fails corpus validation. |
| Search | `crates/harp/src/search.rs` | Index `knowledge/rsi/` plus the three registered packet roots, never the entire `knowledge/` tree. |
| Repository verifier | `crates/harp/src/repository.rs`, `docs/import-map.tsv` | Permit the legitimate current `knowledge/rsi/` tree and update imported Markdown destinations. |
| Tests | `crates/harp/src/corpus/tests.rs`, `crates/harp/tests/{cli.rs,dgm_knowledge_packet.rs,weng_teaching_curriculum.rs,credible_docs_style.rs}`, `atlas/src/content/canonical.test.ts` | New paths compile, old Markdown placement fails, packets link correctly, loose knowledge files are excluded from search. |
| Weng projection | `scripts/build_weng_course.py`, `reference/*.html`, `lessons/*.html` only if regenerated output changes | Use `knowledge/rsi/` only for canonical Markdown/card route values while retaining structured inputs under `content/`. |
| Derived Atlas | `atlas/src/content/generated/corpus.json`, `atlas/dist/harp-atlas.html`, `atlas/dist/harp-atlas.receipt.json` | Regenerated from new canonical paths; TypeScript module paths remain unchanged. |
| Contributor docs | `AGENTS.md`, `README.md`, `docs/product-contract.md`, `atlas/README.md`, `RESOURCES.md` | State the new authority model and correct reader links. |

Historical design and plan documents remain historical records. Do not rewrite
old `docs/superpowers/plans/` or `docs/superpowers/specs/` just because they
describe the previous layout.

## Invariants

| Boundary | Structural invariant | Enforcement |
| --- | --- | --- |
| Technical prose | All canonical Markdown is below explicit `knowledge/` roots; `content/` contains no `.md` file at any depth. | Corpus `load()` calls a symlink-safe `validate_no_content_markdown()` before reading registries. |
| Controlled corpus membership | A loose Markdown file under `knowledge/` is not automatically canonical or searchable. | Corpus uses registered routes/registries; search uses a fixed root list. |
| Structured contracts | Registry JSON/TSV and diagnostics remain beneath `content/`. | Keep contract constants and diagnostic paths unchanged; rewrite only Markdown-valued fields. |
| Evidence | Captured files remain untouched. | No `evidence/` move or text rewrite; only existing source locators continue to resolve. |
| Generated output | Atlas artifacts remain generated under `atlas/src/content/generated/` and `atlas/dist/`. | Existing safe output validation, static export receipt, and repository verification remain enabled. |
| Symlinks | Neither the no-Markdown check nor search may traverse a symlink. | Reuse `HeldDirectory` path protections and add a recursive extension listing that rejects every symlink encountered. |

### Task 1: Add the managed-root and no-Markdown contract

**Files:**
- Modify: `crates/harp/src/fs.rs`
- Modify: `crates/harp/src/corpus/mod.rs`
- Modify: `crates/harp/src/corpus/contracts.rs`
- Modify: `crates/harp/src/corpus/lessons.rs`
- Modify: `crates/harp/src/corpus/render.rs`
- Modify: `crates/harp/src/corpus/tests.rs`
- Modify: `crates/harp/tests/cli.rs`
- Modify: `atlas/src/content/canonical.test.ts`

- [ ] **Step 1: Change corpus test fixtures to the target canonical root and add failing ownership tests**

In `crates/harp/src/corpus/tests.rs`, make `fixture()` create:

```rust
let root = repo.path().join("knowledge/rsi");
fs::create_dir_all(root.join("chapters")).unwrap();
fs::create_dir_all(root.join("concepts")).unwrap();
```

Replace every fixture-only canonical Markdown path with its
`knowledge/rsi/...` equivalent while leaving `content/retained-concepts.tsv`,
`content/lessons.json`, `content/systems/system_readings.json`, and
`content/diagnostics/**/*.json` unchanged.

Add these tests:

```rust
#[test]
fn rejects_markdown_beneath_the_structured_content_root() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    fs::write(repo.path().join("content/forbidden.md"), "# Forbidden\n").unwrap();

    let error = compile(repo.path()).unwrap_err();

    assert_eq!(error.code(), "knowledge.rsi.content_markdown");
    assert!(error.message.contains("content/forbidden.md"));
}

#[cfg(unix)]
#[test]
fn rejects_symlinks_while_scanning_the_structured_content_root() {
    use std::os::unix::fs::symlink;

    let repo = fixture();
    write_complete_fixture(repo.path());
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("forbidden.md"), "# Forbidden\n").unwrap();
    symlink(outside.path(), repo.path().join("content/linked")).unwrap();

    let error = compile(repo.path()).unwrap_err();

    assert_eq!(error.code(), "fs.symlink");
}
```

Change the canonical-path unit assertion to the target path:

```rust
let row =
    "aflow\tsystem-reading\tknowledge/rsi/systems/aflow.md\t\tharness-search";
assert_eq!(entry.canonical_markdown_path, "knowledge/rsi/systems/aflow.md");
```

In `crates/harp/tests/cli.rs`, rewrite the temporary search fixture to create:

```rust
fs::create_dir_all(repo.path().join("knowledge/rsi")).expect("RSI knowledge");
fs::create_dir_all(repo.path().join("knowledge/meta_harness")).expect("packet");
fs::create_dir_all(repo.path().join("knowledge/darwin_godel_machine"))
    .expect("packet");
fs::create_dir_all(repo.path().join("knowledge/harness_benchmarks"))
    .expect("packet");
fs::create_dir_all(repo.path().join("knowledge/private")).expect("loose knowledge");
```

Use these documents:

```rust
fs::write(
    repo.path().join("knowledge/rsi/intro.md"),
    "# Recursive improvement\n\nA bounded recursive improvement loop.\n",
)
.expect("RSI document");
fs::write(
    repo.path().join("knowledge/private/draft.md"),
    "# Private draft\n\nThis must not enter the search index.\n",
)
.expect("loose document");
```

Make the existing query assertion expect
`"path":"knowledge/rsi/intro.md"`, query `"private draft"`, and assert that
the serialized result does not contain `knowledge/private/draft.md`.

In `atlas/src/content/canonical.test.ts`, replace the helper and fixture paths:

```ts
const chapterPath = (id: string): string =>
  `knowledge/rsi/chapters/${id}.md`;
```

and replace every `content/weng/` and `content/systems/` expected path with
`knowledge/rsi/weng/` and `knowledge/rsi/systems/`, respectively.

- [ ] **Step 2: Run the focused tests to prove the old implementation fails**

Run:

```sh
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo test -p harp --test cli
cd atlas && corepack pnpm run test -- canonical.test.ts
```

Expected: failures because production constants and path guards still accept
`content/.../*.md`, no recursive Markdown ownership check exists, and search
still indexes `content/`.

- [ ] **Step 3: Add a symlink-safe regular-file extension traversal**

In `crates/harp/src/fs.rs`, add this public method inside `impl HeldDirectory`:

```rust
pub fn regular_files_with_extension(
    &self,
    relative: &Path,
    extension: &str,
    label: &str,
) -> Result<Vec<PathBuf>, AppError> {
    let root = self.resolve_read_target(relative, label)?;
    let metadata = fs::symlink_metadata(&root)
        .map_err(|error| AppError::io("fs.metadata", label, error))?;
    if !metadata.is_dir() {
        return Err(AppError::invalid_input(
            "fs.type",
            format!("{label} must be a real directory"),
        ));
    }

    let mut files = Vec::new();
    self.collect_regular_files_with_extension(relative, extension, label, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_regular_files_with_extension(
    &self,
    relative: &Path,
    extension: &str,
    label: &str,
    files: &mut Vec<PathBuf>,
) -> Result<(), AppError> {
    let directory = self.resolve_read_target(relative, label)?;
    let mut entries = fs::read_dir(&directory)
        .map_err(|error| AppError::io("fs.read_dir", label, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| AppError::io("fs.read_dir", label, error))?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let name = entry.file_name();
        let child = relative.join(&name);
        let metadata = fs::symlink_metadata(entry.path())
            .map_err(|error| AppError::io("fs.metadata", label, error))?;
        if metadata.file_type().is_symlink() {
            return Err(AppError::invalid_input(
                "fs.symlink",
                format!("{label} cannot traverse a symlink: {}", child.display()),
            ));
        }
        if metadata.is_dir() {
            self.collect_regular_files_with_extension(&child, extension, label, files)?;
        } else if metadata.is_file()
            && child.extension().and_then(|value| value.to_str()) == Some(extension)
        {
            files.push(child);
        }
    }
    Ok(())
}
```

This method deliberately starts from `resolve_read_target`, recursively uses
normal relative paths, and rejects a symlink rather than following it.

- [ ] **Step 4: Make the corpus use `knowledge/rsi/` and reject Markdown in `content/`**

In `crates/harp/src/corpus/mod.rs`, add:

```rust
pub(super) const RSI_MARKDOWN_ROOT: &str = "knowledge/rsi";
pub(super) const CONTENT_ROOT: &str = "content";
```

Keep the existing `COVERAGE_PATH`, registry paths, and diagnostics paths under
`content/`. Rewrite every Markdown path in `READER_ROUTES`,
`AUXILIARY_DOCUMENTS`, and `REQUIRED_CHAPTERS` to start with
`knowledge/rsi/`.

At the top of `contracts::load`, before `load_retained_concepts`, call:

```rust
validate_no_content_markdown(repository)?;
```

Add the helper in `crates/harp/src/corpus/contracts.rs`:

```rust
fn validate_no_content_markdown(repository: &HeldDirectory) -> Result<(), AppError> {
    let markdown = repository.regular_files_with_extension(
        Path::new(CONTENT_ROOT),
        "md",
        "structured RSI content",
    )?;
    if let Some(path) = markdown.first() {
        return Err(invalid(
            "knowledge.rsi.content_markdown",
            format!(
                "technical Markdown must live under knowledge/, not {}",
                path.display()
            ),
        ));
    }
    Ok(())
}
```

Replace path guards exactly:

```rust
system.canonical_markdown_path.starts_with("knowledge/rsi/systems/")
section.companion_path.starts_with("knowledge/rsi/weng/")
lesson.canonical_markdown_path.starts_with("knowledge/rsi/lessons/")
```

In `parse_coverage_row` and `normalize_canonical_path`, require:

```rust
cells[2].starts_with("knowledge/rsi/concepts/")
cells[2].starts_with("knowledge/rsi/systems/")
value.starts_with("knowledge/rsi/chapters/")
    || value.starts_with("knowledge/rsi/concepts/")
    || value.starts_with("knowledge/rsi/systems/")
```

In `crates/harp/src/corpus/render.rs`, replace the system-reading branch with:

```rust
if path.starts_with("knowledge/rsi/systems/") {
    return Some(stem.to_owned());
}
```

Preserve `normalize_link_path` support for both `content` and `knowledge`:
Markdown needs to link to structured JSON/TSV contracts in `content/`, even
though no Markdown may reside there.

- [ ] **Step 5: Run focused unit and integration tests**

Run:

```sh
cargo fmt --all -- --check
cargo test -p harp corpus::tests::rejects_markdown_beneath_the_structured_content_root \
  --lib -- --exact --test-threads=1
cargo test -p harp corpus::tests::rejects_symlinks_while_scanning_the_structured_content_root \
  --lib -- --exact --test-threads=1
```

Expected: the new fixture-level ownership tests pass: an actual
`content/*.md` fixture fails with `knowledge.rsi.content_markdown`, and a
`content/` symlink fails with `fs.symlink`.

Do **not** run the workspace-wide corpus, CLI, or Atlas tests yet, and do not
commit this intermediate state. The new guard correctly rejects the still
unmoved production `content/**/*.md` corpus. Task 2 must move that corpus in
the same atomic staged change; it then runs the whole suite.

### Task 2: Move canonical Markdown and rebase all path-bearing contracts

**Files:**
- Move: all tracked `content/**/*.md` → `knowledge/rsi/**`
- Move: `knowledge/lw_rsi_harness.md` → `knowledge/rsi/lw_rsi_harness.md`
- Modify: `content/coverage-map.tsv`
- Modify: `content/harness_deep_dive_manifest.tsv`
- Modify: `content/lessons.json`
- Modify: `content/systems/system_readings.json`
- Modify: `content/weng-claim-ladder.json`
- Modify: `content/weng-reading-map.json`
- Modify: `content/weng-source-cards.tsv`
- Modify: `docs/import-map.tsv`
- Modify: `crates/harp/src/repository.rs`
- Modify: `crates/harp/src/corpus/tests.rs`
- Modify: `crates/harp/tests/{cli.rs,credible_docs_style.rs,dgm_knowledge_packet.rs,weng_teaching_curriculum.rs}`
- Modify: `atlas/src/app/ReaderApp.test.tsx`

- [ ] **Step 1: Move only tracked canonical Markdown with Git-aware renames**

Run this from the repository root:

```sh
set -euo pipefail
git ls-files -z -- 'content/*.md' 'content/**/*.md' |
  while IFS= read -r -d '' source; do
    destination="knowledge/rsi/${source#content/}"
    mkdir -p "$(dirname "$destination")"
    git mv "$source" "$destination"
  done
git mv knowledge/lw_rsi_harness.md knowledge/rsi/lw_rsi_harness.md

test "$(git ls-files 'content/*.md' 'content/**/*.md' | wc -l | tr -d ' ')" = 0
test "$(git ls-files 'knowledge/rsi/**/*.md' 'knowledge/rsi/*.md' | wc -l | tr -d ' ')" = 136
```

Do not move any non-Markdown `content/` file. Do not touch untracked
`knowledge/Untitled.md`, `.obsidian/`, pasted images, `CONTEXT.md`, or the
untracked prior essay plan/spec.

- [ ] **Step 2: Rewrite only registry fields whose values designate canonical Markdown**

Use field-aware transformations; do not globally replace `content/`, because
diagnostic JSON, source registries, and Weng machine inputs remain there.

Apply these exact replacements:

```text
content/coverage-map.tsv:
  canonical_markdown_path values
  content/{chapters,concepts,systems}/*.md
  -> knowledge/rsi/{chapters,concepts,systems}/*.md

content/harness_deep_dive_manifest.tsv:
  any *.md canonical route value under content/
  -> the equivalent knowledge/rsi/ route

content/lessons.json:
  every canonical_markdown_path value
  content/lessons/*.md -> knowledge/rsi/lessons/*.md

content/systems/system_readings.json:
  every canonical_markdown_path value
  content/systems/*.md -> knowledge/rsi/systems/*.md
  leave diagnostic_case_path values under content/diagnostics/ unchanged

content/weng-reading-map.json:
  every companion_path value
  content/weng/*.md -> knowledge/rsi/weng/*.md

content/weng-claim-ladder.json:
  example_route and canonical_concept_links[].route values ending in .md
  content/... -> knowledge/rsi/...

content/weng-source-cards.tsv:
  card_path and canonical_route fields ending in .md
  content/... -> knowledge/rsi/...
```

Use a JSON round-trip for the three JSON structures so only the named fields
change and a trailing newline remains. For example, use this exact
`content/lessons.json` rewrite pattern:

```sh
node --input-type=module <<'NODE'
import { readFile, writeFile } from "node:fs/promises";

const path = "content/lessons.json";
const document = JSON.parse(await readFile(path, "utf8"));
for (const lesson of document.lessons) {
  lesson.canonical_markdown_path =
    lesson.canonical_markdown_path.replace(/^content\//, "knowledge/rsi/");
}
await writeFile(path, JSON.stringify(document, null, 2) + "\n");
NODE
```

For TSVs, parse the header to find named columns, update only values ending in
`.md` whose value begins `content/`, and preserve tab order and row order.

Update `docs/import-map.tsv` destinations for each row whose current
destination is a Markdown path below `content/`:

```text
content/<relative>.md -> knowledge/rsi/<relative>.md
```

Leave original percent-encoded `source_path_percent_encoded` values unchanged:
they record the imported source history, not the current canonical layout.

In `crates/harp/src/repository.rs`, remove this old forbidden-reference
pattern:

```rust
["knowledge", "rsi"].join("/"),
```

It previously rejected source-repository coupling, but now names Harp's
legitimate canonical root. Keep all other forbidden-reference patterns.

Add a repository unit test that initializes a temporary Git repo, stages
`knowledge/rsi/intro.md`, and asserts:

```rust
assert_eq!(verify_forbidden_references(repo.path()).unwrap(), 0);
```

This prevents a future source-coupling scan from reintroducing a ban on the
current product-owned path.

- [ ] **Step 3: Rewrite local links according to their resolved destination, not by string prefix**

Create `target/rewrite_knowledge_links.py` as a temporary migration helper,
run it once, inspect the diff, then delete it before staging. The helper must:

1. enumerate only `knowledge/rsi/**/*.md` and the three registered packet
   roots;
2. calculate each local target against the **pre-move** source path;
3. map a resolved `content/<relative>.md` target to
   `knowledge/rsi/<relative>.md`;
4. retain resolved targets in `content/`, `evidence/`, `labs/`, `crates/`, and
   registered `knowledge/` packets;
5. recompute a relative destination from the **post-move** source directory;
6. preserve query strings, anchors, external URLs, mailto links, and link
   titles; and
7. fail rather than silently rewriting an unresolvable local target.

Use this complete helper:

```python
from __future__ import annotations

import os
import re
from pathlib import Path, PurePosixPath

ROOT = Path.cwd()
MOVED_PREFIX = PurePosixPath("content")
TARGET_PREFIX = PurePosixPath("knowledge/rsi")
PACKETS = (
    PurePosixPath("knowledge/darwin_godel_machine"),
    PurePosixPath("knowledge/meta_harness"),
    PurePosixPath("knowledge/harness_benchmarks"),
)
INLINE = re.compile(r"(?P<prefix>!?\[[^\]]*\]\()(?P<target><[^>]+>|[^)\s]+)(?P<suffix>(?:\s+[^)]*)?\))")
REFERENCE = re.compile(r"(?m)^(?P<prefix>\[[^\]]+\]:\s*)(?P<target><[^>]+>|[^\s]+)(?P<suffix>(?:\s+.*)?)$")


def is_external(target: str) -> bool:
    return (
        target.startswith("#")
        or target.startswith(("http://", "https://", "mailto:", "data:"))
        or "://" in target
    )


def split_fragment(target: str) -> tuple[str, str]:
    path, separator, fragment = target.partition("#")
    return path, separator + fragment if separator else ""


def normalize(base: PurePosixPath, target: str) -> PurePosixPath:
    parts: list[str] = []
    incoming = PurePosixPath(target.lstrip("/")) if target.startswith("/") else base / target
    for part in incoming.parts:
        if part in ("", "."):
            continue
        if part == "..":
            if not parts:
                raise ValueError(f"link escapes repository: {target}")
            parts.pop()
        else:
            parts.append(part)
    resolved = PurePosixPath(*parts)
    if not resolved.parts or resolved.parts[0] not in {"content", "knowledge", "evidence", "labs", "crates"}:
        raise ValueError(f"link is outside supported repository roots: {target}")
    return resolved


def moved(path: PurePosixPath) -> PurePosixPath:
    if path.parts[:1] == MOVED_PREFIX.parts and path.suffix == ".md":
        return TARGET_PREFIX / path.relative_to(MOVED_PREFIX)
    return path


def old_source(path: PurePosixPath) -> PurePosixPath:
    if path.parts[:2] == TARGET_PREFIX.parts:
        return MOVED_PREFIX / path.relative_to(TARGET_PREFIX)
    return path


def rewrite_target(source: PurePosixPath, raw: str) -> str:
    bracketed = raw.startswith("<") and raw.endswith(">")
    value = raw[1:-1] if bracketed else raw
    if is_external(value):
        return raw
    path_part, fragment = split_fragment(value)
    if not path_part:
        return raw
    resolved = moved(normalize(old_source(source).parent, path_part))
    if not (ROOT / resolved).exists():
        raise ValueError(f"{source}: target does not exist after migration: {raw}")
    relative = os.path.relpath(resolved, source.parent).replace(os.sep, "/")
    rewritten = relative + fragment
    return f"<{rewritten}>" if bracketed else rewritten


def replace(match: re.Match[str], source: PurePosixPath) -> str:
    return match.group("prefix") + rewrite_target(source, match.group("target")) + match.group("suffix")


def rewrite_file(path: Path) -> None:
    source = PurePosixPath(path.relative_to(ROOT).as_posix())
    text = path.read_text(encoding="utf-8")
    text = INLINE.sub(lambda match: replace(match, source), text)
    text = REFERENCE.sub(lambda match: replace(match, source), text)
    path.write_text(text, encoding="utf-8")


paths = sorted((ROOT / "knowledge/rsi").rglob("*.md"))
for packet in PACKETS:
    paths.extend(sorted((ROOT / packet).rglob("*.md")))
for path in paths:
    rewrite_file(path)
```

Run:

```sh
python3 target/rewrite_knowledge_links.py
rm target/rewrite_knowledge_links.py
git diff --check
```

Then make these explicit content edits that are semantic, not just path
rewrites:

- In every DGM packet document, replace the old authority notice:

  ```markdown
  > This file is a learning projection. Canonical claims live under `content/`;
  > primary-source captures and pinned implementation files live under
  > `evidence/`.
  ```

  with:

  ```markdown
  > This file is a maintained Harp technical packet. It separates Harp's
  > synthesis from primary-source claims; primary-source captures and pinned
  > implementation files live under `evidence/`.
  ```

- Update `AUTHORITY_NOTICE` in
  `crates/harp/tests/dgm_knowledge_packet.rs` to the same exact text.
- In DGM maintenance material, describe `knowledge/rsi/` as the managed RSI
  canonical corpus and keep references to machine-readable source registries
  under `content/sources/`.
- In `knowledge/rsi/lw_rsi_harness.md`, preserve the existing field-note
  content but make all moved Markdown links resolve within `knowledge/rsi/`;
  retain the Weng reading-map JSON link under `content/`.
- In the Meta-Harness and benchmark packets, rewrite canonical article links
  to `../rsi/...` but retain links to `content/sources/*.tsv` and other
  structured inputs.

- [ ] **Step 4: Update test fixtures and assertions that encode old physical paths**

Apply the same target-root conversion to:

```text
crates/harp/src/corpus/tests.rs
crates/harp/tests/cli.rs
crates/harp/tests/credible_docs_style.rs
crates/harp/tests/dgm_knowledge_packet.rs
crates/harp/tests/weng_teaching_curriculum.rs
atlas/src/app/ReaderApp.test.tsx
```

Required assertion changes include:

```rust
// dgm_knowledge_packet.rs local link fixture
"[local](../rsi/systems/dgm.md#problem-and-rsi-relevance)\n"
"[dgm]: ../rsi/systems/dgm.md \"DGM\"\n"
```

and:

```ts
expect(document.canonical_markdown_path).toBe(
  "knowledge/rsi/chapters/evaluation-promotion-containment.md",
);
```

Update Weng curriculum test helper calls so source-card validation uses:

```rust
assert_resolved_file_beneath(
    &row["card_path"],
    "knowledge/rsi/weng-sources",
    "card_path",
);
```

Do not replace assertions for diagnostic JSON or source registries that
correctly remain under `content/`.

- [ ] **Step 5: Verify the moved corpus before derived output regeneration**

Run:

```sh
git diff --check
test "$(find content -type f -name '*.md' | wc -l | tr -d ' ')" = 0
test "$(find knowledge/rsi -type f -name '*.md' | wc -l | tr -d ' ')" = 136
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo test -p harp --test cli
cargo test -p harp --test dgm_knowledge_packet
cargo test -p harp --test weng_teaching_curriculum
cargo test -p harp --test credible_docs_style
cargo run -p harp -- check
```

Expected: all canonical Markdown references resolve beneath `knowledge/rsi/`;
`content/` still supplies the structured registries and diagnostics; corpus
counts remain unchanged; and no local link points to a removed file.

### Task 3: Repoint search and regenerate teaching and Atlas projections

**Files:**
- Modify: `crates/harp/src/search.rs`
- Modify: `crates/harp/tests/cli.rs`
- Modify: `scripts/build_weng_course.py`
- Modify: `reference/weng-source-cards.html`
- Modify: `reference/weng-harness-map.html`
- Modify: `reference/rsi-claim-ladder.html`
- Modify: `reference/harness-comparison-matrix.html`
- Modify: `atlas/src/content/generated/corpus.json`
- Modify: `atlas/dist/harp-atlas.html`
- Modify: `atlas/dist/harp-atlas.receipt.json`

- [ ] **Step 1: Replace search roots with the explicit managed knowledge roots**

In `crates/harp/src/search.rs`, replace:

```rust
let roots = [
    ("content", "canonical-markdown", "md"),
    ("knowledge/meta_harness", "canonical-markdown", "md"),
    ("evidence/weng/text", "weng-source", "txt"),
    ("evidence/rlm/text", "rlm-source", "txt"),
];
```

with:

```rust
let roots = [
    ("knowledge/rsi", "canonical-markdown", "md"),
    (
        "knowledge/darwin_godel_machine",
        "canonical-markdown",
        "md",
    ),
    ("knowledge/meta_harness", "canonical-markdown", "md"),
    (
        "knowledge/harness_benchmarks",
        "canonical-markdown",
        "md",
    ),
    ("evidence/weng/text", "weng-source", "txt"),
    ("evidence/rlm/text", "rlm-source", "txt"),
];
```

Keep the existing `WalkDir` symlink rejection. Do not replace this list with a
generic `knowledge/` walk; that would incorrectly index editor state and
unregistered drafts.

- [ ] **Step 2: Update Weng projector paths and its deterministic self-tests**

In `scripts/build_weng_course.py`, retain these structured input constants:

```python
MATRIX = ROOT / "content/weng-source-cards.tsv"
COMPARISON_MATRIX = ROOT / "content/weng-comparison-matrix.tsv"
CLAIM_LADDER = ROOT / "content/weng-claim-ladder.json"
STATUS = ROOT / "content/weng-course-status.json"
```

Replace the Markdown card root with:

```python
CARD_ROOT = ROOT / "knowledge/rsi/weng-sources"
```

Replace each `SECTION_METADATA` canonical companion route:

```text
../content/weng/<name>.md
-> ../knowledge/rsi/weng/<name>.md
```

In rendered reference-page introduction text, make the distinction explicit:

```html
Generated from structured inputs under <code>content/</code> and canonical
Markdown under <code>knowledge/rsi/weng-sources/</code>.
```

Change rendered canonical-card and claim-ladder links from `../content/...md`
to `../knowledge/rsi/...md`. Retain `../content/weng-claim-ladder.json` and
other structured-input links. Update the builder self-test fixtures so
`card_path` is `knowledge/rsi/weng-sources/test.md` and `canonical_route` is
`knowledge/rsi/test.md`.

- [ ] **Step 3: Regenerate and check the Weng teaching projections**

Run:

```sh
python3 scripts/build_weng_course.py --self-test
python3 scripts/build_weng_course.py
python3 scripts/build_weng_course.py --check
```

Expected: all four `reference/*.html` files are deterministic and reference
canonical Markdown below `knowledge/rsi/`, while their JSON/TSV inputs still
link below `content/`.

- [ ] **Step 4: Regenerate corpus JSON and offline Atlas export**

Run:

```sh
cargo run -p harp -- check
cargo run -p harp -- build
cargo run -p harp -- build --check
cd atlas && corepack pnpm run test
cd atlas && corepack pnpm run test:export
```

Expected:

- `atlas/src/content/generated/corpus.json` contains
  `knowledge/rsi/...` canonical paths;
- `atlas/dist/harp-atlas.html` and
  `atlas/dist/harp-atlas.receipt.json` are regenerated together; and
- TypeScript still imports its generated payload from
  `atlas/src/content/generated/corpus.json`.

- [ ] **Step 5: Verify search and projection boundaries**

Run:

```sh
cargo run -p harp -- search refresh
cargo run -p harp -- search status
cargo run -p harp -- --format json search query "recursive improvement" --limit 5
cargo run -p harp -- --format json search query "Private draft" --limit 5
```

Expected:

- the first query returns a path below `knowledge/rsi/` or a registered
  packet root;
- the second returns an empty results array and never exposes
  `knowledge/Untitled.md` or any untracked local note; and
- the search receipt is current.

### Task 4: Publish the contributor-facing authority contract and land the migration

**Files:**
- Modify: `AGENTS.md`
- Modify: `README.md`
- Modify: `docs/product-contract.md`
- Modify: `atlas/README.md`
- Modify: `RESOURCES.md`
- Modify: `docs/import-receipt.md`

- [ ] **Step 1: Update repository ownership instructions**

Replace the `AGENTS.md` canonical-ownership bullet with:

```markdown
- Managed Markdown under `knowledge/rsi/` and registered topic packets under
  `knowledge/` are the only authorities for technical prose. `content/` is
  reserved for structured machine-readable contracts and diagnostics; do not
  add Markdown beneath it.
```

In `README.md`, replace the old `content/`/Meta-Harness description with a
short authority statement naming:

```text
knowledge/rsi/
knowledge/darwin_godel_machine/
knowledge/meta_harness/
knowledge/harness_benchmarks/
```

State that generated JSON and HTML are derived artifacts and that `content/`
contains structured contracts and diagnostics.

- [ ] **Step 2: Update the product and Atlas contracts**

In `docs/product-contract.md`:

- make the product boundary say canonical RSI and SICP Markdown is under
  `knowledge/rsi/`;
- describe the named packet roots as canonical technical prose;
- preserve `content/diagnostics/` as the diagnostics contract home;
- state that `content/weng-sources` no longer exists and that
  `knowledge/rsi/weng-sources/` owns the 42 source-card Markdown documents;
- retain all JSON/TSV authority statements under `content/`;
- replace the search root list with the four explicit managed knowledge roots
  plus the two evidence-text roots; and
- retain corpus-count and evidence/licensing claims unchanged.

In `atlas/README.md`, replace:

```markdown
[Return to the canonical RSI orientation packet](../content/rsi_index.md).
```

with:

```markdown
[Return to the canonical RSI orientation packet](../knowledge/rsi/rsi_index.md).
```

Describe canonical chapters, concepts, systems, and lessons as living under
`knowledge/rsi/`, with `coverage-map.tsv` remaining a structured `content/`
input.

In `RESOURCES.md`, update the deconstruction link to:

```markdown
- [`knowledge/rsi/rsi_harness_by_lil_log_deconstructed.md`](knowledge/rsi/rsi_harness_by_lil_log_deconstructed.md)
```

- [ ] **Step 3: Inspect the staged surface and run the complete release gate in isolation**

Stage only migration files:

```sh
git add \
  AGENTS.md README.md RESOURCES.md \
  docs/product-contract.md docs/import-map.tsv \
  crates/harp/src crates/harp/tests \
  scripts/build_weng_course.py \
  atlas/src atlas/dist \
  content \
  knowledge/rsi \
  knowledge/darwin_godel_machine \
  knowledge/meta_harness \
  knowledge/harness_benchmarks \
  reference lessons
git add -u -- content knowledge docs atlas crates scripts
git diff --cached --check
git diff --cached --name-only
```

Confirm that no unrelated untracked file appears in the staged set. Then
isolate it:

```sh
git stash push --keep-index -u -m 'trae: isolate knowledge prose authority migration'
mise run verify
```

Expected: all Rust, Python, Atlas, LFS, repository, and generated-artifact
checks pass except possibly the final import receipt digest.

- [ ] **Step 4: Refresh the import receipt only if the verifier reports the required digest**

If `mise run verify` fails with:

```text
import receipt payload digest is stale; expected `<sha256>`
```

replace only the digest in `docs/import-receipt.md` with the exact value in
that error. Do not invent a digest and do not alter source-repository history.

Restage it and rerun:

```sh
git add docs/import-receipt.md
git diff --cached --check
mise run verify
```

Expected: full success.

- [ ] **Step 5: Commit focused changes and restore unrelated work**

Commit the structural migration first if the isolated staged surface naturally
separates from the contributor docs; otherwise make one atomic migration
commit because the corpus, search, Weng projection, generated artifacts, and
authority docs must agree.

Use one of these exact messages, keeping the required trailer exactly once:

```sh
git commit -m "refactor: move canonical prose to knowledge" \
  -m "Make knowledge/rsi the managed RSI Markdown authority, retain content for structured contracts and diagnostics, and update corpus, search, Weng projections, and Atlas artifacts." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

If contributor docs and the receipt are a separate verified second commit:

```sh
git commit -m "docs: publish knowledge prose authority" \
  -m "Document the managed knowledge roots and refresh the import receipt after the canonical prose migration." \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

Restore the isolated work:

```sh
stash_ref="$(git stash list | rg -m1 'trae: isolate knowledge prose authority migration' | cut -d: -f1)"
git stash pop "$stash_ref"
git status --short
git log --format='%h%n%B' -2
```

Expected: unrelated user files return as untracked, migration commits contain
the exact co-author trailer once each, and no unrelated file is committed.

## Final verification checklist

- [ ] `find content -type f -name '*.md'` prints no files.
- [ ] `find knowledge/rsi -type f -name '*.md' | wc -l` prints `136`.
- [ ] `cargo run -p harp -- check` preserves the published corpus counts.
- [ ] `cargo run -p harp -- build --check` succeeds.
- [ ] `python3 scripts/build_weng_course.py --check` succeeds.
- [ ] `cargo run -p harp -- search status` succeeds after refresh.
- [ ] Search excludes an unregistered `knowledge/` scratch Markdown fixture.
- [ ] `cd atlas && corepack pnpm run test:export` succeeds.
- [ ] `mise run verify` succeeds on the staged migration surface.
- [ ] `git diff --cached --check` is clean.
- [ ] No evidence byte, license record, or captured artifact changed.
- [ ] No AHE case-study content was added; it remains the next task after this migration lands.

## Plan self-review

- **Spec coverage:** Tasks 1–4 cover the `knowledge/rsi/` move, no-Markdown
  `content/` invariant, explicit packet/search roots, structured contract
  retention, cross-root link repair, Weng projections, Atlas derivation,
  contributor docs, receipt refresh, and isolated validation.
- **No placeholders:** Every task names exact files, required values, commands,
  expected outcomes, and commit text.
- **Type consistency:** Rust constants, error codes, helper names, and target
  paths are used consistently as `RSI_MARKDOWN_ROOT`, `CONTENT_ROOT`,
  `regular_files_with_extension`, and `knowledge/rsi/...`.
- **Scope discipline:** The plan leaves evidence untouched, preserves generated
  output locations and schemas, does not index arbitrary `knowledge/` files,
  does not absorb untracked local material, and defers the AHE packet.

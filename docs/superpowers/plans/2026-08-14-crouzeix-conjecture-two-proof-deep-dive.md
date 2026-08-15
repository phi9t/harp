# Crouzeix Conjecture Two-Proof Deep Dive Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a source-backed, rights-aware Harp packet that derives and compares the Jin and Lorist-Schwenninger proof mechanisms, audits Jin's Lean formalization, and exposes the packet through searchable, math-capable, section-addressable Atlas routes.

**Architecture:** The work lands in three independently verifiable commits. The first adds remote-only source receipts, normalized verification logs, and an offline verifier. The second adds reusable registered-document routing, explicit heading fragments, and packet-scoped KaTeX-to-MathML rendering. The third adds and registers the thirteen-document packet, its claim ledger, source registry, search root, generated corpus, Atlas export, and exact-count updates.

**Tech Stack:** Rust 1.92, `pulldown-cmark` 0.13.4, TypeScript 5.9, React 19, Vite 8, Vitest 4, KaTeX 0.18.4, shell acquisition scripts, Markdown with YAML frontmatter and TeX math, SQLite FTS5, Git LFS, and `mise`.

**Approved design:** `docs/superpowers/specs/2026-08-14-crouzeix-conjecture-deep-dive-design.md`

---

## File map

### Evidence boundary

- Create `crates/harp/src/sources/crouzeix.rs`: strict parsers and offline verification for the Crouzeix source and verification manifests.
- Modify `crates/harp/src/sources.rs`: register the verifier, include its receipts in source accounting, and expose test-only helpers to the child module.
- Create `evidence/crouzeix_conjecture/PROVENANCE.md`: source classes, acquisition method, normalization rules, rights boundary, and claim ceiling.
- Create `evidence/crouzeix_conjecture/acquire.sh`: explicit networked acquisition, detached Git verification, Lean build/scan runs, log normalization, and deterministic manifest generation.
- Create `evidence/crouzeix_conjecture/source_manifest.tsv`: remote-only artifact receipts.
- Create `evidence/crouzeix_conjecture/verification_manifest.tsv`: revision-bound Harp-local build and scan receipts.
- Create `evidence/crouzeix_conjecture/verification/*.log`: normalized metadata-only logs for two Jin revisions.
- Modify `crates/harp/tests/cli.rs`: update exact source-report accounting.
- Modify `docs/product-contract.md`: declare the remote-only Crouzeix evidence bundle and its verification ceiling.
- Modify `docs/import-receipt.md`: refresh the tracked payload digest after the evidence commit settles.

### Reader substrate

- Modify `crates/harp/src/corpus/mod.rs`: build route targets before rendering and pass them into the renderer.
- Modify `crates/harp/src/corpus/contracts.rs`: parse explicit heading IDs for the packet and validate packet-local anchor uniqueness.
- Modify `crates/harp/src/corpus/render.rs`: route-target precedence, fragment-preserving links, packet-scoped math parsing, explicit heading attributes, and escaped math spans with `data-tex`.
- Modify `crates/harp/src/corpus/tests.rs`: red-green tests for reader-route precedence, auxiliary fragments, explicit heading IDs, and scoped math parsing.
- Modify `atlas/package.json` and `atlas/pnpm-lock.yaml`: pin `katex` 0.18.4 and `@types/katex` 0.16.8.
- Create `atlas/src/app/math.ts`: deterministic KaTeX rendering with MathML-only output and fail-visible fallback.
- Create `atlas/src/app/math.test.ts`: valid inline/display math, escaping, invalid math, and no external resource tests.
- Modify `atlas/src/app/routes.ts`: optional document section query.
- Modify `atlas/src/app/routes.test.ts`: round-trip and invalid-section route tests.
- Modify `atlas/src/app/ChapterReader.tsx`: render math, focus and scroll explicit section IDs, and scope selection to the current article.
- Create `atlas/src/app/ChapterReader.test.tsx`: MathML and section-focus behavior.
- Modify `atlas/src/styles/components.css`: stable inline/display math and focused-heading styles.
- Modify `atlas/tests/static-export.test.mjs`: assert bundled KaTeX, route grammar, formulas, and absence of external assets.
- Regenerate `atlas/dist/harp-atlas.html` and `atlas/dist/harp-atlas.receipt.json`.
- Modify `docs/import-receipt.md`: refresh the tracked payload digest after the reader commit settles.

### Canonical packet

- Create `knowledge/crouzeix_conjecture/crouzeix_conjecture_index.md`: packet status, source identities, expert/guided routes, and five-minute two-proof model.
- Create `knowledge/crouzeix_conjecture/01_problem_and_prior_barrier.md`: conjecture, spectral sets, sharpness, and the prior symmetrized barrier.
- Create `knowledge/crouzeix_conjecture/02_shared_power_family.md`: bounded Harp inference connecting the Cayley family to the explicit power sequence.
- Create `knowledge/crouzeix_conjecture/03_jin_proof_spine.md`: Jin's end-to-end argument and limit order.
- Create `knowledge/crouzeix_conjecture/04_jin_positive_real_completion.md`: complete hypotheses, cancellation, ordered Gramians, and norm endpoint.
- Create `knowledge/crouzeix_conjecture/05_lorist_schwenninger_proof.md`: perturbation recurrence, compressed product bound, double-layer realization, and rational consequence.
- Create `knowledge/crouzeix_conjecture/06_proof_comparison.md`: fixed comparison contract and completely bounded limitation.
- Create `knowledge/crouzeix_conjecture/07_jin_lean_verification.md`: revision identities, theorem DAG, build/scan receipts, and trust boundary.
- Create `knowledge/crouzeix_conjecture/08_ai_assisted_discovery.md`: two AI disclosures, Jin task contract, action matrix, and missing transcripts.
- Create `knowledge/crouzeix_conjecture/09_status_and_critical_assessment.md`: publication states, source contradictions, confidence ceiling, and missing evidence.
- Create `knowledge/crouzeix_conjecture/glossary.md`: symbols and proof-specific terms.
- Create `knowledge/crouzeix_conjecture/source_registry.md`: source IDs, immutable identities, local receipt routes, quotation audit, and claim ceilings.
- Create `knowledge/crouzeix_conjecture/claim_evidence_ledger.md`: canonical `CC-*` claim entries.
- Create `crates/harp/tests/crouzeix_conjecture_knowledge_packet.rs`: packet shape, ledger, locators, mechanisms, status, links, and registration tests.
- Modify `crates/harp/src/corpus/mod.rs`: add one reader route and twelve auxiliary documents.
- Modify `crates/harp/src/search.rs`: add the packet root.
- Modify `crates/harp/src/corpus/tests.rs`: update fixture documents and exact packet registration checks.
- Modify `crates/harp/tests/cli.rs`: update canonical document and search fixtures.
- Modify `atlas/src/content/types.ts`: add `crouzeix-conjecture` to the closed route union.
- Modify `atlas/src/content/canonical.test.ts`: add the twelfth route fixture.
- Modify `atlas/src/app/AtlasApp.tsx`: visible Crouzeix navigation and configured route labels.
- Modify `atlas/src/app/routes.test.ts`: first-class Crouzeix route coverage.
- Modify `docs/product-contract.md`: packet ownership, evidence, search, and exact counts.
- Regenerate `atlas/src/content/generated/corpus.json`, `atlas/dist/harp-atlas.html`, and `atlas/dist/harp-atlas.receipt.json`.
- Modify `docs/import-receipt.md`: final packet payload digest.

## Fixed claim roster

The packet test requires these stable claim IDs. Do not renumber them after publication.

| Claim ID | Class | Owning subject |
|---|---|---|
| `CC-001` | `SOURCE CLAIM` | Crouzeix's constant-two conjecture |
| `CC-002` | `EVIDENCE` | The \(2\times2\) nilpotent sharpness witness |
| `CC-003` | `SOURCE CLAIM` | Crouzeix-Palencia \(1+\sqrt2\) result |
| `CC-004` | `EVIDENCE` | Symmetrized double-layer identity |
| `CC-005` | `INFERENCE` | One-step treatment loses the coupling used by the new routes |
| `CC-010` | `EVIDENCE` | Full Jin positive-real completion hypotheses |
| `CC-011` | `EVIDENCE` | Auxiliary basis permits repeated target values |
| `CC-012` | `EVIDENCE` | Herglotz kernel is positive |
| `CC-013` | `EVIDENCE` | Origin sample cancels the diagonal correction |
| `CC-014` | `EVIDENCE` | Ordered pre-Gramian inequality |
| `CC-015` | `EVIDENCE` | Positive difference plus anticommutator inequality implies \(\widehat P\preceq2I\) |
| `CC-016` | `EVIDENCE` | First nonconstant Gramian term implies \(\|T\|\le2\) |
| `CC-017` | `EVIDENCE` | Fixed-domain simple-spectrum limit precedes outer-domain limit |
| `CC-020` | `EVIDENCE` | Lean exports the polynomial theorem |
| `CC-021` | `EVIDENCE` | Lean proposition definition and proving theorem are distinct declarations |
| `CC-022` | `EVIDENCE` | Audited manuscript digest is stale at repository HEAD |
| `CC-023` | `EVIDENCE` | Build/axiom audit result at `565b6a3` |
| `CC-024` | `EVIDENCE` | Build/axiom audit result at `9df0783` |
| `CC-025` | `EVIDENCE` | Commit-scoped prohibited-token scans |
| `CC-030` | `SOURCE CLAIM` | Lorist-Schwenninger Lemma 1 |
| `CC-031` | `EVIDENCE` | Lorist-Schwenninger recurrence |
| `CC-032` | `INFERENCE` | Equation 1 gives the uniform \(E_nT^n\) bound |
| `CC-033` | `EVIDENCE` | Double-layer realization \(E_n=\alpha(f^n)(A)\) |
| `CC-034` | `INFERENCE` | Functional calculus independently bounds \(E_nT^n\) in the application |
| `CC-035` | `SOURCE CLAIM` | Lorist-Schwenninger rational \(2\)-spectral-set conclusion |
| `CC-036` | `SOURCE CLAIM` | Lorist-Schwenninger abstract uniform-algebra variant |
| `CC-037` | `SOURCE CLAIM` | Neither route directly yields the completely bounded case |
| `CC-040` | `INFERENCE` | Both proofs retain a complete power family |
| `CC-041` | `EVIDENCE` | Lorist-Schwenninger describe Jin's proof as independent and different |
| `CC-042` | `EVIDENCE` | Jin attributes the sampling/cancellation idea to ChatGPT |
| `CC-043` | `EVIDENCE` | Lorist-Schwenninger disclose ChatGPT 5.6 Pro strategy exploration |
| `CC-044` | `MISSING` | Underlying agent transcripts and approach registries are absent |
| `CC-045` | `MISSING` | Jin's Preprints.org manuscript bytes are not mapped to a Git revision |
| `CC-046` | `MISSING` | Actual Annals submission status is not established |

## Fixed source-receipt roster

`source_manifest.tsv` contains exactly these 29 receipt IDs. Repository
archives bind the complete Git trees; file-level rows pin the documents whose
byte identity or status is discussed directly.

```text
JIN-565-ARCHIVE
JIN-565-V4-TEX
JIN-565-FORMALIZATION-MAP
JIN-565-MANUSCRIPT-AUDIT
JIN-565-AXIOM-AUDIT
JIN-565-VERIFY
JIN-565-LEAN-TOOLCHAIN
JIN-565-LAKE-MANIFEST
JIN-HEAD-ARCHIVE
JIN-HEAD-README
JIN-HEAD-PROMPT
JIN-HEAD-V4-TEX
JIN-HEAD-ANNMATH-TEX
JIN-HEAD-FORMALIZATION-MAP
JIN-HEAD-MANUSCRIPT-AUDIT
JIN-HEAD-AXIOM-AUDIT
JIN-HEAD-VERIFY
JIN-HEAD-LEAN-TOOLCHAIN
JIN-HEAD-LAKE-MANIFEST
JIN-PREPRINTS-V1-METADATA
LS-ARXIV-V1-ATOM
LS-ARXIV-V1-SOURCE-ARCHIVE
LS-ARXIV-V1-TEX
LS-ARXIV-V1-PDF
CROUZEIX-2007
CROUZEIX-PALENCIA-2017
DELYON-DELYON-1999
RANSFORD-SCHWENNINGER-2018
SCHWENNINGER-DEVRIES-2025
```

---

### Task 1: Add the strict Crouzeix evidence verifier

**Files:**

- Create: `crates/harp/src/sources/crouzeix.rs`
- Modify: `crates/harp/src/sources.rs:13-20`
- Modify: `crates/harp/src/sources.rs:150-252`
- Test: `crates/harp/src/sources/crouzeix.rs`

- [ ] **Step 1: Add failing verifier tests**

Create the child module with tests first. The tests use a temporary repository and write the exact two TSV headers from the approved design.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn fixture() -> tempfile::TempDir {
        let repo = tempdir().unwrap();
        let root = repo.path().join(ROOT);
        fs::create_dir_all(root.join("verification")).unwrap();
        fs::write(root.join("PROVENANCE.md"), "# Provenance\n").unwrap();
        fs::write(root.join("acquire.sh"), "#!/bin/sh\nset -eu\n").unwrap();
        fs::write(
            root.join("source_manifest.tsv"),
            format!(
                "{SOURCE_HEADER}\n\
                 crouzeix-source-receipt/v1\tJIN-HEAD-README\tJIN-REPO-HEAD\tgit-artifact\tmetadata\t\
                 git:9df07838327b988e3924453daa29c8cd726d34b0\t\
                 https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/README.md\t\
                 README.md\t357\t{}\t-\t2026-08-14\tnot-present-at-revision\tmetadata-only\n",
                "0".repeat(64),
            ),
        )
        .unwrap();
        let script_digest = sha256_file(&root.join("acquire.sh")).unwrap();
        let empty_digest = format!("{:x}", Sha256::digest([]));
        let mut verification = format!("{VERIFICATION_HEADER}\n");
        for (receipt, commit, operation, log) in [
            (
                "JIN-565-BUILD",
                "565b6a3e0659b6e0785f783b016c3f6d9f171fa5",
                "lean-build",
                "verification/jin-565b6a3-build.log",
            ),
            (
                "JIN-565-SCAN",
                "565b6a3e0659b6e0785f783b016c3f6d9f171fa5",
                "source-scan",
                "verification/jin-565b6a3-scan.log",
            ),
            (
                "JIN-HEAD-BUILD",
                "9df07838327b988e3924453daa29c8cd726d34b0",
                "lean-build",
                "verification/jin-9df0783-build.log",
            ),
            (
                "JIN-HEAD-SCAN",
                "9df07838327b988e3924453daa29c8cd726d34b0",
                "source-scan",
                "verification/jin-9df0783-scan.log",
            ),
        ] {
            fs::write(repo.path().join(ROOT).join(log), "").unwrap();
            verification.push_str(&format!(
                "crouzeix-verification-receipt/v1\t{receipt}\tJIN-REPO-HEAD\t{commit}\t{}\t{operation}\t{}\t{}\t\
                 crouzeix-log-normalization/v1\tleanprover/lean4:v4.28.0\t\
                 8f9d9cff6bd728b17a24e163c9402775d9e6a365\t2026-08-14T00:00:00Z\t0\tpassed\t{log}\t0\t{empty_digest}\n",
                "1".repeat(40),
                "2".repeat(64),
                script_digest,
            ));
        }
        fs::write(root.join("verification_manifest.tsv"), verification).unwrap();
        repo
    }

    fn replace_source_url(repo_root: &Path, replacement: &str) {
        let path = repo_root.join(ROOT).join("source_manifest.tsv");
        let text = fs::read_to_string(&path).unwrap();
        let mut rows = text.lines();
        let header = rows.next().unwrap();
        let mut fields = rows.next().unwrap().split('\t').collect::<Vec<_>>();
        fields[6] = replacement;
        fs::write(path, format!("{header}\n{}\n", fields.join("\t"))).unwrap();
    }

    #[test]
    fn rejects_unknown_files_and_symlinks_in_the_evidence_root() {
        let repo = fixture();
        fs::write(
            repo.path().join("evidence/crouzeix_conjecture/copied-source.lean"),
            "theorem copied : True := by trivial\n",
        )
        .unwrap();

        let error = verify(repo.path()).unwrap_err();
        assert_eq!(error.code(), "sources.crouzeix.roster");
    }

    #[test]
    fn source_manifest_requires_identity_bound_urls_and_unique_receipts() {
        let repo = fixture();
        replace_source_url(
            repo.path(),
            "https://github.com/jinshanmu/CrouzeixConjecture/blob/main/README.md",
        );

        let error = verify(repo.path()).unwrap_err();
        assert_eq!(error.code(), "sources.crouzeix.source_manifest");
    }

    #[test]
    fn verification_receipts_bind_commands_logs_and_acquisition_script() {
        let repo = fixture();
        fs::write(
            repo.path().join("evidence/crouzeix_conjecture/acquire.sh"),
            "#!/bin/sh\nexit 0\n# changed\n",
        )
        .unwrap();

        let error = verify(repo.path()).unwrap_err();
        assert_eq!(error.code(), "sources.crouzeix.verification_manifest");
    }
}
```

The fixture must create only:

```text
PROVENANCE.md
acquire.sh
source_manifest.tsv
verification_manifest.tsv
verification/jin-565b6a3-build.log
verification/jin-565b6a3-scan.log
verification/jin-9df0783-build.log
verification/jin-9df0783-scan.log
```

- [ ] **Step 2: Run the focused test and verify it fails**

Run:

```sh
cargo test -p harp sources::crouzeix::tests --lib -- --test-threads=1
```

Expected: compilation fails because `sources::crouzeix` is not registered or `verify` is not implemented.

- [ ] **Step 3: Implement strict parsing and verification**

Add to `crates/harp/src/sources.rs`:

```rust
mod crouzeix;
```

Implement in `crates/harp/src/sources/crouzeix.rs`:

```rust
pub(super) const ROOT: &str = "evidence/crouzeix_conjecture";

#[derive(Debug)]
pub(super) struct Report {
    pub(super) source_receipts: usize,
    pub(super) verification_receipts: usize,
}

pub(super) fn verify(repo_root: &Path) -> Result<Report, AppError> {
    verify_exact_roster(repo_root)?;
    let source_receipts = verify_source_manifest(repo_root)?;
    let verification_receipts = verify_verification_manifest(repo_root)?;
    Ok(Report {
        source_receipts,
        verification_receipts,
    })
}
```

Use exact header constants:

```rust
const SOURCE_HEADER: &str = "schema_version\treceipt_id\tsource_id\tsource_class\trole\timmutable_identity\tsource_url\tupstream_path\tbytes\tsha256\tlocal_path\tobserved\tlicense_status\tredistribution_status";
const VERIFICATION_HEADER: &str = "schema_version\treceipt_id\tsource_id\tsource_commit\tsource_tree\toperation\tcommand_sha256\tacquisition_script_sha256\tnormalization_version\ttoolchain\tmathlib_revision\tobserved_at_utc\texit_code\tresult\tlog_path\tlog_bytes\tlog_sha256";
```

Implement closed enums with `matches!`, reject whitespace-normalization differences, bind Git URLs to the declared 40-hex commit, bind arXiv URLs to `vN`, require `local_path == "-"`, reject duplicate `receipt_id` and duplicate `(source_id, immutable_identity, upstream_path)`, and verify local log byte counts and SHA-256.

`verify_exact_roster` must walk `evidence/crouzeix_conjecture` with `WalkDir::follow_links(false)`, reject every symlink, and compare the sorted relative paths with the exact eight-file roster.

- [ ] **Step 4: Wire report accounting**

In `sources::verify`, add:

```rust
let crouzeix = crouzeix::verify(repo_root)?;
```

Include both receipt counts in `evidence_artifacts`:

```rust
evidence_artifacts: weng
    + rlm
    + self_improving_agents_survey
    + sicp
    + benchmarks
    + meta_harness.site_artifacts
    + meta_harness.normalized_artifacts
    + crouzeix.source_receipts
    + crouzeix.verification_receipts,
```

- [ ] **Step 5: Run focused tests**

Run:

```sh
cargo test -p harp sources::crouzeix::tests --lib -- --test-threads=1
```

Expected: all Crouzeix verifier tests pass.

---

### Task 2: Add acquisition, manifests, normalized verification receipts, and evidence commit

**Files:**

- Create: `evidence/crouzeix_conjecture/PROVENANCE.md`
- Create: `evidence/crouzeix_conjecture/acquire.sh`
- Create: `evidence/crouzeix_conjecture/source_manifest.tsv`
- Create: `evidence/crouzeix_conjecture/verification_manifest.tsv`
- Create: `evidence/crouzeix_conjecture/verification/jin-565b6a3-build.log`
- Create: `evidence/crouzeix_conjecture/verification/jin-565b6a3-scan.log`
- Create: `evidence/crouzeix_conjecture/verification/jin-9df0783-build.log`
- Create: `evidence/crouzeix_conjecture/verification/jin-9df0783-scan.log`
- Modify: `crates/harp/tests/cli.rs:330-339`
- Modify: `docs/product-contract.md:166-182`
- Modify: `docs/import-receipt.md:125-130`

- [ ] **Step 1: Write the deterministic acquisition script**

`acquire.sh` must:

1. require an empty destination temp root;
2. set `GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`, and `GIT_NO_REPLACE_OBJECTS=1`;
3. clone `https://github.com/jinshanmu/CrouzeixConjecture.git`;
4. reject alternates, replacement refs, config includes, dirty state, and symlinks;
5. verify commits `565b6a3e0659b6e0785f783b016c3f6d9f171fa5` and `9df07838327b988e3924453daa29c8cd726d34b0`;
6. fetch exact arXiv `2608.03841v1` source, PDF, and Atom metadata to temporary files;
7. request the versioned Preprints.org metadata page once with a browser-like
   user agent; on HTTP 200 record bytes and SHA-256, and on any other status
   record `bytes=-`, `sha256=-`, `redistribution_status=metadata-only`, the
   status code, and observation date in `PROVENANCE.md`;
8. run each Jin revision's `Lean/verify.sh`;
9. run a metadata-only `.lean` scan that emits `path<TAB>line<TAB>token_class<TAB>line_sha256`, never source text;
10. normalize checkout paths, detect source excerpts, scan secrets, cap logs at 2 MiB, and write deterministic manifests sorted by `receipt_id`.

Use a shell array of forbidden scan classes:

```sh
scan_patterns='
sorry
admit
custom-axiom
unsafe
native-decide
implemented-by
option-override
'
```

The inline Python normalizer must hash NUL-delimited argv and write the exact manifest headers from Task 1.

- [ ] **Step 2: Run acquisition**

Run:

```sh
evidence/crouzeix_conjecture/acquire.sh
```

Expected:

```text
Crouzeix evidence captured: 29 source receipts, 4 verification receipts
```

If Preprints.org blocks byte acquisition, the corresponding row must use:

```text
bytes=-
sha256=-
redistribution_status=metadata-only
```

and `PROVENANCE.md` must record the HTTP status and observation date without claiming manuscript correspondence.

- [ ] **Step 3: Verify source and verification manifests**

Run:

```sh
cargo test -p harp sources::crouzeix::tests --lib -- --test-threads=1
cargo run -p harp -- sources verify
```

Expected source report after 29 source rows and 4 verification rows:

```text
sources verified: 337 evidence artifacts, 105 snapshot files
```

- [ ] **Step 4: Update CLI and product contracts**

Extend `SourcesReport` with:

```rust
pub crouzeix_source_receipts: usize,
pub crouzeix_verification_receipts: usize,
```

Set them from the child verifier report. Change the CLI JSON expectation:

```rust
let envelope: Value = serde_json::from_slice(&output).expect("JSON envelope");
assert_eq!(envelope["data"]["evidence_artifacts"], 337);
assert_eq!(envelope["data"]["crouzeix_source_receipts"], 29);
assert_eq!(envelope["data"]["crouzeix_verification_receipts"], 4);
```

Add to `docs/product-contract.md`:

```markdown
- remote-only, hash-pinned Crouzeix proof artifacts and four Harp-local
  revision-bound Lean verification receipts under
  `evidence/crouzeix_conjecture/`;
```

State that source verification confirms receipt structure and local logs, not remote availability, mathematical correctness, or peer review.

- [ ] **Step 5: Refresh the import receipt**

Run:

```sh
git add crates/harp/src/sources.rs crates/harp/src/sources/crouzeix.rs \
  crates/harp/tests/cli.rs evidence/crouzeix_conjecture docs/product-contract.md
cargo run -q -p harp -- repository verify
```

Expected: failure naming the new payload digest.

Patch only the digest in `docs/import-receipt.md`, then stage it.

- [ ] **Step 6: Verify the evidence commit**

Run:

```sh
cargo test -p harp sources::crouzeix::tests --lib -- --test-threads=1
cargo test -p harp --test cli
cargo run -p harp -- sources verify
cargo run -p harp -- repository verify
mise run verify
git diff --cached --check
```

Expected: all commands pass.

- [ ] **Step 7: Commit**

```sh
git add crates/harp/src/sources.rs crates/harp/src/sources/crouzeix.rs \
  crates/harp/tests/cli.rs evidence/crouzeix_conjecture \
  docs/product-contract.md docs/import-receipt.md
git commit -m "evidence: add Crouzeix proof receipts" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 3: Add Rust route-target, heading, fragment, and scoped-math support

**Files:**

- Modify: `crates/harp/src/corpus/mod.rs:302-359`
- Modify: `crates/harp/src/corpus/contracts.rs:121-175`
- Modify: `crates/harp/src/corpus/contracts.rs:1369-1430`
- Modify: `crates/harp/src/corpus/render.rs:1-224`
- Test: `crates/harp/src/corpus/tests.rs`

- [ ] **Step 1: Write failing corpus tests**

Add tests:

```rust
#[test]
fn reader_route_targets_precede_document_routes_and_fragments_survive() {
    let targets = render::route_targets(&[], &BTreeMap::new());
    assert_eq!(
        targets.get("knowledge/rsi/source_registry.md"),
        Some(&render::RouteTarget::Reader("sources"))
    );
    let mut fragment_targets = BTreeMap::new();
    fragment_targets.insert(
        "knowledge/example/ledger.md".to_owned(),
        render::RouteTarget::Document {
            document_id: "example-ledger".to_owned(),
            heading_ids: BTreeSet::from([
                "cc-013-origin-sample-cancels-the-correction".to_owned(),
            ]),
        },
    );
    assert_eq!(
        render::offline_link_destination_with_targets(
            "ledger.md#cc-013-origin-sample-cancels-the-correction",
            Path::new("knowledge/example"),
            &fragment_targets,
        ),
        "#documents/example-ledger?section=cc-013-origin-sample-cancels-the-correction"
    );
}

#[test]
fn math_and_heading_attributes_are_scoped_to_the_crouzeix_packet() {
    let packet = render::render_markdown(
        "## Bound {#bound}\n\n$\\|T\\|\\le2$",
        "knowledge/crouzeix_conjecture/example.md",
        &[],
    );
    assert!(packet.contains("id=\"bound\""));
    assert!(packet.contains("class=\"math math-inline\""));
    assert!(packet.contains("data-tex=\""));

    let legacy = render::render_markdown(
        "Revenue moved from $0.0291 to $0.6371.",
        "knowledge/rsi/systems/aflow.md",
        &[],
    );
    assert!(!legacy.contains("math-inline"));
}
```

- [ ] **Step 2: Run tests and verify failure**

Run:

```sh
cargo test -p harp corpus::tests --lib -- --test-threads=1
```

Expected: failure because `RouteTarget`, `route_targets`, and scoped options do not exist.

- [ ] **Step 3: Implement pre-render route targets**

Add:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum RouteTarget {
    Reader(&'static str),
    Chapter(String),
    Document {
        document_id: String,
        heading_ids: BTreeSet<String>,
    },
}
```

Build a `BTreeMap<String, RouteTarget>` before document compilation:

1. insert coverage chapters;
2. insert auxiliary documents;
3. insert published systems and lesson documents;
4. insert reader routes last so reader ownership wins.

Change:

```rust
render::compile_document(source, &coverage, &all_sources)
```

to:

```rust
render::compile_document(source, &coverage, &all_sources, &route_targets)
```

- [ ] **Step 4: Implement explicit headings and packet-scoped math**

Create:

```rust
fn markdown_options(source_path: &str) -> Options {
    let mut options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_DEFINITION_LIST;
    if source_path.starts_with("knowledge/crouzeix_conjecture/") {
        options |= Options::ENABLE_HEADING_ATTRIBUTES | Options::ENABLE_MATH;
    }
    options
}
```

Map `InlineMath` and `DisplayMath` to escaped HTML:

```rust
Event::InlineMath(tex) => Event::Html(math_span(&tex, false).into()),
Event::DisplayMath(tex) => Event::Html(math_span(&tex, true).into()),
```

`math_span` must emit:

```html
<span class="math math-inline" data-tex="...">...</span>
```

or:

```html
<span class="math math-display" data-tex="...">...</span>
```

Escape `&`, `<`, `>`, `"`, and `'` in attributes and `&`, `<`, `>` in body text.

Update `heading_ids` to use parser-provided IDs when present and reject duplicate explicit IDs in packet files.

- [ ] **Step 5: Implement fragment-preserving destinations**

Route rewriting rules:

```rust
RouteTarget::Reader(route_id) => format!("#{route_id}"),
RouteTarget::Chapter(concept_id) => format!("#chapters/{concept_id}"),
RouteTarget::Document(document_id) if fragment.is_some() =>
    format!("#documents/{document_id}?section={}", fragment.unwrap()),
RouteTarget::Document(document_id) => format!("#documents/{document_id}"),
```

Validate the fragment against the target document's explicit heading IDs before adding `?section=`.

- [ ] **Step 6: Run corpus tests**

Run:

```sh
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo run -p harp -- build --check
```

Expected: all corpus tests pass and current generated corpus remains unchanged before packet registration.

---

### Task 4: Add Atlas section routing and KaTeX MathML

**Files:**

- Modify: `atlas/package.json`
- Modify: `atlas/pnpm-lock.yaml`
- Create: `atlas/src/app/math.ts`
- Create: `atlas/src/app/math.test.ts`
- Modify: `atlas/src/app/routes.ts`
- Modify: `atlas/src/app/routes.test.ts`
- Modify: `atlas/src/app/ChapterReader.tsx`
- Create: `atlas/src/app/ChapterReader.test.tsx`
- Modify: `atlas/src/app/AtlasApp.tsx`
- Modify: `atlas/src/styles/components.css`
- Modify: `atlas/tests/static-export.test.mjs`
- Regenerate: `atlas/dist/harp-atlas.html`
- Regenerate: `atlas/dist/harp-atlas.receipt.json`
- Modify: `docs/import-receipt.md`

- [ ] **Step 1: Add dependencies**

Run:

```sh
cd atlas
corepack pnpm add katex@0.18.4
corepack pnpm add -D @types/katex@0.16.8
```

Expected: `atlas/package.json` and `atlas/pnpm-lock.yaml` change; no font or CSS dependency is added.

- [ ] **Step 2: Write failing route and math tests**

Add route expectations:

```ts
expect(parseRoute("#documents/context-engineering-deep-dive?section=context-pipeline"))
  .toEqual({
    kind: "document",
    documentId: contextDocumentId,
    sectionId: "context-pipeline",
  });

expect(formatRoute({
  kind: "document",
  documentId: contextDocumentId,
  sectionId: "context-pipeline",
})).toBe("#documents/context-engineering-deep-dive?section=context-pipeline");
```

Add math test:

```ts
it("renders escaped TeX as MathML without external resources", () => {
  const root = document.createElement("div");
  root.innerHTML =
    '<span class="math math-inline" data-tex="\\\\|T\\\\|\\\\le2">\\\\|T\\\\|\\\\le2</span>';
  renderCanonicalMath(root);
  expect(root.querySelector("math")).not.toBeNull();
  expect(root.querySelector("[src], link, style")).toBeNull();
});
```

Add invalid-math test asserting `.math-error` and visible original TeX.

- [ ] **Step 3: Run Atlas tests and verify failure**

Run:

```sh
cd atlas
corepack pnpm exec vitest run src/app/routes.test.ts src/app/math.test.ts
```

Expected: route shape and `renderCanonicalMath` tests fail.

- [ ] **Step 4: Implement typed section routing**

Change:

```ts
| { kind: "document"; documentId: DocumentId }
```

to:

```ts
| {
    kind: "document";
    documentId: DocumentId;
    sectionId: string | null;
  }
```

Parse only canonical IDs matching:

```ts
const sectionIdPattern = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
```

Unknown or malformed sections become `null`, not a fallback route.

- [ ] **Step 5: Implement MathML rendering**

Create:

```ts
import katex from "katex";

export function renderCanonicalMath(root: ParentNode): void {
  for (const element of root.querySelectorAll<HTMLElement>(".math[data-tex]")) {
    const tex = element.dataset.tex ?? element.textContent ?? "";
    try {
      katex.render(tex, element, {
        displayMode: element.classList.contains("math-display"),
        output: "mathml",
        throwOnError: true,
        trust: false,
        strict: "error",
      });
      element.dataset.tex = tex;
      element.classList.remove("math-error");
    } catch {
      element.replaceChildren(document.createTextNode(tex));
      element.dataset.tex = tex;
      element.classList.add("math-error");
    }
  }
}
```

- [ ] **Step 6: Render math and focus sections in `CanonicalDocumentView`**

Add props:

```ts
sectionId?: string | null;
```

Use a `ref` on the article and one `useEffect`:

```ts
useEffect(() => {
  const article = articleRef.current;
  if (!article) return;
  renderCanonicalMath(article);
  if (!sectionId) return;
  const heading = article.querySelector<HTMLElement>(`#${CSS.escape(sectionId)}`);
  if (!heading) return;
  heading.tabIndex = -1;
  heading.focus({ preventScroll: true });
  heading.scrollIntoView?.({ block: "start" });
}, [document.html_sha256, sectionId]);
```

Pass `sectionId` through `AuxiliaryDocumentReader`.

- [ ] **Step 7: Add stable styles**

Add:

```css
.math-inline {
  display: inline-block;
  max-width: 100%;
  vertical-align: -0.12em;
}

.math-display {
  display: block;
  max-width: 100%;
  overflow-x: auto;
  padding-block: 0.5rem;
}

.math-error {
  color: var(--orange-dark);
  font-family: var(--font-utility);
  white-space: pre-wrap;
}

.canonical-markdown :is(h2, h3):focus {
  outline: 2px solid var(--orange);
  outline-offset: 4px;
}
```

- [ ] **Step 8: Run reader tests and export**

Run:

```sh
cd atlas
corepack pnpm exec vitest run
corepack pnpm exec tsc -b
corepack pnpm run export:html
node tests/static-export.test.mjs
```

Expected: all Atlas tests pass; export contains bundled KaTeX code and no external script, stylesheet, font, or network fetch.

- [ ] **Step 9: Refresh the import receipt**

Stage all reader files and the two derived Atlas files, run `repository verify`, patch the reported digest, and restage:

```sh
git add crates/harp/src/corpus/mod.rs crates/harp/src/corpus/contracts.rs \
  crates/harp/src/corpus/render.rs crates/harp/src/corpus/tests.rs \
  atlas/package.json atlas/pnpm-lock.yaml atlas/src/app atlas/src/styles/components.css \
  atlas/tests/static-export.test.mjs atlas/dist/harp-atlas.html \
  atlas/dist/harp-atlas.receipt.json
cargo run -q -p harp -- repository verify
```

- [ ] **Step 10: Verify the reader commit**

Run:

```sh
cargo test -p harp corpus::tests --lib -- --test-threads=1
cd atlas && corepack pnpm run test:export
cd ..
cargo run -p harp -- repository verify
mise run verify
git diff --cached --check
```

Expected: all commands pass.

- [ ] **Step 11: Commit**

```sh
git add crates/harp/src/corpus/mod.rs crates/harp/src/corpus/contracts.rs \
  crates/harp/src/corpus/render.rs crates/harp/src/corpus/tests.rs \
  atlas/package.json atlas/pnpm-lock.yaml atlas/src/app atlas/src/styles/components.css \
  atlas/tests/static-export.test.mjs atlas/dist/harp-atlas.html \
  atlas/dist/harp-atlas.receipt.json docs/import-receipt.md
git commit -m "feat: add addressable math documents to Atlas" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 5: Add the failing packet contract

**Files:**

- Create: `crates/harp/tests/crouzeix_conjecture_knowledge_packet.rs`

- [ ] **Step 1: Define exact file, frontmatter, and claim contracts**

Start with:

```rust
const EXPECTED_FILES: [&str; 13] = [
    "01_problem_and_prior_barrier.md",
    "02_shared_power_family.md",
    "03_jin_proof_spine.md",
    "04_jin_positive_real_completion.md",
    "05_lorist_schwenninger_proof.md",
    "06_proof_comparison.md",
    "07_jin_lean_verification.md",
    "08_ai_assisted_discovery.md",
    "09_status_and_critical_assessment.md",
    "claim_evidence_ledger.md",
    "crouzeix_conjecture_index.md",
    "glossary.md",
    "source_registry.md",
];

const REQUIRED_CLAIMS: [&str; 34] = [
    "CC-001", "CC-002", "CC-003", "CC-004", "CC-005",
    "CC-010", "CC-011", "CC-012", "CC-013", "CC-014",
    "CC-015", "CC-016", "CC-017", "CC-020", "CC-021",
    "CC-022", "CC-023", "CC-024", "CC-025", "CC-030",
    "CC-031", "CC-032", "CC-033", "CC-034", "CC-035",
    "CC-036", "CC-037", "CC-040", "CC-041", "CC-042",
    "CC-043", "CC-044", "CC-045", "CC-046",
];
```

The test must reuse the field and relationship semantics from `credible_docs_style.rs`:

- source-derived entries require `Mode` and `Source stability`;
- dated observations require `Observed`;
- inferences require `Weakens if` and `Falsified by`;
- missing entries require `Resolves when`;
- `Locator` links must resolve beneath `evidence/`;
- repeated `Relationship` fields use `<kind> <CC-ID>`.

- [ ] **Step 2: Add mechanism and status assertions**

Assert the packet includes:

```rust
assert!(completion.contains("H\\text{ analytic on }\\mathbb D"));
assert!(completion.contains("|\\lambda_i|\\le1"));
assert!(completion.contains("4\\widehat Y-\\widehat Y\\widehat P"));
assert!(lorist.contains("\\|T^n\\|"));
assert!(lorist.contains("M(2+M)"));
assert!(!lorist.contains("missing hypothesis"));
assert!(lean.contains("positiveRealCompletionStatement"));
assert!(status.contains("Preprints.org"));
assert!(status.contains("metadata only"));
```

Assert every non-index file ends with:

```markdown
Back to the [Crouzeix conjecture index](crouzeix_conjecture_index.md).
```

- [ ] **Step 3: Run the packet test and verify failure**

Run:

```sh
cargo test -p harp --test crouzeix_conjecture_knowledge_packet
```

Expected: failure because `knowledge/crouzeix_conjecture/` does not exist.

---

### Task 6: Write the source registry, glossary, ledger, and foundations

**Files:**

- Create: `knowledge/crouzeix_conjecture/crouzeix_conjecture_index.md`
- Create: `knowledge/crouzeix_conjecture/01_problem_and_prior_barrier.md`
- Create: `knowledge/crouzeix_conjecture/02_shared_power_family.md`
- Create: `knowledge/crouzeix_conjecture/glossary.md`
- Create: `knowledge/crouzeix_conjecture/source_registry.md`
- Create: `knowledge/crouzeix_conjecture/claim_evidence_ledger.md`

- [ ] **Step 1: Create frontmatter and authority notice**

Use this exact frontmatter shape:

```yaml
---
id: crouzeix-conjecture-index
title: Crouzeix conjecture two-proof index
type: index
status: active
created: 2026-08-14
updated: 2026-08-14
tags: [crouzeix-conjecture, numerical-range, spectral-set, formal-verification]
confidence: medium
canonical: crouzeix_conjecture_index.md
---
```

Every file includes:

```markdown
> This file is a maintained Harp technical packet. It separates Harp's
> synthesis from primary-source claims; primary-source receipts and local
> verification logs live under `evidence/`.
```

- [ ] **Step 2: Write the source registry**

Required `##` source headings:

```markdown
## JIN-V4-AUDITED: Formalization-matched Git manuscript
## JIN-REPO-HEAD: Pinned repository head
## JIN-V4-HEAD: Later v4 manuscript
## JIN-ANNMATH: Annals-formatted manuscript
## JIN-PREPRINTS-V1: Preprints.org metadata record
## LS-ARXIV-V1: Lorist-Schwenninger arXiv v1
## CROUZEIX-2007: Earlier universal numerical-range bound
## CROUZEIX-PALENCIA-2017: One-plus-square-root-two result
## DELYON-DELYON-1999: Double-layer calculus
## RANSFORD-SCHWENNINGER-2018: Prior proof analysis
## SCHWENNINGER-DEVRIES-2025: Double-layer review
## HARP-LOCAL-VERIFY: Local build and scan observations
```

Each entry states class, title, local manifest/log route, immutable identity, semantic locators, what it can prove, and what it cannot prove.

- [ ] **Step 3: Write the canonical ledger**

Create all 34 claims from the fixed roster. Every source-derived claim links:

```markdown
- Source: [JIN-V4-AUDITED](source_registry.md#jin-v4-audited-formalization-matched-git-manuscript)
- Locator: [JIN-V4-AUDITED-TEX receipt](../../evidence/crouzeix_conjecture/source_manifest.tsv#L24)
- Upstream locator: [Positive-real completion theorem](https://github.com/jinshanmu/CrouzeixConjecture/blob/565b6a3e0659b6e0785f783b016c3f6d9f171fa5/preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L319)
```

Use explicit claim heading IDs:

```markdown
## CC-010: Jin completion hypotheses {#cc-010-jin-completion-hypotheses}
```

- [ ] **Step 4: Write foundations and index routes**

The index must link all thirteen files and expose:

- expert route;
- guided route;
- "How does Jin remove the adjoint correction?";
- "How does the \(2\)-dilation recurrence work?";
- "What does Lean actually verify?";
- "What did the AI systems contribute?";
- "What evidence remains missing?".

`01_problem_and_prior_barrier.md` owns `CC-001` through `CC-005`.

`02_shared_power_family.md` owns `CC-040` and explicitly says the shared-power-family framing is Harp's inference, not source terminology.

- [ ] **Step 5: Run the packet test**

Run:

```sh
cargo test -p harp --test crouzeix_conjecture_knowledge_packet
```

Expected: still fails because numbered mechanism/status chapters are missing, but source registry and ledger parsing pass.

---

### Task 7: Write the Jin proof and Lean audit chapters

**Files:**

- Create: `knowledge/crouzeix_conjecture/03_jin_proof_spine.md`
- Create: `knowledge/crouzeix_conjecture/04_jin_positive_real_completion.md`
- Create: `knowledge/crouzeix_conjecture/07_jin_lean_verification.md`

- [ ] **Step 1: Write the Jin proof spine**

Required sections:

```markdown
## Normalize on a fixed outer domain
## Build the full Cayley family
## Produce a completion modulo the auxiliary adjoint algebra
## Apply positive-real completion
## Pass from simple spectrum to the target matrix
## Shrink the outer domains
## Polynomial and rational consequences
```

State `CC-017` with the two limits in the formalized order. Keep the older `f_eta`/Stein route in a separate revision-history subsection.

- [ ] **Step 2: Write the positive-real completion derivation**

Required displayed equations:

```tex
|\lambda_i|\le1,\qquad
H\text{ analytic on }\mathbb D,\qquad H(0)=I
```

```tex
H(w)-(I-wT)^{-1}\in\operatorname{alg}(B^*)
```

```tex
w_i=\frac{\overline{\lambda_i}}2,\qquad
v=-G^{-1}Pu
```

```tex
4Y-YG^{-1}P-PG^{-1}Y\succeq0
```

```tex
4\widehat Y-\widehat Y\widehat P-\widehat P\widehat Y\succeq0
```

Derive `CC-013` through `CC-016` in order.

- [ ] **Step 3: Write the Lean audit**

Draw two source-level graphs:

```text
crouzeixConjecture
  <- holomorphicCrouzeixBound
  <- fixed-domain double-layer bound
  <- positiveRealCompletionStatement
  <- PositiveRealCompletionStatement
```

and:

```text
crouzeixRationalSpectralSetCorollary
crouzeixRationalBound
crouzeixConstantTwo_isLeast_finTwo
```

State that `PositiveRealCompletionStatement` is a proposition definition and `positiveRealCompletionStatement` proves it.

Link `CC-023`, `CC-024`, and `CC-025` to local verification logs.

- [ ] **Step 4: Run packet and math checks**

Run:

```sh
cargo test -p harp --test crouzeix_conjecture_knowledge_packet
cargo test -p harp corpus::tests --lib -- --test-threads=1
```

Expected: packet test still reports the missing Lorist/comparison/discovery/status files; all Jin mechanism assertions pass.

---

### Task 8: Write the Lorist-Schwenninger proof and comparison

**Files:**

- Create: `knowledge/crouzeix_conjecture/05_lorist_schwenninger_proof.md`
- Create: `knowledge/crouzeix_conjecture/06_proof_comparison.md`

- [ ] **Step 1: Write Lemma 1 and recurrence**

Include:

```tex
E_n=2V^*Q^{*n}V-T^{*n}
```

```tex
\kappa m_n-m_{n+1}\ge r_n
```

```tex
m_1\ge\kappa^{-N}m_{N+1}
 +\sum_{n=1}^{N}\kappa^{-n}r_n
```

Then make the compressed step explicit:

```tex
\|T^n\|=\|T^{*n}\|
\le2\|V^*Q^{*n}V\|+\|E_n\|
\le2+M
```

```tex
\|E_nT^n\|\le M(2+M)
```

Do not describe this as a missing hypothesis.

- [ ] **Step 2: Write the double-layer realization**

Include:

```tex
Vx=2^{-1/2}P_\Omega(\cdot)^{1/2}x,\qquad Qg=fg
```

```tex
E_n=\alpha(f^n)(A)
```

```tex
E_nT^n=(\alpha(f^n)f^n)(A)
```

Separate source statements (`CC-030`, `CC-031`, `CC-033`, `CC-035`, `CC-036`, `CC-037`) from Harp derivations (`CC-032`, `CC-034`).

- [ ] **Step 3: Write the fixed comparison**

Use the design's seven-row comparison table:

- retained family;
- finite-dimensional lemma;
- decisive cancellation/estimate;
- order-sensitive input;
- double-layer route;
- formalization evidence;
- completely bounded limitation.

`CC-041` must quote or paraphrase the arXiv v1 sentence that Jin's proof appeared independently, while noting that this is the authors' statement.

- [ ] **Step 4: Run packet tests**

Run:

```sh
cargo test -p harp --test crouzeix_conjecture_knowledge_packet
```

Expected: only discovery/status files and registration checks remain failing.

---

### Task 9: Write AI-assisted discovery and status chapters

**Files:**

- Create: `knowledge/crouzeix_conjecture/08_ai_assisted_discovery.md`
- Create: `knowledge/crouzeix_conjecture/09_status_and_critical_assessment.md`

- [ ] **Step 1: Write the AI contribution matrix**

Use:

| Documented action | Source | Context/tool | Supported conclusion |
|---|---|---|---|
| Search for a complete standalone proof | Jin prompt receipt | Multi-agent instruction | The task contract asked for diverse approaches and adversarial checks |
| Sample at half conjugate eigenvalues and add origin sample | Jin disclosure | OpenAI ChatGPT | The manuscript attributes this idea to ChatGPT |
| Explore proof strategies | LS disclosure | ChatGPT 5.6 Pro | The authors report strategy exploration |
| Prepare exposition/bibliography/typesetting | Jin disclosure | OpenAI ChatGPT | Assistance beyond the central proof idea |

Add `CC-044` for absent transcripts, per-agent prompts, complete rejected-route history, time, and token accounting.

- [ ] **Step 2: Write status and source-identity boundaries**

Required sections:

```markdown
## Strongest justified conclusion
## Jin Git manuscript versus Preprints.org metadata
## Manuscript versus Lean revision identity
## Lorist-Schwenninger publication status
## What the two proof artifacts do not validate about each other
## Evidence that would raise confidence
```

Use `CC-045` for unmapped Preprints.org manuscript bytes and `CC-046` for absent Annals submission evidence.

- [ ] **Step 3: Run complete packet tests**

Run:

```sh
cargo test -p harp --test crouzeix_conjecture_knowledge_packet
```

Expected: content, frontmatter, ledger, source, backlink, and mechanism checks pass; only corpus/search/Atlas registration checks fail.

---

### Task 10: Register the packet, add the first-class route, regenerate outputs, and commit

**Files:**

- Modify: `crates/harp/src/corpus/mod.rs`
- Modify: `crates/harp/src/search.rs:236-247`
- Modify: `crates/harp/src/corpus/tests.rs`
- Modify: `crates/harp/tests/cli.rs`
- Modify: `atlas/src/content/types.ts`
- Modify: `atlas/src/content/canonical.test.ts`
- Modify: `atlas/src/app/AtlasApp.tsx`
- Modify: `atlas/src/app/routes.test.ts`
- Modify: `docs/product-contract.md`
- Regenerate: `atlas/src/content/generated/corpus.json`
- Regenerate: `atlas/dist/harp-atlas.html`
- Regenerate: `atlas/dist/harp-atlas.receipt.json`
- Modify: `docs/import-receipt.md`

- [ ] **Step 1: Register one route and twelve auxiliaries**

Change array sizes:

```rust
pub(super) const READER_ROUTES: [(&str, &str, &str); 12] = [
```

Add:

```rust
(
    "crouzeix-conjecture",
    "Crouzeix",
    "knowledge/crouzeix_conjecture/crouzeix_conjecture_index.md",
),
```

Change:

```rust
pub(super) const AUXILIARY_DOCUMENTS: [(&str, &str); 28] = [
```

Add IDs:

```text
crouzeix-problem-and-prior-barrier
crouzeix-shared-power-family
crouzeix-jin-proof-spine
crouzeix-jin-positive-real-completion
crouzeix-lorist-schwenninger-proof
crouzeix-proof-comparison
crouzeix-jin-lean-verification
crouzeix-ai-assisted-discovery
crouzeix-status-and-critical-assessment
crouzeix-glossary
crouzeix-source-registry
crouzeix-claim-evidence-ledger
```

- [ ] **Step 2: Add search root**

Add:

```rust
(
    "knowledge/crouzeix_conjecture",
    "canonical-markdown",
    "md",
),
```

Update search fixtures to create and query:

```markdown
# Crouzeix

The origin sample cancels the diagonal correction.
```

- [ ] **Step 3: Add TypeScript route identity and visible navigation**

Add `"crouzeix-conjecture"` to `ReaderRouteId` and `readerRouteIds`.

Add route fixture:

```ts
{
  route_id: "crouzeix-conjecture",
  label: "Crouzeix",
  canonical_markdown_path:
    "knowledge/crouzeix_conjecture/crouzeix_conjecture_index.md",
}
```

In `AtlasApp`, resolve the route from `canonicalReaderRoutes` and add a visible Crouzeix button. Change legacy labels to use:

```ts
canonicalReaderRoutes.find(
  (candidate) => candidate.route.route_id === route.routeId,
)?.route.label ?? route.routeId
```

- [ ] **Step 4: Update exact counts and product contract**

Update:

```text
canonical_documents: 82
reader routes: 12
auxiliary documents: 28
```

Do not change retained concepts, coverage, systems, Weng sections, diagnostics, lessons, source-registry rows, or evidence edges.

Document the packet as a bounded research packet on AI-assisted theorem discovery, formal verification, and evidence discipline, outside the RSI taxonomy.

- [ ] **Step 5: Run focused registration tests**

Run:

```sh
cargo test -p harp --test crouzeix_conjecture_knowledge_packet
cargo test -p harp corpus::tests --lib -- --test-threads=1
cargo test -p harp --test cli
cd atlas && corepack pnpm exec vitest run
```

Expected: all focused tests pass.

- [ ] **Step 6: Regenerate all three Atlas artifacts**

Run:

```sh
cargo run -p harp -- build
cargo run -p harp -- build --check
cd atlas
corepack pnpm run export:html
node tests/static-export.test.mjs
cd ..
```

Expected:

```text
Harp corpus checked: 82 documents, 75 concepts
atlas/src/content/generated/corpus.json matches canonical inputs
wrote .../atlas/dist/harp-atlas.html
```

- [ ] **Step 7: Verify search**

Run:

```sh
cargo run -p harp -- search refresh
cargo run -p harp -- search query '"origin sample"' --limit 5
```

Expected: a result with path:

```text
knowledge/crouzeix_conjecture/04_jin_positive_real_completion.md
```

- [ ] **Step 8: Refresh final import receipt**

Stage all packet, registration, test, contract, lockfile, and generated paths. Run:

```sh
cargo run -q -p harp -- repository verify
```

Patch only the reported digest into `docs/import-receipt.md` and restage it.

- [ ] **Step 9: Run release gate**

Run:

```sh
git diff --cached --check
mise run verify
git status --short
```

Expected: release gate passes. Only the intentionally ignored local research bundle may exist outside Git status.

- [ ] **Step 10: Commit**

```sh
git add knowledge/crouzeix_conjecture \
  crates/harp/src/corpus/mod.rs crates/harp/src/search.rs \
  crates/harp/src/corpus/tests.rs crates/harp/tests/cli.rs \
  crates/harp/tests/crouzeix_conjecture_knowledge_packet.rs \
  atlas/src/content/types.ts atlas/src/content/canonical.test.ts \
  atlas/src/app/AtlasApp.tsx atlas/src/app/routes.test.ts \
  atlas/src/content/generated/corpus.json atlas/dist/harp-atlas.html \
  atlas/dist/harp-atlas.receipt.json docs/product-contract.md \
  docs/import-receipt.md
git commit -m "docs: add Crouzeix conjecture two-proof packet" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

---

### Task 11: Final branch review and landing preparation

**Files:**

- Review all files changed from `c092c56`.

- [ ] **Step 1: Review commit topology**

Run:

```sh
git log --oneline --decorate c092c56..HEAD
git diff --stat c092c56..HEAD
git status --short
```

Expected: design commit plus three implementation commits, no tracked changes.

- [ ] **Step 2: Run code and spec review**

Use the code-review workflow against:

```text
c092c56..HEAD
```

Review axes:

- Harp standards and `AGENTS.md`;
- approved two-proof design;
- rights and provenance;
- mathematical hypothesis/order fidelity;
- generated artifact and receipt consistency.

- [ ] **Step 3: Resolve every blocking review finding**

For each accepted finding:

1. add or update a regression test;
2. reproduce the failure;
3. patch the smallest owning module;
4. rerun focused tests;
5. refresh generated artifacts and import receipt if tracked bytes changed;
6. rerun `mise run verify`;
7. amend the owning focused commit without changing its trailer.

- [ ] **Step 4: Run final exact-commit gate**

Run:

```sh
git status --porcelain=v1
mise run verify
cargo run -q -p harp -- repository verify
git log -4 --format='%H%n%B%n---'
```

Expected:

- clean tracked worktree;
- full gate passes;
- repository verifies 521 import rows;
- every new commit contains exactly one
  `Co-authored-by: TRAE CLI <noreply@bytedance.com>` trailer.

- [ ] **Step 5: Report landing-ready state**

Report:

- worktree path;
- branch name;
- four commit hashes and subjects;
- source and verification receipt counts;
- packet document count;
- final corpus and search counts;
- `mise run verify` result;
- remaining ignored local research paths;
- no push performed.

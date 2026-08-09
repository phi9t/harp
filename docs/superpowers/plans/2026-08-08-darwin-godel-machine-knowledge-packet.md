# Darwin Gödel Machine knowledge packet implementation plan

> **For agentic workers:** Implement this plan task by task. Keep
> `knowledge/darwin_godel_machine/` non-authoritative and preserve `content/`
> as Harp's only canonical technical-prose root.

**Goal:** Build an Obsidian-friendly, MTS-level DGM research and learning packet
with expert and guided routes, source-backed technical chapters, exercises,
claim accounting, and automated integrity checks.

**Architecture:** The packet is a learning projection over canonical Harp
content and pinned DGM evidence. Sixteen focused Markdown files divide
navigation, foundations, mechanism, implementation, evaluation, safety,
critique, teaching, and maintenance. A Rust integration test enforces packet
shape without adding the packet to the Atlas corpus.

**Tech stack:** Markdown, YAML frontmatter, Mermaid, LaTeX, Rust integration
tests, and the existing Harp verification tasks.

---

## File map

### Learning packet

- `knowledge/darwin_godel_machine/darwin_godel_machine_index.md`: Obsidian
  entrypoint and reading routes.
- `knowledge/darwin_godel_machine/01_orientation.md`: prerequisites,
  vocabulary, five-minute model, and claim boundary.
- `knowledge/darwin_godel_machine/02_paper_walkthrough.md`: section-by-section
  paper argument and reading questions.
- `knowledge/darwin_godel_machine/03_algorithm_derivation.md`: equations,
  pseudocode, baselines, staged evaluation, and worked selection example.
- `knowledge/darwin_godel_machine/04_system_architecture.md`: data flow,
  candidate/envelope split, state transitions, and trust boundaries.
- `knowledge/darwin_godel_machine/05_repository_walkthrough.md`: pinned source
  walkthrough in runtime order.
- `knowledge/darwin_godel_machine/06_evaluation_analysis.md`: result tables,
  accounting boundaries, transfer, cost, and confounds.
- `knowledge/darwin_godel_machine/07_open_endedness.md`: stepping stones,
  archive semantics, diversity, and search limits.
- `knowledge/darwin_godel_machine/08_safety_and_failure.md`: containment,
  objective hacking, private-test exposure, and minimum controls.
- `knowledge/darwin_godel_machine/09_critical_review.md`: consequence-ordered
  MTS review.
- `knowledge/darwin_godel_machine/10_successor_design.md`: matched next-cycle
  successor experiment.
- `knowledge/darwin_godel_machine/learning_path.md`: exercises, checkpoints,
  and answer guidance.
- `knowledge/darwin_godel_machine/glossary.md`: DGM terminology and common
  misunderstandings.
- `knowledge/darwin_godel_machine/claim_evidence_crosswalk.md`: material claim
  ledger.
- `knowledge/darwin_godel_machine/source_registry.md`: packet source classes
  and claim ceilings.
- `knowledge/darwin_godel_machine/maintenance.md`: update and verification
  runbook.

### Integrity check

- `crates/harp/tests/dgm_knowledge_packet.rs`: validates file count,
  frontmatter, authority notices, backlinks, relative links, source IDs,
  crosswalk columns, pinned revision, placeholder absence, and the missing
  `README.md`.

## Task 1: Build navigation and foundations

**Files:**

- Create: `knowledge/darwin_godel_machine/darwin_godel_machine_index.md`
- Create: `knowledge/darwin_godel_machine/01_orientation.md`
- Create: `knowledge/darwin_godel_machine/02_paper_walkthrough.md`
- Create: `knowledge/darwin_godel_machine/glossary.md`
- Create: `knowledge/darwin_godel_machine/source_registry.md`

**Acceptance criteria:**

- The index exposes expert, guided, and question-driven routes.
- Each file has the required frontmatter and authority notice.
- Orientation distinguishes Gödel Machine, DGM, harness improvement, successor
  improvement, and full RSI.
- The paper walkthrough covers all main sections and load-bearing appendices.
- The glossary defines every symbol used by the derivation.
- The source registry states what each source can and cannot prove.

**Verification:**

```sh
rg -n '^## ' knowledge/darwin_godel_machine/{darwin_godel_machine_index,01_orientation,02_paper_walkthrough,glossary,source_registry}.md
```

Expected: each file has a stable section structure and no missing file.

## Task 2: Build mechanism and implementation chapters

**Files:**

- Create: `knowledge/darwin_godel_machine/03_algorithm_derivation.md`
- Create: `knowledge/darwin_godel_machine/04_system_architecture.md`
- Create: `knowledge/darwin_godel_machine/05_repository_walkthrough.md`
- Create: `knowledge/darwin_godel_machine/07_open_endedness.md`

**Acceptance criteria:**

- The derivation defines every term in the parent-selection equation.
- A four-agent numerical example reconciles unnormalized weights and
  probabilities to rounding.
- Architecture includes end-to-end and trust-boundary Mermaid diagrams.
- The repository walkthrough cites the pinned snapshot in runtime order.
- Open-endedness separates lineage diversity from behavioral diversity.

**Verification:**

```sh
rg -n 'a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2|```mermaid|Worked example' \
  knowledge/darwin_godel_machine/{03_algorithm_derivation,04_system_architecture,05_repository_walkthrough,07_open_endedness}.md
```

Expected: pinned revision, diagrams, and numerical example are present.

## Task 3: Build evaluation, safety, critique, and teaching chapters

**Files:**

- Create: `knowledge/darwin_godel_machine/06_evaluation_analysis.md`
- Create: `knowledge/darwin_godel_machine/08_safety_and_failure.md`
- Create: `knowledge/darwin_godel_machine/09_critical_review.md`
- Create: `knowledge/darwin_godel_machine/10_successor_design.md`
- Create: `knowledge/darwin_godel_machine/learning_path.md`
- Create: `knowledge/darwin_godel_machine/claim_evidence_crosswalk.md`
- Create: `knowledge/darwin_godel_machine/maintenance.md`

**Acceptance criteria:**

- Result tables state benchmark, task count, metric, model, locator, and
  reproduction status.
- Safety distinguishes paper safeguards, released-code behavior, and
  recommendations.
- Critical review orders findings by consequence.
- Successor design specifies a matched parent/child next-cycle test.
- Learning path reaches system-design and experiment-design exercises.
- The crosswalk accounts for every major quantitative and architectural claim.
- Maintenance documents exact update and verification steps.

**Verification:**

```sh
rg -n 'Reproduction status|objective hacking|next-cycle|Claim ID|Back to the DGM index' \
  knowledge/darwin_godel_machine/{06_evaluation_analysis,08_safety_and_failure,09_critical_review,10_successor_design,learning_path,claim_evidence_crosswalk,maintenance}.md
```

Expected: each required contract appears in its owning document.

## Task 4: Add and run packet integrity checks

**Files:**

- Create: `crates/harp/tests/dgm_knowledge_packet.rs`
- Modify: `docs/import-receipt.md` only if `repository verify` reports a new
  expected payload digest.

**Implementation contract:**

The integration test uses only the Rust standard library. It:

1. enumerates the exact 16 expected Markdown files;
2. rejects `README.md`;
3. parses the frontmatter envelope and checks required keys;
4. checks the exact authority notice;
5. checks the index backlink on every non-index file;
6. validates repository-relative Markdown links that do not contain a URL;
7. rejects absolute local paths and placeholder tokens;
8. compares the commit in `05_repository_walkthrough.md` and
   `source_registry.md` with `evidence/implementations/dgm/REVISION`;
9. checks the crosswalk header; and
10. checks `DGM` and `DGM-REPO` in the packet source registry.

**Verification:**

```sh
cargo test -p harp --test dgm_knowledge_packet
cargo run -p harp -- check
cargo run -p harp -- build --check
cargo run -p harp -- sources verify
cd atlas && corepack pnpm run test
cd atlas && corepack pnpm run test:export
cargo run -p harp -- repository verify
git diff --check
```

Expected: all commands pass. The packet remains outside the compiled Atlas
document count.

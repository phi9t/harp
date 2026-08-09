# Darwin Gödel Machine knowledge packet design

**Status:** Approved design, pending implementation planning
**Date:** 2026-08-08
**Audience:** Mixed Member of Technical Staff audience
**Packet root:** `knowledge/darwin_godel_machine/`

## 1. Objective

Create a deep and broad learning packet for the Darwin Gödel Machine paper and
released implementation. The packet should help an MTS-level reader:

1. reconstruct the paper's argument and algorithm;
2. trace the released implementation from archive selection through evaluation;
3. distinguish candidate-controlled code from the protected search envelope;
4. audit reported results, transfer claims, safety controls, and confounds;
5. explain open-ended search and stepping-stone behavior;
6. critique whether the evidence supports recursive self-improvement; and
7. design a more rigorous successor experiment.

The packet optimizes for system design, technical critique, and teaching. It is
not a reproduction report or a replacement for Harp's canonical DGM article.

## 2. Ownership boundary

`content/` remains Harp's only authority for technical prose. The new packet is
a non-authoritative learning projection.

The packet may:

- reorganize canonical material into reading routes;
- derive equations and walk through algorithms for teaching;
- pose exercises and design-review questions;
- compare paper claims with released-code behavior;
- link claims to canonical Harp pages and pinned evidence; and
- state researcher assessments when they are labeled as such.

The packet must not:

- become the sole home of a material technical claim;
- introduce a benchmark number without a canonical or primary-source locator;
- rewrite captured evidence;
- claim independent reproduction;
- change the Atlas corpus contract;
- become a build, test, or runtime dependency; or
- depend on another local checkout.

Every document will include a short notice:

> This file is a learning projection. Canonical claims live under `content/`;
> primary-source captures and pinned implementation files live under
> `evidence/`.

## 3. Reading model

The packet has two routes over the same documents.

### 3.1 Expert route

An experienced agent-systems engineer should be able to read:

1. `darwin_godel_machine_index.md`;
2. `04_system_architecture.md`;
3. `05_repository_walkthrough.md`;
4. `06_evaluation_analysis.md`;
5. `09_critical_review.md`; and
6. `10_successor_design.md`.

This route prioritizes ownership boundaries, implementation flow, experimental
accounting, and design consequences.

### 3.2 Guided route

A reader who wants derivations and checkpoints should read:

1. `darwin_godel_machine_index.md`;
2. `01_orientation.md`;
3. `02_paper_walkthrough.md`;
4. `03_algorithm_derivation.md`;
5. `07_open_endedness.md`;
6. `04_system_architecture.md`;
7. `05_repository_walkthrough.md`;
8. `06_evaluation_analysis.md`;
9. `08_safety_and_failure.md`;
10. `09_critical_review.md`;
11. `10_successor_design.md`; and
12. `learning_path.md`.

The guided route moves from vocabulary and motivation to equations, concrete
code, experimental critique, and successor-system design.

## 4. Packet structure

```text
knowledge/darwin_godel_machine/
├── darwin_godel_machine_index.md
├── 01_orientation.md
├── 02_paper_walkthrough.md
├── 03_algorithm_derivation.md
├── 04_system_architecture.md
├── 05_repository_walkthrough.md
├── 06_evaluation_analysis.md
├── 07_open_endedness.md
├── 08_safety_and_failure.md
├── 09_critical_review.md
├── 10_successor_design.md
├── learning_path.md
├── glossary.md
├── claim_evidence_crosswalk.md
├── source_registry.md
└── maintenance.md
```

No `README.md` will be created. `darwin_godel_machine_index.md` is the
Obsidian-visible packet entrypoint.

## 5. Document contracts

### 5.1 `darwin_godel_machine_index.md`

Owns packet navigation, not technical claims.

Required sections:

- packet status and authority notice;
- one-paragraph orientation;
- expert route;
- guided route;
- question-driven routes such as "How does parent selection work?" and "Does
  DGM demonstrate RSI?";
- document map;
- source boundary;
- maintenance status; and
- links to canonical DGM content and pinned implementation evidence.

All packet documents link back to this index.

### 5.2 `01_orientation.md`

Introduces the problem, prerequisites, vocabulary, and claim boundary.

Required topics:

- theoretical Gödel Machine versus empirical DGM;
- coding agent, self-modification, archive, parent, child, and stepping stone;
- frozen model versus mutable agent code;
- candidate state versus protected envelope;
- what the paper claims;
- what the paper does not claim; and
- a five-minute conceptual model.

### 5.3 `02_paper_walkthrough.md`

Reconstructs the paper section by section without paraphrasing every paragraph.

Required topics:

- argument map from motivation to conclusion;
- related-work positioning;
- method;
- experiment design;
- results;
- transfer;
- safety;
- limitations;
- appendices that carry material evidence; and
- questions to ask while reading each section.

Each section records the paper locator and separates author claims from packet
assessment.

### 5.4 `03_algorithm_derivation.md`

Teaches the algorithm from first principles.

Required topics:

- archive state;
- eligibility rule;
- sigmoid-scaled performance;
- child-count exploration bonus;
- normalized parent-selection probability;
- sampling with replacement;
- self-modification and evaluation;
- validity and archive admission;
- staged evaluation;
- baseline algorithms; and
- worked numerical example.

The worked example will calculate parent probabilities for at least four
agents with different scores and child counts. It will show how a lower-scoring
underexplored lineage can remain selectable.

### 5.5 `04_system_architecture.md`

Owns the system-design view.

Required topics:

- end-to-end data flow;
- mutable candidate boundary;
- protected outer-loop boundary;
- diagnostic model role;
- coding model role;
- benchmark harness role;
- patch-lineage materialization;
- archive metadata;
- execution and timeout boundaries;
- component/responsibility matrix;
- state-transition walkthrough; and
- trust-boundary diagram.

At least two Mermaid diagrams are required:

1. end-to-end self-improvement flow; and
2. candidate versus protected-envelope boundary.

### 5.6 `05_repository_walkthrough.md`

Explains the released repository at pinned commit
`a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`.

Required paths:

- `DGM_outer.py`;
- `self_improve_step.py`;
- `coding_agent.py`;
- `coding_agent_polyglot.py`;
- `llm_withtools.py`;
- `prompts/self_improvement_prompt.py`;
- `tools/bash.py`;
- `tools/edit.py`;
- `utils/evo_utils.py`;
- `swe_bench/harness.py`;
- `swe_bench/report.py`; and
- `polyglot/harness.py`.

The walkthrough follows runtime order rather than file-name order. It includes:

- entrypoints;
- important functions;
- data written between stages;
- patch accumulation;
- model choices;
- error and timeout handling;
- benchmark-specific differences;
- released-code caveats; and
- a "where to start changing the system" table.

### 5.7 `06_evaluation_analysis.md`

Owns quantitative accounting and experimental critique.

Required topics:

- SWE-bench Verified and Polyglot task shapes;
- 10, 50, 140, and 200-task evaluation stages;
- pass@1 boundary;
- model assignments;
- parallelism;
- full-benchmark versus subset numbers;
- no-self-improvement baseline;
- no-open-ended-exploration baseline;
- greedy-selection ablation;
- handcrafted baselines;
- cross-model transfer;
- cross-benchmark transfer;
- cross-language transfer;
- cost and elapsed-time reports;
- stochasticity and winner's-curse risks;
- contamination and private-test exposure boundary; and
- what Harp did and did not reproduce.

Every result table includes benchmark scope, task count, metric, model, source
locator, and reproduction status.

### 5.8 `07_open_endedness.md`

Explains why the archive matters.

Required topics:

- hill climbing versus branching archive search;
- stepping stones;
- temporary performance regressions;
- exploitation versus underexplored-lineage pressure;
- relation to quality-diversity and Go-Explore ideas;
- why `keep_all` is meaningful;
- why archive growth is not itself progress;
- lineage diversity versus behavioral diversity; and
- what would make the search more genuinely open-ended.

### 5.9 `08_safety_and_failure.md`

Owns safety analysis and concrete failure modes.

Required topics:

- generated-code execution;
- Docker containment as implemented;
- limits of treating Docker use as a sandbox proof;
- API credentials and network boundaries;
- time and resource limits;
- evaluator gaming;
- objective hacking in the hallucination case study;
- private-test information in diagnostic prompts;
- archive and rollback claims;
- interpretability loss across generations;
- dual-use risk; and
- minimum controls for future experiments.

The chapter separates paper-described safeguards, released-code behavior, and
recommended controls.

### 5.10 `09_critical_review.md`

Acts as an MTS design-review memo.

Required sections:

- strongest contribution;
- strongest empirical evidence;
- strongest implementation evidence;
- central causal claim;
- unresolved confounds;
- paper-to-code mismatches;
- benchmark and selection risks;
- recursive-improvement claim ceiling;
- alternative explanations;
- what evidence would change the assessment; and
- final technical judgment.

Findings should be ordered by consequence, not by paper section.

### 5.11 `10_successor_design.md`

Turns critique into a more rigorous system proposal.

Required topics:

- explicit candidate package;
- immutable evaluator package;
- development, validation, and held-out splits;
- matched parent-versus-child proposal protocol;
- root-tree budget accounting;
- append-only lineage receipts;
- sandbox and permission policy;
- multi-objective promotion;
- diversity descriptors;
- reproducible model and prompt identity;
- next-cycle improvement metric;
- stopping rules;
- failure recovery; and
- minimum credible experiment.

This is a design exercise, not a claim that the proposed system exists.

### 5.12 `learning_path.md`

Provides exercises and checkpoints.

Required levels:

1. explain the loop in plain language;
2. calculate parent-selection probabilities;
3. trace one child from parent metadata to benchmark report;
4. classify mutable and protected components;
5. audit a reported result;
6. identify an objective-hacking path;
7. conduct a system-design review; and
8. specify a matched next-cycle experiment.

Each exercise includes a prompt, expected reasoning points, and a collapsible
answer or answer link. Exercises should test explanation and design judgment,
not factual recall alone.

### 5.13 `glossary.md`

Owns packet terminology and notation.

Each entry includes:

- concise definition;
- DGM-specific meaning;
- common misunderstanding; and
- canonical source link.

### 5.14 `claim_evidence_crosswalk.md`

Records the packet's major claims.

Columns:

| Claim ID | Class | Claim | Canonical home | Primary evidence | Locator | Reproduction status | Confidence | Caveat |
|---|---|---|---|---|---|---|---|---|

Allowed classes are `EVIDENCE`, `CLAIM`, `INFERENCE`, and `MISSING`.

No chapter may contain an important quantitative or architectural claim that
cannot be mapped to this crosswalk or an existing canonical Harp claim ledger.

### 5.15 `source_registry.md`

Lists only sources used by this packet.

Required source classes:

- DGM paper;
- pinned DGM repository snapshot;
- canonical Harp DGM article;
- relevant Harp RSI chapters;
- Gödel Machine source record;
- quality-diversity source record;
- Go-Explore or other open-ended-search sources already admitted by Harp; and
- comparison systems used in the critique.

The registry states what each source can and cannot prove.

### 5.16 `maintenance.md`

Defines packet upkeep.

Required rules:

- canonical prose changes happen under `content/` first;
- implementation claims remain pinned to a 40-character commit;
- snapshot bytes remain unchanged;
- new quantitative claims need source locators;
- packet links use repository-relative paths;
- the index owns navigation;
- duplicate explanation should be reduced by linking to canonical pages;
- stale claims must be marked, not silently updated; and
- verification commands and expected outcomes are recorded.

## 6. Obsidian conventions

The packet is designed to work in Obsidian without requiring Obsidian-specific
runtime support.

Conventions:

- `darwin_godel_machine_index.md` is the only packet index.
- Every file has YAML frontmatter with `id`, `title`, `type`, `status`,
  `created`, `updated`, `tags`, `confidence`, and `canonical`.
- `type` uses the existing topic-packet vocabulary: `index`, `concept`,
  `deep-dive`, `claim-ledger`, `source-registry`, `learning-path`,
  `derivation`, or `runbook`.
- Links use standard relative Markdown links for portability.
- Optional Obsidian wikilinks may appear only when a standard Markdown link is
  also present.
- Headings use stable wording so links do not churn.
- Mermaid diagrams remain fenced text.
- Equations use standard LaTeX delimiters supported by Obsidian.
- Each document ends with "Back to the DGM index."

## 7. Evidence and citation rules

The packet uses three evidence tiers.

### 7.1 Canonical Harp synthesis

Canonical claims link to:

- `content/systems/dgm.md`;
- `content/chapters/`;
- `content/concepts/`;
- `content/lessons/`; and
- maintained claim or source registries.

### 7.2 Primary paper evidence

Paper claims cite section, appendix, figure, table, or algorithm. Quantitative
claims state their accounting boundary.

### 7.3 Pinned implementation evidence

Code claims cite files beneath:

`evidence/implementations/dgm/snapshot/`

and state commit:

`a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`.

Packet assessments use `INFERENCE`; absent tests or unavailable evidence use
`MISSING`.

## 8. Teaching style

The writing should assume engineering maturity without assuming prior
open-ended-evolution research.

Rules:

- start each chapter with concrete learning outcomes;
- explain the mechanism before discussing significance;
- derive equations symbol by symbol;
- use one worked example before generalizing;
- connect abstract claims to released code;
- include design-review questions;
- state the claim ceiling repeatedly where readers commonly overgeneralize;
- avoid promotional language; and
- prefer precise diagrams, tables, and causal traces over long narrative.

## 9. Verification design

The packet requires lightweight automated consistency checks.

Implementation planning should add a repository-owned check that verifies:

- all 16 required packet files exist;
- no `README.md` exists in the packet;
- every document links to `darwin_godel_machine_index.md`;
- every document has required frontmatter;
- every relative link resolves;
- the pinned DGM commit matches `evidence/implementations/dgm/REVISION`;
- the authority notice appears in every document;
- `claim_evidence_crosswalk.md` has the required columns;
- `source_registry.md` includes `DGM` and `DGM-REPO`;
- no placeholders such as `TODO`, `TBD`, or `FIXME` remain; and
- no absolute local paths appear.

The check should fit Harp's existing verification model without making the
learning packet part of the Atlas corpus.

The final implementation must also run:

```sh
cargo run -p harp -- check
cargo run -p harp -- build --check
cargo run -p harp -- sources verify
cd atlas && corepack pnpm run test
cd atlas && corepack pnpm run test:export
cargo run -p harp -- repository verify
git diff --check
```

If the packet check is implemented as a new `mise` task, `mise run verify`
must invoke it.

## 10. Non-goals

This packet will not:

- rerun the DGM experiments;
- download or redistribute the full experiment-log archive;
- claim the reported results were independently reproduced;
- add a second canonical DGM article;
- copy the DGM paper into `knowledge/`;
- copy the DGM repository outside the existing evidence snapshot;
- modify the DGM implementation;
- redesign the Atlas UI;
- add a repository-wide license; or
- change Harp's canonical document counts unless a separate approved design
  requires it.

## 11. Acceptance criteria

The implementation is complete when:

1. all packet files exist under `knowledge/darwin_godel_machine/`;
2. `darwin_godel_machine_index.md` provides expert, guided, and
   question-driven routes;
3. the packet covers paper mechanism, implementation, evaluation,
   open-endedness, safety, critique, and successor design;
4. equations and one numerical parent-selection example are correct;
5. code walkthrough claims cite the pinned snapshot;
6. quantitative claims state scope and reproduction status;
7. exercises cover comprehension through successor-experiment design;
8. the claim crosswalk and source registry account for material claims;
9. automated packet consistency checks pass;
10. Harp's canonical ownership contract remains unchanged; and
11. the full repository verification gate passes.

## 12. Approved design deltas

The approved design differs from a generic research packet in four ways:

- the entrypoint is `darwin_godel_machine_index.md`, not `README.md`;
- the audience is mixed MTS, with expert and guided routes;
- the packet prioritizes system design, critique, and teaching; and
- the packet is a non-authoritative learning overlay over canonical `content/`
  and pinned `evidence/`.

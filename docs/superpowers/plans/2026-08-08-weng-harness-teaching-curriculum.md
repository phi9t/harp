# Weng Harness Teaching Curriculum Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a source-grounded teaching workspace that covers all nine
sections of Weng's harness article and gives a complete, retrievable 90-second
card for all 39 numbered references and three substantive body-linked sources.

**Architecture:** Canonical source cards live as structured Markdown under
`content/weng-sources/`; a checked TSV matrix owns the 42-source coverage
contract. A standard-library Python builder renders non-authoritative reference
HTML from the canonical cards, while ten short authored lessons follow Weng's
argument. A Rust integration test and the existing release gate enforce
coverage, source promotion, links, assets, accessibility, and generated-byte
parity.

**Tech Stack:** Markdown, TSV, HTML, CSS, browser JavaScript, Python 3 standard
library, Rust integration tests, existing Harp source/corpus verification, and
mise.

---

## Invariant Map

| Boundary | Owner | Invariant |
|---|---|---|
| Technical claims | `content/` | No material paper explanation exists only in teaching HTML. |
| Captured source bytes | `evidence/` | Research tasks read but never normalize captured text. |
| Source identity and ceiling | `content/sources/` | A source is promoted from identity-only only after primary-text inspection and locator capture. |
| Coverage completeness | `content/weng-source-cards.tsv` | Exactly 42 unique cards: 39 numbered references plus three body-linked sources. |
| Teaching projections | `reference/`, `lessons/` | HTML compresses canonical cards and section companions; it is never evidence. |
| Learner state | `learning-records/` | Exposure does not count as retrieval; no learning record is created without demonstrated understanding. |
| Generated teaching bytes | `scripts/build_weng_course.py` | `--check` reproduces checked-in generated references exactly. |
| Release integrity | `mise run verify` | Course verification supplements source, corpus, Atlas, LFS, and repository checks. |

The completion assertions are exact:

- exactly 42 source cards;
- exactly 39 numbered-reference locators;
- exactly 3 body-link locators;
- exactly 9 Weng section IDs;
- exactly 10 lesson files; and
- exactly 4 reference files.

## File Map

### Canonical course content

- `content/weng-source-cards.tsv`: 42-row coverage and lesson-assignment matrix.
- `content/weng-course-status.json`: staged completeness state for incremental
  verification.
- `content/weng-sources/*.md`: one canonical five-field source card per source.
- `content/sources/source_registry.tsv`: promotes the 20 currently
  `vendored-uninspected` numbered references after inspection.
- `content/sources/evidence_graph.tsv`: upgrades identity-only citation notes
  to bounded primary-source claim ownership.

### Teaching state

- `MISSION.md`: approved learner mission and observable success conditions.
- `RESOURCES.md`: curated primary-source routes and explicit remaining gaps.
- `NOTES.md`: teaching preferences, including section-first sequencing and the
  90-second card contract.

### Builder and verifier

- `scripts/build_weng_course.py`: parses canonical cards and deterministically
  writes generated reference HTML.
- `crates/harp/tests/weng_teaching_curriculum.rs`: validates source roster,
  card schema, coverage matrix, source status, lesson inclusion, HTML links,
  shared assets, and non-authoritative placement.
- `mise.toml`: runs the course builder in check mode and the curriculum test in
  the complete release gate.

### Shared teaching assets

- `assets/course.css`: print-friendly shared visual system.
- `assets/quiz.js`: accessible retrieval feedback.
- `assets/coverage.js`: card filtering and progress display.

### Generated reference surfaces

- `reference/weng-harness-map.html`
- `reference/weng-source-cards.html`

### Authored reference surfaces

- `reference/rsi-claim-ladder.html`
- `reference/harness-comparison-matrix.html`

### Lessons

- `lessons/0001-system-being-improved.html`
- `lessons/0002-harness-design-patterns.html`
- `lessons/0003-harness-vs-core-intelligence.html`
- `lessons/0004-context-engineering.html`
- `lessons/0005-workflow-design-and-auto-research.html`
- `lessons/0006-self-improving-harnesses.html`
- `lessons/0007-evolutionary-search.html`
- `lessons/0008-joint-harness-weight-optimization.html`
- `lessons/0009-future-challenges-and-evaluation.html`
- `lessons/0010-synthesis-and-oral-defense.html`

No file is added under `learning-records/` until the learner demonstrates a
non-trivial concept through retrieval.

## Canonical Source Card Contract

Every `content/weng-sources/<slug>.md` uses this exact shape:

```markdown
---
source_id: ACE
title: Agentic Context Engineering: Evolving Contexts for Self-Improving Language Models
weng_locator: reference-7
section_id: context-engineering
primary_url: https://iclr.cc/virtual/2026/poster/10008343
captured_path: evidence/weng/text/ace.txt
publication_state: ICLR 2026 official poster
evidence_state: card-complete
edited_object_family: context-artifact
claim_ceiling: Author-reported mechanism, results, costs, and limitations; no independent reproduction.
lesson_ids: 0004,0010
card_path: content/weng-sources/ace.md
canonical_route: content/systems/ace.md
---

# Agentic Context Engineering

## Problem

Repeated monolithic prompt rewriting can shorten useful context, erase rare
details, and collapse a growing memory into generic advice.

## Core mechanism

ACE maintains an itemized context playbook. A Generator produces trajectories,
a Reflector extracts lessons from successes and failures, and a Curator adds,
merges, refines, or removes identified bullets instead of rewriting one prompt
blob.

## Reported evidence

The authors report improvements across agent and domain-specific benchmarks,
offline prompt optimization, and online memory adaptation, along with lower
rollout cost than selected comparison methods and component ablations in
Section 4 and Appendix A. Harp has not independently reproduced these results.

## Key limitation

ACE improves a persistent context artifact under a handcrafted update
workflow. More context can retain stale or duplicated rules, and the study does
not show that the procedure which learns the playbook becomes a better learner
of future procedures.

## Why Weng cites it

Weng uses ACE as the first step in the context-engineering progression: move
from an ever-growing transcript to a structured context artifact that can
accumulate lessons without full-prompt rewrites.
```

The body headings are required exactly once. Quantitative text names benchmark,
task count, model, metric, and reproduction status when those facts are
material.

Frontmatter values are one-line scalars. `lesson_ids` uses a comma-separated
scalar such as `0001,0002`, not YAML sequence syntax.

Every card includes `0010` because the synthesis lesson samples the complete
card corpus. It also includes each topical lesson that directly teaches it.

`canonical_route` must resolve to a maintained Harp Markdown file. Use a
dedicated system/chapter/deep-dive route when one exists; otherwise use the
source's Weng section companion under `content/weng/`. A card may not point to
itself or use `evidence/` as its canonical route.

## Coverage Matrix Contract

`content/weng-source-cards.tsv` has this header:

```text
source_id	weng_locator	title	section_id	card_path	primary_url	captured_path	evidence_state	claim_ceiling	lesson_ids	retrieval_state
```

Initial `retrieval_state` is `unseen` for all rows. The first implementation
does not mutate retrieval state automatically; later teaching sessions update
it only after evidence of recall.

`content/weng-course-status.json` controls incremental verification:

```json
{
  "schema_version": 1,
  "state": "building",
  "expected_source_cards": 42,
  "expected_lessons": 10,
  "expected_references": 4
}
```

Allowed transitions are:

```text
building -> cards-complete -> complete
```

- `building`: validate every present row/card, allow 0–42 rows, and require the
  matrix to be a subset of the 42-source roster.
- `cards-complete`: require all 42 cards, all source promotions, and generated
  references.
- `complete`: additionally require all 10 lessons and all four reference
  surfaces.

The verifier rejects backward or unknown states semantically by accepting only
these three exact values. Git history records the transition order.

The lesson assignment is:

| Lesson | Source IDs |
|---|---|
| `0001` | `GOOD-1965,YUDKOWSKY-2008,ASP,ABSOLUTE-ZERO,SELF-REWARDING,SPIN,KARPATHY-AUTORESEARCH,ANTHROPIC-RSI` |
| `0002` | `KARPATHY-AUTORESEARCH` |
| `0003` | `HARNESS-DISENTANGLE` |
| `0004` | `ACE,MCE,META-HARNESS` |
| `0005` | `AI-SCIENTIST,SCIENTISTONE,AUTODATA,ADAS,SELF-REFINE,AFLOW` |
| `0006` | `STOP,SELF-HARNESS,HARNESS-DISENTANGLE,AHE,WENG-REWARD` |
| `0007` | `PROMPTBREEDER,GEPA,ALPHAEVOLVE,SHINKAEVOLVE,THETAEVOLVE,DGM,HYPERAGENTS,LEARNING-DISCOVER,EPISTEMIC-DISCOVERY,DEMOEVOLVE` |
| `0008` | `SIA,CONTINUAL-HARNESS` |
| `0009` | `NOT-SCIENTISTS,GPT5-SCIENCE,PAPERBENCH,REBENCH,MLEBENCH,SCIENCEAGENTBENCH,COREBENCH,KERNELBENCH` |
| `0010` | all 42 source IDs, sampled through the source-card reference |

`RLM-PAPER` is linked from Lesson `0003` as a complementary source but does not
receive a Weng source-card row.

Matrix rows encode the union. For example:

```text
ACE -> 0004,0010
KARPATHY-AUTORESEARCH -> 0001,0002,0010
HARNESS-DISENTANGLE -> 0003,0006,0010
```

Use this exact row-assignment map:

```text
GOOD-1965=0001,0010
YUDKOWSKY-2008=0001,0010
ASP=0001,0010
ABSOLUTE-ZERO=0001,0010
SELF-REWARDING=0001,0010
SPIN=0001,0010
ACE=0004,0010
MCE=0004,0010
META-HARNESS=0004,0010
AI-SCIENTIST=0005,0010
SCIENTISTONE=0005,0010
AUTODATA=0005,0010
ADAS=0005,0010
SELF-REFINE=0005,0010
AFLOW=0005,0010
STOP=0006,0010
SELF-HARNESS=0006,0010
PROMPTBREEDER=0007,0010
GEPA=0007,0010
ALPHAEVOLVE=0007,0010
SHINKAEVOLVE=0007,0010
THETAEVOLVE=0007,0010
DGM=0007,0010
HYPERAGENTS=0007,0010
LEARNING-DISCOVER=0007,0010
EPISTEMIC-DISCOVERY=0007,0010
SIA=0008,0010
NOT-SCIENTISTS=0009,0010
GPT5-SCIENCE=0009,0010
PAPERBENCH=0009,0010
REBENCH=0009,0010
MLEBENCH=0009,0010
SCIENCEAGENTBENCH=0009,0010
COREBENCH=0009,0010
KERNELBENCH=0009,0010
HARNESS-DISENTANGLE=0003,0006,0010
AHE=0006,0010
CONTINUAL-HARNESS=0008,0010
DEMOEVOLVE=0007,0010
KARPATHY-AUTORESEARCH=0001,0002,0010
WENG-REWARD=0006,0010
ANTHROPIC-RSI=0001,0010
```

### Task 1: Add the failing curriculum contract

**Files:**

- Create: `crates/harp/tests/weng_teaching_curriculum.rs`

- [ ] **Step 1: Write the source-roster and card-schema test**

Create an integration test using only Harp's existing dependencies with these
constants:

```rust
const NUMBERED_SOURCE_IDS: [&str; 39] = [
    "GOOD-1965",
    "YUDKOWSKY-2008",
    "ASP",
    "ABSOLUTE-ZERO",
    "SELF-REWARDING",
    "SPIN",
    "ACE",
    "MCE",
    "META-HARNESS",
    "AI-SCIENTIST",
    "SCIENTISTONE",
    "AUTODATA",
    "ADAS",
    "SELF-REFINE",
    "AFLOW",
    "STOP",
    "SELF-HARNESS",
    "PROMPTBREEDER",
    "GEPA",
    "ALPHAEVOLVE",
    "SHINKAEVOLVE",
    "THETAEVOLVE",
    "DGM",
    "HYPERAGENTS",
    "LEARNING-DISCOVER",
    "EPISTEMIC-DISCOVERY",
    "SIA",
    "NOT-SCIENTISTS",
    "GPT5-SCIENCE",
    "PAPERBENCH",
    "REBENCH",
    "MLEBENCH",
    "SCIENCEAGENTBENCH",
    "COREBENCH",
    "KERNELBENCH",
    "HARNESS-DISENTANGLE",
    "AHE",
    "CONTINUAL-HARNESS",
    "DEMOEVOLVE",
];

const BODY_LINK_SOURCE_IDS: [&str; 3] = [
    "KARPATHY-AUTORESEARCH",
    "WENG-REWARD",
    "ANTHROPIC-RSI",
];

const REQUIRED_HEADINGS: [&str; 5] = [
    "## Problem",
    "## Core mechanism",
    "## Reported evidence",
    "## Key limitation",
    "## Why Weng cites it",
];
```

The first test must:

1. derive the Weng roster from `content/sources/evidence_graph.tsv`;
2. assert equality with the constants;
3. require `content/weng-source-cards.tsv` and
   `content/weng-course-status.json`;
4. require unique IDs and a matrix roster that is a subset of the 42-source
   constants;
5. resolve every `card_path` and `captured_path`;
6. reject a present numbered row whose source registry status is `MISSING`;
7. require the five headings once per card;
8. require `evidence_state=card-complete`;
9. require one of the nine section IDs; and
10. reject `RLM-PAPER` from the Weng matrix; and
11. resolve every card's `canonical_route`, require it under `content/`, and
    reject a route equal to `card_path`; and
12. require every `lesson_ids` set to contain `0010`.

Parse `content/weng-course-status.json` with `serde_json::Value`, which is
already a Harp dependency. Use these concrete helpers:

```rust
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("Harp workspace root")
        .to_path_buf()
}

fn row_maps(relative: &str) -> Vec<BTreeMap<String, String>> {
    let text = fs::read_to_string(workspace_root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"));
    let mut lines = text.lines();
    let header = lines
        .next()
        .expect("TSV header")
        .split('\t')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    lines
        .filter(|line| !line.is_empty())
        .map(|line| {
            let values = line.split('\t').map(str::to_owned).collect::<Vec<_>>();
            assert_eq!(values.len(), header.len(), "malformed row in {relative}");
            header.iter().cloned().zip(values).collect()
        })
        .collect()
}

fn course_state() -> String {
    let path = workspace_root().join("content/weng-course-status.json");
    let value: Value =
        serde_json::from_slice(&fs::read(path).expect("read course status"))
            .expect("valid course status JSON");
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["expected_source_cards"], 42);
    assert_eq!(value["expected_lessons"], 10);
    assert_eq!(value["expected_references"], 4);
    value["state"]
        .as_str()
        .expect("course state string")
        .to_owned()
}

fn source_registry() -> BTreeMap<String, String> {
    row_maps("content/sources/source_registry.tsv")
        .into_iter()
        .map(|row| (row["source_id"].clone(), row["label"].clone()))
        .collect()
}

fn matrix_rows() -> Vec<BTreeMap<String, String>> {
    row_maps("content/weng-source-cards.tsv")
}

fn weng_roster() -> (BTreeSet<String>, BTreeSet<String>) {
    let rows = row_maps("content/sources/evidence_graph.tsv");
    let numbered = rows
        .iter()
        .filter(|row| {
            row["source_id"] == "WENG-HARNESS" && row["relationship"] == "cites"
        })
        .map(|row| row["target_id"].clone())
        .collect();
    let body = rows
        .iter()
        .filter(|row| {
            row["source_id"] == "WENG-HARNESS"
                && row["relationship"] == "body-links"
        })
        .map(|row| row["target_id"].clone())
        .collect();
    (numbered, body)
}

fn assert_all_sources_promoted(
    rows: &[BTreeMap<String, String>],
    registry: &BTreeMap<String, String>,
) {
    for row in rows {
        assert_ne!(
            registry.get(&row["source_id"]).map(String::as_str),
            Some("MISSING"),
            "{} remains identity-only",
            row["source_id"]
        );
    }
}

fn assert_files_exist(paths: &[&str]) {
    for relative in paths {
        assert!(
            workspace_root().join(relative).is_file(),
            "missing {relative}"
        );
    }
}
```

State semantics:

```rust
match state.as_str() {
    "building" => assert!(rows.len() <= 42),
    "cards-complete" => {
        assert_eq!(rows.len(), 42);
        assert_all_sources_promoted(&rows, &registry);
        assert_files_exist(&REFERENCES);
    }
    "complete" => {
        assert_eq!(rows.len(), 42);
        assert_all_sources_promoted(&rows, &registry);
        assert_files_exist(&REFERENCES);
        assert_files_exist(&LESSONS);
    }
    _ => panic!("unknown Weng course state: {state}"),
}
```

- [ ] **Step 2: Add teaching-surface checks**

In the same test file, add checks that require:

```rust
const LESSONS: [&str; 10] = [
    "lessons/0001-system-being-improved.html",
    "lessons/0002-harness-design-patterns.html",
    "lessons/0003-harness-vs-core-intelligence.html",
    "lessons/0004-context-engineering.html",
    "lessons/0005-workflow-design-and-auto-research.html",
    "lessons/0006-self-improving-harnesses.html",
    "lessons/0007-evolutionary-search.html",
    "lessons/0008-joint-harness-weight-optimization.html",
    "lessons/0009-future-challenges-and-evaluation.html",
    "lessons/0010-synthesis-and-oral-defense.html",
];

const REFERENCES: [&str; 4] = [
    "reference/weng-harness-map.html",
    "reference/weng-source-cards.html",
    "reference/rsi-claim-ladder.html",
    "reference/harness-comparison-matrix.html",
];
```

When `state=complete`, for every lesson:

- require `../assets/course.css`;
- require at least one `<form` or `data-quiz`;
- require `aria-label` or a `<legend>`;
- require a primary-source URL;
- require the phrase `Ask your teacher`;
- for Lessons 1–9, require every source ID assigned to that lesson in the TSV
  to appear in a `#source-<ID>` link;
- for Lesson 10, require links to `weng-source-cards.html` and
  `../assets/coverage.js`; and
- reject inline `<style>` and inline reusable scripts.

When `state` is `cards-complete` or `complete`, for every reference:

- require the shared stylesheet;
- reject absolute local paths; and
- validate local `href` and `src` targets.

- [ ] **Step 3: Run the test and confirm the intended red state**

Run:

```sh
cargo test -p harp --test weng_teaching_curriculum -- --nocapture
```

Expected: failure because `content/weng-source-cards.tsv` does not exist.

- [ ] **Step 4: Keep the red test uncommitted until Task 2 is green**

Do not commit a workspace-wide failing test. Task 2 adds the minimal building
state and commits the test and scaffolding together.

### Task 2: Establish teaching state, builder, and empty generated surfaces

**Files:**

- Create: `MISSION.md`
- Create: `RESOURCES.md`
- Create: `NOTES.md`
- Create: `scripts/build_weng_course.py`
- Create: `assets/course.css`
- Create: `assets/quiz.js`
- Create: `assets/coverage.js`
- Create: `content/weng-source-cards.tsv`
- Create: `content/weng-course-status.json`

- [ ] **Step 1: Write the teaching mission**

Create `MISSION.md`:

```markdown
# Mission: Weng Harness Engineering and Its Research Lineage

## Why

Build enough durable understanding to explain Weng's practical recursive
self-improvement argument, discuss every cited research direction without
hand-waving, and judge whether each work supports harness improvement,
successor improvement, or only a narrower result.

## Success looks like

- Give a coherent 10-minute explanation of Weng's complete argument without notes.
- Give a 90-second explanation of any source in the 42-source matrix.
- Name each work's edited object, reported evidence, and key limitation.
- Explain why Weng cites the work and whether that use is strong, weak, or illustrative.
- Compare adjacent systems without collapsing evaluator, persistence, budget, or claim ceiling.

## Constraints

- Primary-source grounding is mandatory.
- Lessons follow Weng's argument rather than bibliography order.
- Teaching artifacts are non-authoritative projections over `content/`.
- Retrieval practice, not page views, determines progress.

## Out of scope

- Independent reproduction of every benchmark result.
- Full implementation tutorials for all cited systems.
- Treating every self-editing loop as demonstrated recursive improvement.
```

- [ ] **Step 2: Write resource and preference files**

Create `NOTES.md`:

```markdown
# Teaching Notes

- Default to section-first teaching.
- Use the five-field, 90-second source-card contract.
- Keep lessons short and terminal-compatible; HTML is for review and retrieval.
- Do not create learning records for exposure alone.
- Preserve claim ceilings and label author-reported results.
```

Create `RESOURCES.md` with:

- Weng's article as the curriculum anchor;
- `content/weng-reading-map.json` as the section map;
- `content/sources/source_registry.tsv` as the source identity registry;
- `content/sources/evidence_graph.tsv` as the citation topology;
- `evidence/weng/text/` as local primary-text captures;
- `content/rsi_harness_by_lil_log_deconstructed.md` as canonical synthesis;
- the DGM and Meta-Harness packet indexes as deeper examples; and
- a `## Gaps` section listing the 20 identity-only sources by ID before
  research promotion.

- [ ] **Step 3: Create the matrix header**

Create `content/weng-source-cards.tsv` with only the approved header from the
coverage matrix contract. This keeps the contract visible while the research
tasks add rows.

Create `content/weng-course-status.json` with the exact `building` payload from
the staged completeness contract.

- [ ] **Step 4: Implement the deterministic card parser**

Create `scripts/build_weng_course.py` with:

```python
from __future__ import annotations

import argparse
import csv
import html
import json
import re
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / "content/weng-source-cards.tsv"
STATUS = ROOT / "content/weng-course-status.json"
CARD_ROOT = ROOT / "content/weng-sources"
REFERENCE_ROOT = ROOT / "reference"

FIELDS = (
    "source_id",
    "weng_locator",
    "title",
    "section_id",
    "card_path",
    "primary_url",
    "captured_path",
    "evidence_state",
    "claim_ceiling",
    "lesson_ids",
    "retrieval_state",
)
HEADINGS = (
    "Problem",
    "Core mechanism",
    "Reported evidence",
    "Key limitation",
    "Why Weng cites it",
)
SECTION_ORDER = (
    "system-being-improved",
    "harness-design-patterns",
    "harness-layer-vs-core-intelligence",
    "context-engineering",
    "workflow-design-and-search",
    "self-improving-harnesses",
    "evolutionary-search",
    "joint-harness-weight-optimization",
    "future-challenges",
)
SECTION_META = (
    (
        "system-being-improved",
        "What system is being improved?",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#case-study-coding-agent-harness",
        "../content/weng/01-system-being-improved.md",
        "../lessons/0001-system-being-improved.html",
    ),
    (
        "harness-design-patterns",
        "Harness design patterns",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#harness-design-patterns",
        "../content/weng/02-harness-design-patterns.md",
        "../lessons/0002-harness-design-patterns.html",
    ),
    (
        "harness-layer-vs-core-intelligence",
        "Harness layer versus core intelligence",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#harness-layer-vs-core-intelligence",
        "../content/weng/03-harness-layer-vs-core-intelligence.md",
        "../lessons/0003-harness-vs-core-intelligence.html",
    ),
    (
        "context-engineering",
        "Context engineering",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#context-engineering",
        "../content/weng/04-context-engineering.md",
        "../lessons/0004-context-engineering.html",
    ),
    (
        "workflow-design-and-search",
        "Workflow design and search",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#workflow-design",
        "../content/weng/05-workflow-design-and-search.md",
        "../lessons/0005-workflow-design-and-auto-research.html",
    ),
    (
        "self-improving-harnesses",
        "Self-improving harnesses",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#self-improving-harness",
        "../content/weng/06-self-improving-harnesses.md",
        "../lessons/0006-self-improving-harnesses.html",
    ),
    (
        "evolutionary-search",
        "Evolutionary search",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#evolutionary-search",
        "../content/weng/07-evolutionary-search.md",
        "../lessons/0007-evolutionary-search.html",
    ),
    (
        "joint-harness-weight-optimization",
        "Joint harness and weight optimization",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#joint-optimization-with-model-weights",
        "../content/weng/08-joint-harness-weight-optimization.md",
        "../lessons/0008-joint-harness-weight-optimization.html",
    ),
    (
        "future-challenges",
        "Future challenges",
        "https://lilianweng.github.io/posts/2026-07-04-harness/#future-challenges",
        "../content/weng/09-future-challenges.md",
        "../lessons/0009-future-challenges-and-evaluation.html",
    ),
)
CARD_METADATA = (
    "source_id",
    "title",
    "weng_locator",
    "section_id",
    "primary_url",
    "captured_path",
    "publication_state",
    "evidence_state",
    "edited_object_family",
    "claim_ceiling",
    "lesson_ids",
    "card_path",
    "canonical_route",
)
ALLOWED_STATES = {"building", "cards-complete", "complete"}


@dataclass(frozen=True)
class Card:
    metadata: dict[str, str]
    sections: dict[str, str]


def parse_frontmatter(text: str) -> tuple[dict[str, str], str]:
    if not text.startswith("---\n"):
        raise ValueError("card must start with frontmatter")
    frontmatter, body = text[4:].split("\n---\n", 1)
    metadata: dict[str, str] = {}
    for line in frontmatter.splitlines():
        key, value = line.split(":", 1)
        key = key.strip()
        if key in metadata:
            raise ValueError(f"duplicate frontmatter key: {key}")
        metadata[key] = value.strip()
    if tuple(metadata) != CARD_METADATA:
        raise ValueError(f"unexpected card metadata: {tuple(metadata)}")
    return metadata, body


def parse_sections(body: str) -> dict[str, str]:
    sections: dict[str, str] = {}
    current: str | None = None
    lines: list[str] = []
    for line in body.splitlines():
        if line.startswith("## "):
            if current is not None:
                sections[current] = "\n".join(lines).strip()
            current = line[3:].strip()
            lines = []
        elif current is not None:
            lines.append(line)
    if current is not None:
        sections[current] = "\n".join(lines).strip()
    if tuple(sections) != HEADINGS:
        raise ValueError(f"unexpected card sections: {tuple(sections)}")
    return sections


def load_cards() -> list[Card]:
    with MATRIX.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != FIELDS:
            raise ValueError("unexpected matrix header")
        rows = list(reader)
    if len({row["source_id"] for row in rows}) != len(rows):
        raise ValueError("duplicate source ID")
    if any(not all(row[field].strip() for field in FIELDS) for row in rows):
        raise ValueError("matrix row contains an empty field")
    if any(
        not re.fullmatch(r"\d{4}(,\d{4})*", row["lesson_ids"])
        for row in rows
    ):
        raise ValueError("invalid lesson_ids")
    cards = []
    for row in rows:
        path = ROOT / row["card_path"]
        metadata, body = parse_frontmatter(path.read_text(encoding="utf-8"))
        if metadata["source_id"] != row["source_id"]:
            raise ValueError(f"source ID mismatch for {path}")
        if metadata["card_path"] != row["card_path"]:
            raise ValueError(f"card path mismatch for {path}")
        for field in (
            "title",
            "weng_locator",
            "section_id",
            "primary_url",
            "captured_path",
            "evidence_state",
            "claim_ceiling",
            "lesson_ids",
        ):
            if metadata[field] != row[field]:
                raise ValueError(f"{field} mismatch for {path}")
        if metadata["section_id"] not in SECTION_ORDER:
            raise ValueError(f"unknown Weng section for {path}")
        if not metadata["primary_url"].startswith(("https://", "http://")):
            raise ValueError(f"invalid primary URL for {path}")
        captured = ROOT / metadata["captured_path"]
        if (
            not metadata["captured_path"].startswith("evidence/")
            or not captured.is_file()
        ):
            raise ValueError(f"invalid captured path for {path}")
        canonical = ROOT / metadata["canonical_route"]
        if (
            not metadata["canonical_route"].startswith("content/")
            or metadata["canonical_route"] == metadata["card_path"]
            or not canonical.is_file()
        ):
            raise ValueError(f"invalid canonical route for {path}")
        cards.append(Card(metadata, parse_sections(body)))
    locator_order = {
        **{f"reference-{index}": index for index in range(1, 40)},
        "body-link-workflow-automation": 40,
        "body-link-reward-hacking": 41,
        "body-link-ai-progress": 42,
    }
    return sorted(
        cards,
        key=lambda card: (
            locator_order[card.metadata["weng_locator"]],
            card.metadata["source_id"],
        ),
    )


def load_status() -> str:
    data = json.loads(STATUS.read_text(encoding="utf-8"))
    if data != {
        "schema_version": 1,
        "state": data.get("state"),
        "expected_source_cards": 42,
        "expected_lessons": 10,
        "expected_references": 4,
    }:
        raise ValueError("unexpected Weng course status contract")
    state = data["state"]
    if state not in ALLOWED_STATES:
        raise ValueError(f"unknown Weng course state: {state}")
    return state
```

The complete rendering API is:

```python
def paragraphs(text: str) -> str:
    blocks = [block.strip() for block in text.split("\n\n") if block.strip()]
    return "".join(f"<p>{html.escape(block)}</p>" for block in blocks)


def render_card(card: Card) -> str:
    metadata = card.metadata
    source_id = metadata["source_id"]
    fields = "".join(
        (
            f'<section class="card-field" aria-labelledby="{source_id}-{heading.lower().replace(" ", "-")}">'
            f'<h3 id="{source_id}-{heading.lower().replace(" ", "-")}">{html.escape(heading)}</h3>'
            f"{paragraphs(card.sections[heading])}</section>"
        )
        for heading in HEADINGS
    )
    return (
        f'<article id="source-{html.escape(source_id)}" data-source-card '
        f'data-source-id="{html.escape(source_id)}" '
        f'data-section="{html.escape(metadata["section_id"])}" '
        f'data-family="{html.escape(metadata["edited_object_family"])}">'
        f'<p class="source-id">{html.escape(source_id)}</p>'
        f'<h2>{html.escape(metadata["title"])}</h2>'
        f"{fields}"
        f'<p><a href="{html.escape(metadata["primary_url"])}">Primary source</a></p>'
        f'<p><a href="../{html.escape(metadata["canonical_route"])}">Canonical Harp route</a></p>'
        "</article>"
    )


def render_source_cards(cards: list[Card]) -> str:
    card_html = "".join(render_card(card) for card in cards)
    return (
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">"
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
        "<title>Weng Source Cards</title>"
        "<link rel=\"stylesheet\" href=\"../assets/course.css\"></head>"
        "<body><main><h1>Weng Source Cards</h1>"
        '<form aria-label="Filter source cards"><label for="source-query">Filter</label>'
        '<input id="source-query" data-source-query type="search"></form>'
        f'<p data-result-count>{len(cards)} sources</p><div data-card-list>{card_html}</div>'
        "</main><script src=\"../assets/coverage.js\"></script></body></html>\n"
    )


def render_harness_map(cards: list[Card]) -> str:
    section_counts: dict[str, int] = {}
    for card in cards:
        section = card.metadata["section_id"]
        section_counts[section] = section_counts.get(section, 0) + 1
    sections = "".join(
        f'<li data-section="{html.escape(section)}">'
        f'<strong>{html.escape(title)}</strong>: {count} sources · '
        f'<a href="{html.escape(original)}">Original</a> · '
        f'<a href="{html.escape(companion)}">Companion</a> · '
        f'<a href="{html.escape(lesson)}">Lesson</a></li>'
        for section, title, original, companion, lesson in SECTION_META
        for count in (section_counts.get(section, 0),)
    )
    return (
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">"
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
        "<title>Weng Harness Map</title>"
        "<link rel=\"stylesheet\" href=\"../assets/course.css\"></head>"
        "<body><main><h1>Weng Harness Map</h1>"
        "<p>prompts → structured context → workflow → harness code → optimizer code</p>"
        f"<ol>{sections}</ol></main></body></html>\n"
    )


def write_or_check(path: Path, content: str, check: bool) -> None:
    encoded = content.encode("utf-8")
    if check:
        if not path.is_file() or path.read_bytes() != encoded:
            raise SystemExit(f"generated teaching reference is stale: {path.relative_to(ROOT)}")
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(encoded)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    state = load_status()
    cards = load_cards()
    if state not in {"cards-complete", "complete"}:
        raise SystemExit("teaching references require cards-complete state")
    if len(cards) != 42:
        raise SystemExit(f"expected 42 source cards, found {len(cards)}")
    write_or_check(
        REFERENCE_ROOT / "weng-source-cards.html",
        render_source_cards(cards),
        args.check,
    )
    write_or_check(
        REFERENCE_ROOT / "weng-harness-map.html",
        render_harness_map(cards),
        args.check,
    )


if __name__ == "__main__":
    main()
```

Before rendering, `main()` must:

- sort by Weng locator and source ID;
- read `content/weng-course-status.json`;
- reject generation unless state is `cards-complete` or `complete`; and
- require `len(cards) == 42` before writing references.

- [ ] **Step 5: Add shared assets**

Create `assets/course.css`:

```css
:root {
  color-scheme: light;
  --ink: #18201f;
  --muted: #5c6865;
  --paper: #f8f6ef;
  --panel: #ffffff;
  --line: #cfd7d2;
  --accent: #0b6b5d;
  --evidence: #e5f3ee;
  --limit: #fff0dc;
  font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  color: var(--ink);
  background: var(--paper);
  line-height: 1.6;
}

main {
  width: min(72ch, calc(100% - 2rem));
  margin: 0 auto;
  padding: 2rem 0 4rem;
}

h1,
h2,
h3 {
  line-height: 1.2;
}

p,
li {
  font-family: Georgia, "Times New Roman", serif;
}

a {
  color: var(--accent);
}

:focus-visible {
  outline: 3px solid var(--accent);
  outline-offset: 3px;
}

nav,
form,
[data-source-card],
.reference-panel {
  border: 1px solid var(--line);
  border-radius: 0.75rem;
  background: var(--panel);
  padding: 1rem;
}

[data-card-list] {
  display: grid;
  gap: 1rem;
}

.card-field[aria-labelledby$="-reported-evidence"] {
  background: var(--evidence);
}

.card-field[aria-labelledby$="-key-limitation"] {
  background: var(--limit);
}

.card-field {
  border-radius: 0.5rem;
  padding: 0.25rem 0.75rem;
  margin-block: 0.75rem;
}

.eyebrow,
.source-id {
  color: var(--muted);
  font: 600 0.8rem/1.4 system-ui, sans-serif;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

@media print {
  nav,
  form,
  script {
    display: none;
  }

  body {
    background: #fff;
  }

  [data-source-card] {
    break-inside: avoid;
    border-color: #777;
  }
}
```

Create `assets/quiz.js`:

```javascript
(() => {
  const attach = (form) => {
    const feedback = form.querySelector("[data-feedback]");
    if (!(feedback instanceof HTMLElement)) {
      throw new Error("quiz form is missing [data-feedback]");
    }
    feedback.setAttribute("aria-live", "polite");
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      const answer = new FormData(form).get("answer");
      const correct = form.dataset.answer;
      feedback.textContent =
        answer === correct
          ? "Correct. Explain why the other choice is weaker."
          : "Not yet. Re-read the edited-object and evidence boundaries.";
    });
  };

  const init = () => {
    document.querySelectorAll("[data-quiz]").forEach((form) => {
      if (form instanceof HTMLFormElement) {
        attach(form);
      }
    });
  };

  window.HarpQuiz = { init };
  init();
})();
```

Create `assets/coverage.js`:

```javascript
(() => {
  const normalize = (value) => value.trim().toLowerCase();

  const init = () => {
    const query = document.querySelector("[data-source-query]");
    const count = document.querySelector("[data-result-count]");
    const cards = [...document.querySelectorAll("[data-source-card]")];
    if (!(query instanceof HTMLInputElement) || !(count instanceof HTMLElement)) {
      return;
    }

    const apply = () => {
      const needle = normalize(query.value);
      let visible = 0;
      cards.forEach((card) => {
        if (!(card instanceof HTMLElement)) {
          return;
        }
        const haystack = normalize(
          `${card.dataset.sourceId ?? ""} ${card.dataset.section ?? ""} ` +
            `${card.dataset.family ?? ""} ${card.textContent ?? ""}`,
        );
        const show = needle === "" || haystack.includes(needle);
        card.hidden = !show;
        visible += Number(show);
      });
      count.textContent = `${visible} of ${cards.length} sources`;
    };

    query.addEventListener("input", apply);
    apply();
  };

  init();
})();
```

- [ ] **Step 6: Run the contract**

Run:

```sh
cargo test -p harp --test weng_teaching_curriculum -- --nocapture
```

Expected: the Rust test passes in `building` state with zero valid rows. Before
the 42-card matrix is complete, run the parser as:

```sh
python3 - <<'PY'
from scripts.build_weng_course import load_cards
assert load_cards() == []
PY
```

Do not create or commit empty generated references.

- [ ] **Step 7: Commit the green contract and scaffolding**

```sh
git add crates/harp/tests/weng_teaching_curriculum.rs MISSION.md RESOURCES.md \
  NOTES.md content/weng-source-cards.tsv content/weng-course-status.json \
  scripts/build_weng_course.py assets
git commit -m "feat: scaffold Weng teaching curriculum" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

### Task 3: Research historical RSI and model self-improvement sources

**Files:**

- Create:
  - `content/weng-sources/good-1965.md`
  - `content/weng-sources/yudkowsky-2008.md`
  - `content/weng-sources/asp.md`
  - `content/weng-sources/absolute-zero.md`
  - `content/weng-sources/self-rewarding.md`
  - `content/weng-sources/spin.md`
- Modify:
  - `content/weng-source-cards.tsv`
  - `content/sources/source_registry.tsv`
  - `content/sources/evidence_graph.tsv`
  - `RESOURCES.md`

- [ ] **Step 1: Inspect the six captured primary texts**

Read:

```text
evidence/weng/text/good-1965.txt
evidence/weng/text/yudkowsky-2008.txt
evidence/weng/text/asp.txt
evidence/weng/text/absolute-zero.txt
evidence/weng/text/self-rewarding.txt
evidence/weng/text/spin.txt
```

For each source, record section/page locators for the problem, mechanism,
reported evidence, and limitation before drafting.

- [ ] **Step 2: Write six canonical cards**

Use the exact card contract. Assign all six to
`section_id=system-being-improved` and `lesson_ids=0001,0010`.

Use these edited-object families:

| Source | Family |
|---|---|
| `GOOD-1965` | `historical-rsi-framing` |
| `YUDKOWSKY-2008` | `historical-rsi-framing` |
| `ASP` | `model-self-play-or-weight-adaptation` |
| `ABSOLUTE-ZERO` | `model-self-play-or-weight-adaptation` |
| `SELF-REWARDING` | `model-self-play-or-weight-adaptation` |
| `SPIN` | `model-self-play-or-weight-adaptation` |

The historical cards may report conceptual arguments rather than experiments;
their `Reported evidence` section states that boundary explicitly.

- [ ] **Step 3: Promote registry and citation edges**

For each of the six registry rows:

- change status from `MISSING` to `EVIDENCE`;
- change representation from `vendored-uninspected` to
  `vendored-inspected`;
- replace `Identity only` with the bounded inspected claim ceiling; and
- preserve URL, revision/digest, publication state, and capture date.

For each Weng citation edge, replace `Identity edge only` with a note naming
what the primary source owns. Keep relationship and reference number unchanged.

- [ ] **Step 4: Append six exact matrix rows**

Use `reference-1` through `reference-6`, the source registry title and URL,
the local capture, `card-complete`, lesson `0001`, and retrieval state `unseen`.

- [ ] **Step 5: Run focused verification**

```sh
cargo run -p harp -- check
cargo test -p harp --test weng_teaching_curriculum -- --nocapture
```

Expected: curriculum test reports 6 of 42 cards and the next missing source ID.

- [ ] **Step 6: Commit the research batch**

```sh
git add content/weng-sources content/weng-source-cards.tsv \
  content/sources/source_registry.tsv content/sources/evidence_graph.tsv \
  RESOURCES.md
git commit -m "docs: interpret Weng RSI foundation sources" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

### Task 4: Research workflow and prompt-evolution sources

**Files:**

- Create:
  - `content/weng-sources/scientistone.md`
  - `content/weng-sources/autodata.md`
  - `content/weng-sources/promptbreeder.md`
  - `content/weng-sources/gepa.md`
  - `content/weng-sources/shinkaevolve.md`
  - `content/weng-sources/thetaevolve.md`
- Modify:
  - `content/weng-source-cards.tsv`
  - `content/sources/source_registry.tsv`
  - `content/sources/evidence_graph.tsv`
  - `RESOURCES.md`

- [ ] **Step 1: Inspect the six captures**

Read:

```text
evidence/weng/text/scientistone.txt
evidence/weng/text/autodata.txt
evidence/weng/text/promptbreeder.txt
evidence/weng/text/gepa.txt
evidence/weng/text/shinkaevolve.txt
evidence/weng/text/thetaevolve.txt
```

- [ ] **Step 2: Write cards and assignments**

| Source | Section | Lesson | Family |
|---|---|---|---|
| `SCIENTISTONE` | `workflow-design-and-search` | `0005` | `automated-research-system` |
| `AUTODATA` | `workflow-design-and-search` | `0005` | `automated-research-system` |
| `PROMPTBREEDER` | `evolutionary-search` | `0007` | `evolutionary-program-or-population-search` |
| `GEPA` | `evolutionary-search` | `0007` | `evolutionary-program-or-population-search` |
| `SHINKAEVOLVE` | `evolutionary-search` | `0007` | `evolutionary-program-or-population-search` |
| `THETAEVOLVE` | `evolutionary-search` | `0007` | `evolutionary-program-or-population-search` |

The cards must distinguish prompt evolution, workflow evolution, program
evolution, and test-time learning rather than calling all four “agent
improvement.”

- [ ] **Step 3: Promote six registry rows and edges**

Apply the same status/representation/claim-ceiling promotion contract from
Task 3 without changing source identity.

- [ ] **Step 4: Append matrix rows**

Use reference locators `reference-11`, `reference-12`, `reference-18`,
`reference-19`, `reference-21`, and `reference-22`.

- [ ] **Step 5: Verify and commit**

```sh
cargo run -p harp -- check
cargo test -p harp --test weng_teaching_curriculum -- --nocapture
git add content/weng-sources content/weng-source-cards.tsv \
  content/sources/source_registry.tsv content/sources/evidence_graph.tsv \
  RESOURCES.md
git commit -m "docs: interpret Weng workflow evolution sources" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

Expected curriculum count after commit: 12 of 42 cards.

### Task 5: Research discovery and demonstration-guided evolution

**Files:**

- Create:
  - `content/weng-sources/hyperagents.md`
  - `content/weng-sources/learning-discover.md`
  - `content/weng-sources/epistemic-discovery.md`
  - `content/weng-sources/demoevolve.md`
- Modify:
  - `content/weng-source-cards.tsv`
  - `content/sources/source_registry.tsv`
  - `content/sources/evidence_graph.tsv`
  - `RESOURCES.md`

- [ ] **Step 1: Inspect four captures**

Read:

```text
evidence/weng/text/hyperagents.txt
evidence/weng/text/learning-discover.txt
evidence/weng/text/epistemic-discovery.txt
evidence/weng/text/demoevolve.txt
```

- [ ] **Step 2: Write four cards**

All four use `section_id=evolutionary-search`, `lesson_ids=0007,0010`.

Use:

| Source | Family |
|---|---|
| `HYPERAGENTS` | `harness-code-search` |
| `LEARNING-DISCOVER` | `evolutionary-program-or-population-search` |
| `EPISTEMIC-DISCOVERY` | `evolutionary-program-or-population-search` |
| `DEMOEVOLVE` | `evolutionary-program-or-population-search` |

The reported-evidence field must state whether the evaluation targets agent
quality, problem discovery, uncertainty, or sparse-feedback improvement.

- [ ] **Step 3: Promote registry rows and citation edges**

Promote references 24, 25, 26, and 39 under the same bounded evidence rules.

- [ ] **Step 4: Append matrix rows, verify, and commit**

```sh
cargo run -p harp -- check
cargo test -p harp --test weng_teaching_curriculum -- --nocapture
git add content/weng-sources content/weng-source-cards.tsv \
  content/sources/source_registry.tsv content/sources/evidence_graph.tsv \
  RESOURCES.md
git commit -m "docs: interpret Weng discovery search sources" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

Expected curriculum count after commit: 16 of 42 cards.

### Task 6: Research future-challenge benchmark sources

**Files:**

- Create:
  - `content/weng-sources/gpt5-science.md`
  - `content/weng-sources/mlebench.md`
  - `content/weng-sources/scienceagentbench.md`
  - `content/weng-sources/corebench.md`
- Modify:
  - `content/weng-source-cards.tsv`
  - `content/sources/source_registry.tsv`
  - `content/sources/evidence_graph.tsv`
  - `RESOURCES.md`

- [ ] **Step 1: Inspect four captures**

Read:

```text
evidence/weng/text/gpt5-science.txt
evidence/weng/text/mlebench.txt
evidence/weng/text/scienceagentbench.txt
evidence/weng/text/corebench.txt
```

- [ ] **Step 2: Write cards with explicit benchmark accounting**

All cards use `section_id=future-challenges`, `lesson_ids=0009,0010`.

Use:

| Source | Family |
|---|---|
| `GPT5-SCIENCE` | `automated-research-system` |
| `MLEBENCH` | `evaluation-benchmark` |
| `SCIENCEAGENTBENCH` | `evaluation-benchmark` |
| `COREBENCH` | `evaluation-benchmark` |

For each benchmark, `Reported evidence` states task count, source domain,
metric, compared agent/model, and whether the result comes from the abstract,
main paper, or appendix. For GPT-5 Science, preserve the company-report
evidence class.

- [ ] **Step 3: Promote references 29 and 32–34**

Update source rows and evidence edges without changing publication identity.

- [ ] **Step 4: Append matrix rows, verify, and commit**

```sh
cargo run -p harp -- check
cargo test -p harp --test weng_teaching_curriculum -- --nocapture
git add content/weng-sources content/weng-source-cards.tsv \
  content/sources/source_registry.tsv content/sources/evidence_graph.tsv \
  RESOURCES.md
git commit -m "docs: interpret Weng research benchmark sources" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

Expected curriculum count after commit: 20 of 42 cards and zero identity-only
numbered references.

### Task 7: Normalize the 22 already inspected and body-linked sources

**Files:**

- Create 22 cards under `content/weng-sources/`:
  - `ace.md`
  - `mce.md`
  - `meta-harness.md`
  - `ai-scientist.md`
  - `adas.md`
  - `self-refine.md`
  - `aflow.md`
  - `stop.md`
  - `self-harness.md`
  - `alphaevolve.md`
  - `dgm.md`
  - `sia.md`
  - `not-scientists.md`
  - `paperbench.md`
  - `rebench.md`
  - `kernelbench.md`
  - `harness-disentangle.md`
  - `ahe.md`
  - `continual-harness.md`
  - `karpathy-autoresearch.md`
  - `weng-reward.md`
  - `anthropic-rsi.md`
- Modify:
  - `content/weng-source-cards.tsv`
  - `content/sources/source_registry.tsv`
  - `RESOURCES.md`

- [ ] **Step 1: Reuse canonical claims, not prior summaries**

For each source:

1. inspect its current source-registry ceiling;
2. read the canonical system/chapter route when present;
3. verify the cited primary locator in local captured text;
4. compress into the five-field card; and
5. preserve non-reproduction language.

Do not promote a claim beyond the existing inspected source ceiling.

- [ ] **Step 2: Write the 22 cards**

Use the lesson assignment table in this plan. Body-linked locators are:

```text
body-link-workflow-automation
body-link-reward-hacking
body-link-ai-progress
```

`ANTHROPIC-RSI` is currently identity-only. Inspect
`evidence/weng/text/anthropic-rsi.txt`, promote its source row to bounded
company-essay evidence, and state that it owns framing rather than experimental
proof.

- [ ] **Step 3: Complete the 42-row matrix**

Sort rows by:

1. numbered reference order 1–39;
2. body-linked sources in the order Autoresearch, Weng Reward, Anthropic RSI.

Assert:

```sh
awk -F '\t' 'NR>1 {n++; ids[$1]++} END {
  if (n != 42) exit 1;
  for (id in ids) if (ids[id] != 1) exit 1
}' content/weng-source-cards.tsv
```

Keep `content/weng-course-status.json` in `building` state. A complete 42-row
card matrix is valid during `building`; Task 8 advances the state only when all
reference surfaces are ready.

The future `cards-complete` payload used by Task 8 is:

```json
{
  "schema_version": 1,
  "state": "cards-complete",
  "expected_source_cards": 42,
  "expected_lessons": 10,
  "expected_references": 4
}
```

- [ ] **Step 4: Run the curriculum contract**

```sh
cargo test -p harp --test weng_teaching_curriculum -- --nocapture
```

Expected: card and source-promotion checks pass; lesson/reference checks still
fail because teaching HTML does not exist.

- [ ] **Step 5: Commit complete canonical cards**

```sh
git add content/weng-sources content/weng-source-cards.tsv \
  content/sources/source_registry.tsv RESOURCES.md
git commit -m "docs: complete Weng source card corpus" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

### Task 8: Generate reference surfaces

**Files:**

- Create:
  - `reference/weng-harness-map.html`
  - `reference/weng-source-cards.html`
  - `reference/rsi-claim-ladder.html`
  - `reference/harness-comparison-matrix.html`
- Modify:
  - `scripts/build_weng_course.py`

- [ ] **Step 1: Complete deterministic reference generation**

Change `content/weng-course-status.json` from `building` to `cards-complete`
before invoking the builder.

`render_source_cards` must include:

- one `<article data-source-card>` per matrix row;
- source ID, locator, section, family, and evidence state as data attributes;
- all five card fields;
- primary-source and canonical-route links;
- filter controls with labels;
- a visible 42-card result count; and
- a generated provenance notice.

`render_harness_map` must include:

- all nine Weng sections in order;
- the optimization ladder;
- card counts by section;
- links to the original article anchors; and
- links to the matching section companion and lesson.

- [ ] **Step 2: Build generated references**

Run:

```sh
python3 scripts/build_weng_course.py
python3 scripts/build_weng_course.py --check
```

Expected: both generated files are byte-current.

- [ ] **Step 3: Author the claim ladder**

`reference/rsi-claim-ladder.html` defines:

```text
task iteration
-> persistent adaptation
-> harness improvement
-> successor improvement
-> demonstrated recursive improvement
```

Each level includes:

- required evidence;
- a positive example;
- a common overclaim; and
- links to canonical Harp concepts.

- [ ] **Step 4: Author the comparison matrix**

`reference/harness-comparison-matrix.html` compares all 42 cards with compact
columns:

- edited object;
- persistence;
- evaluator;
- update operator;
- model-weight change;
- evidence class; and
- claim ceiling.

The matrix may abbreviate cells but links every row to the full generated card.

- [ ] **Step 5: Verify and commit**

```sh
python3 scripts/build_weng_course.py --check
cargo test -p harp --test weng_teaching_curriculum -- --nocapture
git add scripts/build_weng_course.py reference assets \
  content/weng-course-status.json
git commit -m "feat: add Weng course reference surfaces" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

Expected: reference checks pass; lesson checks remain red.

### Task 9: Author the ten section-first lessons

**Files:**

- Create all ten `lessons/*.html` files from the file map.

- [ ] **Step 1: Use one shared lesson shell**

Lesson 1 uses this complete shell. Lessons 2–10 preserve the same element IDs,
navigation, asset links, quiz structure, feedback node, primary-source block,
and teacher invitation while replacing the lesson-specific prose and card
links defined in Steps 2–5.

```html
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>What System Is Being Improved? · Weng Harness Curriculum</title>
  <link rel="stylesheet" href="../assets/course.css">
</head>
<body>
  <main>
    <nav aria-label="Course navigation">
      <a href="../reference/weng-harness-map.html">Course map</a>
      <a href="0002-harness-design-patterns.html">Next lesson</a>
    </nav>
    <p class="eyebrow">Weng Harness Curriculum · Lesson 1</p>
    <h1>What system is being improved?</h1>
    <section aria-labelledby="win">
      <h2 id="win">Learner win</h2>
      <p>Separate a model update, a training-pipeline change, and a harness
      change, then state what additional evidence would make the change
      recursive.</p>
    </section>
    <section aria-labelledby="explanation">
      <h2 id="explanation">The expanded system boundary</h2>
      <p>Weng treats the deployed system as more than model weights. The
      harness chooses context, tools, memory, control flow, verification, and
      persistence. A training pipeline produces future weights; a deployment
      harness governs how current weights observe and act.</p>
      <p>Changing either component can improve task outcomes. Recursive
      improvement requires the accepted successor to become better at
      producing later valid successors, not merely to score higher once.</p>
    </section>
    <section aria-labelledby="sources">
      <h2 id="sources">Source cards</h2>
      <ul>
        <li><a href="../reference/weng-source-cards.html#source-GOOD-1965">Good 1965</a></li>
        <li><a href="../reference/weng-source-cards.html#source-YUDKOWSKY-2008">Yudkowsky 2008</a></li>
        <li><a href="../reference/weng-source-cards.html#source-ABSOLUTE-ZERO">Absolute Zero</a></li>
        <li><a href="../reference/weng-source-cards.html#source-KARPATHY-AUTORESEARCH">Karpathy Autoresearch</a></li>
      </ul>
      <p><a href="https://lilianweng.github.io/posts/2026-07-04-harness/">
      Read the primary curriculum anchor: Weng's original article.</a></p>
    </section>
    <form data-quiz data-answer="harness" aria-labelledby="quiz-title">
      <fieldset>
        <legend id="quiz-title">Which change leaves model weights fixed?</legend>
        <label><input type="radio" name="answer" value="weights"> Fine-tune the model on new trajectories.</label>
        <label><input type="radio" name="answer" value="harness"> Add a typed test-and-retry tool loop.</label>
        <button type="submit">Check answer</button>
        <p data-feedback></p>
      </fieldset>
    </form>
    <section aria-labelledby="compare">
      <h2 id="compare">Compare</h2>
      <p>Absolute Zero changes model behavior through training. Autoresearch
      changes an external experiment workflow around a fixed model. Explain
      why both are relevant to RSI but neither result alone proves a
      self-accelerating successor loop.</p>
    </section>
    <section aria-labelledby="next">
      <h2 id="next">Continue</h2>
      <p>Ask your teacher to challenge your model/training/harness boundary
      before continuing to harness design patterns.</p>
    </section>
  </main>
  <script src="../assets/quiz.js"></script>
</body>
</html>
```

The lesson links source cards by fragment:

```text
../reference/weng-source-cards.html#source-ACE
```

- [ ] **Step 2: Author Lessons 1–3**

Lesson 1:

- explains the system boundary;
- introduces references 1–6 and both body-linked framing sources;
- retrieval: classify model, training-pipeline, and harness changes; and
- comparison: Absolute Zero versus Autoresearch.

Lesson 2:

- teaches loops, files, and subagents as substrates;
- retrieval: classify task iteration versus persistent adaptation; and
- comparison: transient transcript versus durable artifact.

Lesson 3:

- teaches external procedure and internalization;
- uses Harness Disentangle plus RLM as a complementary boundary;
- retrieval: activation versus adherence versus outcome; and
- comparison: fixed weights with two harnesses.

- [ ] **Step 3: Author Lessons 4–6**

Lesson 4:

- compares ACE, MCE, and Meta-Harness by edited object.

Lesson 5:

- compares expert workflow, meta-agent search, and MCTS workflow search.

Lesson 6:

- compares STOP, Self-Harness, and AHE;
- emphasizes evaluator integrity and external promotion.

Each lesson includes one source-card recall prompt and one adjacent-system
discrimination prompt.

- [ ] **Step 4: Author Lessons 7–9**

Lesson 7:

- organizes ten evolutionary-search sources by population, mutation, archive,
  and evaluator;
- distinguishes prompt, program, harness, and discovery search.

Lesson 8:

- compares SIA and Continual Harness;
- includes the four-cell `W_old/H_old` crossed evaluation.

Lesson 9:

- compares eight research/evaluation sources;
- asks which benchmark measures replication, engineering, data-driven
  discovery, or kernel generation; and
- includes reward-hacking and long-term-maintainability questions.

- [ ] **Step 5: Author Lesson 10**

Lesson 10 contains:

- a blank argument-map retrieval canvas;
- a 42-card randomizer using `coverage.js`;
- five oral-defense prompts;
- a rubric that separates accuracy, scope, and evidence calibration; and
- no claim that completion equals durable learning.

- [ ] **Step 6: Verify all lesson contracts**

Change `content/weng-course-status.json` from `cards-complete` to `complete`,
then run:

```sh
cargo test -p harp --test weng_teaching_curriculum -- --nocapture
```

Expected: all curriculum tests pass.

- [ ] **Step 7: Commit lessons**

```sh
git add lessons content/weng-course-status.json
git commit -m "feat: add Weng harness teaching lessons" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

### Task 10: Integrate release verification and complete the audit

**Files:**

- Modify as required:
  - `docs/product-contract.md`
  - `docs/import-receipt.md`
  - generated Atlas artifacts only if canonical content changes alter them

- [ ] **Step 1: Update product documentation**

Document:

- canonical card root under `content/weng-sources/`;
- teaching projections under `reference/` and `lessons/`;
- 42-card completeness contract;
- course builder check in the release gate; and
- teaching files remaining outside Atlas compilation.

Add to `verify-rust` after `cargo run -p harp -- sources verify`:

```sh
python3 scripts/build_weng_course.py --check
```

Do not add a network fetch.

- [ ] **Step 2: Run focused verification**

```sh
cargo test -p harp --test weng_teaching_curriculum -- --nocapture
python3 scripts/build_weng_course.py --check
cargo run -p harp -- check
cargo run -p harp -- sources verify
```

Expected: all commands pass, 42 cards are present, and no numbered Weng source
has `MISSING` status.

- [ ] **Step 3: Run a prompt-to-artifact audit**

Run:

```sh
python3 - <<'PY'
from pathlib import Path
import csv

with Path("content/weng-source-cards.tsv").open(newline="", encoding="utf-8") as handle:
    rows = list(csv.DictReader(handle, delimiter="\t"))

assert len(rows) == 42
assert len({row["source_id"] for row in rows}) == 42
assert sum(row["weng_locator"].startswith("reference-") for row in rows) == 39
assert sum(row["weng_locator"].startswith("body-link-") for row in rows) == 3
assert {row["section_id"] for row in rows} == {
    "system-being-improved",
    "harness-design-patterns",
    "harness-layer-vs-core-intelligence",
    "context-engineering",
    "workflow-design-and-search",
    "self-improving-harnesses",
    "evolutionary-search",
    "joint-harness-weight-optimization",
    "future-challenges",
}
assert all(row["evidence_state"] == "card-complete" for row in rows)
assert all(row["retrieval_state"] == "unseen" for row in rows)
assert all(Path(row["card_path"]).is_file() for row in rows)
assert all(Path(row["captured_path"]).is_file() for row in rows)
assert len(list(Path("lessons").glob("*.html"))) == 10
assert len(list(Path("reference").glob("*.html"))) == 4
print("WENG_CURRICULUM_AUDIT_OK")
PY
```

Expected: `WENG_CURRICULUM_AUDIT_OK`.

- [ ] **Step 4: Regenerate derived Harp artifacts**

If source-registry or canonical content changes affect the corpus:

```sh
cargo run -p harp -- build
cd atlas && corepack pnpm run export:html
```

Return to the repository root.

- [ ] **Step 5: Update the tracked payload receipt**

Stage the complete intended surface, run:

```sh
cargo run -p harp -- repository verify
```

If it reports a stale payload digest, update only the digest in
`docs/import-receipt.md`, restage it, and rerun the command.

- [ ] **Step 6: Run the full release gate**

```sh
mise run verify
git diff --cached --check
```

Expected:

- Rust formatting, clippy, and workspace tests pass;
- curriculum test passes;
- source verification passes;
- search is current;
- Meta-Harness lab tests pass;
- Atlas lint, types, tests, and export pass;
- LFS and repository verification pass; and
- no authored whitespace errors remain.

- [ ] **Step 7: Commit integration metadata**

```sh
git add docs/product-contract.md docs/import-receipt.md \
  atlas/src/content/generated/corpus.json \
  atlas/dist/harp-atlas.html atlas/dist/harp-atlas.receipt.json \
  mise.toml
git commit -m "chore: verify Weng teaching curriculum" \
  -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

Omit generated paths from `git add` when they are unchanged.

## Implementation Order and Checkpoints

1. Tasks 1–2 establish the red contract and teaching infrastructure.
2. Tasks 3–6 promote the 20 identity-only sources in four reviewable batches.
3. Task 7 normalizes the existing 22 sources and completes the 42-card matrix.
4. Task 8 creates compact reusable references.
5. Task 9 creates learner-facing lessons.
6. Task 10 proves the exact prompt-to-artifact contract and lands receipts.

Do not author lessons before all 42 canonical cards exist. Otherwise teaching
prose will become the accidental claim authority.

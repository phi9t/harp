# Weng Harness Teaching Curriculum Design

**Status:** Approved design, pending implementation planning
**Date:** 2026-08-08
**Mission:** Explain every main idea in Lilian Weng's *Harness Engineering for
Self-Improvement* and every cited work at a useful high level
**Teaching root:** repository root plus `lessons/`, `reference/`,
`learning-records/`, and `assets/`

## 1. Objective

Build a stateful teaching curriculum that lets the learner:

1. reconstruct Weng's complete argument across all nine maintained sections;
2. explain every numbered citation and substantive body-linked source at a
   useful high level;
3. distinguish task iteration, persistent adaptation, harness improvement,
   successor improvement, and demonstrated recursive improvement;
4. compare adjacent research systems without collapsing their edited objects,
   evaluators, budgets, or claim ceilings;
5. audit whether a reported result supports the interpretation Weng gives it;
   and
6. retain the material through retrieval practice rather than recognize it
   only while reading.

The curriculum is complete only when it accounts for:

- 9 of 9 Weng section companions;
- 39 of 39 numbered references;
- 3 of 3 substantive body-linked sources; and
- 42 of 42 source cards under the approved explanation contract.

## 2. Current Coverage Baseline

Harp already has:

- a byte-pinned Weng article capture;
- a nine-section reading map;
- citation edges for all 39 numbered references;
- three body-linked source edges;
- 16 full system readings, 14 of which correspond directly to numbered Weng
  references;
- several inspected supporting papers and benchmark sources; and
- local text captures for all 20 numbered references that remain
  identity-only.

The teaching gap is not source discovery. It is interpretation coverage.

At design time:

- all 9 article sections are mapped;
- 19 of 39 numbered citations have an inspected evidence status in the source
  registry;
- 20 of 39 numbered citations remain `MISSING` /
  `vendored-uninspected`; and
- the three body-linked sources have mixed evidence depth.

No lesson may imply exhaustive coverage until the source-card matrix reaches
42 of 42 complete cards and every numbered reference has moved out of
identity-only status.

## 3. Teaching Mission

`MISSION.md` will record:

### Why

The learner wants to understand Weng's practical RSI framing well enough to
explain the full argument, discuss every cited research direction without
hand-waving, and evaluate whether each work supports harness improvement,
successor improvement, or only a narrower result.

### Success looks like

- Give a coherent 10-minute explanation of Weng's complete argument without
  notes.
- Give a 90-second explanation of any source in the coverage matrix.
- Place a paper in the correct harness-optimization family and name its edited
  object.
- State the paper's reported evidence and one important limitation without
  overstating reproduction.
- Explain why Weng cites it and whether that use is strong, weak, or
  illustrative.
- Compare two adjacent works using a stable comparison vocabulary.

### Constraints

- Primary-source grounding is mandatory.
- Captured upstream bytes remain immutable.
- `content/` remains the only technical-prose authority.
- Teaching artifacts are non-authoritative projections.
- Lessons should be short and retrieval-driven.
- The curriculum follows Weng's argument, not bibliography order.

### Out of scope

- Independent reproduction of every benchmark result.
- Full implementation walkthroughs for all 42 sources.
- Mathematical derivation beyond what is needed for high-level explanation.
- Treating all cited works as demonstrations of RSI.

## 4. Explanation Contract

Every source receives one source card that a learner can explain in roughly
90 seconds.

Each card has exactly these semantic fields:

1. **Problem:** What problem does the work try to solve?
2. **Core mechanism:** What object changes, and how?
3. **Reported evidence:** What did the authors evaluate or report?
4. **Key limitation:** What should the learner not infer?
5. **Why Weng cites it:** What role does the work play in Weng's argument?

Cards also carry metadata:

- source ID;
- title;
- Weng reference number or body-link locator;
- public primary-source URL;
- local captured-text path;
- publication state;
- evidence status;
- Weng section;
- edited-object family;
- claim ceiling; and
- canonical Harp route where one exists.

### 4.1 Edited-object families

Use one primary family:

- historical RSI framing;
- model self-play or weight adaptation;
- context artifact;
- context-management mechanism;
- workflow or agent-program search;
- harness-code search;
- evolutionary program or population search;
- joint harness and weight adaptation;
- automated research system;
- evaluation benchmark;
- safety or reward-hacking framing; or
- implementation case study.

### 4.2 Reported-evidence rule

Cards summarize author-reported evidence at the source's admitted claim
ceiling. They must not silently promote:

- abstract claims into implementation correctness;
- benchmark results into independent reproduction;
- task gains into recursive improvement;
- an archive into behavioral diversity;
- Docker use into a sandbox proof; or
- a company essay into peer-reviewed evidence.

## 5. Coverage Matrix

The machine-readable coverage matrix will own completeness. It will contain one
row per source card and these columns:

| Field | Meaning |
|---|---|
| `source_id` | Stable Harp source identifier |
| `weng_locator` | Reference number or body-link section |
| `title` | Maintained source title |
| `section_id` | Weng section that teaches the source |
| `card_path` | Canonical teaching-card locator |
| `primary_url` | Primary public source |
| `captured_path` | Local captured text or implementation source |
| `evidence_state` | `identity-only`, `inspected`, or `card-complete` |
| `claim_ceiling` | Strongest supported use |
| `lesson_ids` | Lessons that exercise this card |
| `retrieval_state` | `unseen`, `introduced`, `retrieved`, or `durable` |

Required invariants:

- exactly 42 unique source IDs;
- exactly 39 numbered-reference locators;
- exactly 3 body-link locators;
- every row maps to one of the nine Weng sections;
- every row has a primary URL and local capture;
- no `card-complete` row has an identity-only registry status;
- every source appears in at least one lesson;
- every lesson source link resolves to a matrix row; and
- no source card is counted twice because one paper appears in two discussions.

## 6. Curriculum Architecture

Use a section-first sequence. Source cards are supporting units, not the main
navigation.

### Lesson 1: The system being improved

Teach:

- classical RSI framing;
- model versus training pipeline versus deployment harness;
- the candidate/protected-envelope split; and
- Autoresearch as a bounded workflow example.

Sources include:

- Good 1965;
- Yudkowsky 2008;
- Anchored Self-Play;
- Absolute Zero;
- Self-Rewarding Language Models;
- SPIN;
- Anthropic's RSI essay; and
- Karpathy Autoresearch.

### Lesson 2: Harness design patterns

Teach:

- workflow loops;
- filesystem-backed state;
- subagents and backend jobs;
- coding-agent tools; and
- why these are prerequisites rather than RSI evidence.

The lesson reuses implementation examples but does not create duplicate source
cards for systems already counted elsewhere.

### Lesson 3: Harness versus core intelligence

Teach:

- external procedure versus internalized behavior;
- activation, adherence, and outcome as separate capabilities;
- harness-induced capability without weight changes; and
- compatibility between model and harness.

Sources include:

- Harness Updating Is Not Harness Benefit; and
- Recursive Language Models as a non-Weng complementary anchor.

RLM is taught as a boundary example but is not added to the 42-source Weng
count because it post-dates and is outside Weng's bibliography.

### Lesson 4: Context engineering

Teach the progression:

```text
context artifact
-> context-construction skill
-> executable context-management harness
```

Sources:

- ACE;
- MCE; and
- Meta-Harness.

### Lesson 5: Workflow design and automated research

Teach:

- expert-authored research workflows;
- evidence-linked research systems;
- generated-data workflows;
- meta-agent search over agent programs; and
- MCTS over executable workflows.

Sources:

- AI Scientist;
- ScientistOne;
- Autodata;
- ADAS;
- Self-Refine; and
- AFlow.

### Lesson 6: Self-improving harnesses

Teach:

- code as a harness representation;
- improving the improver;
- bounded propose/evaluate/accept loops;
- observability-driven evolution; and
- external promotion authority.

Sources:

- STOP;
- Self-Harness;
- Harness Updating Is Not Harness Benefit;
- AHE; and
- Weng's reward-hacking article.

### Lesson 7: Evolutionary search

Teach:

- prompt evolution;
- reflective population search;
- program evolution;
- stepping stones and archive policies;
- task-time discovery; and
- demonstration-augmented evolution.

Sources:

- Promptbreeder;
- GEPA;
- AlphaEvolve;
- ShinkaEvolve;
- ThetaEvolve;
- DGM;
- Hyperagents;
- Learning to Discover at Test Time;
- Epistemic Uncertainty for Test-Time Discovery; and
- DemoEvolve.

### Lesson 8: Joint harness and weight optimization

Teach:

- feedback routing between harness and weights;
- model-harness distribution shift;
- DAgger-style relabeling;
- continual policy learning; and
- why joint gains are hard to attribute.

Sources:

- SIA; and
- Continual Harness.

### Lesson 9: Future challenges and evaluation

Teach:

- why paper generation is not scientific discovery;
- weak evaluators and scientific taste;
- negative results and memory;
- diversity collapse;
- reward hacking;
- long-term maintainability; and
- human oversight.

Sources:

- Why LLMs Aren't Scientists Yet;
- Early Science Acceleration Experiments with GPT-5;
- PaperBench;
- RE-Bench;
- MLE-bench;
- ScienceAgentBench;
- CORE-Bench; and
- KernelBench.

### Lesson 10: Synthesis and oral defense

The learner must:

- reconstruct Weng's causal chain;
- draw the optimization ladder;
- classify unfamiliar systems by edited object;
- compare selected adjacent papers;
- identify evidence/claim mismatches; and
- answer randomly sampled source-card prompts.

## 7. Research Workstream

The 20 identity-only numbered references are the first research backlog.

Each source is interpreted from the local primary-source capture under one
comparison contract:

- authors' problem statement;
- edited object and consumer contract;
- mechanism;
- reported evaluation;
- strongest limitation or confound;
- Weng's use of the source; and
- Harp assessment.

Promote a source registry row only after:

1. the primary text is inspected;
2. material claims have locators;
3. quantitative scopes are explicit;
4. the card passes the five-field contract; and
5. the evidence graph note reflects the new claim ceiling.

The workstream may batch related papers, but each source retains an independent
review record.

## 8. Teaching Workspace Files

### Root state

- `MISSION.md`: approved learning mission.
- `RESOURCES.md`: high-trust primary sources and explicit gaps.
- `NOTES.md`: teaching preferences and operational notes.

### Reference documents

- `reference/weng-harness-map.html`: nine-section argument map and optimization
  ladder.
- `reference/weng-source-cards.html`: filterable 42-card reference.
- `reference/rsi-claim-ladder.html`: task improvement through recursive
  improvement.
- `reference/harness-comparison-matrix.html`: edited object, evaluator,
  persistence, and claim ceiling.

### Lessons

- `lessons/0001-system-being-improved.html` through
  `lessons/0010-synthesis-and-oral-defense.html`.

### Reusable assets

- `assets/course.css`: print-friendly shared visual system.
- `assets/quiz.js`: accessible retrieval checks with immediate feedback.
- `assets/coverage.js`: source-card filtering and progress display.

### Learning records

Create a learning record only after the learner demonstrates understanding.
Do not record exposure as mastery.

## 9. Lesson Contract

Every lesson:

- states one concrete learner win;
- names the Weng section and primary source;
- links every discussed paper to its source card;
- contains no unsupported technical claim;
- includes at least one retrieval check;
- includes one compare-or-classify exercise;
- recommends one primary source for further reading;
- ends with a follow-up invitation;
- uses shared assets rather than duplicated inline behavior; and
- remains short enough for one focused session.

Quiz options should not leak answers through length or formatting.

## 10. Retrieval and Durability

Use four retrieval forms:

1. **Free recall:** reconstruct a section without notes.
2. **Card recall:** answer the five fields for a random source.
3. **Discrimination:** distinguish adjacent systems with similar names.
4. **Transfer:** classify a new system by edited object and evidence.

Durability rule:

- introduction does not count as retrieval;
- one correct same-session answer marks `retrieved`;
- a correct later-session answer marks `durable`; and
- incorrect recall schedules the source in the next interleaved lesson.

## 11. Verification

Add a repository-owned curriculum verifier that checks:

- coverage matrix has exactly 42 unique source rows;
- 39 numbered and 3 body-linked locators are present;
- all nine section IDs are covered;
- every source card has the five semantic fields;
- no numbered source remains identity-only;
- all local links resolve;
- all lesson and reference HTML files load shared assets;
- every source appears in a lesson;
- quiz controls have accessible labels;
- no absolute local paths or placeholder markers appear; and
- teaching files remain outside canonical Atlas compilation unless explicitly
  registered later.

The full release gate remains:

```sh
mise run verify
```

Teaching verification supplements rather than replaces canonical corpus,
source, Atlas, licensing, and repository checks.

## 12. Ownership and Evidence Boundaries

- `content/` owns technical prose and claim ceilings.
- `evidence/` owns immutable upstream captures and source licenses.
- `reference/` and `lessons/` are teaching projections.
- Source cards may compress canonical claims but may not become their sole
  home.
- If teaching work discovers a new material claim or correction, update
  `content/` first, regenerate derived artifacts, then update the card.
- Do not rewrite captured source text for pedagogy.

## 13. Completion Gates

The curriculum is complete only when:

1. all 9 section lessons exist;
2. the synthesis lesson exists;
3. all 42 source cards pass the explanation contract;
4. all 20 currently identity-only numbered references have inspected evidence
   status;
5. the learner can give the 10-minute Weng explanation;
6. the learner can answer a random sample of source cards without notes;
7. the coverage verifier passes;
8. `mise run verify` passes; and
9. the prompt-to-artifact audit finds no uncovered section, citation,
   body-linked source, lesson, or source card.

Passing tests without 42-source semantic coverage does not satisfy the goal.

## 14. Non-goals

- Reproduce every paper's experiments.
- Produce full paper-by-paper implementation tutorials.
- Replace primary papers with summaries.
- Claim learning from page views alone.
- Add the teaching workspace to the canonical Atlas corpus by default.
- Create one monolithic lesson that overloads working memory.

## Appendix A. Required Source Roster

The coverage verifier compares the matrix against this exact semantic roster
and against `content/sources/evidence_graph.tsv`.

### Numbered references

1. `GOOD-1965`
2. `YUDKOWSKY-2008`
3. `ASP`
4. `ABSOLUTE-ZERO`
5. `SELF-REWARDING`
6. `SPIN`
7. `ACE`
8. `MCE`
9. `META-HARNESS`
10. `AI-SCIENTIST`
11. `SCIENTISTONE`
12. `AUTODATA`
13. `ADAS`
14. `SELF-REFINE`
15. `AFLOW`
16. `STOP`
17. `SELF-HARNESS`
18. `PROMPTBREEDER`
19. `GEPA`
20. `ALPHAEVOLVE`
21. `SHINKAEVOLVE`
22. `THETAEVOLVE`
23. `DGM`
24. `HYPERAGENTS`
25. `LEARNING-DISCOVER`
26. `EPISTEMIC-DISCOVERY`
27. `SIA`
28. `NOT-SCIENTISTS`
29. `GPT5-SCIENCE`
30. `PAPERBENCH`
31. `REBENCH`
32. `MLEBENCH`
33. `SCIENCEAGENTBENCH`
34. `COREBENCH`
35. `KERNELBENCH`
36. `HARNESS-DISENTANGLE`
37. `AHE`
38. `CONTINUAL-HARNESS`
39. `DEMOEVOLVE`

### Substantive body-linked sources

40. `KARPATHY-AUTORESEARCH`
41. `WENG-REWARD`
42. `ANTHROPIC-RSI`

`RLM-PAPER` remains a complementary teaching anchor in Lesson 3 but is outside
the 42-source Weng completeness denominator.

---
id: rsi-harness-benchmark-field-guide
title: Harness benchmark field guide and MCE protocol
type: technical-review
status: active
created: 2026-08-09
updated: 2026-08-09
tags: [benchmarks, harnesses, evaluation, meta-harness, mce, evidence]
---

# Harness benchmark field guide and MCE protocol

This is Harp's canonical technical review of the benchmark surfaces used to
argue for harness improvement. It pairs a field guide for executable-harness
systems with a complete operational companion for Meta Context Engineering
(MCE). It does **not** synthesize a leaderboard: the systems differ in
candidate object, evaluator, visibility, repetitions, and resource envelope.

## Evidence tiers and claim ceilings

| Tier | Authority in this packet | What it can support | What it cannot support |
|---|---|---|---|
| Paper-era author report | [MCE], Meta-Harness, DGM, Self-Harness, AHE, Harness Disentangle, and DarwinX papers captured below `evidence/` | Described protocol, reported outcome, and stated limitation at the cited locator | An independent reproduction or a current implementation guarantee |
| Pinned implementation behavior | [META-HARNESS-REPO] and [DGM-REPO] snapshots | Present-day input, candidate, controller, and verifier interfaces | Historical run identity or reported score |
| Immutable source receipt | `evidence/benchmarks/manifest.tsv` | URL, immutable identity, byte count, SHA-256, visibility, and license status | Permission to redistribute a remote-only task or evidence that it was run |
| Harp-local evidence | Hash-verified MIT SWE-bench evaluator file and existing Meta-Harness interface experiment | Receipt correctness and the explicitly recorded local interface boundary | Benchmark quality, held-out score, paid-model result, or transfer claim |
| Missing evidence | Seeds, complete dataset object revisions beyond the recorded receipts, raw run traces, serving identities, cost receipts, and rights clearance | A clearly bounded non-claim | A reproduction claim |

The MCE primary source is the immutable arXiv v2 capture:
`evidence/weng/artifacts/pdf/mce.pdf` (PDF digest in
`evidence/weng/receipts/mce.tsv`) and
`evidence/weng/text/mce.txt` (text digest in the same receipt). The
[canonical arXiv page](https://arxiv.org/abs/2601.21557) is the paper landing
page; the captured evidence above remains the claim anchor.

## How to read a harness benchmark

Score comparison is meaningful only after fixing the object that changes, the
authority that runs the verifier, the task visibility, and the number of
independent attempts. The following matrix is a protocol map, not a ranking.

| System / path | Task object | Mutable object | Environment and verifier | Score / repetition | Development versus held-out | Candidate visibility | Justified conclusion |
|---|---|---|---|---|---|---|---|
| Meta-Harness text classification | Dataset examples and a `MemorySystem` contract | Python context/memory system | Released controller imports candidate and calls task benchmark | Validation during evolution; explicit finalization for test | Controller delays `--test`, but release includes test data | Proposer sees archive, code, scores, and traces | Paper and pinned source support executable context-policy search; OS isolation is still needed for held-out protection |
| Meta-Harness TB2 | 30-task development subset or full 89 tasks | `Terminus2` subclass / harness code | Harbor runner, import gate, optional smoke task, task verifier | Search trials; optional five-trial final run | Development subset versus full dataset, not a secret-proof boundary | Candidate is imported by controller in released reference path | `extract-elf` is a cheap smoke contract, not a general score |
| Darwin Gödel Machine (DGM) repository repair | SWE-bench Verified task / selected subsets | Agent repository and lineage patches | Dockerized patch application and SWE evaluator | Staged 10/60/200-task checks in reported/released paths | Search subsets and larger evaluation stages | Diagnostic path may inspect private test material in its setup | DGM demonstrates a more recursive mutable agent surface, but protected evaluator control remains external |
| Darwin Gödel Machine (DGM) Polyglot | Language exercise, including C++ `all-your-base` | Same agent repository | Polyglot harness and container execution | Search/evaluation stages in paper and source | Separate Polyglot path; cross-benchmark transfer is author-reported | Candidate source and archive are visible to the improver | A language-exercise result is not interchangeable with repository repair |
| DarwinX Terminal-Bench 2.1 | Same 89 verifier tasks | Proprietary Monet harness skill and code layers | Paper-described proposer, reasoned verifier, `avg@3` screens, `avg@5` confirmation | Official `avg@5`; task-budget timeouts fail | Same suite drives evolution and reporting | No public optimizer or Monet source located | Supports author-reported full-system harness gain, not operator attribution or reproduction |
| DarwinX TerminalWorld | 94 train tasks and 41 disjoint held-out tasks | Same harness layers plus archive and merge selection | Adaptive training subsets; single-attempt held-out verifier | Held-out pass@1 | Report split is disjoint; same task family and environment | Raw archive and merge artifacts unavailable | Supports suggestive archive-diversity evidence; merged harness exceeds strongest specialist by one task |
| DarwinX WebArena-Infinity | 300 synthetic intents and 1,260 real tasks | Browser skills, prompt policy, and harness code | LLM-judge screens; deterministic report verifiers; static plus LLM action audit | Audit-clean real-task pass@1 | Intents and reward source change; nine of ten report apps have synthetic counterparts | Real tasks and verifiers hidden from the reported evolution loop | Supports a large author-reported full-system gain and validity shift; every attempted merge is reverted |
| Self-Harness | 64 task cases | Harness rules derived from observed weaknesses | Fixed base model and regression gate | Paper reports held-in/held-out split | 32 held-in / 32 held-out cases | Controlled by the reported protocol | Supports bounded harness editing, not a general successor-producer result |
| Agentic Harness Engineering (AHE) | TB2 then SWE-bench Verified | Observable harness components | Three-role evidence loop and benchmark controller | Ten TB2 iterations; reported frozen transfer | Evolves on TB2, transfers without re-evolution to SWE-bench Verified | Evidence corpus is intentionally exposed to the evolver | Supports author-reported transfer under its fixed envelope; no local run |
| Harness Disentangle | SWE-bench Verified, MCP-Atlas, SkillsBench | Skills; MCP also exposes prompt/memory surfaces | Benchmark-specific read-only tools/evaluators | Per-task means; SkillsBench reports five trials | Varies task solver and evolver separately | Skills are editable but evaluator files read-only | Separates update quality from benefit, activation, and adherence |
| DemoEvolve / Continual Harness | Demonstration-guided game setting / online game episode | Harness state | Their reported task-specific evaluators | Comparator shape only | Not a common held-out benchmark basis | Different online/offline semantics | Useful for evaluator design vocabulary, not a comparable score table |

### Four concrete task receipts

- **Terminal-Bench 2 `extract-elf`.** Meta-Harness and Self-Harness use this
  as a fast bring-up/smoke task. The pinned Meta-Harness reference says to
  import-check, run `extract-elf`, then expand to hard or full evaluation
  (`evidence/implementations/meta_harness/snapshot/reference_examples/terminal_bench_2/README.md`).
  The task specification and Python verifier are remote-only, hash-pinned
  receipts at Terminal-Bench revision `d28711d0…`; their redistribution status
  remains `unconfirmed` in `evidence/benchmarks/manifest.tsv`.
- **SWE-bench Verified `astropy__astropy-12907`.** This is a selected
  repository-repair example in DGM's released subset
  (`evidence/implementations/dgm/snapshot/swe_bench/subsets/big.json`) and
  appears in the captured DGM paper. Harp vendors only the MIT-licensed
  evaluator `evidence/benchmarks/swe-bench/grading.py`, plus its license; the
  dataset row remains remote-only at Hugging Face revision
  `c104f840…`.
- **Aider Polyglot C++ `cpp__all-your-base`.** DGM's Polyglot small subset
  includes the task. The remote receipt pins the C++ test file at
  Polyglot revision `7e0611e…`; no task source is vendored because rights
  clearance was not recorded.
- **SkillsBench `flink-query`.** Harness Disentangle uses it to show a
  different measurement: without the evolved skill, the fixed solver omits a
  FINISH-event filter and scores 0.67; with either evolved skill at turn zero
  it scores 1.0 (`evidence/weng/text/harness-disentangle.txt:488-570`).
  This is an **activation/adherence** case, not a proxy for benchmark score or
  a universal task-difficulty ranking.

## System-specific protocol notes

### Meta-Harness

The text-classification candidate subclasses `MemorySystem`, predicts before
ground truth, learns afterward, and round-trips serializable state. The
controller evaluates validation data during evolution and exposes test
evaluation only at explicit finalization
(`evidence/implementations/meta_harness/snapshot/reference_examples/text_classification/meta_harness.py`).
That is a protocol boundary, not access control if test bytes share a readable
filesystem.

The TB2 reference pins baselines, model, trials, concurrency, timeout, a
30-task development option, and full 89-task option. Candidate import is
execution; type checking after import cannot protect a long-lived evaluator
from module-level effects. The experimental Harbor controller improves this by
staging source into a short-lived child, but Harp did not run the dependent
benchmark environment. See the evidence-tiered
[Meta-Harness deep dive](../meta_harness/meta_harness_deep_dive.md).

### DGM, Self-Harness, AHE, and Harness Disentangle

DGM alters the coding-agent repository itself; its external controller still
owns archive update, Docker execution, task subset, score, and promotion.
Its SWE and Polyglot paths should be read as different evaluators, rather than
as one interchangeable coding score. The detailed source walk is in
`knowledge/darwin_godel_machine/`.

Self-Harness's reported 64-case split makes regression gating visible, while
AHE's reported TB2-to-SWE transfer is a more direct development/held-out
exercise. AHE fixes its base model and evolves the full observable harness;
the source reports ten TB2 iterations and then a frozen-harness transfer
(`evidence/weng/text/ahe.txt:280-374`). Those are paper-era author reports.

Harness Disentangle evaluates three benchmarks and distinguishes an evolver
writing a useful update from a task agent loading and following it. SkillsBench
is particularly valuable because it makes an apparently “good” skill
measurable at two separate boundaries: load action formatting (activation) and
following the loaded procedure (adherence).

### DarwinX

DarwinX keeps several harness lineages, applies a bounded-regression fitness
gate, and requires higher-fidelity confirmation before a node may steer later
search. Its paper reports four regimes: in-domain TB2.1 evolution, held-out
TerminalWorld tasks, synthetic-to-real WAI transfer, and one-way TB2.1-to-SWE-V
transfer.

The benchmark evidence supports the full harness-selection system. It does not
separate the archive, parent selector, reasoned verifier, regression gate,
recombination operator, or adaptive inference effort. WAI's large result comes
from one accepted lineage because every merge is reverted. TerminalWorld has
the only positive merge result, one task beyond the strongest specialist.

The complete [DarwinX packet](../darwinx/darwinx_index.md) records the paper's
`avg@k`, inference-compute, action-policy, anti-cheating, and reproducibility
boundaries.

## MCE experimental protocol

### Overall contract

**EVIDENCE — [MCE], §§3–4 and Appendix A–E,
`evidence/weng/text/mce.txt:103-865,881-2125`.** MCE runs a bi-level loop:

1. A **meta-agent** reads skill history, implementation folders, and aggregate
   train/validation metrics; it writes a new self-contained learning skill.
2. A **base agent** receives that skill, a batch of rollouts, prior best
   context, and utilities; it writes context files and
   `retrieve_context.py`.
3. A fixed generator consumes a one-shot `query → context` interface. It does
   not interact with the artifacts during inference in the reported baseline
   protocol.
4. Evaluation feeds results back to the meta level. The paper describes five context-optimization epochs,
   with sub-iterations and selection of the
   best context for the next epoch.

“Offline” here means learning context from saved training rollouts and
evaluating a generated answer against task labels; it is not an online,
mid-episode harness refiner. The base agent has filesystem/tool authority
inside its assigned iteration workspace, while the generator remains a
one-shot caller. The paper uses a generator, reflector, and meta-agent role
structure; the Appendix E utilities say the LLM utility is fixed to DeepSeek
V3.1 and embeddings to `text-embedding-3-small`. The reported comparison says
baselines are budget-matched, but the capture does not supply complete raw run
traces, random seeds, full serving identities, cost receipts, or an
independent reproduction.

The paper reports rollout/time accounting and performance aggregates in its
main experiment section. Harp records them as author reports only. It does not
run paid models, containers, remote sandboxes, or MCE task evaluations.

### Appendix A — Task and generator contracts

| Task | Sampled split / selection | Input → output contract | Metric and test boundary | Generator prompt structure |
|---|---|---|---|---|
| FiNER | Debt/credit-focused subset; 200/100/100 train/validation/test, 12 XBRL tags drawn from a 139-tag vocabulary | Highlighted entity plus sentence → tag name | Exact tag prediction on 100 held-out samples | XBRL-expert role; tag-option list; `{context}`; question; JSON `reasoning` plus `final_answer` |
| USPTO-50k | 50 train + 30 validation sampled from train; 100 original-test examples; stratified across 10 reaction types | Product SMILES and reaction type → precursor SMILES, dot-separated | Test split, exact retrosynthesis evaluation described by source | Organic-chemist role; `{question}` and strategic `{context}`; analyze functional groups/disconnection; JSON reasoning and precursor-only final answer |
| Symptom2Disease | 200/50 stratified training/validation; preserved original 212-sample test | Symptom narrative → one of 22 disease labels | Case/whitespace-normalized exact diagnosis on preserved test | Medical diagnostician; enumerated labels; `{context}` and symptoms; final `[DIAGNOSIS]name[/DIAGNOSIS]` |
| LawBench criminal charge | Random 200/50/100 samples | Chinese case facts → one or more Chinese charges | Micro-F1; set comparison for multi-charge case | Chinese judge role; `{context}` and case facts; output `[罪名]charge1;charge2<eoa>` |
| AEGIS2 | Four worst LLaMA-3-SafeGuard categories; 400 train, 128 validation, 140 balanced test | User prompt → `safe`/`unsafe`, optional categories | Safe/unsafe F1; categories secondary | Safety-policy context wrapped in XML-like markers; conversation; JSON `reasoning`, `Safety_Categories`, and `final_answer` |

The complete literal task specifications and prompt templates are immutable
source text at `evidence/weng/text/mce.txt:884-1111`. The table is an
operational transcription: it preserves field names, format gates, split
boundaries, and selection rules while locating the unmodified canonical text
for the long tag list, Chinese prose, and full raw blocks.

### Appendix B — Agent-prompt contracts

#### Meta-agent contract

The meta-agent is instructed to evolve a **self-contained** context-learning
skill, not iteration-specific advice. It reads:

- `{workspace_base}/meta_agent/` for iteration history;
- full `train.jsonl` for task understanding;
- `evaluations.json` for aggregate `train_acc` and `val_acc`;
- previous `skills/` and read-only sub-iteration folders;
- prior context and retrieval code as artifacts to inspect, not silently
  overwrite.

It writes only
`{iter_name}/.claude/skills/learning-context/SKILL.md`; the prompt identifies
that as the output location and limits write authority to that skill
directory. It must analyze skill → implementation → result history, use
agentic crossover, preserve useful mechanisms, diagnose regressions and
overfitting, make a concrete executable procedure, and end with validation
checks. The complete role, workspace tree, historical-metrics instructions,
quality gates, and final-answer requirements are at
`evidence/weng/text/mce.txt:1112-1328`.

#### Base-agent contract

The base agent receives the selected learning skill, a batch of training
rollouts, prior best context, task specification, and utility modules. It can
read context, examples, batch data, and historical artifacts; it writes
`context/` markdown/files plus `retrieve_context.py`. Its required procedure
is to inspect the task and previous context, analyze successes/failures,
synthesize generalizable guidance, curate conflicting rules, implement or
update retrieval, and validate syntax, paths, and output shape before
completion. The prompt makes quality gates explicit: do not memorize examples,
do not remove useful prior knowledge without evidence, preserve the output
contract, and check that the retrieval interface returns context for a query.

The long raw prompt includes exact directory names, tool/utility descriptions,
and final response framing at `evidence/weng/text/mce.txt:1329-1587`. This
packet intentionally does not reformat or substitute that source block.

### Appendix C — Candidate skill representations

Appendix C presents learned skills as executable candidate representations, not
as a scalar “prompt.” Across examples, the representation includes:

- a methodology for batch review and error clustering;
- reflection/curation instructions that turn failures into generalized rules;
- batch synthesis procedures and explicit anti-overfitting constraints;
- context artifacts split into files plus a query-conditioned retrieval
  function; and
- admissibility requirements: valid workspace paths, stable interface,
  executable/parseable retrieval code, and a final validation step.

The concrete examples and their exact skill text are preserved at
`evidence/weng/text/mce.txt:1470-1587`; Appendix D then shows how those
representations changed task by task.

### Appendix D — Evolved task strategies

| Task | Initial → evolved strategy | Artifact organization / retrieval | Context-size observation and stated rationale |
|---|---|---|---|
| FiNER | Broad tag-pattern collection → semantic distinctions for confusable financial tags | Taxonomy-aware guidance and targeted retrieval | Context is organized to distinguish nearby XBRL meanings rather than repeat tag strings |
| USPTO-50k | Reaction-pattern accumulation → mechanism/disconnection-oriented chemistry workflow | Reaction-type context plus synthesis heuristics | Source argues structured chemical reasoning generalizes better than memorized reactions |
| Symptom2Disease | Generic diagnosis rules → discriminators and symptom “essences” | 1,440-line prioritized retrieval cascade; critical/surgical/semantic/error rules plus fallback | Extensive routing is justified as preventing irrelevant rules from contaminating a fine-grained 22-class decision |
| LawBench | Charge-pattern rules → structural fact decomposition | 11 files, about 30 KB; 177-line trigger-based augmentation | Source reports reduction from an earlier 88 KB context and attributes improved generalization to READ → DECOMPOSE → MATCH → VALIDATE rather than keyword matching |
| AEGIS2 | Error taxonomy → precision/recall-controlled safety distinctions for an 8B generator | 12 files; 995-line precision-first cascade with early safe returns | Small-model limitation motivates concise rules and exclusions; early safe routing is intended to reduce false positives |

Appendix D's detailed initial/optimal skill excerpts, file names, retrieval
architecture, and stated generalization interpretations are at
`evidence/weng/text/mce.txt:1588-2030`. The measured line counts and KB figures
are source-reported artifacts, not locally regenerated files.

### Appendix E — Utility implementation contract

The supplied `utils/llm.py` contract at
`evidence/weng/text/mce.txt:2031-2125` specifies:

- structured outputs through a Pydantic schema;
- `.with_retry(stop_after_attempt=3)`;
- temperature `0`;
- `MAX_CONCURRENCY = 50`;
- `MAX_LLM_CALLS = 100` per batch, with a `ValueError` above the limit;
- a semaphore around async invocations and `asyncio.gather` batch execution;
- OpenRouter credentials, base URL, and model selection read from environment
  configuration defined verbatim at the immutable source locator; and
- a separate embedding utility fixed to OpenAI
  `text-embedding-3-small`.

This is a service configuration and retry/concurrency contract, not a complete
model-serving receipt: actual provider routing, account, seed, token billing,
and raw responses are absent from the capture.

## What this packet does and does not establish

The receipt layer supports stable identity and rights-aware review. The field
guide supports a causal question for each system: **which mutable object was
evaluated by which protected verifier under which development/held-out
boundary?** The MCE companion provides enough operational detail to inspect
the claimed five-epoch bi-level protocol without inventing missing artifacts.

### No local reproduction

It does not establish a synthetic winner, independent benchmark reproduction,
cost-normalized comparison, or demonstrated recursive successor improvement.
Those would require sealed task/evaluator revisions, documented access control,
raw per-run traces, repeated seeds, model-serving and cost receipts, and a
matched next-cycle test in which an accepted child produces better later
accepted children under the same protected envelope.

## Reading routes

- [Meta-Harness system reading](../rsi/systems/meta-harness.md)
- [DGM system reading](../rsi/systems/dgm.md)
- [DarwinX population-selection packet](../darwinx/darwinx_index.md)
- [MCE system reading](../rsi/systems/mce.md)
- [Harness Disentangle system reading](../rsi/systems/harness-disentangle.md)
- [Evaluator integrity and promotion](../rsi/evaluator_integrity_and_promotion.md)
- [Benchmark receipt manifest](../../evidence/benchmarks/manifest.tsv)

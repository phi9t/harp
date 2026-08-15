---
id: rsi-meta-harness-deep-dive
title: Meta-Harness - evidence-tiered implementation deep dive
type: technical-deep-dive
mode: TECHNICAL REVIEW
status: active
created: 2026-08-08
updated: 2026-08-09
tags: [recursive-self-improvement, meta-harness, context-engineering, harness-search, provenance, ace, mce, trae-cli, codex-cli]
confidence: high
---

# Meta-Harness: from paper mechanism to pinned interfaces

Mode: `TECHNICAL REVIEW`.

> Meta-Harness searches executable policy around a fixed model. This document
> separates what the paper reports, what the dated project page reports, what
> the pinned public code does, and what one local TRAE proposal experiment
> actually established. It does not merge those tiers into one reproduction
> claim.

## Source tiers and claim ceilings

| Tier | Identity and local evidence | What it supports | Claim ceiling |
|---|---|---|---|
| arXiv v1 paper | [META-HARNESS], `evidence/weng/text/meta-harness.txt` | Paper-era outer-loop design, experiment protocol, ablations, and author-reported results. | Primary author report, not independent reproduction. |
| dated project page | [META-HARNESS-SITE], `evidence/meta_harness/site/index.html`, captured 2026-08-08 with HTTP `Last-Modified` 2026-08-04 | First-party tables, Pareto data, the visualized TerminalBench trajectory, and links to code and artifact repositories. | Mutable first-party presentation. "COLM 2026" is a dated site claim because no official venue or OpenReview record was located. |
| pinned public repositories | [META-HARNESS-REPO] at `44b9942127847f7421db70d8c7e48407f09a3c70` and [META-HARNESS-TB2-ARTIFACT] at `57fefdb2ff84af3fd81b69d67814acbe69bd0743` | Present-day interfaces, control flow, evaluation boundaries, and tests at those revisions. | Source behavior only. The main repository is cleaned paper code, not an independent reproduction; the artifact has no license file at its pinned revision. |
| local TRAE proposal experiment | [META-HARNESS-TRAE-RUN], `evidence/meta_harness/trae_run/receipt.json` | One `gpt-5.4` proposal iteration, strict response/write boundaries, and independent candidate-interface assessment. | Proposal-schema evidence only. No benchmark score, paid model evaluation, or held-out result was reproduced. |

The page's venue metadata appears in
`evidence/meta_harness/site/index.html:36-45`; its capture metadata and bytes are
fixed in `evidence/meta_harness/site_manifest.tsv`. The repository describes
itself as cleaned paper code and says it was not tested beyond verifying that
it runs in
`evidence/implementations/meta_harness/snapshot/README.md:47-56`.

## Candidate, archive, proposer, evaluator, and Pareto flow

**EVIDENCE. [META-HARNESS], §§1 and 3,
`evidence/weng/text/meta-harness.txt:100-103,197-265`.** The paper's abstract
machine is:

```text
candidate harness H_i
  -> evaluator on search tasks
  -> score vector + execution traces
  -> append code, scores, and traces to archive D
  -> proposer inspects selected parts of D
  -> candidate harness H_(i+1)
  -> Pareto retention
```

The proposer is not given one precompressed state. It uses filesystem tools to
select prior code, scores, and traces from an archive larger than its context
window. The external controller owns task execution, scoring, archive
publication, the fixed iteration budget, and final test-set evaluation. The
paper says the proposer sees search-set results but never test-set results
(`evidence/weng/text/meta-harness.txt:237-245`).

**INFERENCE.** The filesystem is both memory and query interface. This removes
a fixed summarizer from the critical path, but does not remove selection bias:
the proposer still chooses which evidence to inspect, and the evaluator still
defines what survives.

## Text-classification contract

### Candidate interface

**EVIDENCE. [META-HARNESS-REPO].** Every candidate is expected to subclass
`MemorySystem`, accept an LLM callable, predict before seeing ground truth,
learn from a batch afterward, and round-trip serializable state. The interface
also exposes prompt metadata and context length
(`evidence/implementations/meta_harness/snapshot/reference_examples/text_classification/memory_system.py:61-126`).

The loader converts `agents/name.py` to a module below the
`text_classification` package, imports it, and returns the first concrete
`MemorySystem` subclass
(`evidence/implementations/meta_harness/snapshot/reference_examples/text_classification/inner_loop.py:479-504`).
This makes package identity part of the candidate contract; Python syntax alone
is insufficient.

### Search and finalization

**EVIDENCE. [META-HARNESS-REPO].** The outer loop declares baseline files,
run-local `pending_eval.json`, validation frontier, evolution summary, result
directory, and finalization receipt
(`evidence/implementations/meta_harness/snapshot/reference_examples/text_classification/meta_harness.py:26-47`).
It import-checks proposed systems before benchmarking
(`meta_harness.py:192-215`), evaluates candidates on validation data during
evolution (`meta_harness.py:470-527`), and exposes held-out evaluation only
through explicit finalization (`meta_harness.py:286-357`).

The provider-free tests enforce that ordinary evolution never passes `--test`,
that finalization writes a blocking state, and that incomplete finalization
does not falsely close the run
(`evidence/implementations/meta_harness/snapshot/reference_examples/text_classification/tests/test_evolution_contracts.py:66-188`).
In a fresh `uv --native-tls` environment, all 28 shipped provider-free tests
passed on 2026-08-08. The bounded execution findings are recorded in
`evidence/meta_harness/source_test_findings.tsv`.

**BOUNDARY.** The public repository includes test data. Its README calls the
split operational isolation rather than access control
(`evidence/implementations/meta_harness/snapshot/reference_examples/text_classification/README.md:43-58`).
A malicious or overly privileged proposer could still read those bytes unless
the operating-system boundary prevents it.

## TerminalBench-2 boundaries

**EVIDENCE. [META-HARNESS-REPO].** The reference experiment fixes two
baselines, the 89-task full set, a model, trial count, concurrency, and timeout
(`evidence/implementations/meta_harness/snapshot/reference_examples/terminal_bench_2/meta_harness.py:80-112`).
Candidate validation imports the named class and requires a `Terminus2`
subclass before evaluation (`meta_harness.py:140-155`). A smoke task can reject
runtime failures before the expensive sweep (`meta_harness.py:459-520`).

The evaluation script validates the import path before invoking Harbor, chooses
either a 30-task development subset or the full dataset, and assembles the
model, environment, concurrency, attempts, and timeout arguments
(`evidence/implementations/meta_harness/snapshot/reference_examples/terminal_bench_2/scripts/run_eval.sh:1-98`).
The outer loop then:

1. reads `pending_eval.json`;
2. normalizes the named class to `AgentHarness`;
3. import-checks and optionally smoke-tests each candidate;
4. evaluates only survivors;
5. updates per-task and aggregate frontier state; and
6. optionally performs a five-trial final run
   (`meta_harness.py:680-880`).

**IMPORT BOUNDARY.** Validation imports candidate code in the outer Python
process. Import is execution: module-level code runs before subclass checks.
The released flow therefore relies on proposer trust, environment isolation,
and narrow writable surfaces in addition to type checks.

**ENVIRONMENT FINDING.** On 2026-08-08, a fresh
`uv sync --native-tls` could not resolve the pinned
`terminal-bench` Git revision
`1a6ffa9674b571da0ed040c470cb40c4d85f9b9b`. Harp performed static syntax and
trust-boundary review instead of claiming a TerminalBench runtime
reproduction.
The exact resolution failure is retained in
`evidence/meta_harness/source_test_findings.tsv`.

The separate artifact repository reports 76.4% over 89 tasks and five trials
and attributes the released harness to environment bootstrapping
(`evidence/implementations/meta_harness_tb2_artifact/snapshot/README.md:5-36`).
Those are artifact-author claims; Harp did not execute the benchmark.

## Experimental Harbor controller

The experimental Harbor path strengthens one dangerous boundary.

**EVIDENCE. [META-HARNESS-REPO].** Its trusted controller states that
candidate code is not imported by the long-lived outer agent. It first scans
for task-specific forbidden references, parses the AST, and requires an async
`AgentHarness.run` on a `BaseAgent` subclass
(`evidence/implementations/meta_harness/snapshot/experimental/harbor_meta_harness/controller.py:1-5,127-175`).

Accepted source is copied to a temporary directory as `candidate.py`; only the
child Harbor process receives that directory through `PYTHONPATH`
(`controller.py:201-292`). The test suite proves that leakage rejection occurs
before any child run and that the staged directory is exposed only to the
child (`evidence/implementations/meta_harness/snapshot/experimental/harbor_meta_harness/tests/test_controller.py:112-140,214-235`).

The outer agent uses an immutable bounded `EvaluationState`: `record` returns a
new value with one less evaluation and an appended result
(`evidence/implementations/meta_harness/snapshot/experimental/harbor_meta_harness/agents/meta_harness.py:33-39`).
Each scarce `evaluate_harness` call downloads candidate source, launches the
short-lived controller, records the result, and returns remaining budget
(`agents/meta_harness.py:53-101`). Its test checks that the original state
remains unchanged (`tests/test_controller.py:238-245`).

**ENVIRONMENT FINDING.** Harbor dependency resolution reached the LiteLLM Rust
bridge but failed because transitive AWS crates require Rust 1.94.1 while Harp
pins Rust 1.92.0. Static syntax, source, and test-boundary inspection therefore
do not become a claim that Harbor's full suite ran locally.
The exact toolchain finding is retained in
`evidence/meta_harness/source_test_findings.tsv`.

## Dated site state

### Pareto data

**CLAIM. [META-HARNESS-SITE].** The page says the text-classification search
uses three datasets, 20 iterations, two candidates per iteration, and holds test
sets until final evaluation
(`evidence/meta_harness/site/index.html:281-292`). It reports a 48.6% best
accuracy at 45.5K additional context characters, compared with ACE at 40.9%
and 203K. The exact plotted points are preserved in
`evidence/meta_harness/site/static/data/pareto.js:1`.

These are first-party reported results. The site capture does not convert them
into local measurements.

### Illustrative trajectory

**CLAIM. [META-HARNESS-SITE].** The page explicitly says its hard 19-task,
10-iteration evolution is illustrative, not the final reported run. It starts
from Terminus-KIRA at 28.5% and reaches 46.5% at iteration 7
(`evidence/meta_harness/site/index.html:104-120`). The complete trajectory is
preserved in `evidence/meta_harness/site/static/data/evo.js:1`.

The visible path is instructive because regressions remain visible:

- shorter output and forced early completion lose information or end work too
  soon;
- safe file inspection and database-copy rules improve robustness;
- testing as the target service user produces the largest displayed jump; and
- both iteration-8 proposals fail to import.

That last failure is not incidental. A code-search loop must preserve invalid
programs as evidence while preventing them from contaminating the evaluator.

## Local TRAE proposal experiment

### Boundary

**EVIDENCE. [META-HARNESS-TRAE-RUN].** Harp prepared an ignored workspace from
the pinned `MemorySystem` interface, three shipped baselines, and dated Pareto
data. The reference state explicitly says that per-dataset validation history,
`frontier_val.json`, validation traces, raw upstream proposer traces, benchmark
history, and held-out results were not reconstructed
(`evidence/meta_harness/trae_run/normalized/reference_state.json`).

The first schema-valid attempt used explicit `gpt-5.4`, exactly three
candidates, a workspace-write sandbox, no-approval headless execution, and only
read/search, shell, write, and edit tools. TRAE CLI 0.200.19 rejects simultaneous
legacy permission-mode and sandbox overrides; the raw archive preserves that
infrastructure attempt and the compatible
`approval_policy=never` plus `sandbox=workspace-write` command.

### Result

The proposal schema and exact write set passed. The valid candidate count is zero. All three files import
`reference.text_classification.memory_system`, while the pinned loader imports
candidates below the `text_classification` package. Independent isolated
imports therefore failed before cold-start, synthetic `predict`,
`learn_from_batch`, and state round-trip checks could complete
(`evidence/meta_harness/trae_run/normalized/validation.json:1-47`).

The invalid candidates remain in the normalized bundle and raw archive. The
receipt records three candidates, zero valid candidates, no benchmark call, no
held-out test, no reproduced score, and a successful secret scan without
redaction (`evidence/meta_harness/trae_run/receipt.json:1-30`).

**INTERPRETATION.** This is a proposal-interface result, not a negative
benchmark result. It validates that TRAE can inspect the source-backed state,
write the requested artifacts, and satisfy the final response schema. It also
shows why producer-side self-validation is weaker than consumer-side import
validation: the proposer tested its own alternate package setup and declared
success, while the upstream-compatible loader rejected every candidate.

No benchmark score, paid model evaluation, or held-out result was reproduced.

## Failure modes

### Held-out leakage

The text-classification controller delays test invocation, but test bytes are
present in the release. A `--test` convention alone does not isolate search.
The operating system must deny and audit reads.

### Candidate import execution

Both released examples import candidate modules to validate them. Module import
executes top-level code. The experimental Harbor controller is safer because a
short-lived child owns candidate import, but it still needs sandbox and secret
boundaries.

### Proposer privilege

The paper intentionally gives a coding-agent proposer broad filesystem access.
If evaluator code, held-out data, credentials, or baseline files share that
authority domain, search can optimize the measurement process rather than the
harness.

### Archive growth

Raw traces improve diagnosis but grow with candidates, tasks, and trials.
Retrieval cost, stale evidence, storage cost, and private-data retention become
part of the algorithm. Archive size is not progress.

### Score-only selection

Scalar scores hide why a candidate failed. The paper's information ablation
reports a large advantage from full traces
(`evidence/weng/text/meta-harness.txt:370-386`), but that does not make every
trace useful or safe.

### Trace contamination

Search-set inputs, grader details, secrets, or task-specific paths can enter
traces and then influence later candidates. Trace access must be classified,
filtered at the producer boundary, and audited after capture.

### Malformed candidates

Syntax, class shape, package identity, cold start, runtime behavior, and state
round-trip are separate gates. The site trajectory and local TRAE run both
contain import-invalid proposals, showing that schema-valid metadata is not
executable validity.

### Weak reproduction evidence

A README result, a cleaned repository, a five-file artifact, or one local
proposal iteration does not reproduce the paper. Reproduction requires the
same tasks, model, prompts, evaluator, search budget, candidate history,
resource accounting, held-out protocol, and repeated outcomes.

## ACE and MCE

| System | Main mutable object | Update mechanism | External fixed points |
|---|---|---|---|
| [ACE](https://arxiv.org/abs/2510.04618) | Structured context bullets or playbook state | Generator, reflector, and curator apply incremental updates from feedback. | Task model, workflow schema, evaluator, and update roles. |
| [MCE](https://arxiv.org/abs/2601.21557) | A context-engineering skill plus the context artifacts it produces | Outer meta-agent evolves the learning procedure; inner agent executes it. | Meta-agent, base model, task/evaluator set, and skill runtime. |
| Meta-Harness | Full executable harness code around a fixed model | Coding-agent proposer queries archived code, scores, and traces; controller evaluates and retains candidates. | Proposer, evaluator, tasks, model, archive policy, and promotion authority. |

The ladder increases the mutable surface from artifact to learning procedure to
executable policy. It also increases the integrity burden. More expressive
search can fix deeper failures, but it can also import code, read private
evidence, mutate accounting, or exploit a weak evaluator.

## Harness optimization is not demonstrated recursive successor improvement

Meta-Harness demonstrates author-reported harness optimization under external
search and evaluation. The accepted harness changes how a task model stores,
retrieves, and presents information. It does not replace the coding-agent
proposer, evaluator, task distribution, archive policy, or deployment
authority.

The missing recursive test is matched next-cycle improvement: under the same
model, tasks, permissions, evaluator, archive policy, and total budget, does an
accepted child become a better producer of later accepted children than its
parent? Neither the paper, dated site, pinned code, artifact, nor local TRAE run
provides that comparison.

## Harp context control plane for Trae CLI and Codex CLI

Status: approved system design, pending implementation.

This section translates ACE, MCE, and the Meta-Harness boundary analysis into a
Harp-owned product subsystem. It is a design, not a claim that the subsystem is
implemented or that either provider exposes a complete effective-prompt
manifest today.

### Decision summary

- Harp owns a local learning control plane around coding-agent CLIs. It does not
  place a learner inside either provider's live model/tool loop.
- One provider-neutral command launches and observes work:
  `harp run --provider trae|codex --workflow auto -- <task>`.
- The workflow registry supports CI repair, code review, dependency updates,
  general coding, and later workflow families through the same package
  contract.
- Harp pins one immutable context release to each episode and injects only a
  bounded user-level context supplement. It never edits `AGENTS.md`, provider
  configuration, installed skills, sandbox policy, approvals, network policy,
  credentials, MCP capability, or hidden evaluators.
- Provider adapters first target the shared non-interactive `exec --json`
  surface. App Server adapters may later add richer thread, compaction, and
  subagent telemetry behind the same Harp interface.
- V1 is observation plus reviewable suggestions. It cannot automatically
  install a skill, promote a playbook, switch an active release, or modify a
  target repository.
- Raw events remain private local evidence. ACE and MCE consume a separately
  validated and sanitized episode export, never the raw provider archive.
- Promotion remains unavailable until replay, audit, shadow, canary, rollback,
  and evaluator-integrity controls exist.

### Product boundary

Harp becomes the authority for six control-plane objects:

1. workflow definitions and routing;
2. immutable context releases;
3. deterministic context selection and rendering;
4. provider launch and observation;
5. normalized episodes and outcome receipts; and
6. reviewable ACE and MCE candidates.

Trae CLI and Codex CLI remain execution providers. They own model access,
authentication, their internal agent loop, tool implementation, session
persistence, provider configuration, sandbox enforcement, approvals, hooks,
skills, and App Server protocols. A target repository owns its source,
`AGENTS.md` chain, tests, review policy, and any optional Harp policy it
commits.

The optional repository policy path is `.harp/context-control.json`. It may
disable Harp, restrict allowed workflow IDs, pin an approved release, lower
context budgets, or require specific verification labels. It cannot broaden
provider permissions, replace `AGENTS.md`, define executable selectors, or
inject free-form prompt text. Harp records its digest in the episode manifest.

The initial subsystem is local-first. It requires no hosted registry and does
not transmit episodes to Harp. A future shared registry may distribute signed
approved releases, but repository and tenant isolation must remain explicit.

### Chosen architecture

Three integration approaches were considered:

| Approach | Advantage | Cost or risk | Decision |
|---|---|---|---|
| Wrapper-first shared control plane | Uses the stable common denominator, produces one normalized contract, and can ship incrementally. | The wrapper cannot initially reconstruct every provider-internal context layer. | Chosen. |
| App Server first | Rich lifecycle, thread, item, compaction, and subagent events. | Experimental and version-divergent protocols increase initial coupling. | Later adapter. |
| Native skills and hooks first | Most native provider experience. | Installation state, activation, and provider-specific behavior weaken cross-provider attribution. | Optional enrichment only. |

The wrapper contract must not assume that similar command names imply identical
semantics. On 2026-08-09, the locally installed `traecli 0.200.19` and
`codex-cli 0.144.5` both exposed non-interactive JSONL execution, output-file,
resume, and App Server surfaces. Their flags and feature sets still differ.
`harp providers doctor` therefore probes the installed binary, records its
version and supported capability set, and fails before a run when a required
capability is absent.

### Trust and authority layers

The subsystem preserves four separate layers:

| Layer | Examples | Harp learning authority |
|---|---|---|
| Protected runtime | model instructions, sandbox, approvals, network, credentials, MCP and tool capabilities | None |
| Human repository contract | root and nested `AGENTS.md`, source ownership, required verification | Read-only input; suggestions remain outside the contract |
| Governed context artifacts | workflow package, playbook, selector, renderer, exported skill candidate | Candidate generation and explicit versioning |
| Ephemeral execution state | task, plan, hypotheses, touched files, command results, provider events | Episode-local observation only |

Precedence is not learned. Harp's supplement is user-level advisory context. It
cannot override a higher-priority provider rule or a repository contract. If a
learned item conflicts with `AGENTS.md`, the provider must follow
`AGENTS.md`; the episode should record the conflict as evidence against the
item.

### Component model

#### Workflow registry

The registry loads structured workflow packages from
`content/context_control/workflows/`. Technical prose remains in `knowledge/`;
the runtime packages contain JSON contracts and JSONL playbook items only.

The four initial packages are:

| Workflow | Positive boundary | Required negative boundary |
|---|---|---|
| `ci_repair` | A reproducible build, test, lint, type-check, packaging, or CI failure is present. | Do not activate for feature work without a failing verification signal. |
| `code_review` | The requested outcome is findings about an existing change or revision. | Do not silently become a patch-authoring workflow. |
| `dependency_update` | A dependency, lockfile, toolchain, or compatibility update is requested. | Do not broaden into unrelated modernization. |
| `general_coding` | A bounded repository change does not match a more specific package. | It is a fallback, not a catch-all source of high-priority policy. |

`--workflow auto` runs deterministic routing over the request and bounded
repository metadata. An explicit valid workflow always wins. Ambiguity falls
back to `general_coding`; it does not trigger a second model call. The router
emits considered workflows, scores, matched rules, rejection reasons, and the
selected workflow.

A package contains:

```text
content/context_control/workflows/<workflow>/
├── manifest.json
├── routing.json
├── context_schema.json
├── verification.json
├── outcome_rules.json
└── playbook.jsonl
```

The manifest declares schema version, package identity, supported task tags,
context budget, compatible selector and renderer versions, and the exact
digests of package members. Routing, verification, and outcome rules are data,
not executable scripts. Generated query-time code is deferred until Harp has a
separate read-only, network-disabled, subprocess-disabled execution boundary.

#### Release compiler

A context release is the immutable, content-addressed combination of:

- workflow package versions;
- approved playbook item versions;
- selector and renderer versions;
- context schemas and token budgets; and
- provider/model compatibility constraints.

The release ID is the SHA-256 digest of canonical identity JSON. Creation time,
local display names, and publication receipts do not participate in identity.
The target repository revision also does not participate: the episode pins the
release and repository snapshot as separate facts, avoiding a new release for
every commit.

Published release directories are immutable. A change creates a new release;
it never edits the previous directory. V1 ships one baseline release compiled
from the four human-reviewed workflow packages and no learned additions.

#### Context resolver

The resolver is a pure bounded function:

```text
resolve(task, repository_snapshot, workflow, release, budget)
  -> ContextBundle
```

`ContextBundle` contains:

```text
release_id
workflow
repository_invariants
workflow_steps
relevant_patterns
anti_patterns
verification_expectations
selected_item_ids
rejected_item_ids_and_reasons
routing_trace
estimated_tokens
rendered_context_sha256
```

Selection is deterministic, read-only, network-free, and subprocess-free.
Identical inputs produce identical bundle bytes. The renderer uses stable
ordering and bounded output. It may omit low-value advice to satisfy the budget
but may not omit protected repository instructions because those instructions
remain provider-owned context, not Harp playbook items.

#### Provider adapters

`TraeExecAdapter` and `CodexExecAdapter` implement one interface:

```text
probe() -> ProviderCapabilities
prepare(RunRequest) -> ProviderInvocation
execute(ProviderInvocation, EventSink) -> ProviderExit
```

V1 invokes `traecli exec --json` or `codex exec --json`, streams provider output
to the terminal as the providers emit it, and writes the original JSONL stdout
bytes, stderr bytes, exit status, and final-message file to the private episode.
Harp does not reinterpret JSONL as a polished interactive transcript in V1.
Machine-readable Harp status goes to stderr during a run so captured provider
stdout remains byte-exact. It does not use `--ephemeral` because provider
session persistence and the exposed session identity are useful evidence.

The adapter preserves native user configuration and repository rules. It does
not add `--ignore-user-config`, `--ignore-rules`, a more permissive sandbox, or
a weaker approval policy. Common typed options such as model, profile, sandbox,
and approval policy may be passed explicitly and translated per provider.
Arbitrary native-argument passthrough is excluded from V1 because it makes
security-sensitive conflicts and manifest reconstruction ambiguous.

The command delimiter belongs to the task:

```text
harp run [Harp options] -- <task words...>
```

Arguments after `--` are joined as the user task; they are not provider flags.
Harp constructs one byte-stable initial request with separate delimited
sections for the context release and the original user task. The request
records the supplement digest and task digest. The full task remains private
episode data because it may contain proprietary or sensitive material.

Provider-native review commands, hooks, skills, and App Server events may later
enrich an episode, but they cannot replace the provider-neutral episode
contract.

#### Episode normalizer

The normalizer converts known provider events into a versioned Harp event
vocabulary without deleting the raw event:

```text
run_started
assistant_message
tool_requested
tool_completed
verification_observed
file_change_observed
approval_observed
subagent_observed
compaction_observed
run_completed
provider_event_unknown
```

Every normalized record carries a monotonic episode sequence, provider
identity, raw-line digest, and provider event type. Unknown or malformed lines
are retained as opaque records and mark normalization as degraded; they are not
silently dropped. Provider timestamps remain evidence fields, while Harp's
sequence is the ordering authority.

The wrapper can prove only the context it injected and the repository metadata
it read. Until a provider exposes the fully assembled prompt, skill activation,
and compaction inputs, the episode declares
`context_manifest_completeness: partial`. This prevents wrapper telemetry from
masquerading as complete causal attribution.

#### Outcome labeler

Automatic labels are narrow facts:

- provider process exit status;
- whether event capture completed;
- pre-run and post-run Git HEAD;
- tracked-change and status digests;
- whether a verification command was observed and its recorded exit status;
- whether the provider requested approval; and
- whether a policy or archive-integrity check failed.

An observed zero exit code is not automatically “the requirement passed.”
Agent self-reports are weak evidence. Stronger outcomes arrive through explicit
user labels or attached verifier reports:

```text
accepted
accepted_with_substantive_changes
rejected
reverted
ci_passed
ci_failed
review_confirmed
review_refuted
environment_blocked
```

Attachments are schema-validated, content-addressed, and linked rather than
merged into historical events.

#### Candidate workshop

The workshop operates offline over sanitized episode exports. ACE emits typed
playbook operations:

```text
ADD
UPDATE
ADD_EXCEPTION
MERGE
DEPRECATE
```

MCE emits a complete workflow-package candidate: routing changes, schema,
playbook representation, selection policy, token allocation, verification
requirements, or an exportable provider skill. Generated `SKILL.md` files are
candidate artifacts under Harp state, not a second prose authority inside this
repository.

The candidate generator may use either provider through an explicit optimizer
adapter and strict output schema. It runs outside the target repository in a
bounded workspace. It receives sanitized evidence, cannot read raw archives or
hidden evaluation, and cannot write an active release.

### Context and episode manifests

The run manifest pins the execution tuple:

```json
{
  "schema_version": "harp-episode/v1",
  "episode_id": "ep-...",
  "repository_id": "sha256:...",
  "repository_head": "abc123...",
  "repository_state_sha256": "sha256:...",
  "provider": "trae",
  "provider_version": "0.200.19",
  "provider_capabilities_sha256": "sha256:...",
  "workflow": "ci_repair",
  "release_id": "sha256:...",
  "context_bundle_sha256": "sha256:...",
  "context_manifest_completeness": "partial",
  "configuration_mode": "native-defaults",
  "status": "running"
}
```

Before provider launch, Harp durably publishes the running manifest and exact
context bundle. Completion adds a separate receipt rather than rewriting
identity fields. A provider crash, signal, malformed stream, or local capture
failure leaves an inspectable incomplete episode.

Repository identity is local and non-exportable by default. It is derived from
the Git common directory so worktrees share one local episode namespace. A
sanitized export replaces local paths and remote names with opaque digests.
Repository state records HEAD plus digests and counts for status and tracked
changes; it does not retain patch bytes or untracked file contents unless a
future explicit capture mode is selected.

### Local storage

State-root precedence is:

1. explicit `HARP_HOME`;
2. `$XDG_STATE_HOME/harp`; then
3. `~/.local/state/harp`.

The layout is:

```text
${HARP_HOME}/
├── repositories/<repository_id>/
│   ├── episodes/<episode_id>/
│   │   ├── manifest.json
│   │   ├── context.json
│   │   ├── normalized.jsonl
│   │   ├── outcome.json
│   │   ├── completion.json
│   │   └── raw/<provider>/
│   ├── candidates/
│   └── exports/
├── releases/sha256-<digest>/
│   ├── identity.json
│   ├── manifest.json
│   ├── context_template.json
│   └── items.jsonl
└── registry/
```

The state root and episode directories are owner-only. Harp rejects symlinked
roots, non-directory ancestors, unsafe ownership or permissions, path
traversal, non-regular files, and replacement races. Publication uses held
directory handles, temporary regular files, synchronization, atomic rename,
and post-lock identity checks. Existing release files are never replaced.

Raw provider streams may contain credentials or proprietary text. Harp never
records environment values or provider auth files. Raw bytes are preserved
unchanged in private state and sealed with a deterministic member inventory.
Sanitization creates a derivative export; it never rewrites the primary
evidence. Secret detection or uncertain redaction blocks learning export while
leaving the private raw episode inspectable.

### Run lifecycle

`harp run` follows seven phases:

1. **Preflight.** Resolve Git identity, inspect state, probe provider
   capability, validate typed options, and establish secure episode storage.
2. **Route.** Select one workflow and preserve the full routing trace.
3. **Pin.** Resolve a compatible immutable release and publish the running
   manifest.
4. **Build context.** Select and render a bounded deterministic bundle.
5. **Execute.** Launch the provider, forward signals and output, and capture raw
   events without changing the provider's authority.
6. **Normalize and label.** Produce typed events, Git-state receipts, and
   narrow automatic outcome facts.
7. **Seal.** Publish the completion receipt and make the episode eligible for
   explicit sanitization and later suggestion work.

Release changes are forbidden during a run. A future Harp-managed resume uses
the original release by default and records an explicit old/new migration
receipt if the user chooses to upgrade. A session resumed directly through the
provider bypasses that guarantee and must not be represented as a
Harp-controlled continuation.

### User-facing commands

The intended surface is:

```text
harp run --provider trae --workflow auto -- <task>
harp run --provider codex --workflow code_review -- <task>

harp providers doctor

harp episodes list
harp episodes show <episode_id>
harp episodes label <episode_id> --outcome accepted
harp episodes attach <episode_id> --verification <report.json>
harp episodes export <episode_id> --sanitized

harp releases list
harp releases inspect <release_id>
harp releases diff <old_release> <new_release>

harp suggest ace --provider trae|codex --workflow <workflow>
harp suggest mce --provider trae|codex --workflow all
harp candidates list
harp candidates inspect <candidate_id>
```

Only provider doctor, baseline release inspection, observation runs, and
episode accounting belong to the first implementation slices. Suggestion
commands arrive after sanitized export and deterministic episode validation.
Promotion, shadow, canary, and rollback states are modeled but have no V1
mutation commands.

### Candidate and release lifecycle

```text
DRAFT
  -> STATIC_VALIDATED
  -> REPLAY_EVALUATED
  -> HUMAN_REVIEWED
  -> SHADOW
  -> CANARY
  -> PROMOTED
  -> DEPRECATED | ROLLED_BACK
```

V1 stops at `HUMAN_REVIEWED`. Later promotion requires:

- zero safety, authority, and evaluator-integrity failures;
- deterministic runtime selection;
- bounded discovery and selected-context budgets;
- paired replay against the active release;
- no unacceptable workflow-slice regression;
- a statistically credible correctness gain or meaningful cost reduction;
- provider/model compatibility evidence; and
- an immutable rollback target.

Safety and hidden-evaluation integrity are hard gates, not weighted reward
terms.

### Workflow evaluation contracts

| Workflow | Primary outcome | Reward-hacking checks |
|---|---|---|
| `ci_repair` | Required verification passes and the repair is accepted. | Test deletion, assertion weakening, blanket skipping, local-only substitution, and unrelated retries. |
| `code_review` | Findings are confirmed, precise, non-duplicative, and severity-calibrated. | Invented defects, style-only inflation, repeated findings, and unverified certainty. |
| `dependency_update` | Build and tests pass with a correct lockfile and bounded compatibility change. | Broad upgrades, regenerated noise, disabled checks, and undocumented compatibility loss. |
| `general_coding` | Requirements and repository verification pass without regression. | Scope expansion, superficial command success, and unverified completion claims. |

ACE evaluation compares the active release with one typed operation or bounded
operation cluster removed. MCE must additionally prove workflow-routing
quality, context-budget compliance, deterministic selection, and absence of
runtime side effects. Correlational Reflector praise cannot independently
promote an item.

### Failure behavior

- Missing or incompatible provider capability fails before launch.
- Release or context validation failure fails before launch and publishes no
  provider process; the failed preflight record is retained separately from
  episodes.
- Provider nonzero exit, crash, or signal produces an incomplete or failed
  completion receipt without changing the release.
- Malformed or unknown provider events are retained and mark normalization
  degraded.
- If capture storage fails after provider launch, Harp warns and excludes the
  episode from learning. It does not automatically kill a provider that may
  already be making useful repository changes.
- A failed secret scan blocks sanitized export; it never destroys or rewrites
  raw evidence.
- A stale or incompatible workflow package cannot be selected.
- Conflicting context items fail static validation or are emitted as a
  candidate conflict, not resolved silently.

### Verification strategy

The production subsystem requires:

- schema and canonical-byte tests for every manifest and receipt;
- symlink, ownership, permission, traversal, lock, and replacement-race tests;
- golden routing tests across all built-in workflows and ambiguous tasks;
- token-budget, stable-ordering, and no-side-effect tests for context
  resolution;
- adapter tests against fake provider binaries and captured JSONL fixtures;
- parity tests proving both providers receive the same release and rendered
  context for the same normalized request;
- unknown-event and malformed-stream preservation tests;
- signal-forwarding, nonzero-exit, interruption, and partial-capture tests;
- secret-scan and sanitized-export adversarial tests;
- outcome tests that reject assistant self-report as verified success;
- replay tests that isolate context-item and router effects; and
- compatibility tests for every explicitly supported provider version.

`mise run verify` remains provider-neutral and offline. It uses fake binaries
and fixtures, never a paid or authenticated live agent invocation. Explicit
live smoke tests remain opt-in and produce private local episodes.

### Staged delivery

1. **Foundation:** typed workflow, release, context, provider, episode, outcome,
   and candidate contracts; secure local state; baseline release compiler.
2. **Provider observation:** `providers doctor` and `harp run` through both
   `exec --json` adapters with raw capture and terminal streaming.
3. **Workflow context:** deterministic routing and context resolution for all
   four built-in workflows.
4. **Outcome accounting:** episode inspection, labels, verification
   attachments, Git-state receipts, and sanitized export.
5. **ACE suggestion:** batch reflection, typed curation operations, conflict
   analysis, and paired replay.
6. **MCE suggestion:** reviewable workflow-package and exported-skill
   candidates with static and replay validation.
7. **Deep adapters and promotion:** App Server telemetry, Harp-managed resume,
   compaction and subagent manifests, shadowing, canarying, signed releases, and
   rollback.

The first useful implementation slice is stages 1 through 3. It must support
both providers and all four workflow packages before optimizer work begins.

### Acceptance criteria

The observation subsystem is complete when:

- the same Harp release can launch either provider without changing provider
  security or repository contracts;
- every run durably pins provider version, repository state, workflow, release,
  context bundle, command, and manifest-completeness level before inference;
- raw provider output round-trips byte-for-byte through the private archive;
- known events normalize deterministically and unknown events remain
  inspectable;
- all built-in workflows route through data packages rather than hard-coded
  provider branches;
- no active release changes during an episode;
- interrupted and failed runs remain valid inspectable evidence;
- learning export fails closed on secrets or provenance gaps; and
- the complete offline Harp gate passes without either provider binary,
  credentials, another checkout, or network access.

### Explicitly deferred

- complete effective-prompt reconstruction where providers do not expose it;
- automatic `AGENTS.md`, skill, hook, or provider-configuration edits;
- model calls on the request critical path for context routing;
- arbitrary provider-argument passthrough;
- generated runtime selector code;
- cloud synchronization or a multi-tenant service;
- automatic promotion or mid-thread release upgrades;
- benchmark or hidden-evaluation access by candidate generators; and
- claims of recursive successor improvement.

## Reading routes

- [[knowledge/harness_benchmarks/harness_benchmark_field_guide|Harness benchmark field guide]]
- [[knowledge/rsi/systems/meta-harness|Meta-Harness system reading]]
- [[knowledge/rsi/context_engineering_deep_dive|Context-engineering deep dive]]
- [[knowledge/rsi/evaluator_integrity_and_promotion|Evaluator integrity and promotion]]
- [[knowledge/rsi/source_registry|Source registry guide]]
- [Original paper](https://arxiv.org/abs/2603.28052)
- [First-party project page](https://yoonholee.com/meta-harness/)
- Pinned implementation capture:
  `evidence/implementations/meta_harness/snapshot/README.md`.
- Local run receipt: `evidence/meta_harness/trae_run/receipt.json`.

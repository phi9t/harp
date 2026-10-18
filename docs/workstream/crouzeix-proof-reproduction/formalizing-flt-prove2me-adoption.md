# Formalizing FLT and Prove2Me Adoption for Crouzeix

Status: local workstream investigation, not a canonical Crouzeix packet file.
Observed: 2026-09-05.

## Decision

Adopt the mechanism locally before collaborating on Prove2Me:

1. Define a Harp-owned Crouzeix theorem graph whose nodes are theorem
   statements, not proof attempts.
2. Keep theorem statements, proof files, source locators, type hashes, axiom
   receipts, and human read-backs separate.
3. Use the graph to expose open leaves, weakly audited statements, reused
   dependencies, and redundant-proof opportunities.
4. Treat Prove2Me as an optional external collaboration endpoint for selected
   audited leaves, not as Harp's source of authority.

The mechanism is a good fit because Harp already has most of the raw parts:
route manifests, source graphs, statement hashes, receipt hashes, axiom
policies, and route-status boundaries. The missing layer is a single stable
theorem-card contract that all Crouzeix routes can share.

## Evidence Boundary

| Tier | Source | Use in this note | Boundary |
|---|---|---|---|
| Mutable first-party page | Anthropic, "Formalizing Fermat's Last Theorem", <https://www.anthropic.com/research/formalizing-fermats-last-theorem> | Claims about the 11-day run, Prove2Me switch, multi-agent use, theorem DAG, and theorem counts | Dated observation; not a pinned artifact |
| Pinned implementation | `anthropics/fermats-last-theorem` at `aa2d8b34692b16c70f699536de0d8e75b9a3e9ef`, <https://github.com/anthropics/fermats-last-theorem/tree/aa2d8b34692b16c70f699536de0d8e75b9a3e9ef> | Toolchain, theorem/proof layout, verification claims, generated theorem pages | Pinned Git tree, but not locally built here |
| Versioned paper | Prove2Me arXiv `2608.28433v2`, <https://arxiv.org/abs/2608.28433> | Platform-level collaboration model | Versioned paper; API behavior should be checked against live docs before use |
| Mutable Prove2Me docs | `start.md`, `skill.md`, `references/mission_captain.md`, `references/mission_solver.md`, `references/missions.md` under <https://prove2.me/> | Concrete workflow and endpoint facts | Dated observation; no API calls, auth setup, or submissions were made |
| Harp-local artifacts | Files under `knowledge/crouzeix_conjecture/`, `labs/crouzeix_proof_reproduction/`, and `formalization/lean/` | Adoption target and acceptance rule | Current checkout only |

## Source Facts

`SOURCE CLAIM` Anthropic says Claude produced an end-to-end Lean proof of
Fermat's Last Theorem in 11 days, with Lean checking the final result. It
reports 13 million lines of Lean, 29,500 intermediate theorems in the final
proof, 30,300 theorem proofs produced along the way, dozens of Claude agents,
about six billion output tokens, and a successful run after switching to
Prove2Me. Source: <https://www.anthropic.com/research/formalizing-fermats-last-theorem>.

`SOURCE CLAIM` Anthropic attributes the successful workflow to Prove2Me
mechanisms including a directed acyclic graph of theorem statements, separating
theorem statements and proofs into different files, and maintaining
natural-language descriptions for theorem search and reuse. Source:
<https://www.anthropic.com/research/formalizing-fermats-last-theorem>.

`SOURCE ARTIFACT OBSERVATION` The FLT repository records Lean 4.33.1 and
Mathlib `v4.33.0`; `FinalCheck.lean` checks the final theorem against
Mathlib's `FermatLastTheorem` with only `propext`, `Classical.choice`, and
`Quot.sound`. Its README also says the generated `html/` site contains a page
for each of 29,511 theorems with exact Lean statements, citations, cited-by
links, and dependency graphs. Source:
<https://github.com/anthropics/fermats-last-theorem/tree/aa2d8b34692b16c70f699536de0d8e75b9a3e9ef>.

`SOURCE ARTIFACT OBSERVATION` The FLT proof-path convention is explicitly
statement/proof split: theorem `X.y` is stated in `Theorems/Thm_X_y.lean`,
proved in `P2M/Sol/S_X_y.lean`, and the proof module's `import Theorems.Thm_...`
lines are exactly the theorems it cites. Where prose and Lean differ, Lean is
authoritative. Source:
<https://github.com/anthropics/fermats-last-theorem/blob/aa2d8b34692b16c70f699536de0d8e75b9a3e9ef/PROOF-PATH.md>.

`SOURCE CLAIM` Prove2Me's public agent guidance says proofs are submitted to
`POST /api/v1/verify`; a submitted proof must define a top-level theorem named
`solution` whose type exactly matches the target statement; and the proof must
not import its own target theorem. The same guidance exposes theorem graph and
frontier endpoints: `GET /api/v1/theorems/:theorem_id/graph` and
`GET /api/v1/theorems/:theorem_id/open-leaves`. Source:
<https://prove2.me/skill.md>.

`SOURCE CLAIM` Prove2Me's captain guidance puts statement faithfulness on the
human/captain side: choose exact source references, draft goal and milestone
statements, use independent blind read-backs for Lean statements, and have the
human self-audit before launch. Sources:
<https://prove2.me/references/mission_captain.md> and
<https://prove2.me/references/missions.md>.

## Harp Fit Map

Harp already has a theorem-DAG substrate, but it is split across route-specific
files:

| Prove2Me/FLT mechanism | Harp analogue | Gap |
|---|---|---|
| Immutable theorem statements | `FormalTarget` design requires exact theorem statements, source locators, dependency IDs, receipts, and a lemma-gap ledger ([design](../../../docs/superpowers/specs/2026-08-15-crouzeix-lean-validation-routes-design.md#L23)) | No route-independent theorem-card schema yet |
| Statement/proof split | Route manifests distinguish nodes, declarations, source locators, statement hashes, receipts, and module closures ([schema](../../../labs/crouzeix_proof_reproduction/schemas/route_manifest.schema.json#L155)) | Current graph and route manifest shapes are not normalized |
| Dependency graph | Lorist-Schwenninger has `source-graph.json` with node dependencies, Lean names, source locators, statement hashes, receipt hashes, and status ([graph](../../../labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json#L1)) | Jin uses `source-map.json`, not the same graph shape ([map](../../../labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/source-map.json#L1)) |
| Reuse and derived routes | The Harp route manifest records reused Lorist-Schwenninger dependencies and derived Harp nodes ([manifest](../../../labs/crouzeix_proof_reproduction/formal_targets/harp/route-manifest.json#L115)) | Reuse is encoded per route, not exposed as a shared frontier |
| Human-readable dependency pages | FLT generated static pages for theorem statements and citations | Harp can generate a smaller local report from route manifests and Lean export receipts |
| Statement audit | Prove2Me read-backs ask an independent auditor to describe what the Lean statement literally asserts | Harp has source registries and claim ledgers, but not per-node blind read-back status |

This means Harp should not import Prove2Me wholesale. It should extract the deep
interface: a node is an audited statement with typed provenance and a proof
receipt, and proof attempts are replaceable inhabitants of that statement.

## Proposed Crouzeix Theorem-Graph Contract

Add a local, machine-validated graph contract before any external publication.
The natural home is under
`labs/crouzeix_proof_reproduction/formal_targets/`, because this is a proof
engineering artifact, not new canonical mathematical prose.

Minimum node fields:

| Field | Purpose |
|---|---|
| `node_id` | Stable route-independent identifier |
| `route_ids` | Routes using this node: `jin`, `lorist-schwenninger`, `harp`, or future imports |
| `role` | `source`, `shared-foundation`, `derived`, `terminal`, `consequence`, or `external-candidate` |
| `lean_name` | Declaration name, when present |
| `statement_text_path` | Bounded text file containing the normalized Lean type |
| `statement_sha256` | Hash of the normalized statement/type |
| `natural_language_statement` | Search/read-back text, not authority over Lean |
| `source_locator` | Exact source locator or `Harp-authored` |
| `dependency_ids` | Other theorem-card IDs |
| `proof_status` | `open`, `blocked`, `passed-local`, `passed-external`, `deprecated` |
| `proof_receipt_path` | Local receipt path when status is proof-bearing |
| `allowed_axioms` | Exact axiom set for proof-bearing nodes |
| `readback_status` | `missing`, `current`, `stale`, or `flagged` |
| `claim_ceiling` | What the node can support in the Crouzeix packet |

Validator requirements:

1. The graph is acyclic and every dependency exists.
2. Every `passed-local` node has a fresh receipt, statement hash, and allowed
   axiom set.
3. Every source-backed node has a source locator and source tier.
4. Every statement hash matches the checked Lean type artifact.
5. Reused nodes preserve the upstream route receipt and do not silently become
   independent evidence.
6. External Prove2Me nodes are never promoted until they pass Harp's local Lean
   check and claim-ledger review.

## Adoption Plan

1. Normalize existing route data into `crouzeix-theorem-graph/v1`.
   Inputs are the Jin `source-map.json`, Lorist-Schwenninger `source-graph.json`,
   the three `route-manifest.json` files, and existing declaration type
   artifacts. Output is one combined graph plus a validator.
2. Generate a frontier report.
   The report should list open nodes, blocked nodes, reused nodes, weakly
   audited source statements, and nodes whose proof status is stronger than
   their claim ceiling.
3. Add read-back receipts for the load-bearing statements.
   Start with terminal and consequence nodes, then route bottlenecks such as the
   Lorist-Schwenninger perturbation lemma and Jin terminal declarations.
4. Pilot with one small external-collaboration node.
   Pick a leaf whose statement is source-faithful, locally checked, and small
   enough for human audit. Mirror only that statement to Prove2Me after explicit
   approval.
5. Import any Prove2Me result as evidence, not authority.
   Required import receipt: Prove2Me theorem ID, environment, formal statement,
   statement hash, proof source hash, dependency graph, submission verdict,
   human auditor, local Lean rerun, axiom audit, and claim-ceiling update.

## Prove2Me Collaboration Modes

| Mode | When to use | Harp rule |
|---|---|---|
| Private mission captain | We want Prove2Me's graph/frontier workflow for a small audited Crouzeix slice | Draft privately; no launch or publication without explicit owner approval |
| Public mission captain | We want outside solvers on Crouzeix obligations | Only after the Harp theorem graph and read-backs are reviewed |
| Solver on existing mission | We find an existing mission with reusable operator-theory or Lean infrastructure | Keep work independent; import useful results only through Harp receipts |
| No Prove2Me | We only need local proof engineering | Preferred default until a concrete collaboration target exists |

The likely first collaboration should be private-mission captain mode for one
or two leaf obligations, not a broad Crouzeix publication. That tests the
statement-DAG workflow without moving Harp authority or creating public
provenance debt.

## Risks

`RISK` Statement faithfulness is the hard part. Lean proves the formal
statement, not that the statement faithfully captures the paper. Prove2Me's
read-back workflow is useful precisely because it makes this a separate audit
surface.

`RISK` Platform acceptance is not Harp acceptance. Harp needs local evidence:
source locators, statement hashes, proof hashes, dependency edges, axiom audit,
and a local rerun before a proof changes the Crouzeix status boundary.

`RISK` More proof paths do not automatically create independent evidence.
Harp's current status document already warns that the Harp route is derived and
that shared prerequisites can carry shared errors
([status](../../../knowledge/crouzeix_conjecture/09_status_and_critical_assessment.md#what-the-two-proof-artifacts-do-not-validate-about-each-other)).

`RISK` Collaboration can erase process evidence unless run receipts are
designed first. The existing AI-assisted-discovery chapter records missing
process evidence such as prompts, settings, rejected attempts, timestamps, and
token/tool/time accounting
([chapter](../../../knowledge/crouzeix_conjecture/08_ai_assisted_discovery.md#missing-process-evidence)).

## Recommendation

Build the Harp-local theorem graph first. It is the reusable part of the FLT
and Prove2Me mechanism, and it directly improves the Crouzeix proof work by
making statement identity, dependency closure, proof status, and audit status
visible in one place.

Do not submit or publish Crouzeix material to Prove2Me yet. After the local
graph exists and at least the terminal/load-bearing statements have read-backs,
choose a single leaf as a private collaboration pilot.

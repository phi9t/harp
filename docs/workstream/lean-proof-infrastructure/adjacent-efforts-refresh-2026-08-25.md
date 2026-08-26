# Adjacent-effort refresh, 2026-08-25

Status: research input, not an accepted architecture

Base snapshot: [`research.md`](research.md), dated 2026-08-23

## Scope and evidence boundary

This refresh reuses exactly the precommitted adjacent-effort cohort and primary
artifact ledger in the base snapshot: 24 systems across six strata and 38
artifacts. The artifact accounting boundary is 24 Git repositories, one
Hugging Face dataset revision, and 13 versioned papers or publication records.
No system was added.

Repository identities were resolved from each repository's advertised default
ref with `git ls-remote`; stable release identities were also resolved where
the ledger used a release. The dataset identity came from the first-party
Hugging Face API. Fixed arXiv versions were checked against the first-party
arXiv feed. Detailed inspection was limited to the six systems whose source
identity changed relative to the ledger, using GitHub source, comparisons,
release notes, and commit records. No third-party summaries were used and no
link was followed more than one hop.

"Changed" below means changed relative to the frozen ledger identity. It does
not necessarily mean that the upstream event occurred after the calendar date
printed on the snapshot: the snapshot has no time-of-day, and three newer
artifacts were already public before 2026-08-23. A default branch and a stable
release are recorded separately when they are different channels.

Evidence classes remain separate:

- an immutable commit or release supports what that source contains;
- an accepted upstream plan supports its declared scope, not successful
  deployment;
- a dated API/ref observation supports only the identity observed on
  2026-08-25;
- architectural consequences below are researcher inferences;
- `MISSING` and `unresolved` mean the capped primary corpus did not support a
  stronger conclusion.

## Cohort and version delta

Summary: 6 systems changed, 17 were unchanged, and 1 remains unresolved. At
the artifact level, seven repository entries changed because LeanEval owns two
repositories in the ledger.

| System | 2026-08-23 ledger identity | 2026-08-25 observation | Classification | Harp-v1 materiality |
|---|---|---|---|---|
| Lean 4, Lake, official server | [`v4.33.0` / `d8b1897`](https://github.com/leanprover/lean4/tree/d8b18978322de05a8f3dba51ef03cf5461676c17) | Stable [`v4.33.1` / `819816b`](https://github.com/leanprover/lean4/tree/819816b2e0a3bf405af45ae5c7af2491d8f5bee6); default `master` [`b9c9eb9`](https://github.com/leanprover/lean4/tree/b9c9eb9a3acefaa242c1da2a31e3723e94fde3a5) | Changed | **Yes:** kernel-vulnerability and replay policy |
| elan | [`v4.2.3` / `b6cec7e`](https://github.com/leanprover/elan/tree/b6cec7e10fe4965a605aaf60d1cb4a5837f0462b) | [`v4.2.4` / `227caca`](https://github.com/leanprover/elan/tree/227caca133724d5516bee25c2aeb3e609478f2d8) | Changed | Operational postcondition only |
| mathlib4 and cache | [`v4.32.1` / `520045a`](https://github.com/leanprover-community/mathlib4/tree/520045ab14e26149ee970e2e617ca04b09bde5d6) | Stable [`v4.33.1` / `0df444a`](https://github.com/leanprover-community/mathlib4/tree/0df444a360eaa60ab8c11dca51a86af692955474); default `master` [`bbc52f9`](https://github.com/leanprover-community/mathlib4/tree/bbc52f9636cb2200cf01ae3eb391953f2d05636c) | Changed | **Yes:** cache schema, artifact set, and trust provenance |
| Reservoir | [`4bde19c`](https://github.com/leanprover/reservoir/tree/4bde19c2dba9244d97336768ce8a97aaddbd2b72) | Same default-ref commit | Unchanged | None |
| Community REPL | [`5d5c49d`](https://github.com/leanprover-community/repl/tree/5d5c49d13dfc0c1d2df43a27c3e56e02ad81b9c3) | Same default-ref commit | Unchanged | None |
| Pantograph | [`dev` / `d704b85`](https://github.com/leanprover/Pantograph/tree/d704b851542b1d2caf1287f65c49f5011f687c05) | Same `dev` commit | Unchanged | None |
| LeanInteract | [`976edd7`](https://github.com/augustepoiroux/LeanInteract/tree/976edd7d38a99e1ea4c2dfabeb8ad98baffca3c8) | Same default-ref commit | Unchanged | None |
| Lean4Kit description | [OpenReview `98w016SKJg`](https://openreview.net/forum?id=98w016SKJg) | Fixed record still challenge-gated; no implementation identity in the capped corpus | **Unresolved** | Preserve the prior `MISSING` result |
| Kimina Lean Server | [`fb2393d`](https://github.com/project-numina/kimina-lean-server/tree/fb2393de3461db35eda4c714e3fd21187e92ec90) | Same default-ref commit | Unchanged | None |
| LeanDojo-v2 | [`baed5ea`](https://github.com/lean-dojo/LeanDojo-v2/tree/baed5eae6e87a65a446d9f54af07aab2154e7599) plus content-hashed paper | Same repo commit and paper identity | Unchanged | None |
| doc-gen4 | [`97d4ecd`](https://github.com/leanprover/doc-gen4/tree/97d4ecdfc8e09e7f511724c25e303d448de6a3db) | Same default-ref commit | Unchanged | None |
| LeanSearch v2 | [`94f4888`](https://github.com/frenzymath/LeanSearch-v2/tree/94f4888cbaf9f4322535755f86cbac690ec18080), arXiv `2605.13137v2`, client [`ba67e21`](https://github.com/leanprover-community/LeanSearchClient/tree/ba67e212be1197b84c1f1f6299488a10a3002713) | Same repository, paper, and client identities | Unchanged | None |
| Lean Workbook | Dataset [`2e066e3`](https://huggingface.co/datasets/internlm/Lean-Workbook/tree/2e066e310b2c6d2c27616927ae131f82901c8f1c) plus NeurIPS 2024 record | Same dataset revision and publication record | Unchanged | None |
| ReProver | [`fd6d99c`](https://github.com/lean-dojo/ReProver/tree/fd6d99c01e8bd8fd8f3fd1de4cd0bc4a7f158eaa) plus NeurIPS 2023 record | Same identities | Unchanged | None |
| LeanCopilot | [`6073803`](https://github.com/lean-dojo/LeanCopilot/tree/60738032a1398a4e83524107424a11322a753609) plus PMLR 288 | Same identities | Unchanged | None |
| DeepSeek-Prover-V2 | [`e598a57`](https://github.com/deepseek-ai/DeepSeek-Prover-V2/tree/e598a57ea3284997d4a2a168a069fdd5064afbc8) plus arXiv `2504.21801v2` | Same identities | Unchanged | None |
| Goedel-Prover-V2 | [`2e9036e`](https://github.com/Goedel-LM/Goedel-Prover-V2/tree/2e9036e118464aa96a8bebaf9f5b9d091aa3585c) plus arXiv `2508.03613v1` | Same identities | Unchanged | None |
| Kimina-Prover Preview | [`7abd61a`](https://github.com/MoonshotAI/Kimina-Prover-Preview/tree/7abd61a5d9861bf5c15b195c216c1bb233ac32e4) plus arXiv `2504.11354v1` | Same identities | Unchanged | None |
| lean-eval | Harness [`b91d475`](https://github.com/leanprover/lean-eval/tree/b91d4757aa0d7776c02540c9089df54fa0d0658a), submissions [`90383f7`](https://github.com/leanprover/lean-eval-submissions/tree/90383f788caa07808e8a36ee457357b11bb02964) | Harness [`de48559`](https://github.com/leanprover/lean-eval/tree/de48559590eb8cff2125e8e933b3e13bb8a3ff98), submissions [`2bdeb2b`](https://github.com/leanprover/lean-eval-submissions/tree/2bdeb2be1d1cd504a567dd01a3b040ae6e341827) | Changed | **Yes:** bounded lifecycle, authority, and subtraction after overbuilding |
| miniF2F | [`e4f1130`](https://github.com/facebookresearch/miniF2F/tree/e4f113090ad82d64f8ce064d2f55b613a9b6bded) plus OpenReview `9ZPegFuFTFv` | Same source commit; live OpenReview metadata was challenge-gated | Unchanged source; record metadata unresolved | None |
| PutnamBench | [`dfb0a47`](https://github.com/trishullab/PutnamBench/tree/dfb0a47a1c1ec3a10f2a9acfdf41a2043920f33c) plus NeurIPS 2024 record | Same identities | Unchanged | None |
| Formal Conjectures | [`488aade`](https://github.com/google-deepmind/formal-conjectures/tree/488aade228ec37880b8fec178c173c07d279bb53) plus arXiv `2605.13171v1` | Source [`169abb8`](https://github.com/google-deepmind/formal-conjectures/tree/169abb8eeb725609230e597ffcd03abe10ea412d); paper still `v1` | Changed | **Yes:** correction lineage and external-proof status semantics |
| ProofNet | [`509ad79`](https://github.com/zhangir-azerbayev/ProofNet/tree/509ad79710ed4f46ff5c282ed5640c1aa9ac3f30) plus arXiv `2302.12433v1` | Same identities | Unchanged | None |
| Public Crouzeix snapshot | [`9df0783`](https://github.com/jinshanmu/CrouzeixConjecture/tree/9df07838327b988e3924453daa29c8cd726d34b0) | [`f9d5c8d`](https://github.com/jinshanmu/CrouzeixConjecture/tree/f9d5c8d39bece41ceedf6346ef50ad1fb393260e) | Changed | **Yes:** transcript provenance is now partially public |

## Material deltas

### Lean 4: a successful kernel check needs a vulnerability lifecycle

Lean `v4.33.1` is not an ordinary cosmetic patch. The official release notes
encourage users of `v4.33.0` and earlier to upgrade and enumerate runtime and
kernel defects that could support false proofs or unsound results: an official-
kernel reference-count overflow, problematic older GMP behavior, an `is_prop`
soundness issue, order-dependent `is_def_eq` caching, and added verification of
generated recursors. The notes also show that "independent kernel" is not one
binary property: nanoda caught some of the described exploits but accepted the
reported `is_prop` exploit. The immutable release source is
[`819816b`](https://github.com/leanprover/lean4/tree/819816b2e0a3bf405af45ae5c7af2491d8f5bee6),
and the owned explanation is the [Lean 4.33.1 release
note](https://lean-lang.org/doc/reference/stable/releases/v4.33.1/).

This changes both the trust policy and the environment identity. A receipt must
bind the exact kernel/runtime/toolchain and the certification policy that
approved it. A later soundness advisory needs a typed transition such as
`replay-required` or `superseded`, followed by certification under an approved
new checker set; it must not silently mutate the old receipt. Checker diversity
should be recorded per checker/version and interpreted per vulnerability, not
collapsed into an `independent = true` flag.

For the current Crouzeix campaign, this is not permission to update the pinned
toolchain or hydrate dependencies during route work. It adds a trust-policy
review before promotion and a replay path after promotion. Applicability of the
4.33.1 defects to Harp's pinned toolchain is `MISSING` from this refresh.

### mathlib cache: cache format and trust policy are versioned inputs

Between the frozen cache implementation and `v4.33.1`, mathlib increments its
root-hash generation, adds `.ir.sig` and `.ir.sig.hash` to the cached artifact
set, repairs decompression state across multi-container download rounds, and
changes the documented high-trust writer set to include `v4.*` release tags.
These changes are visible in the [exact release
comparison](https://github.com/leanprover-community/mathlib4/compare/520045ab14e26149ee970e2e617ca04b09bde5d6...0df444a360eaa60ab8c11dca51a86af692955474),
the pinned [`Cache/IO.lean`](https://github.com/leanprover-community/mathlib4/blob/0df444a360eaa60ab8c11dca51a86af692955474/Cache/IO.lean),
and the pinned [cache trust
policy](https://github.com/leanprover-community/mathlib4/blob/0df444a360eaa60ab8c11dca51a86af692955474/Cache/SECURITY.md).

A Harp cache reference should bind the cache-policy/hash-generation
identity and expected artifact kinds, not merely a directory path or Lake
manifest. Cache bytes remain an accelerator rather than proof authority. Harp's
existing no-hydration rule during proof iteration is still the right v1
boundary; acquisition and cache mutation remain a separately authorized
maintenance workflow whose postconditions are verified.

### elan: command success is not an environment observation

Elan `v4.2.4` makes installation of an already installed toolchain report
success and fixes a garbage-collection panic when a listed toolchain directory
is missing. The exact behavior change is in the [release
comparison](https://github.com/leanprover/elan/compare/b6cec7e10fe4965a605aaf60d1cb4a5837f0462b...227caca133724d5516bee25c2aeb3e609478f2d8)
and pinned [`CHANGELOG.md`](https://github.com/leanprover/elan/blob/227caca133724d5516bee25c2aeb3e609478f2d8/CHANGELOG.md).

This does not require a new module. It reinforces that a setup command's exit
status is not the environment identity: preflight must observe the resolved
toolchain and artifacts after any authorized setup. Proof work itself should
continue to perform read-only resolution and fail with a typed missing-cache or
missing-toolchain block.

### LeanEval: lifecycle value survived; a qualification control plane did not

LeanEval adopted a completion plan on 2026-08-25 after implementation
experience. It retains immutable submission snapshots, append-only lifecycle
state, exact historical source/toolchain pins, distinct terminal replay
outcomes, bounded recovery, and an explicit human production go/no-go. It also
separates a useful production launch from completion of historical replay. The
same plan explicitly removes Formal Conjectures integration, experimental
kernel promotion, a persistent model-identity qualification system, exhaustive
failure injection, contention matrices, and a dedicated qualification control
plane. It says to use the smallest existing state mechanism that satisfies an
actual atomicity requirement. These are declared scope decisions, not evidence
that production has launched; the plan says production capabilities were
disabled at adoption. See the pinned [completion
plan](https://github.com/leanprover/lean-eval/blob/de48559590eb8cff2125e8e933b3e13bb8a3ff98/docs/overhaul-completion-plan.md).

The companion implementation repository then removed the excluded machinery.
GitHub's accounting for that single cleanup commit is 52 additions and 77,527
deletions across 97 files, including persistent qualification workers,
experimental-kernel schemas/runners, workflows, fixtures, and accumulated
evidence narratives. The accounting boundary is the complete GitHub commit
stat, including generated declarations and retained evidence files; it is not
a hand-count of authored logic. See [cleanup commit
`2bdeb2b`](https://github.com/leanprover/lean-eval-submissions/commit/2bdeb2be1d1cd504a567dd01a3b040ae6e341827).

This delta changes v1 most directly. It supports the approved Harp-integrated,
Crouzeix-first boundary and argues against building a
second scheduler, a persistent evaluator-qualification service, or a general
Lean platform before the golden trace. Crash testing should cover the few
authority-bearing effect boundaries, not an exhaustive combinatorial matrix.
Current contracts and operating facts belong in maintained documentation;
attempt history belongs in immutable evidence rather than growing into a
parallel operating ledger.

### Formal Conjectures: theorem text alone is not a task identity

The changed Formal Conjectures source contains several unusually sharp
provenance examples:

- A sign correction to a dependency turns three statements that were
  refutable at `n = 0` into the intended identities. The three theorem
  statements themselves remain byte-identical while their status changes from
  open to solved and external proofs are linked. See [commit
  `1bd0e70`](https://github.com/google-deepmind/formal-conjectures/commit/1bd0e70d325bcae22edb8d77e946da79c4d0d378).
- Three declarations acquire `formal_proof` links while their in-repository
  statements and `sorry`s remain unchanged; the commit separately argues type
  correspondence and records the external toolchain and axiom audit. See
  [commit `a14a7a7`](https://github.com/google-deepmind/formal-conjectures/commit/a14a7a739a220899eaf0990f04977b4dc47a1e33).
- Other commits correct an inequality direction or replace an unknown
  `answer(sorry)` with `answer(False)` plus an external proof link. See the
  [direction correction](https://github.com/google-deepmind/formal-conjectures/commit/8ffb799cd90108d7600d9a26b2219fec2aef6946)
  and [Weak Tiling status change](https://github.com/google-deepmind/formal-conjectures/commit/de53b30d2e6b50bc18273344edaff99266deb886).

Harp's proof-task identity must include the transitive definition
closure, statement/correction lineage, artifact channel, and correspondence
obligation. A `solved` label or external proof URL is not a route receipt. This
reinforces the current Crouzeix route-manifest design: exact declaration types,
dependency graph, external artifact identity, axiom/import checks, and separate
source review all remain first-release gates.

### Public Crouzeix: a rich transcript is now public, but it is not a receipt ledger

The current public repository adds a privacy-filtered conversation record for
the proof-development task and persisted subagent branches. Its manifest
reports 13 threads, 916 agent-to-agent messages, 9,173 reasoning items with
readable summaries, 1,379 safe command call/result pairs, and a 5,541,995-byte
rendered conversation. It also reports exclusions: 1,621 reasoning items without a
public summary, encrypted continuations and messages, platform instructions,
protected tool records, and private orchestration arguments. See the pinned
[conversation manifest](https://github.com/jinshanmu/CrouzeixConjecture/blob/f9d5c8d39bece41ceedf6346ef50ad1fb393260e/conversation-019f7059-public/README.md)
and [rendered record](https://github.com/jinshanmu/CrouzeixConjecture/blob/f9d5c8d39bece41ceedf6346ef50ad1fb393260e/conversation-019f7059-public/conversation-and-reasoning.md).

The record adds substantial public provenance. It exposes many failed
routes, counterexample searches, agent messages, commands, a sampled goal
usage report, and the final proof argument. It does not expose a stable
model/provider/run identity, private orchestration arguments, every reasoning
item, a route registry, normalized attempt/result identities, or independent
certification and review receipts. It is ordered by logical task tree rather
than presented as a replayable event log.

The previous statement that worker transcripts and rejected attempts were
absent must be corrected to **partially present at the current commit**. The
architectural conclusion survives: conversation artifacts are valuable
context and audit evidence, but Harp must extract authority-bearing attempts,
nodes, observations, reviews, and receipts into versioned immutable contracts.

## Corrections to the 2026-08-23 snapshot

1. The Lean and mathlib ledger entries remain valid frozen comparison
   artifacts, but they were not the latest stable releases on the snapshot's
   calendar date. [Lean `v4.33.1`](https://github.com/leanprover/lean4/releases/tag/v4.33.1)
   and [mathlib `v4.33.1`](https://github.com/leanprover-community/mathlib4/releases/tag/v4.33.1)
   were published on 2026-08-21. Future ledgers should distinguish `selected frozen version`,
   `current stable release`, and `default-branch head` explicitly.
2. The [public Crouzeix conversation commit](https://github.com/jinshanmu/CrouzeixConjecture/commit/f9d5c8d39bece41ceedf6346ef50ad1fb393260e)
   is dated 2026-08-16, so it also predates the calendar snapshot. The earlier absence claim is true only for
   the pinned `9df0783` tree, not for the upstream default branch that already
   existed on 2026-08-23.
3. The Lean 4 and mathlib default branches are development channels distinct
   from their latest stable tags. Comparing a stable tag directly to current
   `master` can report divergence and unrelated development work; adoption
   decisions must compare like-for-like channels.
4. The [LeanEval submissions comparison](https://github.com/leanprover/lean-eval-submissions/compare/90383f788caa07808e8a36ee457357b11bb02964...2bdeb2be1d1cd504a567dd01a3b040ae6e341827)
   contains 246 commits, including
   substantial machinery later deleted by the current head. Commit history
   proves that work occurred; it does not prove that the machinery is a current
   product contract. The accepted completion plan and current tree define the
   dated state.
5. Formal Conjectures `formal_proof` metadata may point to a proof in another
   repository and toolchain while the catalog declaration still contains
   `sorry`. "Solved in the catalog" must not be read as "this repository's
   aggregate target contains the proof."

## Implications for Harp v1

The refresh changes several details of the provisional design and leaves its core
boundary intact.

### Required now

1. **Add a certification trust-policy identity.** Keep the exact Lean/runtime
   commit in `EnvironmentRef`, and bind every certification receipt to an
   approved checker-policy revision. Model later advisories as replay-required
   state, preserving old receipts rather than rewriting them.
2. **Keep CPFR-087 through CPFR-091 as the v1 acceptance trace.** Harp's durable
   engine remains the only scheduler and state authority. Do not add training,
   reinforcement learning, a standalone Lean service, or a persistent
   qualification control plane before the three-route evidence closure is
   complete.
3. **Bound crash testing to authority-bearing effects.** Exercise provider
   invocation reconciliation, create-only candidate publication,
   certification-receipt persistence, and atomic bundle promotion. Do not make
   an exhaustive failure/contention matrix a v1 completion condition.
4. **Make closure and correction lineage part of task identity.** Exact theorem
   bytes are insufficient. Bind imported definitions, route/source graph,
   formal-target lock, external artifact channel, and correspondence decision.
5. **Treat cache policy as a versioned accelerator contract.** Record the hash
   generation/policy identity, expected artifact set, cache ancestry, and
   provenance in preflight. Continue to prohibit acquisition during proof work
   and deny the cache any proof authority.
6. **Ingest transcripts as projections, not truth stores.** A transcript
   artifact may be content-addressed and linked to attempts, with an explicit
   exclusions/redaction manifest. Acceptance must depend on normalized
   candidate, environment, check, review, and promotion receipts instead.

### Milestone split

Use two explicit milestones rather than calling all future infrastructure v1:

- **Crouzeix v1 / complete-local:** the Jin, Lorist--Schwenninger, and Harp
  routes have complete fail-closed manifests and receipts, separate
  mathematical/correspondence review, and an explicit human promotion result.
- **Portable replay follow-on:** reproduce sealed receipts across supported
  machines, ingest historical attempt material, add vulnerability-triggered
  replay operations, and only then generalize adapters or dataset export.

### Unchanged conclusions

- Fast interaction and fresh certification remain separate lanes.
- The source-aware proof program remains the useful unit above tactic states.
- Harp's existing `TaskGraph` remains the macro executor; a Lean adapter owns
  only the micro-loop.
- Kernel acceptance, source correspondence, mathematical review, and human
  promotion remain separate authorities.
- A general LeanDojo successor, trainer, or benchmark platform remains outside
  the first product boundary.

## MISSING and uncertainty

- Lean4Kit's live OpenReview record was blocked by a challenge page on
  2026-08-25. The capped corpus still provides no public implementation,
  protocol schema, release, or compatibility matrix.
- The miniF2F repository identity is unchanged, but its live OpenReview record
  was also challenge-gated. No publication-metadata delta is claimed.
- This refresh inspected the mathlib cache contract, not the mathematical
  source delta across the full release upgrade. Any Harp mathlib/toolchain
  migration needs its own source, declaration-type, import, and proof replay
  audit.
- Lean `master` and mathlib `master` were resolved but not treated as adoption
  targets. Their development-branch changes were not synthesized into v1.
- The LeanEval completion plan is an accepted first-party scope document while
  production capabilities were disabled. It does not establish operational
  success, latency, recovery rates, or security effectiveness.
- The current Crouzeix transcript is a curated public projection. Its hidden
  and excluded content cannot support model independence, chronological replay,
  or complete budget accounting.
- No fresh Harp Lean build, cache mutation, toolchain migration, axiom audit,
  or route certification was run. Whether any Lean 4.33.1 advisory affects the
  exact Harp toolchain is not established here.

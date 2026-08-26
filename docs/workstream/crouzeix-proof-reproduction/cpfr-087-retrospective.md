# CPFR-087 retrospective

## Outcome

CPFR-087 certified the Harp finite-horizon proof route as
`complete-local`. This is a route-local result, not a whole-program completion
claim. The [execution ledger](execution-ledger-003.tsv) records CPFR-087 through
`release-verified`; the [tracker](tracker.org) still marks the ticket
`IMPLEMENTING`. CPFR-088 and later phases, full-repository `mise run verify`,
the final `docs/import-receipt.md` refresh, local landing, and Kata closure all
remain pending at the time of this note.

The route candidate is commit
`909b7813cb7c7c164345b265fcca11691670edb4`, tree
`b9724fd0a3e84eb47c4f9b8cdcb5141f6abddf17`. It contains 24 proof-graph
nodes, 35 shared-foundation modules, a 59-module active closure, and exactly 11
lower-level Lorist--Schwenninger support-module reuses. The final manifest raw
SHA-256 is
`4be259ffa3056d0baae71aacae975f19d11c341686c6da9c2257cfbfea5a96fe`;
its publication-stable contract SHA-256 is
`281b2b58e0fad6c0f0fd586b0a0192e6fcca7a42433718f1eeb46143faa67e17`.
The closure SHA-256 is
`d0901748dd51b8850603918f528b2da2feb7b3fe653c7f5be77423f40b5ab137`,
and the terminal declaration-type SHA-256 is
`d2a15e37b9f4b0b2dca077aa6b55e71e5819eb7b81fc7aa7ae39f0b18a04c927`.

The route receipt raw/self SHA-256 values are
`5bc448e9a27968d6b73c7448fe72e41b52e2c1ec82d7b7ddf898dcf1aaa79883`
and `79d6c84da4482f70e8735dca83dde531bd3cd0a2890f68ecabda02a260052220`.
The mathematical and novelty review raw/self SHA-256 values are
`60e9bbc25979c8b2e4f38e0d03fa3fc68c8ff077949f94e81a024a56dd1957d6`
and `1d115fbfbcca9122126ca2e18836079c355c36e29590c08260eaf20b1c1db118`.
Both artifacts bind the same candidate commit, tree, and manifest contract.

The focused final gate ran at
`8b2fc8899664d7af71017354a3918293d71f30d3`. It passed 3,395 Lean jobs,
757 Python tests with three skips, and 100 Rust Crouzeix tests. Route
validation returned `status=complete` and `claim_level=complete-local`. The
canonical dependency cache contained 134,192 records before and after, with
unchanged metadata SHA-256
`e22fd9dbbb48edface954c43ee9c62e169c1a268b4997deda331c6bbc1625c36`.

## Evidence corpus

The factual record for this retrospective is limited to committed repository
state and the independent post-execution review. The main committed sources
are:

- the [CPFR-087 tracker subtree](tracker.org), which records preflight, mapped
  validation, mathematical review, publication, Standards review, and the
  focused final gate;
- the [execution ledger](execution-ledger-003.tsv), whose monotone CPFR-087
  rows end at `release-verified`;
- the [Harp route manifest](../../../labs/crouzeix_proof_reproduction/formal_targets/harp/route-manifest.json),
  including its LS route dependency, graph, closure, theorem identities, and
  publication bindings;
- the immutable [route receipt](../../../evidence/crouzeix_conjecture/routes/harp/receipt.json)
  and [mathematical and novelty review](../../../evidence/crouzeix_conjecture/reviews/harp.json);
- commits `e9685196f68fce2da1201b176d2eb6b4ef35e56e` through
  `442ae2a03cd1b1b38413f613cb86d82a2b1f484c`, which record the phase's
  claim, contract, manifest, review, publication, repairs, and gate metadata;
  and
- the earlier [CPFR-086 retrospective](cpfr-086-retrospective.md), used only
  to distinguish inherited LS publication lessons from CPFR-087-specific
  findings.

Observed facts in the sections below are directly represented in those
artifacts or in the independent review. Interpretations are labeled and do not
change the route claim. No private memory or agent transcript is evidence for
this note.

## Phase-by-phase variance

The following table separates the planned action from what execution required.
The Cost column records concrete rework or risk, not elapsed-time estimates.

| Phase | Planned | Actual | Variance | Cause | Cost | Structural improvement |
|---|---|---|---|---|---|---|
| Claim and preflight | Create the worktree, link the canonical cache, run `preflight --route harp`, claim Kata, and record `implementing`. | Commit `e968519` recorded claimed metadata and a successful preflight result but omitted the exact command; commit `f377578` added it. The tracker heading remained `TODO` until commit `8b2fc88`, after publication and review. | Claim state, tracker heading, command evidence, and the first ledger row were not one validated transaction. | The plan described each artifact but did not define an atomic pre-commit claim checklist or validator. | A metadata repair commit and later state repair were required. Recovery had to infer whether the route was really claimed. | Validate tracker heading and claimed fields, exact preflight command/result/cache link, and the matching `implementing` ledger row before the first metadata commit. |
| Reuse contract | Require explicit reuse for imported LS support. | The first interpretation tried to map six LS proof nodes onto eleven imported support modules. Commit `edff4d1` established support-module semantics, nullable reused node IDs where no LS proof node exists, and one `route_dependencies` binding to the landed LS manifest and receipt. | Proof-node identity and support-module reuse had been conflated. | The plan used "node" and "module" as if their rosters were isomorphic. They are not. | Python, schema, and Rust contracts all needed coordinated changes before the manifest could be trustworthy. | Define proof nodes and support modules as separate sets. Require exact support-module coverage and bind their authority through `route_dependencies`; use a reused node ID only when a real source-route proof node exists. |
| Cross-language schema | Extend Python, JSON Schema, and Rust for Harp reuse. | The contract needed canonical full-instance tests in all three validators, not only field-shape assertions and hand-built partial fixtures. Commit `edff4d1` added strict route dependency handling and full Harp/legacy instance checks. | Initial parity reasoning was local to fields rather than the complete manifest. | Conditional JSON Schema branches, Python normalization, and Rust parsing can each accept a different object while individual field tests pass. | Review and repair touched all three contract implementations. | Keep canonical valid and invalid complete instances and require identical acceptance across JSON Schema, Python, and Rust. |
| Derivation graph | Write the full cubature, finite-dimensional, recurrence, perturbation, limit, theorem, and consequence graph. | The initial mathematical review found the double-layer application at the wrong dependency layer. The accepted graph makes `harp-double-layer-application` a direct dependency of the counting-L2 witness and therefore a transitive dependency of the terminal theorem. | A mathematically load-bearing edge existed in the inventory but not at its actual consumer. | Node presence was checked more strongly than direct provider-edge accuracy. | The graph required repair and a fresh review before publication. | For every load-bearing declaration, compare direct manifest edges with the declaration body, imports, and provider report before freezing the candidate. |
| Mathematical review | Review all route nodes and the novelty/reuse boundary before publication. | The final reviewer passed candidate `909b781`, tree `b9724fd`, with 24 nodes, 11 LS reuses, and zero findings after the double-layer repair. | One review rewind occurred before approval. | The verifier found a causal dependency error that type and closure checks could not detect. | Publication was correctly delayed; no published artifact had to be replaced. | Keep Spec review before publication and turn each graph finding into a direct-edge regression test. |
| Publication | Run each create-only publisher once, then validate the route. | `publish-route` and `publish-review` each ran once. Commit `d257aa1` bound the receipt; commit `c3b03a7` published the review and final certificate. Validation advanced from `mapped / incomplete` to `complete-local`. | The publication transaction itself matched the plan. Post-publication tests did not. | Tests encoded the authoring lifecycle as if it were live canonical state. | The immutable published artifacts stayed valid, but the focused gate could not pass until tests were repaired. | Use immutable authoring, receipt-bound, and published fixtures. Reserve live canonical tests for the current final state. |
| Post-publication focused gate | Re-run Harp route, publication, and validation tests. | The first no-Lean run found stale unpublished-state assertions plus package-import failures. Commit `2e8c9f8` added published-state assertions and repaired the `formal_target` to `tickets` import chain. | Test lifecycle and Python invocation mode had not been modeled explicitly. | Several modules assumed direct-script sibling imports. Broad `ImportError` fallbacks could also hide a real dependency failure. | A repair commit and two independent narrow reviews were required after evidence publication. | Support package and direct-script execution with explicit `__package__` branches. Forbid broad `ImportError` fallbacks and `sys.path` masking. |
| Whole-phase Standards review | Review the frozen complete-local candidate read-only. | `/root/cpfr087_final_standards` passed reviewed HEAD `c3b03a7` with zero findings and authorized only the release gate. | No standards repair was required, but the later verifier changed the tracker from `TODO` to `IMPLEMENTING`. | The verifier prompt said read-only but did not require a before/after tracked snapshot or explicitly forbid repairs. | Commit `8b2fc88` mixed verifier-driven metadata repair into the transition, weakening reviewer independence. | Snapshot tracked state before and after every independent verifier. Any drift is failure; the verifier reports the defect and the controller assigns a separate repair. |
| Focused final gate | Run the exact phase checks in a tracked-state-preserving sandbox without cache hydration. | Several Seatbelt iterations discovered required namespaces for test temp trees, LS receipt locks, macOS metadata, and bounded `xcrun_db-*` writes. The final policy passed 25 of 25 positive and negative probes. Python used an ignored, offline, unseeded uv environment with CPython 3.12.10. | Runtime namespace discovery occurred during the expensive gate instead of before it. | Python, Git, xcrun, tests, and audit helpers create or inspect different narrow namespaces even when the repository is logically read-only. | Earlier gate attempts failed before the definitive 3,395 Lean, 757 Python plus three skips, and 100 Rust results. | Precompute and probe the exact runtime namespace manifest before an expensive gate. Default to uv Python and pass `sys.executable` to child processes. |
| Manifest identity attestation | Recompute and record the canonical manifest identity before final metadata. | The verifier narrative once spliced an incorrect manifest raw hash. The controller independently recomputed the canonical SHA-256 `4be259ffa3056d0baae71aacae975f19d11c341686c6da9c2257cfbfea5a96fe` from the manifest before writing release metadata. | The prose result and the canonical artifact briefly disagreed; the committed tracker and ledger contain the corrected value. | A digest was copied manually from prose instead of consumed from canonical machine output. | One additional attestation check was required; no proof, manifest, receipt, or review byte changed. | Consume canonical JSON machine output or reference a schema-validated typed evidence object. Never copy identity hashes from narrative prose. |
| Identity audit and ledger | Recompute identities and append one release-verified ledger row. | The audit helper initially lacked repository package visibility and needed `PYTHONPATH` or module execution. The release row also needed normalization after an initial key inconsistency. The committed row now has the agreed 19 note keys. | Audit execution and ledger shape were treated as incidental shell details. | A helper outside the repository root does not inherit package imports, and free-form note keys lack compile-time guarantees. | Identity reporting and the final transition needed correction even though proof artifacts did not change. | Invoke audit helpers with repository-root `PYTHONPATH` or `python -m`; assert the exact release-note key set until typed JSON evidence replaces it. |
| Commit and landing control | Produce focused commits, then leave landing to the controller. | The first nine CPFR-087 commits after base `629b8aa` lack the mandatory `Co-authored-by: TRAE CLI <noreply@bytedance.com>` trailer. The last two contain it. No history was rewritten because the earlier commits already bind reviewed or published evidence. | Commit-message policy was not gated before each commit. | The plan required focused commits but had no mechanical trailer assertion. | Historical policy debt remains visible. Rewriting it would invalidate evidence-bound identities. | Check the trailer after every future commit and audit the whole candidate range before publication. Do not rewrite these nine historical commits. |

## Failed assumptions and root causes

**Observed facts.** The initial claim record asserted a ready preflight without
the exact command. The tracker state and ledger state then diverged: the
ledger said `implementing`, while the tracker heading still said `TODO`. The
later phase verifier repaired that tracked heading despite its read-only role.
These are three forms of the same defect. Phase ownership was distributed over
several files without a validated transaction.

**Interpretation.** The right immediate control is a claim transaction, even
before a general ticket API exists. The first metadata commit should be
rejected unless tracker heading, claimed fields, preflight evidence, cache
link, and implementing ledger row agree. If a future phase has no preflight
command, the plan should state the required assertions and test; inventing a
command name would create false evidence.

**Observed facts.** The LS route has six proof nodes relevant to its own
certificate, while Harp directly reuses eleven lower-level LS support modules.
The accepted Harp manifest therefore binds the LS route manifest and receipt
once through `route_dependencies`, then enumerates eleven `reused-route`
support nodes. A support node may have no corresponding LS route proof-node ID.

**Interpretation.** Route evidence inheritance and proof-graph identity are
different relations. A route dependency proves which prior certificate is
trusted. A support-module entry proves which lower-level implementation is
load-bearing. Forcing a false one-to-one mapping loses information in both
directions.

**Observed facts.** Review found the double-layer identity listed but attached
at the wrong layer. Cross-language work also needed full-instance parity tests,
because field-level checks did not prove that Python, Rust, and JSON Schema
accepted the same Harp manifest.

**Interpretation.** Roster validation is necessary but weaker than causal graph
validation. The plan must verify direct provider edges at their first
load-bearing consumer and run complete instances through all contract
implementations.

**Observed facts.** Publication made old tests stale. Those tests still expected
an unpublished route. Package-mode imports then exposed a sibling-import chain
that worked only when files were launched directly. The repair used explicit
`__package__` branches and added checks against broad `except ImportError`.

**Interpretation.** Lifecycle fixtures and import modes are public contracts. A
live repository file cannot stand in for historical authoring state, and broad
import fallbacks can misclassify a dependency's internal failure as a request
to switch import modes.

**Observed facts.** The final sandbox was reached only after discovering
several runtime namespaces. The audit helper also failed until repository
package resolution was made explicit. The final release row needed a fixed key
set after an inconsistent draft.

**Interpretation.** Hermetic verification needs a declared runtime model before
it starts. Discovering namespaces, interpreters, helper imports, and evidence
keys during the gate turns verification into implementation.

**Observed facts.** A verifier narrative once spliced an incorrect manifest raw
hash. Before release metadata was recorded, the controller recomputed the
manifest itself and corrected the attested value to
`4be259ffa3056d0baae71aacae975f19d11c341686c6da9c2257cfbfea5a96fe`.
The correction changed the attestation, not the manifest or its bound receipt
and review.

**Interpretation.** Manual transcription made narrative prose an accidental
data path. Identity metadata should consume canonical JSON machine output or
reference a schema-validated typed evidence object. A hash copied from prose is
not an independent recomputation.

## Verifier effectiveness

The mathematical review was effective where build and schema validation were
not. It found the misplaced double-layer dependency before publication. The
final review then recomputed the route contract, 24-node graph, and 11-module
reuse boundary and returned `PASS` with no remaining findings. This supports
the ordering Spec review, repair, fresh Spec review, publication.

The cross-language validators were effective after full-instance parity was
added. Python and Rust both reject malformed route dependencies and validate
the bound LS manifest and receipt. JSON Schema constrains the Harp-specific
shape. The important improvement is that canonical complete objects, rather
than isolated schema fragments, now exercise the three interpretations.

The post-publication focused gate was effective because it caught two classes
of integration defect that the route review did not cover: lifecycle-stale
tests and package-mode imports. The repair did not change proof or published
evidence semantics. It changed tests and import boundaries, then received
independent narrow Spec and Standards review.

The release verifier was effective at execution isolation. Its final 25-probe
matrix allowed intended tool startup and bounded temporary writes while
denying tracked source, Git metadata, canonical dependency cache, undeclared
temporary paths, and network writes. Its weakness was role discipline. A
read-only verifier should have reported the tracker heading mismatch, but the
verification sequence instead produced commit `8b2fc88` to repair it. Future
verifiers need a tracked before/after snapshot and a categorical ban on
repairs.

The release ledger is useful but only partially typed. Its final note records
the exact gate facts, but the independent review found an initial key
inconsistency. Until a typed evidence object exists, the plan now requires the
exact 19-key set and rejects missing, duplicate, or extra keys.

## Cache and proof-iteration efficiency

The initial mapped-route gate ran the cached `CrouzeixHarp` target once: 3,395
jobs in 25 seconds total, with 14 seconds of scanning, two seconds checking the
cache, and nine seconds in Lake. Publication ran the same 3,395-job target in
18 seconds total. The focused final gate ran 3,395 jobs in 11 wrapper seconds
and 14.15 wall-clock seconds. None of those records shows dependency hydration.

The strongest cache evidence is the final before/after snapshot. Both sides had
134,192 records and SHA-256
`e22fd9dbbb48edface954c43ee9c62e169c1a268b4997deda331c6bbc1625c36`.
The successful phase therefore rebuilt only allowed Harp-owned outputs against
the warm shared cache. No `lake update`, Mathlib cache fetch, or
`mise run lean-cache` was needed.

The proof iteration itself was bounded. The main review repair changed the
manifest's causal graph, not Lean proof code. The avoidable cost was runtime
policy discovery during final verification. The gate should start only after a
cheap namespace probe covers the selected Python interpreter, Git and xcrun
metadata, `/private/tmp/tmp*` test roots,
`/private/tmp/harp-ls-receipts-locks`, the audit root, build output roots, and
the bounded Darwin `xcrun_db-*` path.

The final Python environment was intentionally ignored, offline, and unseeded:
`.build/cpfr087-python`, CPython 3.12.10, whose real interpreter was
`/Users/bytedance/.local/share/uv/python/cpython-3.12.10-macos-aarch64-none/bin/python3.12`.
This removed reliance on whichever `python3` appeared first outside the gate.
Child-process tests should inherit `sys.executable`; only an explicit system
Python compatibility probe should invoke `/usr/bin/python3`.

## Context recovery quality

Recovery was strong once the immutable artifacts existed. The candidate commit
and tree, contract digest, receipt, review, closure, theorem type, and LS
dependency can all be recomputed from repository files. The monotone ledger
shows `locally-verified`, `spec-approved`, `evidence-published`,
`standards-approved`, and `release-verified`. A new controller can determine
the exact route state without relying on narration.

The opening state was weaker. The first ledger note used result-only preflight
metadata, the tracker omitted the exact command, and the heading did not match
the claimed state. Commit `f377578` repaired the missing command. Commit
`8b2fc88` repaired the heading much later. This made the earliest recovery
question needlessly ambiguous: was the route merely assigned, or had the
claim transaction completed?

The release row is dense enough to recover the final gate, but free-form notes
make key drift easy. The immediate plan change fixes its exact 19 keys. A fully
typed ledger remains later architecture work because introducing it inside
CPFR-087 would widen scope and migrate existing evidence formats.

## Publication and landing safety

Publication preserved the intended trust boundary. The route and review
publishers each ran exactly once. The review was frozen against candidate
`909b781`, tree `b9724fd`, and manifest contract `281b2b58`. The published
review is byte-identical to its candidate. The manifest binds the already
landed LS manifest SHA-256
`c8c4aa731781015a356b0cc43f080df36a9d489a0b60700fea8e331c5993fbaa`
and receipt raw SHA-256
`f672bb002c9d7ebc516683d61ba87b2c1ff2bed891daac0e6781aeb50cc7a01b`.
No publisher rerun or evidence rewrite was needed after the test-only repair.

The route has not landed. The worktree HEAD for this retrospective starts at
`442ae2a03cd1b1b38413f613cb86d82a2b1f484c`, whose tree is
`49395da7a0c56b759733e2751f45edc14905b290`. The controller still owns the
full repository gate, final payload digest, import receipt, local fast-forward,
tracker terminal transition, and Kata closure. CPFR-088 through CPFR-091 and
the plan-review phase are not made complete by this route certificate.

Commit history has one explicit debt. The first nine CPFR-087 commits after
base `629b8aa` omit the mandatory co-author trailer; commits `8b2fc88` and
`442ae2a` contain it. Rewriting the first nine would change hashes already bound
by reviews, receipts, or ledger evidence. The safe response is to retain that
history, disclose the debt, and gate every future commit and publication range
for the exact trailer.

## Plan changes

The goal plan now makes the following immediate changes for Task 6 and future
route phases:

- treat the initial claim as one validated transaction across tracker state,
  claimed metadata, preflight command/result/cache link, and ledger row;
- distinguish proof-node reuse from support-module reuse and bind inherited
  route authority with `route_dependencies`;
- run canonical complete manifest instances through JSON Schema, Python, and
  Rust and require acceptance parity;
- check direct provider edges for every load-bearing dependency, including the
  double-layer consumer edge;
- keep immutable authoring, receipt-bound, and published fixtures, while the
  live canonical test asserts only current final state;
- support package and direct-script imports through explicit `__package__`
  branches, without broad `ImportError` fallbacks or `sys.path` mutation;
- freeze and probe a Seatbelt namespace manifest before the expensive gate,
  use offline unseeded uv Python by default, and pass `sys.executable` to child
  processes;
- run audit helpers with repository-root `PYTHONPATH` or as repository modules;
- require a tracked before/after snapshot for read-only verifiers and forbid
  verifier repairs;
- require the exact 19 release-ledger note keys or a future typed JSON evidence
  reference; and
- validate the exact co-author trailer before every future commit, confirm it
  after the commit, and audit the complete candidate range before publication,
  without rewriting existing evidence-bound commits.

These changes are assertions and tests that fit the current plan. Broader
transactional ticket APIs, generated cross-language validators, package-only
CLIs, a reusable hermetic-policy generator, a capability-separated verifier
runtime, and a fully typed ledger remain later architecture work.

## Rejected changes

- Do not permit broad temporary-directory or home-directory writes to make
  tests pass. The final gate proved bounded namespaces were sufficient.
- Do not hydrate or update the shared Lean cache. The final before/after digest
  and record count were identical.
- Do not rerun either publisher to repair tests, imports, metadata, or prose.
  The existing receipt and review are immutable and valid.
- Do not rewrite the first nine commits to add trailers. Their hashes already
  participate in evidence and review history. Prevent recurrence instead.
- Do not weaken tracker ownership or let a verifier repair tracked state. The
  verifier reports drift; the controller assigns and records a separate repair.
- Do not add `sys.path` manipulation to hide package-boundary defects. Use
  explicit package/direct execution branches and repository-root module
  invocation.
- Do not interpret `complete-local` as whole-program completion. It certifies
  this Harp route only; the local six-row bundle, reader reconciliation,
  program review, final repository gate, landing, worktree audit, and final
  plan review remain pending.

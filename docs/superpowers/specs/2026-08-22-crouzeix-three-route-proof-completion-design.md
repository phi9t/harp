# Crouzeix three-route proof completion design

## Objective

Finish the Harp Crouzeix program at a stronger boundary than “the terminal
declaration compiles.” Jin, Lorist--Schwenninger, and Harp must each have a
route-isolated build, an explicit mathematical correspondence ledger, a
bounded axiom and provider audit, immutable execution evidence, and a
reader-facing status derived from those artifacts.

The primary theorem shared by all three routes is the finite-dimensional
polynomial bound:

```lean
∀ {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
  (A : Matrix n n ℂ) (p : Polynomial ℂ),
  ‖polynomialEval p A‖ ≤
    2 * maxPolynomialModulusOnNumericalRange A p
```

The supported consequence ceiling is scalar polynomial and rational
spectrality for nonzero complex Hilbert spaces. This project does not claim
complete boundedness, a matrix-valued functional calculus, a constructive
proof, peer review, publication, or author endorsement.

## Current baseline

Local `master` at `c8237e3` contains three compiling terminal declarations:

| Route | Terminal declaration | Current evidence | Remaining gap |
|---|---|---|---|
| Jin | `CrouzeixConjecture.crouzeixConjecture` | Three source-map rows have passed proof-slice receipts; the shared aggregate and final release gate pass. | The three-row map records two interfaces and one terminal wrapper, not the complete load-bearing proof chain; Jin lacks a dedicated provider-isolated Lake target and current route-level aggregate receipt. |
| Lorist--Schwenninger | `CrouzeixConjecture.loristSchwenningerMainTheorem` | The dedicated aggregate compiles, a read-only mathematical audit reported no critical or important issue, and provider closure is checked. | The authoritative six-node graph still records two passed and four blocked nodes; no v2 receipt set has been published or selected. |
| Harp | `CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem` | The broad aggregate compiles and the finite-atomic witness API passed focused review. | There is no Harp derivation ledger, dedicated Lake target, route receipt, or published local evidence row. The route reuses eleven lower-level LS modules and must not be called mathematically independent. |

The repository-wide `verification_manifest.tsv` still records only the older
upstream Jin acquisition observations. The optional local formalization bundle
is absent, and its Rust enforcement flag remains `false`. This is the correct
pre-publication state.

## Completion vocabulary

Each route moves through the following monotone claim levels:

| Level | Meaning | Required artifact |
|---|---|---|
| `authored` | A local declaration and proof term exist. | Tracked Lean source. |
| `compiled` | The route target elaborates in the pinned shared root. | Successful route command output. |
| `audited` | The terminal and named load-bearing declarations have only allowed axioms, and the active import closure satisfies the route policy. | Axiom and provider reports. |
| `mapped` | Every load-bearing proof step is connected to a pinned source span or, for Harp, to an explicit derivation/reuse entry. | Validated route ledger. |
| `receipt-backed` | Command, inputs, closure, dependency artifacts, output, declarations, and audits are hash-bound in immutable evidence. | Validated route receipt. |
| `complete-local` | All prior levels hold and an independent mathematical review has no unresolved Critical or Important finding. | Review manifest plus route receipt. |

No tracker state, agent report, broad aggregate compile, or source scan may
skip a level. Documentation must name the actual level.

## Architecture

### Route-isolated build roots

Keep one physical Lake project and one mathlib dependency cache, but expose
three non-default route libraries:

```text
formalization/lean/CrouzeixJin.lean
formalization/lean/CrouzeixLoristSchwenninger.lean
formalization/lean/CrouzeixHarp.lean
```

`Crouzeix.lean` remains the integration aggregate. Route targets are evidence
boundaries:

- Jin forbids `Crouzeix.LoristSchwenninger.*` and `Crouzeix.Harp.*`.
- Lorist--Schwenninger forbids `Crouzeix.Jin.*` and `Crouzeix.Harp.*`.
- Harp forbids `Crouzeix.Jin.*` and the LS terminal/provider modules, while
  permitting an enumerated set of lower-level LS lemmas.

The Harp allowlist must name every reused LS module. Prefix-wide LS access is
not permitted. Adding or removing a reused module changes the Harp route
ledger and invalidates its receipt.

### Route ledgers

Add one strict `crouzeix-route-proof-manifest/v1` contract and one ledger per
route:

```text
labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json
labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/route-manifest.json
labs/crouzeix_proof_reproduction/formal_targets/harp/route-manifest.json
```

Each ledger records:

- route identity and claim kind (`source-faithful` or `derived`);
- pinned source identities;
- exact terminal statement and consequence declarations;
- aggregate module and build target;
- ordered proof nodes with declaration, module, dependencies, and normalized
  declaration-type digest;
- source locators for Jin and LS;
- derivation class and reused-node provenance for Harp;
- the complete active module-closure digest;
- review-manifest path and digest; and
- route-receipt path and digest once published.

The Jin ledger supplements rather than rewrites the immutable three-row
proof-slice history. The LS ledger references the six-node graph and its
selected v2 receipts. The Harp ledger must state that the finite atomic
construction is Harp-owned while its scalar, norm-attainment, boundary, and
moment helpers are reused from LS.

### Proof dependency manifests

Do not make agents maintain every transitive import edge manually. Generate a
deterministic route report from active Lean syntax and validated metadata:

```json
{
  "schema_version": "crouzeix-route-proof-report/v1",
  "route_id": "lorist-schwenninger",
  "terminal_declaration": "CrouzeixConjecture.loristSchwenningerMainTheorem",
  "module_closure_sha256": "64 lowercase hex characters",
  "status": "passed"
}
```

The generated report joins four observations:

1. active transitive local imports;
2. declared proof-node dependencies;
3. `#print axioms` output for terminal and load-bearing declarations; and
4. source/derivation locators from the route ledger.

It fails closed on missing managed imports, duplicate or cyclic nodes, an
unmapped load-bearing declaration, forbidden provider imports, unexpected
axioms, or a theorem-type digest mismatch.

### Evidence publication

Route publication is two-phase:

```text
prepare -> execute once -> candidate directories -> validate -> atomic ledger promotion
```

Candidate directories are never overwritten. A successful rename followed by
a durability failure remains a visible unreferenced candidate and is reported
for recovery. Promotion updates an entire route atomically, so downstream
consumers never observe half of a proof chain.

After all three route ledgers are authoritative, publish a six-row local
formalization bundle:

```text
jin-main-theorem
jin-closed-numerical-range
ls-main-theorem
ls-closed-numerical-range
harp-main-theorem
harp-closed-numerical-range
```

The bundle uses one broad aggregate build but contains separate provider
reports and axiom audits. The Rust verifier flips the local-bundle requirement
to mandatory in the same commit that adds the genuine bundle.

### Cache and execution contract

The canonical dependency cache is `<harp-root>/formalization/lean/.lake` in the primary
checkout. Every proof worktree links its `formalization/lean/.lake` to that
cache. A proof command must fail before Lake if the link, toolchain, manifest,
or required dependency artifacts are absent or inconsistent.

Proof iteration must never run dependency hydration commands. In particular,
do not run `lake update`, `lake --try-cache exe cache get Mathlib`, or
`mise run lean-cache`. A missing shared cache is an operator-visible blocked
precondition. Harp-owned `.olean` files may be rebuilt.

The proof loop is ordered from cheapest to most expensive:

```text
static checks -> focused tests -> route target -> route receipt dry run
-> independent review -> lean-all -> full repository gate
```

A no-build preflight prints the selected toolchain, cache identity, aggregate
root, route closure, and expected commands without invoking Lean. Worktree-local
`mise` trust is supplied for one invocation through `MISE_TRUSTED_CONFIG_PATHS`;
the workflow does not mutate persistent trust state.

## Mathematical completion criteria

### Jin

The Jin route is complete locally when:

- the dedicated Jin target compiles without LS or Harp imports;
- the complete load-bearing chain is mapped to the pinned Jin v4 manuscript
  and formalization records;
- the existing three proof-slice receipts still validate unchanged;
- a new route receipt binds the complete active closure, terminal type, and
  axiom audit; and
- an independent reviewer confirms that the Harp port preserves Jin's
  cancellation, Gramian, positive-real completion, double-layer,
  simple-spectrum, and limit arguments.

The blocked upstream v4.28 clean-room build remains a separate provenance
fact. It may be rerun only when a compatible pre-existing cache is supplied;
it is not required for the Harp-local completeness claim.

### Lorist--Schwenninger

The LS route is complete locally when:

- the dedicated LS target compiles;
- every one of the six canonical graph nodes has a validated v2 receipt;
- all six receipts bind one aggregate execution identity and ordered allowed
  axioms;
- the graph and library inventory are atomically promoted to the current
  declarations; and
- an independent reviewer confirms the source orientation `T = op(P)`,
  `Q = M_h`, the recurrence, scalar contradiction, double-layer realization,
  and both limiting passages.

### Harp

The Harp route is complete locally when:

- the dedicated Harp target compiles without Jin or LS terminal providers;
- its derivation ledger records the finite positive cubature, finite atomic
  counting-L2 model, horizon recurrence, perturbation endpoint, double-layer
  instantiation, and outer limits;
- every LS reuse is explicit and no stronger independence claim is made;
- a route receipt binds the full closure, terminal type, provider report, and
  axiom audit; and
- an independent reviewer confirms both the mathematical soundness and the
  advertised novelty boundary.

## Verification implementation boundary

The current Python and Rust validation code is large enough to make future
changes risky: the principal Rust module is over six thousand lines and the
five Python proof-evidence modules plus tests exceed eighteen thousand lines.
Do not perform a broad rewrite before publication. Add the Python route
contract and CLI through narrow modules, add Rust route validation under
`crates/harp/src/sources/crouzeix/route.rs`, reuse current hardened filesystem
primitives, and extract other shared code only when a characterization test
proves byte-for-byte behavioral parity.

The production CLI must expose these operations without accepting arbitrary
commands or environment variables:

```text
preflight --route jin|lorist-schwenninger|harp|all
validate --route jin|lorist-schwenninger|harp|all
publish-ls
publish-local
```

`preflight` and `validate` are read-only. The two `publish` operations are
create-only and must print recovery metadata for partial publication.

## Review boundary

Each route requires two independent reviews over a frozen commit:

- **Spec review:** source correspondence, theorem strength, hypotheses,
  constants, orientations, limits, and claim ceiling.
- **Standards review:** proof bypasses, import/provider boundaries, generated
  evidence integrity, filesystem safety, cache behavior, and maintainability.

No Critical or Important finding may remain. Reviewer prose is stored as a
hash-bound review manifest; it does not replace compiler or receipt evidence.

## Non-goals

- No new live blind proof-search run.
- No requirement to rebuild Jin's historical toolchain from source.
- No remote cache or persistent build daemon.
- No rewriting immutable v1 receipts.
- No claim that Harp is mathematically independent of LS.
- No extension to completely bounded or abstract uniform-algebra statements.
- No push.

## Definition of done

The three-route program is complete only when all three route ledgers are
`complete-local`, all route receipts and reviews validate independently in
Python and Rust, the six-row local bundle is mandatory, canonical prose is
reconciled to the evidence, generated Atlas outputs are current, and the full
warm-cache repository gate passes on the exact commit fast-forwarded to local
`master`.

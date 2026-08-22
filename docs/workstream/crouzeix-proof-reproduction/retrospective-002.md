# Crouzeix three-route execution retrospective

## Scope and claim ceiling

This retrospective distills the agentic execution that began with the Lean
worktree inventory and continued through the Jin, Lorist--Schwenninger, and
Harp proof work landed on 2026-08-22. It is process evidence, not mathematical
evidence. Lean source, source-mapping ledgers, immutable proof receipts, and
independent theorem reviews remain authoritative for proof claims.

The inspected parent session contained 50 user turns, 76 context compactions,
199 agent-spawn events, 269 agent-interaction events, and 387 patch events.
Those counts describe the execution shape; they do not measure proof quality.

## Outcome

The session produced a materially stronger proof base:

- one shared Lean root and one reusable mathlib cache;
- a Harp-local Jin terminal theorem with three passed source-map receipts;
- a compiled Lorist--Schwenninger terminal theorem with a provider-isolated
  aggregate;
- a compiled Harp finite-horizon terminal theorem;
- provider-neutral consequence adapters; and
- fail-closed Python and Rust machinery for LS receipt publication, provider
  closure checks, local evidence publication, and repository verification.

The landing on local `master` is intentionally narrower than proof-program
completion. It does not promote the historical LS graph, publish the local
formalization bundle, or upgrade reader-facing claims.

## Actions that moved the work

| Resolving action | Enabling context | Tool or skill | Durable result |
|---|---|---|---|
| Consolidated independent Lean projects into one root | Repeated builds showed dependency work dominating proof iteration | Shared `lakefile.toml`, `mise` tasks, and cache-aware wrapper | One mathlib cache and focused library targets |
| Separated terminal providers from neutral consequences | LS and Harp needed common analysis without circular trust | Lean import refactor plus provider-closure audit | Route-specific terminal theorems over shared lower layers |
| Rejected publication when contracts disagreed | Python and Rust assigned v1/v2 target semantics differently | RED/GREEN receipt fixtures and two-axis code review | Preserved v1 compatibility and dedicated v2 semantics |
| Hardened create-only publication | Review exposed rename, cleanup, symlink, and mutation races | Descriptor-relative filesystem operations and adversarial tests | Recoverable candidate publication without silent overwrite |
| Deferred evidence promotion | Compiling theorem sources were newer than the authoritative LS graph | Claim-level discipline and branch-finishing review | Clean source/infra landing without fabricated status |
| Preserved cache and operator state | The user made proof iteration speed an explicit constraint | Canonical `.lake` link, preflight checks, and worktree isolation | No dependency hydration during the final landing gate |
| Reconstructed the long session from its exact log | Repeated compaction made summaries insufficient for process analysis | Session-log inspection and three independent reflection lenses | Trace-backed workflow corrections rather than anecdotal lessons |

## What worked

### One shared Lean root changed the economics of iteration

Consolidating the projects under `formalization/lean/` removed repeated
dependency graphs and made focused proof builds possible. Once the shared
cache was stable, the final `lean-all` gate spent seconds checking cache
preconditions and 198 seconds on the project build, rather than hydrating
mathlib again. This is the correct default for all future proof work.

### Proof claims became layered instead of binary

The work eventually separated:

1. a declaration existing in source;
2. a theorem compiling;
3. an axiom audit passing;
4. a provider-isolated route compiling;
5. a source or derivation map matching the theorem chain; and
6. immutable evidence being published and enforced.

That distinction prevented the final landing from rewriting the old LS graph
or claiming unpublished evidence as complete.

### Independent review found defects that compilation could not

Standards and Spec reviews found incorrect v1/v2 receipt semantics, incomplete
aggregate closure binding, unsafe post-rename recovery semantics, missing
managed-import failures, and an LS policy that could admit Harp imports. These
were evidence-boundary defects; a successful Lean build alone could not expose
them.

### Provider-neutral extraction made route comparison meaningful

Moving shared Hilbert, spectral-set, boundary, and radial machinery out of
provider-owned modules made it possible to state what is genuinely distinct.
The Harp route has a different finite-horizon atomic dilation construction,
but deliberately reuses lower-level LS lemmas. It is terminal-provider
independent, not mathematically disjoint.

## What created avoidable churn

### The thread carried too many programs

NNG4, autodiff foundations, Lean build consolidation, Jin, exposition, LS,
Harp, evidence publication, and branch cleanup all occupied one long-running
session. Repeated compaction forced state reconstruction and made stale plans
look current. Future proof programs need a stable tracked handoff before each
major route or infrastructure phase.

### Implementation outran the evidence contract

Receipt publishers and validators were written before their shared contract
was frozen as adversarial tests. That produced repeated review-and-repair
cycles, including a publisher/validator schema inversion and synthetic tests
that did not exercise the real six-node graph. The contract must become a
failing round-trip test before implementation begins.

### Parallelism exceeded file ownership

Many agents were useful for source reading and theorem/API exploration, but
multiple writers worked in the same dirty worktree and repeatedly touched the
same proof and receipt files. This made test results transient and forced
frequent re-audits. Parallel agents should own disjoint files, or be read-only;
one integration owner should control shared files and commits.

### Expensive gates ran before the tree was frozen

Several full builds and reviews preceded later protocol changes. The final
release gate then still found a forbidden vocabulary token and Clippy errors.
The correct order is static policy, focused unit tests, route compile, frozen
diff review, then one release gate. Any edit after the release gate invalidates
the payload digest and requires the final gate again.

### Status records lagged behind source

At landing time, all three terminal declarations compiled, but the LS graph
still recorded two passed and four blocked nodes, and Harp had no route ledger.
The old tracker also marked historical implementation tickets `DONE`, which
describes completion of those ticket scopes rather than present proof status.
Future status summaries must be derived from current receipts and manifests,
not inferred from ticket state or compilation alone.

### Historical test fixtures depended on moving Git state

The first post-landing release replay failed five LS receipt tests. Their helper
reconstructed receipt-bound historical module bytes with `git show HEAD:path`;
after the proof landing changed `Scalar.lean`, `HEAD` no longer named the bytes
recorded by the immutable v1 receipt. The repair stores the historical source as
a digest-named fixture and verifies its bytes before use. Tests for immutable
evidence must carry immutable fixtures, not derive history from a moving ref.

### Worktree and tool preflight happened too late

The session rediscovered missing `lake` paths, worktree-local `mise` trust,
stale local oleans, and cache placement after expensive commands had started.
Every proof task needs a no-build preflight that resolves the toolchain, checks
the shared cache, verifies the worktree root, and prints the exact command that
will run.

## Operating model for the next phase

1. One tracked ticket, one implementation owner, and one isolated worktree.
2. Parallel agents are read-only unless their file sets are disjoint and
   declared before editing.
3. Freeze every receipt or manifest contract as a concrete failing
   producer-to-consumer test before implementation.
4. Run static checks and focused tests before any Lean invocation.
5. Run the smallest provider-isolated Lean target that proves the changed
   route.
6. Run `lean-all` and the repository release gate only after the candidate is
   frozen.
7. Never hydrate Lean dependencies during proof iteration. Missing cache state
   is a typed preflight failure requiring owner action, not permission to run a
   cold build.
8. Publish immutable receipts only after the exact source tree passes both
   Standards and Spec review.
9. Promote a route ledger atomically; do not expose a half-promoted dependency
   chain.
10. Derive documentation claims from validated ledger state and regenerate
    reader artifacts afterward.

## Verification ladder

| Stage | Purpose | Expected cadence |
|---|---|---|
| Static policy | Formatting, forbidden tokens, import parsing, schema shape | Every edit |
| Focused unit tests | Publisher/validator or theorem-support behavior | Every RED/GREEN slice |
| Focused Lean target | The changed route and its downstream declarations | After a proof slice stabilizes |
| Route receipt dry run | Exact command, closure, artifact, provider, and axiom bindings | Before publication |
| Independent review | Mathematical Spec plus implementation Standards | Once per frozen route candidate |
| `lean-all` | Shared-root regression | Once per frozen landing |
| `mise run verify` | Repository release gate | Once before landing, repeated only after a repair |

## Durable conclusion

The proof project should now optimize for evidence convergence rather than raw
theorem production. The three terminal declarations exist and compile. The
remaining work is to make each route independently buildable, source- or
derivation-mapped, receipt-backed, independently reviewed, and represented
truthfully in the canonical reader surface.

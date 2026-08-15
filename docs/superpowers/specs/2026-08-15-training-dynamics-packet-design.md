# Training Dynamics packet design

## Purpose

Add a self-contained advanced Harp packet that begins where Mathematical
Foundations module 6 ends: it explains the behavior of sequences of training
updates, rather than merely defining one update. The packet serves ML
practitioners who can read the six-module foundation route and want a rigorous
bridge from quadratic optimization to optimizer behavior in neural-network
training.

The release is self-contained: every prerequisite, definition, worked example,
and formal theorem needed for its stated route lives in this repository. It
will not require a local external checkout at runtime, build time, test time,
or documentation-link time. Any useful ideas discovered during the input audit
will be reconstructed as newly written, source-bounded exposition; incomplete
or unsupported material is excluded rather than assumed.

## Reader boundary

This is an advanced packet, not a seventh core module. The mathematical
foundations index will link to it as an optional follow-on after module 6.
Readers need linear algebra, gradients/Hessians, positive-definite quadratics,
expectation/covariance, and the basic gradient-descent update. The packet will
not claim that its controlled-model theorems prove corresponding behavior for
general deep networks.

## Self-contained coverage contract

The packet will define the executed-update state before it draws a conclusion:
parameters, optimizer state, data/order, batch and accumulation boundary,
schedule, numerical policy, and target measurement. It will give a reader a
small experiment contract—one changed variable, held variables, primary
metric, stability guard, and quality guard—so a diagnostic is falsifiable
rather than a post-hoc story.

The core route fully develops four connected mechanisms: deterministic
quadratic dynamics, scalar momentum dynamics, finite-support stochastic
gradient identities, and a measurement/transfer protocol. It also introduces
adaptive optimizer state, feature/parameterization change, coupled learners,
nonstationary data, proxy transfer, and systems update equivalence as named
boundary cases, each with its missing assumptions and measurement questions.
Those boundary cases are a self-contained orientation, not an assertion that
the core route has formalized or experimentally settled them. The index will
map them to explicitly deferred advanced follow-ons, so the packet has no
silent prerequisite or unmarked coverage gap.

## Packet shape

Create `knowledge/training_dynamics/` as a first-class Atlas reader route with
the following self-contained documents:

1. `training_dynamics_index.md` — reader contract, prerequisites, route, and
   explicit theorem-to-practice boundary.
2. `01_quadratic_gradient_descent.md` — eigendirections, exact recurrences,
   stable step sizes, conditioning, and original solved problems.
3. `02_momentum_and_acceleration.md` — heavy-ball recurrences, two-state
   dynamics, stability regions in scalar controlled models, and original
   solved problems.
4. `03_stochastic_gradients.md` — unbiased estimators, noise covariance,
   minibatch effects, and what expectation does not guarantee pathwise.
5. `04_diagnostics_and_transfer.md` — learning curves, gradient norms,
   curvature/noise probes, intervention design, update-equivalence checks,
   scaling-state accounting, and limits on transfer to neural networks.
6. `glossary.md` — shared notation for parameters, gradients, optimizer state,
   spectra, noise, and diagnostic quantities.
7. `source_registry.md` — bibliographic identity and copyright boundary for
   the public primary sources supporting the packet.
8. `claim_evidence_ledger.md` — locator-backed claims that separate exact
   theorem, model assumption, source report, and Harp inference.

Each of the four modules will contain six original problems with complete
solutions. The packet therefore starts with 24 problems, selected for formal
tractability and practical interpretation instead of imitating any source
exercise.

## Mathematical and Lean 4 boundary

The proof-bearing core is intentionally small and staged:

- Define scalar and finite-dimensional quadratic objectives.
- Formalize gradient descent as a recurrence around a minimizer.
- Prove contraction along an eigendirection when the step size lies in the
  stable interval, and provide counterexamples outside it.
- Formalize the heavy-ball update in one dimension and prove selected
  recurrence/stability lemmas.
- Formalize expectation identities for simple finite-support stochastic
  gradients only after the deterministic spine is stable.

The companion will use an isolated Lake/Lean 4 project with a pinned Lean and
mathlib toolchain. Its modules will mirror theorem dependencies, not prose
file names, and CI will run `lake build`. A theorem may link to a Lean
declaration only when that declaration builds. Empirical observations,
diagnostics, and neural-network transfer discussion remain prose claims with
their evidence boundary; they are never represented as Lean theorems.

## Atlas and corpus integration

Register `training-dynamics` as a reader route and every packet document as an
auxiliary document and search root. Reuse the existing mathematics rendering
boundary for `knowledge/training_dynamics/`, so TeX and explicit heading IDs
are validated and emitted safely. Add a Training dynamics navigation button,
route parsing/formatting coverage, reader rendering tests, and math-rendering
tests. Regenerate the corpus and offline export together.

The packet stays separate from retained RSI concepts, coverage-map obligations,
diagnostic contracts, and existing source-verification counts. Packet prose,
registries, ledgers, formalization files, build scripts, and tests must contain
no local external-checkout name, path, link, or dependency. The pinned public
Lean/mathlib toolchain is a deliberate exception and is documented in the
formalization manifest.

## Verification

- Rust: exact packet roster, frontmatter, backlinks, locator format, absence
  of external local paths, original problem identifiers, solution coverage,
  reader route, search indexing, and math rendering.
- Lean: `lake build` proves the pinned theorem set and rejects unsound or
  incomplete declarations.
- Atlas: route parsing, navigation, reader rendering, and all packet TeX
  render to MathML without fallback.
- Release: rebuild corpus, refresh search, rebuild/export Atlas, refresh the
  import receipt only after the tracked payload settles, then run the
  repository verification gate.

## Non-goals

- No claim that the packet explains all neural-network training dynamics.
- No copying of input prose, captures, code, or proprietary/local artifacts.
- No unpinned Lean installation, global toolchain dependency, or cross-checkout
  runtime/build/test dependency.
- No formal proof of empirical optimizer comparisons or large-model outcomes.

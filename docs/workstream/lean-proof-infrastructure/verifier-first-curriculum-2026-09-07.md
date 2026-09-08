# Verifier-first mathematical curriculum

User direction, 2026-09-07: build workload verifiers first. Start with
Cauchy–Schwarz, then pursue the broadest feasible formalization coverage of
Spivak, Lax, and both Bishop books before advancing to grand research campaigns.
The user's subsequent clarification makes all four books systematic coverage
targets, not sources of a few selected demonstrations. This supersedes Crouzeix as the
first new implementation workload. Existing Crouzeix evidence remains a later
regression workload.

## First vertical slice: finite real Cauchy–Schwarz

Freeze the target, for every natural number n and vectors x,y in R^n:

    (sum_i x_i * y_i)^2 <= (sum_i x_i^2) * (sum_i y_i^2)

Sums range over Fin n; n=0 and zero vectors are included. No positivity,
nonzero-vector, or positive-dimension assumptions are added. Equality
characterization and complex inner-product spaces are later targets.

Deliver the verifier before an automated prover. Its input is a sealed
candidate proof of the fixed statement in the pinned environment. Its output
distinguishes accepted proof, rejected candidate, and infrastructure failure.
Acceptance requires target/definition agreement, actual Lean checking,
transitive axiom/dependency checks, and a receipt binding exact inputs and
checker identity. A timeout or missing dependency never counts as a theorem
refutation or successful check.

Verifier acceptance corpus:

| Candidate | Required result |
|---|---|
| Known-good proof of the exact target | Accept with receipt |
| Same target with another valid proof | Accept independently |
| Target with an extra hypothesis or only n=1 | Reject as target mismatch |
| Direct sorry or helper hiding sorry/custom axiom | Reject |
| Redefined operation or substituted statement | Reject |
| Changed proof bytes paired with old receipt | Reject stale evidence |
| Invalid Lean source | Reject as compilation failure |
| Timeout, worker crash, missing pinned dependency | Infrastructure failure, no acceptance |

The pinned Mathlib already contains
`Finset.sum_mul_sq_le_sq_mul_sq` in
`Mathlib/Algebra/Order/BigOperators/Ring/Finset.lean`. A wrapper using it is a
positive verifier control and an explicitly labeled reuse result. A subsequent
proof-reconstruction exercise must declare its permitted foundations and
inspect actual dependencies, including indirect reuse of the target. Neither
mode is evidence of discovering Cauchy–Schwarz. A name blacklist alone is not
a robust reconstruction policy.

Completion means the positive and negative corpus has actually run, the
statement has been reviewed, and a fresh replay reproduces the result. No
multi-agent scheduler, trained retriever, remote service, or theorem registry
is needed to establish this first slice.

## Expand by mathematical dependencies

The following is a topic ladder, not a claim that exact book statements have
already been extracted. Freeze edition, section/equation locator, hypotheses,
definitions, and expected result before admitting each target. Begin with small
dependency-ordered packets, but retain every chapter in the coverage inventory.
Small packets are the delivery unit, not a limit on book coverage. Do not label
a book formalized from a handful of examples.

| Track | Small initial candidates | Later candidates |
|---|---|---|
| Lax: linear algebra | Inner-product identities, orthogonal projection, finite-dimensional least squares | Adjoint operators, self-adjoint spectral theorem, positive-definite forms |
| Spivak: Calculus on Manifolds | Euclidean continuity, derivative of a linear map, chain rule | Inverse/implicit function theorems, integration, differential forms, Stokes |
| Bishop: Pattern Recognition and Machine Learning (2006), confirmed by user | Finite probability identities and finite-dimensional least-squares results | Gaussian identities, carefully specified KL/variational results and optimization claims |
| Christopher M. Bishop and Hugh Bishop: Deep Learning: Foundations and Concepts (2024), user-supplied PDF | Shared probability foundations; derivatives of simple differentiable networks and loss functions | Backpropagation/automatic differentiation correctness, specified gradient-descent results, attention identities |

Topic dependencies may cross books: linear algebra supports Spivak and least
squares supports Bishop. Each new workload earns its own verifier fixtures
before proof automation is evaluated. Explicitly test missing regularity,
invertibility, support, normalization, and integrability assumptions where
relevant; numerical examples are not substitutes for these conditions.

Source identity anchors:

- [Spivak publisher contents](https://www.routledge.com/Calculus-On-Manifolds-A-Modern-Approach-To-Classical-Theorems-Of-Advanced/Spivak/p/book/9780805390216).
- [Lax, Linear Algebra and Its Applications, expanded second edition, 2007](https://uat.store.wiley.com/en-us/linear-algebra-and-its-applications-2nd-edition-p-9780471751564), explicitly selected by the user on 2026-09-07. Include its numerical-algorithm chapters and all sixteen appendices.
- [Bishop, Pattern Recognition and Machine Learning (2006), user-confirmed source](https://www.microsoft.com/en-us/research/wp-content/uploads/2006/01/Bishop-Pattern-Recognition-and-Machine-Learning-2006.pdf).
- Christopher M. Bishop and Hugh Bishop, *Deep Learning: Foundations and Concepts*, Springer, 2024; ISBN 978-3-031-45468-4 (eBook), DOI `10.1007/978-3-031-45468-4`. User-supplied file `bishop-deep-learning.pdf`, SHA-256 `277ac35c73df83a72a6fefde790c097168959c7c054bf1d25f8f5ebf0ed195c7`. Bibliographic pages and contents inspected; the PDF was not copied into the repository.

The deep-learning book is an additional track, not a replacement for PRML.
Contents anchors for later statement extraction are Chapter 6 (Deep Neural
Networks), Chapter 7 (Gradient Descent), Chapter 8 (Backpropagation), especially
8.1 (Evaluation of Gradients) and 8.2 (Automatic Differentiation), and Chapter
12.1 (Attention). These are candidate topic locations, not verified theorem
statements. Reuse shared linear algebra/probability/calculus foundations while
preserving separate source mappings for each book. Specify differentiability
conditions for gradient results, and keep real-arithmetic identities separate
from floating-point algorithm claims. Empirical training claims do not become
theorems merely because they appear in the book. Cauchy–Schwarz remains first.

## Comprehensive coverage contract

All chapters and mathematical appendices of all four books are in scope for
inventory. Capture named theorems, lemmas, propositions, corollaries,
definitions, mathematically substantive unnumbered claims and derivations,
algorithm specifications and provable properties, and exercises with precise
mathematical content. Include worked examples when they establish a reusable
identity, illustrate a necessary condition, or furnish a counterexample.
Do not silently exclude an item because it is difficult or absent from Mathlib.

For each source item, record a stable source locator, edition identity, item
kind, intended statement, explicit assumptions, prerequisite items, Mathlib
correspondence, verifier obligations, Lean declaration/proof references, review
and replay evidence, and any remaining gap. Book source identity must be fixed
before exact theorem numbering is accepted. Lax's edition is confirmed; record
the actual source-file digest before extracting item-level locators.

Keep inventory and proof state separate:

- Inventory: not yet inspected, extracted, or classified non-formal with a reason.
- Mathematical result: unstated, stated, proved by library reuse, proved by new
  derivation, refuted as written, or blocked with a concrete prerequisite.
- Acceptance: verifier fixtures pending/passing, semantic review pending/current,
  and replay pending/current/stale.

A proof with an unfinished prerequisite is conditional, not completed book
coverage. A corrected statement retains the original source claim and an
explicit explanation of the difference. Approximate algorithms need stated
error models; convergence claims need their actual assumptions. Empirical
architecture or training observations remain classified observations rather
than being converted into unsupported universal propositions.

For each chapter, report inventory progress separately from formal acceptance:
extracted eligible items, accepted items, unresolved items, and exclusions with
reasons. A chapter percentage is provisional until its inventory is complete.
Report theorem/derivation, exercise, and algorithm coverage separately so many
easy exercises cannot conceal a missing central theorem. Record coverage by
reuse and by new proof separately. One Lean result may cover matching items in
multiple books, but each source correspondence must be checked; this is not
multiple independent proofs.

The early-candidate table above is only an entry sequence. In particular, the
deep-learning track includes later chapters on convolutional networks,
structured distributions, transformers, graph neural networks, sampling,
latent-variable models, adversarial generative models, normalizing flows,
autoencoders, and diffusion models. Their mathematical definitions, identities,
and justified algorithm properties belong in the inventory even when formal
proofs will require substantial prior infrastructure. The supplied PDF's
contents list 20 chapters; no complete item inventory has yet been extracted.

Use shared foundations across the books and unlock work in dependency order.
Maintain both chapter views and prerequisite views. Prioritize load-bearing
results while reserving explicit work for remaining sections and exercises;
capstone-only completion is not the target. Each reporting cycle must identify
the largest remaining coverage gaps. Difficulty changes sequencing and cost,
not the recorded scope.

The objective is maximum defensible mathematical coverage, not an advance
promise that every sentence will become a theorem. Completion or a deliberate
scope reduction requires an itemized account of unresolved material; never
shrink the denominator to match what happened to be proved.

## Advancement rule

Advance only with reviewed statements, passing adversarial verifier fixtures,
completed Lean proofs, explicit library reuse, reproducible receipts, and
measured time/cost. Evaluate proof reconstruction separately from library
application. Build orchestration only when these small workloads demonstrate
an actual coordination bottleneck. Grand campaigns follow demonstrated
coverage and reliability on this curriculum.

Current state: direction and first acceptance contract recorded; verifier and
proof fixtures are not yet implemented or run.

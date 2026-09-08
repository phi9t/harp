# Four-book coverage map

Planning inventory, 2026-09-07. This records chapter families, not an extracted
theorem census or completed formalization. The user selected Lax's expanded
second edition, 2007. All four books remain in scope. Wave identifiers refer to
[the implementation roadmap](../../superpowers/plans/2026-09-07-textbook-formalization-waves.md).

## Source control

Each extracted item must bind book ID, edition/printing, source digest, printed
page, PDF page, section, and local item locator. Page offsets are not assumed
constant. Corrections retain both the original claim and correction provenance.
Keep copyrighted books outside the repository; store locators, original
formalizations, short necessary excerpts, and source/license records. A public
PDF URL is not a redistribution license.

| Book ID | Source evidence | Remaining source work |
|---|---|---|
| `lax-2007-2e` | User-confirmed edition; [university-hosted textbook scan](https://matematicas.unex.es/~navarro/algebralineal/lax.pdf), copyright/contents/prefaces inspected | Freeze the bytes used for item extraction and record digest; review errata |
| `spivak-com` | [Publisher contents](https://www.routledge.com/Calculus-On-Manifolds-A-Modern-Approach-To-Classical-Theorems-Of-Advanced/Spivak/p/book/9780805390216); [university-hosted textbook scan](https://web.math.ucsb.edu/~bigelow/books/spivak.pdf) | Freeze printing: inspected scan has March 1968 corrected-printing preface and Addenda, despite 1965 copyright |
| `bishop-prml-2006` | [User-confirmed PDF](https://www.microsoft.com/en-us/research/wp-content/uploads/2006/01/Bishop-Pattern-Recognition-and-Machine-Learning-2006.pdf); local SHA-256 `4ee767e0a6b04fa05ba7e599e9dbb4637a94a4407ccedf0b4d316b1fd7c8ec64` | Extract item-level inventory and attach errata revision |
| `bishop-dlfc-2024` | User-supplied PDF, Christopher M. Bishop and Hugh Bishop; [publisher DOI](https://doi.org/10.1007/978-3-031-45468-4); SHA-256 `277ac35c73df83a72a6fefde790c097168959c7c054bf1d25f8f5ebf0ed195c7` | Extract item-level inventory and attach errata revision |

University mirrors contain primary textbook text but are not publisher-hosted
digital editions. Their provenance must stay explicit. Chapter labels below
are topic summaries where shortened, not assertions of exact printed headings.

## Lax: all chapters and appendices

| Chapter | Coverage family | First eligible wave |
|---|---|---|
| 1 | Vector spaces | W3 |
| 2 | Dual spaces | W3 |
| 3 | Linear mappings | W3 |
| 4 | Matrices | W3 |
| 5 | Determinants and traces | W3 |
| 6 | Spectral theory | W3 |
| 7 | Euclidean/inner-product geometry | W3 |
| 8 | Self-adjoint spectral theory | W3 |
| 9 | Matrix and vector calculus | W4 |
| 10 | Matrix inequalities | W5 |
| 11 | Mechanics | W7 |
| 12 | Convexity | W5 |
| 13 | Optimization duality | W5 |
| 14 | Normed spaces | W5 |
| 15 | Bounded linear mappings | W5 |
| 16 | Entrywise-positive matrices | W5 |
| 17 | Algorithms for linear systems | W7 |
| 18 | Algorithms for self-adjoint eigenvalue problems | W7 |
| 19 | Selected exercise solutions | Map to originating exercises; no duplicate proof count |

All sixteen appendix families receive their own inventory entries. Assign each
an exact printed appendix label during source extraction:

| Appendix families | First eligible wave |
|---|---|
| Special determinants; Pfaffians; symplectic algebra; tensors | W3, then W6 for geometric uses |
| Lattices | W3 |
| Fast matrix multiplication; FFT | W7 |
| Gershgorin bounds; eigenvalue multiplicity; spectral radius; Jordan form | W3, with numerical consequences in W7 |
| Lorentz transformations | W7 |
| Compactness of the unit ball | W5 |
| Commutators | W3 |
| Lyapunov stability | W7 |
| Numerical range | W7, later bridge to Crouzeix |

## Spivak: all five chapters, corrections, exercises

| Chapter | Included section families | First eligible wave |
|---|---|---|
| 1 Functions on Euclidean space | Norms, inner products, subsets, continuity | W3 |
| 2 Differentiation | Foundational results, derivatives, partial derivatives, inverse/implicit functions, notation | W4 |
| 3 Integration | Rectangle-partition definitions, negligible sets, integrability, iterated integration, partitions of unity, substitution | W5 |
| 4 Integration on chains | Multilinear algebra, differential forms, geometric definitions, fundamental integration theorem | W6 |
| 5 Integration on manifolds | Manifolds, forms on manifolds, Stokes, volume, vector-calculus consequences | W6 |
| Addenda/corrections | Claim-specific amendments | Before the affected item is admitted |

The printed Chapter 2 contents include a derivatives section omitted from the
publisher webpage. Inventory against the actual text, not the webpage alone.
Riemann integration, content-zero arguments, and signed orientations require
source-faithful statements or proved compatibility bridges.

## Bishop: Pattern Recognition and Machine Learning

| Chapter | Coverage family | First eligible wave |
|---|---|---|
| 1 Introduction | Probability/decision theory identities, model-selection definitions; classify empirical material | W5 |
| 2 Probability distributions | Discrete/continuous normalization, moments, conjugacy, exponential families | W5 |
| 3 Linear models for regression | Least squares, Bayesian regression, evidence calculations | W5 |
| 4 Linear models for classification | Discriminants, probabilistic classifiers, logistic identities | W5 |
| 5 Neural networks | Differentiable networks, gradient identities, training assumptions | W8 |
| 6 Kernel methods | Positive-definite kernels, feature maps, Gaussian processes | W7 |
| 7 Sparse kernel machines | SVM duality, margins, sparse Bayesian models | W7 |
| 8 Graphical models | Factorization, conditional independence, exact inference | W7 |
| 9 Mixture models and EM | Mixtures, responsibilities, EM lower bound and monotonicity | W7 |
| 10 Approximate inference | Variational identities, approximations with explicit guarantees | W7 |
| 11 Sampling methods | Invariance, detailed balance, convergence assumptions | W9 |
| 12 Continuous latent variables | PCA, probabilistic PCA, factor analysis | W7 |
| 13 Sequential data | Markov models, HMM recursion, state-space models | W7/W9 |
| 14 Combining models | Mixtures of experts, boosting/ensemble mathematical properties | W8 |
| A Data sets | Data provenance and empirical classification, not invented theorems | W0 inventory |
| B Probability distributions | Shared probability library and source mapping | W5 |
| C Properties of matrices | Shared algebra library and source mapping | W3 |
| D Calculus of variations | Variational hypotheses and functional derivatives | W7 |
| E Lagrange multipliers | Constraint qualifications and stationarity | W5 |

## Bishop and Bishop: Deep Learning, Foundations and Concepts

| Chapter | Coverage family | First eligible wave |
|---|---|---|
| 1 The deep learning revolution | Precise definitions/claims; classify historical and empirical prose | W0 inventory |
| 2 Probabilities | Probability identities and decision rules | W5 |
| 3 Standard distributions | Normalization, moments, conjugate calculations | W5 |
| 4 Single-layer networks: regression | Linear predictors, loss, Bayesian identities | W5 |
| 5 Single-layer networks: classification | Classification losses and gradients | W5/W8 |
| 6 Deep neural networks | Network semantics and differentiation | W8 |
| 7 Gradient descent | Descent guarantees, stochastic assumptions, convergence distinctions | W8 |
| 8 Backpropagation | Gradient evaluation, forward/reverse AD correctness | W8 |
| 9 Regularization | Penalty identities, invariance, precisely qualified guarantees | W8 |
| 10 Convolutional networks | Convolution definitions, shapes, boundary conventions, equivariance | W8 |
| 11 Structured distributions | Graph factorizations, conditional independence, inference | W7 |
| 12 Transformers | Attention/normalization identities, masks, shapes, permutation properties | W8 |
| 13 Graph neural networks | Message passing, permutation equivariance, readout invariance | W8 |
| 14 Sampling | Sampler semantics, invariance and convergence | W9 |
| 15 Discrete latent variables | Mixtures, latent inference, EM relationships | W7 |
| 16 Continuous latent variables | Latent Gaussian models and inference | W7 |
| 17 Generative adversarial networks | Objectives, distributional optima under explicit assumptions | W9 |
| 18 Normalizing flows | Bijections, Jacobians, density change of variables | W9 |
| 19 Autoencoders | Reconstruction, variational objectives, probabilistic semantics | W9 |
| 20 Diffusion models | Finite diffusion identities; continuous-time SDE results as separate packets | W9 |
| A Linear algebra | Matrix identities, traces/determinants, matrix derivatives, eigenvectors | W3/W4 |
| B Calculus of variations | Function-space assumptions and stationary conditions | W7 |
| C Lagrange multipliers | Constraint qualifications and stationarity | W5 |

## Inventory procedure and completion denominator

For every section, inspect prose claims, displays, worked examples, exercises,
solutions, and corrections. Assign each mathematical item a stable ID. A
definition can be represented and reviewed without being counted as a proved
theorem. A derivation may need several intermediate targets. Link them to one
source item and report both source-item completion and proof-obligation counts.

Every chapter starts `uninspected`; no count here implies zero eligible items.
A second reviewer checks section boundaries, missing displays, and exercise
lists before the inventory is `reviewed`. Empirical and expository items retain
a classification and reason. Difficult mathematics stays eligible and blocked
when necessary; it is not reclassified merely to improve the percentage.

Report separately: inventory completeness; definition representation;
theorem/derivation acceptance; exercise acceptance; algorithm specification,
exact correctness, convergence, and floating-point guarantees. Also separate
library reuse from new derivation. Shared results map to multiple source items
only after each correspondence is reviewed.

Each wave contains two queues: dependency-unlocking results and residual
sections/exercises. Start with a 70/30 effort allocation, a planning policy to
revisit using measured throughput. The residual queue cannot disappear when
capstones pass. W10 reconciles every remaining source item; it is not permission
to postpone difficult chapter inventory until the end.

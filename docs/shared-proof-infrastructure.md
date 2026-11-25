# Shared proof infrastructure and mathematical ownership

Harp keeps the knowledge, literature, education, and proof-study workbench.
Mathematical research repositories own their new theorem development and exact
acceptance statements. Shared infrastructure owns execution and evidence.

| Owner | Responsibility |
| --- | --- |
| Harp | Maintained explanations, sources and claim ledgers, curricula, exercises, exploratory examples, and education-specific acceptance policies |
| [lean-ml-foundations](https://github.com/phi9t/lean-ml-foundations) | New ML mathematics, explicit assumptions, theorem statements, proof sources, and selected proof contracts |
| Other mathematical pursuit repositories | Their respective mathematical scope, proof authority, toolchain pins, and research history |
| [insula-lean](https://github.com/phi9t/insula-lean) | Lean profiles, immutable dependencies, isolated builds, exact statement/axiom audits, and replay |
| [argonaut-control-plane](https://github.com/phi9t/argonaut-control-plane) | Job admission, dispatch, cancellation, results, and guarded publication |
| [insula](https://github.com/phi9t/insula) | General host execution, process lifetime, and isolation |

## First implemented consumer boundary

[The shared Linear contract](../lab/math/shared/README.md) binds one complete
mathematical target to exact existing source bytes and the pinned Lean
toolchain. An optional installed `insula-lean` projector creates its separate
managed project. Harp does not import a neighboring infrastructure checkout,
provide another generic exporter, or initialize a second scheduler.

The standalone `lab/math/lean` project and its existing wrapper commands remain
available. The new research seed copies 13 selected files into
`lean-ml-foundations`, preserves their original namespaces and byte provenance,
and keeps every Harp example in place. Its compact foundations aggregate selects
the six research modules through `PublicTheorems`; Harp's broader aggregate,
educational Cauchy-Schwarz policy, and course material remain under Harp's
acceptance rules. NNG4 and Crouzeix are outside this ML seed.

The seed contains finite-support probability and Bayesian identities, scalar
training and optimization models, finite linear-model identities, and algebraic
autodiff notation. Moving them does not widen their theorem claims. Future
research belongs in its domain repository, and a teaching update is an explicit
source/provenance change rather than an implicit synchronization of two mutable
copies.

## Execution and retirement conditions

The shared path must qualify the exact source, toolchain, immutable dependency
profile, expected theorem type, allowed axioms, and native cleanup before it
replaces an existing operation. A source projection is preparation evidence;
it is not a successful proof or isolated-execution receipt. No incomplete
cache, unsupported Lean version, or unrun native check may be silently promoted.

Harp's current `durable/`, `insula/`, and `runner/` implementations remain during
the transition. Retire an operation only after its external replacement proves
the relevant admission, restart/recovery, cancellation, cleanup, verification,
and approved-landing behavior. Keep original journals and old receipts bound
to the code that produced them. Educational dependency restrictions, such as
the Cauchy-Schwarz foundation policy, remain mathematical contracts even after
execution moves to the shared runner.

No existing RH/P2M or other research worktree, reservation, cache, or toolchain
is migrated by this change. Different mathematical projects keep their own
repository and toolchain boundaries; compatible immutable dependency artifacts
may be shared by exact profile identity.

## Argonaut estate

Use the prepared `/mnt/data_infra` estate environment for checkout, XDG,
toolchain, cache, temporary, state, and output locations. Project build outputs
remain separate even when immutable dependencies are shared. The entrypoint
must set the relevant environment before invoking tools; no bootstrap operation
may fall back to the root-mounted home directory.

Machine paths, provider credentials, inventories, and native receipts are
deployment inputs. Public contracts record immutable source and mathematical
identities without embedding those private details.

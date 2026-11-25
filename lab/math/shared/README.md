# Shared Lean execution contract

Harp owns the mathematical target and its educational interpretation.
`insula-lean` owns the generic projection, immutable dependency profile,
isolated compiler, exact statement and axiom audit, and replay. This directory
is a declarative consumer interface; it does not add a controller or change
Harp's standalone Lean package.

[linear.json](linear.json) selects the unchanged
`MathematicalFoundations/Linear.lean` source under `lab/math/lean` and the complete
type of `MathematicalFoundations.Linear.linear_finite_coordinate_reconstruction`.
It audits that one finite-coordinate statement; it does not audit every theorem
compiled in the module or establish a broader ML result.

[provenance.json](provenance.json) binds the contract hash, full repository source
paths, original Git blobs, and SHA-256 digests to Harp commit
`c50a2478e64d25fe8924dba33116f0f7f344bcde`. No theorem bytes, original namespaces,
toolchain pins, or standalone dependency locks are changed by projection.

Run `python3 scripts/check_shared_lean_contract.py` from the Harp root to check
the source/contract hashes and this selection without a Lean installation. The
result reports `proof_verified: false`; only the compiler/auditor can check the
mathematical statement.

## Optional shared execution

Use the projector from an approved installed `insula-lean` source revision,
recorded in the deployment receipt. The paths below name an installed tool and
explicit estate locations; no neighboring source checkout is required by Harp.

```sh
python3 /absolute/installed-proof-infra/scripts/prepare_project.py \
  --source-root /absolute/harp/lab/math/lean \
  --contract /absolute/harp/lab/math/shared/linear.json \
  --destination /absolute/harp-estate/projections/fresh-linear \
  --dependency mathlib=QUALIFIED_64_HEX_PROFILE_KEY

insula-lean build \
  --config /absolute/approved-runner-config.json \
  --project /absolute/harp-estate/projections/fresh-linear \
  --agent harp-linear
```

The profile key must identify qualified immutable dependencies matching Lean
`4.32.1` and the Mathlib revision pinned by Harp's existing lock. The projector
creates the restricted managed Lake configuration and package manifest only in
the new destination. It must reject a changed source hash or unsupported
toolchain rather than silently upgrading Harp or copying a mutable build tree.

An installed projector or a successful projection is not proof that native
execution is qualified. Keep the matching build, exact statement/axiom audit,
and cleanup receipt for a claim of isolated success. Local Harp verification
and its educational dependency policies remain available and retain their
existing scope.

On Argonaut, run through the prepared estate environment and keep all source,
XDG, tooling, cache, profile, temporary, and result paths beneath
`/mnt/data_infra`. When dispatched, the existing control plane owns stable job
keys, status, output, and cancellation; a completed command is not a deliverable
until its terminal result confirms cleanup. This contract initializes no job
store or background service.

See [the repository boundary](../../../docs/shared-proof-infrastructure.md) for
the research/education split and legacy-runtime retirement conditions.

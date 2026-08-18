# Training Dynamics formalization

The Training Dynamics Lean library formalizes controlled training-dynamics
models. It does not make empirical claims about deep networks.

`elan` is an external bootstrap prerequisite. Once it is installed, the pinned
toolchain and Lake manifest in `../lean/` define the reproducible Lean and
mathlib inputs. Training Dynamics shares that Lake root with the other Harp
Lean libraries so mathlib and dependency artifacts are cached once for proof
iteration.

Bootstrap elan from the matching platform artifact on the official,
versioned [elan v4.2.3 release](https://github.com/leanprover/elan/releases/tag/v4.2.3).
Before extracting or executing it, verify the artifact against the published
checksum or signature and obtain approval for the persistent tooling write; do
not execute an artifact without published verification data.

Run `mise run lean-training` from the repository root for focused iteration, or
`mise run lean-all` to warm and check the full shared root. The compatibility
wrapper `scripts/check_training_dynamics_lean.sh` delegates to the same shared
checker.

The wrapper normally checks `../lean/TrainingDynamics.lean` and
`../lean/TrainingDynamics/`. Its exact-shape `--project-for-test <directory>`
mode exists only for isolated tests; it has no environment-based project
override.

Before invoking Lake, the wrapper applies a deliberately strict textual
all-occurrences ban to the proof-placeholder spelling matched in every scoped
`.lean` file, excluding `.lake` output. This is not Lean-token-aware proof
detection: comments and string literals are also rejected. Remove every
matching occurrence before building.

# Training Dynamics formalization

This Lake project formalizes controlled training-dynamics models. It does not
make empirical claims about deep networks.

`elan` is an external bootstrap prerequisite. Once it is installed, the pinned
toolchain and Lake manifest define the reproducible Lean and mathlib inputs.

Bootstrap elan from the matching platform artifact on the official,
versioned [elan v4.2.3 release](https://github.com/leanprover/elan/releases/tag/v4.2.3).
Before extracting or executing it, verify the artifact against the published
checksum or signature and obtain approval for the persistent tooling write; do
not execute an artifact without published verification data.

Run `../../scripts/check_training_dynamics_lean.sh` from this directory, or
`mise run verify-lean` from the repository root.

The wrapper normally checks this fixed project directory. Its exact-shape
`--project-for-test <directory>` mode exists only for isolated tests; it has no
environment-based project override.

Before invoking Lake, the wrapper applies a deliberately strict textual
all-occurrences ban to the proof-placeholder spelling matched in every scoped
`.lean` file, excluding `.lake` output. This is not Lean-token-aware proof
detection: comments and string literals are also rejected. Remove every
matching occurrence before building.

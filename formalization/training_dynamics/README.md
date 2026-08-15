# Training Dynamics formalization

This Lake project formalizes controlled training-dynamics models. It does not
make empirical claims about deep networks.

`elan` is an external bootstrap prerequisite. Once it is installed, the pinned
toolchain and Lake manifest define the reproducible Lean and mathlib inputs.

After reviewing and explicitly approving the official installer, bootstrap
elan with:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh | sh -s -- -y --no-modify-path
```

This command downloads and runs a remote installer and writes persistent
tooling, so it must not be run without that approval.

Run `../../scripts/check_training_dynamics_lean.sh` from this directory, or
`mise run verify-lean` from the repository root.

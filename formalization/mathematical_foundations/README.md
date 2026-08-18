# Mathematical Foundations Lean formalization

This Lean library holds original Harp mathematical statements only. It does not
claim to formalize external prose, source texts, or theorems not expressly
authored for Harp.

## Reproducibility and verification

The required Lean toolchain is pinned in `../lean/lean-toolchain`, and the
committed `../lean/lake-manifest.json` pins the complete public dependency
graph. The shared Lake project uses mathlib's public `v4.32.1` tag and keeps
one `.lake` cache for Harp's Lean libraries.

Do not install Lean with a mutable installer pipe. Obtain versioned official
release metadata, download the matching versioned release artifact, and verify
its published checksum (and signature when available) before use. Install it
only with a task-scoped temporary `ELAN_HOME`; do not alter a global Lean
installation. Ordinary verification uses the shared wrapper with the committed
manifest. Run `lake update` in `../lean/` only as an intentional, reviewed lock
refresh; after that review, obtain compiled dependencies with
`lake exe cache get` and run `lake build`.

Run the repository wrapper from the repository root:

```sh
scripts/check_mathematical_foundations_lean.sh
```

For focused iteration, run `mise run lean-foundations`. To warm and check the
full shared root once, run `mise run lean-all`. The `mise` tasks provide the
task-scoped `ELAN_HOME` below `/private/tmp/harp-mathematical-foundations-elan`.
The wrapper canonicalizes the path and rejects every other location, so it
cannot use a global Elan home.

The wrapper always selects `../lean/MathematicalFoundations.lean` and
`../lean/MathematicalFoundations/` in normal use. Its sole test-only override
is `--project-for-test <directory>`. Before invoking Lake, it scans every Lean
source outside `.lake` and rejects every occurrence of `sorry`; the deliberate
all-occurrences ban prevents proof placeholders from entering the
formalization.

# Mathematical Foundations Lean formalization

This independent Lean project holds original Harp mathematical statements only.
It does not claim to formalize external prose, source texts, or theorems not
expressly authored for Harp.

## Reproducibility and verification

The required Lean toolchain is pinned in `lean-toolchain`, and the committed
`lake-manifest.json` pins the complete public dependency graph. The project
uses mathlib's public `v4.32.1` tag.

Do not install Lean with a mutable installer pipe. Obtain versioned official
release metadata, download the matching versioned release artifact, and verify
its published checksum (and signature when available) before use. Install it
only with a task-scoped temporary `ELAN_HOME`; do not alter a global Lean
installation. Ordinary verification uses `lake build` with the committed
manifest. Run `lake update` only as an intentional, reviewed lock refresh;
after that review, obtain compiled dependencies with `lake exe cache get` and
run `lake build`.

Run the repository wrapper from the repository root:

```sh
scripts/check_mathematical_foundations_lean.sh
```

The wrapper always selects this script's project directory in normal use. Its
sole test-only override is `--project-for-test <directory>`. Before invoking
Lake, it scans every Lean source outside `.lake` and rejects every occurrence
of `sorry`; the deliberate all-occurrences ban prevents proof placeholders
from entering the formalization.

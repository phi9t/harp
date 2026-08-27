# Crouzeix proof reproduction lab

This lab runs a prospective proof-search study. It does not replay an
unobserved historical run and does not certify mathematical correctness.

## Prepare

Extract the public historical prompt to a temporary path from its pinned Git
revision, then prepare a create-only run:

```sh
python3 labs/crouzeix_proof_reproduction/prepare_run.py \
  --run-dir labs/crouzeix_proof_reproduction/.runs/historical-001 \
  --arm historical \
  --historical-prompt /tmp/crouzeix_conjecture_prompt.txt
```

Use `--arm orchestrated` for the externally controlled treatment. Preparation
validates the 4,106-byte source and its pinned SHA-256, records one output-path
normalization, then discards the raw upstream bytes. The run tree retains only
the theorem, normalized execution prompt, Harp-authored phase prompts and
schemas, and identity receipts.

Prepare the ticketed expert-frontier arm separately:

```sh
python3 labs/crouzeix_proof_reproduction/prepare_frontier.py \
  --run-dir labs/crouzeix_proof_reproduction/.runs/expert-frontier-001 \
  --historical-prompt /tmp/crouzeix_conjecture_prompt.txt \
  --cli "$(command -v traecli)" \
  --model gpt-5.6-sol \
  --timeout-seconds 3600

python3 labs/crouzeix_proof_reproduction/prepare_frontier.py \
  --run-dir labs/crouzeix_proof_reproduction/.runs/expert-frontier-001 \
  --check
```

This creates the E-arm run specification, copies the blind expert/evaluator
prompt and schema inputs, records prompt/schema/config/CLI digests, and
pre-creates the five deterministic generation-zero expert tickets. It does not
start provider calls.

## Execute

Inspect `run_spec.json`, then run the prepared study:

```sh
python3 labs/crouzeix_proof_reproduction/run_experiment.py \
  labs/crouzeix_proof_reproduction/.runs/historical-001
```

Historical runs make one root call. Orchestrated runs make three independent
route calls, one controller call, an optional redirect, one synthesis call,
two independent critic calls, and one repair call. Ticketed frontier runs make
root expert calls only after the prepared runtime tickets exist. The harness
refuses partial resume or reuse of an existing call ID.

The resulting `run_receipt.json` reports typed call outcomes and usage
accounting. A candidate is not mathematically certified by that receipt. In
the orchestrated arm, automatic promotion requires an unchanged candidate,
zero critic findings, and no theorem-strength obligation; any changed repair
requires later independent re-review.

## Boundaries

- Blind generation has no network, proof manuscript, Lean source, Harp packet,
  or evaluator rubric.
- Every provider call is create-only and content-addressed.
- Failed, timed-out, malformed, and non-promoted runs are retained.
- Reference-aware review starts only after a candidate digest is sealed.
- Live model calls are explicit operator actions and never run from
  `mise run verify`.
- The Jin formal-validation lane is sealed by `formal_target.lock.json`. In the
  source map, the terminal row's `statement_sha256` is the formal target
  statement digest pinned by that lock, not a digest of the local theorem body
  after Harp renaming or assembly refactors.

## Published local certification artifacts

The proof-reproduction lab also consumes published local certification
artifacts under `evidence/crouzeix_conjecture/`. Those artifacts are
`complete-local` only: they certify named local declarations and bundle rows,
not peer review, publication, author endorsement, or complete boundedness.

Use these read-only validation commands when checking the published evidence:

```sh
python3 labs/crouzeix_proof_reproduction/proof_evidence.py validate --route all

python3 - <<'PY'
from pathlib import Path
from labs.crouzeix_proof_reproduction.local_formalization_validation import validate_local_formalization_bundle

validate_local_formalization_bundle(Path.cwd())
print("local bundle validated")
PY

cargo run -q -p harp -- sources verify
```

Equivalent installed CLI form: `harp sources verify`.

The six-row local bundle is
`evidence/crouzeix_conjecture/local_formalization/manifest.tsv` with SHA-256
`efc8469255219938b958d687d4a62506df937918bd659df18e47eaeb85a71de7`.
Its rows bind the terminal declarations
`CrouzeixConjecture.crouzeixConjecture`,
`CrouzeixConjecture.loristSchwenningerMainTheorem`, and
`CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem`, plus the closed-range
consequence declarations
`CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet`,
`CrouzeixConjecture.loristSchwenningerClosedOperatorNumericalRange_isTwoSpectralSet`,
and
`CrouzeixConjecture.harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet`.
The Lorist-Schwenninger closed-range row uses `source_node_id` `-`, so the
bundle certifies that consequence without attributing it to a named route node.

The approved experiment contract is
`docs/superpowers/specs/2026-08-14-crouzeix-proof-reproduction-design.md`.

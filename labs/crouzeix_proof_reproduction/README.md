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
  current checked-in preflight, production Jin runtime materialization is
  `blocked` by insufficient disk for the pinned Mathlib cache; fixture tests may
  exercise the receipt path, but they are not a Lean proof.

The approved experiment contract is
`docs/superpowers/specs/2026-08-14-crouzeix-proof-reproduction-design.md`.

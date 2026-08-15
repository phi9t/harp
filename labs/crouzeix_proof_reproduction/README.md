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

## Boundaries

- Blind generation has no network, proof manuscript, Lean source, Harp packet,
  or evaluator rubric.
- Every provider call is create-only and content-addressed.
- Failed, timed-out, malformed, and non-promoted runs are retained.
- Reference-aware review starts only after a candidate digest is sealed.
- Live model calls are explicit operator actions and never run from
  `mise run verify`.

The approved experiment contract is
`docs/superpowers/specs/2026-08-14-crouzeix-proof-reproduction-design.md`.

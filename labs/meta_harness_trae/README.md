# Meta-Harness TRAE supplement

This opt-in supplement records one bounded TRAE CLI proposer iteration against
the pinned Meta-Harness text-classification interface. It validates proposal
shape and candidate behavior only. It does not reproduce a benchmark score,
paid-model evaluation, held-out result, or recursive self-improvement claim.

Run the deterministic tests:

```sh
python3 -m unittest discover -s labs/meta_harness_trae/tests -v
```

The live proposer command is intentionally outside `mise run verify`.
`prepare_run.py` creates an ignored workspace, `run_once.py` invokes TRAE once,
and `publish_run.py` validates and normalizes the result before producing the
tracked receipt and deterministic raw archive. The recorded run under
`evidence/meta_harness/trae_run/` is the release-gate input.

Safe raw-archive extraction:

```sh
python3 labs/meta_harness_trae/unpack.py \
  evidence/meta_harness/trae_run/raw_traecli_run.tar.gz \
  evidence/meta_harness/trae_run/raw_members.tsv \
  /tmp/harp_meta_harness_trae_run
```

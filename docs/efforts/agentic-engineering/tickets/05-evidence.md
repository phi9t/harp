# 05 evidence: verification and evidence records (F9)

Type: feature
Status: open
Owner:
Blocked by:
Claimed:

## Description
Build `tools/agents/evidence.py` exactly to `milestone-b/contracts/evidence-record.md` (schema, append-only claim sidecar with a hash chain, Python API, CLI). Receipts elsewhere copied declared values and hashed the wrong thing; run evidence must name what ran and separate measured from declared.

## Interface
`evidence.py record RUN_DIR [--repo R]`, `verify DIGEST`, `show DIGEST`; module API `record_run`, `load`, `verify`, `append_claim_outcome`. A run directory is one produced by `tools/agents/worker.py launch` (files `run.json`, `prompt.md`, `events.jsonl`, `verdict.json`, `last-message.md`, `stderr.log`); tests build such a directory by hand.

## Acceptance
- [ ] Test: record then verify passes; the digest is sha256 of the canonical record without its digest field.
- [ ] Test: tampering with the record file, with the claims sidecar (including deleting a middle line), or with a recomputable input (the events file, the prompt, the diff) makes verify fail and name the problem.
- [ ] Test: two records of identical runs have distinct digests.
- [ ] Test: measured and declared are never merged; a field has exactly one provenance; the model name from run.json is declared, the prompt digest is measured.
- [ ] Test: append_claim_outcome is append-only and hash-chained; the record file is never rewritten.
- [ ] Test: CORENIUS_EVIDENCE and out_dir select the directory; the default is gitignored (assert that .corenius/ is already ignored by the repo's .gitignore, and do not edit .gitignore yourself).
- [ ] tools/agents/checks.d/50-evidence.sh runs the suite and exits non-zero on failure.

## Comments

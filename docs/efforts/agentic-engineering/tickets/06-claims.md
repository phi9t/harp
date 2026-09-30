# 06 claims: claim checking for lead verification (F10)

Type: feature
Status: open
Owner:
Blocked by:
Claimed:

## Description
Build `tools/agents/claims.py`. In the first trial the lead verified a sample of the worker's verdicts by hand; that step is the value of the lead and needs a small tool. Write against `milestone-b/contracts/evidence-record.md`: the module `evidence` is being written by another worker at the same time, so import it lazily and, in your tests, use a small stub module that implements `append_claim_outcome(digest, claim_id, passed, command, output_digest, out_dir=None)`.

## Interface
Verification reports look like `docs/reference/milano.md` and `docs/efforts/multi-agent-bench/evidence/research-*.md`: numbered or bulleted lines, each with a tag (`[R]` read, `[X]` executed, `[I]` inferred) and often a citation in backticks, either `git -C REPO show SHA:PATH` or `path:line`.
`claims.py sample FILE --n N [--seed S] [--tags R,X]` parses claims, gives each a stable id (sha256 prefix of the normalized text), picks N deterministically (same file, n, seed gives the same claims; changing the seed changes the pick), and prints for each the id, the text and the exact command the lead should run (`git show` when cited, else `sed -n 'L,Lp' path`; none for an uncited claim, which is flagged UNCITED).
`claims.py record FILE --claim ID (--passed | --failed) [--output FILE] [--evidence DIGEST]` appends the outcome to a local JSONL under `.corenius/claims/` (gitignored) and, with --evidence, calls `evidence.append_claim_outcome` with the digest of the output file.
`claims.py summary FILE` prints sampled versus recorded outcomes.

## Acceptance
- [ ] Test on a report in the trial-1 format (use a short fixture with the three tag kinds and both citation forms).
- [ ] Test: sampling is deterministic, seed-sensitive, never returns more than exist, and skips nothing silently (an uncited claim is flagged).
- [ ] Test: record writes the JSONL outcome and calls the evidence stub with the output file digest; a missing evidence module with --evidence fails with a clear message.
- [ ] Test: the printed command for a path:line citation reproduces the cited line when run.
- [ ] tools/agents/checks.d/60-claims.sh runs the suite and exits non-zero on failure.

## Comments

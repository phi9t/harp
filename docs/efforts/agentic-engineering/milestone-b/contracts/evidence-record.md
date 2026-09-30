# Contract: evidence records (F9, used by F10)

Location: `<main worktree>/.corenius/evidence/` (gitignored), overridable with the `CORENIUS_EVIDENCE` environment variable or an `out_dir` argument.

**Record file** `<hex>.json`, where `<hex>` is the sha256 of the canonical JSON (sorted keys, no whitespace) of the record *without* its `digest` field:

```
{
  "schema": "corenius.evidence.v1",
  "digest": "sha256:<hex>",
  "kind": "worker-run",
  "recorded_at": "<UTC time>",
  "fields": {
    "<name>": {"value": <json>, "provenance": "measured" | "declared", "source": "<how it was measured, or where it was declared>"}
  }
}
```

`measured` fields are recomputed by the tool from files (run dir, git, the event stream): worker model flags (from `run.json`, declared: the config says so), prompt digest (sha256 of `prompt.md`),
base sha, worktree diff digest (sha256 of `git diff` of the worktree at record time against the base), event-stream digest (sha256 of `events.jsonl`), verdict (from `verdict.json`), timings.
`declared` fields are what a file or a person claims without the tool recomputing it (for example the model name in `run.json`). A field is never both; `measured` and `declared`
values for the same name are stored under distinct names. Two identical runs recorded twice give two distinct records (`recorded_at` and a random `nonce` field are part of the digest input).

**Append-only outcomes.** Records are never modified. Claim outcomes go to a sidecar `<hex>.claims.jsonl`, one JSON object per line
`{"claim_id", "passed", "command", "output_digest", "recorded_at", "prev": "<sha256 of the previous line or of the record digest>"}` (a hash chain).

**Python API** (module `evidence`, importable from `tools/agents`):
- `record_run(run_dir, repo=None, out_dir=None) -> str` returns `"sha256:<hex>"`.
- `load(digest, out_dir=None) -> dict`.
- `verify(digest, out_dir=None) -> list[str]` returns a list of problems (empty means it verifies): record digest mismatch, claims chain broken, recomputable fields that no longer match.
- `append_claim_outcome(digest, claim_id, passed, command, output_digest, out_dir=None) -> None`.

**CLI:** `evidence.py record RUN_DIR [--repo R]` prints the digest; `evidence.py verify DIGEST` exits 0 when there are no problems, 1 otherwise and prints them;
`evidence.py show DIGEST` prints the record and its outcomes.

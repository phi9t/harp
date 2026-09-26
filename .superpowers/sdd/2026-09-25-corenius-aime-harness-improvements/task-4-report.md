## Task 4 Report: Fresh Provider Per Sample Flag And Execution Mode Wiring

### What I implemented

- Added an operator-selectable CLI flag `--fresh-provider-per-sample` to:
  - `corenius eval aime`
  - `corenius eval aime-sanity`
- Wired the flag through CLI option construction into:
  - `AimeEvalOptions::execution_mode`
  - `AimeSanityOptions::execution_mode` (new field), then forwarded in `run_aime_sanity`
- Made provider execution mode explicit by extracting the per-prompt ACP execution into:
  - `run_aime_provider_prompt(options, provider, prompt)`
  - This helper chooses:
    - `SharedProvider`: clone/reuse the passed provider
    - `FreshProviderPerSample`: construct a new `TraeCliAcpProvider` per sample
- Verified that `manifest.json` and `run_metrics.json` record `execution_mode` as:
  - `"fresh_provider_per_sample"` when the flag is set
  - unchanged default `"shared_provider"` otherwise

### Tests and results

- Focused tests:
  - `cargo test eval_aime_sanity_records_fresh_provider_per_sample_mode` (PASS)
  - `cargo test builds_manifest_with_dataset_digest` (PASS)
- Full suite:
  - `cargo test` (PASS)

### TDD Evidence

#### RED

Command:

```bash
cargo test eval_aime_sanity_records_fresh_provider_per_sample_mode
```

Output (expected failure):

```
error: unexpected argument '--fresh-provider-per-sample' found
```

Why this was expected:
- The new CLI flag did not exist yet, so clap rejected the unknown argument and exited with code 2.

#### GREEN

Commands:

```bash
cargo test eval_aime_sanity_records_fresh_provider_per_sample_mode
cargo test builds_manifest_with_dataset_digest
```

Output:
- Both tests passed after wiring the flag + execution_mode propagation.

### Files changed

- `src/cli.rs`
- `src/eval.rs`
- `tests/cli_tui.rs`

### Self-review findings

- Manifest schema_version remains `1` while including `execution_mode` (per controller ruling).
- Default behavior remains `SharedProvider` when the flag is not set.
- Provider execution mode is now an explicit choice in evaluator state via `run_aime_provider_prompt`, while still spawning an ACP process per prompt under the hood (consistent with current ACP helper behavior).
- Commit trailer requirement enforced: commit message ends with exactly one `Co-authored-by: TRAE CLI <noreply@bytedance.com>` line.

### Concerns

- None.

## Task 4 Fix Report (Review Follow-up): Add non-sanity fresh mode CLI coverage

### What I changed

- Added CLI-level test coverage that `corenius eval aime --fresh-provider-per-sample` records:
  - `manifest.json.execution_mode == "fresh_provider_per_sample"`
  - `run_metrics.json.execution_mode == "fresh_provider_per_sample"`
- The test uses a cheap fixture:
  - local cached dataset file for `2025-I` only
  - `--years 2025-I --limit 1` to keep runtime small
  - existing `fake_traecli` via `TRAECLI_PATH`

### Tests and results

- Focused:
  - `cargo test eval_aime_records_fresh_provider_per_sample_mode` (PASS)
- Full:
  - `cargo test` (PASS)

### Commands and output

Focused command:

```bash
cargo test eval_aime_records_fresh_provider_per_sample_mode
```

Output:

- `test eval_aime_records_fresh_provider_per_sample_mode ... ok`

Full suite command:

```bash
cargo test
```

Output:

- All tests passed.

### Files changed

- `tests/cli_tui.rs`

### Concerns

- None.


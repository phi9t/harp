# Corenius AIME Harness Improvements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add auditable AIME regrading, scripted ACP provider faults, run metrics, and optional per-sample provider isolation while keeping Corenius' evaluator AIME-specific.

**Architecture:** Keep the public CLI thin in `src/cli.rs` and deepen the existing AIME evaluator in `src/eval.rs`. Reuse the current `AimeSample` artifact as the durable raw record, add small private scoring/artifact helpers, and exercise provider behavior through the real ACP adapter by pointing `TRAECLI_PATH` at a scripted executable in CLI tests.

**Tech Stack:** Rust, Tokio, Clap, Serde JSON/JSONL, assert_cmd, tempfile, Python-based executable fixtures for ACP CLI integration tests.

**Spec:** `docs/superpowers/specs/2026-09-24-corenius-aime-harness-design.md`

## Global Constraints

- Keep Corenius focused on AIME; do not introduce a generic benchmark framework.
- Original run artifacts are immutable; replay and regrade always write independent run directories.
- Regrade updates only derived evaluator fields: `extracted_answer`, `passed`, `score`, and `failure_kind`.
- `passed` and `score` remain the scoring source of truth.
- Prompt contamination is only a failure kind when the sample does not pass.
- `eval aime-sanity` remains the cheap deterministic gate for scorer, artifact, and provider-path changes.
- Full `eval aime --years 2024 2025-I 2025-II` remains an explicit expensive gate, not an automatic test.
- Pi-specific harness integration is deferred until the actual Pi harness source is available.
- Preserve existing artifact compatibility for `samples.jsonl`, `summary.json`, `triage.md`, and `manifest.json`.
- Use the current production ACP provider adapter path in CLI tests; do not add a provider shortcut inside `src/eval.rs`.

## Review Focus

- Empty source artifacts: `aime-regrade --from-run` against an empty `samples.jsonl` must fail clearly instead of emitting empty success artifacts; covered in Task 2.
- Malformed prior samples: `aime-regrade` must report the bad line number from `samples.jsonl`; covered in Task 2.
- Metric percentiles on small runs: p50/p95 must be deterministic for one or two samples and preserve sample order; covered in Task 3.
- Execution mode truthfulness: manifests and metrics must record the selected evaluator mode, and tests must pin both default and `--fresh-provider-per-sample`; covered in Task 4.
- Scripted provider cursor safety: a CLI test script must fail if requests arrive out of order, if script entries are exhausted, or if entries remain unconsumed when full consumption is required; covered in Task 5.

---

## File Structure

- Modify `src/eval.rs`
  - Add scorer helper structs/functions for regrade.
  - Add `AimeRegradeOptions`.
  - Add regrade artifact generation and `scorer-delta.json`.
  - Add `AimeExecutionMode`, run metrics structs, aggregation, and artifact writing.
  - Thread execution mode through `AimeEvalOptions` and `AimeSanityOptions`.
- Modify `src/cli.rs`
  - Add `eval aime-regrade`.
  - Add default regrade run directory helper.
  - Add `--fresh-provider-per-sample` to `eval aime` and `eval aime-sanity`.
- Create `tests/support/mod.rs`
  - Export test support helpers.
- Create `tests/support/scripted_acp.rs`
  - Generate an executable ACP test double with FIFO scripted behavior.
- Modify `tests/cli_tui.rs`
  - Use the scripted ACP helper for new provider-path tests and keep existing tests compatible.

---

### Task 1: Shared AIME Scorer And Regrade Delta Primitives

**Files:**
- Modify: `src/eval.rs`

**Interfaces:**
- Consumes: existing `AimeSample`, `AimeFailureKind`, `normalize_answer`, `extract_final_answer`, and `classify_failure`.
- Produces:
  - `#[derive(Clone, Debug, PartialEq, Eq)] struct AimeScoredFields { extracted_answer: String, passed: bool, score: u8, failure_kind: Option<AimeFailureKind> }`
  - `fn score_aime_response(expected_answer: &str, response: &str, error: Option<&str>) -> AimeScoredFields`
  - `fn regrade_sample(sample: &AimeSample) -> AimeSample`
  - `#[derive(Clone, Debug, Serialize, PartialEq, Eq)] struct AimeScorerDelta { id: String, old_extracted_answer: String, new_extracted_answer: String, old_passed: bool, new_passed: bool, old_failure_kind: Option<AimeFailureKind>, new_failure_kind: Option<AimeFailureKind> }`
  - `fn scorer_delta(before: &AimeSample, after: &AimeSample) -> Option<AimeScorerDelta>`

- [ ] **Step 1: Write failing scorer tests**

Add these tests inside `#[cfg(test)] mod tests` in `src/eval.rs`:

```rust
#[test]
fn score_aime_response_requires_final_marker() {
    let scored = score_aime_response("70", "The answer is 70.", None);

    assert_eq!(scored.extracted_answer, "");
    assert!(!scored.passed);
    assert_eq!(scored.score, 0);
    assert_eq!(scored.failure_kind, Some(AimeFailureKind::NoFinal));
}

#[test]
fn regrade_sample_updates_only_derived_fields() {
    let mut old = sample("2025-I:I-1", "2025-I", false, None);
    old.expected_answer = "70".to_string();
    old.extracted_answer = "999".to_string();
    old.response = "updated reasoning\nFINAL: 70\n".to_string();
    old.failure_kind = Some(AimeFailureKind::WrongFinal);

    let new = regrade_sample(&old);

    assert_eq!(new.id, old.id);
    assert_eq!(new.year, old.year);
    assert_eq!(new.tags, old.tags);
    assert_eq!(new.source, old.source);
    assert_eq!(new.expected_answer, old.expected_answer);
    assert_eq!(new.elapsed_seconds, old.elapsed_seconds);
    assert_eq!(new.return_status, old.return_status);
    assert_eq!(new.error, old.error);
    assert_eq!(new.prompt, old.prompt);
    assert_eq!(new.response, old.response);
    assert_eq!(new.usage, old.usage);
    assert_eq!(new.extracted_answer, "70");
    assert!(new.passed);
    assert_eq!(new.score, 1);
    assert_eq!(new.failure_kind, None);
}

#[test]
fn scorer_delta_reports_changed_and_skips_unchanged_samples() {
    let mut old = sample("2025-I:I-1", "2025-I", false, None);
    old.expected_answer = "70".to_string();
    old.extracted_answer = "999".to_string();
    old.response = "FINAL: 70\n".to_string();
    old.failure_kind = Some(AimeFailureKind::WrongFinal);
    let new = regrade_sample(&old);

    let delta = scorer_delta(&old, &new).unwrap();
    assert_eq!(delta.id, "2025-I:I-1");
    assert_eq!(delta.old_extracted_answer, "999");
    assert_eq!(delta.new_extracted_answer, "70");
    assert!(!delta.old_passed);
    assert!(delta.new_passed);
    assert_eq!(delta.old_failure_kind, Some(AimeFailureKind::WrongFinal));
    assert_eq!(delta.new_failure_kind, None);

    assert_eq!(scorer_delta(&new, &new), None);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test score_aime_response_requires_final_marker
cargo test regrade_sample_updates_only_derived_fields
cargo test scorer_delta_reports_changed_and_skips_unchanged_samples
```

Expected: FAIL because `score_aime_response`, `regrade_sample`, and `scorer_delta` do not exist.

- [ ] **Step 3: Add scorer primitives**

In `src/eval.rs`, add the structs near `AimeFailureKind`:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
struct AimeScoredFields {
    extracted_answer: String,
    passed: bool,
    score: u8,
    failure_kind: Option<AimeFailureKind>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct AimeScorerDelta {
    id: String,
    old_extracted_answer: String,
    new_extracted_answer: String,
    old_passed: bool,
    new_passed: bool,
    old_failure_kind: Option<AimeFailureKind>,
    new_failure_kind: Option<AimeFailureKind>,
}
```

Add these helper functions near `classify_failure`:

```rust
fn score_aime_response(
    expected_answer: &str,
    response: &str,
    error: Option<&str>,
) -> AimeScoredFields {
    let expected = normalize_answer(expected_answer);
    let extracted = extract_final_answer(response).unwrap_or_default();
    let passed = error.is_none() && extracted == expected;
    AimeScoredFields {
        extracted_answer: extracted,
        passed,
        score: u8::from(passed),
        failure_kind: classify_failure(passed, response, error),
    }
}

fn regrade_sample(sample: &AimeSample) -> AimeSample {
    let scored = score_aime_response(
        &sample.expected_answer,
        &sample.response,
        sample.error.as_deref(),
    );
    let mut regraded = sample.clone();
    regraded.extracted_answer = scored.extracted_answer;
    regraded.passed = scored.passed;
    regraded.score = scored.score;
    regraded.failure_kind = scored.failure_kind;
    regraded
}

fn scorer_delta(before: &AimeSample, after: &AimeSample) -> Option<AimeScorerDelta> {
    if before.extracted_answer == after.extracted_answer
        && before.passed == after.passed
        && before.failure_kind == after.failure_kind
    {
        return None;
    }
    Some(AimeScorerDelta {
        id: before.id.clone(),
        old_extracted_answer: before.extracted_answer.clone(),
        new_extracted_answer: after.extracted_answer.clone(),
        old_passed: before.passed,
        new_passed: after.passed,
        old_failure_kind: before.failure_kind.clone(),
        new_failure_kind: after.failure_kind.clone(),
    })
}
```

In `run_aime_cases`, replace the local extracted/pass/failure calculation with:

```rust
let scored = score_aime_response(&case.answer, &response, error.as_deref());
let sample = AimeSample {
    id: case.id,
    year: case.year,
    tags: case.tags,
    source: loaded.source.clone(),
    expected_answer: normalize_answer(&case.answer),
    extracted_answer: scored.extracted_answer,
    passed: scored.passed,
    score: scored.score,
    elapsed_seconds,
    return_status,
    error,
    prompt,
    response,
    failure_kind: scored.failure_kind,
    usage,
};
```

- [ ] **Step 4: Run tests to verify they pass**

Run:

```bash
cargo test score_aime_response_requires_final_marker
cargo test regrade_sample_updates_only_derived_fields
cargo test scorer_delta_reports_changed_and_skips_unchanged_samples
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/eval.rs
git commit -m "refactor: share AIME scoring for regrade"
```

---

### Task 2: `eval aime-regrade` Command And Scorer Delta Artifacts

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/eval.rs`
- Modify: `tests/cli_tui.rs`

**Interfaces:**
- Consumes: `load_replay_samples`, `regrade_sample`, `scorer_delta`, `summarize_samples`, `render_triage_markdown`, `write_pretty_json`, `append_jsonl`.
- Produces:
  - `pub struct AimeRegradeOptions { pub workspace: PathBuf, pub from_run: PathBuf, pub run_dir: PathBuf }`
  - `#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)] #[serde(rename_all = "snake_case")] pub enum AimeExecutionMode { SharedProvider, FreshProviderPerSample }`
  - `pub async fn run_aime_regrade(options: AimeRegradeOptions) -> anyhow::Result<AimeSummary>`
  - `fn default_aime_regrade_run_dir(workspace: &Path) -> PathBuf`
  - CLI: `corenius eval aime-regrade --from-run <dir> [--workspace <path>] [--run-dir <path>]`

- [ ] **Step 1: Write failing unit tests for regrade run data**

Add these tests to `src/eval.rs`:

```rust
#[tokio::test]
async fn run_aime_regrade_writes_regraded_samples_and_delta() {
    let temp = tempfile::tempdir().unwrap();
    let from_run = temp.path().join("from-run");
    let regrade_run = temp.path().join("regrade-run");
    std::fs::create_dir_all(&from_run).unwrap();

    let mut old = sample("2025-I:I-1", "2025-I", false, None);
    old.expected_answer = "70".to_string();
    old.extracted_answer = "999".to_string();
    old.response = "new scorer sees final\nFINAL: 70\n".to_string();
    old.failure_kind = Some(AimeFailureKind::WrongFinal);
    std::fs::write(
        from_run.join("samples.jsonl"),
        format!("{}\n", serde_json::to_string(&old).unwrap()),
    )
    .unwrap();

    let summary = run_aime_regrade(AimeRegradeOptions {
        workspace: temp.path().to_path_buf(),
        from_run: from_run.clone(),
        run_dir: regrade_run.clone(),
    })
    .await
    .unwrap();

    assert_eq!(summary.total, 1);
    assert_eq!(summary.correct, 1);
    let original = std::fs::read_to_string(from_run.join("samples.jsonl")).unwrap();
    assert!(original.contains("\"extracted_answer\":\"999\""));

    let regraded = std::fs::read_to_string(regrade_run.join("samples.jsonl")).unwrap();
    assert!(regraded.contains("\"extracted_answer\":\"70\""));
    assert!(regraded.contains("\"passed\":true"));

    let delta: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(regrade_run.join("scorer-delta.json")).unwrap())
            .unwrap();
    assert_eq!(delta["schema_version"], 1);
    assert_eq!(delta["changed_sample_count"], 1);
    assert_eq!(delta["changes"][0]["id"], "2025-I:I-1");

    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(regrade_run.join("manifest.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["dataset"], format!("regrade:{}", from_run.display()));
    assert_eq!(manifest["source_run_dir"], from_run.display().to_string());
    assert_eq!(manifest["regraded_from_sample_count"], 1);
}

#[tokio::test]
async fn run_aime_regrade_fails_for_empty_samples() {
    let temp = tempfile::tempdir().unwrap();
    let from_run = temp.path().join("from-run");
    std::fs::create_dir_all(&from_run).unwrap();
    std::fs::write(from_run.join("samples.jsonl"), "").unwrap();

    let error = run_aime_regrade(AimeRegradeOptions {
        workspace: temp.path().to_path_buf(),
        from_run,
        run_dir: temp.path().join("regrade-run"),
    })
    .await
    .unwrap_err();

    assert!(error.to_string().contains("AIME regrade selection is empty"));
}

#[tokio::test]
async fn run_aime_regrade_reports_malformed_sample_line() {
    let temp = tempfile::tempdir().unwrap();
    let from_run = temp.path().join("from-run");
    std::fs::create_dir_all(&from_run).unwrap();
    std::fs::write(
        from_run.join("samples.jsonl"),
        format!(
            "{}\nnot-json\n",
            serde_json::to_string(&sample("2025-I:I-1", "2025-I", false, None)).unwrap()
        ),
    )
    .unwrap();

    let error = run_aime_regrade(AimeRegradeOptions {
        workspace: temp.path().to_path_buf(),
        from_run,
        run_dir: temp.path().join("regrade-run"),
    })
    .await
    .unwrap_err();

    let message = format!("{error:#}");
    assert!(message.contains("samples.jsonl"));
    assert!(message.contains("line 2"));
}
```

- [ ] **Step 2: Write failing CLI tests**

Add these tests to `tests/cli_tui.rs`:

```rust
#[test]
fn eval_aime_regrade_writes_independent_artifacts() {
    let temp = tempdir().unwrap();
    let from_run = temp.path().join("from-run");
    let regrade_run = temp.path().join("regrade-run");
    fs::create_dir_all(&from_run).unwrap();
    fs::write(
        from_run.join("samples.jsonl"),
        prior_sample_json("2025-I:I-1", "2025-I", "70", false) + "\n",
    )
    .unwrap();

    Command::cargo_bin("corenius")
        .unwrap()
        .args([
            "eval",
            "aime-regrade",
            "--from-run",
            from_run.to_str().unwrap(),
            "--workspace",
            temp.path().to_str().unwrap(),
            "--run-dir",
            regrade_run.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("\"total\": 1"));

    assert!(from_run.join("samples.jsonl").exists());
    assert!(regrade_run.join("manifest.json").exists());
    assert!(regrade_run.join("samples.jsonl").exists());
    assert!(regrade_run.join("summary.json").exists());
    assert!(regrade_run.join("triage.md").exists());
    assert!(regrade_run.join("scorer-delta.json").exists());
}

#[test]
fn eval_aime_regrade_fails_for_missing_samples() {
    let temp = tempdir().unwrap();
    let from_run = temp.path().join("from-run");
    fs::create_dir_all(&from_run).unwrap();

    Command::cargo_bin("corenius")
        .unwrap()
        .args([
            "eval",
            "aime-regrade",
            "--from-run",
            from_run.to_str().unwrap(),
            "--workspace",
            temp.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("samples.jsonl"));
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run:

```bash
cargo test run_aime_regrade_writes_regraded_samples_and_delta
cargo test run_aime_regrade_fails_for_empty_samples
cargo test run_aime_regrade_reports_malformed_sample_line
cargo test eval_aime_regrade_writes_independent_artifacts
cargo test eval_aime_regrade_fails_for_missing_samples
```

Expected: FAIL because the command and regrade runner do not exist.

- [ ] **Step 4: Add regrade options and manifest context**

In `src/eval.rs`, add:

```rust
#[derive(Clone, Debug)]
pub struct AimeRegradeOptions {
    pub workspace: PathBuf,
    pub from_run: PathBuf,
    pub run_dir: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AimeExecutionMode {
    SharedProvider,
    FreshProviderPerSample,
}

#[derive(Clone, Debug)]
enum AimeDerivedRunKind {
    Replay,
    Regrade,
}

#[derive(Clone, Debug)]
struct AimeDerivedRunContext {
    kind: AimeDerivedRunKind,
    source_run_dir: PathBuf,
    source_sample_count: usize,
}
```

Replace `AimeReplayContext` usage with `AimeDerivedRunContext`. Preserve existing replay manifest fields and add this optional manifest field:

```rust
pub execution_mode: AimeExecutionMode,
#[serde(skip_serializing_if = "Option::is_none")]
pub regraded_from_sample_count: Option<usize>,
```

In `build_manifest`, set:

```rust
execution_mode: options.execution_mode,
source_run_dir: derived_context
    .as_ref()
    .map(|context| context.source_run_dir.display().to_string()),
replayed_from_sample_count: derived_context.as_ref().and_then(|context| match context.kind {
    AimeDerivedRunKind::Replay => Some(context.source_sample_count),
    AimeDerivedRunKind::Regrade => None,
}),
regraded_from_sample_count: derived_context.as_ref().and_then(|context| match context.kind {
    AimeDerivedRunKind::Replay => None,
    AimeDerivedRunKind::Regrade => Some(context.source_sample_count),
}),
```

Add this delta wrapper near `AimeScorerDelta`:

```rust
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct AimeScorerDeltaReport {
    schema_version: u8,
    source_run_dir: String,
    changed_sample_count: usize,
    changes: Vec<AimeScorerDelta>,
}
```

Add `pub execution_mode: AimeExecutionMode` to `AimeEvalOptions` and set it to `AimeExecutionMode::SharedProvider` in every current production and test construction. Task 4 will make this operator-selectable.

- [ ] **Step 5: Implement `run_aime_regrade`**

Add this function near `run_aime_replay`:

```rust
pub async fn run_aime_regrade(options: AimeRegradeOptions) -> anyhow::Result<AimeSummary> {
    let original_samples = load_replay_samples(&options.from_run)?;
    if original_samples.is_empty() {
        bail!("AIME regrade selection is empty");
    }

    tokio::fs::create_dir_all(&options.run_dir)
        .await
        .with_context(|| format!("create run dir {}", options.run_dir.display()))?;
    let samples_path = options.run_dir.join("samples.jsonl");
    let _ = tokio::fs::remove_file(&samples_path).await;

    let loaded = LoadedDataset {
        source: format!("regrade:{}", options.from_run.display()),
        cases: Vec::new(),
        cache_files: vec![dataset_cache_file(&options.from_run.join("samples.jsonl"))?],
    };
    let eval_options = AimeEvalOptions {
        workspace: options.workspace,
        source: AimeDatasetSource::Replay {
            from_run: options.from_run.clone(),
        },
        run_dir: options.run_dir.clone(),
        model: None,
        timeout_ms: 0,
        stream: false,
        selection: AimeSelection {
            years: years_from_samples(&original_samples),
            offset: 0,
            limit: None,
            filter_id: Vec::new(),
            include_timeout_probe: true,
        },
        execution_mode: AimeExecutionMode::SharedProvider,
    };
    let derived_context = AimeDerivedRunContext {
        kind: AimeDerivedRunKind::Regrade,
        source_run_dir: options.from_run.clone(),
        source_sample_count: original_samples.len(),
    };
    let manifest = build_manifest(
        &eval_options,
        &loaded,
        Some(derived_context.clone()),
        started_at_utc(),
        std::env::args().collect(),
    );
    write_pretty_json(&options.run_dir.join("manifest.json"), &manifest).await?;

    let mut regraded_samples = Vec::new();
    let mut changes = Vec::new();
    for old in original_samples {
        let new = regrade_sample(&old);
        if let Some(delta) = scorer_delta(&old, &new) {
            changes.push(delta);
        }
        append_jsonl(&samples_path, &new).await?;
        regraded_samples.push(new);
    }

    let summary = summarize_samples(&regraded_samples, &eval_options);
    write_pretty_json(&options.run_dir.join("summary.json"), &summary).await?;
    tokio::fs::write(
        options.run_dir.join("triage.md"),
        render_triage_markdown(&summary, &regraded_samples, Some(&derived_context)),
    )
    .await?;
    write_pretty_json(
        &options.run_dir.join("scorer-delta.json"),
        &AimeScorerDeltaReport {
            schema_version: 1,
            source_run_dir: options.from_run.display().to_string(),
            changed_sample_count: changes.len(),
            changes,
        },
    )
    .await?;
    Ok(summary)
}
```

Add:

```rust
fn years_from_samples(samples: &[AimeSample]) -> Vec<String> {
    let mut years = Vec::new();
    for sample in samples {
        if !years.contains(&sample.year) {
            years.push(sample.year.clone());
        }
    }
    years
}
```

- [ ] **Step 6: Add CLI command**

In `src/cli.rs`, import `AimeRegradeOptions` and `run_aime_regrade`.

Add an `EvalCommand::AimeRegrade` variant:

```rust
AimeRegrade {
    #[arg(long)]
    from_run: PathBuf,
    #[arg(long)]
    workspace: Option<PathBuf>,
    #[arg(long)]
    run_dir: Option<PathBuf>,
},
```

Add this match arm:

```rust
EvalCommand::AimeRegrade {
    from_run,
    workspace,
    run_dir,
} => {
    let workspace = workspace_or_current(workspace)?;
    let run_dir = run_dir.unwrap_or_else(|| default_aime_regrade_run_dir(&workspace));
    let summary = run_aime_regrade(AimeRegradeOptions {
        workspace,
        from_run,
        run_dir,
    })
    .await?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
```

Add this default next to existing AIME run-dir helpers:

```rust
fn default_aime_regrade_run_dir(workspace: &Path) -> PathBuf {
    workspace
        .join(".scratch/aime-benchmark")
        .join(format!("regrade-{}", timestamp_for_path()))
}
```

- [ ] **Step 7: Run tests to verify they pass**

Run:

```bash
cargo test run_aime_regrade_writes_regraded_samples_and_delta
cargo test run_aime_regrade_fails_for_empty_samples
cargo test run_aime_regrade_reports_malformed_sample_line
cargo test eval_aime_regrade_writes_independent_artifacts
cargo test eval_aime_regrade_fails_for_missing_samples
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add src/eval.rs src/cli.rs tests/cli_tui.rs
git commit -m "feat: add AIME regrade artifacts"
```

---

### Task 3: Run Metrics Artifact

**Files:**
- Modify: `src/eval.rs`
- Modify: `tests/cli_tui.rs`

**Interfaces:**
- Consumes: `AimeSample`, `AimeFailureKind`, `AimeExecutionMode`, `failure_counts`, `failure_kind_name`, `write_pretty_json`.
- Produces:
  - `#[derive(Clone, Debug, Serialize)] pub struct AimeRunMetrics { ... }`
  - `fn build_run_metrics(samples: &[AimeSample], execution_mode: AimeExecutionMode, timeout_ms: u64) -> AimeRunMetrics`
  - `run_metrics.json` for fresh eval, replay, and regrade runs.

- [ ] **Step 1: Write failing unit tests**

Add these tests to `src/eval.rs`:

```rust
#[test]
fn run_metrics_preserve_order_and_compute_percentiles() {
    let mut first = sample("2025-I:I-1", "2025-I", true, None);
    first.elapsed_seconds = 1.0;
    let mut second = sample("2025-I:I-2", "2025-I", false, None);
    second.elapsed_seconds = 3.0;
    second.failure_kind = Some(AimeFailureKind::WrongFinal);
    let mut third = sample("2025-I:I-3", "2025-I", false, Some("ACP prompt timed out"));
    third.elapsed_seconds = 2.0;
    third.failure_kind = Some(AimeFailureKind::TimeoutNoResponse);

    let metrics = build_run_metrics(
        &[first, second, third],
        AimeExecutionMode::FreshProviderPerSample,
        5000,
    );

    assert_eq!(metrics.schema_version, 1);
    assert_eq!(metrics.sample_count, 3);
    assert_eq!(metrics.execution_mode, AimeExecutionMode::FreshProviderPerSample);
    assert_eq!(metrics.timeout_ms, 5000);
    assert_eq!(
        metrics.samples.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
        vec!["2025-I:I-1", "2025-I:I-2", "2025-I:I-3"]
    );
    assert_eq!(metrics.aggregate.min_elapsed_seconds, Some(1.0));
    assert_eq!(metrics.aggregate.p50_elapsed_seconds, Some(2.0));
    assert_eq!(metrics.aggregate.p95_elapsed_seconds, Some(3.0));
    assert_eq!(metrics.aggregate.max_elapsed_seconds, Some(3.0));
    assert_eq!(metrics.aggregate.timeout_count, 1);
    assert_eq!(metrics.aggregate.provider_error_count, 0);
    assert_eq!(metrics.aggregate.wrong_final_count, 1);
}

#[test]
fn run_metrics_handle_empty_samples() {
    let metrics = build_run_metrics(&[], AimeExecutionMode::SharedProvider, 1);

    assert_eq!(metrics.sample_count, 0);
    assert_eq!(metrics.aggregate.total_elapsed_seconds, 0.0);
    assert_eq!(metrics.aggregate.min_elapsed_seconds, None);
    assert_eq!(metrics.aggregate.p50_elapsed_seconds, None);
    assert_eq!(metrics.aggregate.p95_elapsed_seconds, None);
    assert_eq!(metrics.aggregate.max_elapsed_seconds, None);
}
```

- [ ] **Step 2: Add CLI assertion for `run_metrics.json`**

In `eval_aime_sanity_runs_tracked_dataset_and_writes_artifacts`, after manifest assertions, add:

```rust
let metrics: serde_json::Value =
    serde_json::from_str(&fs::read_to_string(run_dir.join("run_metrics.json")).unwrap()).unwrap();
assert_eq!(metrics["schema_version"], 1);
assert_eq!(metrics["sample_count"], 1);
assert_eq!(metrics["execution_mode"], "shared_provider");
assert_eq!(metrics["timeout_ms"], 5000);
assert_eq!(metrics["samples"][0]["id"], "2025-I:I-1");
assert_eq!(metrics["samples"][0]["passed"], true);
assert_eq!(metrics["aggregate"]["provider_error_count"], 0);
```

- [ ] **Step 3: Run tests to verify they fail**

Run:

```bash
cargo test run_metrics_preserve_order_and_compute_percentiles
cargo test run_metrics_handle_empty_samples
cargo test eval_aime_sanity_runs_tracked_dataset_and_writes_artifacts
```

Expected: FAIL because run metrics types and artifact do not exist.

- [ ] **Step 4: Add metrics data structures**

In `src/eval.rs`, add:

```rust
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AimeRunMetrics {
    pub schema_version: u8,
    pub sample_count: usize,
    pub execution_mode: AimeExecutionMode,
    pub timeout_ms: u64,
    pub samples: Vec<AimeRunMetricSample>,
    pub aggregate: AimeRunMetricAggregate,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AimeRunMetricSample {
    pub id: String,
    pub elapsed_seconds: f64,
    pub passed: bool,
    pub failure_kind: Option<AimeFailureKind>,
    pub return_status: Option<String>,
    pub had_error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageRecord>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AimeRunMetricAggregate {
    pub total_elapsed_seconds: f64,
    pub min_elapsed_seconds: Option<f64>,
    pub p50_elapsed_seconds: Option<f64>,
    pub p95_elapsed_seconds: Option<f64>,
    pub max_elapsed_seconds: Option<f64>,
    pub timeout_count: usize,
    pub provider_error_count: usize,
    pub no_final_count: usize,
    pub wrong_final_count: usize,
    pub prompt_contamination_count: usize,
}
```

- [ ] **Step 5: Implement metrics helpers**

Add near `summarize_samples`:

```rust
fn build_run_metrics(
    samples: &[AimeSample],
    execution_mode: AimeExecutionMode,
    timeout_ms: u64,
) -> AimeRunMetrics {
    let mut elapsed = samples
        .iter()
        .map(|sample| sample.elapsed_seconds)
        .collect::<Vec<_>>();
    elapsed.sort_by(|left, right| left.total_cmp(right));
    let count_kind = |kind: AimeFailureKind| {
        samples
            .iter()
            .filter(|sample| sample.failure_kind.as_ref() == Some(&kind))
            .count()
    };
    AimeRunMetrics {
        schema_version: 1,
        sample_count: samples.len(),
        execution_mode,
        timeout_ms,
        samples: samples
            .iter()
            .map(|sample| AimeRunMetricSample {
                id: sample.id.clone(),
                elapsed_seconds: sample.elapsed_seconds,
                passed: sample.passed,
                failure_kind: sample.failure_kind.clone(),
                return_status: sample.return_status.clone(),
                had_error: sample.error.is_some(),
                usage: sample.usage.clone(),
            })
            .collect(),
        aggregate: AimeRunMetricAggregate {
            total_elapsed_seconds: samples.iter().map(|sample| sample.elapsed_seconds).sum(),
            min_elapsed_seconds: elapsed.first().copied(),
            p50_elapsed_seconds: percentile_nearest_rank(&elapsed, 0.50),
            p95_elapsed_seconds: percentile_nearest_rank(&elapsed, 0.95),
            max_elapsed_seconds: elapsed.last().copied(),
            timeout_count: count_kind(AimeFailureKind::TimeoutNoResponse),
            provider_error_count: count_kind(AimeFailureKind::ProviderError),
            no_final_count: count_kind(AimeFailureKind::NoFinal),
            wrong_final_count: count_kind(AimeFailureKind::WrongFinal),
            prompt_contamination_count: count_kind(AimeFailureKind::PromptContamination),
        },
    }
}

fn percentile_nearest_rank(sorted_values: &[f64], percentile: f64) -> Option<f64> {
    if sorted_values.is_empty() {
        return None;
    }
    let rank = (percentile * sorted_values.len() as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted_values.len() - 1);
    Some(sorted_values[index])
}
```

Derive or implement `Clone` for `UsageRecord` if it is not already cloneable.

- [ ] **Step 6: Write metrics artifacts**

In `run_aime_cases`, after `summary.json` and before returning:

```rust
let metrics = build_run_metrics(&samples, options.execution_mode, options.timeout_ms);
write_pretty_json(&options.run_dir.join("run_metrics.json"), &metrics).await?;
```

In `run_aime_regrade`, after `summary.json`:

```rust
let metrics = build_run_metrics(
    &regraded_samples,
    eval_options.execution_mode,
    eval_options.timeout_ms,
);
write_pretty_json(&options.run_dir.join("run_metrics.json"), &metrics).await?;
```

- [ ] **Step 7: Run tests to verify they pass**

Run:

```bash
cargo test run_metrics_preserve_order_and_compute_percentiles
cargo test run_metrics_handle_empty_samples
cargo test eval_aime_sanity_runs_tracked_dataset_and_writes_artifacts
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add src/eval.rs tests/cli_tui.rs
git commit -m "feat: write AIME run metrics"
```

---

### Task 4: Fresh Provider Per Sample Flag And Execution Mode Wiring

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/eval.rs`
- Modify: `tests/cli_tui.rs`

**Interfaces:**
- Consumes: `AimeExecutionMode`, `run_aime_cases`, `build_manifest`, `build_run_metrics`.
- Produces:
  - `AimeSanityOptions::execution_mode`
  - `AimeEvalOptions::execution_mode`
  - CLI flag `--fresh-provider-per-sample` on `eval aime` and `eval aime-sanity`.
  - Manifest and metrics values of `"fresh_provider_per_sample"` when requested.

- [ ] **Step 1: Write failing CLI test for fresh mode on sanity**

Add this test to `tests/cli_tui.rs`:

```rust
#[test]
fn eval_aime_sanity_records_fresh_provider_per_sample_mode() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("fake-traecli-fresh");
    let run_dir = temp.path().join("run");
    fake_traecli(&server, "cli-fresh", "FINAL: 70\n");

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "eval",
            "aime-sanity",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--dataset",
            "evals/aime_sanity.jsonl",
            "--filter-id",
            "2025-I:I-1",
            "--run-dir",
            run_dir.to_str().unwrap(),
            "--timeout-ms",
            "5000",
            "--fresh-provider-per-sample",
        ])
        .assert()
        .success();

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["execution_mode"], "fresh_provider_per_sample");
    let metrics: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("run_metrics.json")).unwrap()).unwrap();
    assert_eq!(metrics["execution_mode"], "fresh_provider_per_sample");
}
```

- [ ] **Step 2: Write failing unit test for manifest mode**

In `builds_manifest_with_dataset_digest`, add this assertion:

```rust
assert_eq!(manifest.execution_mode, AimeExecutionMode::SharedProvider);
```

- [ ] **Step 3: Run tests to verify they fail**

Run:

```bash
cargo test eval_aime_sanity_records_fresh_provider_per_sample_mode
cargo test builds_manifest_with_dataset_digest
```

Expected: FAIL because the CLI flag is not wired.

- [ ] **Step 4: Add CLI flag and option wiring**

In `src/cli.rs`, add to `EvalCommand::Aime` and `EvalCommand::AimeSanity`:

```rust
#[arg(long)]
fresh_provider_per_sample: bool,
```

When constructing options, pass:

```rust
execution_mode: if fresh_provider_per_sample {
    AimeExecutionMode::FreshProviderPerSample
} else {
    AimeExecutionMode::SharedProvider
},
```

Import `AimeExecutionMode`.

In `src/eval.rs`, add `pub execution_mode: AimeExecutionMode` to `AimeSanityOptions`, and pass it through in `run_aime_sanity`.

- [ ] **Step 5: Make provider execution explicit**

Extract the prompt execution block in `run_aime_cases` into:

```rust
async fn run_aime_provider_prompt(
    options: &AimeEvalOptions,
    provider: &TraeCliAcpProvider,
    prompt: String,
) -> anyhow::Result<crate::provider::acp::AcpOutput> {
    let provider = match options.execution_mode {
        AimeExecutionMode::SharedProvider => provider.clone(),
        AimeExecutionMode::FreshProviderPerSample => {
            TraeCliAcpProvider::new(options.workspace.clone(), options.model.clone())
        }
    };
    let acp_options = AcpOptions {
        command: provider.command.clone(),
        args: provider.args(),
        cwd: options.workspace.clone(),
        model: provider.model.clone(),
        prompt,
        timeout_ms: options.timeout_ms,
    };
    if options.stream {
        let sink: ProviderEventSink = Arc::new(|event| {
            Box::pin(async move {
                if let ProviderEvent::AssistantDelta { delta } = event {
                    print!("delta: {delta}");
                }
                Ok(())
            }) as Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>>
        });
        run_acp_prompt_streaming(acp_options, Some(sink)).await
    } else {
        run_acp_prompt(acp_options).await
    }
}
```

Use this helper from `run_aime_cases`. The current TraeCLI ACP helper starts a process per prompt; this extraction still makes the requested mode explicit in evaluator state and leaves room for a persistent provider adapter later.

- [ ] **Step 6: Run tests to verify they pass**

Run:

```bash
cargo test eval_aime_sanity_records_fresh_provider_per_sample_mode
cargo test builds_manifest_with_dataset_digest
```

Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/eval.rs src/cli.rs tests/cli_tui.rs
git commit -m "feat: record AIME provider execution mode"
```

---

### Task 5: Scripted ACP Fault Harness For CLI Tests

**Files:**
- Create: `tests/support/mod.rs`
- Create: `tests/support/scripted_acp.rs`
- Modify: `tests/cli_tui.rs`

**Interfaces:**
- Consumes: existing CLI tests using `TRAECLI_PATH`.
- Produces:
  - `tests/support/mod.rs` with `pub mod scripted_acp;`
  - `pub enum ScriptedAcpStep`
  - `pub struct ScriptedAcp`
  - `pub fn scripted_acp(path: &Path, steps: &[ScriptedAcpStep])`
  - `pub fn assert_script_consumed(path: &Path)`

- [ ] **Step 1: Create test support module declarations**

At the top of `tests/cli_tui.rs`, add:

```rust
mod support;

use support::scripted_acp::{assert_script_consumed, scripted_acp, ScriptedAcpStep};
```

Create `tests/support/mod.rs`:

```rust
pub mod scripted_acp;
```

- [ ] **Step 2: Write failing scripted ACP tests**

Add these tests to `tests/cli_tui.rs`:

```rust
#[test]
fn eval_aime_scripted_acp_covers_success_wrong_no_final_and_provider_error() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("scripted-traecli");
    let run_dir = temp.path().join("run");
    scripted_acp(
        &server,
        &[
            ScriptedAcpStep::Success { text: "FINAL: 70\n" },
            ScriptedAcpStep::Success { text: "FINAL: 999\n" },
            ScriptedAcpStep::Success { text: "No final marker here." },
            ScriptedAcpStep::ProviderError { message: "scripted 503" },
        ],
    );

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &server)
        .args([
            "eval",
            "aime-sanity",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--dataset",
            "evals/aime_sanity.jsonl",
            "--filter-id",
            "2025-I:I-1",
            "--filter-id",
            "2025-I:I-2",
            "--filter-id",
            "2025-I:I-3",
            "--filter-id",
            "2025-I:I-4",
            "--run-dir",
            run_dir.to_str().unwrap(),
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .success()
        .stdout(contains("\"total\": 4"))
        .stdout(contains("\"correct\": 1"));

    let samples = fs::read_to_string(run_dir.join("samples.jsonl")).unwrap();
    assert!(samples.contains("\"failure_kind\":\"wrong_final\""));
    assert!(samples.contains("\"failure_kind\":\"no_final\""));
    assert!(samples.contains("\"failure_kind\":\"provider_error\""));
    assert_script_consumed(&server);
}

#[test]
fn eval_aime_scripted_acp_covers_contamination_and_timeout() {
    let temp = tempdir().unwrap();
    let contaminated_server = temp.path().join("scripted-contaminated");
    let contaminated_run = temp.path().join("contaminated-run");
    scripted_acp(
        &contaminated_server,
        &[ScriptedAcpStep::Success {
            text: "I will invoke skills and open files.\nFINAL: 999\n",
        }],
    );

    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &contaminated_server)
        .args([
            "eval",
            "aime-sanity",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--dataset",
            "evals/aime_sanity.jsonl",
            "--filter-id",
            "2025-I:I-1",
            "--run-dir",
            contaminated_run.to_str().unwrap(),
            "--timeout-ms",
            "5000",
        ])
        .assert()
        .success();
    let samples = fs::read_to_string(contaminated_run.join("samples.jsonl")).unwrap();
    assert!(samples.contains("\"failure_kind\":\"prompt_contamination\""));
    assert_script_consumed(&contaminated_server);

    let timeout_server = temp.path().join("scripted-timeout");
    let timeout_run = temp.path().join("timeout-run");
    scripted_acp(&timeout_server, &[ScriptedAcpStep::Stall]);
    Command::cargo_bin("corenius")
        .unwrap()
        .env("TRAECLI_PATH", &timeout_server)
        .args([
            "eval",
            "aime-sanity",
            "--workspace",
            temp.path().to_str().unwrap(),
            "--dataset",
            "evals/aime_sanity.jsonl",
            "--filter-id",
            "2025-I:I-1",
            "--run-dir",
            timeout_run.to_str().unwrap(),
            "--timeout-ms",
            "200",
        ])
        .assert()
        .success();
    let timeout_samples = fs::read_to_string(timeout_run.join("samples.jsonl")).unwrap();
    assert!(timeout_samples.contains("\"failure_kind\":\"timeout_no_response\""));
}

#[test]
#[should_panic(expected = "scripted ACP fixture was not fully consumed")]
fn scripted_acp_assert_consumed_fails_on_unconsumed_script() {
    let temp = tempdir().unwrap();
    let server = temp.path().join("scripted-unconsumed");
    scripted_acp(
        &server,
        &[ScriptedAcpStep::Success { text: "FINAL: 70\n" }],
    );

    assert_script_consumed(&server);
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run:

```bash
cargo test eval_aime_scripted_acp_covers_success_wrong_no_final_and_provider_error
cargo test eval_aime_scripted_acp_covers_contamination_and_timeout
cargo test scripted_acp_assert_consumed_fails_on_unconsumed_script
```

Expected: FAIL because `tests/support/scripted_acp.rs` does not exist.

- [ ] **Step 4: Implement scripted ACP helper**

Create `tests/support/scripted_acp.rs`:

```rust
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use serde_json::json;

pub enum ScriptedAcpStep {
    Success { text: &'static str },
    ProviderError { message: &'static str },
    Stall,
    MalformedLine,
}

pub fn scripted_acp(path: &Path, steps: &[ScriptedAcpStep]) {
    let script_path = path.with_extension("script.json");
    let cursor_path = path.with_extension("cursor");
    let steps_json = steps
        .iter()
        .map(|step| match step {
            ScriptedAcpStep::Success { text } => {
                json!({"kind": "success", "text": text})
            }
            ScriptedAcpStep::ProviderError { message } => {
                json!({"kind": "provider_error", "message": message})
            }
            ScriptedAcpStep::Stall => json!({"kind": "stall"}),
            ScriptedAcpStep::MalformedLine => json!({"kind": "malformed_line"}),
        })
        .collect::<Vec<_>>();
    fs::write(&script_path, serde_json::to_string_pretty(&steps_json).unwrap()).unwrap();
    fs::write(&cursor_path, "0").unwrap();
    fs::write(
        path,
        format!(
            r#"#!/usr/bin/env python3
import json, os, sys, time
script_path = {script_path:?}
cursor_path = {cursor_path:?}
if sys.argv[1:3] != ["acp", "serve"]:
    raise SystemExit("expected acp serve")
steps = json.load(open(script_path))
cursor = int(open(cursor_path).read().strip() or "0")
session_id = "scripted-" + str(cursor)
def advance():
    global cursor
    cursor += 1
    open(cursor_path, "w").write(str(cursor))
def respond(msg, result):
    print(json.dumps({{"jsonrpc":"2.0","id":msg["id"],"result":result}}), flush=True)
for line in sys.stdin:
    msg = json.loads(line)
    method = msg.get("method")
    if method == "initialize":
        respond(msg, {{"protocolVersion": 1}})
    elif method == "session/new":
        if "cwd" not in msg.get("params", {{}}):
            raise SystemExit("session/new missing cwd")
        respond(msg, {{"sessionId": session_id}})
    elif method == "session/prompt":
        params = msg.get("params", {{}})
        if params.get("sessionId") != session_id:
            raise SystemExit("session/prompt wrong sessionId")
        prompt = params.get("prompt")
        if not isinstance(prompt, list) or not prompt or prompt[0].get("type") != "text":
            raise SystemExit("session/prompt missing text prompt")
        if cursor >= len(steps):
            raise SystemExit("script exhausted")
        step = steps[cursor]
        kind = step["kind"]
        if kind == "success":
            text = step["text"]
            print(json.dumps({{"jsonrpc":"2.0","method":"session/update","params":{{"sessionId":session_id,"update":{{"sessionUpdate":"agent_message_chunk","content":{{"type":"text","text":text}}}}}}}}), flush=True)
            respond(msg, {{"stopReason":"end_turn","usage":{{"inputTokens":1,"outputTokens":1}}}})
            advance()
        elif kind == "provider_error":
            print(json.dumps({{"jsonrpc":"2.0","id":msg["id"],"error":{{"code":-32000,"message":step["message"]}}}}), flush=True)
            advance()
        elif kind == "stall":
            time.sleep(60)
        elif kind == "malformed_line":
            print("{{not-json", flush=True)
            advance()
        else:
            raise SystemExit("unknown script kind " + kind)
"#,
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

pub fn assert_script_consumed(path: &Path) {
    let script_path = path.with_extension("script.json");
    let cursor_path = path.with_extension("cursor");
    let steps: Vec<serde_json::Value> =
        serde_json::from_str(&fs::read_to_string(script_path).unwrap()).unwrap();
    let cursor: usize = fs::read_to_string(cursor_path)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(cursor, steps.len(), "scripted ACP fixture was not fully consumed");
}
```

If Rust raw string delimiters conflict with the embedded Python braces, use `r##"..."##` for the outer string.

- [ ] **Step 5: Migrate one old fake-provider test**

Replace `eval_aime_records_wrong_and_contaminated_failure_kinds` with a scripted ACP version or delete the duplicated assertions after the new scripted tests cover both cases. Keep `fake_traecli` for simple smoke tests that do not need FIFO behavior.

- [ ] **Step 6: Run tests to verify they pass**

Run:

```bash
cargo test eval_aime_scripted_acp_covers_success_wrong_no_final_and_provider_error
cargo test eval_aime_scripted_acp_covers_contamination_and_timeout
cargo test scripted_acp_assert_consumed_fails_on_unconsumed_script
cargo test eval_aime_records_wrong_and_contaminated_failure_kinds
```

Expected: PASS. If the old migrated test name no longer exists, run the two new scripted tests plus `cargo test eval_aime`.

- [ ] **Step 7: Commit**

```bash
git add tests/support/mod.rs tests/support/scripted_acp.rs tests/cli_tui.rs
git commit -m "test: add scripted ACP AIME fixtures"
```

---

### Task 6: Full Verification And Manual Smoke

**Files:**
- Modify only if verification exposes a defect in files already touched by Tasks 1-5.

**Interfaces:**
- Consumes: all previous tasks.
- Produces: verified AIME hardening branch with no known formatter, test, or clippy failures.

- [ ] **Step 1: Run formatter check**

Run: `cargo fmt --check`

Expected: PASS.

- [ ] **Step 2: Run full tests**

Run: `cargo test`

Expected: PASS.

- [ ] **Step 3: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`

Expected: PASS.

- [ ] **Step 4: Run fake-provider sanity smoke**

Run:

```bash
cargo run -- eval aime-sanity \
  --workspace . \
  --filter-id 2025-I:I-1 \
  --run-dir .scratch/aime-sanity/design-smoke \
  --timeout-ms 5000
```

Expected: PASS if `TRAECLI_PATH` is configured for a working provider; otherwise record that the provider smoke was skipped because no provider was configured. Do not claim this smoke passed unless it actually ran.

- [ ] **Step 5: Run regrade smoke on the sanity artifact**

Run:

```bash
cargo run -- eval aime-regrade \
  --from-run .scratch/aime-sanity/design-smoke \
  --run-dir .scratch/aime-benchmark/regrade-design-smoke
```

Expected: PASS if Step 4 produced `.scratch/aime-sanity/design-smoke/samples.jsonl`; otherwise skip with the same provider-availability note.

- [ ] **Step 6: Optional bounded real-provider sample**

Run only if a real provider is available and the operator wants a live provider check:

```bash
TRAECLI_PATH=/data02/home/philip.yang/.local/bin/traecli \
cargo run -- eval aime \
  --workspace /data02/home/philip.yang/workspace/corenius \
  --model gpt-5.2 \
  --years 2025-I \
  --limit 1 \
  --timeout-ms 120000 \
  --stream
```

Expected: A run directory containing `manifest.json`, `samples.jsonl`, `summary.json`, `triage.md`, and `run_metrics.json`. This is not required for CI completion.

- [ ] **Step 7: Inspect git status**

Run: `git status --short`

Expected: only intentional files from this plan plus pre-existing unrelated files. Do not stage unrelated files from outside this plan.

- [ ] **Step 8: Commit verification fixes if needed**

If verification required fixes, commit them:

```bash
git add src/eval.rs src/cli.rs tests/cli_tui.rs tests/support/mod.rs tests/support/scripted_acp.rs
git commit -m "fix: stabilize AIME harness artifacts"
```

If no fixes were needed, do not create an empty commit.

---

## Self-Review Notes

- Spec coverage: regrade command, immutable artifacts, scorer delta, scripted ACP fault behavior, run metrics, fresh-provider flag, sanity/full validation gates, and deferred Pi-specific work are all covered.
- Placeholder scan: no unresolved placeholders or open-ended implementation instructions remain.
- Type consistency: `AimeExecutionMode`, `AimeRegradeOptions`, `AimeScoredFields`, `AimeScorerDelta`, and `AimeDerivedRunContext` names are consistent across tasks.
- Review focus coverage: each listed failure mode has an owning test in Tasks 2-5.

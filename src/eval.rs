use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Command;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::core::{ProviderEvent, ProviderEventSink, UsageRecord};
use crate::provider::acp::{
    AcpOptions, TraeCliAcpProvider, run_acp_prompt, run_acp_prompt_streaming,
};

const PROMPT_TEMPLATE: &str = r#"You are running in an automated AIME benchmark. Solve only the math problem below.

Do not open tools, inspect files, invoke skills, describe workflow rules, or mention benchmark machinery.
Return concise mathematical reasoning followed by a final line exactly of the form FINAL: <integer>.
The final answer must be a single integer in [0, 999].

Problem:
{problem}
"#;

const DEFAULT_AIME_YEARS: &[&str] = &["2024", "2025-I", "2025-II"];
const GAIR_AIME_BASE_URL: &str =
    "https://raw.githubusercontent.com/GAIR-NLP/AIME-Preview/main/eval/data/aime";

#[derive(Clone, Debug)]
pub struct AimeSanityOptions {
    pub workspace: PathBuf,
    pub dataset: PathBuf,
    pub run_dir: PathBuf,
    pub model: Option<String>,
    pub timeout_ms: u64,
    pub stream: bool,
    pub include_timeout_probe: bool,
    pub filter_id: Vec<String>,
    pub execution_mode: AimeExecutionMode,
}

#[derive(Clone, Debug)]
pub struct AimeEvalOptions {
    pub workspace: PathBuf,
    pub source: AimeDatasetSource,
    pub run_dir: PathBuf,
    pub model: Option<String>,
    pub timeout_ms: u64,
    pub stream: bool,
    pub selection: AimeSelection,
    pub execution_mode: AimeExecutionMode,
}

#[derive(Clone, Debug)]
pub struct AimeReplayOptions {
    pub workspace: PathBuf,
    pub from_run: PathBuf,
    pub run_dir: PathBuf,
    pub model: Option<String>,
    pub timeout_ms: u64,
    pub stream: bool,
    pub failed_only: bool,
    pub filter_id: Vec<String>,
}

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
pub enum AimeDatasetSource {
    TrackedJsonl {
        path: PathBuf,
    },
    Gair {
        years: Vec<String>,
        data_dir: PathBuf,
        refresh_data: bool,
    },
    Replay {
        from_run: PathBuf,
    },
}

#[derive(Clone, Debug)]
pub struct AimeSelection {
    pub years: Vec<String>,
    pub offset: usize,
    pub limit: Option<usize>,
    pub filter_id: Vec<String>,
    pub include_timeout_probe: bool,
}

impl AimeSelection {
    pub fn default_for_full() -> Self {
        Self {
            years: DEFAULT_AIME_YEARS
                .iter()
                .map(|year| (*year).to_string())
                .collect(),
            offset: 0,
            limit: None,
            filter_id: Vec::new(),
            include_timeout_probe: true,
        }
    }

    fn default_for_sanity(filter_id: Vec<String>, include_timeout_probe: bool) -> Self {
        Self {
            years: Vec::new(),
            offset: 0,
            limit: None,
            filter_id,
            include_timeout_probe,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AimeCase {
    pub id: String,
    pub year: String,
    pub answer: String,
    pub problem: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AimeSample {
    pub id: String,
    pub year: String,
    pub tags: Vec<String>,
    pub source: String,
    pub expected_answer: String,
    pub extracted_answer: String,
    pub passed: bool,
    pub score: u8,
    pub elapsed_seconds: f64,
    pub return_status: Option<String>,
    pub error: Option<String>,
    pub prompt: String,
    pub response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_kind: Option<AimeFailureKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AimeFailureKind {
    TimeoutNoResponse,
    ProviderError,
    NoFinal,
    WrongFinal,
    PromptContamination,
}

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

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct AimeScorerDeltaReport {
    schema_version: u8,
    source_run_dir: String,
    changed_sample_count: usize,
    changes: Vec<AimeScorerDelta>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AimeSummary {
    pub total: usize,
    pub correct: usize,
    pub accuracy: f64,
    pub model: Option<String>,
    pub years: Vec<String>,
    pub dataset: String,
    pub run_dir: String,
    pub timeout_ms: u64,
    pub by_year: BTreeMap<String, AimeYearSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_timeout_probe: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AimeYearSummary {
    pub total: usize,
    pub correct: usize,
    pub accuracy: f64,
}

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

#[derive(Clone, Debug, Serialize)]
pub struct AimeRunManifest {
    pub schema_version: u8,
    pub command_argv: Vec<String>,
    pub workspace: String,
    pub run_dir: String,
    pub dataset: String,
    pub years: Vec<String>,
    pub model: Option<String>,
    pub timeout_ms: u64,
    pub stream: bool,
    pub started_at_utc: String,
    pub corenius_version: String,
    pub git_commit: Option<String>,
    pub git_status_short: Vec<String>,
    pub dataset_cache_files: Vec<AimeDatasetCacheFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_run_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replayed_from_sample_count: Option<usize>,
    pub execution_mode: AimeExecutionMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regraded_from_sample_count: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AimeDatasetCacheFile {
    pub path: String,
    pub byte_length: u64,
    pub sha256: String,
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

pub async fn run_aime_sanity(options: AimeSanityOptions) -> anyhow::Result<AimeSummary> {
    let eval_options = AimeEvalOptions {
        workspace: options.workspace,
        source: AimeDatasetSource::TrackedJsonl {
            path: options.dataset,
        },
        run_dir: options.run_dir,
        model: options.model,
        timeout_ms: options.timeout_ms,
        stream: options.stream,
        selection: AimeSelection::default_for_sanity(
            options.filter_id,
            options.include_timeout_probe,
        ),
        execution_mode: options.execution_mode,
    };
    run_aime_eval(eval_options).await
}

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

pub async fn run_aime_eval(options: AimeEvalOptions) -> anyhow::Result<AimeSummary> {
    let loaded = load_dataset(&options).await?;
    let selected = select_cases(loaded.cases.clone(), &options.selection)?;
    if selected.is_empty() {
        bail!("AIME selection is empty");
    }
    let eval_cases = selected
        .into_iter()
        .map(AimeRunCase::from_case)
        .collect::<Vec<_>>();
    run_aime_cases(options, loaded, eval_cases, None).await
}

pub async fn run_aime_replay(options: AimeReplayOptions) -> anyhow::Result<AimeSummary> {
    let replay_samples = load_replay_samples(&options.from_run)?;
    let selected = select_replay_samples(replay_samples, options.failed_only, &options.filter_id)?;
    if selected.is_empty() {
        bail!("AIME replay selection is empty");
    }
    let cases = selected
        .iter()
        .map(AimeRunCase::from_sample)
        .collect::<Vec<_>>();
    let years = years_from_cases(&cases);
    let loaded = LoadedDataset {
        source: format!("replay:{}", options.from_run.display()),
        cases: Vec::new(),
        cache_files: vec![dataset_cache_file(&options.from_run.join("samples.jsonl"))?],
    };
    let derived_context = AimeDerivedRunContext {
        kind: AimeDerivedRunKind::Replay,
        source_run_dir: options.from_run.clone(),
        source_sample_count: cases.len(),
    };
    let eval_options = AimeEvalOptions {
        workspace: options.workspace,
        source: AimeDatasetSource::Replay {
            from_run: options.from_run,
        },
        run_dir: options.run_dir,
        model: options.model,
        timeout_ms: options.timeout_ms,
        stream: options.stream,
        selection: AimeSelection {
            years,
            offset: 0,
            limit: None,
            filter_id: options.filter_id,
            include_timeout_probe: true,
        },
        execution_mode: AimeExecutionMode::SharedProvider,
    };
    run_aime_cases(eval_options, loaded, cases, Some(derived_context)).await
}

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

    let mut summary = summarize_samples(&regraded_samples, &eval_options);
    summary.dataset = loaded.source.clone();
    write_pretty_json(&options.run_dir.join("summary.json"), &summary).await?;
    let metrics = build_run_metrics(
        &regraded_samples,
        eval_options.execution_mode,
        eval_options.timeout_ms,
    );
    write_pretty_json(&options.run_dir.join("run_metrics.json"), &metrics).await?;
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

async fn run_aime_cases(
    options: AimeEvalOptions,
    loaded: LoadedDataset,
    cases: Vec<AimeRunCase>,
    derived_context: Option<AimeDerivedRunContext>,
) -> anyhow::Result<AimeSummary> {
    tokio::fs::create_dir_all(&options.run_dir)
        .await
        .with_context(|| format!("create run dir {}", options.run_dir.display()))?;
    let samples_path = options.run_dir.join("samples.jsonl");
    let _ = tokio::fs::remove_file(&samples_path).await;
    let manifest = build_manifest(
        &options,
        &loaded,
        derived_context.clone(),
        started_at_utc(),
        std::env::args().collect(),
    );
    write_pretty_json(&options.run_dir.join("manifest.json"), &manifest).await?;

    let provider = TraeCliAcpProvider::new(options.workspace.clone(), options.model.clone());
    let mut samples = Vec::new();
    for case in cases {
        let prompt = case.prompt.clone();
        eprintln!("RUN id={} expected={}", case.id, case.answer);
        let started = Instant::now();
        let result = run_aime_provider_prompt(&options, &provider, prompt.clone()).await;
        let elapsed_seconds = started.elapsed().as_secs_f64();
        let (response, return_status, usage, error) = match result {
            Ok(output) => (output.text, output.stop_reason, output.usage, None),
            Err(error) => (String::new(), None, None, Some(error.to_string())),
        };
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
        append_jsonl(&samples_path, &sample).await?;
        eprintln!(
            "RESULT id={} extracted={} expected={} passed={} elapsed={:.1}s",
            sample.id,
            sample.extracted_answer,
            sample.expected_answer,
            sample.passed,
            sample.elapsed_seconds
        );
        samples.push(sample);
    }

    let summary = summarize_samples(&samples, &options);
    write_pretty_json(&options.run_dir.join("summary.json"), &summary).await?;
    let metrics = build_run_metrics(&samples, options.execution_mode, options.timeout_ms);
    write_pretty_json(&options.run_dir.join("run_metrics.json"), &metrics).await?;
    tokio::fs::write(
        options.run_dir.join("triage.md"),
        render_triage_markdown(&summary, &samples, derived_context.as_ref()),
    )
    .await?;
    Ok(summary)
}

struct LoadedDataset {
    source: String,
    cases: Vec<AimeCase>,
    cache_files: Vec<AimeDatasetCacheFile>,
}

#[derive(Clone, Debug)]
struct AimeRunCase {
    id: String,
    year: String,
    answer: String,
    prompt: String,
    tags: Vec<String>,
}

impl AimeRunCase {
    fn from_case(case: AimeCase) -> Self {
        let id = case_key(&case);
        let prompt = prompt_for(&case);
        Self {
            id,
            year: case.year,
            answer: case.answer,
            prompt,
            tags: case.tags,
        }
    }

    fn from_sample(sample: &AimeSample) -> Self {
        Self {
            id: sample.id.clone(),
            year: sample.year.clone(),
            answer: sample.expected_answer.clone(),
            prompt: sample.prompt.clone(),
            tags: sample.tags.clone(),
        }
    }
}

async fn load_dataset(options: &AimeEvalOptions) -> anyhow::Result<LoadedDataset> {
    match &options.source {
        AimeDatasetSource::TrackedJsonl { path } => {
            let dataset_path = resolve_dataset(path)?;
            Ok(LoadedDataset {
                source: display_dataset(path),
                cases: load_cases(&dataset_path)?,
                cache_files: vec![dataset_cache_file_with_display(
                    &dataset_path,
                    &display_dataset(path),
                )?],
            })
        }
        AimeDatasetSource::Gair {
            years,
            data_dir,
            refresh_data,
        } => {
            let mut cases = Vec::new();
            let mut cache_files = Vec::new();
            for year in years {
                let path = ensure_gair_year(year, data_dir, *refresh_data).await?;
                let year_cases = load_gair_cases(year, &path)?;
                validate_gair_year_count(year, &path, year_cases.len())?;
                cases.extend(year_cases);
                cache_files.push(dataset_cache_file(&path)?);
            }
            Ok(LoadedDataset {
                source: "gair".to_string(),
                cases,
                cache_files,
            })
        }
        AimeDatasetSource::Replay { from_run } => Ok(LoadedDataset {
            source: format!("replay:{}", from_run.display()),
            cases: Vec::new(),
            cache_files: vec![dataset_cache_file(&from_run.join("samples.jsonl"))?],
        }),
    }
}

pub fn load_cases(path: &Path) -> anyhow::Result<Vec<AimeCase>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read dataset {}", path.display()))?;
    let mut cases = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let case: AimeCase = serde_json::from_str(line)
            .with_context(|| format!("parse {} line {}", path.display(), index + 1))?;
        cases.push(case);
    }
    Ok(cases)
}

fn load_gair_cases(year: &str, path: &Path) -> anyhow::Result<Vec<AimeCase>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("read GAIR dataset {}", path.display()))?;
    let mut cases = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        cases.push(parse_gair_case(year, line).with_context(|| {
            format!("parse GAIR dataset {} line {}", path.display(), index + 1)
        })?);
    }
    Ok(cases)
}

async fn ensure_gair_year(
    year: &str,
    data_dir: &Path,
    refresh_data: bool,
) -> anyhow::Result<PathBuf> {
    ensure!(
        DEFAULT_AIME_YEARS.contains(&year),
        "unsupported AIME year {year}"
    );
    let path = data_dir.join(format!("{year}.jsonl"));
    if path.exists() && !refresh_data {
        return Ok(path);
    }
    tokio::fs::create_dir_all(data_dir)
        .await
        .with_context(|| format!("create AIME data dir {}", data_dir.display()))?;
    let base_url =
        std::env::var("CORENIUS_AIME_BASE_URL").unwrap_or_else(|_| GAIR_AIME_BASE_URL.to_string());
    let url = format!("{}/{}", base_url.trim_end_matches('/'), gair_filename(year));
    let body = reqwest::get(&url)
        .await
        .with_context(|| format!("fetch {url}"))?
        .error_for_status()
        .with_context(|| format!("fetch {url}"))?
        .bytes()
        .await
        .with_context(|| format!("read response body from {url}"))?;
    tokio::fs::write(&path, body)
        .await
        .with_context(|| format!("write AIME cache {}", path.display()))?;
    Ok(path)
}

fn gair_filename(year: &str) -> String {
    format!("test{year}.jsonl")
}

fn expected_gair_year_count(year: &str) -> anyhow::Result<usize> {
    match year {
        "2024" => Ok(30),
        "2025-I" | "2025-II" => Ok(15),
        _ => bail!("unsupported AIME year {year}"),
    }
}

fn validate_gair_year_count(year: &str, path: &Path, actual: usize) -> anyhow::Result<()> {
    let expected = expected_gair_year_count(year)?;
    ensure!(
        actual == expected,
        "expected {expected} AIME cases for {year}, found {actual} in {}",
        path.display()
    );
    Ok(())
}

fn parse_gair_case(year: &str, line: &str) -> anyhow::Result<AimeCase> {
    let value: Value = serde_json::from_str(line)?;
    let id = match value.get("id").context("GAIR row missing id")? {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        _ => bail!("GAIR row id must be a string or number"),
    };
    let answer = match value.get("answer").context("GAIR row missing answer")? {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        _ => bail!("GAIR row answer must be a string or number"),
    };
    let problem = value
        .get("problem")
        .and_then(Value::as_str)
        .context("GAIR row missing problem")?
        .to_string();
    Ok(AimeCase {
        id,
        year: year.to_string(),
        answer,
        problem,
        tags: vec![year.to_string()],
    })
}

fn select_cases(cases: Vec<AimeCase>, options: &AimeSelection) -> anyhow::Result<Vec<AimeCase>> {
    let year_set = options.years.iter().cloned().collect::<HashSet<_>>();
    let mut selected = cases
        .into_iter()
        .filter(|case| {
            (year_set.is_empty() || year_set.contains(&case.year))
                && (options.include_timeout_probe
                    || !case.tags.iter().any(|tag| tag == "timeout-probe"))
        })
        .collect::<Vec<_>>();
    if options.offset > selected.len() {
        selected.clear();
    } else {
        selected = selected.into_iter().skip(options.offset).collect();
    }
    if let Some(limit) = options.limit {
        selected.truncate(limit);
    }
    if !options.filter_id.is_empty() {
        let filters = options.filter_id.iter().cloned().collect::<HashSet<_>>();
        selected.retain(|case| filters.contains(&case_key(case)));
    }
    if !options.filter_id.is_empty() && selected.len() != options.filter_id.len() {
        bail!(
            "selected {} cases for {} requested filters",
            selected.len(),
            options.filter_id.len()
        );
    }
    Ok(selected)
}

fn prompt_for(case: &AimeCase) -> String {
    PROMPT_TEMPLATE.replace("{problem}", &case.problem)
}

fn case_key(case: &AimeCase) -> String {
    format!("{}:{}", case.year, case.id)
}

fn normalize_answer(answer: &str) -> String {
    answer
        .trim()
        .parse::<u16>()
        .map(|value| value.to_string())
        .unwrap_or_else(|_| answer.trim().to_string())
}

#[cfg(test)]
fn extract_answer(text: &str) -> String {
    extract_final_answer(text)
        .or_else(|| {
            let mut last = None;
            for token in numeric_tokens(text) {
                last = Some(token);
            }
            last
        })
        .unwrap_or_default()
}

fn extract_final_answer(text: &str) -> Option<String> {
    let mut final_answer = None;
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if let Some(offset) = lower.find("final:") {
            final_answer = first_int(&line[offset + "final:".len()..]);
        }
    }
    final_answer
}

fn first_int(text: &str) -> Option<String> {
    numeric_tokens(text).next()
}

fn numeric_tokens(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|character: char| !character.is_ascii_digit())
        .filter(|token| !token.is_empty() && token.len() <= 3)
        .filter_map(|token| token.parse::<u16>().ok())
        .filter(|value| *value <= 999)
        .map(|value| value.to_string())
}

async fn append_jsonl<T: Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await?;
    file.write_all(serde_json::to_string(value)?.as_bytes())
        .await?;
    file.write_all(b"\n").await?;
    Ok(())
}

fn resolve_dataset(path: &Path) -> anyhow::Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest_dir.join(path))
}

fn display_dataset(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn load_replay_samples(from_run: &Path) -> anyhow::Result<Vec<AimeSample>> {
    let path = from_run.join("samples.jsonl");
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("read replay samples {}", path.display()))?;
    let mut samples = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let sample: AimeSample = serde_json::from_str(line)
            .with_context(|| format!("parse {} line {}", path.display(), index + 1))?;
        samples.push(sample);
    }
    Ok(samples)
}

fn select_replay_samples(
    samples: Vec<AimeSample>,
    failed_only: bool,
    filter_id: &[String],
) -> anyhow::Result<Vec<AimeSample>> {
    let filters = filter_id.iter().cloned().collect::<HashSet<_>>();
    let selected = samples
        .into_iter()
        .filter(|sample| !failed_only || !sample.passed)
        .filter(|sample| filters.is_empty() || filters.contains(&sample.id))
        .collect::<Vec<_>>();
    if !filters.is_empty() && selected.len() != filters.len() {
        bail!(
            "selected {} replay samples for {} requested filters",
            selected.len(),
            filters.len()
        );
    }
    Ok(selected)
}

fn years_from_cases(cases: &[AimeRunCase]) -> Vec<String> {
    let mut years = Vec::new();
    for case in cases {
        if !years.contains(&case.year) {
            years.push(case.year.clone());
        }
    }
    years
}

fn years_from_samples(samples: &[AimeSample]) -> Vec<String> {
    let mut years = Vec::new();
    for sample in samples {
        if !years.contains(&sample.year) {
            years.push(sample.year.clone());
        }
    }
    years
}

fn classify_failure(passed: bool, response: &str, error: Option<&str>) -> Option<AimeFailureKind> {
    if passed {
        return None;
    }
    if let Some(error) = error {
        let lower = error.to_ascii_lowercase();
        if lower.contains("timed out") || lower.contains("timeout") {
            return Some(AimeFailureKind::TimeoutNoResponse);
        }
        return Some(AimeFailureKind::ProviderError);
    }
    if extract_final_answer(response).is_none() {
        return Some(AimeFailureKind::NoFinal);
    }
    if response_has_prompt_contamination(response) {
        return Some(AimeFailureKind::PromptContamination);
    }
    Some(AimeFailureKind::WrongFinal)
}

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

fn response_has_prompt_contamination(response: &str) -> bool {
    let lower = response.to_ascii_lowercase();
    [
        "using-superpowers",
        "invoke skills",
        "open files",
        "inspect files",
        "workflow rules",
        "benchmark machinery",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn dataset_cache_file(path: &Path) -> anyhow::Result<AimeDatasetCacheFile> {
    dataset_cache_file_with_display(path, &path.display().to_string())
}

fn dataset_cache_file_with_display(
    path: &Path,
    display_path: &str,
) -> anyhow::Result<AimeDatasetCacheFile> {
    let bytes = std::fs::read(path).with_context(|| format!("read dataset {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(AimeDatasetCacheFile {
        path: display_path.to_string(),
        byte_length: bytes.len() as u64,
        sha256: format!("{:x}", hasher.finalize()),
    })
}

fn started_at_utc() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

fn build_manifest(
    options: &AimeEvalOptions,
    loaded: &LoadedDataset,
    derived_context: Option<AimeDerivedRunContext>,
    started_at_utc: String,
    command_argv: Vec<String>,
) -> AimeRunManifest {
    AimeRunManifest {
        schema_version: 1,
        command_argv,
        workspace: options.workspace.display().to_string(),
        run_dir: options.run_dir.display().to_string(),
        dataset: loaded.source.clone(),
        years: if options.selection.years.is_empty() {
            years_from_loaded(loaded)
        } else {
            options.selection.years.clone()
        },
        model: options.model.clone(),
        timeout_ms: options.timeout_ms,
        stream: options.stream,
        started_at_utc,
        corenius_version: crate::VERSION.to_string(),
        git_commit: git_output(&options.workspace, &["rev-parse", "HEAD"]).ok(),
        git_status_short: git_output(&options.workspace, &["status", "--short"])
            .map(|output| output.lines().map(str::to_string).collect())
            .unwrap_or_default(),
        dataset_cache_files: loaded.cache_files.clone(),
        source_run_dir: derived_context
            .as_ref()
            .map(|context| context.source_run_dir.display().to_string()),
        replayed_from_sample_count: derived_context.as_ref().and_then(|context| match context.kind {
            AimeDerivedRunKind::Replay => Some(context.source_sample_count),
            AimeDerivedRunKind::Regrade => None,
        }),
        execution_mode: options.execution_mode,
        regraded_from_sample_count: derived_context.as_ref().and_then(|context| match context.kind {
            AimeDerivedRunKind::Replay => None,
            AimeDerivedRunKind::Regrade => Some(context.source_sample_count),
        }),
    }
}

fn git_output(cwd: &Path, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("run git {}", args.join(" ")))?;
    ensure!(
        output.status.success(),
        "git {} failed with status {}",
        args.join(" "),
        output.status
    );
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn write_pretty_json<T: Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    tokio::fs::write(path, format!("{json}\n"))
        .await
        .with_context(|| format!("write {}", path.display()))
}

fn summarize_samples(samples: &[AimeSample], options: &AimeEvalOptions) -> AimeSummary {
    let correct = samples.iter().filter(|sample| sample.passed).count();
    let mut by_year = BTreeMap::<String, AimeYearSummary>::new();
    for sample in samples {
        let year = by_year
            .entry(sample.year.clone())
            .or_insert(AimeYearSummary {
                total: 0,
                correct: 0,
                accuracy: 0.0,
            });
        year.total += 1;
        year.correct += usize::from(sample.passed);
    }
    for year in by_year.values_mut() {
        year.accuracy = accuracy(year.correct, year.total);
    }
    AimeSummary {
        total: samples.len(),
        correct,
        accuracy: accuracy(correct, samples.len()),
        model: options.model.clone(),
        years: if options.selection.years.is_empty() {
            by_year.keys().cloned().collect()
        } else {
            options.selection.years.clone()
        },
        dataset: match &options.source {
            AimeDatasetSource::TrackedJsonl { path } => display_dataset(path),
            AimeDatasetSource::Gair { .. } => "gair".to_string(),
            AimeDatasetSource::Replay { from_run } => format!("replay:{}", from_run.display()),
        },
        run_dir: options.run_dir.display().to_string(),
        timeout_ms: options.timeout_ms,
        by_year,
        included_timeout_probe: match &options.source {
            AimeDatasetSource::TrackedJsonl { .. } => Some(options.selection.include_timeout_probe),
            AimeDatasetSource::Gair { .. } | AimeDatasetSource::Replay { .. } => None,
        },
    }
}

fn accuracy(correct: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        correct as f64 / total as f64
    }
}

fn years_from_loaded(loaded: &LoadedDataset) -> Vec<String> {
    let mut years = Vec::new();
    for case in &loaded.cases {
        if !years.contains(&case.year) {
            years.push(case.year.clone());
        }
    }
    years
}

fn render_triage_markdown(
    summary: &AimeSummary,
    samples: &[AimeSample],
    derived_context: Option<&AimeDerivedRunContext>,
) -> String {
    let mut output = String::new();
    output.push_str("# AIME benchmark triage\n\n");
    output.push_str(&format!(
        "- total: {}\n- correct: {}\n- accuracy: {:.3}\n- dataset: {}\n- timeout_ms: {}\n\n",
        summary.total, summary.correct, summary.accuracy, summary.dataset, summary.timeout_ms
    ));
    if let Some(context) = derived_context {
        output.push_str(&format!("- source_run_dir: {}\n", context.source_run_dir.display()));
        match context.kind {
            AimeDerivedRunKind::Replay => {
                output.push_str(&format!(
                    "- replayed_from_sample_count: {}\n\n",
                    context.source_sample_count
                ));
            }
            AimeDerivedRunKind::Regrade => {
                output.push_str(&format!(
                    "- regraded_from_sample_count: {}\n\n",
                    context.source_sample_count
                ));
            }
        }
    }
    output.push_str("## By year\n\n");
    output.push_str("| year | total | correct | accuracy |\n");
    output.push_str("| --- | ---: | ---: | ---: |\n");
    for (year, row) in &summary.by_year {
        output.push_str(&format!(
            "| {} | {} | {} | {:.3} |\n",
            year, row.total, row.correct, row.accuracy
        ));
    }
    output.push_str("\n## Failure counts\n\n");
    output.push_str("| failure_kind | count |\n");
    output.push_str("| --- | ---: |\n");
    for (kind, count) in failure_counts(samples) {
        output.push_str(&format!("| {} | {} |\n", failure_kind_name(kind), count));
    }
    output.push_str("\n## Problems\n\n");
    output.push_str(
        "| id | year | expected | extracted | status | failure_kind | elapsed_seconds | note |\n",
    );
    output.push_str("| --- | --- | ---: | ---: | --- | --- | ---: | --- |\n");
    for sample in samples {
        let status = if sample.passed { "pass" } else { "fail" };
        let note = sample.error.as_deref().unwrap_or_default();
        let failure_kind = sample
            .failure_kind
            .as_ref()
            .map(failure_kind_name)
            .unwrap_or_default();
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {:.3} | {} |\n",
            escape_md_cell(&sample.id),
            escape_md_cell(&sample.year),
            escape_md_cell(&sample.expected_answer),
            escape_md_cell(&sample.extracted_answer),
            status,
            failure_kind,
            sample.elapsed_seconds,
            escape_md_cell(note)
        ));
    }
    output
}

fn escape_md_cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

fn failure_counts(samples: &[AimeSample]) -> BTreeMap<&AimeFailureKind, usize> {
    let mut counts = BTreeMap::new();
    for sample in samples {
        if let Some(kind) = &sample.failure_kind {
            *counts.entry(kind).or_insert(0) += 1;
        }
    }
    counts
}

fn failure_kind_name(kind: &AimeFailureKind) -> &'static str {
    match kind {
        AimeFailureKind::TimeoutNoResponse => "timeout_no_response",
        AimeFailureKind::ProviderError => "provider_error",
        AimeFailureKind::NoFinal => "no_final",
        AimeFailureKind::WrongFinal => "wrong_final",
        AimeFailureKind::PromptContamination => "prompt_contamination",
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_gair_rows_into_aime_cases() {
        let case = parse_gair_case("2024", r#"{"id":60,"problem":"P","answer":"007"}"#).unwrap();

        assert_eq!(case.id, "60");
        assert_eq!(case_key(&case), "2024:60");
        assert_eq!(case.year, "2024");
        assert_eq!(case.answer, "007");
        assert_eq!(case.problem, "P");
        assert_eq!(case.tags, vec!["2024"]);

        let case =
            parse_gair_case("2025-I", r#"{"id":"I-1","problem":"P","answer":"70"}"#).unwrap();
        assert_eq!(case_key(&case), "2025-I:I-1");
        assert_eq!(case.tags, vec!["2025-I"]);
    }

    #[test]
    fn selects_cases_by_year_offset_limit_and_filter_id() {
        let cases = vec![
            aime_case("2024", "60", "1"),
            aime_case("2024", "61", "2"),
            aime_case("2025-I", "I-1", "3"),
            aime_case("2025-II", "II-1", "4"),
            aime_case("2025-II", "II-2", "5"),
        ];
        let selected = select_cases(
            cases,
            &AimeSelection {
                years: vec!["2025-II".to_string(), "2024".to_string()],
                offset: 1,
                limit: Some(2),
                filter_id: vec!["2024:61".to_string()],
                include_timeout_probe: true,
            },
        )
        .unwrap();

        assert_eq!(
            selected.iter().map(case_key).collect::<Vec<_>>(),
            vec!["2024:61"]
        );
    }

    #[test]
    fn extracts_final_answer_and_fallback_number() {
        assert_eq!(extract_answer("working\nFINAL: 007\n"), "7");
        assert_eq!(extract_answer("first 12 then final: 345 and 678"), "345");
        assert_eq!(
            extract_answer("no final marker; candidates 42 and 999"),
            "999"
        );
        assert_eq!(extract_answer("out of range 1000"), "");
    }

    #[test]
    fn prompt_keeps_aime_runs_math_only() {
        let prompt = prompt_for(&aime_case("2025-I", "I-12", "510"));

        assert!(prompt.contains("Solve only the math problem below."));
        assert!(prompt.contains("Do not open tools, inspect files, invoke skills"));
        assert!(prompt.contains("FINAL: <integer>"));
    }

    #[test]
    fn summarizes_samples_by_year() {
        let samples = vec![
            sample("2024:60", "2024", true, None),
            sample("2024:61", "2024", false, None),
            sample("2025-I:I-1", "2025-I", true, None),
        ];

        let summary = summarize_samples(
            &samples,
            &AimeEvalOptions {
                workspace: PathBuf::from("."),
                source: AimeDatasetSource::Gair {
                    years: vec!["2024".to_string(), "2025-I".to_string()],
                    data_dir: PathBuf::from(".scratch/aime-benchmark/data"),
                    refresh_data: false,
                },
                run_dir: PathBuf::from(".scratch/aime-benchmark/run-test"),
                model: Some("gpt-5.2".to_string()),
                timeout_ms: 300_000,
                stream: false,
                selection: AimeSelection {
                    years: vec!["2024".to_string(), "2025-I".to_string()],
                    offset: 0,
                    limit: None,
                    filter_id: Vec::new(),
                    include_timeout_probe: true,
                },
                execution_mode: AimeExecutionMode::SharedProvider,
            },
        );

        assert_eq!(summary.total, 3);
        assert_eq!(summary.correct, 2);
        assert_eq!(summary.years, vec!["2024", "2025-I"]);
        assert_eq!(summary.by_year["2024"].total, 2);
        assert_eq!(summary.by_year["2024"].correct, 1);
        assert_eq!(summary.by_year["2025-I"].accuracy, 1.0);
    }

    #[test]
    fn triage_markdown_covers_pass_wrong_and_error() {
        let samples = vec![
            sample("2024:60", "2024", true, None),
            sample("2024:61", "2024", false, None),
            sample("2025-I:I-1", "2025-I", false, Some("ACP prompt timed out")),
        ];
        let summary = summarize_samples(
            &samples,
            &AimeEvalOptions {
                workspace: PathBuf::from("."),
                source: AimeDatasetSource::Gair {
                    years: vec!["2024".to_string(), "2025-I".to_string()],
                    data_dir: PathBuf::from(".scratch/aime-benchmark/data"),
                    refresh_data: false,
                },
                run_dir: PathBuf::from(".scratch/aime-benchmark/run-test"),
                model: None,
                timeout_ms: 1,
                stream: false,
                selection: AimeSelection::default_for_full(),
                execution_mode: AimeExecutionMode::SharedProvider,
            },
        );
        let triage = render_triage_markdown(&summary, &samples, None);

        assert!(triage.contains("| year | total | correct | accuracy |"));
        assert!(triage.contains("| 2024 | 2 | 1 | 0.500 |"));
        assert!(triage.contains("## Failure counts"));
        assert!(triage.contains("| failure_kind | count |"));
        assert!(triage.contains("| timeout_no_response | 1 |"));
        assert!(triage.contains("| wrong_final | 1 |"));
        assert!(triage.contains(
            "| id | year | expected | extracted | status | failure_kind | elapsed_seconds | note |"
        ));
        assert!(triage.contains("| 2024:60 | 2024 | 7 | 7 | pass |  |"));
        assert!(triage.contains("| 2024:61 | 2024 | 7 | 8 | fail | wrong_final |"));
        assert!(triage.contains("ACP prompt timed out"));
    }

    #[test]
    fn classifies_failure_kinds_in_priority_order() {
        assert_eq!(
            classify_failure(false, "", Some("ACP prompt timed out after 10ms")),
            Some(AimeFailureKind::TimeoutNoResponse)
        );
        assert_eq!(
            classify_failure(false, "", Some("json-rpc error from provider")),
            Some(AimeFailureKind::ProviderError)
        );
        assert_eq!(
            classify_failure(false, "", None),
            Some(AimeFailureKind::NoFinal)
        );
        assert_eq!(
            classify_failure(
                false,
                "I need to invoke skills and open files.\nFINAL: 8",
                None
            ),
            Some(AimeFailureKind::PromptContamination)
        );
        assert_eq!(
            classify_failure(false, "ordinary reasoning\nFINAL: 8", None),
            Some(AimeFailureKind::WrongFinal)
        );
        assert_eq!(
            classify_failure(true, "using-superpowers\nFINAL: 7", None),
            None
        );
    }

    #[test]
    fn no_final_with_matching_number_is_not_a_pass() {
        assert_eq!(extract_answer("The answer is 7."), "7");
        assert_eq!(extract_final_answer("The answer is 7."), None);
        assert_eq!(
            classify_failure(false, "The answer is 7.", None),
            Some(AimeFailureKind::NoFinal)
        );
    }

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

    #[test]
    fn builds_manifest_with_dataset_digest() {
        let temp = tempfile::tempdir().unwrap();
        let dataset = temp.path().join("tracked.jsonl");
        std::fs::write(&dataset, "{\"id\":\"1\"}\n").unwrap();
        let loaded = LoadedDataset {
            source: "tracked".to_string(),
            cases: vec![],
            cache_files: vec![dataset_cache_file(&dataset).unwrap()],
        };
        let options = AimeEvalOptions {
            workspace: temp.path().join("workspace"),
            source: AimeDatasetSource::TrackedJsonl {
                path: dataset.clone(),
            },
            run_dir: temp.path().join("run"),
            model: Some("model-a".to_string()),
            timeout_ms: 123,
            stream: true,
            selection: AimeSelection {
                years: vec!["2025-I".to_string()],
                offset: 0,
                limit: Some(1),
                filter_id: vec!["2025-I:I-1".to_string()],
                include_timeout_probe: true,
            },
            execution_mode: AimeExecutionMode::SharedProvider,
        };
        let manifest = build_manifest(
            &options,
            &loaded,
            Some(AimeDerivedRunContext {
                kind: AimeDerivedRunKind::Replay,
                source_run_dir: PathBuf::from("old-run"),
                source_sample_count: 2,
            }),
            "2026-09-23T00:00:00Z".to_string(),
            vec![
                "corenius".to_string(),
                "eval".to_string(),
                "aime-replay".to_string(),
            ],
        );

        assert_eq!(manifest.schema_version, 1);
        assert_eq!(
            manifest.command_argv,
            vec!["corenius", "eval", "aime-replay"]
        );
        assert_eq!(manifest.dataset, "tracked");
        assert_eq!(manifest.years, vec!["2025-I"]);
        assert_eq!(manifest.model.as_deref(), Some("model-a"));
        assert_eq!(manifest.timeout_ms, 123);
        assert!(manifest.stream);
        assert_eq!(manifest.source_run_dir.as_deref(), Some("old-run"));
        assert_eq!(manifest.replayed_from_sample_count, Some(2));
        assert_eq!(manifest.regraded_from_sample_count, None);
        assert_eq!(manifest.execution_mode, AimeExecutionMode::SharedProvider);
        assert_eq!(manifest.dataset_cache_files.len(), 1);
        assert_eq!(manifest.dataset_cache_files[0].byte_length, 11);
        assert_eq!(
            manifest.dataset_cache_files[0].sha256,
            "e81e0aa57be1f0ad8b210b3e6348740e38a380365692b6d6129c53a84264ac1d"
        );
    }

    fn aime_case(year: &str, id: &str, answer: &str) -> AimeCase {
        AimeCase {
            id: id.to_string(),
            year: year.to_string(),
            answer: answer.to_string(),
            problem: "P".to_string(),
            tags: vec![year.to_string()],
        }
    }

    fn sample(id: &str, year: &str, passed: bool, error: Option<&str>) -> AimeSample {
        AimeSample {
            id: id.to_string(),
            year: year.to_string(),
            tags: vec![year.to_string()],
            source: "gair".to_string(),
            expected_answer: "7".to_string(),
            extracted_answer: if passed { "7" } else { "8" }.to_string(),
            passed,
            score: u8::from(passed),
            elapsed_seconds: 1.25,
            return_status: if error.is_none() {
                Some("end_turn".to_string())
            } else {
                None
            },
            error: error.map(str::to_string),
            prompt: "prompt".to_string(),
            response: if passed {
                "FINAL: 7".to_string()
            } else {
                "FINAL: 8".to_string()
            },
            failure_kind: classify_failure(
                passed,
                if passed { "FINAL: 7" } else { "FINAL: 8" },
                error,
            ),
            usage: None,
        }
    }

    #[test]
    fn run_metrics_preserve_order_and_compute_percentiles() {
        let mut first = sample("2025-I:I-1", "2025-I", true, None);
        first.elapsed_seconds = 1.0;
        let mut second = sample("2025-I:I-2", "2025-I", false, None);
        second.elapsed_seconds = 3.0;
        second.failure_kind = Some(AimeFailureKind::WrongFinal);
        let mut third =
            sample("2025-I:I-3", "2025-I", false, Some("ACP prompt timed out"));
        third.elapsed_seconds = 2.0;
        third.failure_kind = Some(AimeFailureKind::TimeoutNoResponse);

        let metrics = build_run_metrics(
            &[first, second, third],
            AimeExecutionMode::FreshProviderPerSample,
            5000,
        );

        assert_eq!(metrics.schema_version, 1);
        assert_eq!(metrics.sample_count, 3);
        assert_eq!(
            metrics.execution_mode,
            AimeExecutionMode::FreshProviderPerSample
        );
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
        assert_eq!(summary.dataset, format!("regrade:{}", from_run.display()));
        let original = std::fs::read_to_string(from_run.join("samples.jsonl")).unwrap();
        assert!(original.contains("\"extracted_answer\":\"999\""));

        let regraded = std::fs::read_to_string(regrade_run.join("samples.jsonl")).unwrap();
        assert!(regraded.contains("\"extracted_answer\":\"70\""));
        assert!(regraded.contains("\"passed\":true"));

        let delta: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(regrade_run.join("scorer-delta.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(delta["schema_version"], 1);
        assert_eq!(delta["changed_sample_count"], 1);
        assert_eq!(delta["changes"][0]["id"], "2025-I:I-1");

        let manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(regrade_run.join("manifest.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(manifest["dataset"], format!("regrade:{}", from_run.display()));
        assert_eq!(manifest["source_run_dir"], from_run.display().to_string());
        assert_eq!(manifest["regraded_from_sample_count"], 1);

        let triage = std::fs::read_to_string(regrade_run.join("triage.md")).unwrap();
        assert!(triage.contains(&format!("- dataset: regrade:{}", from_run.display())));
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
}

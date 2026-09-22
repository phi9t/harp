use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ProviderEvent, ProviderEventSink, UsageRecord};
use crate::provider::acp::{
    AcpOptions, TraeCliAcpProvider, run_acp_prompt, run_acp_prompt_streaming,
};

const PROMPT_TEMPLATE: &str = r#"Solve this AIME problem. Return the final answer as a single integer in [0, 999].

Problem:
{problem}

Important: End your response with a line exactly of the form FINAL: <integer>."#;

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

#[derive(Clone, Debug, Serialize)]
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
    pub usage: Option<UsageRecord>,
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
    };
    run_aime_eval(eval_options).await
}

pub async fn run_aime_eval(options: AimeEvalOptions) -> anyhow::Result<AimeSummary> {
    let loaded = load_dataset(&options).await?;
    let selected = select_cases(loaded.cases, &options.selection)?;
    if selected.is_empty() {
        bail!("AIME selection is empty");
    }
    tokio::fs::create_dir_all(&options.run_dir)
        .await
        .with_context(|| format!("create run dir {}", options.run_dir.display()))?;
    let samples_path = options.run_dir.join("samples.jsonl");
    let _ = tokio::fs::remove_file(&samples_path).await;

    let provider = TraeCliAcpProvider::new(options.workspace.clone(), options.model.clone());
    let mut samples = Vec::new();
    for case in selected {
        let prompt = prompt_for(&case);
        eprintln!("RUN id={} expected={}", case_key(&case), case.answer);
        let started = Instant::now();
        let result = if options.stream {
            let sink: ProviderEventSink = Arc::new(|event| {
                Box::pin(async move {
                    if let ProviderEvent::AssistantDelta { delta } = event {
                        print!("delta: {delta}");
                    }
                    Ok(())
                })
                    as Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>>
            });
            run_acp_prompt_streaming(
                AcpOptions {
                    command: provider.command.clone(),
                    args: provider.args(),
                    cwd: options.workspace.clone(),
                    model: provider.model.clone(),
                    prompt: prompt.clone(),
                    timeout_ms: options.timeout_ms,
                },
                Some(sink),
            )
            .await
        } else {
            run_acp_prompt(AcpOptions {
                command: provider.command.clone(),
                args: provider.args(),
                cwd: options.workspace.clone(),
                model: provider.model.clone(),
                prompt: prompt.clone(),
                timeout_ms: options.timeout_ms,
            })
            .await
        };
        let elapsed_seconds = started.elapsed().as_secs_f64();
        let (response, return_status, usage, error) = match result {
            Ok(output) => (output.text, output.stop_reason, output.usage, None),
            Err(error) => (String::new(), None, None, Some(error.to_string())),
        };
        let expected = normalize_answer(&case.answer);
        let extracted = extract_answer(&response);
        let passed = error.is_none() && extracted == expected;
        let sample = AimeSample {
            id: case_key(&case),
            year: case.year,
            tags: case.tags,
            source: loaded.source.clone(),
            expected_answer: expected,
            extracted_answer: extracted,
            passed,
            score: u8::from(passed),
            elapsed_seconds,
            return_status,
            error,
            prompt,
            response,
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
    let summary_json = serde_json::to_string_pretty(&summary)?;
    tokio::fs::write(
        options.run_dir.join("summary.json"),
        format!("{summary_json}\n"),
    )
    .await?;
    tokio::fs::write(
        options.run_dir.join("triage.md"),
        render_triage_markdown(&summary, &samples),
    )
    .await?;
    Ok(summary)
}

struct LoadedDataset {
    source: String,
    cases: Vec<AimeCase>,
}

async fn load_dataset(options: &AimeEvalOptions) -> anyhow::Result<LoadedDataset> {
    match &options.source {
        AimeDatasetSource::TrackedJsonl { path } => {
            let dataset_path = resolve_dataset(path)?;
            Ok(LoadedDataset {
                source: display_dataset(path),
                cases: load_cases(&dataset_path)?,
            })
        }
        AimeDatasetSource::Gair {
            years,
            data_dir,
            refresh_data,
        } => {
            let mut cases = Vec::new();
            for year in years {
                let path = ensure_gair_year(year, data_dir, *refresh_data).await?;
                let year_cases = load_gair_cases(year, &path)?;
                validate_gair_year_count(year, &path, year_cases.len())?;
                cases.extend(year_cases);
            }
            Ok(LoadedDataset {
                source: "gair".to_string(),
                cases,
            })
        }
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

fn extract_answer(text: &str) -> String {
    let mut final_answer = None;
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if let Some(offset) = lower.find("final:") {
            final_answer = first_int(&line[offset + "final:".len()..]);
        }
    }
    final_answer
        .or_else(|| {
            let mut last = None;
            for token in numeric_tokens(text) {
                last = Some(token);
            }
            last
        })
        .unwrap_or_default()
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
        },
        run_dir: options.run_dir.display().to_string(),
        timeout_ms: options.timeout_ms,
        by_year,
        included_timeout_probe: match &options.source {
            AimeDatasetSource::TrackedJsonl { .. } => Some(options.selection.include_timeout_probe),
            AimeDatasetSource::Gair { .. } => None,
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

fn render_triage_markdown(summary: &AimeSummary, samples: &[AimeSample]) -> String {
    let mut output = String::new();
    output.push_str("# AIME benchmark triage\n\n");
    output.push_str(&format!(
        "- total: {}\n- correct: {}\n- accuracy: {:.3}\n- dataset: {}\n- timeout_ms: {}\n\n",
        summary.total, summary.correct, summary.accuracy, summary.dataset, summary.timeout_ms
    ));
    output.push_str("## By year\n\n");
    output.push_str("| year | total | correct | accuracy |\n");
    output.push_str("| --- | ---: | ---: | ---: |\n");
    for (year, row) in &summary.by_year {
        output.push_str(&format!(
            "| {} | {} | {} | {:.3} |\n",
            year, row.total, row.correct, row.accuracy
        ));
    }
    output.push_str("\n## Problems\n\n");
    output.push_str("| id | year | expected | extracted | status | elapsed_seconds | note |\n");
    output.push_str("| --- | --- | ---: | ---: | --- | ---: | --- |\n");
    for sample in samples {
        let status = if sample.passed { "pass" } else { "fail" };
        let note = sample.error.as_deref().unwrap_or_default();
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {:.3} | {} |\n",
            escape_md_cell(&sample.id),
            escape_md_cell(&sample.year),
            escape_md_cell(&sample.expected_answer),
            escape_md_cell(&sample.extracted_answer),
            status,
            sample.elapsed_seconds,
            escape_md_cell(note)
        ));
    }
    output
}

fn escape_md_cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
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
            },
        );
        let triage = render_triage_markdown(&summary, &samples);

        assert!(triage.contains("| year | total | correct | accuracy |"));
        assert!(triage.contains("| 2024 | 2 | 1 | 0.500 |"));
        assert!(triage.contains("| 2024:60 | 2024 | 7 | 7 | pass |"));
        assert!(triage.contains("| 2024:61 | 2024 | 7 | 8 | fail |"));
        assert!(triage.contains("ACP prompt timed out"));
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
            response: "response".to_string(),
            usage: None,
        }
    }
}

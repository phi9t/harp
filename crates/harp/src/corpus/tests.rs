use std::fs;

use tempfile::TempDir;

use super::contracts::{
    normalize_link_path, parse_coverage_row, validate_evidence_graph, validate_local_links,
};
use super::render::{offline_link_destination, render_markdown};
use super::rules::PrimaryClassification;
use super::*;

const TEST_WENG_DIGEST: &str = "76f3fdeddb0505f2080eff1497dd6fe2856e300ae8c17356a753e5d4da4e1fef";

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("Harp workspace root")
}

fn formalization_lean_sources(root: &Path) -> Vec<std::path::PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut sources = Vec::new();

    while let Some(directory) = pending.pop() {
        for entry in
            fs::read_dir(directory).expect("formalization source directory must be readable")
        {
            let entry = entry.expect("formalization source entry must be readable");
            let file_type = entry
                .file_type()
                .expect("formalization source file type must be readable");
            if file_type.is_dir() {
                if entry.file_name() != ".lake" {
                    pending.push(entry.path());
                }
            } else if file_type.is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|value| value == "lean")
            {
                sources.push(entry.path());
            }
        }
    }

    sources.sort();
    sources
}

fn contains_nonportable_path_reference(source: &str) -> bool {
    source.as_bytes().iter().enumerate().any(|(index, byte)| {
        if *byte != b'/' {
            return false;
        }

        let starts_token = index == 0
            || matches!(
                source.as_bytes()[index - 1],
                b' ' | b'\n' | b'\r' | b'\t' | b'\'' | b'"' | b'(' | b'[' | b'{' | b'='
            );
        let starts_component = source
            .as_bytes()
            .get(index + 1)
            .is_some_and(u8::is_ascii_alphanumeric);

        starts_token && starts_component
    }) || source.contains("~/")
        || source.contains("~\\")
        || source.contains("file://")
        || source.as_bytes().windows(3).any(|window| {
            window[0].is_ascii_alphabetic()
                && window[1] == b':'
                && matches!(window[2], b'\\' | b'/')
        })
        || source.contains("\\\\")
        || source.starts_with("//")
}

#[test]
fn detects_nonportable_path_references_without_rejecting_math_syntax() {
    for path in [
        "/opt/harp/input",
        "/var/tmp/input",
        "/Volumes/data/input",
        "/usr/local/input",
        "\"/private/tmp/input\"",
        "~/input",
        "~\\input",
        "file:///tmp/input",
        "C:\\input",
        "\\\\server\\share",
    ] {
        assert!(
            contains_nonportable_path_reference(path),
            "{path} must be rejected as a nonportable path reference"
        );
    }

    for source in [
        "a / b",
        "Corollary/application",
        "/- Lean doc comment -/",
        "x ~ y",
    ] {
        assert!(
            !contains_nonportable_path_reference(source),
            "{source} is syntax, not a nonportable path reference"
        );
    }
}

fn fixture() -> TempDir {
    let target = workspace_root().join("target");
    fs::create_dir_all(&target).unwrap();
    let repo = tempfile::Builder::new()
        .prefix("rsi-")
        .tempdir_in(target)
        .unwrap();
    let root = repo.path().join("knowledge/rsi");
    fs::create_dir_all(root.join("chapters")).unwrap();
    fs::create_dir_all(root.join("concepts")).unwrap();
    fs::create_dir_all(repo.path().join("atlas/src/content/generated")).unwrap();
    repo
}

fn write_complete_fixture(repo: &Path) {
    let mut coverage = String::from(COVERAGE_HEADER);
    coverage.push('\n');
    let mut retained = String::from("concept_id\tsource_ids\n");
    for (id, path) in REQUIRED_CHAPTERS {
        coverage.push_str(&format!("{id}\tchapter\t{path}\t\t\n"));
        let sources = if id == "recursive-improvement-loop" {
            "SELF-REFINE"
        } else {
            ""
        };
        retained.push_str(&format!("{id}\t{sources}\n"));
        let title = id.replace('-', " ");
        let markdown = format!(
            "# {title}\n\n## Technical mechanism\n\nMechanism text.\n\n<details>\n{SOURCE_SUMMARY}\n\n- Original source.\n\n</details>\n"
        );
        fs::write(repo.join(path), markdown).unwrap();
    }
    for (_, _, path) in READER_ROUTES {
        if !repo.join(path).exists() {
            fs::create_dir_all(repo.join(path).parent().unwrap()).unwrap();
            fs::write(
                repo.join(path),
                "# Route document\n\n## Canonical route content\n\nRoute text.\n",
            )
            .unwrap();
        }
    }
    for (document_id, path) in AUXILIARY_DOCUMENTS {
        fs::create_dir_all(repo.join(path).parent().unwrap()).unwrap();
        fs::write(
            repo.join(path),
            format!(
                "# {}\n\n## Technical mechanism\n\nAuxiliary document fixture.\n",
                document_id.replace('-', " ")
            ),
        )
        .unwrap();
    }
    let registry = repo.join(SOURCE_REGISTRY_PATH);
    fs::create_dir_all(registry.parent().unwrap()).unwrap();
    fs::write(registry, "source_id\nSELF-REFINE\n").unwrap();
    fs::write(
        repo.join(EVIDENCE_GRAPH_PATH),
        format!("{EVIDENCE_GRAPH_HEADER}\n"),
    )
    .unwrap();
    fs::write(repo.join(COVERAGE_PATH), coverage).unwrap();
    fs::write(repo.join("content/retained-concepts.tsv"), retained).unwrap();
    write_diagnostic_fixture(repo);
}

fn write_diagnostic_fixture(repo: &Path) {
    let root = workspace_root();
    for path in [
        rules::WORKSHEET_FIELDS_PATH,
        rules::DIAGNOSTIC_RULES_PATH,
        rules::EXPORT_SCHEMA_PATH,
    ] {
        let output = repo.join(path);
        fs::create_dir_all(output.parent().unwrap()).unwrap();
        fs::copy(root.join(path), output).unwrap();
    }
    for case_id in [
        "self-refine",
        "ace",
        "mce",
        "adas",
        "aflow",
        "stop",
        "self-harness",
        "ahe",
        "alphaevolve",
        "dgm",
        "sia",
        "continual-harness",
    ] {
        let path = format!("content/diagnostics/cases/{case_id}.json");
        let output = repo.join(&path);
        fs::create_dir_all(output.parent().unwrap()).unwrap();
        fs::copy(root.join(path), output).unwrap();
    }
    let lesson_path = "knowledge/rsi/lessons/01-self-refine.md";
    let manifest = serde_json::json!({
        "schema_version": 1,
        "lessons": [{
            "order": 1,
            "lesson_id": "self-refine",
            "title": "Self-Refine",
            "canonical_markdown_path": lesson_path,
            "case_ids": ["self-refine"],
            "system_ids": [],
            "prompt_ids": ["claim-ceiling"],
            "concept_ids": ["recursive-improvement-loop"],
            "transfer_case_ids": ["self-refine"]
        }]
    });
    let manifest_path = repo.join(lessons::LESSONS_PATH);
    fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    let mut bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    bytes.push(b'\n');
    fs::write(manifest_path, bytes).unwrap();
    let output = repo.join(lesson_path);
    fs::create_dir_all(output.parent().unwrap()).unwrap();
    fs::write(
        output,
        "# Self-Refine\n\n## Observe\n\nObserve.\n\n## Predict\n\nPredict.\n\n## Compare\n\nCompare.\n\n## Explain\n\nExplain.\n\n## Missing fact\n\nMissing.\n\n## Transfer\n\nTransfer.\n",
    )
    .unwrap();
}

fn mutate_json(repo: &Path, relative: &str, mutate: impl FnOnce(&mut serde_json::Value)) {
    let path = repo.join(relative);
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    mutate(&mut value);
    let mut bytes = serde_json::to_vec_pretty(&value).unwrap();
    bytes.push(b'\n');
    fs::write(path, bytes).unwrap();
}

fn write_complete_system_registry_fixture(repo: &Path) {
    let systems = [
        (
            "autoresearch",
            "KARPATHY-AUTORESEARCH",
            "system-being-improved",
        ),
        ("ace", "ACE", "context-engineering"),
        ("mce", "MCE", "context-engineering"),
        ("meta-harness", "META-HARNESS", "self-improving-harnesses"),
        ("adas", "ADAS", "workflow-design-and-search"),
        ("aflow", "AFLOW", "workflow-design-and-search"),
        ("stop", "STOP", "self-improving-harnesses"),
        ("self-harness", "SELF-HARNESS", "self-improving-harnesses"),
        (
            "harness-disentangle",
            "HARNESS-DISENTANGLE",
            "harness-layer-vs-core-intelligence",
        ),
        ("ahe", "AHE", "self-improving-harnesses"),
        ("alphaevolve", "ALPHAEVOLVE", "evolutionary-search"),
        ("dgm", "DGM", "evolutionary-search"),
        ("ai-scientist", "AI-SCIENTIST", "system-being-improved"),
        ("sia", "SIA", "joint-harness-weight-optimization"),
        (
            "continual-harness",
            "CONTINUAL-HARNESS",
            "joint-harness-weight-optimization",
        ),
        ("rlm", "RLM-PAPER", "harness-layer-vs-core-intelligence"),
    ];
    let rows = systems
        .iter()
        .map(|(system_id, source_id, section_id)| {
            let diagnostic_case_path = matches!(
                *system_id,
                "ace"
                    | "mce"
                    | "adas"
                    | "aflow"
                    | "stop"
                    | "self-harness"
                    | "ahe"
                    | "alphaevolve"
                    | "dgm"
                    | "sia"
                    | "continual-harness"
            )
            .then(|| format!("content/diagnostics/cases/{system_id}.json"));
            serde_json::json!({
                "system_id": system_id,
                "title": system_id.replace('-', " "),
                "source_ids": [source_id],
                "treatment": "full",
                "publication_state": "planned",
                "canonical_markdown_path": format!("knowledge/rsi/systems/{system_id}.md"),
                "weng_section_ids": [section_id],
                "paper_routes": [{
                    "source_id": source_id,
                    "public_url": format!("https://example.com/{system_id}"),
                    "captured_locator": format!("evidence/weng/text/{system_id}.txt:1-2"),
                    "reading_sequence": {
                        "method": "§2",
                        "algorithm": "§3, Algorithm 1",
                        "main_evaluation": "§4, Results",
                        "ablation": "§4, Table 1",
                        "limitations": "§5, Limitations",
                        "appendix": "Appendix A"
                    }
                }],
                "diagnostic_case_path": diagnostic_case_path,
                "related_system_ids": []
            })
        })
        .collect::<Vec<_>>();
    let path = repo.join(SYSTEM_READINGS_PATH);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "schema_version": 1,
        "systems": rows
    }))
    .unwrap();
    bytes.push(b'\n');
    fs::write(path, bytes).unwrap();
    let capture_root = repo.join("evidence/weng/text");
    fs::create_dir_all(&capture_root).unwrap();
    for (system_id, _, _) in systems {
        fs::write(
            capture_root.join(format!("{system_id}.txt")),
            "fixture line one\nfixture line two\n",
        )
        .unwrap();
    }

    let mut source_ids = systems
        .iter()
        .map(|(_, source_id, _)| (*source_id).to_owned())
        .collect::<Vec<_>>();
    source_ids.push("WENG-HARNESS".to_owned());
    source_ids.push("SELF-REFINE".to_owned());
    source_ids.extend((1..=39).map(|number| format!("SOURCE-{number}")));
    source_ids.sort();
    source_ids.dedup();

    let retained_path = repo.join(RETAINED_CONCEPTS_PATH);
    let retained = fs::read_to_string(&retained_path).unwrap();
    let retained = retained.replacen(
        "recursive-improvement-loop\tSELF-REFINE\n",
        &format!("recursive-improvement-loop\t{}\n", source_ids.join(",")),
        1,
    );
    fs::write(retained_path, retained).unwrap();

    let registry_path = repo.join(SOURCE_REGISTRY_PATH);
    let mut registry = String::from("source_id\tversion_or_digest\n");
    for source_id in &source_ids {
        let digest = if source_id == "WENG-HARNESS" {
            format!("sha256:{TEST_WENG_DIGEST}")
        } else {
            "fixture".to_owned()
        };
        registry.push_str(&format!("{source_id}\t{digest}\n"));
    }
    fs::write(registry_path, registry).unwrap();

    let mut graph = format!("{EVIDENCE_GRAPH_HEADER}\n");
    for number in 1..=39 {
        graph.push_str(&format!(
            "EVIDENCE\tWENG-HARNESS\tcites\tSOURCE-{number}\tReference {number}\tverified\tIdentity edge only\n"
        ));
    }
    fs::write(repo.join(EVIDENCE_GRAPH_PATH), graph).unwrap();
}

fn write_complete_weng_fixture(repo: &Path) {
    let sections = [
        ("system-being-improved", "System being improved"),
        ("harness-design-patterns", "Harness design patterns"),
        (
            "harness-layer-vs-core-intelligence",
            "Harness layer versus core intelligence",
        ),
        ("context-engineering", "Context engineering"),
        ("workflow-design-and-search", "Workflow design and search"),
        ("self-improving-harnesses", "Self-improving harnesses"),
        ("evolutionary-search", "Evolutionary search"),
        (
            "joint-harness-weight-optimization",
            "Joint harness and weight optimization",
        ),
        ("future-challenges", "Future challenges"),
    ];
    let systems: serde_json::Value =
        serde_json::from_slice(&fs::read(repo.join(SYSTEM_READINGS_PATH)).unwrap()).unwrap();
    let system_rows = systems["systems"].as_array().unwrap();
    let map_sections = sections
        .iter()
        .enumerate()
        .map(|(index, (section_id, title))| {
            let companion_path = format!("knowledge/rsi/weng/{:02}-{section_id}.md", index + 1);
            let system_ids = system_rows
                .iter()
                .filter(|row| {
                    row["weng_section_ids"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|id| id == section_id)
                })
                .map(|row| row["system_id"].as_str().unwrap())
                .collect::<Vec<_>>();
            let markdown = format!(
                "# {title}\n\n## What Weng claims\n\nClaim.\n\n## Mechanism\n\nMechanism.\n\n## Hidden assumption\n\nAssumption.\n\n## Demonstrated versus proposed\n\nBoundary.\n\n## What would weaken this interpretation\n\nFalsifier.\n\n## Reader checkpoint\n\nQuestion.\n\n<details>\n<summary>Check your answer</summary>\n\nAnswer.\n\n</details>\n"
            );
            let path = repo.join(&companion_path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, markdown).unwrap();
            serde_json::json!({
                "order": index + 1,
                "section_id": section_id,
                "title": title,
                "public_url": format!("https://lilianweng.github.io/posts/2026-07-04-harness/#{section_id}"),
                "captured_locator": format!("evidence/weng/artifacts/html/weng-harness.html#{section_id}"),
                "companion_path": companion_path,
                "companion_section": "mechanism",
                "system_ids": system_ids,
                "exercise_ids": [format!("{section_id}-checkpoint")],
                "figure_locators": []
            })
        })
        .collect::<Vec<_>>();
    let mut bytes = serde_json::to_vec_pretty(&serde_json::json!({
        "schema_version": 1,
        "source_id": "WENG-HARNESS",
        "source_sha256": TEST_WENG_DIGEST,
        "sections": map_sections
    }))
    .unwrap();
    bytes.push(b'\n');
    fs::write(repo.join(WENG_MAP_PATH), bytes).unwrap();
    let capture = repo.join("evidence/weng/artifacts/html/weng-harness.html");
    fs::create_dir_all(capture.parent().unwrap()).unwrap();
    let anchors = sections
        .iter()
        .map(|(section_id, _)| format!("<h2 id=\"{section_id}\"></h2>"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(capture, anchors).unwrap();
}

#[test]
fn weng_map_requires_contiguous_order_unique_ids_and_captured_locators() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    write_complete_system_registry_fixture(repo.path());
    write_complete_weng_fixture(repo.path());
    mutate_json(repo.path(), WENG_MAP_PATH, |value| {
        value["sections"][1]["order"] = serde_json::json!(1);
    });
    assert_eq!(
        compile(repo.path()).unwrap_err().code(),
        "knowledge.rsi.weng_order"
    );
}

#[test]
fn weng_map_rejects_unknown_system_and_companion_ids() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    write_complete_system_registry_fixture(repo.path());
    write_complete_weng_fixture(repo.path());
    mutate_json(repo.path(), WENG_MAP_PATH, |value| {
        value["sections"][0]["system_ids"] = serde_json::json!(["absent-system"]);
    });
    assert_eq!(
        compile(repo.path()).unwrap_err().code(),
        "knowledge.rsi.weng_system"
    );
}

#[test]
fn weng_map_binds_the_registered_weng_digest() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    write_complete_system_registry_fixture(repo.path());
    write_complete_weng_fixture(repo.path());
    mutate_json(repo.path(), WENG_MAP_PATH, |value| {
        value["source_sha256"] = serde_json::json!("0".repeat(64));
    });
    assert_eq!(
        compile(repo.path()).unwrap_err().code(),
        "knowledge.rsi.weng_digest"
    );
}

#[test]
fn weng_map_compiles_each_guided_companion_exactly_once() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    write_complete_system_registry_fixture(repo.path());
    write_complete_weng_fixture(repo.path());

    let corpus = compile(repo.path()).unwrap();
    let companions = corpus
        .documents
        .iter()
        .filter(|document| {
            document
                .canonical_markdown_path
                .starts_with("knowledge/rsi/weng/")
        })
        .collect::<Vec<_>>();

    assert_eq!(companions.len(), 9);
    assert_eq!(
        companions
            .iter()
            .map(|document| document.canonical_markdown_path.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        9
    );
    assert!(companions
        .iter()
        .all(|document| document.concept_id.starts_with("weng-")));
}

#[test]
fn v5_payload_contains_ordered_weng_sections() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    write_complete_system_registry_fixture(repo.path());
    write_complete_weng_fixture(repo.path());

    let corpus = compile(repo.path()).unwrap();

    assert_eq!(corpus.schema_version, "rsi-technical-atlas/v5");
    assert_eq!(corpus.weng_sections.len(), 9);
    assert_eq!(corpus.systems.len(), 16);
    assert!(corpus
        .weng_sections
        .iter()
        .enumerate()
        .all(|(index, section)| usize::from(section.order) == index + 1));
}

#[test]
fn system_reading_coverage_requires_a_system_markdown_path_without_a_section() {
    let row = "aflow\tsystem-reading\tknowledge/rsi/systems/aflow.md\t\tharness-search";
    let entry = parse_coverage_row(2, row).unwrap();

    assert_eq!(entry.coverage_depth, CoverageDepth::SystemReading);
    assert_eq!(
        entry.canonical_markdown_path,
        "knowledge/rsi/systems/aflow.md"
    );
    assert_eq!(entry.section_id, None);
}

#[test]
fn a_published_system_compiles_as_a_system_reading_document() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    write_complete_system_registry_fixture(repo.path());
    write_complete_weng_fixture(repo.path());

    let aflow_path = "knowledge/rsi/systems/aflow.md";
    fs::create_dir_all(repo.path().join("knowledge/rsi/systems")).unwrap();
    fs::write(
        repo.path().join(aflow_path),
        "# AFlow\n\n## Algorithm\n\nMCTS workflow search.\n\n<details>\n<summary>Original sources for this mechanism</summary>\n\n- Official paper.\n\n</details>\n\n## Evaluation\n\nAuthor-reported evaluation.\n\n## Claim ceiling\n\nNo independent RSI claim.\n",
    )
    .unwrap();
    mutate_json(repo.path(), SYSTEM_READINGS_PATH, |value| {
        let aflow = value["systems"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|system| system["system_id"] == "aflow")
            .unwrap();
        aflow["publication_state"] = serde_json::json!("published");
    });
    let coverage_path = repo.path().join(COVERAGE_PATH);
    let coverage = fs::read_to_string(&coverage_path).unwrap();
    fs::write(
        coverage_path,
        format!("{coverage}aflow\tsystem-reading\t{aflow_path}\t\tharness-search\n"),
    )
    .unwrap();
    let retained_path = repo.path().join(RETAINED_CONCEPTS_PATH);
    let retained = fs::read_to_string(&retained_path).unwrap();
    let retained = retained
        .lines()
        .map(|line| {
            let Some((concept_id, sources)) = line.split_once('\t') else {
                return line.to_owned();
            };
            if concept_id != "recursive-improvement-loop" {
                return line.to_owned();
            }
            let sources = sources
                .split(',')
                .filter(|source| *source != "AFLOW")
                .collect::<Vec<_>>()
                .join(",");
            format!("{concept_id}\t{sources}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(retained_path, format!("{retained}\naflow\tAFLOW\n")).unwrap();

    let corpus = compile(repo.path()).unwrap();
    let system_readings = corpus
        .coverage
        .iter()
        .filter(|entry| entry.coverage_depth == CoverageDepth::SystemReading)
        .collect::<Vec<_>>();
    let aflow = corpus
        .documents
        .iter()
        .find(|document| document.canonical_markdown_path == aflow_path)
        .unwrap();

    assert_eq!(system_readings.len(), 1);
    assert_eq!(aflow.concept_id, "aflow");
    assert!(aflow.html.contains("<h2>Algorithm</h2>"));
    assert!(aflow.html.contains("<h2>Evaluation</h2>"));
    assert!(aflow.html.contains("<h2>Claim ceiling</h2>"));
}

#[test]
fn repository_defines_the_strict_diagnostic_contract_files() {
    let root = workspace_root();
    for path in [
        "content/diagnostics/worksheet-fields.json",
        "content/diagnostics/rules.json",
        "content/diagnostics/export-schema.json",
        "content/diagnostics/cases/self-refine.json",
        "content/diagnostics/cases/aflow.json",
        "content/diagnostics/cases/continual-harness.json",
    ] {
        assert!(
            root.join(path).is_file(),
            "missing diagnostic contract {path}"
        );
    }
}

#[test]
fn full_system_readings_require_a_complete_original_source_sequence() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    write_complete_system_registry_fixture(repo.path());
    mutate_json(repo.path(), SYSTEM_READINGS_PATH, |value| {
        value["systems"][0]["paper_routes"][0]["reading_sequence"]
            .as_object_mut()
            .unwrap()
            .remove("ablation");
    });

    let error = compile(repo.path()).unwrap_err();

    assert_eq!(error.code(), "knowledge.rsi.system_registry");
}

#[test]
fn v5_payload_contains_all_approved_diagnostic_cases() {
    let corpus = compile(workspace_root()).unwrap();
    let expected = BTreeMap::from([
        ("ace", PrimaryClassification::PersistentAdaptation),
        ("adas", PrimaryClassification::HarnessImprovement),
        ("aflow", PrimaryClassification::HarnessImprovement),
        ("alphaevolve", PrimaryClassification::PersistentAdaptation),
        ("ahe", PrimaryClassification::HarnessImprovement),
        (
            "continual-harness",
            PrimaryClassification::JointHarnessWeightAdaptation,
        ),
        ("dgm", PrimaryClassification::HarnessImprovement),
        ("mce", PrimaryClassification::HarnessImprovement),
        ("self-harness", PrimaryClassification::HarnessImprovement),
        ("self-refine", PrimaryClassification::OutputRefinement),
        ("sia", PrimaryClassification::JointHarnessWeightAdaptation),
        ("stop", PrimaryClassification::HarnessImprovement),
    ]);
    let actual = corpus
        .diagnostics
        .cases
        .iter()
        .map(|case| {
            (
                case.case_id.as_str(),
                (case.expected_classification, case.expected_claim_ceiling),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let expected = expected
        .into_iter()
        .map(|(case_id, classification)| (case_id, (classification, classification)))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(corpus.schema_version, "rsi-technical-atlas/v5");
    assert_eq!(actual, expected);
}

#[test]
fn diagnostic_contract_matches_the_approved_fields_and_rules() {
    let corpus = compile(workspace_root()).unwrap();
    let fields = corpus
        .diagnostics
        .worksheet
        .fields
        .iter()
        .map(|field| field.field_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        fields,
        [
            "candidate-name",
            "system-kind",
            "editable-components",
            "proposal-owner",
            "task-outcome-measured",
            "task-outcome-improved",
            "persistence-scope",
            "persistence-mechanism",
            "later-consumer-present",
            "accepted-generation-edge",
            "generation-edge-owner",
            "rejected-lineage-retained",
            "accepted-child-produces-later-candidates",
            "held-out-outcomes",
            "evaluator-owner",
            "candidate-can-write-evaluator",
            "development-heldout-split",
            "matched-proposal-protocol",
            "complete-root-tree-accounting",
            "permissions-external",
            "archive-external",
            "promotion-external",
            "rollback-external",
            "next-cycle-gain-measured",
            "next-cycle-gain-positive",
            "evidence-status",
            "controls-present",
            "ablations-present",
        ]
    );

    let rule_ids = corpus
        .diagnostics
        .rules
        .rules
        .iter()
        .map(|rule| rule.rule_id.as_str())
        .collect::<BTreeSet<_>>();
    for required in [
        "class.joint-harness-weight",
        "class.automated-ai-research",
        "class.harness-improvement",
        "class.persistent-adaptation",
        "class.output-refinement",
        "ceiling.recursive-demonstrated",
        "ceiling.successor-improvement",
        "ceiling.joint-harness-weight",
        "ceiling.automated-ai-research",
        "ceiling.harness-improvement",
        "ceiling.persistent-adaptation",
        "ceiling.output-refinement",
        "integrity.evaluator-write",
        "integrity.permissions-internal",
        "integrity.archive-internal",
        "authority.promotion-internal",
        "budget.incomplete-root-tree",
        "generation.no-protected-edge",
        "generation.no-next-cycle-measurement",
        "evidence.no-heldout-split",
    ] {
        assert!(
            rule_ids.contains(required),
            "missing required rule {required}"
        );
    }
}

#[test]
fn handwritten_typescript_does_not_own_canonical_technical_prose() {
    fn visit(directory: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|name| name.to_str()) != Some("generated") {
                    visit(&path, files);
                }
            } else if matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("ts" | "tsx")
            ) && !path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains(".test."))
            {
                files.push(path);
            }
        }
    }

    let root = workspace_root().join("atlas/src");
    let mut files = Vec::new();
    visit(&root, &mut files);
    for path in files {
        let source = fs::read_to_string(&path).unwrap();
        for forbidden in [
            "EVIDENCE —",
            "CLAIM —",
            "INFERENCE —",
            "MISSING.",
            "Original sources for this mechanism",
            "Author-reported results",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} owns canonical technical prose marker {forbidden:?}",
                path.display()
            );
        }
    }
}

#[test]
fn v5_payload_contains_six_ordered_comparison_lessons() {
    let corpus = compile(workspace_root()).unwrap();

    assert_eq!(corpus.lessons.len(), 6);
    assert!(corpus
        .lessons
        .iter()
        .enumerate()
        .all(|(index, lesson)| usize::from(lesson.order) == index + 1));
    for lesson in &corpus.lessons {
        assert!(corpus.documents.iter().any(|document| {
            document.concept_id == format!("lesson-{}", lesson.lesson_id)
                && document.canonical_markdown_path == lesson.canonical_markdown_path
        }));
    }
}

#[test]
fn auxiliary_technical_documents_are_compiled_and_routable() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    fs::write(
        repo.path()
            .join("knowledge/rsi/codex_state_continuity_and_compaction.md"),
        "# Codex continuity\n\n## Mechanism\n\nState.\n",
    )
    .unwrap();
    fs::write(
        repo.path()
            .join("knowledge/rsi/context_engineering_deep_dive.md"),
        "# Context engineering\n\n## Mechanism\n\nContext.\n",
    )
    .unwrap();
    fs::create_dir_all(repo.path().join("knowledge/rsi/sicp")).unwrap();
    fs::write(
        repo.path().join("knowledge/rsi/sicp/agentic_eval_apply.md"),
        "# Agentic eval/apply\n\n## Mechanism\n\nEvaluate before applying.\n",
    )
    .unwrap();
    let harness = repo
        .path()
        .join("knowledge/rsi/chapters/harness-engineering.md");
    let mut markdown = fs::read_to_string(&harness).unwrap();
    markdown.push_str("\n[Codex continuity](../codex_state_continuity_and_compaction.md)\n");
    fs::write(harness, markdown).unwrap();

    let corpus = compile(repo.path()).unwrap();
    let document = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "codex-state-continuity")
        .unwrap();
    let context_document = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "context-engineering-deep-dive")
        .unwrap();
    let agentic_eval_apply_document = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "agentic-eval-apply")
        .unwrap();
    let harness = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "harness-engineering")
        .unwrap();

    assert_eq!(
        document.canonical_markdown_path,
        "knowledge/rsi/codex_state_continuity_and_compaction.md"
    );
    assert_eq!(
        context_document.canonical_markdown_path,
        "knowledge/rsi/context_engineering_deep_dive.md"
    );
    assert_eq!(
        agentic_eval_apply_document.canonical_markdown_path,
        "knowledge/rsi/sicp/agentic_eval_apply.md"
    );
    assert!(harness
        .html
        .contains("href=\"#documents/codex-state-continuity\""));
}

#[test]
fn agentic_eval_apply_is_discoverable_and_links_to_registered_documents() {
    let corpus = compile(workspace_root()).unwrap();

    assert!(corpus.reader_routes.iter().any(|route| {
        route.route_id == "agentic-eval-apply"
            && route.label == "Agentic eval/apply"
            && route.canonical_markdown_path == "knowledge/rsi/sicp/agentic_eval_apply.md"
    }));

    let essay = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "agentic-eval-apply")
        .unwrap();
    for target in [
        "sicp-seminar-09-eval-apply",
        "sicp-evaluator-deep-dive",
        "pi-harness-deep-dive",
        "hermes-harness-deep-dive",
        "codex-harness-deep-dive",
        "agent-harness-architecture-dossier",
    ] {
        assert!(
            essay
                .html
                .contains(&format!("href=\"#documents/{target}\"")),
            "agentic eval/apply is missing the offline route for {target}"
        );
    }
}

#[test]
fn rsi_standalone_consolidation_record_is_registered_with_its_pinned_boundary() {
    let corpus = compile(workspace_root()).unwrap();
    let document = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "rsi-standalone-consolidation")
        .expect("RSI consolidation record must be compiled");

    assert_eq!(
        document.canonical_markdown_path,
        "knowledge/rsi/standalone_consolidation.md"
    );
    for required in [
        "861232a70beed5792c7f73ea00ee4cf4faebea7b",
        "d14d0c5d2f07bfcecebe300d047fbb99e8ee36db",
        "source_reconciliation.tsv",
        "Harp is the only maintained authoring location",
    ] {
        assert!(
            document.html.contains(required),
            "RSI consolidation record is missing {required}"
        );
    }
    assert!(
        AUXILIARY_DOCUMENTS.iter().any(|(document_id, path)| {
            *document_id == "rsi-standalone-consolidation"
                && *path == "knowledge/rsi/standalone_consolidation.md"
        }),
        "RSI consolidation record must be registered as an auxiliary document"
    );
}

#[test]
fn rlm_system_reading_links_eval_apply_and_preserves_code_boundaries() {
    let corpus = compile(workspace_root()).unwrap();
    let document = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "rlm")
        .unwrap();

    for required in [
        "href=\"#agentic-eval-apply\"",
        "rlm-minimal",
        "seven registered adapters",
        "Local, IPython, and Docker",
        "not a reliable tree-wide cap",
        "selected full-repository files",
        "lossy, model-generated context projection",
        "depth-one training adapter",
    ] {
        assert!(
            document.html.contains(required),
            "RLM system reading is missing {required}"
        );
    }

    let system = corpus
        .systems
        .iter()
        .find(|system| system.system_id == "rlm")
        .unwrap();
    assert_eq!(
        system.source_ids,
        ["RLM-PAPER", "RLM-REPO", "RLM-MINIMAL"],
        "RLM system source ownership must include the paper and both code pins"
    );
    assert_eq!(system.paper_routes.len(), 1);
    assert_eq!(system.paper_routes[0].source_id, "RLM-PAPER");
}

#[test]
fn context_engineering_companion_preserves_reader_and_source_contract() {
    let corpus = compile(workspace_root()).unwrap();
    let document = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "context-engineering-deep-dive")
        .unwrap();

    for required in [
        "Context engineering reading map",
        "Context pipeline checkpoint",
        "ACE reader checkpoint",
        "MCE reader checkpoint",
        "Meta-Harness reader checkpoint",
        "href=\"https://arxiv.org/abs/2510.04618\"",
        "href=\"https://arxiv.org/abs/2601.21557\"",
        "href=\"https://arxiv.org/abs/2603.28052\"",
        "href=\"#documents/codex-state-continuity\"",
    ] {
        assert!(
            document.html.contains(required),
            "context-engineering companion is missing {required}"
        );
    }
}

#[test]
fn meta_harness_deep_dive_preserves_evidence_tiers_and_claim_ceiling() {
    let corpus = compile(workspace_root()).unwrap();
    let document = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "meta-harness-deep-dive")
        .unwrap();

    assert_eq!(
        document.canonical_markdown_path,
        "knowledge/meta_harness/meta_harness_deep_dive.md"
    );
    for required in [
        "Source tiers and claim ceilings",
        "arXiv v1 paper",
        "dated project page",
        "pinned public repositories",
        "local TRAE proposal experiment",
        "Candidate, archive, proposer, evaluator, and Pareto flow",
        "Text-classification contract",
        "TerminalBench-2 boundaries",
        "Experimental Harbor controller",
        "illustrative, not the final reported run",
        "valid candidate count is zero",
        "Held-out leakage",
        "Candidate import execution",
        "Proposer privilege",
        "Archive growth",
        "Score-only selection",
        "Trace contamination",
        "Malformed candidates",
        "Weak reproduction evidence",
        "ACE",
        "MCE",
        "Harness optimization is not demonstrated recursive successor improvement",
        "No benchmark score, paid model evaluation, or held-out result was reproduced",
        "evidence/meta_harness/trae_run/normalized/validation.json",
        "evidence/implementations/meta_harness/snapshot/reference_examples/text_classification/meta_harness.py",
        "evidence/implementations/meta_harness/snapshot/experimental/harbor_meta_harness/controller.py",
    ] {
        assert!(
            document.html.contains(required),
            "Meta-Harness deep dive is missing {required}"
        );
    }
}

#[test]
fn benchmark_field_guide_is_registered_and_preserves_protocol_boundaries() {
    let corpus = compile(workspace_root()).unwrap();
    assert!(corpus.reader_routes.iter().any(|route| {
        route.route_id == "benchmarks"
            && route.label == "Benchmarks"
            && route.canonical_markdown_path
                == "knowledge/harness_benchmarks/harness_benchmark_field_guide.md"
    }));

    let document = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "harness-benchmark-field-guide")
        .unwrap();
    assert_eq!(
        document.canonical_markdown_path,
        "knowledge/harness_benchmarks/harness_benchmark_field_guide.md"
    );
    for required in [
        "Evidence tiers and claim ceilings",
        "Terminal-Bench 2",
        "extract-elf",
        "astropy__astropy-12907",
        "cpp__all-your-base",
        "flink-query",
        "Meta-Harness",
        "Darwin Gödel Machine",
        "Self-Harness",
        "Agentic Harness Engineering",
        "Harness Disentangle",
        "Appendix A — Task and generator contracts",
        "FiNER",
        "USPTO-50k",
        "Symptom2Disease",
        "LawBench",
        "AEGIS2",
        "Appendix B — Agent-prompt contracts",
        "Meta-agent contract",
        "Base-agent contract",
        "Appendix C — Candidate skill representations",
        "Appendix D — Evolved task strategies",
        "Appendix E — Utility implementation contract",
        "five context-optimization epochs",
        "No local reproduction",
    ] {
        assert!(
            document.html.contains(required),
            "benchmark field guide is missing {required}"
        );
    }
}

#[test]
fn evaluator_integrity_packet_is_registered_and_keeps_claims_conditional() {
    let corpus = compile(workspace_root()).unwrap();
    assert!(corpus.reader_routes.iter().any(|route| {
        route.route_id == "evaluator-integrity"
            && route.label == "Evaluator integrity"
            && route.canonical_markdown_path
                == "knowledge/evaluator_integrity/evaluator_integrity_benchmark_suite.md"
    }));

    let document = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "evaluator-integrity-benchmark-suite")
        .unwrap();
    assert_eq!(
        document.canonical_markdown_path,
        "knowledge/evaluator_integrity/evaluator_integrity_benchmark_suite.md"
    );
    for required in [
        "GDPval",
        "DeepSWE",
        "FrontierCode 1.1",
        "SWE-bench Verified",
        "Access boundary",
        "Contamination history",
        "Pre-exposure resistance",
        "Live-evaluation resistance",
        "Evaluator disclosure",
        "Benchmark-selection protocol",
        "repository-repair capability",
        "long-horizon engineering execution",
        "tool/internet-policy robustness",
        "economically meaningful knowledge-work output",
        "No cross-benchmark ranking",
        "Under the documented access policy",
    ] {
        assert!(
            document.html.contains(required),
            "evaluator-integrity benchmark packet is missing {required}"
        );
    }
}

#[test]
fn agentic_engineering_packet_is_registered_and_preserves_authority_boundary() {
    let corpus = compile(workspace_root()).unwrap();

    assert!(corpus.reader_routes.iter().any(|route| {
        route.route_id == "agentic-engineering"
            && route.label == "Agentic engineering"
            && route.canonical_markdown_path
                == "knowledge/agentic_engineering/kenn_reference_architecture.md"
    }));

    let reference = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "agentic-engineering-reference")
        .unwrap();
    assert_eq!(
        reference.canonical_markdown_path,
        "knowledge/agentic_engineering/kenn_reference_architecture.md"
    );
    for required in [
        "scaling agent execution",
        "bounded agency",
        "Constitution as harness policy",
        "honor the request",
        "verify reality",
        "human-owned merge",
        "does not reproduce Kenn",
    ] {
        assert!(
            reference.html.contains(required),
            "agentic engineering reference is missing {required}"
        );
    }

    let index = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "agentic-engineering-index")
        .unwrap();
    assert!(
        index.html.contains("href=\"#agentic-engineering\""),
        "agentic engineering index is missing the first-class reference route"
    );
    for target in [
        "agentic-engineering-tool-stack",
        "agentic-engineering-source-registry",
        "agentic-engineering-claim-ledger",
        "agentic-engineering-missing-evidence",
    ] {
        assert!(
            index
                .html
                .contains(&format!("href=\"#documents/{target}\"")),
            "agentic engineering index is missing the offline route for {target}"
        );
    }

    let tool_stack = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "agentic-engineering-tool-stack")
        .unwrap();
    for required in [
        "Kata: durable intent ledger",
        "Forge: human control plane",
        "Ghosthub: session multiplexing",
        "AgentsView: observability plane",
        "roborev: verification plane",
        "Superpowers: reasoning and execution scaffolding",
    ] {
        assert!(
            tool_stack.html.contains(required),
            "agentic engineering tool stack investigation is missing {required}"
        );
    }
}

#[test]
fn verifies_the_nine_chapter_spine_and_native_source_folds() {
    let repo = fixture();
    write_complete_fixture(repo.path());

    let corpus = compile(repo.path()).unwrap();
    let chapter_count = corpus
        .coverage
        .iter()
        .filter(|entry| entry.coverage_depth == CoverageDepth::Chapter)
        .count();
    let covered = corpus
        .coverage
        .iter()
        .map(|entry| entry.concept_id.as_str())
        .collect::<BTreeSet<_>>();
    let retained = corpus
        .retained_concepts
        .iter()
        .map(|entry| entry.concept_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();

    assert_eq!(chapter_count, 9);
    assert_eq!(retained.difference(&covered).count(), 0);
    assert_eq!(
        corpus
            .coverage
            .iter()
            .filter(|entry| !seen.insert(entry.concept_id.as_str()))
            .count(),
        0
    );
}

#[test]
fn compiles_reader_routes_from_canonical_markdown() {
    let repo = fixture();
    write_complete_fixture(repo.path());

    fs::write(
        repo.path().join("knowledge/harp_knowledge_home.md"),
        "---\n\
id: harp-knowledge-home\n\
title: Harp knowledge home\n\
type: research-index\n\
status: active\n\
tags: [harp, knowledge]\n\
confidence: high\n\
---\n\
# Harp knowledge home\n",
    )
    .unwrap();
    fs::create_dir_all(repo.path().join("knowledge/darwinx")).unwrap();
    for (name, id) in [
        ("darwinx_index.md", "darwinx-index"),
        ("01_mechanism_and_selection.md", "darwinx-mechanism"),
        ("02_evaluation_audit.md", "darwinx-evaluation"),
        ("03_critical_review.md", "darwinx-review"),
        ("04_comparative_synthesis.md", "darwinx-synthesis"),
        ("05_successor_experiment.md", "darwinx-successor"),
        ("claim_evidence_ledger.md", "darwinx-claim-ledger"),
        ("source_registry.md", "darwinx-source-registry"),
        ("maintenance.md", "darwinx-maintenance"),
    ] {
        fs::write(
            repo.path().join("knowledge/darwinx").join(name),
            format!(
                "---\nid: {id}\ntitle: {id}\ntype: technical-deep-dive\nstatus: active\ntags: [darwinx]\nconfidence: high\n---\n# {id}\n"
            ),
        )
        .unwrap();
    }

    let corpus = compile(repo.path()).unwrap();

    assert_eq!(
        corpus
            .reader_routes
            .iter()
            .map(|route| route.route_id.as_str())
            .collect::<Vec<_>>(),
        [
            "thesis",
            "loop",
            "methods",
            "harnesses",
            "weng",
            "experiment",
            "sources",
            "agentic-eval-apply",
            "benchmarks",
            "evaluator-integrity",
            "survey",
            "verified-coevolution",
            "agentic-engineering",
            "crouzeix-conjecture",
            "mathematical-foundations",
            "autodiff-geometry",
            "knowledge",
        ]
    );
    assert!(corpus.reader_routes.iter().any(|route| {
        route.route_id == "knowledge"
            && route.canonical_markdown_path == "knowledge/harp_knowledge_home.md"
    }));
    for path in [
        "knowledge/darwinx/darwinx_index.md",
        "knowledge/darwinx/01_mechanism_and_selection.md",
        "knowledge/darwinx/02_evaluation_audit.md",
        "knowledge/darwinx/03_critical_review.md",
        "knowledge/darwinx/04_comparative_synthesis.md",
        "knowledge/darwinx/05_successor_experiment.md",
        "knowledge/darwinx/claim_evidence_ledger.md",
        "knowledge/darwinx/source_registry.md",
        "knowledge/darwinx/maintenance.md",
    ] {
        assert!(corpus
            .documents
            .iter()
            .any(|document| document.canonical_markdown_path == path));
    }
    for route in &corpus.reader_routes {
        assert!(corpus
            .documents
            .iter()
            .any(|document| { document.canonical_markdown_path == route.canonical_markdown_path }));
    }
}

#[test]
fn compiles_the_mathematical_foundations_route_and_auxiliary_documents() {
    let repo = fixture();
    write_complete_fixture(repo.path());

    let corpus = compile(repo.path()).unwrap();

    assert!(corpus.reader_routes.iter().any(|route| {
        route.route_id == "mathematical-foundations"
            && route.label == "Math foundations"
            && route.canonical_markdown_path
                == "knowledge/mathematical_foundations/mathematical_foundations_index.md"
    }));

    let document_ids = corpus
        .documents
        .iter()
        .filter(|document| document.concept_id.starts_with("math-foundations-"))
        .map(|document| document.concept_id.as_str())
        .collect::<Vec<_>>();
    let expected_document_ids = BTreeSet::from([
        "math-foundations-index",
        "math-foundations-linear-spaces-and-maps",
        "math-foundations-orthogonality-spectra-and-decompositions",
        "math-foundations-probability-and-gaussian-models",
        "math-foundations-bayesian-inference-and-information",
        "math-foundations-linear-models-and-regularization",
        "math-foundations-optimization-and-iterative-methods",
        "math-foundations-curriculum-map",
        "math-foundations-glossary",
        "math-foundations-source-registry",
        "math-foundations-claim-evidence-ledger",
        "math-foundations-formalization-map",
    ]);
    let auxiliary_document_ids = AUXILIARY_DOCUMENTS
        .iter()
        .filter(|(_, path)| path.starts_with("knowledge/mathematical_foundations/"))
        .map(|(document_id, _)| *document_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        auxiliary_document_ids.len(),
        12,
        "Mathematical Foundations must register exactly twelve auxiliary documents"
    );
    assert_eq!(&auxiliary_document_ids, &expected_document_ids);
    let actual_document_ids = document_ids.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(
        document_ids.len(),
        12,
        "duplicate or unexpected packet document ID"
    );
    assert_eq!(actual_document_ids, expected_document_ids);
}

#[test]
fn compiles_the_autodiff_geometry_route_and_auxiliary_documents() {
    let repo = fixture();
    write_complete_fixture(repo.path());

    let corpus = compile(repo.path()).unwrap();

    assert!(corpus.reader_routes.iter().any(|route| {
        route.route_id == "autodiff-geometry"
            && route.label == "Autodiff geometry"
            && route.canonical_markdown_path
                == "knowledge/autodiff_geometry/autodiff_geometry_index.md"
    }));

    let expected_document_ids = BTreeSet::from([
        "autodiff-geometry-index",
        "autodiff-geometry-source-registry",
        "autodiff-geometry-claim-evidence-ledger",
        "autodiff-geometry-formalization-roadmap",
        "autodiff-geometry-source-acquisition-and-parsing",
        "autodiff-geometry-layered-curriculum-tracker",
    ]);
    let auxiliary_document_ids = AUXILIARY_DOCUMENTS
        .iter()
        .filter(|(_, path)| path.starts_with("knowledge/autodiff_geometry/"))
        .map(|(document_id, _)| *document_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        auxiliary_document_ids.len(),
        6,
        "Autodiff Geometry must register exactly six auxiliary documents"
    );
    assert_eq!(&auxiliary_document_ids, &expected_document_ids);

    let actual_document_ids = corpus
        .documents
        .iter()
        .filter(|document| document.concept_id.starts_with("autodiff-geometry-"))
        .map(|document| document.concept_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_document_ids, expected_document_ids);
}

#[test]
fn compiles_the_mathematical_foundations_formalization_map() {
    const MAP_ID: &str = "math-foundations-formalization-map";
    const MAP_PATH: &str = "knowledge/mathematical_foundations/formalization_map.md";
    const MANIFEST_PATH: &str = "formalization/lean/MathematicalFoundations/PublicTheorems.lean";
    const STATUSES: [&str; 3] = ["Direct theorem", "Corollary/application", "Prose-only"];
    const INVENTORY_NAMESPACES: [&str; 6] = [
        "MathematicalFoundations.Linear",
        "MathematicalFoundations.Orthogonality",
        "MathematicalFoundations.Probability",
        "MathematicalFoundations.BayesInformation",
        "MathematicalFoundations.LinearModels",
        "MathematicalFoundations.Optimization",
    ];

    assert!(
        AUXILIARY_DOCUMENTS
            .iter()
            .any(|(document_id, path)| *document_id == MAP_ID && *path == MAP_PATH),
        "formalization map must be registered as an auxiliary document"
    );
    assert!(
        !READER_ROUTES.iter().any(|(_, _, path)| *path == MAP_PATH),
        "formalization map must not add a reader route"
    );

    let map = fs::read_to_string(workspace_root().join(MAP_PATH))
        .expect("formalization map must be canonical Markdown");
    let manifest = fs::read_to_string(workspace_root().join(MANIFEST_PATH))
        .expect("formalization map must have a Lean-owned public-theorem manifest");
    let root_module = fs::read_to_string(
        workspace_root().join("formalization/lean/MathematicalFoundations.lean"),
    )
    .expect("formalization root module must be readable");
    assert!(
        root_module.contains("import MathematicalFoundations.PublicTheorems"),
        "formalization root module must compile the public-theorem manifest"
    );
    let manifest_entries = manifest
        .split_once("def publicTheoremManifest : List String := [")
        .and_then(|(_, values)| values.split_once("]\n"))
        .expect("public theorem manifest must be a machine-readable Lean string list")
        .0
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',');
            line.strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let declared_theorems = manifest_entries.iter().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        manifest_entries.len(),
        23,
        "Lean manifest must declare exactly twenty-three public theorems"
    );
    assert_eq!(
        declared_theorems.len(),
        23,
        "Lean manifest must not duplicate a public theorem"
    );
    for identifier in &manifest_entries {
        assert!(
            manifest.contains(&format!("#check {identifier}")),
            "Lean manifest must typecheck listed theorem {identifier}"
        );
    }
    assert!(
        declared_theorems.iter().all(|identifier| {
            INVENTORY_NAMESPACES
                .iter()
                .any(|namespace| identifier.starts_with(&format!("{namespace}.")))
        }),
        "Lean manifest contains a theorem outside the six supported namespaces"
    );

    let inventory = map
        .split_once("## Compiled declaration inventory\n")
        .and_then(|(_, inventory)| inventory.split_once("\n## Module 1"))
        .map(|(inventory, _)| inventory)
        .expect("formalization map must have a declaration inventory before Module 1");
    let inventory_rows = inventory
        .lines()
        .filter_map(|line| {
            let cells = line
                .trim()
                .strip_prefix('|')?
                .strip_suffix('|')?
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>();
            (cells.len() == 2 && cells[0].starts_with("[Module ")).then_some(cells)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        inventory_rows.len(),
        INVENTORY_NAMESPACES.len(),
        "formalization inventory must have one row per supported module"
    );

    let mut inventory_module_numbers = Vec::new();
    let mut inventory_identifiers = Vec::new();
    for cells in inventory_rows {
        let module_number = cells[0]
            .strip_prefix("[Module ")
            .and_then(|label| label.split_once(':'))
            .and_then(|(number, _)| number.parse::<usize>().ok())
            .expect("formalization inventory module label must contain a number");
        let expected_namespace = INVENTORY_NAMESPACES
            .get(
                module_number
                    .checked_sub(1)
                    .expect("module number must be positive"),
            )
            .expect("formalization inventory module number must be supported");
        let identifiers = cells[1]
            .split('`')
            .filter(|identifier| identifier.starts_with("MathematicalFoundations."))
            .collect::<Vec<_>>();
        assert!(
            !identifiers.is_empty(),
            "formalization inventory module {module_number} must name a theorem"
        );
        assert!(
            identifiers
                .iter()
                .all(|identifier| identifier.starts_with(&format!("{expected_namespace}."))),
            "formalization inventory module {module_number} has a theorem from another namespace"
        );
        inventory_module_numbers.push(module_number);
        inventory_identifiers.extend(identifiers.into_iter().map(str::to_owned));
    }
    assert_eq!(
        inventory_module_numbers,
        (1..=6).collect::<Vec<_>>(),
        "formalization inventory modules must be ordered"
    );
    let inventory_set = inventory_identifiers
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        inventory_identifiers.len(),
        23,
        "formalization inventory must name exactly twenty-three theorem identifiers"
    );
    assert_eq!(
        inventory_set.len(),
        23,
        "formalization inventory must not duplicate a theorem identifier"
    );
    assert_eq!(
        inventory_set, declared_theorems,
        "formalization inventory must exactly match the compiled Lean manifest"
    );
    assert_eq!(
        inventory_identifiers, manifest_entries,
        "formalization inventory must preserve the exact Lean manifest ordering"
    );

    assert!(
        !contains_nonportable_path_reference(&map),
        "{MAP_PATH} must not contain a nonportable local-path reference"
    );
    let formalization_root = workspace_root().join("formalization/lean/MathematicalFoundations");
    let lean_sources = formalization_lean_sources(&formalization_root);
    assert!(
        lean_sources
            .iter()
            .any(|path| path.ends_with("PublicTheorems.lean")),
        "formalization source scan must discover the Lean-owned theorem manifest"
    );
    for path in lean_sources {
        let source = fs::read_to_string(&path).expect("formalization source must be readable");
        assert!(
            !contains_nonportable_path_reference(&source),
            "{} must not contain a nonportable local-path reference",
            path.strip_prefix(workspace_root()).unwrap().display()
        );
    }

    let rows = map
        .lines()
        .filter_map(|line| {
            let cells = line
                .trim()
                .strip_prefix('|')?
                .strip_suffix('|')?
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>();
            (cells.len() == 4 && cells[0].starts_with("MF-")).then_some(cells)
        })
        .collect::<Vec<_>>();

    let expected_ids = (1..=6)
        .flat_map(|module| (1..=8).map(move |problem| format!("MF-{module:02}-{problem:02}")))
        .collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        expected_ids.len(),
        "map must have exactly 48 problem rows"
    );

    let actual_ids = rows
        .iter()
        .map(|cells| cells[0].to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        actual_ids, expected_ids,
        "map rows must list each original problem once in module/problem order"
    );

    for cells in rows {
        let statuses = STATUSES
            .iter()
            .filter(|status| cells[1] == **status)
            .count();
        assert_eq!(
            statuses, 1,
            "{} must have exactly one supported status",
            cells[0]
        );
        let lean_identifiers = cells[2]
            .split('`')
            .filter(|identifier| identifier.starts_with("MathematicalFoundations."))
            .collect::<Vec<_>>();
        let has_lean = cells[2].starts_with("Lean: `") && !lean_identifiers.is_empty();
        match cells[1] {
            "Direct theorem" | "Corollary/application" => {
                assert!(has_lean, "{} requires a compiled Lean identifier", cells[0]);
                for identifier in lean_identifiers {
                    assert!(
                        declared_theorems.contains(identifier),
                        "{} names no compiled public Lean theorem: {identifier}",
                        cells[0]
                    );
                }
            }
            "Prose-only" => {
                assert!(
                    !cells[2].contains("Lean:") && lean_identifiers.is_empty(),
                    "{} must not claim a Lean identifier",
                    cells[0]
                );
                assert!(
                    cells[3].starts_with("Exact limitation: ") && cells[3] != "Exact limitation: ",
                    "{} needs an exact prose-only limitation",
                    cells[0]
                );
            }
            _ => unreachable!("status count made this impossible"),
        }
    }

    for declaration in declared_theorems {
        assert!(
            map.contains(&format!("`{declaration}`")),
            "formalization map must inventory compiled public theorem {declaration}"
        );
    }

    let corpus = compile(workspace_root()).expect("formalization map must compile in the corpus");
    let map_document = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == MAP_ID)
        .expect("formalization map must compile as a document");
    assert!(
        map_document.html.contains("class=\"math math-inline\""),
        "formalization map must preserve its mathematical notation through the renderer"
    );
}

#[test]
fn verified_coevolution_packet_is_registered_and_keeps_claims_conditional() {
    let corpus = compile(workspace_root()).unwrap();

    assert!(corpus.reader_routes.iter().any(|route| {
        route.route_id == "verified-coevolution"
            && route.label == "Verified coevolution"
            && route.canonical_markdown_path
                == "knowledge/verified_coevolution_agenda/verified_coevolution_agenda.md"
    }));

    let document = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "verified-coevolution")
        .unwrap();
    assert_eq!(
        document.canonical_markdown_path,
        "knowledge/verified_coevolution_agenda/verified_coevolution_agenda.md"
    );
    for required in [
        "recursive closure",
        "model-harness coevolution",
        "research hypothesis",
        "LADDER-TTRL",
        "PRIME-RL TTRL",
        "VCA-",
        "Recursive dynamics: epistemic drift, behavioral regression, and objective instability",
        "NSRSA",
        "SAHOO",
        "Scrivens",
        "conditional theory",
    ] {
        assert!(
            document.html.contains(required),
            "verified coevolution agenda is missing {required}"
        );
    }

    for document_id in [
        "verified-coevolution-source-registry",
        "verified-coevolution-claim-evidence-ledger",
        "verified-coevolution-experiment-protocol",
        "verified-coevolution-maintenance",
    ] {
        assert!(
            corpus
                .documents
                .iter()
                .any(|document| document.concept_id == document_id),
            "supporting packet document {document_id} is not compiled"
        );
    }
}

#[test]
fn current_fixture_payload_is_byte_stable() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    let first = serde_json::to_vec_pretty(&compile(repo.path()).unwrap()).unwrap();
    let second = serde_json::to_vec_pretty(&compile(repo.path()).unwrap()).unwrap();

    assert_eq!(first, second);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&first).unwrap()["schema_version"],
        "rsi-technical-atlas/v5",
    );
}

#[test]
fn rejects_markdown_beneath_the_structured_content_root() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    fs::write(repo.path().join("content/forbidden.md"), "# Forbidden\n").unwrap();

    let error = compile(repo.path()).unwrap_err();

    assert_eq!(error.code(), "knowledge.rsi.content_markdown");
    assert!(error.message.contains("content/forbidden.md"));
}

#[cfg(unix)]
#[test]
fn rejects_symlinks_while_scanning_the_structured_content_root() {
    use std::os::unix::fs::symlink;

    let repo = fixture();
    write_complete_fixture(repo.path());
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("forbidden.md"), "# Forbidden\n").unwrap();
    symlink(outside.path(), repo.path().join("content/linked")).unwrap();

    let error = compile(repo.path()).unwrap_err();

    assert_eq!(error.code(), "fs.symlink");
}

#[test]
fn rejects_a_retained_concept_omitted_from_the_coverage_map() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    let roster = repo.path().join("content/retained-concepts.tsv");
    let mut retained = fs::read_to_string(&roster).unwrap();
    retained.push_str("unmapped-retained-concept\t\n");
    fs::write(roster, retained).unwrap();

    let error = compile(repo.path()).unwrap_err();

    assert_eq!(error.code(), "knowledge.rsi.retained_coverage");
}

#[test]
fn rejects_a_supporting_page_without_a_concept_section() {
    let error = parse_coverage_row(
            2,
            "task-improvement	supporting-page	knowledge/rsi/chapters/recursive-improvement-loop.md		recursive-improvement-loop",
        )
        .unwrap_err();

    assert_eq!(error.code(), "knowledge.rsi.coverage_section");
}

#[test]
fn rejects_a_malformed_original_source_fold() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    let chapter = repo
        .path()
        .join("knowledge/rsi/chapters/harness-engineering.md");
    let source = fs::read_to_string(&chapter).unwrap();
    fs::write(&chapter, source.replace("</details>", "")).unwrap();

    let error = compile(repo.path()).unwrap_err();

    assert_eq!(error.code(), "knowledge.rsi.source_fold");
}

#[test]
fn preserves_native_details_in_rendered_html() {
    let rendered = render_markdown(
        &format!(
            "# Test\n\n## Mechanism\n\nText.\n\n<details>\n{SOURCE_SUMMARY}\n\n- Source.\n\n</details>\n"
        ),
        "knowledge/rsi/chapters/test.md",
        &[],
    );

    assert!(rendered.contains("<details>"));
    assert!(rendered.contains(SOURCE_SUMMARY));
    assert!(rendered.contains("</details>"));
}

#[test]
fn renders_obsidian_wikilinks_as_offline_routes_without_rewriting_code() {
    let targets = BTreeMap::from([(
        "knowledge/rsi/target.md".to_owned(),
        render::RouteTarget::Document {
            document_id: "target".to_owned(),
            heading_ids: BTreeMap::from([(
                "Result boundary".to_owned(),
                "result-boundary".to_owned(),
            )]),
        },
    )]);
    let rendered = render::render_markdown_with_targets(
        "[[knowledge/rsi/target#Result boundary|Result]]\n\n`[[knowledge/rsi/target]]`\n",
        "knowledge/rsi/source.md",
        &targets,
    );

    assert!(rendered.contains("href=\"#documents/target?section=result-boundary\""));
    assert!(rendered.contains(">Result</a>"));
    assert!(rendered.contains("[[knowledge/rsi/target]]"));
}

#[test]
fn renders_obsidian_callouts_as_escaped_semantic_html() {
    let rendered = render_markdown(
        "> [!warning] <Unsafe & title>\n> Body with **emphasis**.\n",
        "knowledge/rsi/chapters/test.md",
        &[],
    );

    assert!(rendered.contains(
        "<aside class=\"obsidian-callout\" data-callout-type=\"warning\" role=\"note\">"
    ));
    assert!(rendered.contains("<p class=\"obsidian-callout-title\">&lt;Unsafe &amp; title&gt;</p>"));
    assert!(rendered.contains("<p>Body with <strong>emphasis</strong>.</p>"));
    assert!(rendered.contains("</aside>"));
}

#[test]
fn leaves_obsidian_callout_syntax_literal_inside_code() {
    let rendered = render_markdown(
        "```md\n> [!tip] Fence literal\n```\n\n> ~~~md\n> [!tip] Blockquote fence literal\n> ~~~\n\n`> [!tip] Inline literal`\n",
        "knowledge/rsi/chapters/test.md",
        &[],
    );

    assert!(!rendered.contains("obsidian-callout"));
    assert!(rendered.contains("&gt; [!tip] Fence literal"));
    assert!(rendered.contains("[!tip] Blockquote fence literal"));
    assert!(rendered.contains("&gt; [!tip] Inline literal"));
}

#[test]
fn renders_same_note_obsidian_heading_links_to_the_source_document_route() {
    let targets = BTreeMap::from([(
        "knowledge/crouzeix_conjecture/source.md".to_owned(),
        render::RouteTarget::Document {
            document_id: "crouzeix-source".to_owned(),
            heading_ids: BTreeMap::from([(
                "Visible heading".to_owned(),
                "explicit-heading".to_owned(),
            )]),
        },
    )]);
    let rendered = render::render_markdown_with_targets(
        "[[#Visible heading|Jump]]",
        "knowledge/crouzeix_conjecture/source.md",
        &targets,
    );

    assert!(rendered.contains("href=\"#documents/crouzeix-source?section=explicit-heading\""));
    assert!(rendered.contains(">Jump</a>"));
}

#[test]
fn renders_pdf_embeds_as_accessible_fallback_links() {
    let rendered = render_markdown(
        "![[evidence/example/paper.pdf#page=2|Paper]]",
        "knowledge/rsi/chapters/test.md",
        &[],
    );

    assert!(rendered.contains(
        "<a class=\"obsidian-embed-fallback\" data-obsidian-embed=\"true\" href=\"../../evidence/example/paper.pdf#page=2\">Paper</a>"
    ));
}

#[test]
fn renders_source_relative_obsidian_wikilinks_as_offline_routes() {
    let targets = BTreeMap::from([(
        "knowledge/rsi/target.md".to_owned(),
        render::RouteTarget::Document {
            document_id: "target".to_owned(),
            heading_ids: BTreeMap::new(),
        },
    )]);
    let rendered = render::render_markdown_with_targets(
        "[[target|Target]]",
        "knowledge/rsi/source.md",
        &targets,
    );

    assert!(rendered.contains("href=\"#documents/target\""));
    assert!(rendered.contains(">Target</a>"));
}

#[test]
fn wraps_tables_in_a_keyboard_accessible_scroll_region() {
    let rendered = render_markdown(
        "# Test\n\n| Field | Value |\n|---|---|\n| state | durable |\n",
        "knowledge/rsi/chapters/test.md",
        &[],
    );

    assert!(rendered.contains(
        "<div class=\"canonical-table-scroll\" role=\"region\" aria-label=\"Scrollable data table\" tabindex=\"0\"><table>"
    ));
    assert!(rendered.contains("</table></div>"));
}

#[test]
fn reader_route_targets_precede_document_routes_and_fragments_survive() {
    let targets = render::route_targets(&[], &BTreeMap::new());
    assert_eq!(
        targets.get("knowledge/rsi/source_registry.md"),
        Some(&render::RouteTarget::Reader {
            route_id: "sources",
        })
    );

    let mut fragment_targets = BTreeMap::new();
    fragment_targets.insert(
        "knowledge/example/ledger.md".to_owned(),
        render::RouteTarget::Document {
            document_id: "example-ledger".to_owned(),
            heading_ids: BTreeMap::from([(
                "cc-013-origin-sample-cancels-the-correction".to_owned(),
                "cc-013-origin-sample-cancels-the-correction".to_owned(),
            )]),
        },
    );
    assert_eq!(
        render::offline_link_destination_with_targets(
            "ledger.md#cc-013-origin-sample-cancels-the-correction",
            Path::new("knowledge/example"),
            &fragment_targets,
        ),
        "#documents/example-ledger?section=cc-013-origin-sample-cancels-the-correction"
    );
    assert_eq!(
        render::offline_link_destination_with_targets(
            "ledger.md#absent",
            Path::new("knowledge/example"),
            &fragment_targets,
        ),
        "#documents/example-ledger"
    );

    let reader_target = BTreeMap::from([(
        "knowledge/rsi/source_registry.md".to_owned(),
        render::RouteTarget::Reader {
            route_id: "sources",
        },
    )]);
    assert_eq!(
        render::offline_link_destination_with_targets(
            "source_registry.md#registry-format",
            Path::new("knowledge/rsi"),
            &reader_target,
        ),
        "#sources"
    );

    let chapter_target = BTreeMap::from([(
        "knowledge/rsi/chapters/target.md".to_owned(),
        render::RouteTarget::Chapter {
            concept_id: "target".to_owned(),
            document_id: "target".to_owned(),
            heading_ids: BTreeMap::from([("mechanism".to_owned(), "mechanism".to_owned())]),
        },
    )]);
    assert_eq!(
        render::offline_link_destination_with_targets(
            "target.md#mechanism",
            Path::new("knowledge/rsi/chapters"),
            &chapter_target,
        ),
        "#documents/target?section=mechanism"
    );
    assert_eq!(
        render::offline_link_destination_with_targets(
            "target.md",
            Path::new("knowledge/rsi/chapters"),
            &chapter_target,
        ),
        "#chapters/target"
    );

    let mut legacy_sources = BTreeMap::new();
    legacy_sources.insert(
        "knowledge/rsi/systems/legacy.md".to_owned(),
        contracts::ValidatedCanonicalSource {
            path: "knowledge/rsi/systems/legacy.md".to_owned(),
            markdown: "# Legacy\n\n## Generated slug\n".to_owned(),
            body_sha256: sha256(b"# Legacy\n\n## Generated slug\n"),
            entries: Vec::new(),
        },
    );
    let legacy_targets = render::route_targets(&[], &legacy_sources);
    assert_eq!(
        render::offline_link_destination_with_targets(
            "legacy.md#generated-slug",
            Path::new("knowledge/rsi/systems"),
            &legacy_targets,
        ),
        "#documents/legacy?section=generated-slug"
    );
}

#[test]
fn math_and_heading_attributes_are_scoped_to_mathematics_packets() {
    let packet = render_markdown(
        "## Bound {#bound}\n\n$\\|T\\|\\le2$",
        "knowledge/crouzeix_conjecture/example.md",
        &[],
    );
    assert!(packet.contains("id=\"bound\""));
    assert!(packet.contains("class=\"math math-inline\""));
    assert!(packet.contains("data-tex=\""));

    let foundations = render_markdown(
        "## Gradient {#gradient}\n\n$\\nabla f(x)$",
        "knowledge/mathematical_foundations/example.md",
        &[],
    );
    assert!(foundations.contains("id=\"gradient\""));
    assert!(foundations.contains("class=\"math math-inline\""));

    let legacy = render_markdown(
        "Revenue moved from $0.0291 to $0.6371.",
        "knowledge/rsi/systems/aflow.md",
        &[],
    );
    assert!(!legacy.contains("math-inline"));
}

#[test]
fn mathematics_packet_heading_ids_are_unique_and_math_payloads_are_escaped() {
    let error = render::heading_ids_for_source(
        "## First {#duplicate}\n\n## Second {#duplicate}\n",
        "knowledge/crouzeix_conjecture/example.md",
    )
    .unwrap_err();
    assert_eq!(error.code(), "knowledge.rsi.heading_id");

    let foundations_error = render::heading_ids_for_source(
        "## First {#duplicate}\n\n## Second {#duplicate}\n",
        "knowledge/mathematical_foundations/example.md",
    )
    .unwrap_err();
    assert_eq!(foundations_error.code(), "knowledge.rsi.heading_id");

    let rendered = render_markdown("$x<&>\"'$", "knowledge/crouzeix_conjecture/example.md", &[]);
    assert!(rendered.contains("data-tex=\"x&lt;&amp;&gt;&quot;&#39;\""));
    assert!(rendered.contains(">x&lt;&amp;&gt;\"'</span>"));
}

#[test]
fn rewrites_offline_links_to_chapter_routes_and_static_export_repository_files() {
    let coverage = vec![CoverageEntry {
        concept_id: "target".into(),
        coverage_depth: CoverageDepth::Chapter,
        canonical_markdown_path: "knowledge/rsi/chapters/target.md".into(),
        section_id: None,
        parent_concept_id: None,
    }];

    assert_eq!(
        offline_link_destination("target.md", Path::new("knowledge/rsi/chapters"), &coverage,),
        "#chapters/target"
    );
    assert_eq!(
        offline_link_destination(
            "../../../../../evidence/sicp/sicp.pdf#page=12",
            Path::new("knowledge/rsi/sicp/course/capstone"),
            &coverage,
        ),
        "../../evidence/sicp/sicp.pdf#page=12"
    );
    assert_eq!(
        offline_link_destination(
            "../../../../../labs/sicp-evaluator/",
            Path::new("knowledge/rsi/sicp/course/capstone"),
            &coverage,
        ),
        "../../labs/sicp-evaluator"
    );
    assert_eq!(
        offline_link_destination(
            "../../../../../crates/harp/src/sources.rs",
            Path::new("knowledge/rsi/sicp/course/capstone"),
            &coverage,
        ),
        "../../crates/harp/src/sources.rs"
    );
}

#[test]
fn lesson_links_route_to_owned_system_documents() {
    let mut sources = BTreeMap::new();
    sources.insert(
        "knowledge/rsi/systems/aflow.md".to_owned(),
        contracts::ValidatedCanonicalSource {
            path: "knowledge/rsi/systems/aflow.md".to_owned(),
            markdown: "# AFlow\n".to_owned(),
            body_sha256: sha256(b"# AFlow\n"),
            entries: vec![CoverageEntry {
                concept_id: "aflow".to_owned(),
                coverage_depth: CoverageDepth::SystemReading,
                canonical_markdown_path: "knowledge/rsi/systems/aflow.md".to_owned(),
                section_id: None,
                parent_concept_id: Some("harness-search".to_owned()),
            }],
        },
    );

    assert_eq!(
        render::offline_link_destination_with_sources(
            "../systems/aflow.md",
            Path::new("knowledge/rsi/lessons"),
            &[],
            &sources,
        ),
        "#documents/aflow"
    );
}

#[test]
fn resolves_parent_links_without_allowing_repository_escape() {
    assert_eq!(
        normalize_link_path(
            Path::new("knowledge/rsi/chapters"),
            Path::new("../source_registry.md")
        ),
        Some(PathBuf::from("knowledge/rsi/source_registry.md"))
    );
    assert_eq!(
        normalize_link_path(Path::new("content"), Path::new("../../../secret")),
        None
    );
    assert_eq!(
        normalize_link_path(
            Path::new("content"),
            Path::new("../../evidence/sicp/sicp.pdf")
        ),
        None
    );
    assert_eq!(
        normalize_link_path(
            Path::new("knowledge/rsi/systems"),
            Path::new("../../meta_harness/meta_harness_deep_dive.md"),
        ),
        Some(PathBuf::from(
            "knowledge/meta_harness/meta_harness_deep_dive.md"
        ))
    );
}

#[test]
fn validates_local_links_in_product_roots() {
    let repo = fixture();
    let evidence = repo.path().join("evidence/sicp/sicp.pdf");
    fs::create_dir_all(evidence.parent().unwrap()).unwrap();
    fs::write(evidence, b"captured source").unwrap();
    fs::create_dir_all(repo.path().join("labs/sicp-evaluator")).unwrap();
    let source = repo.path().join("crates/harp/src/sources.rs");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(source, b"// materializer\n").unwrap();
    let repository = HeldDirectory::open(repo.path(), "test repository").unwrap();

    validate_local_links(
        "knowledge/rsi/sicp/course/capstone/guide.md",
        "[source](../../../../../evidence/sicp/sicp.pdf)\n[lab](../../../../../labs/sicp-evaluator/)\n[materializer](../../../../../crates/harp/src/sources.rs)\n",
        &repository,
    )
    .unwrap();
}

#[test]
fn validates_obsidian_wikilinks_in_product_roots() {
    let repo = fixture();
    let target = repo.path().join("knowledge/rsi/target.md");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(target, "# Target\n\n## Result boundary\n\nText.\n").unwrap();
    let repository = HeldDirectory::open(repo.path(), "test repository").unwrap();

    validate_local_links(
        "knowledge/rsi/chapters/guide.md",
        "[[knowledge/rsi/target#Result boundary|Result]]",
        &repository,
    )
    .unwrap();

    let error = validate_local_links(
        "knowledge/rsi/chapters/guide.md",
        "[[knowledge/rsi/missing|Missing]]",
        &repository,
    )
    .unwrap_err();
    assert_eq!(error.code(), "knowledge.obsidian.target");
}

#[test]
fn registry_coverage_rejects_unmapped_identity_only_sources() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    let registry = repo.path().join("content/sources/source_registry.tsv");
    fs::create_dir_all(registry.parent().unwrap()).unwrap();
    fs::write(
        registry,
        "source_id\taccess_status\nIDENTITY-ONLY\tvendored-uninspected\n",
    )
    .unwrap();

    let error = compile(repo.path()).unwrap_err();

    assert_eq!(error.code(), "knowledge.rsi.registry_coverage");
}

#[test]
fn rejects_a_missing_authoritative_source_registry() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    fs::remove_file(repo.path().join(SOURCE_REGISTRY_PATH)).unwrap();

    let error = compile(repo.path()).unwrap_err();

    assert_eq!(error.code(), "knowledge.rsi.source_registry");
}

#[test]
fn validates_all_numbered_weng_references_and_locator_suffixes() {
    let repo = fixture();
    let graph = repo.path().join(EVIDENCE_GRAPH_PATH);
    fs::create_dir_all(graph.parent().unwrap()).unwrap();
    let mut text = format!("{EVIDENCE_GRAPH_HEADER}\n");
    for number in 1..=39 {
        let locator = if number == 38 {
            "Reference 38 and Joint Optimization with Model Weights".to_owned()
        } else {
            format!("Reference {number}")
        };
        text.push_str(&format!(
                "EVIDENCE\tWENG-HARNESS\tcites\tSOURCE-{number}\t{locator}\tverified\tIdentity edge only\n"
            ));
    }
    fs::write(&graph, &text).unwrap();
    let registered = std::iter::once("WENG-HARNESS".to_owned())
        .chain((1..=39).map(|number| format!("SOURCE-{number}")))
        .collect::<BTreeSet<_>>();
    let repository = HeldDirectory::open(repo.path(), "RSI test repository").unwrap();

    validate_evidence_graph(&repository, &registered).unwrap();

    fs::write(
        graph,
        text.lines()
            .filter(|line| !line.contains("\tSOURCE-38\t"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )
    .unwrap();
    let error = validate_evidence_graph(&repository, &registered).unwrap_err();
    assert_eq!(error.code(), "knowledge.rsi.weng_references");
}

#[test]
fn rejects_each_declared_content_integrity_failure() {
    struct Case {
        name: &'static str,
        mutate: fn(&Path),
        code: &'static str,
    }

    fn duplicate_owner(repo: &Path) {
        let path = repo.join(COVERAGE_PATH);
        let mut coverage = fs::read_to_string(&path).unwrap();
        coverage.push_str(
            "harness-engineering	chapter	knowledge/rsi/chapters/harness-engineering.md		\n",
        );
        fs::write(path, coverage).unwrap();
    }
    fn missing_chapter(repo: &Path) {
        fs::remove_file(repo.join("knowledge/rsi/chapters/harness-engineering.md")).unwrap();
    }
    fn absent_worked_example(repo: &Path) {
        let path = repo.join(COVERAGE_PATH);
        let mut coverage = fs::read_to_string(&path).unwrap();
        coverage.push_str(
                "example	worked-example	knowledge/rsi/chapters/harness-engineering.md	absent	harness-engineering\n",
            );
        fs::write(path, coverage).unwrap();
        let roster = repo.join(RETAINED_CONCEPTS_PATH);
        let mut retained = fs::read_to_string(&roster).unwrap();
        retained.push_str("example\t\n");
        fs::write(roster, retained).unwrap();
    }
    fn broken_local_link(repo: &Path) {
        let path = repo.join("knowledge/rsi/chapters/harness-engineering.md");
        let mut markdown = fs::read_to_string(&path).unwrap();
        markdown.push_str("\n[Missing](../concepts/absent.md)\n");
        fs::write(path, markdown).unwrap();
    }
    fn malformed_fold(repo: &Path) {
        let path = repo.join("knowledge/rsi/chapters/harness-engineering.md");
        let markdown = fs::read_to_string(&path).unwrap().replace("</details>", "");
        fs::write(path, markdown).unwrap();
    }
    fn metadata_first(repo: &Path) {
        let path = repo.join("knowledge/rsi/chapters/harness-engineering.md");
        let markdown = fs::read_to_string(&path)
            .unwrap()
            .replace("## Technical mechanism", "## Source registry");
        fs::write(path, markdown).unwrap();
    }
    fn registry_as_chapter(repo: &Path) {
        let path = repo.join(COVERAGE_PATH);
        let coverage = fs::read_to_string(&path).unwrap().replace(
            "knowledge/rsi/chapters/harness-engineering.md",
            "knowledge/rsi/source_registry.md",
        );
        fs::write(path, coverage).unwrap();
    }

    for case in [
        Case {
            name: "duplicate primary home",
            mutate: duplicate_owner,
            code: "knowledge.rsi.coverage_owner",
        },
        Case {
            name: "missing chapter",
            mutate: missing_chapter,
            code: "knowledge.rsi.canonical_missing",
        },
        Case {
            name: "absent worked-example section",
            mutate: absent_worked_example,
            code: "knowledge.rsi.coverage_section",
        },
        Case {
            name: "broken local link",
            mutate: broken_local_link,
            code: "knowledge.rsi.local_link",
        },
        Case {
            name: "malformed source fold",
            mutate: malformed_fold,
            code: "knowledge.rsi.source_fold",
        },
        Case {
            name: "metadata before technical content",
            mutate: metadata_first,
            code: "knowledge.rsi.content_order",
        },
        Case {
            name: "registry rendered as chapter",
            mutate: registry_as_chapter,
            code: "knowledge.rsi.coverage_path",
        },
    ] {
        let repo = fixture();
        write_complete_fixture(repo.path());
        (case.mutate)(repo.path());

        let error = compile(repo.path()).unwrap_err();

        assert_eq!(error.code(), case.code, "{}", case.name);
    }
}

#[cfg(unix)]
#[test]
fn build_rejects_a_symlinked_output_without_changing_its_target() {
    use std::os::unix::fs::symlink;

    let repo = fixture();
    write_complete_fixture(repo.path());
    let victim = repo.path().join("victim.json");
    fs::write(&victim, b"do not overwrite\n").unwrap();
    let output = PathBuf::from("atlas/src/content/generated/corpus.json");
    symlink(&victim, repo.path().join(&output)).unwrap();

    let error =
        crate::build_corpus(repo.path(), Some(&output), crate::BuildMode::Write).unwrap_err();

    assert_eq!(error.code(), "fs.symlink");
    assert_eq!(fs::read(victim).unwrap(), b"do not overwrite\n");
}

#[cfg(unix)]
#[test]
fn build_rejects_a_symlinked_output_ancestor_without_writing_outside() {
    use std::os::unix::fs::symlink;

    let repo = fixture();
    write_complete_fixture(repo.path());
    let generated = repo.path().join("atlas/src/content/generated");
    fs::remove_dir(&generated).unwrap();
    let outside = tempfile::tempdir().unwrap();
    symlink(outside.path(), &generated).unwrap();
    let output = PathBuf::from("atlas/src/content/generated/corpus.json");

    let error =
        crate::build_corpus(repo.path(), Some(&output), crate::BuildMode::Write).unwrap_err();

    assert_eq!(error.code(), "fs.symlink");
    assert!(fs::read_dir(outside.path()).unwrap().next().is_none());
}

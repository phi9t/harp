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

fn fixture() -> TempDir {
    let target = workspace_root().join("target");
    fs::create_dir_all(&target).unwrap();
    let repo = tempfile::Builder::new()
        .prefix("rsi-")
        .tempdir_in(target)
        .unwrap();
    let root = repo.path().join("content");
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
    let lesson_path = "content/lessons/01-self-refine.md";
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
                "canonical_markdown_path": format!("content/systems/{system_id}.md"),
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
            let companion_path = format!("content/weng/{:02}-{section_id}.md", index + 1);
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
                .starts_with("content/weng/")
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
    let row = "aflow\tsystem-reading\tcontent/systems/aflow.md\t\tharness-search";
    let entry = parse_coverage_row(2, row).unwrap();

    assert_eq!(entry.coverage_depth, CoverageDepth::SystemReading);
    assert_eq!(entry.canonical_markdown_path, "content/systems/aflow.md");
    assert_eq!(entry.section_id, None);
}

#[test]
fn a_published_system_compiles_as_a_system_reading_document() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    write_complete_system_registry_fixture(repo.path());
    write_complete_weng_fixture(repo.path());

    let aflow_path = "content/systems/aflow.md";
    fs::create_dir_all(repo.path().join("content/systems")).unwrap();
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
            .join("content/codex_state_continuity_and_compaction.md"),
        "# Codex continuity\n\n## Mechanism\n\nState.\n",
    )
    .unwrap();
    fs::write(
        repo.path().join("content/context_engineering_deep_dive.md"),
        "# Context engineering\n\n## Mechanism\n\nContext.\n",
    )
    .unwrap();
    fs::create_dir_all(repo.path().join("content/sicp")).unwrap();
    fs::write(
        repo.path().join("content/sicp/agentic_eval_apply.md"),
        "# Agentic eval/apply\n\n## Mechanism\n\nEvaluate before applying.\n",
    )
    .unwrap();
    let harness = repo.path().join("content/chapters/harness-engineering.md");
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
        "content/codex_state_continuity_and_compaction.md"
    );
    assert_eq!(
        context_document.canonical_markdown_path,
        "content/context_engineering_deep_dive.md"
    );
    assert_eq!(
        agentic_eval_apply_document.canonical_markdown_path,
        "content/sicp/agentic_eval_apply.md"
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
            && route.canonical_markdown_path == "content/sicp/agentic_eval_apply.md"
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
fn rlm_system_reading_links_eval_apply_and_preserves_code_boundaries() {
    let corpus = compile(workspace_root()).unwrap();
    let document = corpus
        .documents
        .iter()
        .find(|document| document.concept_id == "rlm")
        .unwrap();

    for required in [
        "href=\"#documents/agentic-eval-apply\"",
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
            "evaluator-integrity"
        ]
    );
    for route in &corpus.reader_routes {
        assert!(corpus
            .documents
            .iter()
            .any(|document| { document.canonical_markdown_path == route.canonical_markdown_path }));
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
            "task-improvement	supporting-page	content/chapters/recursive-improvement-loop.md		recursive-improvement-loop",
        )
        .unwrap_err();

    assert_eq!(error.code(), "knowledge.rsi.coverage_section");
}

#[test]
fn rejects_a_malformed_original_source_fold() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    let chapter = repo.path().join("content/chapters/harness-engineering.md");
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
        "content/chapters/test.md",
        &[],
    );

    assert!(rendered.contains("<details>"));
    assert!(rendered.contains(SOURCE_SUMMARY));
    assert!(rendered.contains("</details>"));
}

#[test]
fn wraps_tables_in_a_keyboard_accessible_scroll_region() {
    let rendered = render_markdown(
        "# Test\n\n| Field | Value |\n|---|---|\n| state | durable |\n",
        "content/chapters/test.md",
        &[],
    );

    assert!(rendered.contains(
        "<div class=\"canonical-table-scroll\" role=\"region\" aria-label=\"Scrollable data table\" tabindex=\"0\"><table>"
    ));
    assert!(rendered.contains("</table></div>"));
}

#[test]
fn rewrites_offline_links_to_chapter_routes_and_static_export_repository_files() {
    let coverage = vec![CoverageEntry {
        concept_id: "target".into(),
        coverage_depth: CoverageDepth::Chapter,
        canonical_markdown_path: "content/chapters/target.md".into(),
        section_id: None,
        parent_concept_id: None,
    }];

    assert_eq!(
        offline_link_destination("target.md", Path::new("content/chapters"), &coverage,),
        "#chapters/target"
    );
    assert_eq!(
        offline_link_destination(
            "../../../../evidence/sicp/sicp.pdf#page=12",
            Path::new("content/sicp/course/capstone"),
            &coverage,
        ),
        "../../evidence/sicp/sicp.pdf#page=12"
    );
    assert_eq!(
        offline_link_destination(
            "../../../../labs/sicp-evaluator/",
            Path::new("content/sicp/course/capstone"),
            &coverage,
        ),
        "../../labs/sicp-evaluator"
    );
    assert_eq!(
        offline_link_destination(
            "../../../../crates/harp/src/sources.rs",
            Path::new("content/sicp/course/capstone"),
            &coverage,
        ),
        "../../crates/harp/src/sources.rs"
    );
}

#[test]
fn lesson_links_route_to_owned_system_documents() {
    let mut sources = BTreeMap::new();
    sources.insert(
        "content/systems/aflow.md".to_owned(),
        contracts::ValidatedCanonicalSource {
            path: "content/systems/aflow.md".to_owned(),
            markdown: "# AFlow\n".to_owned(),
            body_sha256: sha256(b"# AFlow\n"),
            entries: vec![CoverageEntry {
                concept_id: "aflow".to_owned(),
                coverage_depth: CoverageDepth::SystemReading,
                canonical_markdown_path: "content/systems/aflow.md".to_owned(),
                section_id: None,
                parent_concept_id: Some("harness-search".to_owned()),
            }],
        },
    );

    assert_eq!(
        render::offline_link_destination_with_sources(
            "../systems/aflow.md",
            Path::new("content/lessons"),
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
            Path::new("content/chapters"),
            Path::new("../source_registry.md")
        ),
        Some(PathBuf::from("content/source_registry.md"))
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
            Path::new("content/systems"),
            Path::new("../../knowledge/meta_harness/meta_harness_deep_dive.md"),
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
        "content/sicp/course/capstone/guide.md",
        "[source](../../../../evidence/sicp/sicp.pdf)\n[lab](../../../../labs/sicp-evaluator/)\n[materializer](../../../../crates/harp/src/sources.rs)\n",
        &repository,
    )
    .unwrap();
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
        coverage
            .push_str("harness-engineering	chapter	content/chapters/harness-engineering.md		\n");
        fs::write(path, coverage).unwrap();
    }
    fn missing_chapter(repo: &Path) {
        fs::remove_file(repo.join("content/chapters/harness-engineering.md")).unwrap();
    }
    fn absent_worked_example(repo: &Path) {
        let path = repo.join(COVERAGE_PATH);
        let mut coverage = fs::read_to_string(&path).unwrap();
        coverage.push_str(
                "example	worked-example	content/chapters/harness-engineering.md	absent	harness-engineering\n",
            );
        fs::write(path, coverage).unwrap();
        let roster = repo.join(RETAINED_CONCEPTS_PATH);
        let mut retained = fs::read_to_string(&roster).unwrap();
        retained.push_str("example\t\n");
        fs::write(roster, retained).unwrap();
    }
    fn broken_local_link(repo: &Path) {
        let path = repo.join("content/chapters/harness-engineering.md");
        let mut markdown = fs::read_to_string(&path).unwrap();
        markdown.push_str("\n[Missing](../concepts/absent.md)\n");
        fs::write(path, markdown).unwrap();
    }
    fn malformed_fold(repo: &Path) {
        let path = repo.join("content/chapters/harness-engineering.md");
        let markdown = fs::read_to_string(&path).unwrap().replace("</details>", "");
        fs::write(path, markdown).unwrap();
    }
    fn metadata_first(repo: &Path) {
        let path = repo.join("content/chapters/harness-engineering.md");
        let markdown = fs::read_to_string(&path)
            .unwrap()
            .replace("## Technical mechanism", "## Source registry");
        fs::write(path, markdown).unwrap();
    }
    fn registry_as_chapter(repo: &Path) {
        let path = repo.join(COVERAGE_PATH);
        let coverage = fs::read_to_string(&path).unwrap().replace(
            "content/chapters/harness-engineering.md",
            "content/source_registry.md",
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

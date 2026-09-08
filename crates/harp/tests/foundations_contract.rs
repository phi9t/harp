//! Scoped foundations acceptance; see docs/workstream/harp-mathematics/foundations-contract.md.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

#[test]
fn chapters_01_through_12_preserve_72_card_and_exercise_identities() {
    let coverage = contract("coverage.json");
    let exercises = contract("exercises.json");
    let cards = foundations_rows(&coverage, "items");
    let problems = foundations_rows(&exercises, "exercises");
    assert_eq!(cards.len(), 72);
    assert_eq!(problems.len(), 72);
    let card_ids = cards
        .iter()
        .map(|row| row["item_id"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    let problem_ids = problems
        .iter()
        .map(|row| row["exercise_id"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(card_ids.len(), 72, "duplicate public card ID");
    assert_eq!(problem_ids.len(), 72, "duplicate public exercise ID");
    for chapter in 1..=12 {
        let chapter_cards = cards
            .iter()
            .filter(|row| row["chapter"] == chapter)
            .collect::<Vec<_>>();
        let chapter_problems = problems
            .iter()
            .filter(|row| row["chapter"] == chapter)
            .collect::<Vec<_>>();
        assert_eq!(chapter_cards.len(), 6, "Chapter {chapter} card roster");
        assert_eq!(
            chapter_problems.len(),
            6,
            "Chapter {chapter} exercise roster"
        );
        let path = chapter_cards[0]["prose_path"].as_str().unwrap();
        let source = fs::read_to_string(workspace_root().join(path)).unwrap();
        for index in 1..=6 {
            let item_id = format!("CFT-{chapter:02}-{index:03}");
            let exercise_id = format!("CFT-{chapter:02}-E{index:02}");
            let card = chapter_cards
                .iter()
                .find(|row| row["item_id"] == item_id)
                .expect("retained CFT ID");
            let exercise = chapter_problems
                .iter()
                .find(|row| row["exercise_id"] == exercise_id)
                .expect("retained exercise ID");
            assert_eq!(card["prose_path"], path, "a card moved out of its chapter");
            for (row, anchor) in [
                (*card, item_id.to_lowercase()),
                (
                    *exercise,
                    format!("exercise-{}", exercise_id.to_lowercase()),
                ),
            ] {
                assert_eq!(row["anchor"], anchor);
                assert_eq!(
                    source.matches(&format!("{{#{anchor}}}")).count(),
                    1,
                    "{path}: anchor {anchor} must resolve exactly once"
                );
            }
        }
    }
}

#[test]
fn chapters_01_through_04_publish_exact_cards_with_explicit_proof_boundaries() {
    let coverage = contract("coverage.json");
    for row in foundations_rows(&coverage, "items")
        .into_iter()
        .filter(|row| row["chapter"].as_u64().unwrap() <= 4)
    {
        let id = row["item_id"].as_str().unwrap();
        assert_eq!(row["publication_status"], "active", "{id}");
        assert_eq!(row["lean_correspondence_status"], "exact", "{id}");
        let expected = match id {
            "CFT-01-001" | "CFT-03-005" => "not-applicable",
            "CFT-04-004" | "CFT-04-005" | "CFT-04-006" => "summary",
            // Chapter 1 may preview characteristic polynomials until their owning
            // foundations chapter supplies a prerequisite-safe proof.
            "CFT-01-005" if row["prose_proof_status"] == "summary" => "summary",
            _ => "reconstructible",
        };
        assert_eq!(row["prose_proof_status"], expected, "{id}");
        if expected == "not-applicable" {
            assert_eq!(row["kind"], "definition", "{id}");
        } else if id == "CFT-01-006" {
            assert_eq!(row["kind"], "worked-example", "{id}");
        } else {
            assert_eq!(row["kind"], "theorem", "{id}");
        }
        let source =
            fs::read_to_string(workspace_root().join(row["prose_path"].as_str().unwrap())).unwrap();
        let card = anchored_card(&source, row["anchor"].as_str().unwrap());
        let sections = labeled_sections(card);
        assert!(
            has_content(&sections, "statement") || has_content(&sections, "exact statement"),
            "{id}: missing explicit statement card"
        );
        if expected == "reconstructible" {
            assert!(
                has_content(&sections, "proof"),
                "{id}: missing proof exposition"
            );
        } else if expected == "summary" {
            assert!(
                card.to_lowercase().contains("forward reference")
                    || card.to_lowercase().contains("forward-reference")
                    || card.to_lowercase().contains("preview"),
                "{id}: summary must be visibly labeled as a preview"
            );
        }
    }
}

#[test]
fn chapters_01_through_04_have_24_distinct_solutions_not_public_card_aliases() {
    let coverage = contract("coverage.json");
    let exercises = contract("exercises.json");
    let card_types = coverage["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["lean_declaration"]["type_sha256"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    let mut solution_types = BTreeSet::new();
    for chapter in 1..=4 {
        for index in 1..=6 {
            let id = format!("CFT-{chapter:02}-E{index:02}");
            let row = exercises["exercises"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["exercise_id"] == id)
                .unwrap();
            let solution = &row["lean_solution"];
            let expected = format!("CrouzeixTextbook.Part01.Exercises.Chapter{chapter:02}.exercise_{index:02}_solution");
            assert_eq!(
                solution["declaration"], expected,
                "{id}: needs its own local solution"
            );
            let fingerprint = solution["type_sha256"]
                .as_str()
                .expect("registered solution type fingerprint");
            assert!(
                !card_types.contains(fingerprint),
                "{id}: solution merely restates a public card"
            );
            assert!(
                solution_types.insert(fingerprint),
                "{id}: solution repeats another exercise statement"
            );
        }
    }
    assert_eq!(solution_types.len(), 24);
}

#[test]
fn chapters_05_through_12_remain_pending_in_this_scoped_acceptance() {
    let coverage = contract("coverage.json");
    let exercises = contract("exercises.json");
    let pending_cards = foundations_rows(&coverage, "items")
        .into_iter()
        .filter(|row| row["chapter"].as_u64().unwrap() >= 5)
        .collect::<Vec<_>>();
    let pending_exercises = foundations_rows(&exercises, "exercises")
        .into_iter()
        .filter(|row| row["chapter"].as_u64().unwrap() >= 5)
        .collect::<Vec<_>>();
    assert_eq!(pending_cards.len(), 48);
    assert_eq!(pending_exercises.len(), 48);
    for row in pending_cards {
        assert_ne!(
            row["lean_correspondence_status"], "exact",
            "{}: a later approved wave must revise this scoped contract",
            row["item_id"]
        );
        assert_eq!(row["prose_proof_status"], "summary", "{}", row["item_id"]);
    }
    for row in pending_exercises {
        assert!(
            row["lean_solution"].is_null(),
            "{}: do not claim completion of the pending roster",
            row["exercise_id"]
        );
    }
}

#[test]
fn chapters_01_through_04_meet_the_labeled_editorial_contract() {
    let coverage = contract("coverage.json");
    let paths = foundations_rows(&coverage, "items")
        .into_iter()
        .filter(|row| row["chapter"].as_u64().unwrap() <= 4)
        .map(|row| workspace_root().join(row["prose_path"].as_str().unwrap()))
        .collect::<BTreeSet<_>>();
    let references = paths.iter().map(PathBuf::as_path).collect::<Vec<_>>();
    check_editorial(&references).unwrap_or_else(|errors| panic!("{}", errors.join("\n")));
    for path in paths {
        let source = fs::read_to_string(&path).unwrap();
        let compact = source.split_whitespace().collect::<String>();
        assert!(
            compact.contains(r"A_{\lambda,\alpha}"),
            "{}: missing cumulative matrix family",
            path.display()
        );
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn contract(name: &str) -> Value {
    serde_json::from_slice(
        &fs::read(
            workspace_root()
                .join("content/crouzeix_textbook")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}

fn foundations_rows<'a>(document: &'a Value, key: &str) -> Vec<&'a Value> {
    document[key]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| (1..=12).contains(&row["chapter"].as_u64().unwrap()))
        .collect()
}

fn anchored_card<'a>(source: &'a str, anchor: &str) -> &'a str {
    let (before, remainder) = source
        .split_once(&format!("{{#{anchor}}}"))
        .expect("card anchor");
    let level = heading_level(before.lines().last().expect("card heading"))
        .expect("CFT anchor belongs to a heading");
    let mut offset = 0;
    for line in remainder.split_inclusive('\n') {
        if heading_level(line).is_some_and(|next| next <= level) || line.contains("{#cft-") {
            return &remainder[..offset];
        }
        offset += line.len();
    }
    remainder
}

fn heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    let level = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    ((1..=6).contains(&level) && trimmed[level..].starts_with(' ')).then_some(level)
}

fn has_content(sections: &BTreeMap<String, String>, label: &str) -> bool {
    sections.iter().any(|(key, body)| {
        (key == label || key.ends_with(&format!(". {label}"))) && !body.trim().is_empty()
    })
}

const EDITORIAL_FIXTURE: &str = r#"# Chapter

## Motivation

A redundant linear predictor identifies distinct parameter vectors.

## Historical context

This is pedagogical context, not a historical priority claim.

## ML bridge

**Mathematical object.** A quotient of parameter space by a kernel.

**Exact transfer.** Two parameter vectors have equal linear outputs exactly
when their difference is in that kernel.

**Non-transfer.** A nonlinear predictor need not identify those same vectors.

**Calculation.** For T(x,y,z)=(x+z,y+z), T(-1,-1,1)=(0,0).

## Cumulative matrix calculation

For $A_{\lambda,\alpha}$, $A_{\lambda,\alpha}e_1=\lambda e_1$;
the line generated by $e_1$ is invariant.
"#;

#[test]
fn theorem_cards_accept_adjacent_bold_titles_and_require_the_statement_label() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("chapter.md");
    let source = "### Linear transformations before coordinates {#cft-01-001}\n\n**Definition CFT-01-001 (linear transformation).** **Statement.** A map between vector spaces over the same field is linear when it preserves addition and scalar multiplication.\n\n### Column rule {#cft-01-002}\n\n**Theorem CFT-01-002 (column rule).** **Statement.** Each matrix column gives the coordinates of the corresponding basis image.\n\n**Proof.** Evaluate the coordinate representation at each basis vector.\n";
    fs::write(&path, source).unwrap();
    let read = fs::read_to_string(&path).unwrap();
    for anchor in ["cft-01-001", "cft-01-002"] {
        let sections = labeled_sections(anchored_card(&read, anchor));
        assert!(
            has_content(&sections, "statement"),
            "{anchor}: adjacent bold statement label was lost"
        );
    }

    fs::write(&path, source.replace("**Statement.**", "")).unwrap();
    let changed = fs::read_to_string(&path).unwrap();
    for anchor in ["cft-01-001", "cft-01-002"] {
        let sections = labeled_sections(anchored_card(&changed, anchor));
        assert!(
            !has_content(&sections, "statement"),
            "{anchor}: a title or body is not a statement label"
        );
    }
    assert!(has_content(
        &labeled_sections(anchored_card(&changed, "cft-01-002")),
        "proof"
    ));
}

#[test]
fn theorem_cards_accept_child_heading_fields_and_do_not_borrow_the_next_proof() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("chapter.md");
    let source = "## CFT-01-002 {#cft-01-002}\n\n### Statement\n\nThe zero map sends zero to zero.\n\n### Proof\n\nEvaluate the defining constant function at zero.\n\n## CFT-01-003 {#cft-01-003}\n\n### Statement\n\nThe identity map fixes zero.\n\n### Proof\n\nEvaluate the identity function at zero.\n";
    fs::write(&path, source).unwrap();
    let read = fs::read_to_string(&path).unwrap();
    let card = anchored_card(&read, "cft-01-002");
    let sections = labeled_sections(card);
    assert!(has_content(&sections, "statement"));
    assert!(has_content(&sections, "proof"));
    assert!(
        !card.contains("identity map"),
        "card includes the next statement"
    );

    fs::write(
        &path,
        source.replace(
            "### Proof\n\nEvaluate the defining constant function at zero.\n\n",
            "",
        ),
    )
    .unwrap();
    let changed = fs::read_to_string(&path).unwrap();
    let sections = labeled_sections(anchored_card(&changed, "cft-01-002"));
    assert!(has_content(&sections, "statement"));
    assert!(
        !has_content(&sections, "proof"),
        "card borrows the next theorem's proof"
    );

    // Editorial field headings are accepted as well as the bold labels above.
    let headings = EDITORIAL_FIXTURE
        .replace("**Mathematical object.**", "### Mathematical object\n\n")
        .replace("**Exact transfer.**", "### Exact transfer\n\n")
        .replace("**Non-transfer.**", "### Non-transfer\n\n")
        .replace("**Calculation.**", "### Calculation\n\n");
    fs::write(&path, headings).unwrap();
    assert!(check_editorial(&[path.as_path()]).is_ok());
}

#[test]
fn editorial_contract_accepts_labeled_content_and_rejects_a_missing_field() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("chapter.md");
    fs::write(&path, EDITORIAL_FIXTURE).unwrap();
    assert!(check_editorial(&[path.as_path()]).is_ok());

    fs::write(
        &path,
        EDITORIAL_FIXTURE.replace("**Non-transfer.**", "**An unbounded analogy.**"),
    )
    .unwrap();
    let errors = check_editorial(&[path.as_path()]).unwrap_err();
    assert!(errors.iter().any(|error| error.contains("Non-transfer")));
}

#[test]
fn editorial_contract_rejects_repeated_motivation_and_ml_paragraphs() {
    let temporary = tempfile::tempdir().unwrap();
    let first = temporary.path().join("first.md");
    let second = temporary.path().join("second.md");
    fs::write(&first, EDITORIAL_FIXTURE).unwrap();
    fs::write(&second, EDITORIAL_FIXTURE).unwrap();
    let errors = check_editorial(&[first.as_path(), second.as_path()]).unwrap_err();
    for label in [
        "Motivation",
        "Mathematical object",
        "Exact transfer",
        "Non-transfer",
        "Calculation",
    ] {
        assert!(
            errors
                .iter()
                .any(|error| error.contains(&format!("repeated {label}"))),
            "{errors:?}"
        );
    }
}

const EDITORIAL_LABELS: [&str; 6] = [
    "Motivation",
    "Historical context",
    "Mathematical object",
    "Exact transfer",
    "Non-transfer",
    "Calculation",
];

fn check_editorial(paths: &[&Path]) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let mut paragraphs = BTreeMap::new();
    for path in paths {
        let source = fs::read_to_string(path).expect("read editorial chapter");
        let sections = labeled_sections(&source);
        for label in EDITORIAL_LABELS {
            let key = label.to_lowercase();
            if sections.get(&key).is_none_or(|body| body.trim().is_empty()) {
                errors.push(format!("{}: missing or empty {label}", path.display()));
                continue;
            }
            if label == "Historical context" {
                continue;
            }
            for paragraph in sections[&key].split("\n\n") {
                let normalized = paragraph.split_whitespace().collect::<Vec<_>>().join(" ");
                if normalized.is_empty() {
                    continue;
                }
                if let Some(previous) = paragraphs.insert(normalized, *path) {
                    if previous != *path {
                        errors.push(format!(
                            "{}: repeated {label} paragraph from {}",
                            path.display(),
                            previous.display()
                        ));
                    }
                }
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

// Labels are an editorial interface, not a constraint on sentence wording.
// Accept a Markdown heading or a bold label ending in either a period or colon.
// A theorem title may precede its separately bold Statement or Proof label.
fn labeled_sections(source: &str) -> BTreeMap<String, String> {
    let mut sections = BTreeMap::<String, String>::new();
    let mut current = String::new();
    for line in source.lines() {
        let trimmed = line.trim();
        let marker = if trimmed.starts_with('#') {
            Some((trimmed.trim_start_matches('#').trim(), ""))
        } else {
            trimmed
                .strip_prefix("**")
                .and_then(|rest| rest.split_once("**"))
                .map(|first| {
                    first
                        .1
                        .trim_start()
                        .strip_prefix("**")
                        .and_then(|rest| rest.split_once("**"))
                        .filter(|(label, _)| {
                            matches!(
                                label
                                    .trim()
                                    .trim_end_matches(['.', ':'])
                                    .to_lowercase()
                                    .as_str(),
                                "statement" | "exact statement" | "proof"
                            )
                        })
                        .unwrap_or(first)
                })
        };
        if let Some((label, remainder)) = marker {
            current = label.trim().trim_end_matches(['.', ':']).to_lowercase();
            sections
                .entry(current.clone())
                .or_default()
                .push_str(remainder);
        } else if !current.is_empty() {
            let body = sections.entry(current.clone()).or_default();
            body.push('\n');
            body.push_str(line);
        }
    }
    sections
}

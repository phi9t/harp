use std::fs;
use std::path::Path;

use harp::crouzeix_textbook::{check, generate_correspondence};

const CHAPTER: &str =
    "knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations.md";
const NARRATIVE: &str = "**Theorem (column rule).** **Statement.** For a linear map and finite bases, each column records the coordinates of a basis image.\n\n**Proof.** The entries are the coefficients in the unique basis expansion.\n\n**Boundary.** Without a basis, coordinates need not be unique.\n\n**Formal correspondence.** Lean theorem `CrouzeixTextbook.Part01.matrix_column_is_basis_image` uses `LinearMap.toMatrix_apply`.\n";

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn fixture(card: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    copy_tree(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/crouzeix_textbook/valid"),
        root.path(),
    );
    for relative in [
        "formalization/lean/CrouzeixTextbook/Part01/ObjectsAndRepresentations.lean",
        "formalization/lean/CrouzeixTextbook/Part01/Exercises/Chapter01.lean",
    ] {
        let target = root.path().join(relative);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(root.path().join("fixture/Compile.lean"), target).unwrap();
    }
    let path = root.path().join(CHAPTER);
    let chapter = fs::read_to_string(&path).unwrap();
    let (prefix, rest) = chapter.split_once("#### Purpose").unwrap();
    let (_, suffix) = rest.split_once("### Columns are images").unwrap();
    fs::write(
        path,
        format!("{prefix}{card}\n### Columns are images{suffix}"),
    )
    .unwrap();
    root
}

fn rejects(card: &str, label: &str) {
    let root = fixture(card);
    let diagnostics = check(root.path()).expect_err(label);
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "crouzeix-textbook.markdown.theorem-card"
                && diagnostic.identity.as_deref() == Some("CFT-01-001")
        }),
        "{label}: {diagnostics:#?}"
    );
}

#[test]
fn narrative_accepts_current_approved_column_rule_through_public_check() {
    let chapter = include_str!("../../../knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations.md");
    let (_, card) = chapter.split_once("{#cft-01-002}").unwrap();
    let (card, _) = card.split_once("### Coordinate action").unwrap();
    let root = fixture(card);
    check(root.path()).expect("approved continuous proof with explicit bold labels");
    generate_correspondence(root.path()).expect("publication generator uses the same validator");
}

#[test]
fn narrative_accepts_combined_boundary_provider_and_nested_details() {
    for combined in ["Boundary and Lean provider", "Boundary and Lean providers"] {
        let card = NARRATIVE
            .replace("**Proof.**", "#### Argument details\n\n**Proof.**")
            .replace("**Boundary.**", &format!("**{combined}.**"))
            .replace("**Formal correspondence.**", "\n##### Formal detail\n");
        check(fixture(&card).path()).expect("combined boundary and named Lean provider");
    }
}

#[test]
fn narrative_rejects_missing_empty_and_duplicate_fields() {
    for label in ["Statement", "Proof", "Boundary", "Formal correspondence"] {
        let marker = format!("**{label}.**");
        let (before, after) = NARRATIVE.split_once(&marker).unwrap();
        let end = after.find("\n\n**").unwrap_or(after.len());
        rejects(
            &format!("{before}{}", &after[end..]),
            &format!("missing {label}"),
        );
        rejects(
            &format!("{before}{marker}{}", &after[end..]),
            &format!("empty {label}"),
        );
        rejects(
            &format!("{NARRATIVE}\n{marker} Duplicate value.\n"),
            &format!("duplicate {label}"),
        );
    }
}

#[test]
fn narrative_rejects_labels_in_code_quotes_and_unrelated_prose() {
    for label in ["Statement", "Proof", "Boundary", "Formal correspondence"] {
        let marker = format!("**{label}.**");
        let (before, after) = NARRATIVE.split_once(&marker).unwrap();
        let end = after.find("\n\n**").unwrap_or(after.len());
        let field = format!("{marker}{}", &after[..end]);
        for fake in [
            format!("```markdown\n{field}\n```"),
            format!("    {field}"),
            format!("`{field}`"),
            format!("> {field}"),
            format!("Some discussion mentions {field}"),
        ] {
            rejects(
                &format!("{before}\n\n{fake}\n{}", &after[end..]),
                &format!("fake {label}: {fake}"),
            );
        }
    }
}

#[test]
fn narrative_cannot_borrow_fields_from_the_next_card() {
    let (first, proof) = NARRATIVE.split_once("**Proof.**").unwrap();
    rejects(
        &format!("{first}\n### Next card\n\n**Proof.**{proof}"),
        "next card borrowing",
    );
}

#[test]
fn narrative_cannot_borrow_a_nested_registered_workshop_card() {
    let (first, _) = NARRATIVE.split_once("**Proof.**").unwrap();
    let root = fixture(first);
    let path = root.path().join(CHAPTER);
    let chapter = fs::read_to_string(&path).unwrap().replacen(
        "### Linear transformations",
        "## Linear transformations",
        1,
    );
    fs::write(path, chapter).unwrap();
    let diagnostics = check(root.path()).expect_err("nested registered card cannot supply fields");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "crouzeix-textbook.markdown.theorem-card"
            && diagnostic.identity.as_deref() == Some("CFT-01-001")
    }));
}

#[test]
fn narrative_rejects_empty_fields_masked_by_unrelated_labels() {
    for label in ["Statement", "Proof", "Boundary", "Formal correspondence"] {
        let marker = format!("**{label}.**");
        let (before, after) = NARRATIVE.split_once(&marker).unwrap();
        let end = after.find("\n\n**").unwrap_or(after.len());
        rejects(
            &format!("{before}{marker}\n\n**Unrelated.** A named `Lean.provider` is not the missing field.{}", &after[end..]),
            label,
        );
    }
}

#[test]
fn narrative_correspondence_requires_named_lean_information() {
    let (before, _) = NARRATIVE.split_once("**Formal correspondence.**").unwrap();
    for provider in [
        "Unrelated prose.",
        "Lean.",
        "`x`.",
        "[Some link](https://example.com).",
    ] {
        rejects(
            &format!("{before}**Formal correspondence.** {provider}"),
            provider,
        );
    }
}

#[test]
fn narrative_accepts_a_lean_source_link_as_provider() {
    let (before, _) = NARRATIVE.split_once("**Formal correspondence.**").unwrap();
    let root = fixture(&format!("{before}**Formal correspondence.** [Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean)."));
    check(root.path()).expect("a concrete Lean code link identifies the provider");
}

#[test]
fn narrative_rejects_a_quoted_workshop_as_missing_fields() {
    let quoted_workshop = [
        "Purpose",
        "Statement",
        "Hypothesis ledger",
        "Proof roadmap",
        "Proof",
        "Boundary case",
        "Pedagogical prerequisites",
        "Lean correspondence",
        "Historical context",
        "ML analogy",
    ]
    .map(|label| format!("> #### {label}\n> Quoted example.\n>\n"))
    .join("");
    rejects(
        &format!("**Statement.** A statement alone is incomplete.\n\n{quoted_workshop}"),
        "quoted workshop cannot replace missing narrative fields",
    );
}

#[test]
fn narrative_accepts_quoted_headings_without_changing_its_format() {
    let root = fixture(&format!(
        "> ### Quoted example\n> #### Proof\n> Quoted explanation.\n\n{NARRATIVE}"
    ));
    check(root.path()).expect("quoted headings cannot select the workshop format");
}

#[test]
fn narrative_rejects_malformed_lean_identifiers() {
    let (before, _) = NARRATIVE.split_once("**Formal correspondence.**").unwrap();
    let accepted = ["1.bad", "foo..bar", "foo.", ".foo", "foo.1bar", "foo.'bar"]
        .into_iter()
        .filter(|identifier| {
            check(
                fixture(&format!(
                    "{before}**Formal correspondence.** `{identifier}`."
                ))
                .path(),
            )
            .is_ok()
        })
        .collect::<Vec<_>>();
    assert!(
        accepted.is_empty(),
        "accepted malformed providers: {accepted:?}"
    );
}

#[test]
fn narrative_accepts_plausible_lean_identifiers() {
    let (before, _) = NARRATIVE.split_once("**Formal correspondence.**").unwrap();
    for identifier in [
        "LinearMap.toMatrix_apply",
        "span_minimality",
        "Module.Basis.repr",
        "_root_.Some.provider",
        "Lean.foo'",
    ] {
        check(
            fixture(&format!(
                "{before}**Formal correspondence.** `{identifier}`."
            ))
            .path(),
        )
        .expect(identifier);
    }
}

#[test]
fn narrative_rejects_interrupted_bold_labels() {
    let accepted = [
        "**Pro\nof.**",
        "**Pro  \nof.**",
        "**Pro\\\nof.**",
        "**Pro<!-- hidden -->of.**",
        "**Pro<em></em>of.**",
        "**Pro*o*f.**",
        "**Pro[o](https://example.com)f.**",
    ]
    .into_iter()
    .filter(|label| check(fixture(&NARRATIVE.replace("**Proof.**", label)).path()).is_ok())
    .collect::<Vec<_>>();
    assert!(
        accepted.is_empty(),
        "accepted interrupted labels: {accepted:?}"
    );
}

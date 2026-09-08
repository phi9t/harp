use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::fs::{self, hard_link};
use std::io::{self, BufReader, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{symlink, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use harp::crouzeix_textbook::{
    check, check_lean_receipt, check_packet, generate_correspondence, publish_crouzeix_textbook,
    FormalMode, LeanCorrespondenceStatus, ProseProofStatus, PublicationStatus, TextbookPublishMode,
};
use pulldown_cmark::{Event, Parser};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

const V2_FIXTURE: &str = "tests/fixtures/crouzeix_textbook/valid";

#[test]
fn duality_core_does_not_depend_on_its_scalar_invariant_previews() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let cards = coverage["items"].as_array().unwrap();
    let dependencies = cards
        .iter()
        .map(|row| {
            (
                row["item_id"].as_str().unwrap(),
                row["pedagogical_prerequisites"].as_array().unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for core in ["CFT-04-001", "CFT-04-002", "CFT-04-003"] {
        let mut pending = vec![core];
        let mut visited = BTreeSet::new();
        while let Some(id) = pending.pop() {
            assert!(
                !["CFT-04-004", "CFT-04-005", "CFT-04-006"].contains(&id),
                "{core} depends on the forward reference {id}"
            );
            if visited.insert(id) {
                pending.extend(dependencies[id].iter().map(|value| value.as_str().unwrap()));
            }
        }
    }
}

#[test]
fn foundations_02_04_publish_distinct_compiled_solutions_and_honest_previews() {
    let contracts = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let cards = contracts["items"].as_array().unwrap();
    let exercise_rows = exercises["exercises"].as_array().unwrap();
    for chapter in 2..=4 {
        for index in 1..=6 {
            let id = format!("CFT-{chapter:02}-E{index:02}");
            let row = exercise_rows
                .iter()
                .find(|row| row["exercise_id"] == id)
                .unwrap();
            assert!(
                row["lean_solution"].is_object(),
                "{id} needs a distinct solution"
            );
        }
    }
    let receipt = fresh_textbook_receipt();
    let declarations = receipt["declarations"].as_array().unwrap();
    let card_hashes = cards
        .iter()
        .map(|row| row["lean_declaration"]["type_sha256"].clone())
        .collect::<Vec<_>>();
    let mut solution_hashes = BTreeSet::new();
    for chapter in 2..=4 {
        for index in 1..=6 {
            let id = format!("CFT-{chapter:02}-E{index:02}");
            let row = exercise_rows
                .iter()
                .find(|row| row["exercise_id"] == id)
                .unwrap();
            let name = format!("CrouzeixTextbook.Part01.Exercises.Chapter{chapter:02}.exercise_{index:02}_solution");
            assert_eq!(row["lean_solution"]["declaration"], name);
            let compiled = declarations.iter().find(|row| row["name"] == name).unwrap();
            assert_eq!(compiled["kind"], "theorem", "{id} must not be an alias");
            assert_eq!(compiled["type_sha256"], row["lean_solution"]["type_sha256"]);
            assert!(
                !card_hashes.contains(&compiled["type_sha256"]),
                "{id} repeats a card"
            );
            assert!(solution_hashes.insert(compiled["type_sha256"].as_str().unwrap()));
            let card_id = format!("CFT-{chapter:02}-{index:03}");
            let card = cards.iter().find(|row| row["item_id"] == card_id).unwrap();
            assert_eq!(card["lean_correspondence_status"], "exact");
            let expected_proof = if chapter == 4 && index >= 4 {
                "summary"
            } else if chapter == 3 && index == 5 {
                "not-applicable"
            } else {
                "reconstructible"
            };
            assert_eq!(card["prose_proof_status"], expected_proof, "{card_id}");
        }
    }
}

#[test]
fn harp_chapter_has_six_exact_records_and_distinct_solutions() {
    check(&workspace_root()).expect("the integrated Harp chapter validates");
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let rows = coverage["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["chapter"] == 36)
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 6);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row["item_id"], format!("CFT-36-{:03}", index + 1));
        assert_eq!(row["lean_correspondence_status"], "exact");
        assert_eq!(row["prose_proof_status"], "reconstructible");
        assert_eq!(
            row["formal_mode"],
            if index == 1 {
                "proved-here"
            } else {
                "reexported-proof"
            }
        );
    }
    let solutions = exercises["exercises"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["chapter"] == 36)
        .map(|row| row["lean_solution"]["declaration"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(solutions.len(), 6);
    for index in 1..=6 {
        assert!(solutions.contains(
            format!("CrouzeixTextbook.Part06.Exercises.Chapter36.exercise_{index:02}_solution")
                .as_str()
        ));
    }
}

#[test]
fn harp_chapter_receipt_retains_local_providers_and_scalar_exercise_dependencies() {
    let receipt = fresh_textbook_receipt();
    let rows = receipt["declarations"].as_array().unwrap();
    let row = |name: &str| {
        rows.iter()
            .find(|row| row["name"] == name)
            .expect("Harp receipt row")
    };
    let terminal = row("CrouzeixTextbook.Part06.harp_polynomial_constant_two");
    assert_eq!(
        terminal["direct_dependencies"],
        json!(["CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem"])
    );
    for index in 1..=6 {
        let solution = row(&format!(
            "CrouzeixTextbook.Part06.Exercises.Chapter36.exercise_{index:02}_solution"
        ));
        assert_eq!(solution["kind"], "theorem");
        for dependency in solution["direct_dependencies"].as_array().unwrap() {
            let name = dependency.as_str().unwrap();
            assert!(
                !name.contains("MainTheorem") && !name.contains("jinFinalCrouzeixConjecture"),
                "exercise calls terminal provider {name}"
            );
        }
    }
}

type JsonMutation = fn(&mut Value);
type MarkdownMutation = fn(String) -> String;

fn textbook_lean_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(concat!(
            "set -eu; . \"$1\"; shift; ",
            "test -d \"$HARP_LEAN_CACHE_ROOT\"; ",
            "test -d \"$HARP_ELAN_HOME\"; ",
            "test -x \"$HARP_LEAN_TOOLCHAIN_BIN/lake\"; ",
            "test -x \"$HARP_LEAN_TOOLCHAIN_BIN/lean\"; ",
            "export ELAN_HOME=\"$HARP_ELAN_HOME\"; ",
            "export PATH=\"$HARP_LEAN_TOOLCHAIN_BIN:$PATH\"; ",
            "if [ \"$1\" = lake ]; then shift; exec \"$HARP_LEAN_TOOLCHAIN_BIN/lake\" \"$@\"; fi; ",
            "exec \"$@\""
        ))
        .arg("textbook-lean")
        .arg(workspace_root().join("scripts/harp_xdg_env.sh"))
        .arg(program);
    command
}

fn copy_v2_fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("v2 fixture root");
    let destination = temp.path().join("content/crouzeix_textbook");
    fs::create_dir_all(&destination).expect("v2 fixture contract directory");
    for name in [
        "coverage.json",
        "exercises.json",
        "compatibility_routes.json",
    ] {
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(V2_FIXTURE)
                .join("content/crouzeix_textbook")
                .join(name),
            destination.join(name),
        )
        .unwrap_or_else(|error| panic!("copy v2 fixture {name}: {error}"));
    }
    let knowledge = temp
        .path()
        .join("knowledge/crouzeix_textbook/part_01_linear_structure");
    fs::create_dir_all(&knowledge).expect("v2 fixture knowledge directory");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(V2_FIXTURE)
            .join("knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations.md"),
        knowledge.join("01_objects_and_representations.md"),
    )
    .expect("copy v2 fixture chapter");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(V2_FIXTURE)
            .join("knowledge/crouzeix_textbook/part_01_linear_structure/02_vector_spaces_and_subspaces.md"),
        knowledge.join("02_vector_spaces_and_subspaces.md"),
    )
    .expect("copy second v2 fixture chapter");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(V2_FIXTURE)
            .join("knowledge/crouzeix_textbook/reading_guide.md"),
        temp.path()
            .join("knowledge/crouzeix_textbook/reading_guide.md"),
    )
    .expect("copy v2 fixture support document");
    for name in ["source_registry.md", "claim_evidence_ledger.md"] {
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(V2_FIXTURE)
                .join("knowledge/crouzeix_textbook")
                .join(name),
            temp.path().join("knowledge/crouzeix_textbook").join(name),
        )
        .unwrap_or_else(|error| panic!("copy v2 fixture {name}: {error}"));
    }
    let fixture_artifact = temp.path().join("fixture/Compile.lean");
    fs::create_dir_all(fixture_artifact.parent().expect("fixture artifact parent"))
        .expect("create fixture artifact directory");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(V2_FIXTURE)
            .join("fixture/Compile.lean"),
        &fixture_artifact,
    )
    .expect("copy fixture artifact");
    for relative in [
        "formalization/lean/CrouzeixTextbook/Part01/ObjectsAndRepresentations.lean",
        "formalization/lean/CrouzeixTextbook/Part01/Exercises/Chapter01.lean",
    ] {
        let path = temp.path().join(relative);
        fs::create_dir_all(path.parent().expect("Lean source parent"))
            .expect("create Lean source parent");
        fs::copy(&fixture_artifact, path).expect("copy maintained Lean source fixture");
    }
    temp
}

fn replace_fixture_bytes(root: &Path, name: &str, from: &[u8], to: &[u8]) {
    let path = root.join("content/crouzeix_textbook").join(name);
    let bytes = fs::read(&path).expect("read fixture contract");
    let offset = bytes
        .windows(from.len())
        .position(|window| window == from)
        .unwrap_or_else(|| panic!("fixture does not contain {from:?}"));
    let mut changed = Vec::with_capacity(bytes.len() - from.len() + to.len());
    changed.extend_from_slice(&bytes[..offset]);
    changed.extend_from_slice(to);
    changed.extend_from_slice(&bytes[offset + from.len()..]);
    fs::write(path, changed).expect("mutate fixture contract");
}

fn mutate_fixture_json(root: &Path, name: &str, mutate: impl FnOnce(&mut Value)) {
    let path = root.join("content/crouzeix_textbook").join(name);
    let mut value = read_json(&path);
    mutate(&mut value);
    let mut bytes = serde_json::to_vec_pretty(&value).expect("serialize mutated fixture");
    bytes.push(b'\n');
    fs::write(path, bytes).expect("write mutated fixture");
}

fn mutate_fixture_markdown(root: &Path, name: &str, mutate: impl FnOnce(String) -> String) {
    let path = root.join("knowledge/crouzeix_textbook").join(name);
    let markdown = fs::read_to_string(&path).expect("read fixture Markdown");
    fs::write(path, mutate(markdown)).expect("write mutated fixture Markdown");
}

fn fixture_theorem_mut<'a>(coverage: &'a mut Value, item_id: &str) -> &'a mut Value {
    coverage["items"]
        .as_array_mut()
        .expect("coverage items")
        .iter_mut()
        .find(|row| row["item_id"] == item_id)
        .unwrap_or_else(|| panic!("missing fixture theorem {item_id}"))
}

fn generated_theorem_card(anchor: &str) -> String {
    format!(
        "\n### Generated theorem {{#{anchor}}}\n\n#### Purpose\nPurpose.\n#### Statement\nStatement.\n#### Hypothesis ledger\nHypotheses.\n#### Proof roadmap\nRoadmap.\n#### Proof\nProof.\n#### Boundary case\nBoundary.\n#### Pedagogical prerequisites\nPrerequisites.\n#### Lean correspondence\nLean.\n#### Historical context\nHistory.\n#### ML analogy\nAnalogy.\n"
    )
}

fn visible_markdown_text(markdown: &str) -> String {
    let visible = Parser::new(markdown)
        .filter_map(|event| match event {
            Event::Text(text) | Event::Code(text) => Some(text.into_string()),
            Event::SoftBreak | Event::HardBreak => Some(" ".to_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    visible.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn configure_first_theorem(
    root: &Path,
    formal_mode: &str,
    lean_status: &str,
    prose_status: &str,
    declaration: bool,
    underlying: Option<&str>,
) {
    mutate_fixture_json(root, "coverage.json", |value| {
        let theorem = &mut value["items"][0];
        theorem["formal_mode"] = json!(formal_mode);
        theorem["lean_correspondence_status"] = json!(lean_status);
        theorem["prose_proof_status"] = json!(prose_status);
        if declaration {
            theorem["lean_declaration"]["underlying_declaration"] = json!(underlying);
        } else {
            theorem["lean_declaration"] = Value::Null;
        }
    });
}

fn assert_has_diagnostic(
    diagnostics: &[harp::crouzeix_textbook::TextbookDiagnostic],
    code: &str,
    identity: Option<&str>,
    field: &str,
) {
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == code
                && diagnostic.identity.as_deref() == identity
                && diagnostic.field == field
        }),
        "missing diagnostic code={code} identity={identity:?} field={field}: {diagnostics:#?}"
    );
}

fn first_fixture_chapter(root: &Path) -> PathBuf {
    root.join(
        "knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations.md",
    )
}

fn fixture_support_document(root: &Path) -> PathBuf {
    root.join("knowledge/crouzeix_textbook/reading_guide.md")
}

fn mutate_first_theorem_card(root: &Path, mutate: impl FnOnce(&str) -> String) {
    let chapter = first_fixture_chapter(root);
    let markdown = fs::read_to_string(&chapter).expect("read fixture chapter");
    let marker = "\n### Columns are images";
    let (first, rest) = markdown
        .split_once(marker)
        .expect("fixture has second theorem card");
    fs::write(chapter, format!("{}{}{}", mutate(first), marker, rest))
        .expect("mutate first theorem card");
}

fn receipt_type_hash(normalized_type: &str) -> String {
    format!("{:x}", Sha256::digest(normalized_type.as_bytes()))
}

fn lean_receipt_fixture() -> (tempfile::TempDir, tempfile::TempDir, Value) {
    let fixture = copy_v2_fixture();
    let theorem_type = "theorem fixture type";
    let definition_type = "definition fixture type";
    let exercise_one_type = "exercise one fixture type";
    let exercise_two_type = "exercise two fixture type";
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][0]["lean_declaration"]["type_sha256"] =
            json!(receipt_type_hash(theorem_type));
        value["items"][1]["lean_declaration"]["type_sha256"] =
            json!(receipt_type_hash(definition_type));
    });
    mutate_fixture_json(fixture.path(), "exercises.json", |value| {
        value["exercises"][0]["lean_solution"]["type_sha256"] =
            json!(receipt_type_hash(exercise_one_type));
        value["exercises"][1]["lean_solution"]["type_sha256"] =
            json!(receipt_type_hash(exercise_two_type));
    });
    let mut receipt = json!({
        "schema_version": "crouzeix-textbook-lean-receipt/v1",
        "toolchain": "leanprover/lean4:v4.32.1",
        "target": "CrouzeixTextbook",
        "declarations": [
            {
                "name": "CrouzeixTextbook.Part01.matrix_column_is_basis_image",
                "kind": "theorem",
                "source_path": "formalization/lean/CrouzeixTextbook/Part01/ObjectsAndRepresentations.lean",
                "line": 10,
                "column": 1,
                "normalized_type": theorem_type,
                "type_sha256": receipt_type_hash(theorem_type),
                "direct_dependencies": [],
                "axioms": []
            },
            {
                "name": "CrouzeixTextbook.Part01.LinearTransformation",
                "kind": "definition",
                "source_path": "formalization/lean/CrouzeixTextbook/Part01/ObjectsAndRepresentations.lean",
                "line": 4,
                "column": 1,
                "normalized_type": definition_type,
                "type_sha256": receipt_type_hash(definition_type),
                "direct_dependencies": [],
                "axioms": []
            },
            {
                "name": "CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_01_solution",
                "kind": "theorem",
                "source_path": "formalization/lean/CrouzeixTextbook/Part01/Exercises/Chapter01.lean",
                "line": 12,
                "column": 1,
                "normalized_type": exercise_one_type,
                "type_sha256": receipt_type_hash(exercise_one_type),
                "direct_dependencies": [],
                "axioms": ["propext"]
            },
            {
                "name": "CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_02_solution",
                "kind": "theorem",
                "source_path": "formalization/lean/CrouzeixTextbook/Part01/Exercises/Chapter01.lean",
                "line": 20,
                "column": 1,
                "normalized_type": exercise_two_type,
                "type_sha256": receipt_type_hash(exercise_two_type),
                "direct_dependencies": [],
                "axioms": ["Classical.choice", "Quot.sound", "propext"]
            }
        ]
    });
    sort_receipt_rows(&mut receipt);
    let receipt_dir = tempfile::tempdir().expect("receipt directory");
    (fixture, receipt_dir, receipt)
}

fn sort_receipt_rows(receipt: &mut Value) {
    receipt["declarations"]
        .as_array_mut()
        .expect("receipt declarations")
        .sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
}

fn receipt_row(name: &str, normalized_type: &str, dependencies: &[&str]) -> Value {
    json!({
        "name": name,
        "kind": "theorem",
        "source_path": "formalization/lean/CrouzeixTextbook/Part01/ObjectsAndRepresentations.lean",
        "line": 10,
        "column": 1,
        "normalized_type": normalized_type,
        "type_sha256": receipt_type_hash(normalized_type),
        "direct_dependencies": dependencies,
        "axioms": []
    })
}

fn configure_provider_declaration(
    coverage: &mut Value,
    item_id: &str,
    name: &str,
    normalized_type: &str,
    underlying: Option<&str>,
) {
    let mut declaration = coverage["items"][0]["lean_declaration"].clone();
    declaration["name"] = json!(name);
    declaration["underlying_declaration"] = json!(underlying);
    declaration["type_sha256"] = json!(receipt_type_hash(normalized_type));
    let row = fixture_theorem_mut(coverage, item_id);
    row["formal_mode"] = json!(if underlying.is_some() {
        "reexported-proof"
    } else {
        "proved-here"
    });
    row["prose_proof_status"] = json!("reconstructible");
    row["lean_correspondence_status"] = json!("exact");
    row["lean_declaration"] = declaration;
}

fn write_lean_receipt(directory: &tempfile::TempDir, receipt: &Value) -> PathBuf {
    let path = directory.path().join("receipt.json");
    fs::write(
        &path,
        serde_json::to_vec(receipt).expect("serialize receipt"),
    )
    .expect("write receipt");
    path
}

fn assert_lean_receipt_failure(
    fixture: &tempfile::TempDir,
    receipt_dir: &tempfile::TempDir,
    receipt: &Value,
    field: &str,
) {
    let path = write_lean_receipt(receipt_dir, receipt);
    let diagnostics = check_lean_receipt(fixture.path(), &path).expect_err(field);
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "crouzeix-textbook.lean-receipt" && diagnostic.field == field
        }),
        "missing Lean receipt field {field}: {diagnostics:#?}"
    );
}

fn publication_manifest(root: &Path) -> PathBuf {
    root.join("atlas/src/content/generated/crouzeix_textbook/publication/current.json")
}

fn publication_bytes(root: &Path, outputs: &[PathBuf]) -> Vec<Vec<u8>> {
    outputs
        .iter()
        .map(|output| fs::read(root.join(output)).expect("published output"))
        .collect()
}

fn publication_file_snapshot(
    root: &Path,
    outputs: &[PathBuf],
) -> BTreeMap<PathBuf, (Vec<u8>, u32, u64, i64, i64)> {
    let publication = root.join("atlas/src/content/generated/crouzeix_textbook/publication");
    let mut paths = vec![
        publication.join("current.json"),
        publication.join("publication.lock"),
    ];
    paths.extend(outputs.iter().map(|output| root.join(output)));
    paths
        .into_iter()
        .map(|path| {
            let metadata = fs::metadata(&path).expect("published file metadata");
            let snapshot = (
                fs::read(&path).expect("published file bytes"),
                metadata.permissions().mode() & 0o7777,
                metadata.nlink(),
                metadata.mtime(),
                metadata.mtime_nsec(),
            );
            (path, snapshot)
        })
        .collect()
}

fn retired_dependency_manifest() -> PathBuf {
    PathBuf::from(["content/crouzeix_textbook/theorem", "_dependencies.json"].concat())
}

fn retired_public_theorems_module() -> PathBuf {
    PathBuf::from(
        [
            "formalization/lean/CrouzeixTextbook/",
            "Public",
            "Theorems.lean",
        ]
        .concat(),
    )
}

fn retired_reader_ledger_renderer() -> PathBuf {
    PathBuf::from(["scripts/render_crouzeix_textbook_", "ledgers.py"].concat())
}

const MAX_DELETION_GUARD_TRACKED_FILES: usize = 4_096;
const MAX_DELETION_GUARD_LIST_BYTES: u64 = 512 * 1024;
const MAX_DELETION_GUARD_FILE_BYTES: u64 = 4 * 1024 * 1024;

fn read_nul_terminated_bounded(reader: &mut impl Read, max_bytes: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(max_bytes.min(128));
    for _ in 0..=max_bytes {
        let mut byte = [0_u8; 1];
        reader.read_exact(&mut byte)?;
        if byte[0] == 0 {
            return Ok(bytes);
        }
        if bytes.len() == max_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "NUL-terminated Git header exceeds bound",
            ));
        }
        bytes.push(byte[0]);
    }
    unreachable!("bounded loop returns at its byte limit")
}

fn scan_retired_references_at(root: &Path, paths: &[PathBuf]) -> Result<Vec<String>, String> {
    if paths.len() > MAX_DELETION_GUARD_TRACKED_FILES {
        return Err(format!(
            "tracked-file count {} exceeds {}",
            paths.len(),
            MAX_DELETION_GUARD_TRACKED_FILES
        ));
    }
    let mut included = Vec::new();
    for relative in paths {
        if deletion_guard_excludes(relative) {
            continue;
        }
        if relative.is_absolute()
            || relative
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            return Err(format!(
                "tracked path is not root-relative and normalized: {}",
                relative.display()
            ));
        }
        included.push(relative);
    }
    if included.is_empty() {
        return Ok(Vec::new());
    }

    let mut requests = Vec::new();
    for relative in &included {
        requests.push(b':');
        requests.extend_from_slice(relative.to_string_lossy().as_bytes());
        requests.push(0);
        if requests.len() as u64 > MAX_DELETION_GUARD_LIST_BYTES {
            return Err(format!(
                "index blob request roster exceeds {MAX_DELETION_GUARD_LIST_BYTES} bytes"
            ));
        }
    }

    let mut child = Command::new("git")
        .args(["--no-replace-objects", "cat-file", "--batch", "-Z"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("cannot start bounded index blob reader: {error}"))?;
    let mut stdin = child.stdin.take().expect("piped git stdin");
    let writer = std::thread::spawn(move || -> io::Result<()> {
        stdin.write_all(&requests)?;
        drop(stdin);
        Ok(())
    });
    let mut reader = BufReader::new(child.stdout.take().expect("piped git stdout"));

    macro_rules! fail_batch {
        ($message:expr) => {{
            let _ = child.kill();
            let _ = child.wait();
            let _ = writer.join();
            return Err($message);
        }};
    }

    let mut violations = Vec::new();
    for relative in included {
        let header = match read_nul_terminated_bounded(&mut reader, 128) {
            Ok(header) => header,
            Err(error) => fail_batch!(format!(
                "cannot read index blob header for {}: {error}",
                relative.display()
            )),
        };
        let header = match std::str::from_utf8(&header) {
            Ok(header) => header,
            Err(error) => fail_batch!(format!(
                "index blob header is not UTF-8 for {}: {error}",
                relative.display()
            )),
        };
        let fields = header.split_ascii_whitespace().collect::<Vec<_>>();
        if fields.len() != 3 || fields[1] != "blob" {
            fail_batch!(format!(
                "invalid index blob header for {}: {header:?}",
                relative.display()
            ));
        }
        let size = match fields[2].parse::<u64>() {
            Ok(size) => size,
            Err(error) => fail_batch!(format!(
                "invalid index blob size for {}: {error}",
                relative.display()
            )),
        };
        if size > MAX_DELETION_GUARD_FILE_BYTES {
            fail_batch!(format!(
                "tracked index blob exceeds {} bytes: {}",
                MAX_DELETION_GUARD_FILE_BYTES,
                relative.display()
            ));
        }
        let mut bytes = vec![0_u8; size as usize];
        if let Err(error) = reader.read_exact(&mut bytes) {
            fail_batch!(format!(
                "cannot read exact index blob for {}: {error}",
                relative.display()
            ));
        }
        let mut separator = [0_u8; 1];
        if let Err(error) = reader.read_exact(&mut separator) {
            fail_batch!(format!(
                "cannot read index blob terminator for {}: {error}",
                relative.display()
            ));
        }
        if separator[0] != 0 {
            fail_batch!(format!(
                "invalid index blob terminator for {}",
                relative.display()
            ));
        }
        let text = String::from_utf8_lossy(&bytes);
        if let Some(identity) = retired_reference_in(&text) {
            violations.push(format!("{} names {identity}", relative.display()));
        }
    }
    match writer.join() {
        Ok(Ok(())) => {}
        Ok(Err(error)) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("cannot write index blob requests: {error}"));
        }
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err("index blob request writer panicked".to_owned());
        }
    }
    let status = child
        .wait()
        .map_err(|error| format!("cannot wait for bounded index blob reader: {error}"))?;
    if !status.success() {
        return Err(format!("bounded index blob reader failed with {status}"));
    }
    Ok(violations)
}

fn deletion_guard_excludes(path: &Path) -> bool {
    path.starts_with("docs")
        || path == Path::new("atlas/src/content/generated/corpus.json")
        || path == Path::new("atlas/dist/harp-atlas.html")
}

fn retired_reference_in(text: &str) -> Option<String> {
    let dependency = retired_dependency_manifest()
        .file_name()
        .expect("dependency manifest basename")
        .to_string_lossy()
        .into_owned();
    if text.contains(&dependency) {
        return Some(dependency);
    }
    let renderer = retired_reader_ledger_renderer()
        .file_name()
        .expect("renderer basename")
        .to_string_lossy()
        .into_owned();
    if text.contains(&renderer) {
        return Some(renderer);
    }

    for spelling in [
        ["CrouzeixTextbook/", "Public", "Theorems.lean"].concat(),
        ["CrouzeixTextbook\\", "Public", "Theorems.lean"].concat(),
        ["CrouzeixTextbook.", "Public", "Theorems"].concat(),
    ] {
        if text.contains(&spelling) {
            return Some(spelling);
        }
    }

    let basename = ["Public", "Theorems.lean"].concat();
    if text.lines().any(|line| {
        line.match_indices(&basename).any(|(basename_start, _)| {
            let prefix = &line[..basename_start];
            !["MathematicalFoundations/", "MathematicalFoundations\\"]
                .iter()
                .any(|allowed_component| {
                    prefix
                        .strip_suffix(allowed_component)
                        .is_some_and(|path_prefix| {
                            path_prefix.is_empty()
                                || path_prefix.ends_with('/')
                                || path_prefix.ends_with('\\')
                        })
                })
        })
    }) {
        return Some(basename);
    }
    None
}

fn non_documentation_source_paths() -> Result<Vec<PathBuf>, String> {
    let mut child = Command::new("git")
        .args(["ls-files", "-z", "--"])
        .current_dir(workspace_root())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot enumerate tracked files: {error}"))?;
    let mut bytes = Vec::new();
    child
        .stdout
        .take()
        .expect("piped git stdout")
        .take(MAX_DELETION_GUARD_LIST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read tracked-file roster: {error}"))?;
    if bytes.len() as u64 > MAX_DELETION_GUARD_LIST_BYTES {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!(
            "tracked-file roster exceeds {MAX_DELETION_GUARD_LIST_BYTES} bytes"
        ));
    }
    let status = child
        .wait()
        .map_err(|error| format!("cannot wait for tracked-file enumeration: {error}"))?;
    if !status.success() {
        return Err(format!("git ls-files failed with {status}"));
    }
    let paths = bytes
        .split(|byte| *byte == 0)
        .filter(|bytes| !bytes.is_empty())
        .map(|bytes| PathBuf::from(String::from_utf8_lossy(bytes).into_owned()))
        .collect::<Vec<_>>();
    if paths.len() > MAX_DELETION_GUARD_TRACKED_FILES {
        return Err(format!(
            "tracked-file count {} exceeds {}",
            paths.len(),
            MAX_DELETION_GUARD_TRACKED_FILES
        ));
    }
    Ok(paths)
}

fn index_fixture_paths(root: &Path, paths: &[PathBuf]) {
    if !root.join(".git").is_dir() {
        let status = Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(root)
            .status()
            .expect("initialize fixture repository");
        assert!(status.success(), "git init failed");
    }
    let status = Command::new("git")
        .args(["add", "--"])
        .args(paths)
        .current_dir(root)
        .status()
        .expect("index fixture paths");
    assert!(status.success(), "git add failed");
}

#[test]
fn deletion_old_paths_are_absent_and_have_no_non_documentation_callers() {
    let retired_paths = [
        retired_dependency_manifest(),
        retired_public_theorems_module(),
        retired_reader_ledger_renderer(),
    ];
    for path in &retired_paths {
        assert!(
            !workspace_root().join(path).exists(),
            "retired duplicate still exists: {}",
            path.display()
        );
    }

    let mut callers = Vec::new();
    let paths = non_documentation_source_paths().expect("bounded tracked-file enumeration");
    callers.extend(
        scan_retired_references_at(&workspace_root(), &paths)
            .expect("bounded retired-reference scan"),
    );
    assert!(
        callers.is_empty(),
        "retired duplicate callers remain:\n{}",
        callers.join("\n")
    );
}

#[test]
fn deletion_pedagogical_dependencies_are_published_only_from_coverage() {
    assert!(
        !workspace_root()
            .join(retired_dependency_manifest())
            .exists(),
        "the second pedagogical dependency authority still exists"
    );
    let (fixture, receipt_dir, receipt) = lean_receipt_fixture();
    let receipt_path = write_lean_receipt(&receipt_dir, &receipt);
    mutate_fixture_json(fixture.path(), "coverage.json", |coverage| {
        fixture_theorem_mut(coverage, "CFT-35-001")["pedagogical_prerequisites"] =
            json!(["CFT-29-001"]);
    });

    let published =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect("publish coverage-owned dependency graph");
    let ledger = published
        .outputs
        .iter()
        .find(|path| path.ends_with("pedagogical_dependency_ledger.md"))
        .expect("pedagogical dependency ledger");
    let markdown = fs::read_to_string(fixture.path().join(ledger)).expect("published ledger");
    assert!(markdown.contains("| CFT-35-001 | CFT-29-001 |"));
    assert!(!markdown.contains("| CFT-35-001 | CFT-32-001, CFT-34-001 |"));
}

#[test]
fn deletion_generated_lean_surface_is_the_contract_correspondence() {
    assert!(
        !workspace_root()
            .join(retired_public_theorems_module())
            .exists(),
        "the hand-maintained Lean declaration manifest still exists"
    );
    let aggregate =
        fs::read_to_string(workspace_root().join("formalization/lean/CrouzeixTextbook.lean"))
            .expect("Crouzeix textbook aggregate");
    assert!(aggregate.contains("import CrouzeixTextbook.Correspondence"));
    assert!(!aggregate.contains(&["CrouzeixTextbook.", "Public", "Theorems"].concat()));

    let expected = generate_correspondence(&workspace_root()).expect("contract correspondence");
    let actual = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Correspondence.lean"),
    )
    .expect("generated correspondence");
    assert_eq!(actual, expected);
}

#[test]
fn deletion_mise_publication_uses_a_fresh_receipt_then_rust_write_and_check() {
    assert!(
        !workspace_root()
            .join(retired_reader_ledger_renderer())
            .exists(),
        "the independent reader-ledger renderer still exists"
    );
    let mise = fs::read_to_string(workspace_root().join("mise.toml")).expect("mise config");
    let task = mise
        .split_once("[tasks.crouzeix-textbook-publication]")
        .map(|(_, task)| task)
        .expect("canonical Crouzeix textbook publication task");
    let task = task
        .split("\n[tasks.")
        .next()
        .expect("publication task body");
    let receipt = task
        .find("--receipt-output")
        .expect("fresh compiled receipt step");
    let write = task
        .find("crouzeix-textbook publish")
        .expect("Rust publication write step");
    let check = task
        .find("crouzeix-textbook check")
        .expect("Rust publication check step");
    assert!(
        receipt < write && write < check,
        "publication steps are out of order"
    );
    assert!(!task.contains(&retired_reader_ledger_renderer().display().to_string()));
}

#[test]
fn deletion_atlas_release_depends_on_publication_before_static_consumption() {
    let mise = fs::read_to_string(workspace_root().join("mise.toml")).expect("mise config");
    let config: toml::Value = toml::from_str(&mise).expect("parse mise config");
    let dependencies = config["tasks"]["verify-atlas"]["depends"]
        .as_array()
        .expect("verify-atlas dependencies")
        .iter()
        .map(|value| value.as_str().expect("dependency string"))
        .collect::<BTreeSet<_>>();
    assert!(
        dependencies.contains("crouzeix-textbook-publication"),
        "Atlas verification can consume static inputs before textbook publication"
    );
    assert!(
        !config["tasks"]["crouzeix-textbook-publication"]["depends"]
            .as_array()
            .expect("publication dependencies")
            .iter()
            .any(|dependency| dependency.as_str() == Some("verify-atlas")),
        "publication must not recurse through Atlas verification"
    );
}

#[test]
fn deletion_guard_matches_retired_spellings_including_knowledge() {
    let fixture = tempfile::tempdir().expect("deletion guard fixture");
    let cases = [
        ["CrouzeixTextbook/", "Public", "Theorems.lean"].concat(),
        ["CrouzeixTextbook\\", "Public", "Theorems.lean"].concat(),
        ["CrouzeixTextbook.", "Public", "Theorems"].concat(),
        ["Public", "Theorems.lean"].concat(),
        ["path.ends_with(\"", "Public", "Theorems.lean\")"].concat(),
        ["NotMathematicalFoundations/", "Public", "Theorems.lean"].concat(),
        ["NotMathematicalFoundations\\", "Public", "Theorems.lean"].concat(),
        retired_dependency_manifest()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
        retired_reader_ledger_renderer()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
    ];
    let mut paths = Vec::new();
    for (index, spelling) in cases.iter().enumerate() {
        let relative = if index == 3 {
            PathBuf::from("knowledge/crouzeix_textbook/retired-reference.md")
        } else {
            PathBuf::from(format!("src/reference-{index}.txt"))
        };
        let path = fixture.path().join(&relative);
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture directory");
        fs::write(&path, format!("retired caller: {spelling}\n")).expect("fixture caller");
        paths.push(relative);
    }
    index_fixture_paths(fixture.path(), &paths);
    let violations = scan_retired_references_at(fixture.path(), &paths).expect("bounded scan");
    assert_eq!(violations.len(), cases.len(), "{violations:#?}");

    let allowed = PathBuf::from("src/mathematical-foundations.txt");
    fs::write(
        fixture.path().join(&allowed),
        ["MathematicalFoundations/", "Public", "Theorems.lean\n"].concat(),
    )
    .expect("allowed Mathematical Foundations reference");
    index_fixture_paths(fixture.path(), std::slice::from_ref(&allowed));
    assert!(scan_retired_references_at(fixture.path(), &[allowed])
        .expect("bounded allowed scan")
        .is_empty());
}

#[test]
fn deletion_guard_excludes_only_docs_and_exact_task8_artifacts() {
    let fixture = tempfile::tempdir().expect("deletion guard exclusions fixture");
    let retired = retired_dependency_manifest()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let paths = [
        PathBuf::from("docs/retired-reference.md"),
        PathBuf::from("atlas/src/content/generated/corpus.json"),
        PathBuf::from("atlas/dist/harp-atlas.html"),
        PathBuf::from("atlas/src/content/generated/not-corpus.json"),
    ];
    for relative in &paths {
        let path = fixture.path().join(relative);
        fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture directory");
        fs::write(path, format!("retired caller: {retired}\n")).expect("fixture caller");
    }
    index_fixture_paths(fixture.path(), &paths);

    let violations = scan_retired_references_at(fixture.path(), &paths).expect("bounded scan");
    assert_eq!(
        violations,
        vec![format!("{} names {retired}", paths[3].display())],
        "the exclusion boundary must not widen beyond docs and exact Task8 artifacts"
    );
}

#[test]
fn deletion_guard_rejects_unbounded_file_counts_and_files() {
    let fixture = tempfile::tempdir().expect("deletion guard bounds fixture");
    let too_many = (0..=MAX_DELETION_GUARD_TRACKED_FILES)
        .map(|index| PathBuf::from(format!("src/{index}.txt")))
        .collect::<Vec<_>>();
    assert!(scan_retired_references_at(fixture.path(), &too_many)
        .expect_err("oversized path roster")
        .contains("tracked-file count"));

    let oversized = PathBuf::from("src/oversized.txt");
    fs::create_dir_all(fixture.path().join("src")).expect("fixture src");
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(fixture.path().join(&oversized))
        .expect("oversized fixture");
    file.set_len(MAX_DELETION_GUARD_FILE_BYTES + 1)
        .expect("extend fixture");
    index_fixture_paths(fixture.path(), std::slice::from_ref(&oversized));
    assert!(scan_retired_references_at(fixture.path(), &[oversized])
        .expect_err("oversized tracked file")
        .contains("tracked index blob exceeds"));
}

#[test]
fn deletion_guard_reads_index_pointer_instead_of_large_worktree_materialization() {
    let fixture = tempfile::tempdir().expect("indexed LFS fixture");
    let relative = PathBuf::from("evidence/materialized-lfs.bin");
    let path = fixture.path().join(&relative);
    fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture directory");
    fs::write(
        &path,
        "version https://git-lfs.github.com/spec/v1\noid sha256:0123456789abcdef\nsize 6600000\n",
    )
    .expect("LFS-like pointer");
    index_fixture_paths(fixture.path(), std::slice::from_ref(&relative));

    let materialized = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)
        .expect("open materialized artifact");
    materialized
        .set_len(MAX_DELETION_GUARD_FILE_BYTES + 1)
        .expect("materialize oversized artifact");

    assert!(scan_retired_references_at(fixture.path(), &[relative])
        .expect("scan indexed pointer")
        .is_empty());
}

#[test]
fn deletion_guard_ignores_git_replace_objects_for_index_blobs() {
    let fixture = tempfile::tempdir().expect("Git replacement fixture");
    let relative = PathBuf::from("src/replaced-reference.txt");
    let path = fixture.path().join(&relative);
    fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture directory");
    let retired = ["Public", "Theorems.lean"].concat();
    fs::write(&path, format!("retired caller: {retired}\n")).expect("retired index blob");
    index_fixture_paths(fixture.path(), std::slice::from_ref(&relative));

    let indexed_oid = Command::new("git")
        .args(["rev-parse", &format!(":{}", relative.display())])
        .current_dir(fixture.path())
        .output()
        .expect("resolve indexed object");
    assert!(indexed_oid.status.success(), "git rev-parse failed");
    let indexed_oid = String::from_utf8(indexed_oid.stdout)
        .expect("indexed OID UTF-8")
        .trim()
        .to_owned();

    let clean = fixture.path().join("clean-replacement.txt");
    fs::write(&clean, "clean replacement bytes\n").expect("clean replacement");
    let clean_oid = Command::new("git")
        .args(["hash-object", "-w", "--", "clean-replacement.txt"])
        .current_dir(fixture.path())
        .output()
        .expect("write clean replacement object");
    assert!(clean_oid.status.success(), "git hash-object failed");
    let clean_oid = String::from_utf8(clean_oid.stdout)
        .expect("clean OID UTF-8")
        .trim()
        .to_owned();
    let replace = Command::new("git")
        .args(["replace", &indexed_oid, &clean_oid])
        .current_dir(fixture.path())
        .status()
        .expect("install replacement ref");
    assert!(replace.success(), "git replace failed");

    let violations = scan_retired_references_at(fixture.path(), std::slice::from_ref(&relative))
        .expect("scan original index blob");
    assert_eq!(
        violations,
        vec![format!("{} names {retired}", relative.display())]
    );
}

#[test]
fn runtime_publication_state_is_untracked_and_precisely_ignored() {
    const RUNTIME_ROOT: &str = "atlas/src/content/generated/crouzeix_textbook/publication";
    let tracked = Command::new("git")
        .args(["ls-files", "--", RUNTIME_ROOT])
        .current_dir(workspace_root())
        .output()
        .expect("list tracked runtime publication state");
    assert!(tracked.status.success(), "git ls-files failed");
    assert!(
        tracked.stdout.is_empty(),
        "runtime publication files must not be tracked:\n{}",
        String::from_utf8_lossy(&tracked.stdout)
    );

    let ignored = Command::new("git")
        .args([
            "check-ignore",
            "--no-index",
            "--quiet",
            "--",
            &format!("{RUNTIME_ROOT}/current.json"),
        ])
        .current_dir(workspace_root())
        .status()
        .expect("check runtime publication ignore rule");
    assert!(
        ignored.success(),
        "runtime publication state is not ignored"
    );

    let sibling = Command::new("git")
        .args([
            "check-ignore",
            "--no-index",
            "--quiet",
            "--",
            "atlas/src/content/generated/crouzeix_textbook/canonical.json",
        ])
        .current_dir(workspace_root())
        .status()
        .expect("check neighboring generated path");
    assert!(
        !sibling.success(),
        "runtime ignore rule must not hide neighboring generated artifacts"
    );
}

#[test]
fn publication_check_is_read_only_and_write_switches_the_complete_generation() {
    let (fixture, receipt_dir, receipt) = lean_receipt_fixture();
    let receipt_path = write_lean_receipt(&receipt_dir, &receipt);
    let publication = fixture
        .path()
        .join("atlas/src/content/generated/crouzeix_textbook/publication");
    assert!(
        !publication.exists(),
        "a clean checkout fixture starts without runtime publication state"
    );

    let check_result =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Check)
            .expect("prepare publication in check mode");
    assert_eq!(check_result.theorem_count, 9);
    assert_eq!(check_result.exercise_count, 2);
    assert_eq!(check_result.exact_theorem_correspondence_count, 2);
    assert_eq!(check_result.checked_exercise_solution_count, 2);
    assert_eq!(check_result.outputs.len(), 6);
    assert!(!check_result.matched);
    assert!(!publication.exists(), "check mode must make zero writes");

    let write_result =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect("publish complete generation");
    assert!(
        !write_result.matched,
        "write reports the pre-write match state"
    );
    assert_eq!(write_result.outputs, check_result.outputs);
    for output in &write_result.outputs {
        assert!(
            fixture.path().join(output).is_file(),
            "{}",
            output.display()
        );
    }
    let status_path = write_result
        .outputs
        .iter()
        .find(|path| path.ends_with("status_ledger.md"))
        .expect("status ledger output");
    let status = fs::read_to_string(fixture.path().join(status_path)).expect("status ledger");
    assert!(status.contains("- Exact theorem correspondences: 2"));
    assert!(status.contains("- Checked Lean exercise solutions: 2"));
    assert!(!status.contains("- Exact correspondences:"));
    for axis in ["| Formal mode |", "| Exercise solution |"] {
        assert!(status.contains(axis), "missing truthful status axis {axis}");
    }
    let manifest_before = fs::read(publication_manifest(fixture.path())).expect("manifest");
    let outputs_before = publication_bytes(fixture.path(), &write_result.outputs);
    let files_before = publication_file_snapshot(fixture.path(), &write_result.outputs);
    assert_eq!(files_before[&publication.join("current.json")].1, 0o600);
    assert_eq!(files_before[&publication.join("publication.lock")].1, 0o600);
    for output in &write_result.outputs {
        assert_eq!(files_before[&fixture.path().join(output)].1, 0o444);
    }

    let matched =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Check)
            .expect("check current publication");
    assert!(matched.matched);
    assert_eq!(matched.outputs, write_result.outputs);
    assert_eq!(
        fs::read(publication_manifest(fixture.path())).expect("manifest after check"),
        manifest_before
    );
    assert_eq!(
        publication_bytes(fixture.path(), &matched.outputs),
        outputs_before
    );
    assert_eq!(
        publication_file_snapshot(fixture.path(), &matched.outputs),
        files_before,
        "check mode must not change publication bytes or metadata"
    );
}

#[test]
fn publication_late_invalid_receipt_preserves_all_previous_bytes() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    let receipt_path = write_lean_receipt(&receipt_dir, &receipt);
    let published =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect("initial publication");
    let manifest_before = fs::read(publication_manifest(fixture.path())).expect("manifest");
    let outputs_before = publication_bytes(fixture.path(), &published.outputs);

    receipt["declarations"][0]["type_sha256"] = json!("0".repeat(64));
    write_lean_receipt(&receipt_dir, &receipt);
    publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
        .expect_err("invalid receipt must fail before publication mutation");
    assert_eq!(
        fs::read(publication_manifest(fixture.path())).expect("preserved manifest"),
        manifest_before
    );
    assert_eq!(
        publication_bytes(fixture.path(), &published.outputs),
        outputs_before
    );
}

#[test]
fn publication_write_switches_a_changed_six_ledger_generation_as_one_set() {
    let (fixture, receipt_dir, receipt) = lean_receipt_fixture();
    let receipt_path = write_lean_receipt(&receipt_dir, &receipt);
    let first =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect("initial publication");
    let first_pointer = fs::read(publication_manifest(fixture.path())).expect("first pointer");

    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][1]["publication_status"] = json!("active");
    });
    let second =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect("switch changed publication generation");
    assert!(!second.matched);
    assert_eq!(second.outputs.len(), 6);
    assert_ne!(second.outputs, first.outputs);
    assert!(second
        .outputs
        .iter()
        .all(|output| fixture.path().join(output).is_file()));
    assert_ne!(
        fs::read(publication_manifest(fixture.path())).expect("second pointer"),
        first_pointer
    );
    assert!(
        first
            .outputs
            .iter()
            .all(|output| fixture.path().join(output).is_file()),
        "the previous generation remains immutable but is no longer current"
    );
    assert!(
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Check,)
            .expect("check switched generation")
            .matched
    );
}

#[test]
fn publication_rejects_symlinked_and_multiply_linked_outputs() {
    let (ancestor_fixture, ancestor_receipt_dir, ancestor_receipt) = lean_receipt_fixture();
    let ancestor_receipt_path = write_lean_receipt(&ancestor_receipt_dir, &ancestor_receipt);
    let outside = tempfile::tempdir().expect("outside publication directory");
    fs::create_dir_all(
        ancestor_fixture
            .path()
            .join("atlas/src/content/generated/crouzeix_textbook"),
    )
    .expect("create publication parent");
    symlink(
        outside.path(),
        ancestor_fixture
            .path()
            .join("atlas/src/content/generated/crouzeix_textbook/publication"),
    )
    .expect("symlink publication ancestor");
    publish_crouzeix_textbook(
        ancestor_fixture.path(),
        &ancestor_receipt_path,
        TextbookPublishMode::Write,
    )
    .expect_err("symlinked publication ancestor must fail closed");
    assert_eq!(
        fs::read_dir(outside.path())
            .expect("outside directory")
            .count(),
        0,
        "publication must not write through a symlink ancestor"
    );

    let (symlink_fixture, receipt_dir, receipt) = lean_receipt_fixture();
    let receipt_path = write_lean_receipt(&receipt_dir, &receipt);
    let published = publish_crouzeix_textbook(
        symlink_fixture.path(),
        &receipt_path,
        TextbookPublishMode::Write,
    )
    .expect("initial symlink fixture publication");
    let output = symlink_fixture.path().join(&published.outputs[0]);
    let outside = symlink_fixture.path().join("outside-output.md");
    fs::write(&outside, b"outside").expect("outside output");
    fs::remove_file(&output).expect("remove generated output");
    symlink(&outside, &output).expect("replace generated output with symlink");
    publish_crouzeix_textbook(
        symlink_fixture.path(),
        &receipt_path,
        TextbookPublishMode::Check,
    )
    .expect_err("symlinked output must fail closed");

    let (hardlink_fixture, hardlink_receipt_dir, hardlink_receipt) = lean_receipt_fixture();
    let hardlink_receipt_path = write_lean_receipt(&hardlink_receipt_dir, &hardlink_receipt);
    let published = publish_crouzeix_textbook(
        hardlink_fixture.path(),
        &hardlink_receipt_path,
        TextbookPublishMode::Write,
    )
    .expect("initial hardlink fixture publication");
    let output = hardlink_fixture.path().join(&published.outputs[0]);
    hard_link(
        &output,
        hardlink_fixture.path().join("published-output-hardlink.md"),
    )
    .expect("multiply link generated output");
    publish_crouzeix_textbook(
        hardlink_fixture.path(),
        &hardlink_receipt_path,
        TextbookPublishMode::Write,
    )
    .expect_err("multiply-linked output must fail closed");

    let (pointer_fixture, pointer_receipt_dir, pointer_receipt) = lean_receipt_fixture();
    let pointer_receipt_path = write_lean_receipt(&pointer_receipt_dir, &pointer_receipt);
    publish_crouzeix_textbook(
        pointer_fixture.path(),
        &pointer_receipt_path,
        TextbookPublishMode::Write,
    )
    .expect("initial pointer fixture publication");
    hard_link(
        publication_manifest(pointer_fixture.path()),
        pointer_fixture
            .path()
            .join("publication-pointer-hardlink.json"),
    )
    .expect("multiply link publication pointer");
    publish_crouzeix_textbook(
        pointer_fixture.path(),
        &pointer_receipt_path,
        TextbookPublishMode::Check,
    )
    .expect_err("multiply-linked publication pointer must fail closed");
}

#[test]
fn publication_blocked_generation_preserves_the_current_pointer_and_outputs() {
    let (fixture, receipt_dir, receipt) = lean_receipt_fixture();
    let receipt_path = write_lean_receipt(&receipt_dir, &receipt);
    let published =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect("initial publication");
    let manifest_before = fs::read(publication_manifest(fixture.path())).expect("manifest");
    let outputs_before = publication_bytes(fixture.path(), &published.outputs);

    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][1]["publication_status"] = json!("active");
    });
    let prepared =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Check)
            .expect("prepare changed generation");
    assert!(!prepared.matched);
    let blocked_generation = fixture
        .path()
        .join(prepared.outputs[0].parent().expect("generation parent"));
    assert!(!blocked_generation.exists());
    fs::write(&blocked_generation, b"blocked").expect("block generation directory");

    publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
        .expect_err("blocked generation must fail before pointer switch");
    assert_eq!(
        fs::read(publication_manifest(fixture.path())).expect("preserved manifest"),
        manifest_before
    );
    assert_eq!(
        publication_bytes(fixture.path(), &published.outputs),
        outputs_before
    );
}

#[test]
fn publication_lock_serializes_the_generation_switch() {
    let (fixture, receipt_dir, receipt) = lean_receipt_fixture();
    let receipt_path = write_lean_receipt(&receipt_dir, &receipt);
    let published =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect("initial publication");
    let pointer_before = fs::read(publication_manifest(fixture.path())).expect("pointer");
    let lock_path = fixture
        .path()
        .join("atlas/src/content/generated/crouzeix_textbook/publication/publication.lock");
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&lock_path)
        .expect("persistent publication lock");
    assert_eq!(
        unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
        0
    );
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][1]["publication_status"] = json!("active");
    });
    let error =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect_err("busy publication lock must reject a concurrent switch");
    assert_eq!(error.code(), "crouzeix-textbook.publication.lock-busy");
    assert_eq!(
        fs::read(publication_manifest(fixture.path())).expect("preserved pointer"),
        pointer_before
    );
    assert_eq!(
        publication_bytes(fixture.path(), &published.outputs).len(),
        6
    );
}

#[test]
fn publication_rejects_an_oversized_generation_entry_set() {
    let (fixture, receipt_dir, receipt) = lean_receipt_fixture();
    let receipt_path = write_lean_receipt(&receipt_dir, &receipt);
    let published =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect("initial publication");
    let generation = fixture
        .path()
        .join(published.outputs[0].parent().expect("generation directory"));
    fs::write(generation.join("seventh-ledger.md"), b"unexpected\n")
        .expect("inject seventh generation entry");
    let error =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Check)
            .expect_err("generation enumeration must fail closed at the extra entry");
    assert_eq!(error.code(), "crouzeix-textbook.publication.generation");
}

#[test]
fn publication_objects_have_exact_owner_mode_and_link_contracts() {
    let (fixture, receipt_dir, receipt) = lean_receipt_fixture();
    let receipt_path = write_lean_receipt(&receipt_dir, &receipt);
    let published =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect("publication");
    let owner = unsafe { libc::geteuid() };
    let publication = fixture
        .path()
        .join("atlas/src/content/generated/crouzeix_textbook/publication");
    let generations = publication.join("generations");
    let generation = fixture
        .path()
        .join(published.outputs[0].parent().expect("generation directory"));

    for directory in [&publication, &generations, &generation] {
        let metadata = fs::metadata(directory).expect("publication directory metadata");
        assert!(metadata.is_dir());
        assert_eq!(metadata.uid(), owner);
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o755);
    }
    for (path, mode) in [
        (publication_manifest(fixture.path()), 0o600),
        (publication.join("publication.lock"), 0o600),
    ] {
        let metadata = fs::metadata(path).expect("publication file metadata");
        assert!(metadata.is_file());
        assert_eq!(metadata.uid(), owner);
        assert_eq!(metadata.nlink(), 1);
        assert_eq!(metadata.permissions().mode() & 0o7777, mode);
    }
    for output in published.outputs {
        let metadata = fs::metadata(fixture.path().join(output)).expect("ledger metadata");
        assert!(metadata.is_file());
        assert_eq!(metadata.uid(), owner);
        assert_eq!(metadata.nlink(), 1);
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o444);
    }
}

#[test]
fn lean_receipt_accepts_the_exact_compiler_envelope() {
    let (fixture, receipt_dir, receipt) = lean_receipt_fixture();
    let path = write_lean_receipt(&receipt_dir, &receipt);
    check_lean_receipt(fixture.path(), &path).expect("exact receipt");
}

#[test]
fn publication_status_binds_the_exact_compiler_receipt_identity() {
    let (fixture, receipt_dir, receipt) = lean_receipt_fixture();
    let receipt_bytes = serde_json::to_vec(&receipt).expect("serialize fixture receipt");
    let receipt_sha256 = format!("{:x}", Sha256::digest(&receipt_bytes));
    let receipt_path = write_lean_receipt(&receipt_dir, &receipt);
    let result =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect("publish fixture receipt identity");
    let status_path = result
        .outputs
        .iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name == "status_ledger.md")
        })
        .expect("status ledger output");
    let status = fs::read_to_string(fixture.path().join(status_path)).expect("status ledger");
    for required in [
        format!("- Compiler receipt SHA-256: `{receipt_sha256}`"),
        format!("- Compiler receipt bytes: `{}`", receipt_bytes.len()),
        format!(
            "- Compiler receipt declarations: `{}`",
            array(&receipt, "declarations", "fixture receipt").len()
        ),
        "The compiler receipt identity hashes the exact validated receipt bytes".to_owned(),
        "The six-ledger publication generation is a separate digest".to_owned(),
    ] {
        assert!(
            status.contains(&required),
            "status ledger is missing receipt identity field `{required}`"
        );
    }
    let manifest = read_json(&publication_manifest(fixture.path()));
    assert_ne!(
        string(&manifest, "generation", "publication manifest"),
        receipt_sha256,
        "the six-ledger generation must not be mislabeled as the compiler receipt"
    );
}

#[test]
fn lean_receipt_rejects_missing_declarations() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    receipt["declarations"].as_array_mut().unwrap().remove(0);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "name");
}

#[test]
fn lean_receipt_rejects_stale_source_positions() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    receipt["declarations"][0]["line"] = json!(11);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "line");
}

#[test]
fn lean_receipt_rejects_type_fingerprint_drift() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    receipt["declarations"][0]["normalized_type"] = json!("changed type");
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "normalized_type");
}

#[test]
fn lean_receipt_rejects_unknown_namespace_dependencies() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    receipt["declarations"][0]["direct_dependencies"] = json!(["Foreign.Hidden.fact"]);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "direct_dependencies");
}

#[test]
fn lean_receipt_does_not_infer_provider_ownership_from_namespaces() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][0]["lean_declaration"]["name"] = json!("Crouzeix.Jin.Terminal.fixture");
    });
    receipt["declarations"][3]["name"] = json!("Crouzeix.Jin.Terminal.fixture");
    receipt["declarations"][3]["direct_dependencies"] =
        json!(["Crouzeix.LoristSchwenninger.Terminal.fixture"]);
    sort_receipt_rows(&mut receipt);
    let path = write_lean_receipt(&receipt_dir, &receipt);
    check_lean_receipt(fixture.path(), &path).expect("namespaces alone do not assign providers");
}

#[test]
fn lean_receipt_accepts_a_definition_alias_with_a_definition_provider() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    let provider = "CrouzeixTextbook.Part01.underlying_definition";
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][1]["lean_declaration"]["underlying_declaration"] = json!(provider);
        value["items"][1]["prose_proof_status"] = json!("reconstructible");
    });
    let public = receipt["declarations"]
        .as_array_mut()
        .expect("receipt declarations")
        .iter_mut()
        .find(|row| row["name"] == "CrouzeixTextbook.Part01.LinearTransformation")
        .expect("definition public receipt row");
    public["kind"] = json!("direct-alias");
    public["direct_dependencies"] = json!([provider]);
    let mut provider_row = public.clone();
    provider_row["name"] = json!(provider);
    provider_row["kind"] = json!("definition");
    provider_row["direct_dependencies"] = json!([]);
    receipt["declarations"]
        .as_array_mut()
        .expect("receipt declarations")
        .push(provider_row);
    sort_receipt_rows(&mut receipt);
    let path = write_lean_receipt(&receipt_dir, &receipt);
    check_lean_receipt(fixture.path(), &path)
        .expect("definition alias must be validated against a definition-kind provider");
}

#[test]
fn lean_receipt_rejects_mode_body_mismatch() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    receipt["declarations"][2]["kind"] = json!("theorem");
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "kind");
}

#[test]
fn lean_receipt_rejects_changed_axioms() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    receipt["declarations"][0]["axioms"] = json!(["Classical.choice"]);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "axioms");
}

#[test]
fn lean_receipt_rejects_changed_solution_axioms() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    receipt["declarations"][0]["axioms"] = json!([]);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "axioms");
}

#[test]
fn lean_receipt_rejects_oversized_input() {
    let (fixture, receipt_dir, _) = lean_receipt_fixture();
    let path = receipt_dir.path().join("receipt.json");
    fs::write(&path, vec![b' '; 4 * 1024 * 1024 + 1]).expect("oversized receipt");
    let diagnostics = check_lean_receipt(fixture.path(), &path).expect_err("oversized");
    assert_has_diagnostic(&diagnostics, "crouzeix-textbook.lean-receipt", None, "file");
}

#[test]
fn lean_receipt_rejects_duplicate_rows() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    let duplicate = receipt["declarations"][0].clone();
    receipt["declarations"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "name");
}

#[test]
fn lean_receipt_rejects_unexpected_rows() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    let mut extra = receipt["declarations"][0].clone();
    extra["name"] = json!("CrouzeixTextbook.Unexpected.fact");
    receipt["declarations"].as_array_mut().unwrap().push(extra);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "name");
}

#[test]
fn lean_receipt_rejects_absolute_source_paths() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    receipt["declarations"][0]["source_path"] = json!("/tmp/hidden.lean");
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "source_path");
}

#[test]
fn lean_receipt_rejects_non_strict_declaration_order() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    receipt["declarations"].as_array_mut().unwrap().swap(0, 1);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "name");
}

#[cfg(unix)]
#[test]
fn lean_receipt_rejects_a_hardlinked_input() {
    let (fixture, receipt_dir, receipt) = lean_receipt_fixture();
    let path = write_lean_receipt(&receipt_dir, &receipt);
    fs::hard_link(&path, receipt_dir.path().join("receipt-hardlink.json"))
        .expect("create receipt hardlink");
    let diagnostics = check_lean_receipt(fixture.path(), &path).expect_err("hardlinked receipt");
    assert_has_diagnostic(&diagnostics, "crouzeix-textbook.lean-receipt", None, "file");
}

#[test]
fn lean_receipt_rejects_contract_derived_direct_provider_crossing() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    let jin = "CrouzeixTextbook.Part06.JinPublic";
    let ls = "CrouzeixTextbook.Part06.LsPublic";
    mutate_fixture_json(fixture.path(), "coverage.json", |coverage| {
        configure_provider_declaration(coverage, "CFT-30-001", jin, "jin type", None);
        configure_provider_declaration(coverage, "CFT-33-001", ls, "ls type", None);
    });
    receipt["declarations"].as_array_mut().unwrap().extend([
        receipt_row(jin, "jin type", &[ls]),
        receipt_row(ls, "ls type", &[]),
    ]);
    sort_receipt_rows(&mut receipt);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "direct_dependencies");
}

#[test]
fn lean_receipt_rejects_contract_derived_transitive_provider_crossing() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    let jin = "CrouzeixTextbook.Part06.JinPublic";
    let bridge = "CrouzeixTextbook.Part06.JinUnderlying";
    let ls = "CrouzeixTextbook.Part06.LsPublic";
    mutate_fixture_json(fixture.path(), "coverage.json", |coverage| {
        configure_provider_declaration(coverage, "CFT-30-001", jin, "shared type", Some(bridge));
        configure_provider_declaration(coverage, "CFT-33-001", ls, "ls type", None);
    });
    let mut public = receipt_row(jin, "shared type", &[bridge]);
    public["kind"] = json!("direct-alias");
    receipt["declarations"].as_array_mut().unwrap().extend([
        public,
        receipt_row(bridge, "shared type", &[ls]),
        receipt_row(ls, "ls type", &[]),
    ]);
    sort_receipt_rows(&mut receipt);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "direct_dependencies");
}

#[test]
fn lean_receipt_rejects_solution_equal_to_checkpoint() {
    let (fixture, receipt_dir, receipt) = lean_receipt_fixture();
    mutate_fixture_json(fixture.path(), "exercises.json", |value| {
        value["exercises"][0]["lean_solution"]["declaration"] =
            json!("CrouzeixTextbook.Part01.matrix_column_is_basis_image");
    });
    let path = write_lean_receipt(&receipt_dir, &receipt);
    let diagnostics = check_lean_receipt(fixture.path(), &path).expect_err("same checkpoint");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.field == "lean_solution.declaration" }));
}

#[test]
fn lean_receipt_rejects_exercise_statement_equal_to_a_chapter_checkpoint() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    let checkpoint_type = receipt["declarations"]
        .as_array()
        .expect("receipt declarations")
        .iter()
        .find(|row| row["name"] == "CrouzeixTextbook.Part01.matrix_column_is_basis_image")
        .expect("chapter checkpoint row")["normalized_type"]
        .as_str()
        .expect("checkpoint normalized type")
        .to_owned();
    mutate_fixture_json(fixture.path(), "exercises.json", |value| {
        value["exercises"][0]["lean_solution"]["type_sha256"] =
            json!(receipt_type_hash(&checkpoint_type));
    });
    let solution = receipt["declarations"]
        .as_array_mut()
        .expect("receipt declarations")
        .iter_mut()
        .find(|row| {
            row["name"] == "CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_01_solution"
        })
        .expect("exercise solution receipt row");
    solution["normalized_type"] = json!(checkpoint_type);
    solution["type_sha256"] = json!(receipt_type_hash(
        solution["normalized_type"]
            .as_str()
            .expect("solution normalized type"),
    ));
    let path = write_lean_receipt(&receipt_dir, &receipt);
    let diagnostics = check_lean_receipt(fixture.path(), &path)
        .expect_err("exercise statement must differ from chapter checkpoint");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.identity.as_deref() == Some("CFT-01-E01")
            && diagnostic.field == "lean_solution.type_sha256"
    }));
}

#[test]
fn lean_receipt_rejects_exercise_statement_equal_to_a_cross_chapter_checkpoint() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    let checkpoint_name = "CrouzeixTextbook.Part06.CrossChapterCheckpoint";
    let checkpoint_type = "cross chapter checkpoint type";
    mutate_fixture_json(fixture.path(), "coverage.json", |coverage| {
        configure_provider_declaration(
            coverage,
            "CFT-33-001",
            checkpoint_name,
            checkpoint_type,
            None,
        );
    });
    mutate_fixture_json(fixture.path(), "exercises.json", |exercises| {
        exercises["exercises"][0]["lean_solution"]["type_sha256"] =
            json!(receipt_type_hash(checkpoint_type));
    });
    let solution = receipt["declarations"]
        .as_array_mut()
        .expect("receipt declarations")
        .iter_mut()
        .find(|row| {
            row["name"] == "CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_01_solution"
        })
        .expect("exercise solution receipt row");
    solution["normalized_type"] = json!(checkpoint_type);
    solution["type_sha256"] = json!(receipt_type_hash(checkpoint_type));
    receipt["declarations"]
        .as_array_mut()
        .unwrap()
        .push(receipt_row(checkpoint_name, checkpoint_type, &[]));
    sort_receipt_rows(&mut receipt);

    let path = write_lean_receipt(&receipt_dir, &receipt);
    let diagnostics = check_lean_receipt(fixture.path(), &path)
        .expect_err("exercise statement must differ from every chapter checkpoint");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.identity.as_deref() == Some("CFT-01-E01")
            && diagnostic.field == "lean_solution.type_sha256"
            && diagnostic.observed.contains("CFT-33-001")
    }));
}

#[test]
fn lean_receipt_exercise_requires_a_theorem_not_a_direct_alias() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    let solution = receipt["declarations"]
        .as_array_mut()
        .expect("receipt declarations")
        .iter_mut()
        .find(|row| {
            row["name"] == "CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_01_solution"
        })
        .expect("exercise solution receipt row");
    solution["kind"] = json!("direct-alias");
    solution["direct_dependencies"] =
        json!(["CrouzeixTextbook.Part01.matrix_column_is_basis_image"]);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "kind");
}

#[test]
fn lean_receipt_proved_here_rejects_direct_alias_body() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    receipt["declarations"][3]["kind"] = json!("direct-alias");
    receipt["declarations"][3]["direct_dependencies"] =
        json!(["CrouzeixConjecture.Substantive.fact"]);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "kind");
}

#[test]
fn lean_receipt_reexport_requires_substantive_underlying_declaration() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][0]["formal_mode"] = json!("reexported-proof");
        value["items"][0]["lean_declaration"]["underlying_declaration"] =
            json!("CrouzeixConjecture.Substantive.fact");
    });
    receipt["declarations"][3]["kind"] = json!("direct-alias");
    receipt["declarations"][3]["direct_dependencies"] =
        json!(["CrouzeixConjecture.Substantive.fact"]);
    let mut underlying = receipt["declarations"][3].clone();
    underlying["name"] = json!("CrouzeixConjecture.Substantive.fact");
    underlying["kind"] = json!("direct-alias");
    underlying["direct_dependencies"] = json!([]);
    receipt["declarations"]
        .as_array_mut()
        .unwrap()
        .push(underlying);
    sort_receipt_rows(&mut receipt);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "underlying_declaration");
}

#[test]
fn lean_receipt_reexport_rejects_claimed_target_that_is_only_a_type_dependency() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][0]["formal_mode"] = json!("reexported-proof");
        value["items"][0]["lean_declaration"]["underlying_declaration"] =
            json!("CrouzeixConjecture.Claimed.fact");
    });
    receipt["declarations"][3]["kind"] = json!("direct-alias");
    receipt["declarations"][3]["direct_dependencies"] = json!([
        "CrouzeixConjecture.Actual.fact",
        "CrouzeixConjecture.Claimed.fact"
    ]);
    let mut actual = receipt["declarations"][3].clone();
    actual["name"] = json!("CrouzeixConjecture.Actual.fact");
    actual["kind"] = json!("theorem");
    actual["direct_dependencies"] = json!([]);
    let mut claimed = actual.clone();
    claimed["name"] = json!("CrouzeixConjecture.Claimed.fact");
    receipt["declarations"]
        .as_array_mut()
        .unwrap()
        .extend([actual, claimed]);
    sort_receipt_rows(&mut receipt);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "underlying_declaration");
}

#[test]
fn lean_receipt_reexport_requires_exactly_matching_normalized_types() {
    let (fixture, receipt_dir, mut receipt) = lean_receipt_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][0]["formal_mode"] = json!("reexported-proof");
        value["items"][0]["lean_declaration"]["underlying_declaration"] =
            json!("CrouzeixConjecture.Substantive.fact");
    });
    receipt["declarations"][3]["kind"] = json!("direct-alias");
    receipt["declarations"][3]["direct_dependencies"] =
        json!(["CrouzeixConjecture.Substantive.fact"]);
    let mut underlying = receipt["declarations"][3].clone();
    underlying["name"] = json!("CrouzeixConjecture.Substantive.fact");
    underlying["kind"] = json!("theorem");
    underlying["normalized_type"] = json!("different underlying type");
    underlying["type_sha256"] = json!(receipt_type_hash("different underlying type"));
    underlying["direct_dependencies"] = json!([]);
    receipt["declarations"]
        .as_array_mut()
        .unwrap()
        .push(underlying);
    sort_receipt_rows(&mut receipt);
    assert_lean_receipt_failure(&fixture, &receipt_dir, &receipt, "underlying_declaration");
}

#[test]
fn lean_receipt_correspondence_is_generated_and_sorted() {
    let (fixture, _, _) = lean_receipt_fixture();
    let source = generate_correspondence(fixture.path()).expect("generated correspondence");
    assert!(source.starts_with("-- GENERATED"));
    assert!(source.contains("#check CrouzeixTextbook.Part01.matrix_column_is_basis_image"));
    assert!(
        source.contains("#check CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_01_solution")
    );
    assert!(source.contains(
        "noncomputable def CrouzeixTextbook.ReceiptMarker.declaration0001 :=\n  @CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_01_solution"
    ));
    assert!(!source.contains("#receipt_declaration"));
    let imports = source
        .lines()
        .filter(|line| line.starts_with("import "))
        .collect::<Vec<_>>();
    assert!(imports.windows(2).all(|window| window[0] < window[1]));
}

#[test]
fn lean_correspondence_rejects_injected_names_and_invalid_source_routes() {
    let injected = copy_v2_fixture();
    mutate_fixture_json(injected.path(), "coverage.json", |coverage| {
        coverage["items"][0]["lean_declaration"]["name"] =
            json!("CrouzeixTextbook.Safe\n#check False");
    });
    let diagnostics = generate_correspondence(injected.path()).expect_err("name injection");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.field == "lean_declaration.name"));

    for source_path in [
        "formalization/lean/.lean",
        "formalization/lean/../Injected.lean",
        "formalization/lean/CrouzeixTextbook/Missing.lean",
    ] {
        let fixture = copy_v2_fixture();
        mutate_fixture_json(fixture.path(), "coverage.json", |coverage| {
            coverage["items"][0]["lean_declaration"]["source_path"] = json!(source_path);
        });
        let diagnostics = generate_correspondence(fixture.path()).expect_err(source_path);
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.field == "source_path"));
    }

    let foreign = copy_v2_fixture();
    let foreign_source = foreign
        .path()
        .join("formalization/lean/Foreign/Injected.lean");
    fs::create_dir_all(foreign_source.parent().unwrap()).expect("foreign module parent");
    fs::write(&foreign_source, "theorem injected : True := by trivial\n")
        .expect("foreign module source");
    mutate_fixture_json(foreign.path(), "coverage.json", |coverage| {
        coverage["items"][0]["lean_declaration"]["source_path"] =
            json!("formalization/lean/Foreign/Injected.lean");
    });
    let diagnostics = generate_correspondence(foreign.path()).expect_err("foreign module");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.field == "source_path"));
}

#[cfg(unix)]
#[test]
fn lean_correspondence_rejects_symlinked_and_hardlinked_sources() {
    for hardlink in [false, true] {
        let fixture = copy_v2_fixture();
        let source = fixture
            .path()
            .join("formalization/lean/CrouzeixTextbook/Part01/ObjectsAndRepresentations.lean");
        let replacement = fixture.path().join("replacement.lean");
        fs::write(&replacement, "theorem replacement : True := by trivial\n")
            .expect("replacement source");
        fs::remove_file(&source).expect("remove source");
        if hardlink {
            fs::hard_link(&replacement, &source).expect("hardlink source");
        } else {
            symlink(&replacement, &source).expect("symlink source");
        }
        let diagnostics = generate_correspondence(fixture.path()).expect_err("unsafe source");
        assert!(diagnostics
            .iter()
            .any(|diagnostic| diagnostic.field == "source_path"));
    }
}

#[test]
fn lean_receipt_canonical_correspondence_matches_the_contract_generator() {
    let expected = generate_correspondence(&workspace_root()).expect("canonical correspondence");
    let actual = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Correspondence.lean"),
    )
    .expect("read canonical correspondence");
    assert_eq!(actual, expected);
}

#[test]
fn lean_correspondence_rejects_v1_contracts_without_fallback() {
    let fixture = tempfile::tempdir().expect("v1 correspondence fixture");
    let contracts = fixture.path().join("content/crouzeix_textbook");
    fs::create_dir_all(&contracts).expect("v1 contract directory");
    fs::write(
        contracts.join("coverage.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version": "crouzeix-textbook-coverage/v1",
            "items": [{
                "chapter": 1,
                "lean_declaration": "CrouzeixTextbook.Part01.LinearTransformation",
                "verification_target": "CrouzeixTextbook"
            }]
        }))
        .expect("serialize v1 coverage"),
    )
    .expect("write v1 coverage");
    fs::write(
        contracts.join("exercises.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version": "crouzeix-textbook-exercises/v1",
            "exercises": [{"chapter": 1, "solution_declaration": null}]
        }))
        .expect("serialize v1 exercises"),
    )
    .expect("write v1 exercises");

    let diagnostics = generate_correspondence(fixture.path()).expect_err("v1 must fail closed");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "crouzeix-textbook.contract.deserialize"));
    assert!(!diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "crouzeix-textbook.correspondence"));
}

#[test]
fn lean_correspondence_preserves_v2_diagnostics_for_invalid_contracts() {
    let fixture = copy_v2_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][0]["lean_declaration"] =
            json!("CrouzeixTextbook.Safe\n#eval IO.println \"injected\"");
    });
    let diagnostics = generate_correspondence(fixture.path()).expect_err("invalid v2 contract");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "crouzeix-textbook.contract.deserialize"));
    assert!(!diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "crouzeix-textbook.correspondence"));
}

#[test]
fn canonical_identity_maps_all_current_documents_and_legacy_mismatches() {
    check_packet(&workspace_root()).expect("canonical textbook packet must validate");
    let mapping =
        read_json(&workspace_root().join("content/crouzeix_textbook/compatibility_routes.json"));
    assert_eq!(
        string(&mapping, "schema_version", "compatibility routes"),
        "crouzeix-textbook-compatibility-routes/v1"
    );
    let routes = array(&mapping, "routes", "compatibility routes");
    assert_eq!(routes.len(), 47);
    let mismatches = routes
        .iter()
        .filter(|route| {
            string(route, "canonical_id", "route") != string(route, "legacy_concept_id", "route")
        })
        .count();
    assert_eq!(mismatches, 39);
    assert!(routes.iter().any(|route| {
        string(route, "canonical_id", "route") == "cft-chapter-01-objects-and-representations"
            && string(route, "legacy_concept_id", "route")
                == "crouzeix-textbook-chapter-01"
            && string(route, "canonical_path", "route")
                == "knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations.md"
    }));
}

#[test]
fn canonical_identity_rejects_duplicate_frontmatter_ids() {
    let fixture = copy_v2_fixture();
    let support = fixture_support_document(fixture.path());
    let markdown = fs::read_to_string(&support)
        .expect("read support document")
        .replacen(
            "crouzeix-textbook-reading-guide",
            "cft-chapter-01-objects-and-representations",
            1,
        );
    fs::write(support, markdown).expect("duplicate frontmatter ID");
    let diagnostics = check_packet(fixture.path()).expect_err("duplicate IDs must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.duplicate-identity",
        Some("cft-chapter-01-objects-and-representations"),
        "id",
    );
}

#[test]
fn canonical_identity_rejects_invalid_known_frontmatter_metadata() {
    let cases = [
        (
            "confidence: high",
            "confidence: bogus",
            "invalid confidence",
        ),
        (
            "tags: [crouzeix-textbook]",
            "tags: crouzeix-textbook",
            "malformed tags",
        ),
        (
            "status: active",
            "status: active\nstatus: draft",
            "duplicate known field",
        ),
    ];
    for (from, to, label) in cases {
        let fixture = copy_v2_fixture();
        let chapter = first_fixture_chapter(fixture.path());
        let markdown = fs::read_to_string(&chapter)
            .expect("read chapter")
            .replacen(from, to, 1);
        fs::write(chapter, markdown).expect("mutate known frontmatter metadata");
        let diagnostics = check_packet(fixture.path()).expect_err(label);
        assert_has_diagnostic(
            &diagnostics,
            "crouzeix-textbook.markdown.frontmatter",
            None,
            "frontmatter",
        );
    }
}

#[test]
fn canonical_identity_accepts_a_unique_registered_legacy_alias() {
    let fixture = copy_v2_fixture();
    mutate_fixture_json(fixture.path(), "compatibility_routes.json", |value| {
        value["routes"][1]["legacy_concept_id"] = json!("arbitrary-unique-chapter-02-alias");
    });
    check_packet(fixture.path()).expect("compatibility registration owns its unique aliases");
}

#[test]
fn canonical_identity_rejects_invalid_compatibility_contracts() {
    let malformed = copy_v2_fixture();
    fs::write(
        malformed
            .path()
            .join("content/crouzeix_textbook/compatibility_routes.json"),
        b"{",
    )
    .expect("write malformed compatibility routes");
    let diagnostics = check_packet(malformed.path()).expect_err("malformed routes must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.compatibility-route",
        None,
        "$",
    );

    let oversized = copy_v2_fixture();
    let compatibility = oversized
        .path()
        .join("content/crouzeix_textbook/compatibility_routes.json");
    let mut bytes = fs::read(&compatibility).expect("read compatibility routes");
    bytes.resize(128 * 1024 + 1, b' ');
    fs::write(compatibility, bytes).expect("write oversized compatibility routes");
    let diagnostics = check_packet(oversized.path()).expect_err("oversized routes must fail");
    assert!(diagnostics[0].expected.contains("131072 bytes"));
    assert!(diagnostics[0].expected.contains("compatibility"));
}

#[test]
fn canonical_identity_rejects_route_schema_identity_path_bijection_mutations() {
    let mutations: [(&str, JsonMutation); 5] = [
        ("schema", |value| {
            value["schema_version"] = json!("wrong/v1")
        }),
        ("canonical ID", |value| {
            value["routes"][0]["canonical_id"] = json!("wrong-canonical-id")
        }),
        ("escaped path", |value| {
            value["routes"][0]["canonical_path"] = json!("../outside.md")
        }),
        ("duplicate row", |value| {
            let duplicate = value["routes"][0].clone();
            value["routes"]
                .as_array_mut()
                .expect("routes array")
                .push(duplicate);
        }),
        ("missing row", |value| {
            value["routes"]
                .as_array_mut()
                .expect("routes array")
                .remove(0);
        }),
    ];
    for (label, mutate) in mutations {
        let fixture = copy_v2_fixture();
        mutate_fixture_json(fixture.path(), "compatibility_routes.json", mutate);
        let diagnostics = check_packet(fixture.path()).expect_err(label);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "crouzeix-textbook.markdown.compatibility-route"
                || diagnostic.code == "crouzeix-textbook.markdown.identity-collision"
        }));
    }
}

#[test]
fn canonical_identity_rejects_alias_namespace_collisions() {
    let cases = [
        (
            "crouzeix-textbook-reading-guide",
            "canonical document",
            true,
        ),
        ("CFT-01-001", "theorem", false),
        ("CFT-01-E01", "exercise", false),
        ("crouzeix-textbook-reading-guide", "alias", false),
    ];
    for (alias, label, isolate_canonical_collision) in cases {
        let fixture = copy_v2_fixture();
        mutate_fixture_json(fixture.path(), "compatibility_routes.json", |value| {
            value["routes"][0]["legacy_concept_id"] = json!(alias);
            if isolate_canonical_collision {
                value["routes"][1]["legacy_concept_id"] = json!("legacy-reading-guide");
            }
        });
        let diagnostics = check(fixture.path()).expect_err(label);
        assert_has_diagnostic(
            &diagnostics,
            "crouzeix-textbook.markdown.identity-collision",
            Some(alias),
            "legacy_concept_id",
        );
    }
}

#[test]
fn prose_anchor_rejects_missing_theorem_anchor() {
    let fixture = copy_v2_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][0]["anchor"] = json!("missing-theorem-anchor");
    });
    let diagnostics = check(fixture.path()).expect_err("missing theorem anchor must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.theorem-anchor",
        Some("CFT-01-001"),
        "anchor",
    );
}

#[test]
fn prose_anchor_rejects_duplicate_theorem_anchor() {
    let fixture = copy_v2_fixture();
    let chapter = first_fixture_chapter(fixture.path());
    let mut markdown = fs::read_to_string(&chapter).expect("read chapter");
    markdown.push_str("\n### Duplicate {#linear-transformations-before-coordinates}\n");
    fs::write(chapter, markdown).expect("write duplicate anchor");
    let diagnostics = check(fixture.path()).expect_err("duplicate anchor must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.heading",
        None,
        "heading_id",
    );
}

#[test]
fn prose_anchor_rejects_a_theorem_card_missing_a_required_subsection() {
    let fixture = copy_v2_fixture();
    let chapter = first_fixture_chapter(fixture.path());
    let markdown = fs::read_to_string(&chapter)
        .expect("read chapter")
        .replacen("#### Historical context", "#### Context", 1);
    fs::write(chapter, markdown).expect("remove theorem-card subsection");
    let diagnostics = check(fixture.path()).expect_err("incomplete theorem card must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.theorem-card",
        Some("CFT-01-001"),
        "subsections",
    );
}

#[test]
fn prose_summary_requires_a_stable_anchor_but_defers_exact_proof_subsections() {
    let fixture = copy_v2_fixture();
    configure_first_theorem(
        fixture.path(),
        "proved-here",
        "exact",
        "summary",
        true,
        None,
    );
    let chapter = first_fixture_chapter(fixture.path());
    let markdown = fs::read_to_string(&chapter)
        .expect("read chapter")
        .replacen("#### Historical context", "#### Context", 1);
    fs::write(chapter, markdown).expect("remove summary-row theorem-card subsection");

    check(fixture.path()).expect("summary row does not claim an exact proof card");

    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][0]["anchor"] = json!("missing-summary-anchor");
    });
    let diagnostics = check(fixture.path()).expect_err("summary row still requires an anchor");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.theorem-anchor",
        Some("CFT-01-001"),
        "anchor",
    );
}

#[test]
fn prose_anchor_accepts_valid_nested_headings_and_ignores_fenced_heading_text() {
    let nested = copy_v2_fixture();
    mutate_first_theorem_card(nested.path(), |card| {
        card.replacen(
            "### Linear transformations before coordinates",
            "## Linear transformations before coordinates",
            1,
        )
        .replace("#### ", "### ")
        .replacen(
            "### Proof\n",
            "### Proof\n\n#### Proof detail\nNested detail.\n",
            1,
        )
    });
    let chapter = first_fixture_chapter(nested.path());
    let markdown = fs::read_to_string(&chapter)
        .expect("read nested-heading fixture")
        .replacen("### Columns are images", "## Columns are images", 1);
    fs::write(chapter, markdown).expect("align the following theorem-card level");
    check(nested.path()).expect("deeper nested headings stay inside an H2 theorem card");

    let fenced = copy_v2_fixture();
    mutate_first_theorem_card(fenced.path(), |card| {
        card.replacen(
            "#### Boundary case",
            "```markdown\n#### Proof\n```\n\n#### Boundary case",
            1,
        )
    });
    check(fenced.path()).expect("heading-like fenced code must not duplicate a subsection");
}

#[test]
fn prose_anchor_rejects_duplicate_required_subsections() {
    let duplicate = copy_v2_fixture();
    mutate_first_theorem_card(duplicate.path(), |card| {
        card.replacen(
            "#### Boundary case",
            "#### Proof\nDuplicate proof.\n\n#### Boundary case",
            1,
        )
    });
    let diagnostics = check(duplicate.path()).expect_err("duplicate Proof must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.theorem-card",
        Some("CFT-01-001"),
        "subsections",
    );

    let masked_missing = copy_v2_fixture();
    mutate_first_theorem_card(masked_missing.path(), |card| {
        card.replacen("#### Historical context", "#### Proof", 1)
    });
    let diagnostics =
        check(masked_missing.path()).expect_err("duplicate cannot mask a missing subsection");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.theorem-card",
        Some("CFT-01-001"),
        "subsections",
    );
}

#[test]
fn prose_anchor_rejects_chapter_path_escape() {
    let fixture = copy_v2_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][0]["prose_path"] = json!("../outside.md");
    });
    let diagnostics = check(fixture.path()).expect_err("escaped theorem path must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.theorem-path",
        Some("CFT-01-001"),
        "prose_path",
    );
}

#[test]
fn rejects_nonregular_symlinked_chapter() {
    let fixture = copy_v2_fixture();
    let chapter = first_fixture_chapter(fixture.path());
    let outside = fixture.path().join("outside.md");
    fs::copy(&chapter, &outside).expect("copy outside chapter");
    fs::remove_file(&chapter).expect("remove chapter");
    symlink(outside, chapter).expect("symlink chapter");
    let diagnostics = check(fixture.path()).expect_err("symlinked chapter must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.input",
        None,
        "file",
    );
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "crouzeix-textbook.markdown.input")
        .unwrap();
    assert_eq!(
        diagnostic.path.as_deref(),
        Some(Path::new("knowledge/crouzeix_textbook"))
    );
    assert!(diagnostic.expected.contains("Markdown tree"));
}

#[test]
fn rejects_nonregular_hardlinked_chapter() {
    let fixture = copy_v2_fixture();
    let chapter = first_fixture_chapter(fixture.path());
    hard_link(&chapter, fixture.path().join("chapter-hardlink.md")).expect("hardlink chapter");
    let diagnostics = check(fixture.path()).expect_err("hardlinked chapter must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.input",
        None,
        "file",
    );
}

#[cfg(unix)]
#[test]
fn rejects_nonregular_fifo_and_markdown_directory_nodes() {
    let fifo_fixture = copy_v2_fixture();
    let fifo = fifo_fixture
        .path()
        .join("knowledge/crouzeix_textbook/injected.md");
    let status = Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("run mkfifo");
    assert!(status.success(), "mkfifo must create the test node");
    let diagnostics = check_packet(fifo_fixture.path()).expect_err("FIFO Markdown must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.input",
        None,
        "file",
    );

    let directory_fixture = copy_v2_fixture();
    fs::create_dir(
        directory_fixture
            .path()
            .join("knowledge/crouzeix_textbook/injected.md"),
    )
    .expect("create Markdown-named directory");
    let diagnostics =
        check_packet(directory_fixture.path()).expect_err("Markdown directory must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.input",
        None,
        "file",
    );
}

#[test]
fn rejects_nonregular_bounded_discovery_limits() {
    let depth = copy_v2_fixture();
    let mut nested = depth.path().join("knowledge/crouzeix_textbook");
    for index in 0..9 {
        nested.push(format!("level-{index}"));
    }
    fs::create_dir_all(&nested).expect("create deeply nested packet path");
    fs::write(nested.join("too-deep.md"), b"deep").expect("write deep Markdown");
    let diagnostics = check_packet(depth.path()).expect_err("depth limit must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.input",
        None,
        "file",
    );

    let entries = copy_v2_fixture();
    let root = entries.path().join("knowledge/crouzeix_textbook");
    for index in 0..257 {
        fs::write(root.join(format!("entry-{index}.txt")), b"entry").expect("write packet entry");
    }
    let diagnostics = check_packet(entries.path()).expect_err("entry limit must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.input",
        None,
        "file",
    );

    let files = copy_v2_fixture();
    let root = files.path().join("knowledge/crouzeix_textbook");
    for index in 0..65 {
        fs::write(root.join(format!("extra-{index}.md")), b"file").expect("write packet Markdown");
    }
    let diagnostics = check_packet(files.path()).expect_err("file limit must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.input",
        None,
        "file",
    );

    let aggregate = copy_v2_fixture();
    let root = aggregate.path().join("knowledge/crouzeix_textbook");
    for index in 0..2 {
        fs::write(
            root.join(format!("large-{index}.md")),
            vec![b' '; 512 * 1024],
        )
        .expect("write aggregate Markdown bytes");
    }
    let diagnostics = check_packet(aggregate.path()).expect_err("aggregate limit must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.input",
        None,
        "file",
    );
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "crouzeix-textbook.markdown.input")
        .unwrap();
    assert_eq!(
        diagnostic.path.as_deref(),
        Some(Path::new("knowledge/crouzeix_textbook"))
    );
    assert!(diagnostic.expected.contains("1048576 aggregate bytes"));
}

#[test]
fn rejects_nonregular_oversized_markdown_input() {
    let fixture = copy_v2_fixture();
    let chapter = first_fixture_chapter(fixture.path());
    let relative_chapter = chapter.strip_prefix(fixture.path()).unwrap().to_path_buf();
    let mut bytes = fs::read(&chapter).expect("read fixture chapter");
    bytes.resize(512 * 1024 + 1, b' ');
    fs::write(chapter, bytes).expect("write oversized Markdown");
    let diagnostics = check_packet(fixture.path()).expect_err("oversized Markdown must fail");
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "crouzeix-textbook.markdown.input")
        .expect("oversized Markdown diagnostic");
    assert_eq!(diagnostic.path.as_ref(), Some(&relative_chapter));
    assert_eq!(
        diagnostic.expected,
        "regular single-link textbook Markdown no larger than 524288 bytes"
    );
    assert_eq!(diagnostic.observed, "textbook prose exceeds 524288 bytes");
}

#[test]
fn rejects_nonregular_encoding_and_frontmatter_inputs() {
    for (label, mutate) in [("NUL", b'\0'), ("CRLF", b'\r')] {
        let fixture = copy_v2_fixture();
        let chapter = first_fixture_chapter(fixture.path());
        let mut bytes = fs::read(&chapter).expect("read chapter");
        bytes.push(mutate);
        fs::write(chapter, bytes).expect("write invalid encoding");
        let diagnostics = check(fixture.path()).expect_err(label);
        assert_has_diagnostic(
            &diagnostics,
            "crouzeix-textbook.markdown.encoding",
            None,
            "contents",
        );
    }

    let malformed = copy_v2_fixture();
    let chapter = first_fixture_chapter(malformed.path());
    let markdown = fs::read_to_string(&chapter)
        .expect("read chapter")
        .replacen("id: ", "id ", 1);
    fs::write(chapter, markdown).expect("write malformed frontmatter");
    let diagnostics = check(malformed.path()).expect_err("malformed frontmatter");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.markdown.frontmatter",
        None,
        "frontmatter",
    );
}

#[test]
fn v2_rejects_unknown_and_missing_fields() {
    let unknown = copy_v2_fixture();
    replace_fixture_bytes(
        unknown.path(),
        "coverage.json",
        b"\"chapter\": 1,",
        b"\"chapter\": 1, \"surprise\": true,",
    );
    let diagnostics = check(unknown.path()).expect_err("unknown field must fail");
    assert_eq!(
        diagnostics[0].code,
        "crouzeix-textbook.contract.deserialize"
    );
    assert_eq!(diagnostics[0].field, "surprise");
    assert_eq!(diagnostics[0].expected, "known field");
    assert_eq!(diagnostics[0].observed, "unknown field");

    let missing = copy_v2_fixture();
    replace_fixture_bytes(
        missing.path(),
        "exercises.json",
        b"      \"difficulty\": \"core\",\n",
        b"",
    );
    let diagnostics = check(missing.path()).expect_err("missing field must fail");
    assert_eq!(
        diagnostics[0].code,
        "crouzeix-textbook.contract.deserialize"
    );
    assert_eq!(diagnostics[0].field, "difficulty");
    assert_eq!(diagnostics[0].expected, "present");
    assert_eq!(diagnostics[0].observed, "missing field");
}

#[test]
fn v2_separates_publication_prose_lean_and_review_status() {
    let fixture = copy_v2_fixture();
    replace_fixture_bytes(
        fixture.path(),
        "coverage.json",
        b"\"lean_correspondence_status\": \"exact\"",
        b"\"lean_correspondence_status\": \"unmapped\"",
    );

    let contracts = check(fixture.path()).expect("independent status axes must validate");
    let theorem = &contracts.theorems()[0];
    assert_eq!(theorem.publication_status(), PublicationStatus::Active);
    assert_eq!(
        theorem.prose_proof_status(),
        ProseProofStatus::Reconstructible
    );
    assert_eq!(
        theorem.lean_correspondence_status(),
        LeanCorrespondenceStatus::Unmapped
    );
    assert_eq!(theorem.review_status().source_id(), "LAX-2007");
    assert_eq!(theorem.review_status().status(), "source-checked");
}

#[test]
fn diagnostics_name_identity_field_expected_and_observed() {
    let fixture = copy_v2_fixture();
    replace_fixture_bytes(
        fixture.path(),
        "coverage.json",
        b"\"formal_mode\": \"proved-here\"",
        b"\"formal_mode\": \"checkpoint\"",
    );

    let diagnostics = check(fixture.path()).expect_err("checkpoint cannot claim exact proof");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0],
        harp::crouzeix_textbook::TextbookDiagnostic {
            code: "crouzeix-textbook.contract.invalid-combination",
            identity: Some("CFT-01-001".to_owned()),
            field: "lean_correspondence_status".to_owned(),
            expected: "unmapped or checkpoint".to_owned(),
            observed: "exact".to_owned(),
            path: Some(PathBuf::from("content/crouzeix_textbook/coverage.json")),
            line: None,
            column: None,
        }
    );
    let serialized = serde_json::to_value(&diagnostics[0]).expect("serialize diagnostic");
    assert_eq!(
        serialized
            .as_object()
            .expect("diagnostic object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["code", "column", "expected", "field", "identity", "line", "observed", "path",]
    );
    assert_eq!(FormalMode::Checkpoint.to_string(), "checkpoint");
}

#[test]
fn v2_rejects_an_exercise_solution_that_reuses_a_theorem_declaration() {
    let fixture = copy_v2_fixture();
    replace_fixture_bytes(
        fixture.path(),
        "exercises.json",
        b"CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_01_solution",
        b"CrouzeixTextbook.Part01.matrix_column_is_basis_image",
    );

    let diagnostics = check(fixture.path()).expect_err("solution must be a distinct declaration");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].identity.as_deref(), Some("CFT-01-E01"));
    assert_eq!(diagnostics[0].field, "lean_solution.declaration");
    assert_eq!(
        diagnostics[0].expected,
        "distinct exercise solution declaration"
    );
    assert_eq!(
        diagnostics[0].observed,
        "CrouzeixTextbook.Part01.matrix_column_is_basis_image"
    );
}

#[test]
fn v2_attributes_later_row_deserialization_diagnostics() {
    let missing = copy_v2_fixture();
    replace_fixture_bytes(
        missing.path(),
        "coverage.json",
        b"      \"anchor\": \"columns-are-images\",\n",
        b"",
    );
    let diagnostics = check(missing.path()).expect_err("later row missing field must fail");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].identity.as_deref(), Some("CFT-01-002"));
    assert_eq!(diagnostics[0].field, "anchor");

    let invalid_enum = copy_v2_fixture();
    mutate_fixture_json(invalid_enum.path(), "coverage.json", |value| {
        value["items"][1]["lean_correspondence_status"] = json!("definition");
    });
    let diagnostics = check(invalid_enum.path()).expect_err("later row enum must fail");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].identity.as_deref(), Some("CFT-01-002"));
    assert_eq!(diagnostics[0].field, "lean_correspondence_status");
    assert_eq!(diagnostics[0].observed, "definition");
    assert!(diagnostics[0].line.is_some());
    assert!(diagnostics[0].column.is_some());
}

#[test]
fn v2_reports_exact_nested_deserialization_paths() {
    let fixture = copy_v2_fixture();
    mutate_fixture_json(fixture.path(), "exercises.json", |value| {
        value["exercises"][1]["lean_solution"]
            .as_object_mut()
            .expect("Lean solution object")
            .remove("source_path");
    });

    let diagnostics = check(fixture.path()).expect_err("nested missing field must fail");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].identity.as_deref(), Some("CFT-01-E02"));
    assert_eq!(diagnostics[0].field, "lean_solution.source_path");
    assert!(diagnostics[0].line.is_some());
    assert!(diagnostics[0].column.is_some());
}

#[test]
fn v2_accumulates_independent_file_diagnostics() {
    let fixture = copy_v2_fixture();
    fs::write(
        fixture
            .path()
            .join("content/crouzeix_textbook/coverage.json"),
        b"{",
    )
    .expect("malformed coverage");
    fs::write(
        fixture
            .path()
            .join("content/crouzeix_textbook/exercises.json"),
        b"{",
    )
    .expect("malformed exercises");

    let diagnostics = check(fixture.path()).expect_err("both malformed files must fail");
    assert_eq!(diagnostics.len(), 2);
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic.code == "crouzeix-textbook.contract.deserialize" && diagnostic.field == "$"
    }));
    assert!(diagnostics
        .iter()
        .all(|diagnostic| diagnostic.line.is_some() && diagnostic.column.is_some()));
    assert_eq!(
        diagnostics
            .iter()
            .filter_map(|diagnostic| diagnostic.path.as_deref())
            .collect::<BTreeSet<_>>(),
        [
            Path::new("content/crouzeix_textbook/coverage.json"),
            Path::new("content/crouzeix_textbook/exercises.json"),
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn v2_accepts_the_explicit_formal_mode_combination_table() {
    let cases = [
        (
            "proved-here exact",
            "proved-here",
            "exact",
            "reconstructible",
            true,
            None,
        ),
        (
            "proved-here pre-formal",
            "proved-here",
            "unmapped",
            "summary",
            true,
            None,
        ),
        (
            "proved-here with correspondence pending",
            "proved-here",
            "unmapped",
            "reconstructible",
            true,
            None,
        ),
        (
            "reexported proof",
            "reexported-proof",
            "exact",
            "reconstructible",
            true,
            Some("CrouzeixTextbook.Part01.underlying_proof"),
        ),
        (
            "reexported proof pre-formal",
            "reexported-proof",
            "unmapped",
            "summary",
            true,
            Some("CrouzeixTextbook.Part01.underlying_proof"),
        ),
        (
            "reexported proof with correspondence pending",
            "reexported-proof",
            "unmapped",
            "reconstructible",
            true,
            Some("CrouzeixTextbook.Part01.underlying_proof"),
        ),
        (
            "definition",
            "definition",
            "checkpoint",
            "not-applicable",
            true,
            None,
        ),
        (
            "reviewed definition with reconstructible exposition",
            "definition",
            "exact",
            "reconstructible",
            true,
            None,
        ),
        (
            "reviewed definition alias with definition provider",
            "definition",
            "exact",
            "reconstructible",
            true,
            Some("CrouzeixTextbook.Part01.underlying_definition"),
        ),
        (
            "checkpoint",
            "checkpoint",
            "checkpoint",
            "summary",
            true,
            None,
        ),
        (
            "checkpoint pre-formal",
            "checkpoint",
            "unmapped",
            "summary",
            false,
            None,
        ),
        (
            "informal",
            "informal",
            "not-applicable",
            "not-applicable",
            false,
            None,
        ),
    ];

    for (label, mode, lean, prose, declaration, underlying) in cases {
        let fixture = copy_v2_fixture();
        configure_first_theorem(fixture.path(), mode, lean, prose, declaration, underlying);
        check(fixture.path()).unwrap_or_else(|diagnostics| {
            panic!("allowed combination {label} failed: {diagnostics:#?}")
        });
    }
}

#[test]
fn v2_rejects_disallowed_formal_mode_combinations() {
    let cases = [
        (
            "proved-here not-applicable",
            "proved-here",
            "not-applicable",
            "reconstructible",
            true,
            None,
            "lean_correspondence_status",
        ),
        (
            "reexport without underlying proof",
            "reexported-proof",
            "exact",
            "reconstructible",
            true,
            None,
            "lean_declaration.underlying_declaration",
        ),
        (
            "reexport aliases itself",
            "reexported-proof",
            "exact",
            "reconstructible",
            true,
            Some("CrouzeixTextbook.Part01.matrix_column_is_basis_image"),
            "lean_declaration.underlying_declaration",
        ),
        (
            "definition without declaration",
            "definition",
            "checkpoint",
            "not-applicable",
            false,
            None,
            "lean_declaration",
        ),
        (
            "definition with proof prose status",
            "definition",
            "exact",
            "summary",
            true,
            None,
            "prose_proof_status",
        ),
        (
            "checkpoint claiming exact",
            "checkpoint",
            "exact",
            "summary",
            true,
            None,
            "lean_correspondence_status",
        ),
        (
            "checkpoint with reconstructible prose",
            "checkpoint",
            "checkpoint",
            "reconstructible",
            true,
            None,
            "prose_proof_status",
        ),
        (
            "informal with declaration",
            "informal",
            "not-applicable",
            "not-applicable",
            true,
            None,
            "lean_declaration",
        ),
    ];

    for (label, mode, lean, prose, declaration, underlying, field) in cases {
        let fixture = copy_v2_fixture();
        configure_first_theorem(fixture.path(), mode, lean, prose, declaration, underlying);
        let diagnostics = check(fixture.path()).expect_err(label);
        assert_has_diagnostic(
            &diagnostics,
            "crouzeix-textbook.contract.invalid-combination",
            Some("CFT-01-001"),
            field,
        );
    }
}

#[test]
fn v2_rejects_symlinked_nonregular_and_oversized_contract_inputs() {
    let symlinked = copy_v2_fixture();
    let coverage = symlinked
        .path()
        .join("content/crouzeix_textbook/coverage.json");
    let outside = symlinked.path().join("outside-coverage.json");
    fs::copy(&coverage, &outside).expect("outside coverage copy");
    fs::remove_file(&coverage).expect("remove fixture coverage");
    symlink(&outside, &coverage).expect("symlink fixture coverage");
    let diagnostics = check(symlinked.path()).expect_err("symlinked contract must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.input",
        None,
        "file",
    );

    let nonregular = copy_v2_fixture();
    let exercises = nonregular
        .path()
        .join("content/crouzeix_textbook/exercises.json");
    fs::remove_file(&exercises).expect("remove fixture exercises");
    fs::create_dir(&exercises).expect("directory at exercise contract path");
    let diagnostics = check(nonregular.path()).expect_err("directory contract must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.input",
        None,
        "file",
    );

    let oversized = copy_v2_fixture();
    let coverage = oversized
        .path()
        .join("content/crouzeix_textbook/coverage.json");
    let mut bytes = fs::read(&coverage).expect("coverage bytes");
    bytes.resize(1024 * 1024 + 1, b' ');
    fs::write(&coverage, bytes).expect("oversized coverage");
    let diagnostics = check(oversized.path()).expect_err("oversized contract must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.input",
        None,
        "file",
    );
    assert!(diagnostics[0].expected.contains("1048576 bytes"));
}

#[test]
fn v2_uses_stable_schema_and_uniqueness_diagnostic_codes() {
    let schema = copy_v2_fixture();
    mutate_fixture_json(schema.path(), "coverage.json", |value| {
        value["schema_version"] = json!("crouzeix-textbook-coverage/v3");
    });
    let diagnostics = check(schema.path()).expect_err("unknown schema must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.schema",
        None,
        "schema_version",
    );

    let theorem = copy_v2_fixture();
    mutate_fixture_json(theorem.path(), "coverage.json", |value| {
        value["items"][1]["item_id"] = json!("CFT-01-001");
    });
    let diagnostics = check(theorem.path()).expect_err("duplicate theorem must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.duplicate",
        Some("CFT-01-001"),
        "item_id",
    );

    let exercise = copy_v2_fixture();
    mutate_fixture_json(exercise.path(), "exercises.json", |value| {
        value["exercises"][1]["exercise_id"] = json!("CFT-01-E01");
    });
    let diagnostics = check(exercise.path()).expect_err("duplicate exercise must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.duplicate",
        Some("CFT-01-E01"),
        "exercise_id",
    );

    let solution = copy_v2_fixture();
    mutate_fixture_json(solution.path(), "exercises.json", |value| {
        value["exercises"][1]["lean_solution"]["declaration"] =
            value["exercises"][0]["lean_solution"]["declaration"].clone();
    });
    let diagnostics = check(solution.path()).expect_err("duplicate solution must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.duplicate",
        Some("CFT-01-E02"),
        "lean_solution.declaration",
    );
}

#[test]
fn v2_requires_solution_axioms_to_be_strictly_sorted_and_unique() {
    let fixture = copy_v2_fixture();
    mutate_fixture_json(fixture.path(), "exercises.json", |value| {
        value["exercises"][1]["lean_solution"]["axioms"] =
            json!(["propext", "Classical.choice", "propext"]);
    });
    let diagnostics = check(fixture.path()).expect_err("unsorted duplicate axioms must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.invalid-combination",
        Some("CFT-01-E02"),
        "lean_solution.axioms",
    );
}

#[test]
fn v2_reports_review_identity_and_declaration_semantics() {
    let review = copy_v2_fixture();
    mutate_fixture_json(review.path(), "coverage.json", |value| {
        value["items"][1]["review_status"]["source_id"] = json!("UNKNOWN-SOURCE");
    });
    let diagnostics = check(review.path()).expect_err("review source mismatch must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.invalid-combination",
        Some("CFT-01-002"),
        "review_status.source_id",
    );

    let chapter = copy_v2_fixture();
    mutate_fixture_json(chapter.path(), "coverage.json", |value| {
        value["items"][1]["chapter"] = json!(2);
    });
    let diagnostics = check(chapter.path()).expect_err("chapter identity mismatch must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.invalid-combination",
        Some("CFT-01-002"),
        "item_id",
    );

    let missing_declaration = copy_v2_fixture();
    configure_first_theorem(
        missing_declaration.path(),
        "proved-here",
        "exact",
        "reconstructible",
        false,
        None,
    );
    let diagnostics =
        check(missing_declaration.path()).expect_err("proved theorem needs declaration");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.invalid-combination",
        Some("CFT-01-001"),
        "lean_declaration",
    );

    let reexport = copy_v2_fixture();
    configure_first_theorem(
        reexport.path(),
        "reexported-proof",
        "exact",
        "reconstructible",
        true,
        None,
    );
    let diagnostics = check(reexport.path()).expect_err("reexport needs underlying proof");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.invalid-combination",
        Some("CFT-01-001"),
        "lean_declaration.underlying_declaration",
    );

    let informal = copy_v2_fixture();
    configure_first_theorem(
        informal.path(),
        "informal",
        "not-applicable",
        "not-applicable",
        true,
        None,
    );
    let diagnostics = check(informal.path()).expect_err("informal row cannot declare Lean");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.invalid-combination",
        Some("CFT-01-001"),
        "lean_declaration",
    );
}

#[test]
fn v2_accumulates_independent_semantic_diagnostics() {
    let fixture = copy_v2_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        value["items"][1]["chapter"] = json!(0);
        value["items"][1]["review_status"]["source_id"] = json!("UNKNOWN-SOURCE");
    });

    let diagnostics = check(fixture.path()).expect_err("semantic failures must accumulate");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.invalid-combination",
        Some("CFT-01-002"),
        "chapter",
    );
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.invalid-combination",
        Some("CFT-01-002"),
        "item_id",
    );
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.contract.invalid-combination",
        Some("CFT-01-002"),
        "review_status.source_id",
    );
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.review-source-id",
        Some("CFT-01-002"),
        "review_status.source_id",
    );
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.pedagogical-graph.edge-order",
        Some("CFT-01-002"),
        "pedagogical_prerequisites",
    );
    assert_eq!(diagnostics.len(), 5);
}

#[test]
fn evidence_rejects_unregistered_theorem_source_ids() {
    let fixture = copy_v2_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        let theorem = fixture_theorem_mut(value, "CFT-01-001");
        theorem["source_ids"] = json!(["UNKNOWN-SOURCE"]);
        theorem["review_status"]["source_id"] = json!("UNKNOWN-SOURCE");
    });
    let diagnostics = check(fixture.path()).expect_err("unknown theorem source must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.source-id",
        Some("CFT-01-001"),
        "source_ids",
    );
}

#[test]
fn evidence_rejects_an_unregistered_claim_source() {
    let fixture = copy_v2_fixture();
    mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
        markdown.replacen("|CROUZEIX-PACKET]]", "|UNKNOWN-SOURCE]]", 1)
    });
    let diagnostics = check_packet(fixture.path()).expect_err("unknown claim source must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.claim-source",
        Some("CFT-CL-001"),
        "Source",
    );
}

#[test]
fn evidence_rejects_zero_parsed_and_mixed_claim_source_identities() {
    for source in [".", "---", "not a source identity", "[[malformed-source"] {
        let fixture = copy_v2_fixture();
        mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
            markdown.replacen(
                "[[knowledge/crouzeix_textbook/source_registry#crouzeix-packet|CROUZEIX-PACKET]]",
                source,
                1,
            )
        });
        let diagnostics = check_packet(fixture.path()).expect_err("source must parse an identity");
        assert_has_diagnostic(
            &diagnostics,
            "crouzeix-textbook.evidence.claim-source",
            Some("CFT-CL-001"),
            "Source",
        );
    }

    let fixture = copy_v2_fixture();
    mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
        markdown.replacen(
            "[[knowledge/crouzeix_textbook/source_registry#crouzeix-packet|CROUZEIX-PACKET]]",
            "[[knowledge/crouzeix_textbook/source_registry#crouzeix-packet|CROUZEIX-PACKET]], [[knowledge/crouzeix_textbook/source_registry#unknown-source|UNKNOWN-SOURCE]]",
            1,
        )
    });
    let diagnostics = check_packet(fixture.path()).expect_err("every source must resolve");
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code == "crouzeix-textbook.evidence.claim-source"
                    && diagnostic.field == "Source"
            })
            .count(),
        1,
        "mixed source diagnostics: {diagnostics:#?}"
    );
}

#[test]
fn evidence_rejects_zero_parsed_priority_source_identity() {
    let fixture = copy_v2_fixture();
    mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
        markdown
            .replacen(
                "- Statement: The fixture has an inspectable compile receipt.",
                "- Priority claim: yes\n- Statement: The fixture has an inspectable compile receipt.",
                1,
            )
            .replacen(
                "- Locator: `fixture/Compile.lean:1`.",
                "- Locator: `fixture/Compile.lean:1`.\n- Priority evidence source: .\n- Priority evidence locator: Section 1.",
                1,
            )
    });
    let diagnostics = check_packet(fixture.path()).expect_err("priority source must parse");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.priority-route",
        Some("CFT-CL-001"),
        "Priority evidence source",
    );
    assert!(!diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "crouzeix-textbook.evidence.priority-route"
            && diagnostic.field == "Priority evidence locator"
    }));
}

#[test]
fn evidence_rejects_organizational_source_headings_without_source_record_bodies() {
    let fixture = copy_v2_fixture();
    mutate_fixture_markdown(fixture.path(), "source_registry.md", |markdown| {
        format!("{markdown}\n## REFERENCES\n\n- Role: organizational grouping.\n- Boundary: not a source record.\n")
    });
    mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
        markdown.replacen(
            "#crouzeix-packet|CROUZEIX-PACKET",
            "#references|REFERENCES",
            1,
        )
    });
    let diagnostics = check_packet(fixture.path()).expect_err("organization is not a source");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.source-id",
        Some("REFERENCES"),
        "Identity",
    );
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.claim-source",
        Some("CFT-CL-001"),
        "Source",
    );
}

#[test]
fn evidence_requires_source_wikilinks_to_resolve_the_registered_path_and_anchor() {
    for replacement in [
        "[[knowledge/crouzeix_textbook/reading_guide#crouzeix-packet|CROUZEIX-PACKET]]",
        "[[knowledge/crouzeix_textbook/source_registry#lax-2007|CROUZEIX-PACKET]]",
    ] {
        let fixture = copy_v2_fixture();
        mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
            markdown.replacen(
                "[[knowledge/crouzeix_textbook/source_registry#crouzeix-packet|CROUZEIX-PACKET]]",
                replacement,
                1,
            )
        });
        let diagnostics = check_packet(fixture.path()).expect_err("source route must resolve");
        assert_has_diagnostic(
            &diagnostics,
            "crouzeix-textbook.evidence.claim-source",
            Some("CFT-CL-001"),
            "Source",
        );
    }
}

#[test]
fn evidence_rejects_nonexistent_locator_files_and_heading_anchors() {
    for locator in [
        "`fixture/Missing.lean:1`.",
        "[[knowledge/crouzeix_textbook/reading_guide#missing-anchor|Missing]].",
    ] {
        let fixture = copy_v2_fixture();
        mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
            markdown.replacen("`fixture/Compile.lean:1`.", locator, 1)
        });
        let diagnostics = check_packet(fixture.path()).expect_err("locator route must resolve");
        assert_has_diagnostic(
            &diagnostics,
            "crouzeix-textbook.evidence.claim-locator",
            Some("CFT-CL-001"),
            "Locator",
        );
    }
}

#[test]
fn evidence_ignores_fake_fields_inside_code_and_preserves_multiline_fields() {
    let fenced = copy_v2_fixture();
    mutate_fixture_markdown(fenced.path(), "claim_evidence_ledger.md", |markdown| {
        markdown.replacen(
            "- Source: [[knowledge/crouzeix_textbook/source_registry#crouzeix-packet|CROUZEIX-PACKET]]",
            "```text\n- Source: [[knowledge/crouzeix_textbook/source_registry#crouzeix-packet|CROUZEIX-PACKET]]\n```",
            1,
        )
    });
    let diagnostics = check_packet(fenced.path()).expect_err("fenced field must be ignored");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.claim-entry",
        Some("CFT-CL-001"),
        "Source",
    );

    let multiline = copy_v2_fixture();
    mutate_fixture_markdown(multiline.path(), "claim_evidence_ledger.md", |markdown| {
        markdown
            .replacen(
                "- Statement: The fixture has an inspectable compile receipt.",
                "- Statement: The fixture has an inspectable\n  compile receipt.",
                1,
            )
            .replacen(
                "- Locator: `fixture/Compile.lean:1`.",
                "- Locator: Chapter III,\n  printed pages 19--31.",
                1,
            )
    });
    check_packet(multiline.path()).expect("multiline claim fields must validate");
}

#[test]
fn evidence_priority_detection_avoids_sequencing_and_rejects_same_route_bypasses() {
    for statement in [
        "The first-power estimate is inspectable.",
        "The first perturbation is recorded in the fixture.",
    ] {
        let fixture = copy_v2_fixture();
        mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
            markdown.replacen(
                "The fixture has an inspectable compile receipt.",
                statement,
                1,
            )
        });
        check_packet(fixture.path()).expect("sequencing language is not historical priority");
    }

    let same_route = copy_v2_fixture();
    mutate_fixture_markdown(same_route.path(), "claim_evidence_ledger.md", |markdown| {
        markdown
            .replacen(
                "- Statement: The fixture has an inspectable compile receipt.",
                "- Priority claim: yes\n- Statement: This was the first fixture compile receipt.",
                1,
            )
            .replacen(
                "- Locator: `fixture/Compile.lean:1`.",
                "- Locator: `fixture/Compile.lean:1`.\n- Priority evidence source: [[knowledge/crouzeix_textbook/source_registry#crouzeix-packet|CROUZEIX-PACKET]]\n- Priority evidence locator: `fixture/Compile.lean:1`.",
                1,
            )
    });
    let diagnostics = check_packet(same_route.path()).expect_err("priority route must be separate");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.priority-route",
        Some("CFT-CL-001"),
        "Priority evidence source",
    );
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.priority-route",
        Some("CFT-CL-001"),
        "Priority evidence locator",
    );
}

#[test]
fn evidence_detects_bounded_published_priority_language() {
    let fixture = copy_v2_fixture();
    mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
        markdown.replacen(
            "The fixture has an inspectable compile receipt.",
            "The first published compile receipt is recorded in the fixture.",
            1,
        )
    });
    let diagnostics = check_packet(fixture.path()).expect_err("published priority needs evidence");
    for field in ["Priority evidence source", "Priority evidence locator"] {
        assert_has_diagnostic(
            &diagnostics,
            "crouzeix-textbook.evidence.priority-route",
            Some("CFT-CL-001"),
            field,
        );
    }
}

#[test]
fn evidence_compares_priority_locators_by_resolved_route_identity() {
    let fixture = copy_v2_fixture();
    mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
        markdown
            .replacen(
                "- Statement: The fixture has an inspectable compile receipt.",
                "- Priority claim: yes\n- Statement: This was the first fixture compile receipt.",
                1,
            )
            .replacen(
                "- Locator: `fixture/Compile.lean:1`.",
                "- Locator: [[fixture/Compile.lean|Main display]].\n- Priority evidence source: [[knowledge/crouzeix_textbook/source_registry#jin-v4-audited|JIN-V4-AUDITED]]\n- Priority evidence locator: [Different display](fixture/Compile.lean).",
                1,
            )
    });
    let diagnostics = check_packet(fixture.path()).expect_err("route aliases are not separate");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.priority-route",
        Some("CFT-CL-001"),
        "Priority evidence locator",
    );
    assert!(!diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "crouzeix-textbook.evidence.priority-route"
            && diagnostic.field == "Priority evidence source"
    }));
}

#[test]
fn evidence_requires_clickable_source_registry_routes_for_main_and_priority_sources() {
    let main = copy_v2_fixture();
    mutate_fixture_markdown(main.path(), "claim_evidence_ledger.md", |markdown| {
        markdown.replacen(
            "[[knowledge/crouzeix_textbook/source_registry#crouzeix-packet|CROUZEIX-PACKET]]",
            "CROUZEIX-PACKET",
            1,
        )
    });
    let diagnostics = check_packet(main.path()).expect_err("bare main source must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.claim-source",
        Some("CFT-CL-001"),
        "Source",
    );

    let priority = copy_v2_fixture();
    mutate_fixture_markdown(priority.path(), "claim_evidence_ledger.md", |markdown| {
        markdown
            .replacen(
                "- Statement: The fixture has an inspectable compile receipt.",
                "- Priority claim: yes\n- Statement: The fixture has an inspectable compile receipt.",
                1,
            )
            .replacen(
                "- Locator: `fixture/Compile.lean:1`.",
                "- Locator: `fixture/Compile.lean:1`.\n- Priority evidence source: JIN-V4-AUDITED\n- Priority evidence locator: Section 2.",
                1,
            )
    });
    let diagnostics = check_packet(priority.path()).expect_err("bare priority source must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.priority-route",
        Some("CFT-CL-001"),
        "Priority evidence source",
    );
}

#[test]
fn evidence_rejects_each_unsupported_or_unparseable_mixed_locator_component() {
    for locator in [
        "[[knowledge/crouzeix_textbook/reading_guide#fixture-locator|Valid]], [external](https://example.com).",
        "[[knowledge/crouzeix_textbook/reading_guide#fixture-locator|Valid]], `fixture/UnsupportedRoute`.",
        "[[knowledge/crouzeix_textbook/reading_guide#fixture-locator|Valid]], `CrouzeixTextbook.Part06.unresolved_declaration`.",
        "[[knowledge/crouzeix_textbook/reading_guide#fixture-locator|Valid]], [broken](.",
    ] {
        let fixture = copy_v2_fixture();
        mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
            markdown.replacen("`fixture/Compile.lean:1`.", locator, 1)
        });
        let diagnostics = check_packet(fixture.path()).expect_err("every locator component matters");
        assert_has_diagnostic(
            &diagnostics,
            "crouzeix-textbook.evidence.claim-locator",
            Some("CFT-CL-001"),
            "Locator",
        );
    }
}

#[test]
fn evidence_priority_detection_ignores_ordinary_mathematical_ordering() {
    for statement in [
        "The first basis vector is fixed by the example.",
        "The first proof step expands the definition.",
    ] {
        let fixture = copy_v2_fixture();
        mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
            markdown.replacen(
                "The fixture has an inspectable compile receipt.",
                statement,
                1,
            )
        });
        check_packet(fixture.path()).expect("ordinary ordering is not historical priority");
    }
}

#[test]
fn evidence_duplicate_field_diagnostic_points_to_the_duplicate_occurrence() {
    let fixture = copy_v2_fixture();
    mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
        markdown.replacen(
            "- Class: `EVIDENCE`",
            "- Class: `EVIDENCE`\n- Class: `EVIDENCE`",
            1,
        )
    });
    let ledger = fs::read_to_string(
        fixture
            .path()
            .join("knowledge/crouzeix_textbook/claim_evidence_ledger.md"),
    )
    .expect("fixture ledger");
    let duplicate_line = ledger
        .lines()
        .enumerate()
        .filter(|(_, line)| line.starts_with("- Class:"))
        .nth(1)
        .map(|(index, _)| index as u32 + 1)
        .expect("duplicate class field");
    let diagnostics = check_packet(fixture.path()).expect_err("duplicate field must fail");
    let duplicate = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code == "crouzeix-textbook.evidence.claim-entry"
                && diagnostic.field == "Class"
                && diagnostic.observed == "duplicate"
        })
        .expect("duplicate diagnostic");
    assert_eq!(duplicate.line, Some(duplicate_line));
    assert_eq!(duplicate.column, Some(1));
}

#[test]
fn evidence_accumulates_missing_registry_and_ledger_inputs() {
    let fixture = copy_v2_fixture();
    for name in ["source_registry.md", "claim_evidence_ledger.md"] {
        fs::remove_file(
            fixture
                .path()
                .join("knowledge/crouzeix_textbook")
                .join(name),
        )
        .expect("remove fixture input");
    }
    let diagnostics = check_packet(fixture.path()).expect_err("both missing inputs must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.source-id",
        None,
        "source_registry",
    );
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.claim-entry",
        None,
        "claim_ledger",
    );
}

#[test]
fn evidence_rejects_unregistered_review_source_even_when_it_is_also_unlisted() {
    let fixture = copy_v2_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        fixture_theorem_mut(value, "CFT-01-001")["review_status"]["source_id"] =
            json!("UNKNOWN-SOURCE");
    });
    let diagnostics = check(fixture.path()).expect_err("unknown review source must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.review-source-id",
        Some("CFT-01-001"),
        "review_status.source_id",
    );
}

#[test]
fn evidence_rejects_unsupported_claim_classes() {
    for unsupported in ["DIRECT OBSERVATION", "CLAIM"] {
        let fixture = copy_v2_fixture();
        mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
            markdown.replacen("`EVIDENCE`", &format!("`{unsupported}`"), 1)
        });
        let diagnostics = check_packet(fixture.path()).expect_err("unsupported class must fail");
        assert_has_diagnostic(
            &diagnostics,
            "crouzeix-textbook.evidence.claim-class",
            Some("CFT-CL-001"),
            "Class",
        );
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "crouzeix-textbook.evidence.claim-class")
            .expect("claim-class diagnostic");
        let ledger = fs::read_to_string(
            fixture
                .path()
                .join("knowledge/crouzeix_textbook/claim_evidence_ledger.md"),
        )
        .expect("fixture ledger");
        let class_line = ledger
            .lines()
            .position(|line| line.starts_with("- Class:"))
            .expect("class field") as u32
            + 1;
        assert_eq!(diagnostic.line, Some(class_line));
        assert_eq!(diagnostic.column, Some(1));
    }
}

#[test]
fn evidence_rejects_missing_empty_and_unstable_claim_locators() {
    let mutations: [(&str, MarkdownMutation); 3] = [
        ("missing", |markdown| {
            markdown
                .lines()
                .filter(|line| !line.starts_with("- Locator:"))
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        }),
        ("empty", |markdown| {
            markdown.replacen("- Locator: `fixture/Compile.lean:1`.", "- Locator:", 1)
        }),
        ("unstable", |markdown| {
            markdown.replacen(
                "- Locator: `fixture/Compile.lean:1`.",
                "- Locator: latest document.",
                1,
            )
        }),
    ];
    for (label, mutate) in mutations {
        let fixture = copy_v2_fixture();
        mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", mutate);
        let diagnostics = check_packet(fixture.path()).expect_err(label);
        assert_has_diagnostic(
            &diagnostics,
            "crouzeix-textbook.evidence.claim-locator",
            Some("CFT-CL-001"),
            "Locator",
        );
    }
}

#[test]
fn evidence_locator_policy_accepts_specific_routes_and_rejects_vague_text() {
    let positives = [
        "`fixture/Compile.lean:12`.",
        "[[knowledge/crouzeix_textbook/reading_guide#fixture-locator|Fixture locator]].",
        "Chapter III, printed pages 19--31.",
        "Section 2.1.",
        "Equation (3.2).",
        "Theorem 4.",
        "Lemma 7.",
        "Table 2.",
        "Figure 5.",
        "Appendix A.3.",
        "fixture/Compile.lean#L12.",
    ];
    for locator in positives {
        let fixture = copy_v2_fixture();
        mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
            markdown.replacen("`fixture/Compile.lean:1`.", locator, 1)
        });
        check_packet(fixture.path())
            .unwrap_or_else(|diagnostics| panic!("specific locator {locator:?}: {diagnostics:#?}"));
    }

    for locator in [
        "current document.",
        "current page.",
        "above.",
        "below.",
        "here.",
        "TBD.",
        "unknown.",
        "missing.",
        ".",
        "---",
    ] {
        let fixture = copy_v2_fixture();
        mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
            markdown.replacen("`fixture/Compile.lean:1`.", locator, 1)
        });
        let diagnostics = check_packet(fixture.path()).expect_err("vague locator must fail");
        assert_has_diagnostic(
            &diagnostics,
            "crouzeix-textbook.evidence.claim-locator",
            Some("CFT-CL-001"),
            "Locator",
        );
    }
}

#[test]
fn evidence_rejects_priority_claim_without_separate_exact_evidence_route() {
    let fixture = copy_v2_fixture();
    mutate_fixture_markdown(fixture.path(), "claim_evidence_ledger.md", |markdown| {
        markdown.replacen(
            "The fixture has an inspectable compile receipt.",
            "The first published compile receipt is recorded in the fixture.",
            1,
        )
    });
    let diagnostics = check_packet(fixture.path()).expect_err("priority claim must be supported");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.priority-route",
        Some("CFT-CL-001"),
        "Priority evidence source",
    );
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.evidence.priority-route",
        Some("CFT-CL-001"),
        "Priority evidence locator",
    );
}

#[test]
fn pedagogical_graph_rejects_unknown_duplicate_and_self_edges() {
    let cases = [
        (
            "unknown",
            "crouzeix-textbook.pedagogical-graph.unknown-prerequisite",
            "CFT-UNKNOWN",
        ),
        (
            "duplicate",
            "crouzeix-textbook.pedagogical-graph.duplicate-edge",
            "CFT-29-001",
        ),
        (
            "self",
            "crouzeix-textbook.pedagogical-graph.self-edge",
            "CFT-30-001",
        ),
    ];
    for (label, code, prerequisite) in cases {
        let fixture = copy_v2_fixture();
        mutate_fixture_json(fixture.path(), "coverage.json", |value| {
            let row = fixture_theorem_mut(value, "CFT-30-001");
            row["pedagogical_prerequisites"] = if label == "duplicate" {
                json!([prerequisite, prerequisite])
            } else {
                json!([prerequisite])
            };
        });
        let diagnostics = check(fixture.path()).expect_err(label);
        assert_has_diagnostic(
            &diagnostics,
            code,
            Some("CFT-30-001"),
            "pedagogical_prerequisites",
        );
    }
}

#[test]
fn pedagogical_graph_structural_failures_do_not_cascade_into_provider_policy() {
    let unknown = copy_v2_fixture();
    mutate_fixture_json(unknown.path(), "coverage.json", |value| {
        fixture_theorem_mut(value, "CFT-30-001")["pedagogical_prerequisites"] =
            json!(["CFT-UNKNOWN"]);
    });
    let diagnostics = check(unknown.path()).expect_err("unknown prerequisite must fail");
    assert_eq!(diagnostics.len(), 1, "unknown cascade: {diagnostics:#?}");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.pedagogical-graph.unknown-prerequisite",
        Some("CFT-30-001"),
        "pedagogical_prerequisites",
    );

    let duplicate = copy_v2_fixture();
    mutate_fixture_json(duplicate.path(), "coverage.json", |value| {
        fixture_theorem_mut(value, "CFT-30-001")["pedagogical_prerequisites"] =
            json!(["CFT-29-001", "CFT-29-001"]);
    });
    let diagnostics = check(duplicate.path()).expect_err("duplicate prerequisite must fail");
    assert_eq!(diagnostics.len(), 1, "duplicate cascade: {diagnostics:#?}");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.pedagogical-graph.duplicate-edge",
        Some("CFT-30-001"),
        "pedagogical_prerequisites",
    );

    let cycle = copy_v2_fixture();
    mutate_fixture_json(cycle.path(), "coverage.json", |value| {
        fixture_theorem_mut(value, "CFT-01-001")["pedagogical_prerequisites"] =
            json!(["CFT-29-001"]);
    });
    let diagnostics = check(cycle.path()).expect_err("cycle must fail");
    assert_eq!(diagnostics.len(), 1, "cycle cascade: {diagnostics:#?}");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.pedagogical-graph.cycle",
        Some("CFT-01-001"),
        "pedagogical_prerequisites",
    );
}

#[test]
fn pedagogical_graph_allows_acyclic_same_chapter_semantic_order() {
    let fixture = copy_v2_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        fixture_theorem_mut(value, "CFT-01-001")["pedagogical_prerequisites"] =
            json!(["CFT-01-002"]);
        fixture_theorem_mut(value, "CFT-01-002")["pedagogical_prerequisites"] = json!([]);
    });
    check(fixture.path()).expect("acyclic same-chapter semantic edge must validate");
}

#[test]
fn pedagogical_graph_rejects_forward_and_swapped_chapter_edges() {
    let forward = copy_v2_fixture();
    mutate_fixture_json(forward.path(), "coverage.json", |value| {
        fixture_theorem_mut(value, "CFT-01-002")["pedagogical_prerequisites"] =
            json!(["CFT-29-001"]);
        fixture_theorem_mut(value, "CFT-29-001")["pedagogical_prerequisites"] =
            json!(["CFT-01-001"]);
    });
    let diagnostics = check(forward.path()).expect_err("forward chapter edge must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.pedagogical-graph.edge-order",
        Some("CFT-01-002"),
        "pedagogical_prerequisites",
    );

    let swapped = copy_v2_fixture();
    mutate_fixture_json(swapped.path(), "coverage.json", |value| {
        fixture_theorem_mut(value, "CFT-30-001")["pedagogical_prerequisites"] =
            json!(["CFT-31-001"]);
        fixture_theorem_mut(value, "CFT-31-001")["pedagogical_prerequisites"] =
            json!(["CFT-29-001"]);
    });
    let diagnostics = check(swapped.path()).expect_err("swapped provider order must fail");
    assert_has_diagnostic(
        &diagnostics,
        "crouzeix-textbook.pedagogical-graph.edge-order",
        Some("CFT-30-001"),
        "pedagogical_prerequisites",
    );
}

#[test]
fn pedagogical_graph_allows_forward_reader_links_only_for_summary_previews() {
    let fixture = copy_v2_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        let preview = fixture_theorem_mut(value, "CFT-01-001");
        preview["prose_proof_status"] = json!("summary");
        preview["pedagogical_prerequisites"] = json!(["CFT-29-001"]);
        fixture_theorem_mut(value, "CFT-29-001")["pedagogical_prerequisites"] = json!([]);
    });
    check(fixture.path()).expect("summary preview may point readers to a later proof");
}

#[test]
fn pedagogical_graph_reports_a_deterministic_cycle_path() {
    let fixture = copy_v2_fixture();
    mutate_fixture_json(fixture.path(), "coverage.json", |value| {
        fixture_theorem_mut(value, "CFT-01-001")["pedagogical_prerequisites"] =
            json!(["CFT-29-001"]);
    });
    let diagnostics = check(fixture.path()).expect_err("cycle must fail");
    let cycle = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "crouzeix-textbook.pedagogical-graph.cycle")
        .expect("cycle diagnostic");
    assert_eq!(cycle.identity.as_deref(), Some("CFT-01-001"));
    assert_eq!(cycle.field, "pedagogical_prerequisites");
    assert_eq!(
        cycle.observed,
        "CFT-01-001 -> CFT-29-001 -> CFT-01-002 -> CFT-01-001"
    );
}

#[test]
fn pedagogical_graph_generic_validation_accepts_a_structural_complete_graph() {
    let fixture = copy_v2_fixture();
    let mut rows = Vec::new();
    let mut markdown = String::from(
        "---\nid: cft-chapter-02-vector-spaces-and-subspaces\ntitle: Generated graph\ntype: textbook-chapter\nstatus: active\ntags: [crouzeix-textbook]\nconfidence: high\n---\n\n# Generated graph\n",
    );
    for chapter in 1..=35u32 {
        for index in 1..=6u32 {
            let item_id = format!("CFT-{chapter:02}-{index:03}");
            let anchor = format!("generated-{chapter:02}-{index:03}");
            let prerequisites = if chapter == 1 && index == 1 {
                Vec::new()
            } else if index > 1 {
                vec![format!("CFT-{chapter:02}-{:03}", index - 1)]
            } else {
                match chapter {
                    30 | 33 => vec!["CFT-29-006".to_owned()],
                    35 => vec!["CFT-32-006".to_owned(), "CFT-34-006".to_owned()],
                    _ => vec![format!("CFT-{:02}-006", chapter - 1)],
                }
            };
            let source_ids = match chapter {
                29 => vec!["CROUZEIX-PACKET"],
                30..=32 => vec!["JIN-V4-AUDITED"],
                33..=34 => vec!["LS-ARXIV-V1"],
                35 => vec!["JIN-V4-AUDITED", "LS-ARXIV-V1"],
                _ => vec!["LAX-2007"],
            };
            rows.push(json!({
                "item_id": item_id,
                "chapter": chapter,
                "kind": "definition",
                "prose_path": "knowledge/crouzeix_textbook/part_01_linear_structure/02_vector_spaces_and_subspaces.md",
                "anchor": anchor,
                "source_ids": source_ids,
                "pedagogical_prerequisites": prerequisites,
                "publication_status": "draft",
                "prose_proof_status": "not-applicable",
                "lean_correspondence_status": "unmapped",
                "review_status": {"source_id": source_ids[0], "status": "source-checked"},
                "formal_mode": "definition",
                "lean_declaration": null
            }));
            markdown.push_str(&generated_theorem_card(&anchor));
        }
    }
    mutate_fixture_json(fixture.path(), "coverage.json", |coverage| {
        coverage["items"] = Value::Array(rows);
    });
    fs::write(
        fixture.path().join(
            "knowledge/crouzeix_textbook/part_01_linear_structure/02_vector_spaces_and_subspaces.md",
        ),
        markdown,
    )
    .expect("write generated theorem cards");

    let contracts = check(fixture.path()).expect("generic validation must not guess semantics");
    assert_eq!(contracts.theorems().len(), 210);
}

#[test]
fn canonical_pedagogical_graph_matches_the_reviewed_roster_and_policy() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let rows = array(&coverage, "items", "coverage");
    let chapter_of = |item_id: &str| {
        item_id[4..6]
            .parse::<u64>()
            .unwrap_or_else(|error| panic!("invalid CFT chapter in {item_id}: {error}"))
    };
    let mut roots = BTreeSet::new();
    let mut chapter_policy = BTreeMap::<u64, BTreeSet<u64>>::new();
    let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
    let mut roster = rows.iter().collect::<Vec<_>>();
    roster.sort_by_key(|row| string(row, "item_id", "coverage row"));
    let mut serialized_roster = String::new();
    for row in rows {
        let item_id = string(row, "item_id", "coverage row");
        let prerequisites = array(row, "pedagogical_prerequisites", item_id);
        if prerequisites.is_empty() {
            roots.insert(item_id.to_owned());
        }
        graph.insert(
            item_id.to_owned(),
            prerequisites
                .iter()
                .map(|value| value.as_str().expect("prerequisite string").to_owned())
                .collect(),
        );
        for prerequisite in prerequisites {
            let prerequisite = prerequisite.as_str().expect("prerequisite string");
            let prerequisite_chapter = chapter_of(prerequisite);
            if prerequisite_chapter != integer(row, "chapter", item_id) {
                chapter_policy
                    .entry(integer(row, "chapter", item_id))
                    .or_default()
                    .insert(prerequisite_chapter);
            }
        }
    }
    for row in roster {
        let item_id = string(row, "item_id", "coverage row");
        let prerequisites = array(row, "pedagogical_prerequisites", item_id)
            .iter()
            .map(|value| value.as_str().expect("prerequisite string"))
            .collect::<Vec<_>>();
        serialized_roster.push_str(item_id);
        serialized_roster.push('=');
        serialized_roster.push_str(&prerequisites.join(","));
        serialized_roster.push('\n');
    }
    assert_eq!(
        roots,
        BTreeSet::from([
            "CFT-01-001".to_owned(),
            "CFT-02-001".to_owned(),
            "CFT-02-002".to_owned(),
            "CFT-07-006".to_owned(),
            "CFT-17-001".to_owned(),
            "CFT-18-001".to_owned(),
            "CFT-27-001".to_owned(),
        ])
    );
    let reviewed_chapter_policy = BTreeMap::from([
        (3, BTreeSet::from([1, 2])),
        (4, BTreeSet::from([1, 2, 3])),
        (5, BTreeSet::from([1, 4])),
        (6, BTreeSet::from([1, 2, 3, 4, 5])),
        (7, BTreeSet::from([1, 2, 4])),
        (8, BTreeSet::from([1, 2, 5, 7])),
        (9, BTreeSet::from([4, 7, 8])),
        (10, BTreeSet::from([1, 4, 5, 7])),
        (11, BTreeSet::from([1, 3, 4, 7, 10])),
        (12, BTreeSet::from([7, 10, 11])),
        (13, BTreeSet::from([7])),
        (14, BTreeSet::from([6, 9, 11])),
        (15, BTreeSet::from([6, 12, 14])),
        (16, BTreeSet::from([6, 15])),
        (17, BTreeSet::from([6, 13, 16])),
        (18, BTreeSet::from([8, 14])),
        (19, BTreeSet::from([6, 9])),
        (20, BTreeSet::from([6, 7, 9, 13])),
        (21, BTreeSet::from([7, 13, 17, 20])),
        (22, BTreeSet::from([1, 7, 8, 9, 15])),
        (23, BTreeSet::from([3, 6, 7, 9])),
        (24, BTreeSet::from([7, 8, 14])),
        (25, BTreeSet::from([7, 9, 12, 20])),
        (26, BTreeSet::from([8, 15, 20, 25])),
        (27, BTreeSet::from([9, 22])),
        (28, BTreeSet::from([9, 13, 15, 16, 18, 22, 26, 30, 32])),
        (29, BTreeSet::from([1, 9, 13, 16, 20])),
        (30, BTreeSet::from([8, 24, 28, 29])),
        (31, BTreeSet::from([18, 30])),
        (32, BTreeSet::from([16, 17, 29, 30, 31])),
        (33, BTreeSet::from([9, 23, 27, 29])),
        (34, BTreeSet::from([22, 23, 28, 33])),
        (35, BTreeSet::from([17, 20, 21, 29, 32, 34])),
        (36, BTreeSet::from([21, 26, 28, 29, 33, 34])),
    ]);
    assert_eq!(chapter_policy, reviewed_chapter_policy);
    assert_eq!(
        format!("{:x}", Sha256::digest(serialized_roster.as_bytes())),
        "f3102fdcea356e8c8db0fe65a5a21107c90a02d9724d101fb9c2d175c464cc6b",
        "the reviewed 216-row prerequisite roster changed"
    );

    let prerequisites = |item_id: &str| {
        rows.iter()
            .find(|row| string(row, "item_id", "coverage row") == item_id)
            .map(|row| {
                array(row, "pedagogical_prerequisites", item_id)
                    .iter()
                    .map(|value| value.as_str().expect("prerequisite string").to_owned())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_else(|| panic!("missing coverage row {item_id}"))
    };
    for (item_id, required) in [
        ("CFT-30-005", ["CFT-30-003", "CFT-30-004"]),
        ("CFT-31-006", ["CFT-31-003", "CFT-31-005"]),
        ("CFT-33-003", ["CFT-33-001", "CFT-33-002"]),
        ("CFT-34-003", ["CFT-34-002", "CFT-34-004"]),
        ("CFT-34-001", ["CFT-34-003", "CFT-34-005"]),
        ("CFT-35-006", ["CFT-32-006", "CFT-35-001"]),
    ] {
        let actual = prerequisites(item_id);
        for prerequisite in required {
            assert!(
                actual.contains(prerequisite),
                "{item_id} must use {prerequisite}: {actual:?}"
            );
        }
    }
    assert!(prerequisites("CFT-30-001").contains("CFT-29-002"));
    assert!(prerequisites("CFT-33-001").contains("CFT-29-002"));
    assert!(prerequisites("CFT-17-001").is_empty());
    assert!(prerequisites("CFT-18-001").is_empty());

    fn reaches(
        graph: &BTreeMap<String, BTreeSet<String>>,
        start: &str,
        target: &str,
        visited: &mut BTreeSet<String>,
    ) -> bool {
        if !visited.insert(start.to_owned()) {
            return false;
        }
        start == target
            || graph.get(start).is_some_and(|prerequisites| {
                prerequisites
                    .iter()
                    .any(|next| reaches(graph, next, target, visited))
            })
    }
    assert!(reaches(
        &graph,
        "CFT-32-006",
        "CFT-30-001",
        &mut BTreeSet::new()
    ));
    assert!(reaches(
        &graph,
        "CFT-34-006",
        "CFT-33-001",
        &mut BTreeSet::new()
    ));
    assert!(reaches(
        &graph,
        "CFT-35-006",
        "CFT-32-006",
        &mut BTreeSet::new()
    ));
    assert!(reaches(
        &graph,
        "CFT-35-006",
        "CFT-34-006",
        &mut BTreeSet::new()
    ));
    for row in rows
        .iter()
        .filter(|row| matches!(integer(row, "chapter", "coverage row"), 33 | 34))
    {
        let item_id = string(row, "item_id", "coverage row");
        for jin in graph
            .keys()
            .filter(|item_id| matches!(chapter_of(item_id), 30..=32))
        {
            assert!(
                !reaches(&graph, item_id, jin, &mut BTreeSet::new()),
                "LS route {item_id} crosses into Jin route at {jin}"
            );
        }
    }
}

#[test]
fn canonical_theorem_rows_have_216_unique_resolving_targets() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let mut targets = BTreeSet::new();
    for row in array(&coverage, "items", "coverage") {
        let item_id = string(row, "item_id", "coverage row");
        let target = (
            string(row, "prose_path", item_id).to_owned(),
            string(row, "anchor", item_id).to_owned(),
        );
        assert!(
            targets.insert(target.clone()),
            "duplicate target {target:?}"
        );
    }
    assert_eq!(targets.len(), 216);
    check(&workspace_root()).expect("all unique theorem targets must resolve");
}

const SUPPORT_DOCUMENTS: [&str; 11] = [
    "harp_mathematical_audit.md",
    "harp_finite_horizon_remainder.md",
    "crouzeix_textbook_index.md",
    "reading_guide.md",
    "notation_and_glossary.md",
    "theorem_dependency_map.md",
    "exercise_index.md",
    "lean_coverage_ledger.md",
    "source_registry.md",
    "claim_evidence_ledger.md",
    "status_and_scope.md",
];

const CHAPTERS: [&str; 36] = [
    "part_01_linear_structure/01_objects_and_representations.md",
    "part_01_linear_structure/02_vector_spaces_and_subspaces.md",
    "part_01_linear_structure/03_linear_maps_and_exact_structure.md",
    "part_01_linear_structure/04_coordinates_and_duality.md",
    "part_01_linear_structure/05_determinants_trace_and_exterior_algebra.md",
    "part_01_linear_structure/06_eigenvalues_and_polynomial_algebra.md",
    "part_02_geometry_and_calculus/07_inner_product_spaces.md",
    "part_02_geometry_and_calculus/08_positive_operators_and_gram_geometry.md",
    "part_02_geometry_and_calculus/09_operator_norms_and_singular_values.md",
    "part_02_geometry_and_calculus/10_multilinear_maps_and_tensors.md",
    "part_02_geometry_and_calculus/11_differentiation_as_linear_approximation.md",
    "part_02_geometry_and_calculus/12_differential_forms_and_stokes.md",
    "part_03_analysis_and_complex_functions/13_metric_and_normed_spaces.md",
    "part_03_analysis_and_complex_functions/14_sequences_and_series_of_operators.md",
    "part_03_analysis_and_complex_functions/15_complex_differentiability.md",
    "part_03_analysis_and_complex_functions/16_consequences_of_cauchy_theory.md",
    "part_03_analysis_and_complex_functions/17_functions_of_matrices_and_operators.md",
    "part_03_analysis_and_complex_functions/18_positive_real_analytic_functions.md",
    "part_04_operator_theory/19_normality_and_nonnormality.md",
    "part_04_operator_theory/20_numerical_range.md",
    "part_04_operator_theory/21_spectral_sets.md",
    "part_04_operator_theory/22_positive_and_completely_positive_maps.md",
    "part_04_operator_theory/23_compression_and_dilation.md",
    "part_04_operator_theory/24_gramians_and_ordered_matrix_inequalities.md",
    "part_05_crouzeix_machinery/25_convex_boundaries_and_cauchy_layers.md",
    "part_05_crouzeix_machinery/26_double_layer_map.md",
    "part_05_crouzeix_machinery/27_one_plus_sqrt_two_barrier.md",
    "part_05_crouzeix_machinery/28_complete_power_family.md",
    "part_06_constant_two_routes/29_crouzeix_problem_and_sharpness.md",
    "part_06_constant_two_routes/30_jin_positive_real_completion.md",
    "part_06_constant_two_routes/31_jin_correction_cancellation.md",
    "part_06_constant_two_routes/32_jin_constant_two_endpoint.md",
    "part_06_constant_two_routes/33_lorist_schwenninger_perturbation_lemma.md",
    "part_06_constant_two_routes/34_lorist_schwenninger_realization.md",
    "part_06_constant_two_routes/35_comparison_verification_and_boundaries.md",
    "part_06_constant_two_routes/36_harp_finite_horizon_proof.md",
];

const REQUIRED_HEADINGS: [&str; 8] = [
    "## Opening problem",
    "## Conceptual model",
    "## Formal development",
    "## Worked examples",
    "## ML bridge",
    "## Lean translation",
    "## Exercises",
    "## Synthesis and forward dependencies",
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn contracts_root() -> PathBuf {
    workspace_root().join("content/crouzeix_textbook")
}

fn packet_root() -> PathBuf {
    workspace_root().join("knowledge/crouzeix_textbook")
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn object<'a>(value: &'a Value, context: &str) -> &'a Map<String, Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("{context} must be an object"))
}

fn array<'a>(value: &'a Value, key: &str, context: &str) -> &'a Vec<Value> {
    value
        .get(key)
        .unwrap_or_else(|| panic!("{context} is missing `{key}`"))
        .as_array()
        .unwrap_or_else(|| panic!("{context}.{key} must be an array"))
}

fn string<'a>(value: &'a Value, key: &str, context: &str) -> &'a str {
    value
        .get(key)
        .unwrap_or_else(|| panic!("{context} is missing `{key}`"))
        .as_str()
        .unwrap_or_else(|| panic!("{context}.{key} must be a string"))
}

fn integer(value: &Value, key: &str, context: &str) -> u64 {
    value
        .get(key)
        .unwrap_or_else(|| panic!("{context} is missing `{key}`"))
        .as_u64()
        .unwrap_or_else(|| panic!("{context}.{key} must be a nonnegative integer"))
}

fn assert_exact_keys(object: &Map<String, Value>, expected: &[&str], context: &str) {
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "{context} has missing or unknown fields");
}

fn actual_markdown_files(root: &Path) -> BTreeSet<String> {
    fn visit(root: &Path, current: &Path, output: &mut BTreeSet<String>) {
        for entry in fs::read_dir(current)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", current.display()))
        {
            let entry = entry.expect("directory entry");
            let path = entry.path();
            let metadata = entry.metadata().expect("entry metadata");
            if metadata.is_dir() {
                visit(root, &path, output);
            } else if path.extension().is_some_and(|extension| extension == "md") {
                output.insert(
                    path.strip_prefix(root)
                        .expect("packet-relative path")
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }

    let mut output = BTreeSet::new();
    visit(root, root, &mut output);
    output
}

#[test]
fn textbook_packet_has_exact_support_and_chapter_roster() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    assert!(packet_root().is_dir(), "textbook packet root is missing");

    let actual = actual_markdown_files(&packet_root());
    for support in SUPPORT_DOCUMENTS {
        assert!(
            actual.contains(support),
            "missing support document `{support}`"
        );
    }
    let present = CHAPTERS
        .iter()
        .filter(|chapter| actual.contains(**chapter))
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(
        present, CHAPTERS,
        "the active packet has exactly 36 chapters"
    );
    for chapter_number in 1..=CHAPTERS.len() {
        let coverage_count = array(&coverage, "items", "coverage")
            .iter()
            .filter(|item| integer(item, "chapter", "coverage item") == chapter_number as u64)
            .count();
        assert_eq!(coverage_count, 6, "Chapter {chapter_number} coverage count");
        let exercise_count = array(&exercises, "exercises", "exercises")
            .iter()
            .filter(|item| integer(item, "chapter", "exercise") == chapter_number as u64)
            .count();
        assert_eq!(exercise_count, 6, "Chapter {chapter_number} exercise count");
    }
    let expected = SUPPORT_DOCUMENTS
        .iter()
        .copied()
        .chain(present.iter().copied())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual.iter().map(String::as_str).collect::<BTreeSet<_>>(),
        expected,
        "packet contains an unknown or missing Markdown document"
    );
    for document in &actual {
        let text = fs::read_to_string(packet_root().join(document))
            .unwrap_or_else(|error| panic!("failed to read {document}: {error}"));
        assert!(
            text.contains("\nstatus: active\n"),
            "active packet document {document} is not active"
        );
    }

    for chapter in present {
        let text = fs::read_to_string(packet_root().join(chapter)).expect("chapter text");
        let mut cursor = 0;
        for heading in REQUIRED_HEADINGS {
            let offset = text[cursor..]
                .find(heading)
                .unwrap_or_else(|| panic!("{chapter} is missing heading `{heading}`"));
            cursor += offset + heading.len();
        }
        assert!(
            text.contains("Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index"),
            "{chapter} is missing book navigation"
        );
        assert!(
            text.contains("Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-"),
            "{chapter} is missing part navigation"
        );
        assert!(
            text.contains("Previous:"),
            "{chapter} is missing previous navigation"
        );
        assert!(
            text.contains("Next:"),
            "{chapter} is missing next navigation"
        );
    }
}

#[test]
fn textbook_contracts_are_strict_and_cross_referenced() {
    let coverage_value = read_json(&contracts_root().join("coverage.json"));
    let exercise_value = read_json(&contracts_root().join("exercises.json"));
    assert_exact_keys(
        object(&coverage_value, "coverage"),
        &["schema_version", "items"],
        "coverage",
    );
    assert_eq!(
        string(&coverage_value, "schema_version", "coverage"),
        "crouzeix-textbook-coverage/v2"
    );
    assert_exact_keys(
        object(&exercise_value, "exercises"),
        &["schema_version", "exercises"],
        "exercises",
    );
    assert_eq!(
        string(&exercise_value, "schema_version", "exercises"),
        "crouzeix-textbook-exercises/v2"
    );

    let contracts = check(&workspace_root())
        .unwrap_or_else(|diagnostics| panic!("canonical v2 contracts failed: {diagnostics:#?}"));
    assert_eq!(contracts.theorems().len(), 216);
    assert_eq!(contracts.exercises().len(), 216);

    let expected_theorems = (1..=36)
        .flat_map(|chapter| (1..=6).map(move |index| format!("CFT-{chapter:02}-{index:03}")))
        .collect::<BTreeSet<_>>();
    let actual_theorems = contracts
        .theorems()
        .iter()
        .map(|row| row.item_id().to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_theorems, expected_theorems);

    let expected_exercises = (1..=36)
        .flat_map(|chapter| (1..=6).map(move |index| format!("CFT-{chapter:02}-E{index:02}")))
        .collect::<BTreeSet<_>>();
    let actual_exercises = contracts
        .exercises()
        .iter()
        .map(|row| row.exercise_id().to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_exercises, expected_exercises);

    let prerequisites = |item_id: &str| {
        array(&coverage_value, "items", "coverage")
            .iter()
            .find(|row| string(row, "item_id", "coverage row") == item_id)
            .map(|row| {
                array(row, "pedagogical_prerequisites", item_id)
                    .iter()
                    .map(|value| value.as_str().expect("prerequisite string").to_owned())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_else(|| panic!("missing coverage row {item_id}"))
    };
    assert_eq!(
        prerequisites("CFT-30-001"),
        BTreeSet::from(["CFT-28-005".to_owned(), "CFT-29-002".to_owned()])
    );
    assert_eq!(
        prerequisites("CFT-33-001"),
        BTreeSet::from([
            "CFT-23-004".to_owned(),
            "CFT-23-005".to_owned(),
            "CFT-29-002".to_owned(),
        ])
    );
    assert_eq!(
        prerequisites("CFT-35-001"),
        BTreeSet::from(["CFT-29-003".to_owned(), "CFT-34-006".to_owned()])
    );
    assert_eq!(
        prerequisites("CFT-35-006"),
        BTreeSet::from([
            "CFT-29-006".to_owned(),
            "CFT-32-006".to_owned(),
            "CFT-35-001".to_owned(),
        ])
    );
}

#[test]
fn truthful_v2_active_state_remains_explicitly_correspondence_incomplete() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let contracts = check(&workspace_root())
        .unwrap_or_else(|diagnostics| panic!("canonical v2 contracts failed: {diagnostics:#?}"));

    let mode_counts = contracts
        .theorems()
        .iter()
        .fold([0_usize; 4], |mut counts, row| {
            match row.formal_mode() {
                FormalMode::ProvedHere => counts[0] += 1,
                FormalMode::ReexportedProof => counts[1] += 1,
                FormalMode::Checkpoint => counts[2] += 1,
                FormalMode::Definition => counts[3] += 1,
                FormalMode::Informal => panic!("canonical baseline contains an informal row"),
            }
            counts
        });
    assert_eq!(mode_counts, [56, 115, 39, 6]);
    assert!(contracts
        .theorems()
        .iter()
        .all(|row| row.publication_status() == PublicationStatus::Active));
    assert_eq!(
        contracts
            .theorems()
            .iter()
            .filter(|row| row.prose_proof_status() == ProseProofStatus::Summary)
            .count(),
        113
    );
    assert_eq!(
        contracts
            .theorems()
            .iter()
            .filter(|row| row.prose_proof_status() == ProseProofStatus::NotApplicable)
            .count(),
        2
    );
    assert_eq!(
        contracts
            .theorems()
            .iter()
            .filter(|row| row.lean_correspondence_status() == LeanCorrespondenceStatus::Exact)
            .count(),
        108
    );
    assert_eq!(
        contracts
            .theorems()
            .iter()
            .filter(|row| {
                row.lean_correspondence_status() == LeanCorrespondenceStatus::Checkpoint
            })
            .count(),
        39
    );

    let solved = array(&exercises, "exercises", "exercises")
        .iter()
        .filter(|row| {
            row.get("lean_solution")
                .is_some_and(|solution| !solution.is_null())
        })
        .map(|row| string(row, "exercise_id", "exercise").to_owned())
        .collect::<BTreeSet<_>>();
    let mut expected_solved = (1..=6)
        .flat_map(|chapter| (1..=6).map(move |index| format!("CFT-{chapter:02}-E{index:02}")))
        .collect::<BTreeSet<_>>();
    expected_solved.extend([
        "CFT-25-E01".to_owned(),
        "CFT-25-E02".to_owned(),
        "CFT-25-E03".to_owned(),
        "CFT-25-E04".to_owned(),
        "CFT-25-E05".to_owned(),
        "CFT-25-E06".to_owned(),
        "CFT-26-E01".to_owned(),
        "CFT-26-E02".to_owned(),
        "CFT-26-E03".to_owned(),
        "CFT-26-E04".to_owned(),
        "CFT-26-E05".to_owned(),
        "CFT-26-E06".to_owned(),
        "CFT-27-E01".to_owned(),
        "CFT-27-E02".to_owned(),
        "CFT-27-E03".to_owned(),
        "CFT-27-E04".to_owned(),
        "CFT-27-E05".to_owned(),
        "CFT-27-E06".to_owned(),
        "CFT-28-E01".to_owned(),
        "CFT-28-E02".to_owned(),
        "CFT-28-E03".to_owned(),
        "CFT-28-E04".to_owned(),
        "CFT-28-E05".to_owned(),
        "CFT-28-E06".to_owned(),
        "CFT-29-E01".to_owned(),
        "CFT-29-E02".to_owned(),
        "CFT-29-E03".to_owned(),
        "CFT-29-E04".to_owned(),
        "CFT-29-E05".to_owned(),
        "CFT-29-E06".to_owned(),
        "CFT-30-E01".to_owned(),
        "CFT-30-E02".to_owned(),
        "CFT-30-E03".to_owned(),
        "CFT-30-E04".to_owned(),
        "CFT-30-E05".to_owned(),
        "CFT-30-E06".to_owned(),
        "CFT-31-E01".to_owned(),
        "CFT-31-E02".to_owned(),
        "CFT-31-E03".to_owned(),
        "CFT-31-E04".to_owned(),
        "CFT-31-E05".to_owned(),
        "CFT-31-E06".to_owned(),
        "CFT-32-E01".to_owned(),
        "CFT-32-E02".to_owned(),
        "CFT-32-E03".to_owned(),
        "CFT-32-E04".to_owned(),
        "CFT-32-E05".to_owned(),
        "CFT-32-E06".to_owned(),
        "CFT-33-E01".to_owned(),
        "CFT-33-E02".to_owned(),
        "CFT-33-E03".to_owned(),
        "CFT-33-E04".to_owned(),
        "CFT-33-E05".to_owned(),
        "CFT-33-E06".to_owned(),
        "CFT-34-E01".to_owned(),
        "CFT-34-E02".to_owned(),
        "CFT-34-E03".to_owned(),
        "CFT-34-E04".to_owned(),
        "CFT-34-E05".to_owned(),
        "CFT-34-E06".to_owned(),
        "CFT-35-E01".to_owned(),
        "CFT-35-E02".to_owned(),
        "CFT-35-E03".to_owned(),
        "CFT-35-E04".to_owned(),
        "CFT-35-E05".to_owned(),
        "CFT-35-E06".to_owned(),
        "CFT-36-E01".to_owned(),
        "CFT-36-E02".to_owned(),
        "CFT-36-E03".to_owned(),
        "CFT-36-E04".to_owned(),
        "CFT-36-E05".to_owned(),
        "CFT-36-E06".to_owned(),
    ]);
    assert_eq!(solved, expected_solved);

    let maintained_status = fs::read_to_string(packet_root().join("status_and_scope.md"))
        .expect("maintained status and scope");
    let maintained_status = maintained_status
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for expected in [
        "Proof-exposition status: 101 coverage rows are now `reconstructible`; 113 remain `summary` and two other definition rows are `not-applicable`.",
        "Coverage rows: 216, comprising 56 `proved-here`, 115 `reexported-proof`, 39 `checkpoint`, and 6 `definition` rows.",
        "Exact-correspondence rows: 108.",
        "Distinct checked exercise solutions: six each in Chapters 1, 2, 3, 4, 5, 6, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, and 36. The other 108 exercise rows",
    ] {
        assert!(
            maintained_status.contains(expected),
            "status and scope has stale active-state prose: `{expected}`"
        );
    }

    let claim_ledger = fs::read_to_string(packet_root().join("claim_evidence_ledger.md"))
        .expect("maintained claim-evidence ledger");
    let claim_ledger = claim_ledger
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for expected in [
        "216 theorem rows: 56 are `proved-here`, 115 are `reexported-proof`, 39 are `checkpoint`, and six are `definition`.",
        "A fresh 435-row Lean receipt",
        "The contract has 108 distinct exercise solutions; 108 exercises remain correspondence-incomplete.",
        "correspondence axis records 108 exact rows, 39 checkpoints, and 69 unmapped rows.",
    ] {
        assert!(
            claim_ledger.contains(expected),
            "claim-evidence ledger has stale active-state prose: `{expected}`"
        );
    }

    assert_eq!(
        array(&coverage, "items", "coverage")
            .iter()
            .filter(|row| string(row, "lean_correspondence_status", "coverage row") == "unmapped")
            .count(),
        69
    );
}

fn markdown_metric_table(markdown: &str, heading: &str) -> BTreeMap<String, usize> {
    let section = markdown
        .split_once(heading)
        .unwrap_or_else(|| panic!("missing metric-table heading `{heading}`"))
        .1;
    section
        .lines()
        .skip_while(|line| !line.trim_start().starts_with('|'))
        .take_while(|line| line.trim_start().starts_with('|'))
        .skip(2)
        .map(|line| {
            let cells = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(str::trim)
                .collect::<Vec<_>>();
            assert_eq!(cells.len(), 2, "metric table row `{line}`");
            (
                cells[0].to_owned(),
                cells[1]
                    .parse::<usize>()
                    .unwrap_or_else(|_| panic!("non-numeric metric table row `{line}`")),
            )
        })
        .collect()
}

#[test]
fn publication_snapshots_are_structural_projections_of_current_contracts() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let coverage_rows = array(&coverage, "items", "coverage");
    let exercise_rows = array(&exercises, "exercises", "exercises");
    let expected = BTreeMap::from([
        ("theorem rows".to_owned(), coverage_rows.len()),
        (
            "summary prose rows".to_owned(),
            coverage_rows
                .iter()
                .filter(|row| string(row, "prose_proof_status", "coverage row") == "summary")
                .count(),
        ),
        (
            "reconstructible prose rows".to_owned(),
            coverage_rows
                .iter()
                .filter(|row| {
                    string(row, "prose_proof_status", "coverage row") == "reconstructible"
                })
                .count(),
        ),
        (
            "exact correspondence rows".to_owned(),
            coverage_rows
                .iter()
                .filter(|row| string(row, "lean_correspondence_status", "coverage row") == "exact")
                .count(),
        ),
        (
            "unmapped correspondence rows".to_owned(),
            coverage_rows
                .iter()
                .filter(|row| {
                    string(row, "lean_correspondence_status", "coverage row") == "unmapped"
                })
                .count(),
        ),
        (
            "solved exercises".to_owned(),
            exercise_rows
                .iter()
                .filter(|row| !row["lean_solution"].is_null())
                .count(),
        ),
        (
            "unresolved exercises".to_owned(),
            exercise_rows
                .iter()
                .filter(|row| row["lean_solution"].is_null())
                .count(),
        ),
    ]);
    for path in [
        "status_and_scope.md",
        "claim_evidence_ledger.md",
        "part_06_constant_two_routes/35_comparison_verification_and_boundaries.md",
    ] {
        let markdown = fs::read_to_string(packet_root().join(path)).expect("snapshot document");
        assert_eq!(
            markdown_metric_table(&markdown, "Contract snapshot"),
            expected,
            "stale contract snapshot in {path}"
        );
    }

    let chapter_33 = chapter_33_markdown();
    let chapter_33_expected = BTreeMap::from([
        ("solved exercises".to_owned(), 6),
        ("unresolved exercises".to_owned(), 0),
    ]);
    assert_eq!(
        markdown_metric_table(&chapter_33, "### Exercise contract snapshot"),
        chapter_33_expected
    );
}

#[test]
fn canonical_chapter_markers_project_the_correspondence_contracts() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    assert_chapter_markers_project_contracts(&coverage, &exercises);
}

fn assert_chapter_markers_project_contracts(coverage: &Value, exercises: &Value) {
    let coverage_rows = array(coverage, "items", "coverage");
    let exercise_rows = array(exercises, "exercises", "exercises");
    for (chapter_index, relative_path) in CHAPTERS.iter().enumerate() {
        let chapter = u64::try_from(chapter_index + 1).expect("chapter number");
        let prose_path = format!("knowledge/crouzeix_textbook/{relative_path}");
        let chapter_coverage = coverage_rows
            .iter()
            .filter(|row| integer(row, "chapter", "coverage row") == chapter)
            .collect::<Vec<_>>();
        assert!(
            !chapter_coverage.is_empty(),
            "chapter {chapter} coverage rows"
        );
        for row in &chapter_coverage {
            assert_eq!(
                string(row, "prose_path", "coverage row"),
                prose_path,
                "chapter {chapter} coverage path must match CHAPTERS"
            );
        }
        let expected_solutions = exercise_rows
            .iter()
            .filter(|row| integer(row, "chapter", "exercise row") == chapter)
            .filter(|row| {
                row.get("lean_solution")
                    .is_some_and(|solution| !solution.is_null())
            })
            .count();
        let expected_exact = chapter_coverage
            .iter()
            .filter(|row| string(row, "lean_correspondence_status", "coverage row") == "exact")
            .count();
        let markdown = fs::read_to_string(workspace_root().join(&prose_path))
            .unwrap_or_else(|error| panic!("read {prose_path}: {error}"));
        let frontmatter = markdown
            .strip_prefix("---\n")
            .and_then(|rest| rest.split_once("\n---\n"))
            .map(|(frontmatter, _)| frontmatter)
            .unwrap_or_else(|| panic!("{prose_path} has no closed frontmatter"));
        let marker = |name: &str| {
            let values = frontmatter
                .lines()
                .filter_map(|line| line.split_once(": "))
                .filter_map(|(field, value)| (field == name).then_some(value))
                .collect::<Vec<_>>();
            assert_eq!(values.len(), 1, "{prose_path} must define {name} once");
            values[0]
                .parse::<usize>()
                .unwrap_or_else(|error| panic!("{prose_path} has invalid {name}: {error}"))
        };
        assert_eq!(
            marker("lean_exercise_solution_declarations"),
            expected_solutions,
            "{prose_path} must project exercises.json"
        );
        assert_eq!(
            marker("lean_exact_correspondences"),
            expected_exact,
            "{prose_path} must project coverage.json"
        );
    }
}

#[test]
#[should_panic(expected = "coverage path")]
fn chapter_marker_projection_rejects_swapped_equal_count_chapter_targets() {
    let mut coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let rows = coverage["items"].as_array_mut().expect("coverage items");
    for item in 1..=6 {
        let chapter_two_id = format!("CFT-02-{item:03}");
        let chapter_three_id = format!("CFT-03-{item:03}");
        let chapter_two = rows
            .iter()
            .position(|row| row["item_id"] == chapter_two_id)
            .expect("Chapter 2 coverage row");
        let chapter_three = rows
            .iter()
            .position(|row| row["item_id"] == chapter_three_id)
            .expect("Chapter 3 coverage row");
        assert!(chapter_two < chapter_three, "canonical coverage order");
        let (left, right) = rows.split_at_mut(chapter_three);
        let chapter_two_row = &mut left[chapter_two];
        let chapter_three_row = &mut right[0];
        for field in ["prose_path", "anchor"] {
            std::mem::swap(&mut chapter_two_row[field], &mut chapter_three_row[field]);
        }
    }
    assert_chapter_markers_project_contracts(&coverage, &exercises);
}

#[test]
fn chapter_27_publishes_exact_reconstructible_correspondence() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let chapter_rows = array(&coverage, "items", "coverage")
        .iter()
        .filter(|row| integer(row, "chapter", "coverage row") == 27)
        .collect::<Vec<_>>();
    assert_eq!(chapter_rows.len(), 6);
    assert!(chapter_rows.iter().all(|row| {
        string(row, "prose_proof_status", "coverage row") == "reconstructible"
            && string(row, "lean_correspondence_status", "coverage row") == "exact"
    }));

    let prose_path = packet_root().join(CHAPTERS[26]);
    let markdown = fs::read_to_string(&prose_path).expect("Chapter 27 prose");
    assert!(!markdown.contains("navigation-checkpoints"));
    for item in 1..=6 {
        let id = format!("CFT-27-{item:03}");
        assert_eq!(markdown.matches(&format!("### {id} ")).count(), 1);
        assert!(markdown.contains("Public declaration: `CrouzeixTextbook.Part05."));
    }
}

fn chapter_33_markdown() -> String {
    fs::read_to_string(packet_root().join(CHAPTERS[32])).expect("Chapter 33 prose")
}

fn chapter_33_card<'a>(markdown: &'a str, item_id: &str) -> &'a str {
    let marker = format!("### {item_id} ");
    let start = markdown
        .find(&marker)
        .unwrap_or_else(|| panic!("Chapter 33 is missing {item_id}"));
    let tail = &markdown[start..];
    let end = tail[marker.len()..]
        .find("\n### CFT-33-")
        .map(|offset| marker.len() + offset)
        .or_else(|| tail.find("\n## Worked examples"))
        .unwrap_or(tail.len());
    &tail[..end]
}

fn chapter_33_card_field<'a>(card: &'a str, field: &str) -> &'a str {
    let marker = format!("\n#### {field}\n");
    let start = card
        .find(&marker)
        .unwrap_or_else(|| panic!("Chapter 33 card is missing `{field}`"));
    let body = &card[start + marker.len()..];
    let end = body.find("\n#### ").unwrap_or(body.len());
    body[..end].trim()
}

fn chapter_33_release_visible_text_errors(markdown: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for index in 1..=6 {
        let item_id = format!("CFT-33-{index:03}");
        let card = chapter_33_card(markdown, &item_id);
        for (field, required) in [
            ("Purpose", "Motivation."),
            ("Proof", "Worked instance."),
            ("ML analogy", "Mathematical object / ML counterpart."),
            ("ML analogy", "Exact transfer."),
            ("ML analogy", "Non-transfer."),
            ("ML analogy", "Diagnostic."),
        ] {
            let visible = visible_markdown_text(chapter_33_card_field(card, field));
            if !visible.contains(required) {
                errors.push(format!("{item_id} {field} is missing visible `{required}`"));
            }
        }
    }

    let solutions = markdown
        .split_once("### Solution sketches")
        .expect("Chapter 33 complete solutions")
        .1
        .split_once("\n### Exercise contract snapshot")
        .expect("Chapter 33 complete solution boundary")
        .0;
    for (exercise_id, mechanism) in [
        ("CFT-33-E01", "pow_succ"),
        ("CFT-33-E02", "discarded square"),
        ("CFT-33-E03", "Finset.sum_range_succ"),
        ("CFT-33-E04", "mul_ne_zero"),
        ("CFT-33-E05", "κ²(2-κ)"),
        ("CFT-33-E06", "scalar_endpoint_le_two"),
    ] {
        let heading = format!("#### {exercise_id} solution");
        let solution = solutions
            .split_once(&heading)
            .unwrap_or_else(|| panic!("missing complete solution `{heading}`"))
            .1
            .split_once("\n#### CFT-33-")
            .map_or_else(
                || solutions.split_once(&heading).unwrap().1,
                |(body, _)| body,
            );
        if !visible_markdown_text(solution).contains(mechanism) {
            errors.push(format!(
                "{exercise_id} written solution is missing visible `{mechanism}`"
            ));
        }
    }
    errors
}

const JIN_ROUTE_CONTRACT: &str =
    "docs/superpowers/specs/2026-08-27-crouzeix-textbook-jin-route-contract.md";
const JIN_CARD_FIELDS: [&str; 10] = [
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
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum JinPhase {
    BaselineFrozen,
    Chapter30Complete,
    Chapter31Complete,
    Wave2Complete,
}

impl JinPhase {
    const ALL: [Self; 4] = [
        Self::BaselineFrozen,
        Self::Chapter30Complete,
        Self::Chapter31Complete,
        Self::Wave2Complete,
    ];

    fn parse(value: &str) -> Option<Self> {
        match value {
            "baseline-frozen" => Some(Self::BaselineFrozen),
            "chapter-30-complete" => Some(Self::Chapter30Complete),
            "chapter-31-complete" => Some(Self::Chapter31Complete),
            "wave-2-complete" => Some(Self::Wave2Complete),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::BaselineFrozen => "baseline-frozen",
            Self::Chapter30Complete => "chapter-30-complete",
            Self::Chapter31Complete => "chapter-31-complete",
            Self::Wave2Complete => "wave-2-complete",
        }
    }

    fn completes(self, chapter: u64) -> bool {
        match self {
            Self::BaselineFrozen => false,
            Self::Chapter30Complete => chapter == 30,
            Self::Chapter31Complete => matches!(chapter, 30 | 31),
            Self::Wave2Complete => matches!(chapter, 30..=32),
        }
    }
}

#[derive(Clone, Copy)]
struct JinCftSpec {
    item_id: &'static str,
    chapter: u64,
    kind: &'static str,
    baseline_mode: &'static str,
    baseline_correspondence: &'static str,
    public_declaration: &'static str,
    public_file: &'static str,
    provider_declaration: &'static str,
    provider_file: &'static str,
    type_sha256: &'static str,
    hypotheses: &'static [&'static str],
    statement_steps: &'static [&'static str],
    proof_steps: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct JinExerciseSpec {
    exercise_id: &'static str,
    parent: &'static str,
    solution: &'static str,
    probe_declaration: &'static str,
    allowed_prerequisites: &'static [&'static str],
}

const JIN_AXIOMS: [&str; 3] = ["Classical.choice", "Quot.sound", "propext"];

const JIN_EXERCISE_SPECS: [JinExerciseSpec; 18] = [
    JinExerciseSpec {
        exercise_id: "CFT-30-E01",
        parent: "CFT-30-001",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_01_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_30_e01",
        allowed_prerequisites: &["CFT-29-002"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-30-E02",
        parent: "CFT-30-002",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_02_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_30_e02",
        allowed_prerequisites: &["CFT-30-001"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-30-E03",
        parent: "CFT-30-003",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_03_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_30_e03",
        allowed_prerequisites: &["CFT-30-001"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-30-E04",
        parent: "CFT-30-004",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_04_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_30_e04",
        allowed_prerequisites: &["CFT-30-001"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-30-E05",
        parent: "CFT-30-005",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_05_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_30_e05",
        allowed_prerequisites: &["CFT-30-003", "CFT-30-004"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-30-E06",
        parent: "CFT-30-006",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_06_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_30_e06",
        allowed_prerequisites: &["CFT-30-003", "CFT-30-004", "CFT-30-005"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-31-E01",
        parent: "CFT-31-001",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_01_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_31_e01",
        allowed_prerequisites: &["CFT-30-006"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-31-E02",
        parent: "CFT-31-002",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_02_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_31_e02",
        allowed_prerequisites: &["CFT-31-001"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-31-E03",
        parent: "CFT-31-003",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_03_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_31_e03",
        allowed_prerequisites: &["CFT-31-001", "CFT-31-002"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-31-E04",
        parent: "CFT-31-004",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_04_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_31_e04",
        allowed_prerequisites: &["CFT-31-003"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-31-E05",
        parent: "CFT-31-005",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_05_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_31_e05",
        allowed_prerequisites: &["CFT-31-004"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-31-E06",
        parent: "CFT-31-006",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_06_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_31_e06",
        allowed_prerequisites: &["CFT-31-003", "CFT-31-004", "CFT-31-005"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-32-E01",
        parent: "CFT-32-001",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_01_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_32_e01",
        allowed_prerequisites: &["CFT-30-006", "CFT-31-006"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-32-E02",
        parent: "CFT-32-002",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_02_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_32_e02",
        allowed_prerequisites: &["CFT-32-001"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-32-E03",
        parent: "CFT-32-003",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_03_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_32_e03",
        allowed_prerequisites: &["CFT-32-001"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-32-E04",
        parent: "CFT-32-004",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_04_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_32_e04",
        allowed_prerequisites: &["CFT-32-003"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-32-E05",
        parent: "CFT-32-005",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_05_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_32_e05",
        allowed_prerequisites: &["CFT-32-001", "CFT-32-003", "CFT-32-004"],
    },
    JinExerciseSpec {
        exercise_id: "CFT-32-E06",
        parent: "CFT-32-006",
        solution: "CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_06_solution",
        probe_declaration: "CrouzeixTextbook.JinContractProbe.cft_32_e06",
        allowed_prerequisites: &["CFT-32-005"],
    },
];

const JIN_CONTRACT_PROBE_DECLARATIONS: &str = r#"
namespace CrouzeixTextbook.JinContractProbe

open CrouzeixConjecture Filter Set
open scoped BigOperators ComplexConjugate ComplexOrder Matrix Matrix.Norms.L2Operator Topology

axiom cft_30_e01 {n : Type*} [Fintype n] [DecidableEq n]
    (B T : SquareMatrix n) (H : ℂ → SquareMatrix n)
    (hcompletion : IsPositiveRealCompletion B T H) :
    AnalyticOnNhd ℂ H unitDisk ∧ H 0 = 1 ∧
      (∀ z ∈ unitDisk, IsPositiveMatrix (rePart (H z))) ∧
      ∀ z ∈ unitDisk, H z - (1 - z • T)⁻¹ ∈ generatedAlgebra Bᴴ

axiom cft_30_e02 {n : Type*} [Fintype n] [DecidableEq n]
    (B T : SquareMatrix n) (H : ℂ → SquareMatrix n)
    (hB : SimpleDiagonalization B) (lambda : n → ℂ)
    (hT : T = innerConjugation hB.changeBasis (Matrix.diagonal lambda))
    (hlambda : ∀ i, ‖lambda i‖ ≤ 1)
    (hcompletion : IsPositiveRealCompletion B T H) : Commute B T

axiom cft_30_e03 {n : Type*} [Fintype n] [DecidableEq n]
    {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
    (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1) (i j : n) :
    completionP G lambda i j = G i j / (1 - conj (lambda i) * lambda j / 4) ∧
      Hinv * completionP G lambda * Hinv =
        gramian 4 (completionSimilarity H Hinv lambda)

axiom cft_30_e04 {n : Type*} [Fintype n] [DecidableEq n]
    {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
    (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1) (i j : n) :
    completionR G lambda i j = G i j / (1 - conj (lambda i) * lambda j / 2) ∧
      Hinv * completionR G lambda * Hinv =
        gramian 2 (completionSimilarity H Hinv lambda)

axiom cft_30_e05 {n : Type*} [Fintype n] [DecidableEq n]
    {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
    (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1) :
    completionX G lambda = completionR G lambda - completionP G lambda ∧
      Hinv * completionX G lambda * Hinv =
        gramian 2 (completionSimilarity H Hinv lambda) -
          gramian 4 (completionSimilarity H Hinv lambda)

axiom cft_30_e06 {n : Type*} [Fintype n] [DecidableEq n]
    {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
    (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1)
    (hsource : (4 * completionX G lambda -
      completionX G lambda * G⁻¹ * completionP G lambda -
      completionP G lambda * G⁻¹ * completionX G lambda).PosSemidef) :
    (Hinv * (4 * completionX G lambda -
      completionX G lambda * G⁻¹ * completionP G lambda -
      completionP G lambda * G⁻¹ * completionX G lambda) * Hinv).PosSemidef ∧
    Hinv * (4 * completionX G lambda -
      completionX G lambda * G⁻¹ * completionP G lambda -
      completionP G lambda * G⁻¹ * completionX G lambda) * Hinv =
      4 * (gramian 2 (completionSimilarity H Hinv lambda) -
        gramian 4 (completionSimilarity H Hinv lambda)) -
      (gramian 2 (completionSimilarity H Hinv lambda) -
        gramian 4 (completionSimilarity H Hinv lambda)) *
          gramian 4 (completionSimilarity H Hinv lambda) -
      gramian 4 (completionSimilarity H Hinv lambda) *
        (gramian 2 (completionSimilarity H Hinv lambda) -
          gramian 4 (completionSimilarity H Hinv lambda))

axiom cft_31_e01 {n : Type*} [Fintype n] [DecidableEq n]
    (G : SquareMatrix n) (lambda : n → ℂ) (d : ℂ → n → ℂ)
    (z : ℂ) (i j : n) :
    (completionKernelModel G lambda d z *ᵥ
        completionSparseVector (fun _ ↦ (1 : ℂ)) j) i =
      completionKernelModel G lambda d z i j ∧
    completionKernelModel G lambda d z i j =
      G i j * (1 - z * lambda j)⁻¹ + d z i * G i j

axiom cft_31_e02 {n : Type*} [Fintype n] [DecidableEq n]
    (G : SquareMatrix n) (hGunit : IsUnit G) (lambda : n → ℂ)
    (d : ℂ → n → ℂ) (hnormalized : completionKernelModel G lambda d 0 = G) :
    completionDiagonalCorrection d 0 = 0 ∧ d 0 = 0

axiom cft_31_e03 {n : Type*} [Fintype n] [DecidableEq n]
    {G : SquareMatrix n} (hG : G.IsHermitian) (lambda : n → ℂ)
    (hlambda : ∀ i, ‖lambda i‖ ≤ 1) :
    (∀ i j, matrixHerglotzKernel (completionResolventModel G lambda)
      (conj (lambda i) / 2) (conj (lambda j) / 2) i j =
        (4 • completionR G lambda - 2 • completionP G lambda) i j) ∧
    (∀ i j, matrixHerglotzKernel (completionResolventModel G lambda)
      (conj (lambda i) / 2) 0 i j = (G + completionR G lambda) i j) ∧
    (∀ i j, matrixHerglotzKernel (completionResolventModel G lambda)
      0 (conj (lambda j) / 2) i j = (G + completionR G lambda) i j) ∧
    ∀ i j, matrixHerglotzKernel (completionResolventModel G lambda) 0 0 i j =
      (2 • G) i j

axiom cft_31_e04 {n : Type*} [Fintype n] [DecidableEq n]
    {G : SquareMatrix n} (hG : G.IsHermitian) (lambda : n → ℂ)
    (d : ℂ → n → ℂ) (hd0 : d 0 = 0) (u : n → ℂ) :
    let theta := completionUnknownHalfContribution G lambda d u
    let correctionSum := ∑ a, ∑ b,
      star (completionSampleVector G lambda u a) ⬝ᵥ
        (matrixHerglotzKernel (completionCorrectionModel G d)
          (completionSamplePoint lambda a) (completionSamplePoint lambda b) *ᵥ
            completionSampleVector G lambda u b)
    correctionSum = theta + conj theta

axiom cft_31_e05 {n : Type*} [Fintype n] [DecidableEq n]
    (G : SquareMatrix n) (hGunit : IsUnit G) (lambda : n → ℂ)
    (d : ℂ → n → ℂ) (u : n → ℂ) :
    G *ᵥ completionV G (completionP G lambda) u + completionP G lambda *ᵥ u = 0 ∧
      completionUnknownHalfContribution G lambda d u = 0 ∧
      completionUnknownHalfContribution G lambda d u +
        conj (completionUnknownHalfContribution G lambda d u) = 0

axiom cft_31_e06 {n : Type*} [Fintype n] [DecidableEq n]
    (Y : SquareMatrix n) (hY : Y.IsHermitian)
    (hquadratic : ∀ u : n → ℂ, 0 ≤ star u ⬝ᵥ (Y *ᵥ u)) : Y.PosSemidef

axiom cft_32_e01 {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    {M : ℝ} (hM : 0 ≤ M) (C : SquareMatrix n)
    (hbound : ∀ k : ℕ, ‖C ^ k‖ ≤ M)
    (hineq : (4 • (gramian 2 C - gramian 4 C) -
      (gramian 2 C - gramian 4 C) * gramian 4 C -
      gramian 4 C * (gramian 2 C - gramian 4 C)).PosSemidef) : ‖C‖ ≤ 2

axiom cft_32_e02 {n : Type*} [Fintype n] [DecidableEq n]
    (A : SquareMatrix n) (p : Polynomial ℂ) (M : ℝ)
    (hM : M = maxPolynomialModulusOnNumericalRange A p) :
    (M = 0 → polynomialEval p A = 0) ∧
      (0 < M →
        ‖polynomialEval (((M : ℂ)⁻¹) • p) A‖ ≤ 2 →
        ‖polynomialEval p A‖ ≤ 2 * M)

axiom cft_32_e03 (r : RatFunc ℂ) (s : Set ℂ)
    (hfree : RationalPoleFreeOn r s) :
    IsOpen (rationalPoleSet r)ᶜ ∧ s ⊆ (rationalPoleSet r)ᶜ ∧
      DifferentiableOn ℂ (rationalScalarEval r) (rationalPoleSet r)ᶜ

axiom cft_32_e04 {n : Type*} [Fintype n] [DecidableEq n]
    (A : SquareMatrix n) (r : RatFunc ℂ)
    (hfree : RationalPoleFreeOn r (numericalRange A)) (M : ℝ)
    (hM : M = maxRationalModulusOnNumericalRange A r) :
    (M = 0 → rationalMatrixEval r A = 0) ∧
      (0 < M →
        ‖rationalMatrixEval (((M : ℂ)⁻¹) • r) A‖ ≤ 2 →
        ‖rationalMatrixEval r A‖ ≤ 2 * M)

axiom cft_32_e05 {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (A : SquareMatrix n) {U : Set ℂ} (hUopen : IsOpen U)
    (hWU : numericalRange A ⊆ U) {f : ℂ → ℂ}
    (hf : DifferentiableOn ℂ f U) :
    Tendsto (simpleSpectrumHolomorphicEval A f) atTop
      (nhds (holomorphicMatrixEval A f)) ∧
    ∃ N : ℕ, Tendsto
      (fun k ↦ maxFunctionModulusOnSet
        (closure (parallelOuterDomain (numericalRange A) (k + N))) f)
      atTop (nhds (maxFunctionModulusOnSet (numericalRange A) f))

axiom cft_32_e06 {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (A : SquareMatrix n) (p : Polynomial ℂ) :
    holomorphicMatrixEval A (fun z ↦ Polynomial.eval z p) = polynomialEval p A ∧
      maxFunctionModulusOnSet (numericalRange A) (fun z ↦ Polynomial.eval z p) =
        maxPolynomialModulusOnNumericalRange A p

axiom cft_31_002_public {n : Type*} [Fintype n] [DecidableEq n]
    (G : SquareMatrix n) (hGunit : IsUnit G) (lambda : n → ℂ)
    (d : ℂ → n → ℂ) (hnormalized : completionKernelModel G lambda d 0 = G) :
    d 0 = 0

axiom mutation_cft_31_002_without_invertibility {n : Type*} [Fintype n] [DecidableEq n]
    (G : SquareMatrix n) (lambda : n → ℂ) (d : ℂ → n → ℂ)
    (hnormalized : completionKernelModel G lambda d 0 = G) : d 0 = 0

end CrouzeixTextbook.JinContractProbe
"#;

const JIN_CFT_31_002_PROBE_DECLARATION: &str =
    "CrouzeixTextbook.JinContractProbe.cft_31_002_public";
const JIN_CFT_31_002_OMITTED_HYPOTHESIS_MUTATION: &str =
    "CrouzeixTextbook.JinContractProbe.mutation_cft_31_002_without_invertibility";

const JIN_CFT_SPECS: [JinCftSpec; 18] = [
    JinCftSpec {
        item_id: "CFT-30-001",
        chapter: 30,
        kind: "definition",
        baseline_mode: "checkpoint",
        baseline_correspondence: "checkpoint",
        public_declaration: "CrouzeixTextbook.Part06.positive_real_completion",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean",
        provider_declaration: "CrouzeixConjecture.IsPositiveRealCompletion",
        provider_file: "formalization/lean/CrouzeixConjecture/CompletionStatement.lean",
        type_sha256: "d72a6d691c4c807c0b20b620c2e030345c8d923a1c9b31d1a2a2e9fb07732523",
        hypotheses: &["B,T∈M_n(ℂ)", "H:ℂ→M_n(ℂ)", "𝔻={z:|z|<1}"],
        statement_steps: &[
            "IsPositiveRealCompletion(B,T,H)",
            "H analytic on 𝔻",
            "H(0)=I",
            "IsPositiveMatrix(Re H(z)) for z∈𝔻",
            "H(z)-(I-zT)⁻¹∈alg(Bᴴ) for z∈𝔻",
        ],
        proof_steps: &[
            "IsPositiveRealCompletion(B,T,H)",
            "↔ analytic ∧ normalized ∧ positive-real ∧ algebra-defect",
        ],
    },
    JinCftSpec {
        item_id: "CFT-30-002",
        chapter: 30,
        kind: "theorem",
        baseline_mode: "checkpoint",
        baseline_correspondence: "checkpoint",
        public_declaration: "CrouzeixTextbook.Part06.positive_real_completion_statement",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean",
        provider_declaration: "CrouzeixConjecture.PositiveRealCompletionStatement",
        provider_file: "formalization/lean/CrouzeixConjecture/CompletionStatement.lean",
        type_sha256: "23602f64481feee6c2fd6a0f7e743a74f4a1ee8625f11e06048a5c82ff99aeab",
        hypotheses: &[
            "[Nonempty n]",
            "B=S diag(μ)S⁻¹ with μ_i distinct",
            "T=S diag(λ)S⁻¹",
            "|λ_i|≤1",
            "IsPositiveRealCompletion(B,T,H)",
        ],
        statement_steps: &["B and T share S", "B has simple spectrum", "‖T‖≤2"],
        proof_steps: &[
            "simple spectrum belongs to B, not T",
            "shared-basis completion",
            "‖T‖≤2",
        ],
    },
    JinCftSpec {
        item_id: "CFT-30-003",
        chapter: 30,
        kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.completion_gramian_four",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean",
        provider_declaration: "CrouzeixConjecture.completionP_congruence_eq_gramian_four",
        provider_file: "formalization/lean/CrouzeixConjecture/CompletionGramianBridge.lean",
        type_sha256: "d999ab712880d5363dbc792dfeaa9c208ec4eae8435b470a1c25b4344c83fbdc",
        hypotheses: &["G=K²", "K⁻¹K=KK⁻¹=I", "|λ_i|≤1", "P∈M_n(ℂ)"],
        statement_steps: &[
            "P_ij=G_ij/(1-conj(λ_i)λ_j/4)",
            "C=K diag(λ)K⁻¹",
            "K⁻¹PK⁻¹=Gramian_4(C)",
        ],
        proof_steps: &[
            "4⁻ᵏ(Λᵏ)ᴴGΛᵏ",
            "K⁻¹[4⁻ᵏ(Λᵏ)ᴴGΛᵏ]K⁻¹=4⁻ᵏ(Cᵏ)ᴴCᵏ",
            "Σ_k 4⁻ᵏ(Cᵏ)ᴴCᵏ=Gramian_4(C)",
        ],
    },
    JinCftSpec {
        item_id: "CFT-30-004",
        chapter: 30,
        kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.completion_gramian_two",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean",
        provider_declaration: "CrouzeixConjecture.completionR_congruence_eq_gramian_two",
        provider_file: "formalization/lean/CrouzeixConjecture/CompletionGramianBridge.lean",
        type_sha256: "e9a78b5db14824f47398ed1a8a9cdbfcc1bbfae558930039ddbbb5d431dd3f84",
        hypotheses: &["G=K²", "K⁻¹K=KK⁻¹=I", "|λ_i|≤1", "R∈M_n(ℂ)"],
        statement_steps: &[
            "R_ij=G_ij/(1-conj(λ_i)λ_j/2)",
            "C=K diag(λ)K⁻¹",
            "K⁻¹RK⁻¹=Gramian_2(C)",
        ],
        proof_steps: &[
            "2⁻ᵏ(Λᵏ)ᴴGΛᵏ",
            "K⁻¹[2⁻ᵏ(Λᵏ)ᴴGΛᵏ]K⁻¹=2⁻ᵏ(Cᵏ)ᴴCᵏ",
            "Σ_k 2⁻ᵏ(Cᵏ)ᴴCᵏ=Gramian_2(C)",
        ],
    },
    JinCftSpec {
        item_id: "CFT-30-005",
        chapter: 30,
        kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.completion_gramian_difference",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean",
        provider_declaration: "CrouzeixConjecture.completionX_congruence_eq_gramian_difference",
        provider_file: "formalization/lean/CrouzeixConjecture/CompletionGramianBridge.lean",
        type_sha256: "2e7c01edefeb5dc2a3212ae85afc54e9caea46aaacf62c7996ad3de1ab1c1b67",
        hypotheses: &["Define X:=R-P", "G=K²", "K⁻¹K=KK⁻¹=I", "|λ_i|≤1"],
        statement_steps: &["Define X:=R-P", "K⁻¹XK⁻¹=Gramian_2(C)-Gramian_4(C)"],
        proof_steps: &[
            "K⁻¹(R-P)K⁻¹",
            "=K⁻¹RK⁻¹-K⁻¹PK⁻¹",
            "=Gramian_2(C)-Gramian_4(C)",
        ],
    },
    JinCftSpec {
        item_id: "CFT-30-006",
        chapter: 30,
        kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.completion_gramian_source_positive",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean",
        provider_declaration:
            "CrouzeixConjecture.completion_gramian_expression_posSemidef_of_source",
        provider_file: "formalization/lean/CrouzeixConjecture/CompletionGramianBridge.lean",
        type_sha256: "f9075e863bc01786d9635fb4461c6901054f999834318bfaf265db564791c01d",
        hypotheses: &["G=K²", "K=Kᴴ", "K⁻¹=(K⁻¹)ᴴ", "Y=4X-XG⁻¹P-PG⁻¹X⪰0"],
        statement_steps: &[
            "Y=4X-XG⁻¹P-PG⁻¹X⪰0",
            "K⁻¹YK⁻¹⪰0",
            "4(G₂-G₄)-(G₂-G₄)G₄-G₄(G₂-G₄)⪰0",
        ],
        proof_steps: &[
            "Y⪰0",
            "K⁻¹YK⁻¹⪰0",
            "K⁻¹XK⁻¹=G₂-G₄",
            "K⁻¹PK⁻¹=G₄",
            "4(G₂-G₄)-(G₂-G₄)G₄-G₄(G₂-G₄)⪰0",
        ],
    },
    JinCftSpec {
        item_id: "CFT-31-001",
        chapter: 31,
        kind: "definition",
        baseline_mode: "checkpoint",
        baseline_correspondence: "checkpoint",
        public_declaration: "CrouzeixTextbook.Part06.completion_kernel_model",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean",
        provider_declaration: "CrouzeixConjecture.completionKernelModel",
        provider_file: "formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean",
        type_sha256: "b6f13ba7834ee232d6361bbf95f939ad00760f82d267ffde7b3c22ed1b5a48b1",
        hypotheses: &[
            "G∈M_n(ℂ)",
            "Λ=diag(λ)",
            "D(z)=diag(d_i(z))",
            "Q(z)=diag((1-zλ_j)⁻¹)",
        ],
        statement_steps: &[
            "K(z)=GQ(z)+D(z)G",
            "K(z)_ij=G_ij(1-zλ_j)⁻¹+d_i(z)G_ij",
            "1-zλ_j≠0 for every j",
            "Q(z)=(I-zΛ)⁻¹",
        ],
        proof_steps: &[
            "[GQ(z)]_ij=G_ij(1-zλ_j)⁻¹",
            "[D(z)G]_ij=d_i(z)G_ij",
            "K(z)_ij=G_ij(1-zλ_j)⁻¹+d_i(z)G_ij",
        ],
    },
    JinCftSpec {
        item_id: "CFT-31-002",
        chapter: 31,
        kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.completion_kernel_at_zero",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean",
        provider_declaration: "CrouzeixConjecture.completionKernelModel_zero",
        provider_file: "formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean",
        type_sha256: "4749a4faa985491913c85c0e2c8b4194e3338905cc25dd01a2bca6426686e927",
        hypotheses: &["K(0)=G", "G invertible", "D(0) diagonal"],
        statement_steps: &["K(0)=G+D(0)G", "D(0)G=0", "D(0)=0"],
        proof_steps: &["G+D(0)G=G", "D(0)G=0", "D(0)=D(0)(GG⁻¹)=(D(0)G)G⁻¹=0"],
    },
    JinCftSpec {
        item_id: "CFT-31-003",
        chapter: 31,
        kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.sample_origin_quadratic_identity",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean",
        provider_declaration: "CrouzeixConjecture.completionResolventKernel_sampling_quadratic_eq",
        provider_file: "formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean",
        type_sha256: "a19830b823dec21c68e5ab7b248b9d78c9b03518b8605d869ee888192c2db7db",
        hypotheses: &["z_0=0", "z_i=conj(λ_i)/2", "x_0=v=-G⁻¹Pu", "x_i=u_i e_i∈ℂⁿ"],
        statement_steps: &[
            "sample/sample=4R-2P",
            "sample/origin=G+R",
            "origin/sample=G+R",
            "origin/origin=2G",
        ],
        proof_steps: &[
            "a_ij=conj(λ_i)λ_j",
            "2/((1-a_ij/4)(1-a_ij/2))=4/(1-a_ij/2)-2/(1-a_ij/4)",
            "L_0(z_i,z_j)_ij=(4R-2P)_ij",
            "L_0(z_i,0)_ij=(G+R)_ij",
            "L_0(0,z_j)_ij=(G+R)_ij",
            "L_0(0,0)_ij=(2G)_ij",
            "Σ_i Σ_j conj(u_i)(4R-2P)_ij u_j=uᴴ(4R-2P)u",
        ],
    },
    JinCftSpec {
        item_id: "CFT-31-004",
        chapter: 31,
        kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.correction_sampling_identity",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean",
        provider_declaration: "CrouzeixConjecture.completionCorrectionKernel_sampling_quadratic_eq",
        provider_file: "formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean",
        type_sha256: "3fbcbf6aba9f45dc919a68fa512cb886899febda58461c932c11d93054a74053",
        hypotheses: &[
            "G=G*",
            "P=Pᴴ",
            "Δ=diag(d_i(conj(λ_i)/2))",
            "d(0)=0",
            "v=-G⁻¹Pu",
        ],
        statement_steps: &[
            "L_D(z_i,z_j)_ij=(ΔP+PΔᴴ)_ij",
            "L_D(z_i,0)_ij=(ΔG)_ij",
            "L_D(0,z_j)_ij=(GΔᴴ)_ij",
            "L_D(0,0)=0",
            "correction sampling sum=v*(GΔ*)u+u*(ΔG)v+u*(ΔP+PΔ*)u",
            "Θ=u*Δ(Gv+Pu)",
            "correction sampling sum=Θ+conj(Θ)",
        ],
        proof_steps: &[
            "Θ=Σ_i conj(u_i)d_i(conj(λ_i)/2)(Gv+Pu)_i",
            "conj(Θ)=Σ_i conj((Gv+Pu)_i)conj(d_i(conj(λ_i)/2))u_i",
            "correction sampling sum=Θ+conj(Θ)",
        ],
    },
    JinCftSpec {
        item_id: "CFT-31-005",
        chapter: 31,
        kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.correction_sampling_cancels",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean",
        provider_declaration: "CrouzeixConjecture.completionCorrectionKernel_sampling_eq_zero",
        provider_file: "formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean",
        type_sha256: "3883ecf653a417da1c0aceb9c8ee7dd5f2c9444bf7fa7936660051df5e071234",
        hypotheses: &["G=G*", "G invertible", "v=-G⁻¹Pu", "d(0)=0"],
        statement_steps: &["Gv+Pu=0", "Θ=0", "Θ+conj(Θ)=0", "correction sampling sum=0"],
        proof_steps: &[
            "v=-G⁻¹Pu",
            "Gv+Pu=-GG⁻¹Pu+Pu=0",
            "Θ=Σ_i conj(u_i)d_i(conj(λ_i)/2)·0=0",
            "Θ+conj(Θ)=0",
        ],
    },
    JinCftSpec {
        item_id: "CFT-31-006",
        chapter: 31,
        kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.kernel_positivity_implies_X",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean",
        provider_declaration: "CrouzeixConjecture.completion_X_inequality_of_positiveKernel",
        provider_file: "formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean",
        type_sha256: "8ea6a00ebbb6a7326410c88a8d764396afc51794cde409a6c5f3b44855ce7e04",
        hypotheses: &[
            "L_K positive as a matrix kernel on 𝔻",
            "G=G* and G invertible",
            "|λ_i|≤1",
            "d(0)=0",
        ],
        statement_steps: &[
            "sampled-kernel PSD",
            "u*(4X-XG⁻¹P-PG⁻¹X)u≥0 for every u",
            "4X-XG⁻¹P-PG⁻¹X⪰0",
        ],
        proof_steps: &[
            "analyticity + pointwise positive-real ⇒ L_K kernel-positive",
            "L_K kernel-positive ⇒ sampled-kernel PSD",
            "sampled-kernel PSD ⇒ block-matrix PSD",
            "block-matrix PSD ⇒ finite quadratic sum≥0",
            "correction sum=0",
            "finite quadratic sum=u*(4X-XG⁻¹P-PG⁻¹X)u",
            "(4X-XG⁻¹P-PG⁻¹X)ᴴ=4X-XG⁻¹P-PG⁻¹X",
            "quadratic nonnegativity ⇒ matrix PSD",
        ],
    },
    JinCftSpec {
        item_id: "CFT-32-001",
        chapter: 32,
        kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.completion_implies_norm_two",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean",
        provider_declaration:
            "CrouzeixConjecture.norm_completionDiagonalizableMatrix_le_two_of_positiveKernelModel",
        provider_file: "formalization/lean/CrouzeixConjecture/PositiveRealCompletion.lean",
        type_sha256: "1c24687b99a83a92aa728551ff33b388ec0e9f1551a6a36145d826b25311ccd1",
        hypotheses: &[
            "S invertible",
            "G=S*S",
            "C=G¹ᐟ²ΛG⁻¹ᐟ²",
            "|λ_i|≤1",
            "positive completion kernel",
        ],
        statement_steps: &[
            "4(G₂-G₄)-(G₂-G₄)G₄-G₄(G₂-G₄)⪰0",
            "4I-C*C⪰0",
            "‖C‖≤2",
            "‖SΛS⁻¹‖=‖C‖≤2",
        ],
        proof_steps: &[
            "choose e with G₄e=pe",
            "assume p>2",
            "e*(G₂-G₄)e=0",
            "(G₂-G₄)e=0",
            "Y=(1/4)C*C",
            "G₂-G₄-Y⪰₀ and Y⪰₀",
            "e*Ye=0",
            "Ce=0",
            "G₄e=e",
            "G₄e=pe gives p=1, contradiction",
            "G₄≤2I",
            "G₄=I+Σ_{k≥1}4⁻ᵏ(Cᵏ)*Cᵏ≥I",
            "C*C≤4(G₄-I)≤4I",
            "‖C‖≤2",
            "‖SΛS⁻¹‖=‖C‖≤2",
        ],
    },
    JinCftSpec {
        item_id: "CFT-32-002",
        chapter: 32,
        kind: "theorem",
        baseline_mode: "checkpoint",
        baseline_correspondence: "checkpoint",
        public_declaration: "CrouzeixTextbook.Part06.jin_polynomial_constant_two",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean",
        provider_declaration: "CrouzeixConjecture.jinFinalCrouzeixConjecture",
        provider_file: "formalization/lean/CrouzeixConjecture/FinalTheorems.lean",
        type_sha256: "0c2cdfce0642e8d4ab88dc860b6a60e85be855897e5f63fd51babbf67c87fc5d",
        hypotheses: &["A∈M_n(ℂ)", "p∈ℂ[z]", "M=max_{z∈W(A)}|p(z)|"],
        statement_steps: &[
            "q=p/M when M>0",
            "max_{W(A)}|q|≤1",
            "|q_m(z)|≤1 for every z∈closure(Ω_m)",
            "μ_{j,i}=q_m(λ_{j,i})",
            "q_m(A_j)=S_j diag(μ_j) S_j⁻¹",
            "‖p(A)‖≤2M",
        ],
        proof_steps: &[
            "If M=0",
            "Now assume M>0",
            "M_m=max_{z∈closure(Ω_m)}|p(z)|",
            "q_m=p/M_m",
            "|q_m(z)|≤1 for every z∈closure(Ω_m)",
            "A_j→A",
            "σ(A_j)⊆W(A_j)⊆Ω_m",
            "μ_{j,i}=q_m(λ_{j,i})",
            "q_m(A_j)=S_j diag(μ_j) S_j⁻¹",
            "outer-boundary Cauchy identities",
            "construct the positive-real completion",
            "d_j(0)=0",
            "positive Herglotz kernel",
            "CFT-32-001 therefore gives",
            "‖q_m(A_j)‖≤2",
            "First let j→∞ with m fixed",
            "Only then let m→∞",
            "M_m→M",
            "‖p(A)‖≤2M",
        ],
    },
    JinCftSpec {
        item_id: "CFT-32-003",
        chapter: 32,
        kind: "theorem",
        baseline_mode: "checkpoint",
        baseline_correspondence: "checkpoint",
        public_declaration: "CrouzeixTextbook.Part06.jin_rational_spectral_set",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean",
        provider_declaration: "CrouzeixConjecture.crouzeixRationalSpectralSetCorollary",
        provider_file: "formalization/lean/CrouzeixConjecture/FinalTheorems.lean",
        type_sha256: "a68683384ff3984804bccbfaf7f5dea8ca6128d26a0618984bc5d293837f0144",
        hypotheses: &[
            "poles(r)∩W(A)=∅",
            "r(A) defined by rational functional calculus",
        ],
        statement_steps: &["poles(r)∩W(A)=∅", "‖r(A)‖≤2 max_{z∈W(A)}|r(z)|"],
        proof_steps: &[
            "W(A)⊆ℂ\\poles(r)",
            "r holomorphic on a neighborhood of W(A)",
            "holomorphic bound ⇒ rational spectral-set bound",
        ],
    },
    JinCftSpec {
        item_id: "CFT-32-004",
        chapter: 32,
        kind: "theorem",
        baseline_mode: "checkpoint",
        baseline_correspondence: "checkpoint",
        public_declaration: "CrouzeixTextbook.Part06.jin_rational_constant_two",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean",
        provider_declaration: "CrouzeixConjecture.crouzeixRationalBound",
        provider_file: "formalization/lean/CrouzeixConjecture/FinalTheorems.lean",
        type_sha256: "45eabf35921b773b4ec7c3aacb0af28c045bd1f008d6f6ced30c63724d5a91c8",
        hypotheses: &["M=max_{z∈W(A)}|r(z)|", "poles(r)∩W(A)=∅"],
        statement_steps: &["s=r/M when M>0", "max_{W(A)}|s|≤1", "‖r(A)‖≤2M"],
        proof_steps: &[
            "M=0 ⇒ r(A)=0",
            "M>0 ⇒ s=r/M",
            "‖s(A)‖≤2",
            "r(A)=M s(A)",
            "‖r(A)‖≤2M",
        ],
    },
    JinCftSpec {
        item_id: "CFT-32-005",
        chapter: 32,
        kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.holomorphic_constant_two",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean",
        provider_declaration: "CrouzeixConjecture.holomorphicCrouzeixBound",
        provider_file: "formalization/lean/CrouzeixConjecture/HolomorphicOuterLimit.lean",
        type_sha256: "46a9ed9e7bfeb40a079ae46d8cb6d2bbb0d0d330b9eaf2646c0f15ddae2552b8",
        hypotheses: &["U open", "W(A)⊆U", "f holomorphic on U"],
        statement_steps: &["‖f(A)‖≤2 max_{z∈W(A)}|f(z)|"],
        proof_steps: &[
            "A_k→A with A_k simple-spectrum",
            "f(A_k)→f(A) on one fixed outer domain Ω_m",
            "‖f(A)‖≤2 max_{z∈closure(Ω_m)}|f(z)|",
            "closure(Ω_m)↓W(A)",
            "max_{closure(Ω_m)}|f|→max_{W(A)}|f|",
            "‖f(A)‖≤2 max_{W(A)}|f|",
        ],
    },
    JinCftSpec {
        item_id: "CFT-32-006",
        chapter: 32,
        kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.polynomial_from_holomorphic",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean",
        provider_declaration:
            "CrouzeixConjecture.polynomialCrouzeixBound_of_holomorphicCrouzeixBound",
        provider_file: "formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean",
        type_sha256: "0c2cdfce0642e8d4ab88dc860b6a60e85be855897e5f63fd51babbf67c87fc5d",
        hypotheses: &["p∈ℂ[z]", "p holomorphic on ℂ", "W(A) compact"],
        statement_steps: &[
            "holomorphicMatrixEval(A,p)=p(A)",
            "‖p(A)‖≤2 max_{z∈W(A)}|p(z)|",
        ],
        proof_steps: &[
            "outer-domain limit completed before polynomial specialization",
            "f=p",
            "holomorphicMatrixEval(A,p)=polynomialEval(p,A)",
            "maxFunctionModulusOnSet(W(A),p)=maxPolynomialModulusOnNumericalRange(A,p)",
            "‖p(A)‖≤2 max_{W(A)}|p|",
        ],
    },
];

fn jin_cft_spec(item_id: &str) -> &'static JinCftSpec {
    JIN_CFT_SPECS
        .iter()
        .find(|spec| spec.item_id == item_id)
        .unwrap_or_else(|| panic!("missing Jin CFT spec {item_id}"))
}

fn jin_baseline_kind(spec: &JinCftSpec) -> &'static str {
    if spec.item_id == "CFT-32-001" {
        "definition"
    } else {
        spec.kind
    }
}

fn jin_completed_mode(spec: &JinCftSpec) -> &'static str {
    if matches!(spec.item_id, "CFT-30-001" | "CFT-31-001") {
        "definition"
    } else if matches!(spec.item_id, "CFT-30-006" | "CFT-31-002" | "CFT-32-001") {
        "proved-here"
    } else {
        "reexported-proof"
    }
}

fn jin_completed_receipt_kind(spec: &JinCftSpec) -> &'static str {
    if jin_completed_mode(spec) == "proved-here" {
        "theorem"
    } else {
        "direct-alias"
    }
}

fn jin_completed_public_declaration(spec: &JinCftSpec) -> &'static str {
    if spec.item_id == "CFT-31-002" {
        "CrouzeixTextbook.Part06.completion_kernel_normalization_forces_correction_zero"
    } else {
        spec.public_declaration
    }
}

fn jin_completed_provider_declaration(spec: &JinCftSpec) -> &'static str {
    match spec.item_id {
        "CFT-30-002" => "CrouzeixConjecture.positiveRealCompletionStatement",
        "CFT-31-002" => {
            "CrouzeixTextbook.Part06.completion_kernel_normalization_forces_correction_zero_bridge"
        }
        "CFT-32-002" => "CrouzeixConjecture.polynomialCrouzeixBound_of_holomorphicCrouzeixBound",
        "CFT-32-003" => "CrouzeixTextbook.Part06.jin_rational_spectral_set_provider",
        "CFT-32-004" => "CrouzeixConjecture.holomorphicCrouzeixRationalBound",
        _ => spec.provider_declaration,
    }
}

fn jin_completed_provider_file(spec: &JinCftSpec) -> &'static str {
    match spec.item_id {
        "CFT-30-002" => "formalization/lean/CrouzeixConjecture/CompletionDiagonalization.lean",
        "CFT-31-002" => "formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean",
        "CFT-32-002" | "CFT-32-004" => {
            "formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean"
        }
        "CFT-32-003" => "formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean",
        _ => spec.provider_file,
    }
}

fn jin_phase_public_declaration(spec: &JinCftSpec, phase: JinPhase) -> &'static str {
    if phase.completes(spec.chapter) {
        jin_completed_public_declaration(spec)
    } else {
        spec.public_declaration
    }
}

fn jin_phase_provider_declaration(spec: &JinCftSpec, phase: JinPhase) -> &'static str {
    if phase.completes(spec.chapter) {
        jin_completed_provider_declaration(spec)
    } else {
        spec.provider_declaration
    }
}

fn jin_required_exercise_dependencies(spec: &JinExerciseSpec) -> &'static [&'static str] {
    match spec.exercise_id {
        "CFT-31-E01" => &["CrouzeixConjecture.mulVec_completionSparseVector"],
        "CFT-31-E04" => &["CrouzeixConjecture.completionUnknownHalfContribution_eq_matrixPairing"],
        "CFT-31-E05" => &["CrouzeixConjecture.completionUnknownHalfContribution_eq_zero"],
        "CFT-32-E01" => &[
            "CrouzeixConjecture.four_sub_conjTranspose_mul_self_posSemidef_of_gramian_inequality",
            "CrouzeixConjecture.matrix_norm_le_two_of_four_sub_conjTranspose_mul_self_posSemidef",
        ],
        "CFT-32-E03" => &[
            "CrouzeixConjecture.rationalPoleFreeOn_iff_subset_compl",
            "CrouzeixConjecture.differentiableOn_rationalScalarEval",
        ],
        "CFT-32-E05" => &[
            "CrouzeixConjecture.tendsto_simpleSpectrumHolomorphicEval_of_differentiableOn_neighborhood",
            "CrouzeixConjecture.tendsto_maxFunctionModulusOnSet_of_outerApproximation",
        ],
        "CFT-32-E06" => &[
            "CrouzeixConjecture.holomorphicMatrixEval_polynomial",
        ],
        _ => &[],
    }
}

fn jin_baseline_provider_hash(spec: &JinCftSpec) -> &'static str {
    match spec.item_id {
        "CFT-30-001" => "484e9737cde90a145a1db689cb649f22f3ee2f1cf3e299bd52d3a2374b9686a5",
        "CFT-31-001" => "36e752a3852f89e9a1bc7b1e567b974b702759d6ec39097a448fb277a00fb7ce",
        "CFT-32-002" => "5a08b37106dc806e9cb5f69cb03f5e13bd850055d67cb410ad615924faaab9c2",
        "CFT-32-004" => "65ba1f3e8ad1e9fd48dede033d58509648d2c01d5e7cb5d49b09d967d2affe34",
        _ => spec.type_sha256,
    }
}

fn compile_jin_contract_probe(declarations: &str) -> Result<Value, String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    let source_path = temp.path().join("JinContractProbe.lean");
    let names = JIN_EXERCISE_SPECS
        .iter()
        .map(|spec| spec.probe_declaration)
        .chain([
            JIN_CFT_31_002_PROBE_DECLARATION,
            JIN_CFT_31_002_OMITTED_HYPOTHESIS_MUTATION,
        ])
        .map(|name| format!("`{name}"))
        .collect::<Vec<_>>()
        .join(", ");
    let phase = jin_contract_phase(&jin_route_contract());
    let conformance_pairs = JIN_EXERCISE_SPECS
        .iter()
        .filter(|spec| {
            let chapter = spec.exercise_id[4..6]
                .parse::<u64>()
                .expect("exercise chapter");
            phase.completes(chapter)
        })
        .map(|spec| format!("(`{}, `{})", spec.probe_declaration, spec.solution))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        r#"import CrouzeixTextbook.ExportReceipt
{declarations}
open Lean Meta
run_cmd do
  let env ← getEnv
  let conformancePairs : Array (Name × Name) := #[{conformance_pairs}]
  for (contractName, solutionName) in conformancePairs do
    let some contractInfo := env.find? contractName
      | throwError "contract declaration is missing: {{contractName}}"
    let some solutionInfo := env.find? solutionName
      | throwError "solution declaration is missing: {{solutionName}}"
    let sameType ← Lean.Elab.Command.liftCoreM <| MetaM.run' do
      isDefEq contractInfo.type solutionInfo.type
    unless sameType do
      throwError "solution type differs from exact contract: {{solutionName}}"
  let names : Array Name := #[{names}]
  let rows ← Lean.Elab.Command.liftCoreM <|
    CrouzeixTextbook.ExportReceipt.receiptRows env names
  IO.println s!"JIN_CONTRACT_PROBE:{{(Json.arr rows).compress}}"
"#
    );
    fs::write(&source_path, source).map_err(|error| error.to_string())?;
    let output = textbook_lean_command("lake")
        .arg("env")
        .arg("lean")
        .arg(&source_path)
        .current_dir(workspace_root().join("formalization/lean"))
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "contract probe failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    let payload = stdout
        .lines()
        .find_map(|line| line.strip_prefix("JIN_CONTRACT_PROBE:"))
        .ok_or_else(|| "contract probe omitted its JSON marker".to_owned())?;
    serde_json::from_str(payload).map_err(|error| error.to_string())
}

fn jin_contract_probe_receipt() -> &'static Value {
    static RECEIPT: OnceLock<Value> = OnceLock::new();
    RECEIPT.get_or_init(|| {
        compile_jin_contract_probe(JIN_CONTRACT_PROBE_DECLARATIONS)
            .unwrap_or_else(|error| panic!("Jin contract signatures must elaborate: {error}"))
    })
}

fn jin_contract_probe_row(name: &str) -> &'static Value {
    jin_contract_probe_receipt()
        .as_array()
        .expect("Jin contract probe rows")
        .iter()
        .find(|row| row["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("Jin contract probe omitted {name}"))
}

fn compile_jin_exercise_dependency_receipt() -> Result<Value, String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    let source_path = temp.path().join("JinExerciseDependencyProbe.lean");
    let phase = jin_contract_phase(&jin_route_contract());
    let roots = JIN_EXERCISE_SPECS
        .iter()
        .filter(|spec| {
            let chapter = spec.exercise_id[4..6]
                .parse::<u64>()
                .expect("exercise chapter");
            phase.completes(chapter)
        })
        .map(|spec| format!("`{}", spec.solution))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        r#"import CrouzeixTextbook.ExportReceipt
open Lean Meta
namespace CrouzeixTextbook.JinExerciseDependencyProbe
partial def dependencyClosure (env : Environment) : List Name → NameHashSet → NameHashSet
  | [], seen => seen
  | name :: pending, seen =>
      if seen.contains name then dependencyClosure env pending seen
      else
        let seen := seen.insert name
        match env.find? name with
        | none => dependencyClosure env pending seen
        | some info =>
            let dependencies := CrouzeixTextbook.ExportReceipt.directConstants info
            dependencyClosure env (dependencies.toList ++ pending) seen
def isTrackedExerciseDeclaration (env : Environment) (name : Name) : Bool :=
  match CrouzeixTextbook.ExportReceipt.sourceLocation env name with
  | none => false
  | some (sourcePath, _, _) =>
      let rendered := name.toString
      !rendered.contains "._" &&
      ((sourcePath.endsWith "Part06/Chapter30.lean" &&
          rendered.contains "Exercises.Chapter30.") ||
        (sourcePath.endsWith "Part06/Chapter31.lean" &&
          rendered.contains "Exercises.Chapter31.") ||
        (sourcePath.endsWith "Part06/Chapter32.lean" &&
          rendered.contains "Exercises.Chapter32."))
run_cmd do
  let env ← getEnv
  let roots : Array Name := #[{roots}]
  let maintainedClosure := (dependencyClosure env roots.toList {{}}).toArray
    |>.filter CrouzeixTextbook.ExportReceipt.isMaintainedName
    |>.foldl (init := ({{}} : NameHashSet)) fun selected name => selected.insert name
  let declarations := env.constants.toList.foldl (init := maintainedClosure) fun selected (name, _) =>
    if isTrackedExerciseDeclaration env name then selected.insert name else selected
  let names := declarations.toArray
    |>.qsort (fun left right => left.toString < right.toString)
  let mut rows : Array Json := #[]
  for name in names do
    let some info := env.find? name
      | throwError "dependency declaration disappeared: {{name}}"
    let normalizedType ← Lean.Elab.Command.liftCoreM <|
      CrouzeixTextbook.ExportReceipt.normalizedType info
    let dependencies := CrouzeixTextbook.ExportReceipt.directConstants info
      |>.filter (· != name)
    let dependencyJson := dependencies.map (Json.str ·.toString)
    rows := rows.push <| json% {{
      name: $(name.toString),
      kind: $(CrouzeixTextbook.ExportReceipt.declarationKind info),
      normalized_type: $(normalizedType),
      type_sha256: $(CrouzeixTextbook.ExportReceipt.sha256 normalizedType),
      direct_dependencies: $(dependencyJson)
    }}
  IO.println s!"JIN_EXERCISE_DEPENDENCY_RECEIPT:{{(Json.arr rows).compress}}"
end CrouzeixTextbook.JinExerciseDependencyProbe
"#
    );
    fs::write(&source_path, source).map_err(|error| error.to_string())?;
    let output = textbook_lean_command("lake")
        .arg("env")
        .arg("lean")
        .arg(&source_path)
        .current_dir(workspace_root().join("formalization/lean"))
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "exercise dependency probe failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    let payload = stdout
        .lines()
        .find_map(|line| line.strip_prefix("JIN_EXERCISE_DEPENDENCY_RECEIPT:"))
        .ok_or_else(|| "exercise dependency probe omitted its JSON marker".to_owned())?;
    serde_json::from_str(payload).map_err(|error| error.to_string())
}

fn jin_exercise_dependency_receipt() -> &'static Value {
    static RECEIPT: OnceLock<Value> = OnceLock::new();
    RECEIPT.get_or_init(|| {
        compile_jin_exercise_dependency_receipt()
            .unwrap_or_else(|error| panic!("Jin exercise dependency receipt must compile: {error}"))
    })
}

fn jin_exercise_receipt_hash(spec: &JinExerciseSpec) -> String {
    let exercises = read_json(&contracts_root().join("exercises.json"));
    array(&exercises, "exercises", "exercises")
        .iter()
        .find(|row| row["exercise_id"].as_str() == Some(spec.exercise_id))
        .and_then(|row| row["lean_solution"]["type_sha256"].as_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            jin_contract_probe_row(spec.probe_declaration)["type_sha256"]
                .as_str()
                .expect("exercise probe type hash")
                .to_owned()
        })
}

fn jin_contract_probe_errors(receipt: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    let rows = receipt.as_array().expect("Jin contract probe rows");
    for name in JIN_EXERCISE_SPECS
        .iter()
        .map(|spec| spec.probe_declaration)
        .chain([JIN_CFT_31_002_PROBE_DECLARATION])
    {
        let matches = rows
            .iter()
            .filter(|row| row["name"].as_str() == Some(name))
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            errors.push(format!(
                "contract probe must contain exactly one `{name}`, observed {}",
                matches.len()
            ));
            continue;
        }
        let row = matches[0];
        let normalized_type = row["normalized_type"].as_str().unwrap_or_default();
        if row["kind"].as_str() != Some("axiom")
            || normalized_type.is_empty()
            || row["type_sha256"].as_str() != Some(receipt_type_hash(normalized_type).as_str())
        {
            errors.push(format!(
                "contract probe `{name}` lacks compiler-derived axiom/type/hash metadata"
            ));
        }
        let canonical = jin_contract_probe_row(name);
        if row["normalized_type"] != canonical["normalized_type"]
            || row["type_sha256"] != canonical["type_sha256"]
        {
            errors.push(format!(
                "contract probe `{name}` differs from its canonical elaborated signature"
            ));
        }
    }
    errors
}

fn jin_target_provider_hash(spec: &JinCftSpec) -> String {
    match spec.item_id {
        "CFT-30-001" => {
            "d72a6d691c4c807c0b20b620c2e030345c8d923a1c9b31d1a2a2e9fb07732523".to_owned()
        }
        "CFT-30-002" => {
            "720010d2fd3b19be340951f1fda1fd0e02a095476a0bdd4a60e8af086352deb2".to_owned()
        }
        "CFT-30-006" => {
            "54e0b92ee477409a1b741eb87362322c7a801bb9c9ecdb4b61a1320140728baa".to_owned()
        }
        "CFT-31-001" => {
            "b6f13ba7834ee232d6361bbf95f939ad00760f82d267ffde7b3c22ed1b5a48b1".to_owned()
        }
        "CFT-31-002" => {
            "22fb9474d934ab848aa65584ab7f1a7122b901afc9aae3553554bd415ac1eefe".to_owned()
        }
        "CFT-32-001" => {
            "7ca9d7c52fc65635b1b006a3e7895bc26df5a8c906f2d5848787e367a8d71a8c".to_owned()
        }
        "CFT-32-002" => {
            "0c2cdfce0642e8d4ab88dc860b6a60e85be855897e5f63fd51babbf67c87fc5d".to_owned()
        }
        "CFT-32-004" => {
            "45eabf35921b773b4ec7c3aacb0af28c045bd1f008d6f6ced30c63724d5a91c8".to_owned()
        }
        _ => jin_baseline_provider_hash(spec).to_owned(),
    }
}

fn jin_baseline_provider_kind(spec: &JinCftSpec) -> &'static str {
    match spec.item_id {
        "CFT-30-001" | "CFT-30-002" | "CFT-31-001" => "definition",
        "CFT-30-003" | "CFT-30-004" | "CFT-30-005" | "CFT-30-006" | "CFT-31-002" | "CFT-31-003"
        | "CFT-31-004" | "CFT-31-005" | "CFT-31-006" | "CFT-32-001" | "CFT-32-005"
        | "CFT-32-006" => "theorem",
        "CFT-32-002" | "CFT-32-003" | "CFT-32-004" => "direct-alias",
        item_id => panic!("unregistered Jin baseline provider kind for {item_id}"),
    }
}

fn jin_target_provider_kind(spec: &JinCftSpec) -> &'static str {
    match spec.item_id {
        "CFT-30-001" | "CFT-31-001" => "definition",
        "CFT-30-002" | "CFT-30-003" | "CFT-30-004" | "CFT-30-005" | "CFT-30-006" | "CFT-31-002"
        | "CFT-31-003" | "CFT-31-004" | "CFT-31-005" | "CFT-31-006" | "CFT-32-001"
        | "CFT-32-005" | "CFT-32-006" => "theorem",
        "CFT-32-002" | "CFT-32-003" | "CFT-32-004" => "theorem",
        item_id => panic!("unregistered Jin target provider kind for {item_id}"),
    }
}

fn jin_target_fixture_hash(spec: &JinCftSpec) -> String {
    match spec.item_id {
        "CFT-30-002" => {
            "720010d2fd3b19be340951f1fda1fd0e02a095476a0bdd4a60e8af086352deb2".to_owned()
        }
        "CFT-30-006" => {
            "2c90dd2e81251f789c757fc791515b9d4889f0c5c8f0e74fc1b7ce75e8f17356".to_owned()
        }
        // The temporary probe prints the `ℂ` notation while the maintained
        // receipt exporter prints the underlying `Complex` constant.  The
        // proposition is compiler-checked for definitional equality below;
        // publication metadata must use the maintained receipt's exact hash.
        "CFT-31-002" => {
            "c366aa91bab218699594028032e0feca8e1d84f06804e10dd48e71a7625ccb49".to_owned()
        }
        _ => spec.type_sha256.to_owned(),
    }
}

fn jin_baseline_fixture() -> (Value, Value) {
    let items = JIN_CFT_SPECS
        .iter()
        .map(|spec| {
            let underlying = if spec.baseline_correspondence == "unmapped" {
                json!(spec.provider_declaration)
            } else {
                Value::Null
            };
            json!({
                "item_id": spec.item_id,
                "chapter": spec.chapter,
                "kind": jin_baseline_kind(spec),
                "formal_mode": spec.baseline_mode,
                "lean_correspondence_status": spec.baseline_correspondence,
                "prose_proof_status": "summary",
                "lean_declaration": {
                    "name": spec.public_declaration,
                    "underlying_declaration": underlying,
                    "source_path": spec.public_file,
                    "verification_target": "CrouzeixTextbook",
                    "type_sha256": spec.type_sha256,
                    "assumptions": [],
                    "axioms": JIN_AXIOMS,
                },
            })
        })
        .collect::<Vec<_>>();
    let exercises = JIN_EXERCISE_SPECS
        .iter()
        .map(|exercise| {
            let cft = jin_cft_spec(exercise.parent);
            json!({
                "exercise_id": exercise.exercise_id,
                "chapter": cft.chapter,
                "skills": [exercise.parent],
                "starter": null,
                "lean_solution": null,
            })
        })
        .collect::<Vec<_>>();
    (json!({"items": items}), json!({"exercises": exercises}))
}

fn jin_route_contract() -> String {
    fs::read_to_string(workspace_root().join(JIN_ROUTE_CONTRACT))
        .expect("frozen Jin route contract")
}

fn jin_contract_phase(contract: &str) -> JinPhase {
    let prefix = "- Active phase: `";
    let label = contract
        .lines()
        .find_map(|line| line.strip_prefix(prefix)?.strip_suffix("`."))
        .expect("Jin contract active phase");
    JinPhase::parse(label).unwrap_or_else(|| panic!("unknown Jin contract phase {label}"))
}

fn jin_chapter_markdown(chapter: u64) -> String {
    let index = usize::try_from(chapter - 1).expect("chapter index");
    fs::read_to_string(packet_root().join(CHAPTERS[index]))
        .unwrap_or_else(|error| panic!("Chapter {chapter} prose: {error}"))
}

fn jin_card<'a>(markdown: &'a str, item_id: &str, chapter: u64) -> Option<&'a str> {
    let marker = format!("### {item_id} ");
    let start = markdown.find(&marker)?;
    let tail = &markdown[start..];
    let next_marker = format!("\n### CFT-{chapter:02}-");
    let end = tail[marker.len()..]
        .find(&next_marker)
        .map(|offset| marker.len() + offset)
        .or_else(|| tail.find("\n## Worked examples"))
        .unwrap_or(tail.len());
    Some(&tail[..end])
}

fn jin_card_field<'a>(card: &'a str, field: &str) -> Option<&'a str> {
    let marker = format!("\n#### {field}\n");
    let start = card.find(&marker)?;
    let body = &card[start + marker.len()..];
    let end = body.find("\n#### ").unwrap_or(body.len());
    Some(body[..end].trim())
}

fn jin_visible_steps_in_order(body: &str, steps: &[&str]) -> bool {
    let visible = visible_markdown_text(body);
    let mut cursor = 0;
    for step in steps {
        let Some(offset) = visible[cursor..].find(step) else {
            return false;
        };
        cursor += offset + step.len();
    }
    true
}

fn jin_receipt_audit(field: &str) -> Option<BTreeMap<String, String>> {
    let visible = visible_markdown_text(field);
    let payload = visible.split_once("Receipt audit: {")?.1.split_once('}')?.0;
    let mut values = BTreeMap::new();
    for entry in payload.split("; ") {
        let (key, value) = entry.split_once('=')?;
        if values.insert(key.to_owned(), value.to_owned()).is_some() {
            return None;
        }
    }
    Some(values)
}

fn expected_jin_receipt_audit(
    spec: &JinCftSpec,
    active_type_sha256: &str,
) -> BTreeMap<String, String> {
    let mut audit = BTreeMap::from([
        (
            "public-declaration".to_owned(),
            jin_completed_public_declaration(spec).to_owned(),
        ),
        ("public-file".to_owned(), spec.public_file.to_owned()),
        (
            "provider-declaration".to_owned(),
            jin_completed_provider_declaration(spec).to_owned(),
        ),
        (
            "provider-file".to_owned(),
            jin_completed_provider_file(spec).to_owned(),
        ),
        ("type-sha256".to_owned(), active_type_sha256.to_owned()),
        ("axioms".to_owned(), JIN_AXIOMS.join(",")),
        (
            "verification-target".to_owned(),
            "CrouzeixTextbook".to_owned(),
        ),
    ]);
    if matches!(spec.item_id, "CFT-30-006" | "CFT-31-002") {
        audit.insert(
            "provider-type-sha256".to_owned(),
            jin_target_provider_hash(spec).to_owned(),
        );
    }
    audit
}

fn jin_complete_card_errors(markdown: &str, chapter: u64) -> Vec<String> {
    let mut errors = Vec::new();
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let coverage_rows = coverage["items"].as_array().expect("coverage rows");
    for spec in JIN_CFT_SPECS.iter().filter(|spec| spec.chapter == chapter) {
        let item_id = spec.item_id;
        let index = item_id
            .rsplit('-')
            .next()
            .expect("Jin item suffix")
            .parse::<u64>()
            .expect("Jin item index");
        let anchor = format!("{{#cft-{chapter:02}-{index:03}}}");
        if markdown.matches(&anchor).count() != 1 {
            errors.push(format!("{item_id} must have exactly one anchor"));
        }
        let Some(card) = jin_card(markdown, item_id, chapter) else {
            errors.push(format!("{item_id} is missing its theorem card"));
            continue;
        };
        let actual_headings = card
            .lines()
            .filter_map(|line| line.strip_prefix("#### "))
            .collect::<Vec<_>>();
        if actual_headings != JIN_CARD_FIELDS {
            errors.push(format!(
                "{item_id} fields must be exactly {:?}, observed {actual_headings:?}",
                JIN_CARD_FIELDS
            ));
        }
        for field in JIN_CARD_FIELDS {
            let Some(body) = jin_card_field(card, field) else {
                errors.push(format!("{item_id} is missing `{field}`"));
                continue;
            };
            if visible_markdown_text(body).is_empty() {
                errors.push(format!("{item_id} `{field}` has no visible content"));
            }
        }
        for (field, required) in [
            ("Purpose", "Motivation."),
            ("ML analogy", "Mathematical object / ML counterpart."),
            ("ML analogy", "Exact transfer."),
            ("ML analogy", "Non-transfer."),
            ("ML analogy", "Diagnostic."),
        ] {
            if !jin_card_field(card, field)
                .is_some_and(|body| visible_markdown_text(body).contains(required))
            {
                errors.push(format!("{item_id} `{field}` is missing `{required}`"));
            }
        }
        for (field, steps) in [
            ("Hypothesis ledger", spec.hypotheses),
            ("Statement", spec.statement_steps),
            ("Proof", spec.proof_steps),
        ] {
            if !jin_card_field(card, field)
                .is_some_and(|body| jin_visible_steps_in_order(body, steps))
            {
                errors.push(format!(
                    "{item_id} `{field}` omits or misorders scoped obligations {steps:?}"
                ));
            }
        }
        let audit = jin_card_field(card, "Lean correspondence").and_then(jin_receipt_audit);
        let active_type_sha256 = coverage_rows
            .iter()
            .find(|row| row["item_id"].as_str() == Some(spec.item_id))
            .and_then(|row| row["lean_declaration"]["type_sha256"].as_str())
            .unwrap_or(spec.type_sha256);
        if audit.as_ref() != Some(&expected_jin_receipt_audit(spec, active_type_sha256)) {
            errors.push(format!(
                "{item_id} visible receipt audit does not match the exact public/provider map"
            ));
        }
    }
    errors
}

fn jin_pending_card_errors(markdown: &str, chapter: u64) -> Vec<String> {
    let mut errors = Vec::new();
    for spec in JIN_CFT_SPECS.iter().filter(|spec| spec.chapter == chapter) {
        let Some(card) = jin_card(markdown, spec.item_id, chapter) else {
            errors.push(format!("{} is missing its baseline card", spec.item_id));
            continue;
        };
        for field in JIN_CARD_FIELDS {
            let heading = format!("#### {field}");
            let count = card.lines().filter(|line| *line == heading).count();
            if count != 0 {
                errors.push(format!(
                    "{} pending baseline requires `{field}` absent, observed {count}",
                    spec.item_id
                ));
            }
        }
    }
    errors
}

fn jin_phase_metadata_errors(coverage: &Value, exercises: &Value, phase: JinPhase) -> Vec<String> {
    let mut errors = Vec::new();
    let coverage_rows = coverage["items"]
        .as_array()
        .expect("coverage items in phase fixture");
    let exercise_rows = exercises["exercises"]
        .as_array()
        .expect("exercise rows in phase fixture");
    for spec in JIN_CFT_SPECS {
        let matching = coverage_rows
            .iter()
            .filter(|row| row["item_id"].as_str() == Some(spec.item_id))
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            errors.push(format!(
                "{} must have exactly one coverage row, observed {}",
                spec.item_id,
                matching.len()
            ));
            continue;
        }
        let row = matching[0];
        let complete = phase.completes(spec.chapter);
        let expected_kind = if complete {
            spec.kind
        } else {
            jin_baseline_kind(&spec)
        };
        let expected_mode = if complete {
            jin_completed_mode(&spec)
        } else {
            spec.baseline_mode
        };
        let expected_correspondence = if complete {
            "exact"
        } else {
            spec.baseline_correspondence
        };
        let expected_prose = if complete {
            "reconstructible"
        } else {
            "summary"
        };
        for (field, expected) in [
            ("kind", expected_kind),
            ("formal_mode", expected_mode),
            ("lean_correspondence_status", expected_correspondence),
            ("prose_proof_status", expected_prose),
        ] {
            if row[field].as_str() != Some(expected) {
                errors.push(format!(
                    "{} {field} must be `{expected}` in phase `{}`",
                    spec.item_id,
                    phase.label()
                ));
            }
        }
        let declaration = &row["lean_declaration"];
        let expected_public = if complete {
            jin_completed_public_declaration(&spec)
        } else {
            spec.public_declaration
        };
        for (field, expected) in [
            ("name", expected_public),
            ("source_path", spec.public_file),
            ("verification_target", "CrouzeixTextbook"),
        ] {
            if declaration[field].as_str() != Some(expected) {
                errors.push(format!(
                    "{} public declaration {field} must be `{expected}`",
                    spec.item_id
                ));
            }
        }
        let expected_underlying = if complete && jin_completed_mode(&spec) == "proved-here" {
            None
        } else if complete {
            Some(jin_completed_provider_declaration(&spec))
        } else if spec.baseline_correspondence == "unmapped" {
            Some(spec.provider_declaration)
        } else {
            None
        };
        if declaration["underlying_declaration"].as_str() != expected_underlying
            || declaration["type_sha256"].as_str()
                != Some(
                    if complete {
                        jin_target_fixture_hash(&spec)
                    } else {
                        spec.type_sha256.to_owned()
                    }
                    .as_str(),
                )
            || declaration["axioms"] != json!(JIN_AXIOMS)
            || declaration["assumptions"] != json!([])
        {
            errors.push(format!(
                "{} public declaration metadata differs from its exact phase row",
                spec.item_id
            ));
        }

        let suffix = spec
            .item_id
            .rsplit('-')
            .next()
            .expect("Jin CFT suffix")
            .parse::<u64>()
            .expect("Jin CFT index");
        let exercise_id = format!("CFT-{:02}-E{suffix:02}", spec.chapter);
        let matching = exercise_rows
            .iter()
            .filter(|row| row["exercise_id"].as_str() == Some(exercise_id.as_str()))
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            errors.push(format!(
                "{exercise_id} must have exactly one exercise row, observed {}",
                matching.len()
            ));
            continue;
        }
        let exercise = matching[0];
        if exercise["skills"] != json!([spec.item_id]) || !exercise["starter"].is_null() {
            errors.push(format!(
                "{exercise_id} must retain its exact parent skill and null starter"
            ));
        }
        if complete {
            let solution = &exercise["lean_solution"];
            let exercise_spec = JIN_EXERCISE_SPECS
                .iter()
                .find(|candidate| candidate.exercise_id == exercise_id)
                .expect("Jin exercise spec");
            let expected_name = format!(
                "CrouzeixTextbook.Part06.Exercises.Chapter{}.exercise_{suffix:02}_solution",
                spec.chapter
            );
            let expected_file = format!(
                "formalization/lean/CrouzeixTextbook/Part06/Chapter{}.lean",
                spec.chapter
            );
            if solution["declaration"].as_str() != Some(expected_name.as_str())
                || solution["source_path"].as_str() != Some(expected_file.as_str())
                || solution["verification_target"].as_str() != Some("CrouzeixTextbook")
                || solution["type_sha256"].as_str()
                    != Some(jin_exercise_receipt_hash(exercise_spec).as_str())
            {
                errors.push(format!(
                    "{exercise_id} lacks its distinct exact solution metadata in phase `{}`",
                    phase.label()
                ));
            }
        } else if !exercise["lean_solution"].is_null() {
            errors.push(format!(
                "{exercise_id} must keep lean_solution absent in phase `{}`",
                phase.label()
            ));
        }
    }
    errors
}

fn jin_promote_phase_fixture(coverage: &mut Value, exercises: &mut Value, phase: JinPhase) {
    for spec in JIN_CFT_SPECS {
        if !phase.completes(spec.chapter) {
            continue;
        }
        let row = coverage["items"]
            .as_array_mut()
            .expect("coverage items")
            .iter_mut()
            .find(|row| row["item_id"] == spec.item_id)
            .expect("Jin coverage fixture row");
        row["kind"] = json!(spec.kind);
        row["formal_mode"] = json!(jin_completed_mode(&spec));
        row["lean_correspondence_status"] = json!("exact");
        row["prose_proof_status"] = json!("reconstructible");
        row["lean_declaration"]["name"] = json!(jin_completed_public_declaration(&spec));
        row["lean_declaration"]["underlying_declaration"] =
            if jin_completed_mode(&spec) == "proved-here" {
                Value::Null
            } else {
                json!(jin_completed_provider_declaration(&spec))
            };
        row["lean_declaration"]["type_sha256"] = json!(jin_target_fixture_hash(&spec));

        let suffix = spec
            .item_id
            .rsplit('-')
            .next()
            .expect("Jin CFT suffix")
            .parse::<u64>()
            .expect("Jin CFT index");
        let exercise_id = format!("CFT-{:02}-E{suffix:02}", spec.chapter);
        let exercise = exercises["exercises"]
            .as_array_mut()
            .expect("exercise rows")
            .iter_mut()
            .find(|row| row["exercise_id"] == exercise_id)
            .expect("Jin exercise fixture row");
        let exercise_spec = JIN_EXERCISE_SPECS
            .iter()
            .find(|candidate| candidate.exercise_id == exercise_id)
            .expect("Jin exercise spec");
        exercise["lean_solution"] = json!({
            "declaration": format!(
                "CrouzeixTextbook.Part06.Exercises.Chapter{}.exercise_{suffix:02}_solution",
                spec.chapter
            ),
            "source_path": format!(
                "formalization/lean/CrouzeixTextbook/Part06/Chapter{}.lean",
                spec.chapter
            ),
            "verification_target": "CrouzeixTextbook",
            "type_sha256": jin_exercise_receipt_hash(exercise_spec),
        });
    }
}

fn jin_card_field_presence_errors(
    item_id: &str,
    phase: JinPhase,
    observed: &[bool; 10],
) -> Vec<String> {
    let spec = jin_cft_spec(item_id);
    let expected = [phase.completes(spec.chapter); 10];
    JIN_CARD_FIELDS
        .iter()
        .zip(expected)
        .zip(observed)
        .filter(|((_, expected), observed)| *expected != **observed)
        .map(|((field, expected), _)| {
            format!(
                "{item_id} `{field}` presence must be {expected} in phase `{}`",
                phase.label()
            )
        })
        .collect()
}

fn jin_solution_is_compiler_guarded(exporter: &str, declaration: &str) -> bool {
    jin_checked_exercise_names(exporter).is_some_and(|names| names.contains(declaration))
}

fn jin_checked_exercise_names(source: &str) -> Option<BTreeSet<String>> {
    let mut active = String::with_capacity(source.len());
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut block_depth = 0_u64;
    let mut in_string = false;
    while index < bytes.len() {
        if block_depth > 0 {
            if bytes[index..].starts_with(b"/-") {
                block_depth += 1;
                index += 2;
            } else if bytes[index..].starts_with(b"-/") {
                block_depth -= 1;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if !in_string && bytes[index..].starts_with(b"--") {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            active.push('\n');
            continue;
        }
        if !in_string && bytes[index..].starts_with(b"/-") {
            block_depth = 1;
            index += 2;
            continue;
        }
        let byte = bytes[index];
        active.push(char::from(byte));
        if byte == b'"' && (index == 0 || bytes[index - 1] != b'\\') {
            in_string = !in_string;
        }
        index += 1;
    }
    if block_depth != 0 || in_string {
        return None;
    }
    let body = active
        .split_once("def checkedExerciseTheoremNames")?
        .1
        .split_once("#[")?
        .1
        .split_once(']')?
        .0;
    let mut names = BTreeSet::new();
    for token in body.split(',') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let name = token.strip_prefix('`')?;
        if name.is_empty() || !names.insert(name.to_owned()) {
            return None;
        }
    }
    Some(names)
}

fn jin_type_assumes_its_conclusion(normalized_type: &str) -> bool {
    let conclusion = normalized_type
        .rsplit_once(',')
        .map_or(normalized_type, |(_, conclusion)| conclusion)
        .trim();
    let Some((assumption, result)) = conclusion.rsplit_once(" → ") else {
        return false;
    };
    let assumption = assumption
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();
    let result = result
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();
    assumption == result
}

fn jin_transitive_dependencies(
    declarations: &[Value],
    root: &str,
    visited: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    let Some(row) = declarations
        .iter()
        .find(|row| row["name"].as_str() == Some(root))
    else {
        return BTreeSet::new();
    };
    let mut closure = BTreeSet::new();
    for dependency in row["direct_dependencies"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        closure.insert(dependency.to_owned());
        if visited.insert(dependency.to_owned()) {
            closure.extend(jin_transitive_dependencies(
                declarations,
                dependency,
                visited,
            ));
        }
    }
    closure
}

fn jin_exercise_receipt_errors(receipt: &Value, exporter: &str, phase: JinPhase) -> Vec<String> {
    let mut errors = Vec::new();
    let declarations = receipt["declarations"]
        .as_array()
        .expect("exercise receipt declarations");
    let exported = jin_checked_exercise_names(exporter).unwrap_or_default();
    for chapter in 30..=32 {
        if !phase.completes(chapter) {
            continue;
        }
        let prefix = format!("CrouzeixTextbook.Part06.Exercises.Chapter{chapter}.");
        let actual = declarations
            .iter()
            .filter_map(|row| row["name"].as_str())
            .filter(|name| {
                name.contains(&prefix) && (!name.contains("._") || name.starts_with("_private."))
            })
            .collect::<BTreeSet<_>>();
        let expected = JIN_EXERCISE_SPECS
            .iter()
            .filter(|spec| spec.exercise_id[4..6].parse::<u64>() == Ok(chapter))
            .map(|spec| spec.solution)
            .collect::<BTreeSet<_>>();
        let exported_for_chapter = exported
            .iter()
            .map(String::as_str)
            .filter(|name| name.starts_with(&prefix))
            .collect::<BTreeSet<_>>();
        if actual != expected || exported_for_chapter != expected {
            errors.push(format!(
                "Chapter{chapter} exercise namespace/exporter must contain exactly six declarations: expected {expected:?}, receipt {actual:?}, exporter {exported_for_chapter:?}"
            ));
        }
    }
    for spec in JIN_EXERCISE_SPECS {
        let chapter = spec
            .exercise_id
            .split('-')
            .nth(1)
            .expect("exercise chapter")
            .parse::<u64>()
            .expect("exercise chapter number");
        let matches = declarations
            .iter()
            .filter(|row| row["name"].as_str() == Some(spec.solution))
            .collect::<Vec<_>>();
        if !phase.completes(chapter) {
            if !matches.is_empty() || exported.contains(spec.solution) {
                errors.push(format!(
                    "{} must have no receipt/exporter solution before Chapter {chapter} completes",
                    spec.exercise_id
                ));
            }
            continue;
        }
        if matches.len() != 1 {
            errors.push(format!(
                "{} must have exactly one compiler receipt row, observed {}",
                spec.exercise_id,
                matches.len()
            ));
            continue;
        }
        let declaration = matches[0];
        if declaration["kind"].as_str() != Some("theorem") {
            errors.push(format!(
                "{} must elaborate as theorem, never direct/eta alias",
                spec.exercise_id
            ));
        }
        if !exported.contains(spec.solution) {
            errors.push(format!(
                "{} is absent from the active checkedExerciseTheoremNames array",
                spec.exercise_id
            ));
        }
        let normalized_type = declaration["normalized_type"].as_str().unwrap_or("");
        if normalized_type.trim() == "True" || normalized_type.ends_with(": True") {
            errors.push(format!("{} proves only True", spec.exercise_id));
        }
        if jin_type_assumes_its_conclusion(normalized_type) {
            errors.push(format!(
                "{} assumes its conclusion as an input",
                spec.exercise_id
            ));
        }
        let target = jin_contract_probe_row(spec.probe_declaration);
        if declaration["type_sha256"].as_str() != Some(jin_exercise_receipt_hash(&spec).as_str()) {
            errors.push(format!(
                "{} must match its authoritative compiler-receipt type hash",
                spec.exercise_id
            ));
        }
        let parent = jin_cft_spec(spec.parent);
        let signature_dependencies = target["direct_dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>();
        let mut dependencies = BTreeSet::new();
        for dependency in declaration["direct_dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|dependency| !signature_dependencies.contains(dependency))
        {
            dependencies.insert(dependency.to_owned());
            dependencies.extend(jin_transitive_dependencies(
                declarations,
                dependency,
                &mut BTreeSet::from([spec.solution.to_owned(), dependency.to_owned()]),
            ));
        }
        let active_parent_public = jin_phase_public_declaration(parent, phase);
        let active_parent_provider = jin_phase_provider_declaration(parent, phase);
        if dependencies.contains(active_parent_public)
            || dependencies.contains(active_parent_provider)
        {
            errors.push(format!(
                "{} directly reuses its parent checkpoint/provider",
                spec.exercise_id
            ));
        }
        for required in jin_required_exercise_dependencies(&spec) {
            if !dependencies.contains(*required) {
                errors.push(format!(
                    "{} omits required compiler dependency `{required}`",
                    spec.exercise_id
                ));
            }
        }
        for dependency in &dependencies {
            if let Some(cft) = JIN_CFT_SPECS.iter().find(|candidate| {
                jin_phase_public_declaration(candidate, phase) == *dependency
                    || jin_phase_provider_declaration(candidate, phase) == *dependency
            }) {
                if !spec.allowed_prerequisites.contains(&cft.item_id) {
                    errors.push(format!(
                        "{} uses forbidden Jin dependency `{dependency}` ({})",
                        spec.exercise_id, cft.item_id
                    ));
                }
            }
        }
        for terminal in [
            "CrouzeixConjecture.completion_PRX_congruence_eq_gramian_expression",
            "CrouzeixConjecture.jinFinalCrouzeixConjecture",
            "CrouzeixConjecture.crouzeixRationalBound",
            "CrouzeixConjecture.holomorphicCrouzeixBound",
            "CrouzeixConjecture.polynomialCrouzeixBound_of_holomorphicCrouzeixBound",
            "CrouzeixTextbook.Part06.jin_polynomial_constant_two",
            "CrouzeixTextbook.Part06.jin_rational_constant_two",
            "CrouzeixTextbook.Part06.holomorphic_constant_two",
            "CrouzeixTextbook.Part06.polynomial_from_holomorphic",
        ] {
            if dependencies.contains(terminal) {
                errors.push(format!(
                    "{} uses transitive forbidden terminal dependency `{terminal}`",
                    spec.exercise_id
                ));
            }
        }
    }
    errors
}

fn jin_exercise_receipt_fixture(phase: JinPhase) -> (Value, String) {
    let mut declarations = Vec::new();
    let mut names = Vec::new();
    for spec in JIN_EXERCISE_SPECS {
        let chapter = spec
            .exercise_id
            .split('-')
            .nth(1)
            .expect("exercise chapter")
            .parse::<u64>()
            .expect("exercise chapter number");
        if phase.completes(chapter) {
            let target = jin_contract_probe_row(spec.probe_declaration);
            let normalized_type = target["normalized_type"]
                .as_str()
                .expect("exercise probe normalized type")
                .to_owned();
            declarations.push(json!({
                "name": spec.solution,
                "kind": "theorem",
                "normalized_type": normalized_type,
                "type_sha256": jin_exercise_receipt_hash(&spec),
                "direct_dependencies": jin_required_exercise_dependencies(&spec),
            }));
            names.push(format!("`{}", spec.solution));
        }
    }
    (
        json!({"declarations": declarations}),
        format!(
            "def checkedExerciseTheoremNames : Array Name := #[{}]\n",
            names.join(", ")
        ),
    )
}

fn jin_mapping_receipt_fixture(phase: JinPhase) -> Value {
    let mut declarations = Vec::new();
    for spec in JIN_CFT_SPECS {
        let complete = phase.completes(spec.chapter);
        let public = if complete {
            jin_completed_public_declaration(&spec)
        } else {
            spec.public_declaration
        };
        let provider = if complete {
            jin_completed_provider_declaration(&spec)
        } else {
            spec.provider_declaration
        };
        let provider_file = if complete {
            jin_completed_provider_file(&spec)
        } else {
            spec.provider_file
        };
        let type_sha256 = if complete {
            jin_target_fixture_hash(&spec)
        } else {
            spec.type_sha256.to_owned()
        };
        let normalized_type = if complete && spec.item_id == "CFT-31-002" {
            jin_contract_probe_row(JIN_CFT_31_002_PROBE_DECLARATION)["normalized_type"]
                .as_str()
                .expect("CFT-31-002 target normalized type")
                .to_owned()
        } else {
            format!(
                "compiler-normalized-{}-{}",
                if complete { "target" } else { "baseline" },
                spec.item_id
            )
        };
        declarations.push(json!({
            "name": public,
            "kind": if complete { jin_completed_receipt_kind(&spec) } else { "direct-alias" },
            "normalized_type": normalized_type,
            "type_sha256": type_sha256,
            "axioms": JIN_AXIOMS,
            "source_path": spec.public_file,
            "direct_dependencies": [provider],
        }));
        let provider_hash = if complete {
            jin_target_provider_hash(&spec)
        } else {
            jin_baseline_provider_hash(&spec).to_owned()
        };
        let provider_normalized_type = if provider_hash == type_sha256 {
            normalized_type.clone()
        } else {
            format!("compiler-normalized-provider-{}", spec.item_id)
        };
        if !declarations
            .iter()
            .any(|row| row["name"].as_str() == Some(provider))
        {
            declarations.push(json!({
                "name": provider,
                "kind": if complete {
                    jin_target_provider_kind(&spec)
                } else {
                    jin_baseline_provider_kind(&spec)
                },
                "normalized_type": provider_normalized_type,
                "type_sha256": provider_hash,
                "axioms": JIN_AXIOMS,
                "source_path": provider_file,
                "direct_dependencies": [],
            }));
        }
    }
    json!({"target": "CrouzeixTextbook", "declarations": declarations})
}

fn jin_lean_mapping_errors(coverage: &Value, receipt: &Value, phase: JinPhase) -> Vec<String> {
    let mut errors = Vec::new();
    let coverage_rows = coverage["items"].as_array().expect("mapping coverage rows");
    let declarations = receipt["declarations"]
        .as_array()
        .expect("mapping receipt declarations");
    for spec in JIN_CFT_SPECS {
        let complete = phase.completes(spec.chapter);
        let public_name = if complete {
            jin_completed_public_declaration(&spec)
        } else {
            spec.public_declaration
        };
        let provider_name = if complete {
            jin_completed_provider_declaration(&spec)
        } else {
            spec.provider_declaration
        };
        let provider_file = if complete {
            jin_completed_provider_file(&spec)
        } else {
            spec.provider_file
        };
        let public_rows = declarations
            .iter()
            .filter(|row| row["name"].as_str() == Some(public_name))
            .collect::<Vec<_>>();
        if public_rows.len() != 1 {
            errors.push(format!(
                "{} must have exactly one public row `{public_name}`, observed {}",
                spec.item_id,
                public_rows.len()
            ));
            continue;
        }
        let provider_rows = declarations
            .iter()
            .filter(|row| row["name"].as_str() == Some(provider_name))
            .collect::<Vec<_>>();
        if provider_rows.len() != 1 {
            errors.push(format!(
                "{} must have exactly one provider row `{provider_name}`, observed {}",
                spec.item_id,
                provider_rows.len()
            ));
            continue;
        }
        let public = public_rows[0];
        let provider = provider_rows[0];
        let expected_kind = if complete {
            jin_completed_receipt_kind(&spec)
        } else {
            "direct-alias"
        };
        if public["kind"].as_str() != Some(expected_kind)
            || public["source_path"].as_str() != Some(spec.public_file)
            || public["axioms"] != json!(JIN_AXIOMS)
        {
            errors.push(format!(
                "{} public receipt kind/file/axioms differ from the active mapping",
                spec.item_id
            ));
        }
        let expected_provider_kind = if complete {
            jin_target_provider_kind(&spec)
        } else {
            jin_baseline_provider_kind(&spec)
        };
        if provider["kind"].as_str() != Some(expected_provider_kind) {
            errors.push(format!(
                "{} requires exactly one {}-kind provider, observed {:?}",
                spec.item_id, expected_provider_kind, provider["kind"]
            ));
        }
        let expected_provider_hash = if complete {
            jin_target_provider_hash(&spec)
        } else {
            jin_baseline_provider_hash(&spec).to_owned()
        };
        if provider["source_path"].as_str() != Some(provider_file)
            || provider["axioms"] != json!(JIN_AXIOMS)
            || provider["type_sha256"].as_str() != Some(expected_provider_hash.as_str())
        {
            errors.push(format!(
                "{} provider receipt differs from its exact frozen file/hash/axioms/type contract: public={{file:{:?},hash:{:?},axioms:{:?},type:{:?}}}, provider={{file:{:?},hash:{:?},axioms:{:?},type:{:?}}}",
                spec.item_id,
                public["source_path"], public["type_sha256"], public["axioms"], public["normalized_type"],
                provider["source_path"], provider["type_sha256"], provider["axioms"], provider["normalized_type"],
            ));
        }
        let direct_dependencies = public["direct_dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>();
        if !direct_dependencies.contains(provider_name)
            || (expected_kind == "direct-alias"
                && direct_dependencies != BTreeSet::from([provider_name]))
            || (spec.item_id == "CFT-31-002"
                && complete
                && direct_dependencies.contains(spec.provider_declaration))
        {
            errors.push(format!(
                "{} public receipt violates its exact provider/dependency contract",
                spec.item_id
            ));
        }
        let coverage_rows = coverage_rows
            .iter()
            .filter(|row| row["item_id"].as_str() == Some(spec.item_id))
            .collect::<Vec<_>>();
        if coverage_rows.len() != 1 {
            errors.push(format!(
                "{} must have exactly one active coverage row",
                spec.item_id
            ));
            continue;
        }
        let declaration = &coverage_rows[0]["lean_declaration"];
        if declaration["name"] != public["name"]
            || declaration["source_path"] != public["source_path"]
            || declaration["type_sha256"] != public["type_sha256"]
            || declaration["axioms"] != public["axioms"]
            || declaration["verification_target"].as_str() != Some("CrouzeixTextbook")
        {
            errors.push(format!(
                "{} active coverage mapping differs from the fresh compiler receipt",
                spec.item_id
            ));
        }
    }
    errors
}

fn jin_graph_reaches(
    graph: &BTreeMap<String, BTreeSet<String>>,
    start: &str,
    target: &str,
    visited: &mut BTreeSet<String>,
) -> bool {
    if !visited.insert(start.to_owned()) {
        return false;
    }
    start == target
        || graph.get(start).is_some_and(|next| {
            next.iter()
                .any(|node| jin_graph_reaches(graph, node, target, visited))
        })
}

fn jin_routes_are_pedagogically_independent(graph: &BTreeMap<String, BTreeSet<String>>) -> bool {
    let jin = (30..=32)
        .flat_map(|chapter| (1..=6).map(move |index| format!("CFT-{chapter:02}-{index:03}")))
        .collect::<Vec<_>>();
    let ls = (33..=34)
        .flat_map(|chapter| (1..=6).map(move |index| format!("CFT-{chapter:02}-{index:03}")))
        .collect::<Vec<_>>();
    jin.iter().all(|jin_node| {
        ls.iter().all(|ls_node| {
            !jin_graph_reaches(graph, jin_node, ls_node, &mut BTreeSet::new())
                && !jin_graph_reaches(graph, ls_node, jin_node, &mut BTreeSet::new())
        })
    })
}

fn run_jin_closure(lean_root: &Path) -> std::process::Output {
    let script = r#"
import sys
from pathlib import Path
repo = Path(sys.argv[1])
sys.path.insert(0, str(repo / 'labs' / 'crouzeix_proof_reproduction'))
import proof_evidence
try:
    closure = proof_evidence.gather_route_closure(
        Path(sys.argv[2]),
        proof_evidence.ROUTE_POLICIES['jin'],
        cache_identity='contract-test',
        toolchain=proof_evidence.PINNED_TOOLCHAIN,
    )
except proof_evidence.PreflightError as error:
    print(error.reason, file=sys.stderr)
    raise SystemExit(7)
print('\n'.join(closure))
"#;
    Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(workspace_root())
        .arg(lean_root)
        .output()
        .expect("run canonical Jin closure parser")
}

fn fresh_provider_receipt_rows(phase: JinPhase) -> Vec<Value> {
    let temp = tempfile::tempdir().expect("fresh provider probe directory");
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .expect("private provider probe directory permissions");
    let source_path = temp.path().join("JinProviderProbe.lean");
    let provider_names = JIN_CFT_SPECS
        .iter()
        .map(|spec| {
            if phase.completes(spec.chapter) {
                jin_completed_provider_declaration(spec)
            } else {
                spec.provider_declaration
            }
        })
        .collect::<BTreeSet<_>>();
    let quoted_names = provider_names
        .iter()
        .map(|name| format!("`{name}"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        "import CrouzeixTextbook.ExportReceipt\n\
         open Lean\n\
         run_cmd do\n\
           let env ← getEnv\n\
           let names : Array Name := #[{quoted_names}]\n\
           let rows ← Lean.Elab.Command.liftCoreM <|\n\
             CrouzeixTextbook.ExportReceipt.receiptRows env names\n\
           IO.println s!\"JIN_PROVIDER_RECEIPT:{{(Json.arr rows).compress}}\"\n"
    );
    fs::write(&source_path, source).expect("write test-only provider probe");
    let output = textbook_lean_command("lake")
        .arg("env")
        .arg("lean")
        .arg(&source_path)
        .current_dir(workspace_root().join("formalization/lean"))
        .output()
        .expect("run fresh provider compiler probe");
    assert!(
        output.status.success(),
        "fresh provider compiler probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("provider probe UTF-8");
    let payload = stdout
        .lines()
        .find_map(|line| line.strip_prefix("JIN_PROVIDER_RECEIPT:"))
        .expect("provider probe JSON marker");
    serde_json::from_str::<Value>(payload)
        .expect("provider probe JSON")
        .as_array()
        .expect("provider probe rows")
        .clone()
}

fn fresh_textbook_receipt() -> Value {
    let temp = tempfile::tempdir().expect("fresh textbook receipt directory");
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .expect("private receipt directory permissions");
    let receipt_path = temp.path().join("receipt.json");
    let output = textbook_lean_command(workspace_root().join("scripts/check_lean_library.sh"))
        .arg("CrouzeixTextbook")
        .arg("--receipt-output")
        .arg(&receipt_path)
        .output()
        .expect("run fresh CrouzeixTextbook receipt build");
    assert!(
        output.status.success(),
        "fresh CrouzeixTextbook receipt build failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let mut receipt = read_json(&receipt_path);
    let phase = jin_contract_phase(&jin_route_contract());
    let declarations = receipt["declarations"]
        .as_array_mut()
        .expect("fresh textbook receipt declarations");
    let existing = declarations
        .iter()
        .filter_map(|row| row["name"].as_str().map(ToOwned::to_owned))
        .collect::<BTreeSet<_>>();
    declarations.extend(
        fresh_provider_receipt_rows(phase)
            .into_iter()
            .filter(|row| !existing.contains(row["name"].as_str().unwrap_or_default())),
    );
    declarations.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    receipt
}

fn backticked_field<'a>(markdown: &'a str, label: &str) -> &'a str {
    markdown
        .split_once(label)
        .unwrap_or_else(|| panic!("missing field `{label}`"))
        .1
        .split_once('`')
        .unwrap_or_else(|| panic!("missing opening backtick after `{label}`"))
        .1
        .split_once('`')
        .unwrap_or_else(|| panic!("missing closing backtick after `{label}`"))
        .0
}

fn assert_math_steps_in_order(text: &str, label: &str, steps: &[&str]) {
    let mut cursor = 0;
    for step in steps {
        let offset = text[cursor..]
            .find(step)
            .unwrap_or_else(|| panic!("{label} is missing or misorders `{step}`"));
        cursor += offset + step.len();
    }
}

fn claim_section<'a>(ledger: &'a str, claim_id: &str) -> &'a str {
    let marker = format!("## {claim_id}:");
    let start = ledger
        .find(&marker)
        .unwrap_or_else(|| panic!("claim ledger is missing {claim_id}"));
    let tail = &ledger[start..];
    let end = tail[marker.len()..]
        .find("\n## ")
        .map(|offset| marker.len() + offset)
        .unwrap_or(tail.len());
    &tail[..end]
}

fn claim_fields(section: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    let mut current = None::<String>;
    for line in section.lines().skip(1) {
        if let Some(field) = line.strip_prefix("- ") {
            let (name, value) = field
                .split_once(':')
                .unwrap_or_else(|| panic!("malformed claim field `{line}`"));
            fields.insert(name.to_owned(), value.trim().to_owned());
            current = Some(name.to_owned());
        } else if !line.trim().is_empty() {
            let name = current
                .as_ref()
                .unwrap_or_else(|| panic!("claim continuation without field: `{line}`"));
            let value = fields.get_mut(name).expect("current claim field");
            value.push(' ');
            value.push_str(line.trim());
        }
    }
    fields
}

fn validate_ls_status_boundary(fields: &BTreeMap<String, String>) -> Result<(), String> {
    let expected = BTreeMap::from([
        (
            "Checked terminal provider".to_owned(),
            "`CrouzeixConjecture.loristSchwenningerMainTheorem`".to_owned(),
        ),
        (
            "Terminal source".to_owned(),
            "`formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean`".to_owned(),
        ),
        (
            "Fixed-domain provider".to_owned(),
            "`CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two`".to_owned(),
        ),
        (
            "Current verification".to_owned(),
            "[[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-003|CFT-CL-003]]"
                .to_owned(),
        ),
        (
            "Pinned source claim".to_owned(),
            "[[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-006|CFT-CL-006]]"
                .to_owned(),
        ),
        (
            "Source-graph authority".to_owned(),
            "`locator-and-dated-historical-status-only`".to_owned(),
        ),
        (
            "Publication status".to_owned(),
            "`not-established-by-local-compilation`".to_owned(),
        ),
        (
            "Peer-review status".to_owned(),
            "`not-established-by-local-compilation`".to_owned(),
        ),
    ]);
    if fields == &expected {
        Ok(())
    } else {
        Err(format!("LS status boundary mismatch: {fields:?}"))
    }
}

fn has_explicit_heading_anchor(markdown: &str, anchor: &str) -> bool {
    let marker = format!("{{#{anchor}}}");
    markdown.lines().any(|line| {
        line.trim_start().starts_with('#') && line.split_whitespace().any(|word| word == marker)
    })
}

#[test]
fn chapter_33_prose_has_six_unique_complete_theorem_cards() {
    let markdown = chapter_33_markdown();
    assert!(
        !has_explicit_heading_anchor(&markdown, "the-six-item-spine"),
        "Chapter 33 must not route all theorem cards through a generic spine"
    );
    assert!(
        !has_explicit_heading_anchor(
            "a prose mention of {#the-six-item-spine} is not a heading",
            "the-six-item-spine"
        ),
        "ordinary prose must not be mistaken for the retired anchor"
    );
    assert!(
        has_explicit_heading_anchor("### Retired {#the-six-item-spine}", "the-six-item-spine"),
        "the exact retired anchor mutation must be detected"
    );
    let fields = [
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
    ];
    for index in 1..=6 {
        let item_id = format!("CFT-33-{index:03}");
        let anchor = format!("{{#cft-33-{index:03}}}");
        assert_eq!(
            markdown.matches(&anchor).count(),
            1,
            "{item_id} needs one unique canonical anchor"
        );
        let card = chapter_33_card(&markdown, &item_id);
        let mut cursor = 0;
        for field in fields {
            let heading = format!("#### {field}");
            let heading_line = format!("\n{heading}\n");
            assert_eq!(
                card.lines().filter(|line| *line == heading).count(),
                1,
                "{item_id} needs exactly one `{heading}` field"
            );
            let offset = card[cursor..]
                .find(&heading_line)
                .unwrap_or_else(|| panic!("{item_id} has `{heading}` out of order"));
            cursor += offset + heading_line.len();
        }
    }
}

#[test]
fn chapter_33_prose_records_exact_notation_and_pinned_source_locators() {
    let markdown = chapter_33_markdown();
    for required in [
        "## Notation ledger",
        "`E`, `K`",
        "`T`",
        "`V`, `Q`",
        "`E_n`",
        "`κ = ‖T‖`",
        "`T*T x = κ²x`",
        "`d = Q*VTx - κVx`",
        "`b = ‖d‖²`",
        "`m_n = Re ⟪E_n x, (T*)^n x⟫`",
        "conjugate-linear in its first argument and linear in its second",
        "`DilationData.perturbation`",
        "`recurrenceScalar`",
        "`DilationData.displacement`",
        "`DilationData.displacementSq`",
        "`DilationData.firstPerturbationMoment`",
    ] {
        assert!(
            markdown.contains(required),
            "missing notation fact `{required}`"
        );
    }
    let registry = fs::read_to_string(packet_root().join("source_registry.md"))
        .expect("textbook source registry");
    for locator in [
        "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99",
        "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L74-L90",
        "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98",
    ] {
        assert!(
            registry.contains(locator),
            "source registry is missing `{locator}`"
        );
        assert!(
            markdown.contains(locator),
            "Chapter 33 is missing `{locator}`"
        );
    }
}

#[test]
fn chapter_33_prose_routes_labeled_claims_to_structured_ledger_entries() {
    let markdown = chapter_33_markdown();
    for claim_id in ["CFT-CL-003", "CFT-CL-006"] {
        let route = format!(
            "[[knowledge/crouzeix_textbook/claim_evidence_ledger#{}|{claim_id}]]",
            claim_id.to_ascii_lowercase()
        );
        assert!(markdown.contains(&route), "Chapter 33 is missing `{route}`");
    }

    let ledger = fs::read_to_string(packet_root().join("claim_evidence_ledger.md"))
        .expect("textbook claim ledger");
    let evidence_section = claim_section(&ledger, "CFT-CL-003");
    let evidence = claim_fields(evidence_section);
    let source_section = claim_section(&ledger, "CFT-CL-006");
    let source = claim_fields(source_section);
    let required_fields = BTreeSet::from([
        "Caveat",
        "Class",
        "Confidence",
        "Confidence basis",
        "Locator",
        "Mode",
        "Reproduction",
        "Scope",
        "Source",
        "Source stability",
        "Statement",
    ]);
    assert_eq!(
        evidence.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        required_fields
    );
    assert_eq!(
        source.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        required_fields
    );
    assert_eq!(evidence["Class"], "`EVIDENCE`");
    assert!(evidence["Statement"].contains("DilationData.norm_target_le_two"));
    assert!(evidence["Locator"].contains("PerturbationLemma.lean"));
    assert!(evidence["Reproduction"].contains("mise run lean-crouzeix-ls"));
    assert!(evidence["Caveat"].contains("peer review"));
    assert!(evidence["Caveat"].contains("publication"));

    assert_eq!(source["Class"], "`SOURCE CLAIM`");
    assert!(source["Source"].contains("LS-ARXIV-V1"));
    assert!(source["Locator"].contains("source-graph.json"));
    assert!(source["Caveat"].contains("local compilation"));
    assert!(source["Caveat"].contains("peer review"));

    let intermediate = claim_section(&ledger, "CFT-CL-005");
    let intermediate = claim_fields(intermediate);
    assert_eq!(intermediate["Class"], "`EVIDENCE`");
    assert!(intermediate["Source"].contains("LS-ARXIV-V1"));
    assert!(intermediate["Locator"].contains("receipt.json"));
    assert!(intermediate["Locator"].contains("source-graph.json"));
    assert!(intermediate["Scope"].contains("dated receipt"));
    assert!(intermediate["Scope"].contains("CFT-CL-003"));

    let status = fs::read_to_string(packet_root().join("status_and_scope.md"))
        .expect("textbook status and scope");
    let boundary = claim_fields(claim_section(
        &status,
        "Lorist--Schwenninger verification boundary",
    ));
    validate_ls_status_boundary(&boundary).unwrap_or_else(|error| panic!("{error}"));

    let reworded_stale_mutation = BTreeMap::from([
        (
            "Current local provider".to_owned(),
            "`an-intermediate-slice-only`".to_owned(),
        ),
        (
            "Current verification".to_owned(),
            "the endpoint is still awaiting completion".to_owned(),
        ),
        (
            "Pinned source claim".to_owned(),
            "[[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-006|CFT-CL-006]]"
                .to_owned(),
        ),
        (
            "Source-graph authority".to_owned(),
            "`current-local-build-status`".to_owned(),
        ),
        (
            "Publication status".to_owned(),
            "`not-established-by-local-compilation`".to_owned(),
        ),
        (
            "Peer-review status".to_owned(),
            "`not-established-by-local-compilation`".to_owned(),
        ),
    ]);
    assert!(validate_ls_status_boundary(&reworded_stale_mutation).is_err());
}

#[test]
fn status_and_claim_ledgers_name_checked_ls_and_harp_terminal_boundaries() {
    let status = fs::read_to_string(packet_root().join("status_and_scope.md"))
        .expect("textbook status and scope");
    let ledger = fs::read_to_string(packet_root().join("claim_evidence_ledger.md"))
        .expect("textbook claim ledger");
    for document in [&status, &ledger] {
        assert!(document.contains("CrouzeixConjecture.loristSchwenningerMainTheorem"));
        assert!(document.contains("CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem"));
        assert!(
            document.contains("formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean")
        );
        assert!(document.contains("formalization/lean/Crouzeix/Harp/MainTheorem.lean"));
        assert!(document.contains("Harp-derived"));
        assert!(document.contains("not source-derived"));
    }
}

#[test]
fn chapter_33_prose_matches_the_scalar_hypothesis_boundary() {
    let markdown = chapter_33_markdown();
    let combined = chapter_33_card(&markdown, "CFT-33-004");
    let endpoint = chapter_33_card(&markdown, "CFT-33-005");
    let combined_hypotheses = chapter_33_card_field(combined, "Hypothesis ledger");
    let endpoint_hypotheses = chapter_33_card_field(endpoint, "Hypothesis ledger");
    assert!(combined_hypotheses.contains("`κ > 0`"));
    assert!(combined_hypotheses.contains("`(κ-1)² > 0`"));
    assert!(
        !combined_hypotheses.contains("`b ≥ 0`"),
        "CFT-33-004 must match scalar_combined_inequality_of_recurrence_bounds"
    );
    assert!(
        endpoint_hypotheses.contains("`b ≥ 0`"),
        "CFT-33-005 must add the scalar endpoint's nonnegativity hypothesis"
    );
}

#[test]
fn chapter_33_cft_001_002_reconstruct_the_recurrence_and_completed_square() {
    let markdown = chapter_33_markdown();
    let identity = chapter_33_card(&markdown, "CFT-33-001");
    let identity_proof = chapter_33_card_field(identity, "Proof");

    assert_math_steps_in_order(
        identity_proof,
        "CFT-33-001 recurrenceScalar bridge at n",
        &[
            "m_n = Re ⟪x, (E_n T^n)x⟫",
            "E_n T^n = T^n E_n",
            "= Re ⟪x, T^n(E_nx)⟫",
            "= Re ⟪(T*)^n x, E_nx⟫",
            "= Re ⟪E_nx, (T*)^n x⟫",
        ],
    );
    assert_math_steps_in_order(
        identity_proof,
        "CFT-33-001 recurrenceScalar bridge at n+1",
        &[
            "m_{n+1} = Re ⟪x, (E_{n+1} T^(n+1))x⟫",
            "E_{n+1} T^(n+1) = T^(n+1) E_{n+1}",
            "= Re ⟪x, T^(n+1)(E_{n+1}x)⟫",
            "= Re ⟪(T*)^(n+1)x, E_{n+1}x⟫",
            "= Re ⟪E_{n+1}x, (T*)^(n+1)x⟫",
        ],
    );
    for commutation in [
        "E_n T^n = T^n E_n",
        "E_{n+1} T^(n+1) = T^(n+1) E_{n+1}",
        "T E_{n+1}x = E_{n+1}Tx",
    ] {
        assert!(
            identity_proof.contains(commutation),
            "CFT-33-001 omits the distinct commutation `{commutation}`"
        );
    }
    for step in [
        "Re ⟪T E_{n+1}x, y_n⟫",
        "T E_{n+1}x = E_{n+1}Tx",
        "(T*)^(n+1)Tx = κ²y_n",
        "adjacentPowerDefect κ n x",
        "κ m_n - m_{n+1}",
        "(κ²-κ) ‖y_n‖²",
    ] {
        assert!(
            identity_proof.contains(step),
            "CFT-33-001 is missing `{step}`"
        );
    }

    let lower = chapter_33_card(&markdown, "CFT-33-002");
    let lower_proof = chapter_33_card_field(lower, "Proof");
    for step in [
        "a = κ²-κ",
        "z = adjacentPowerDefect κ n x",
        "a‖u - z/(2a)‖² - ‖z‖²/(4a)",
        "-‖z‖²/(4a)",
        "‖z‖ ≤ 2‖d‖",
        "-‖d‖²/a ≤ -‖z‖²/(4a)",
        "-‖d‖²/(κ²-κ) ≤ κ m_n-m_{n+1}",
        "one-dimensional scalar check",
        "Equality",
    ] {
        assert!(lower_proof.contains(step), "CFT-33-002 is missing `{step}`");
    }
    let lower_visible = visible_markdown_text(lower_proof);
    assert_math_steps_in_order(
        &lower_visible,
        "CFT-33-002 defect factorization and contraction chain",
        &[
            "z_n = 2 V* (Q*)^n d",
            "‖z_n‖ = 2‖V*(Q*)^n d‖",
            "≤ 2‖(Q*)^n d‖",
            "≤ 2‖d‖",
        ],
    );
    let lower_lean = chapter_33_card_field(lower, "Lean correspondence");
    for required in [
        "adjacentPowerDefect_factorization",
        "adjacentPowerDefect_norm_le",
        "[[formalization/lean/Crouzeix/LoristSchwenninger/Dilation.lean|Dilation.lean]]",
    ] {
        assert!(
            lower_lean.contains(required),
            "CFT-33-002 Lean audit is missing `{required}`"
        );
    }
}

#[test]
fn chapter_33_cft_003_reconstructs_finite_telescoping_before_the_limit() {
    let markdown = chapter_33_markdown();
    let card = chapter_33_card(&markdown, "CFT-33-003");
    let proof = chapter_33_card_field(card, "Proof");
    let ml_analogy = chapter_33_card_field(card, "ML analogy");

    assert_math_steps_in_order(
        proof,
        "CFT-33-003 finite inequality before its limit",
        &[
            "r_n ≤ κm_n-m_{n+1}",
            "r_n/κ^(n+1) ≤ m_n/κ^n - m_{n+1}/κ^(n+1)",
            "∑_{n=1}^N r_n/κ^(n+1) ≤ m_1/κ - m_{N+1}/κ^(N+1)",
            "Let `N→∞`",
        ],
    );
    for required in [
        "B = data.bound ≥ 0",
        "‖E_n‖ ≤ B",
        "‖T^n‖ ≤ 2+B",
        "|m_n| ≤ ‖x‖ ‖E_n T^n x‖ ≤ ‖E_n‖ ‖T^n‖ ≤ B(2+B)",
        "‖x‖=1",
        "M = B(2+B)",
        "|m_n| ≤ M",
        "κ^(n+1) > 0",
        "κ^(n+1) ≠ 0",
        "|m_{N+1}/κ^(N+1)| ≤ M/κ^(N+1)",
        "M/κ^(N+1) → 0",
        "∑_{n=1}^∞ 1/κ^(n+1) = 1/[κ(κ-1)]",
        "κ²-κ = κ(κ-1)",
        "-b/[κ(κ-1)²] ≤ Re ⟪x, E_1 T x⟫",
        "scalar check",
        "κ = 1",
    ] {
        assert!(
            proof.contains(required),
            "CFT-33-003 is missing `{required}`"
        );
    }

    assert_math_steps_in_order(
        ml_analogy,
        "CFT-33-003 labeled ML analogy",
        &[
            "**Exact transfer.**",
            "**Non-transfer.**",
            "**Diagnostic.**",
        ],
    );
    let normalized_ml_analogy = ml_analogy.split_whitespace().collect::<Vec<_>>().join(" ");
    for required in [
        "reciprocal-gain weighting",
        "finite telescoping certificate",
        "stochastic, nonlinear, or time-varying recurrences",
        "pathwise or probabilistic bounds",
        "weighted terminal term",
        "finite weighted residual sum",
        "reject the analogy",
        "terminal term does not decay",
    ] {
        assert!(
            normalized_ml_analogy.contains(required),
            "CFT-33-003 ML analogy is missing `{required}`"
        );
    }

    let coverage = read_json(&contracts_root().join("coverage.json"));
    let row = array(&coverage, "items", "coverage")
        .iter()
        .find(|row| string(row, "item_id", "coverage row") == "CFT-33-003")
        .expect("missing CFT-33-003");
    let prerequisites = array(row, "pedagogical_prerequisites", "CFT-33-003")
        .iter()
        .map(|value| value.as_str().expect("prerequisite string"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        prerequisites,
        BTreeSet::from(["CFT-33-001", "CFT-33-002"]),
        "CFT-33-003 needs the two preceding LS cards and no Jin item"
    );
    assert!(prerequisites.iter().all(|item| {
        !["CFT-30-", "CFT-31-", "CFT-32-"]
            .iter()
            .any(|prefix| item.starts_with(prefix))
    }));
}

#[test]
fn chapter_33_cft_003_e03_states_the_certificate_and_keeps_its_proof_independent() {
    let markdown = chapter_33_markdown();
    let exercise_marker = "### CFT-33-E03 -- written-proof {#exercise-cft-33-e03}";
    let exercise_tail = markdown
        .split_once(exercise_marker)
        .expect("Chapter 33 E03 exercise")
        .1;
    let exercise = exercise_tail
        .split_once("\n### CFT-33-E04")
        .expect("Chapter 33 E03 boundary")
        .0;
    let normalized_exercise = exercise.split_whitespace().collect::<Vec<_>>().join(" ");
    for required in [
        "∑ i∈Finset.range N [(κ⁻¹)^(i+1)m_(i+1) - (κ⁻¹)^(i+2)m_(i+2)] = κ⁻¹m_1 - (κ⁻¹)^(N+1)m_(N+1)",
        "expand the first few terms",
        "induct on `N`",
    ] {
        assert!(
            normalized_exercise.contains(required),
            "CFT-33-E03 is missing `{required}`"
        );
    }

    let card = chapter_33_card(&markdown, "CFT-33-003");
    let lean_correspondence = chapter_33_card_field(card, "Lean correspondence");
    for link in [
        "[[formalization/lean/Crouzeix/LoristSchwenninger/OperatorRecurrence.lean|DilationData.equation_three_lower_bound]]",
        "[[formalization/lean/Crouzeix/LoristSchwenninger/Recurrence.lean|bounded_recurrence_terminal_tendsto_zero]]",
        "[[formalization/lean/Crouzeix/LoristSchwenninger/Recurrence.lean|inverse_power_weight_sum_tendsto]]",
        "[[formalization/lean/Crouzeix/LoristSchwenninger/RecurrenceScalar.lean|recurrenceScalar_abs_le]]",
    ] {
        assert!(
            lean_correspondence.contains(link),
            "CFT-33-003 needs clickable support link `{link}`"
        );
    }

    let lean = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean"),
    )
    .expect("Chapter 33 Lean module");
    let exercise_proof = lean
        .split_once("theorem exercise_03_solution")
        .expect("Chapter 33 E03 theorem")
        .1
        .split_once("\ntheorem exercise_04_solution")
        .expect("Chapter 33 E03/E04 theorem boundary")
        .0;
    assert!(exercise_proof.contains("induction N"));
    assert!(exercise_proof.contains("Finset.sum_range_succ"));
    assert!(
        !exercise_proof.contains("equation_three_lower_bound"),
        "CFT-33-E03 must prove the finite equality independently"
    );
}

#[test]
fn chapter_33_cft_004_005_reconstruct_equation_four_and_the_scalar_contradiction() {
    let markdown = chapter_33_markdown();
    let combined = chapter_33_card(&markdown, "CFT-33-004");
    let combined_proof = chapter_33_card_field(combined, "Proof");

    assert_math_steps_in_order(
        combined_proof,
        "CFT-33-004 displacement expansion",
        &[
            "A = Q*VTx",
            "b = ‖A-κVx‖²",
            "= ‖A‖² - 2κ Re ⟪A,Vx⟫ + κ²",
            "‖A‖ ≤ ‖VTx‖ = ‖Tx‖ ≤ ‖T‖‖x‖ = κ",
            "m = 2 Re ⟪A,Vx⟫ - κ²",
            "≤ κ² - 2κ Re ⟪A,Vx⟫ + κ²",
            "= 2κ² - κm - κ³",
        ],
    );
    assert_math_steps_in_order(
        combined_proof,
        "CFT-33-004 ordered cross-term identification",
        &[
            "Re ⟪x,E₁Tx⟫",
            "= Re ⟪E₁Tx,x⟫",
            "= 2 Re ⟪V*A,x⟫ - Re ⟪T*Tx,x⟫",
            "= 2 Re ⟪A,Vx⟫ - κ²",
        ],
    );
    for required in [
        "**Unit norm.**",
        "‖x‖=1",
        "**Norm attainment.**",
        "`T.le_opNorm`",
        "**Singular-vector equation.**",
        "T*Tx=κ²x",
        "E_1Tx = 2V*Q*VTx - T*Tx = 2V*A-T*Tx",
        "source Equation (4)",
        "`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98`",
    ] {
        assert!(
            combined_proof.contains(required),
            "CFT-33-004 is missing `{required}`"
        );
    }
    let normalized_combined_proof = combined_proof
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        normalized_combined_proof.contains(
            "Norm attainment is consumed only when deriving `T*Tx=κ²x`; it is not an additional input to Equation (4)"
        ),
        "CFT-33-004 must record where norm attainment is consumed"
    );
    let combined_proof_visible = visible_markdown_text(combined_proof);
    for required in [
        "V*A begins inside the second argument",
        "Real-part conjugate symmetry moves the entire E₁Tx to the first argument",
        "Only then does adjoint transfer rewrite Re ⟪V*A,x⟫ as Re ⟪A,Vx⟫",
    ] {
        assert!(
            combined_proof_visible.contains(required),
            "CFT-33-004 visible cross-term explanation is missing `{required}`"
        );
    }
    assert!(!combined_proof.contains("Adjoint transfer moves `V*` from the first argument to `Vx`"));
    assert!(
        !combined_proof.contains("‖A‖ ≤ ‖VTx‖ = ‖Tx‖ = κ"),
        "CFT-33-004 must not replace the operator-norm bound with norm attainment"
    );
    assert_math_steps_in_order(
        combined_proof,
        "CFT-33-004 substitution of Equation (3) into Equation (4)",
        &[
            "-b/[κ(κ-1)²] ≤ m",
            "-κm ≤ b/(κ-1)²",
            "≤ 2κ² - κ³ + b/(κ-1)²",
            "b - b/(κ-1)² ≤ 2κ² - κ³",
            "b(1 - 1/(κ-1)²) ≤ 2κ² - κ³",
        ],
    );
    let combined_lean = chapter_33_card_field(combined, "Lean correspondence");
    for required in [
        "eb6c1ea9a5220026fc4b0cac90af21419688775723e9353193441e77bee69cad",
        "`Classical.choice`, `Quot.sound`, and `propext`",
        "verification target is `CrouzeixTextbook`",
    ] {
        assert!(
            combined_lean.contains(required),
            "CFT-33-004 Lean audit is missing `{required}`"
        );
    }

    let endpoint = chapter_33_card(&markdown, "CFT-33-005");
    let endpoint_proof = chapter_33_card_field(endpoint, "Proof");
    assert_math_steps_in_order(
        endpoint_proof,
        "CFT-33-005 sign contradiction",
        &[
            "κ>2",
            "κ-1>1",
            "(κ-1)²>1",
            "0<1-1/(κ-1)²",
            "0≤b(1-1/(κ-1)²)",
            "2κ²-κ³ = κ²(2-κ)<0",
        ],
    );
    for required in ["`κ≤1`", "`1<κ<2`", "`κ=2`", "`b≥0`", "κ≤2"] {
        assert!(
            endpoint_proof.contains(required),
            "CFT-33-005 is missing boundary/sign content `{required}`"
        );
    }
    let endpoint_lean = chapter_33_card_field(endpoint, "Lean correspondence");
    for required in [
        "d0d9e72ee420d466774132edb0b84345a1882462e1e68c37702786c94baaf02c",
        "`Classical.choice`, `Quot.sound`, and `propext`",
        "verification target is `CrouzeixTextbook`",
    ] {
        assert!(
            endpoint_lean.contains(required),
            "CFT-33-005 Lean audit is missing `{required}`"
        );
    }

    for (item_id, card, required) in [
        (
            "CFT-33-004",
            combined,
            [
                "Exact transfer",
                "lower-bounds a coupling moment",
                "Non-transfer",
                "stochastic minibatch Jacobian",
                "Diagnostic",
                "identical cross term",
            ],
        ),
        (
            "CFT-33-005",
            endpoint,
            [
                "Exact transfer",
                "nonnegative empirical energy",
                "Non-transfer",
                "Monte Carlo confidence interval",
                "Diagnostic",
                "certified positive lower bound",
            ],
        ),
    ] {
        let ml = visible_markdown_text(chapter_33_card_field(card, "ML analogy"));
        assert_math_steps_in_order(&ml, &format!("{item_id} visible ML analogy"), &required);
    }

    let e04 = markdown
        .split_once("### CFT-33-E04 -- written-proof {#exercise-cft-33-e04}")
        .expect("Chapter 33 E04 exercise")
        .1
        .split_once("\n### CFT-33-E05")
        .expect("Chapter 33 E04 boundary")
        .0;
    for required in [
        "`1<κ`",
        "0<κ",
        "0<(κ-1)²",
        "κ(κ-1)²≠0",
        "exercise_04_solution",
    ] {
        assert!(e04.contains(required), "CFT-33-E04 is missing `{required}`");
    }
    let e05 = markdown
        .split_once("### CFT-33-E05 -- boundary {#exercise-cft-33-e05}")
        .expect("Chapter 33 E05 exercise")
        .1
        .split_once("\n### CFT-33-E06")
        .expect("Chapter 33 E05 boundary")
        .0;
    for required in [
        "0≤b",
        "b(1-1/(κ-1)²)≤2κ²-κ³",
        "κ≤2",
        "exercise_05_solution",
        "9f496c7d5a22e792008c61c6f53c6191cca2373e96b43bab5892bb6399b3f99c",
        "71c430c3b0eae949f33b2e2f7b324dd83046d9e27f39348a2b4fd336e18ae3da",
        "compile as genuine `theorem`",
    ] {
        assert!(e05.contains(required), "CFT-33-E05 is missing `{required}`");
    }
    assert!(e05
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .contains("statements differ from every public Chapter 33 checkpoint"));

    let lean = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean"),
    )
    .expect("Chapter 33 Lean module");
    let e04_proof = lean
        .split_once("theorem exercise_04_solution")
        .expect("Chapter 33 E04 theorem")
        .1
        .split_once("theorem exercise_05_solution")
        .expect("Chapter 33 E04 Lean boundary")
        .0;
    assert!(e04_proof.contains("constructor"));
    assert!(!e04_proof.contains("scalar_combined_inequality"));
    let e05_proof = lean
        .split_once("theorem exercise_05_solution")
        .expect("Chapter 33 E05 theorem")
        .1
        .split_once("\nend Exercises.Chapter33")
        .expect("Chapter 33 exercise namespace boundary")
        .0;
    for required in ["by_contra", "hfactor_pos", "hright_neg"] {
        assert!(
            e05_proof.contains(required),
            "CFT-33-E05 Lean proof is missing `{required}`"
        );
    }
    assert!(!e05_proof.contains("scalar_endpoint_two"));
    assert!(!e05_proof.contains("scalar_contradiction_le_two"));

    let coverage = read_json(&contracts_root().join("coverage.json"));
    for (item_id, provider) in [
        (
            "CFT-33-004",
            "CrouzeixConjecture.LoristSchwenninger.scalar_combined_inequality_of_recurrence_bounds",
        ),
        (
            "CFT-33-005",
            "CrouzeixConjecture.LoristSchwenninger.scalar_endpoint_le_two",
        ),
    ] {
        let row = array(&coverage, "items", "coverage")
            .iter()
            .find(|row| string(row, "item_id", "coverage row") == item_id)
            .unwrap_or_else(|| panic!("missing {item_id}"));
        assert_eq!(
            string(row, "prose_proof_status", item_id),
            "reconstructible"
        );
        assert_eq!(string(row, "lean_correspondence_status", item_id), "exact");
        assert_eq!(string(row, "formal_mode", item_id), "reexported-proof");
        assert_eq!(
            string(&row["lean_declaration"], "underlying_declaration", item_id),
            provider
        );
    }

    let exercises = read_json(&contracts_root().join("exercises.json"));
    for (exercise_id, declaration) in [
        (
            "CFT-33-E04",
            "CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_04_solution",
        ),
        (
            "CFT-33-E05",
            "CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_05_solution",
        ),
    ] {
        let row = array(&exercises, "exercises", "exercises")
            .iter()
            .find(|row| string(row, "exercise_id", "exercise row") == exercise_id)
            .unwrap_or_else(|| panic!("missing {exercise_id}"));
        assert!(
            !row["starter"].is_null(),
            "{exercise_id} needs a Lean starter"
        );
        assert_eq!(
            string(&row["lean_solution"], "declaration", exercise_id),
            declaration
        );
    }
}

#[test]
fn chapter_33_cft_006_assembles_the_operator_argument_and_publishes_distinct_e06() {
    let markdown = chapter_33_markdown();
    let card = chapter_33_card(&markdown, "CFT-33-006");
    let proof = chapter_33_card_field(card, "Proof");
    let hypotheses = chapter_33_card_field(card, "Hypothesis ledger");
    let history = visible_markdown_text(chapter_33_card_field(card, "Historical context"));
    let ml_markdown = chapter_33_card_field(card, "ML analogy");
    let ml = visible_markdown_text(ml_markdown);
    let normalized_proof = proof.split_whitespace().collect::<Vec<_>>().join(" ");

    assert_math_steps_in_order(
        proof,
        "CFT-33-006 operator/scalar assembly",
        &[
            "κ = ‖T‖",
            "by_cases hsmall : κ ≤ 1",
            "κ ≤ 1 < 2",
            "1 < κ",
            "‖x‖ = 1",
            "‖Tx‖ = κ",
            "T*T x = κ²x",
            "b = ‖Q*VTx-κVx‖²",
            "m = Re ⟪E₁Tx,x⟫",
            "-b/[κ(κ-1)²] ≤ m",
            "b ≤ 2κ²-κm-κ³",
            "0 ≤ b",
            "0 < κ",
            "0 < (κ-1)²",
            "κ ≤ 2",
        ],
    );
    for required in [
        "exists_unit_norm_attaining_and_adjoint_apply",
        "equation_three_lower_bound",
        "inner_re_symm",
        "displacementSq_le",
        "scalar_endpoint_le_two",
        "‖A‖ ≤ ‖Tx‖ ≤ ‖T‖‖x‖ = κ",
        "`T.le_opNorm`",
        "Norm attainment is consumed upstream to produce `T*T x=κ²x`; it is not an additional input to Equation (4)",
    ] {
        assert!(
            normalized_proof.contains(required),
            "CFT-33-006 proof is missing `{required}`"
        );
    }

    for required in [
        "**Finite dimensionality.**",
        "**Completeness of `E`.**",
        "**Nontriviality.**",
        "**Completeness of `K`.**",
        "**Isometry of `V`.**",
        "**Contractivity of `Q`.**",
        "**Exact perturbation identity.**",
        "**Uniform perturbation bounds.**",
        "**Commutation and power identities.**",
        "**Norm attainment.**",
        "**Singular-vector relation.**",
        "consumed by",
    ] {
        assert!(
            hypotheses.contains(required),
            "CFT-33-006 hypothesis ledger is missing `{required}`"
        );
    }

    assert_math_steps_in_order(
        &history,
        "CFT-33-006 labeled historical context",
        &["EVIDENCE", "SOURCE CLAIM", "No priority claim"],
    );
    for required in [
        "CFT-CL-003",
        "CFT-CL-006",
        "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99",
        "PerturbationLemma.lean",
    ] {
        assert!(
            history.contains(required),
            "CFT-33-006 history is missing `{required}`"
        );
    }

    assert_math_steps_in_order(
        &ml,
        "CFT-33-006 bounded ML analogy",
        &[
            "Mathematical object / ML counterpart.",
            "Exact transfer.",
            "Non-transfer.",
            "Diagnostic.",
        ],
    );
    for exact_label in [
        "**Mathematical object / ML counterpart.**",
        "**Exact transfer.**",
        "**Non-transfer.**",
        "**Diagnostic.**",
    ] {
        assert!(
            ml_markdown.contains(exact_label),
            "CFT-33-006 ML analogy is missing exact field label `{exact_label}`"
        );
    }
    for obsolete_label in [
        "**Mathematical object.**",
        "**ML counterpart.**",
        "**Non-transfer / diagnostic.**",
    ] {
        assert!(
            !ml_markdown.contains(obsolete_label),
            "CFT-33-006 must use the exact four-field ML grouping, not `{obsolete_label}`"
        );
    }
    for required in [
        "recurrent-linearization stability",
        "same scalar certificate",
        "stochastic or time-varying Jacobians",
        "pathwise or probabilistic bounds",
        "J = [[1,4],[0,1]]",
        "J*J = [[1,4],[4,17]]",
        "κ = 2+√5",
        "x = (1,κ)/√(1+κ²)",
        "J*Jx=κ²x",
        "singular residual is exactly zero",
        "J alone does not determine b or m",
        "dilation data V,Q,E₁",
        "κ>2",
        "infeasible for every b≥0",
        "Reject the scalar certificate",
    ] {
        assert!(
            ml.contains(required),
            "CFT-33-006 ML analogy is missing `{required}`"
        );
    }

    let lean_correspondence = chapter_33_card_field(card, "Lean correspondence");
    for required in [
        "[[formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]]",
        "[[formalization/lean/Crouzeix/LoristSchwenninger/PerturbationLemma.lean|PerturbationLemma.lean]]",
        "CrouzeixTextbook.Part06.perturbation_lemma",
        "CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two",
        "normalized type fingerprint",
        "Classical.choice",
        "verification target is `CrouzeixTextbook`",
    ] {
        assert!(
            lean_correspondence.contains(required),
            "CFT-33-006 Lean correspondence is missing `{required}`"
        );
    }

    for index in 1..=6 {
        let item_id = format!("CFT-33-{index:03}");
        let correspondence =
            chapter_33_card_field(chapter_33_card(&markdown, &item_id), "Lean correspondence");
        let correspondence = correspondence
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        for required in [
            "[[formalization/lean/",
            "type fingerprint",
            "axioms",
            "verification target",
        ] {
            assert!(
                correspondence.contains(required),
                "{item_id} does not expose truthful Lean audit field `{required}`"
            );
        }
    }

    let exercise = markdown
        .split_once("### CFT-33-E06 -- lean-proof {#exercise-cft-33-e06}")
        .expect("Chapter 33 E06 exercise")
        .1
        .split_once("\n### Solution sketches")
        .expect("Chapter 33 E06 boundary")
        .0;
    for required in [
        "κ = ‖T‖",
        "κ≤1",
        "1<κ",
        "norm-attaining unit vector",
        "T*T x=κ²x",
        "b=‖Q*VTx-κVx‖²",
        "m=Re⟪E₁Tx,x⟫",
        "Equation (3)",
        "Equation (4)",
        "prove `‖T‖≤2`",
        "exercise_06_solution",
        "6d8a70779a3993e87edea9466272c8742e362b2ecf9b72431465fd03ea7361c9",
        "statement differs from every public Chapter 33 checkpoint",
    ] {
        assert!(
            exercise.contains(required),
            "CFT-33-E06 is missing `{required}`"
        );
    }
    assert!(
        !exercise.contains("given the branch certificate"),
        "CFT-33-E06 must derive, not assume, the operator-to-scalar certificate"
    );

    let lean = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean"),
    )
    .expect("Chapter 33 Lean module");
    let e06_proof = lean
        .split_once("theorem exercise_06_solution")
        .expect("Chapter 33 E06 theorem")
        .1
        .split_once("\nend Exercises.Chapter33")
        .expect("Chapter 33 exercise namespace boundary")
        .0;
    for required in [
        "by_cases hsmall : κ ≤ 1",
        "exists_unit_norm_attaining_and_adjoint_apply data.T",
        "let b : ℝ := data.displacementSq κ x",
        "let m : ℝ := data.firstPerturbationMoment x",
        "data.equation_three_lower_bound",
        "inner_re_symm",
        "data.displacementSq_le",
        "data.displacementSq_nonneg",
        "scalar_endpoint_le_two",
    ] {
        assert!(
            e06_proof.contains(required),
            "CFT-33-E06 Lean proof is missing `{required}`"
        );
    }
    for assumed_away in ["(κ b m : ℝ)", "hbranch"] {
        assert!(
            !e06_proof.contains(assumed_away),
            "CFT-33-E06 must derive the operator certificate instead of assuming `{assumed_away}`"
        );
    }
    for forbidden in [
        "perturbation_lemma",
        "norm_target_le_two",
        "scalar_endpoint_two",
    ] {
        assert!(
            !e06_proof.contains(forbidden),
            "CFT-33-E06 must not reuse `{forbidden}`"
        );
    }

    let coverage = read_json(&contracts_root().join("coverage.json"));
    let row = array(&coverage, "items", "coverage")
        .iter()
        .find(|row| string(row, "item_id", "coverage row") == "CFT-33-006")
        .expect("missing CFT-33-006");
    assert_eq!(
        string(row, "prose_proof_status", "CFT-33-006"),
        "reconstructible"
    );
    assert_eq!(
        string(row, "lean_correspondence_status", "CFT-33-006"),
        "exact"
    );
    assert_eq!(string(row, "formal_mode", "CFT-33-006"), "reexported-proof");
    assert_eq!(
        string(
            &row["lean_declaration"],
            "underlying_declaration",
            "CFT-33-006"
        ),
        "CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two"
    );

    let exercises = read_json(&contracts_root().join("exercises.json"));
    let exercise_row = array(&exercises, "exercises", "exercises")
        .iter()
        .find(|row| string(row, "exercise_id", "exercise row") == "CFT-33-E06")
        .expect("missing CFT-33-E06");
    assert!(!exercise_row["starter"].is_null());
    assert_eq!(
        string(&exercise_row["lean_solution"], "declaration", "CFT-33-E06"),
        "CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_06_solution"
    );
    assert_eq!(
        string(&exercise_row["lean_solution"], "type_sha256", "CFT-33-E06"),
        "6d8a70779a3993e87edea9466272c8742e362b2ecf9b72431465fd03ea7361c9"
    );
}

#[test]
fn chapter_33_cft_001_through_003_publish_exact_distinct_lean_support() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    for (item_id, provider) in [
        (
            "CFT-33-001",
            "CrouzeixConjecture.LoristSchwenninger.DilationData.recurrence_difference_identity",
        ),
        (
            "CFT-33-002",
            "CrouzeixConjecture.LoristSchwenninger.DilationData.recurrence_difference_lower_bound",
        ),
        (
            "CFT-33-003",
            "CrouzeixConjecture.LoristSchwenninger.DilationData.equation_three_lower_bound",
        ),
    ] {
        let row = array(&coverage, "items", "coverage")
            .iter()
            .find(|row| string(row, "item_id", "coverage row") == item_id)
            .unwrap_or_else(|| panic!("missing {item_id}"));
        assert_eq!(
            string(row, "prose_proof_status", item_id),
            "reconstructible"
        );
        assert_eq!(string(row, "lean_correspondence_status", item_id), "exact");
        assert_eq!(string(row, "formal_mode", item_id), "reexported-proof");
        assert_eq!(
            string(&row["lean_declaration"], "underlying_declaration", item_id),
            provider
        );
    }

    let exercises = read_json(&contracts_root().join("exercises.json"));
    for (exercise_id, declaration) in [
        (
            "CFT-33-E01",
            "CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_01_solution",
        ),
        (
            "CFT-33-E02",
            "CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_02_solution",
        ),
        (
            "CFT-33-E03",
            "CrouzeixTextbook.Part06.Exercises.Chapter33.exercise_03_solution",
        ),
    ] {
        let row = array(&exercises, "exercises", "exercises")
            .iter()
            .find(|row| string(row, "exercise_id", "exercise row") == exercise_id)
            .unwrap_or_else(|| panic!("missing {exercise_id}"));
        assert!(
            !row["starter"].is_null(),
            "{exercise_id} needs a useful Lean starter"
        );
        assert_eq!(
            string(&row["lean_solution"], "declaration", exercise_id),
            declaration
        );
    }

    let lean = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean"),
    )
    .expect("Chapter 33 Lean module");
    assert!(lean.contains("theorem recurrence_difference_identity"));
    assert!(lean.contains("theorem recurrence_difference_lower_bound"));
    assert!(lean.contains("theorem exercise_01_solution"));
    assert!(lean.contains("theorem exercise_02_solution"));
    assert!(lean.contains("theorem exercise_03_solution"));
}

#[test]
fn chapter_01_exercises_02_through_05_publish_prompt_matched_lean_tasks() {
    let lean = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean"),
    )
    .expect("Chapter 1 Lean module");
    let e02 = lean
        .split_once("theorem exercise_02_solution")
        .expect("Chapter 1 E02 solution")
        .1
        .split_once("theorem exercise_03_solution")
        .expect("Chapter 1 E03 boundary")
        .0;
    for mathematical_role in ["!![2, -1; 3, 4]", "![1, 0]", "![2, 3]", "![-1, 4]"] {
        assert!(
            e02.contains(mathematical_role),
            "CFT-01-E02 is missing `{mathematical_role}`"
        );
    }
    let e03 = lean
        .split_once("theorem exercise_03_solution")
        .expect("Chapter 1 E03 solution")
        .1
        .split_once("theorem exercise_04_solution")
        .expect("Chapter 1 E04 boundary")
        .0;
    for mathematical_role in ["∑ j, x j • fun i => A i j", "Matrix.mulVec", "dotProduct"] {
        assert!(
            e03.contains(mathematical_role),
            "CFT-01-E03 is missing `{mathematical_role}`"
        );
    }

    let e04 = lean
        .split_once("theorem exercise_04_solution")
        .expect("Chapter 1 E04 solution")
        .1
        .split_once("theorem exercise_05_solution")
        .expect("Chapter 1 E05 boundary")
        .0;
    for mathematical_role in ["charpoly", "Matrix.det", "Matrix.det_units_conj"] {
        assert!(
            e04.contains(mathematical_role),
            "CFT-01-E04 is missing `{mathematical_role}`"
        );
    }

    let e05 = lean
        .split_once("theorem exercise_05_solution")
        .expect("Chapter 1 E05 solution")
        .1
        .split_once("theorem exercise_06_solution")
        .expect("Chapter 1 E06 boundary")
        .0;
    for mathematical_role in [
        "charpoly",
        ".trace",
        "nonnormalExample_similarity",
        "nonunitary_similarity_changes_output_length_sq",
        "U.transpose * U = 1",
        "dotProduct (U *ᵥ x) (U *ᵥ x) = dotProduct x x",
    ] {
        assert!(
            e05.contains(mathematical_role),
            "CFT-01-E05 is missing `{mathematical_role}`"
        );
    }

    let exercises = read_json(&contracts_root().join("exercises.json"));
    for exercise_id in ["CFT-01-E02", "CFT-01-E03", "CFT-01-E04", "CFT-01-E05"] {
        let row = array(&exercises, "exercises", "exercises")
            .iter()
            .find(|row| string(row, "exercise_id", "exercise row") == exercise_id)
            .unwrap_or_else(|| panic!("missing {exercise_id}"));
        assert!(
            !row["starter"].is_null(),
            "{exercise_id} needs a prompt-matched Lean starter"
        );
    }
}

#[test]
fn chapter_33_prose_separates_history_and_keeps_an_independent_ls_chain() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let rows = array(&coverage, "items", "coverage")
        .iter()
        .filter(|row| integer(row, "chapter", "coverage row") == 33)
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 6);
    assert_eq!(string(rows[0], "kind", "CFT-33-001"), "theorem");
    for (index, row) in rows.iter().enumerate() {
        let (expected_prose, expected_lean) = ("reconstructible", "exact");
        assert_eq!(
            string(row, "prose_proof_status", "coverage row"),
            expected_prose
        );
        assert_eq!(
            string(row, "lean_correspondence_status", "coverage row"),
            expected_lean
        );
        let prerequisites = array(row, "pedagogical_prerequisites", "coverage row")
            .iter()
            .map(|value| value.as_str().expect("prerequisite string"))
            .collect::<BTreeSet<_>>();
        assert!(
            prerequisites.iter().all(|item| {
                !["CFT-30-", "CFT-31-", "CFT-32-"]
                    .iter()
                    .any(|prefix| item.starts_with(prefix))
            }),
            "{} crosses into the Jin branch: {prerequisites:?}",
            string(row, "item_id", "coverage row")
        );
        if index > 0 {
            let previous = format!("CFT-33-{index:03}");
            assert!(
                prerequisites.contains(previous.as_str()),
                "{} must depend on its preceding LS proof step {previous}",
                string(row, "item_id", "coverage row")
            );
        }
    }

    let exercises = read_json(&contracts_root().join("exercises.json"));
    for (index, exercise) in array(&exercises, "exercises", "exercises")
        .iter()
        .filter(|row| integer(row, "chapter", "exercise row") == 33)
        .enumerate()
    {
        assert!(
            !exercise["starter"].is_null(),
            "Chapter 33 E{:02} needs a starter",
            index + 1
        );
        assert!(
            !exercise["lean_solution"].is_null(),
            "Chapter 33 E{:02} needs a checked solution",
            index + 1
        );
    }
}

#[test]
fn chapter_33_scalar_cards_publish_visible_source_and_review_boundaries() {
    let markdown = chapter_33_markdown();
    for (item_id, locator) in [
        (
            "CFT-33-002",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L74-L90",
        ),
        (
            "CFT-33-004",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98",
        ),
        (
            "CFT-33-005",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98",
        ),
    ] {
        let history = visible_markdown_text(chapter_33_card_field(
            chapter_33_card(&markdown, item_id),
            "Historical context",
        ));
        for required in [
            "SOURCE CLAIM",
            "CFT-CL-006",
            "EVIDENCE",
            "CFT-CL-003",
            locator,
            "does not establish peer review, acceptance, or journal publication",
            "does not establish historical priority",
        ] {
            assert!(
                history.contains(required),
                "{item_id} visible history is missing `{required}`"
            );
        }
    }
}

#[test]
fn maintained_receipt_prose_matches_the_current_compiler_receipt() {
    let root = workspace_root();
    let manifest = read_json(&publication_manifest(&root));
    let status_output = array(&manifest, "outputs", "publication manifest")
        .iter()
        .find(|output| string(output, "name", "publication output") == "status_ledger.md")
        .expect("current publication has a status ledger");
    let generated_status =
        fs::read_to_string(root.join(string(status_output, "path", "status ledger output")))
            .expect("current generated status ledger");
    let expected_sha256 = backticked_field(&generated_status, "Compiler receipt SHA-256:");
    let expected_bytes = backticked_field(&generated_status, "Compiler receipt bytes:");
    let expected_declarations =
        backticked_field(&generated_status, "Compiler receipt declarations:");

    let maintained_status = fs::read_to_string(packet_root().join("status_and_scope.md"))
        .expect("maintained status and scope");
    for (label, expected) in [
        ("Compiler receipt SHA-256:", expected_sha256),
        ("Compiler receipt bytes:", expected_bytes),
        ("Compiler receipt declarations:", expected_declarations),
    ] {
        assert_eq!(
            backticked_field(&maintained_status, label),
            expected,
            "status and scope has stale `{label}` metadata"
        );
    }

    let chapter = chapter_33_markdown();
    for (label, expected) in [
        ("Compiler receipt SHA-256:", expected_sha256),
        ("serialized bytes:", expected_bytes),
        ("declaration count:", expected_declarations),
    ] {
        assert_eq!(
            backticked_field(&chapter, label),
            expected,
            "Chapter 33 has stale `{label}` metadata"
        );
    }
}

#[test]
fn chapter_33_release_freeze_exposes_complete_reader_and_lean_audits() {
    let markdown = chapter_33_markdown();
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));

    let status = fs::read_to_string(packet_root().join("status_and_scope.md"))
        .expect("tracked textbook status");
    let receipt_sha256 = backticked_field(&status, "Compiler receipt SHA-256:");
    let receipt_bytes = backticked_field(&status, "Compiler receipt bytes:");
    let receipt_declarations = backticked_field(&status, "Compiler receipt declarations:");
    for required in [receipt_sha256, receipt_bytes, receipt_declarations] {
        assert!(
            markdown.contains(required),
            "Chapter 33 must identify the durable compiler receipt field `{required}`"
        );
    }
    assert!(markdown.contains("Compiler receipt SHA-256"));
    assert!(markdown.contains("[[knowledge/crouzeix_textbook/status_and_scope|status and scope]]"));
    assert!(visible_markdown_text(&markdown).contains(
        "The compiler receipt and the six-ledger publication generation are distinct identities"
    ));
    assert!(
        !markdown.contains("compiler-backed publication pointer"),
        "Chapter 33 must not call the six-ledger pointer a compiler receipt"
    );
    assert!(
        !markdown.contains("[[atlas/src/content/generated"),
        "Atlas is not a vault-root wikilink namespace"
    );

    let visible_errors = chapter_33_release_visible_text_errors(&markdown);
    assert!(
        visible_errors.is_empty(),
        "Chapter 33 release text is incomplete: {visible_errors:#?}"
    );

    for index in 1..=6 {
        let item_id = format!("CFT-33-{index:03}");
        let card = chapter_33_card(&markdown, &item_id);
        let lean = chapter_33_card_field(card, "Lean correspondence");
        let row = array(&coverage, "items", "coverage")
            .iter()
            .find(|row| string(row, "item_id", "coverage row") == item_id)
            .unwrap_or_else(|| panic!("missing {item_id}"));
        let declaration = &row["lean_declaration"];
        let location = format!(
            "{}:{}:{}",
            string(declaration, "source_path", &item_id),
            integer(declaration, "line", &item_id),
            integer(declaration, "column", &item_id)
        );
        for required in [
            string(declaration, "name", &item_id),
            string(declaration, "underlying_declaration", &item_id),
            string(declaration, "type_sha256", &item_id),
            string(declaration, "verification_target", &item_id),
        ] {
            assert!(
                lean.contains(required),
                "{item_id} Lean correspondence is missing `{required}`"
            );
        }
        assert!(
            lean.contains(&location),
            "{item_id} Lean correspondence is missing compiler location `{location}`"
        );
        assert!(
            lean.contains("Direct kernel dependency"),
            "{item_id} must name its direct compiler-reported dependency"
        );
    }

    let exercise_audit = markdown
        .split_once("### Exercise Lean audit")
        .expect("Chapter 33 exercise Lean audit")
        .1
        .split_once("\n### Solution sketches")
        .expect("Chapter 33 exercise audit boundary")
        .0;
    for exercise in array(&exercises, "exercises", "exercises")
        .iter()
        .filter(|row| integer(row, "chapter", "exercise row") == 33)
    {
        let exercise_id = string(exercise, "exercise_id", "exercise row");
        let solution = &exercise["lean_solution"];
        let location = format!(
            "{}:{}:{}",
            string(solution, "source_path", exercise_id),
            integer(solution, "line", exercise_id),
            integer(solution, "column", exercise_id)
        );
        for required in [
            exercise_id,
            string(solution, "declaration", exercise_id),
            string(solution, "type_sha256", exercise_id),
            string(solution, "verification_target", exercise_id),
            location.as_str(),
        ] {
            assert!(
                exercise_audit.contains(required),
                "{exercise_id} exercise audit is missing `{required}`"
            );
        }
    }
    for required in [
        "Direct kernel dependencies",
        "Classical.choice",
        "Quot.sound",
        "propext",
    ] {
        assert!(
            exercise_audit.contains(required),
            "Chapter 33 exercise audit is missing `{required}`"
        );
    }
}

#[test]
fn chapter_33_release_text_cannot_be_satisfied_by_html_comments() {
    let markdown = chapter_33_markdown();
    for (from, to, expected_error) in [
        (
            "**Motivation.**",
            "<!-- **Motivation.** -->",
            "CFT-33-001 Purpose",
        ),
        (
            "**Worked instance.**",
            "<!-- **Worked instance.** -->",
            "CFT-33-001 Proof",
        ),
        (
            "**Exact transfer.**",
            "<!-- **Exact transfer.** -->",
            "CFT-33-001 ML analogy",
        ),
        (
            "The successor law `pow_succ` gives",
            "The successor law <!-- `pow_succ` --> gives",
            "CFT-33-E01 written solution",
        ),
    ] {
        let mutated = markdown.replacen(from, to, 1);
        assert_ne!(mutated, markdown, "negative mutation must change `{from}`");
        let errors = chapter_33_release_visible_text_errors(&mutated);
        assert!(
            errors.iter().any(|error| error.contains(expected_error)),
            "HTML-comment mutation `{from}` escaped visible-text validation: {errors:#?}"
        );
    }
}

#[test]
fn canonical_reader_ledgers_are_truthful_v2_projections() {
    for (name, published_name) in [
        ("lean_coverage_ledger.md", "theorem_coverage_ledger.md"),
        ("exercise_index.md", "exercise_ledger.md"),
        (
            "theorem_dependency_map.md",
            "pedagogical_dependency_ledger.md",
        ),
    ] {
        let guide = fs::read_to_string(packet_root().join(name)).expect("reader guide");
        assert!(guide.contains("mise run crouzeix-textbook-publication"));
        assert!(guide.contains(published_name));
        assert!(guide.contains("current.json"));
        assert!(
            !guide.lines().any(|line| line.trim_start().starts_with('|')),
            "{name} must not duplicate a derived Markdown table"
        );
        for retired_roster in [
            "Theorem rows:",
            "Formal modes:",
            "Lean correspondence:",
            "Exercises:",
            "Checked Lean solutions:",
            "Correspondence-incomplete exercises:",
            "Reviewed roots:",
            "Graph SHA-256:",
        ] {
            assert!(
                !guide.contains(retired_roster),
                "{name} duplicates derived roster {retired_roster}"
            );
        }
    }

    let (fixture, receipt_dir, receipt) = lean_receipt_fixture();
    let receipt_path = write_lean_receipt(&receipt_dir, &receipt);
    let before =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect("initial Rust ledger publication");
    let before_files = before
        .outputs
        .iter()
        .map(|path| {
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                fs::read(fixture.path().join(path)).expect("initial published ledger"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    mutate_fixture_json(fixture.path(), "coverage.json", |coverage| {
        fixture_theorem_mut(coverage, "CFT-35-001")["pedagogical_prerequisites"] =
            json!(["CFT-29-001"]);
        fixture_theorem_mut(coverage, "CFT-35-001")["publication_status"] = json!("active");
    });
    mutate_fixture_json(fixture.path(), "exercises.json", |exercises| {
        exercises["exercises"][0]["difficulty"] = json!("publisher-owned-change");
    });
    let after =
        publish_crouzeix_textbook(fixture.path(), &receipt_path, TextbookPublishMode::Write)
            .expect("contract-mutated Rust ledger publication");
    let after_files = after
        .outputs
        .iter()
        .map(|path| {
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                fs::read(fixture.path().join(path)).expect("mutated published ledger"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for name in [
        "theorem_coverage_ledger.md",
        "exercise_ledger.md",
        "pedagogical_dependency_ledger.md",
        "status_ledger.md",
    ] {
        assert_ne!(
            before_files[name], after_files[name],
            "{name} ignored contracts"
        );
    }

    let claims = fs::read_to_string(packet_root().join("claim_evidence_ledger.md"))
        .expect("claim evidence ledger");
    assert!(claims.contains("mise run crouzeix-textbook-publication"));

    let status =
        fs::read_to_string(packet_root().join("status_and_scope.md")).expect("status and scope");
    assert!(status.contains("maintained reader projection"));
    assert!(status.contains("Rust-owned\nimmutable publication ledgers"));
}

#[test]
fn jin_route_contract_freezes_all_anchors_cards_and_exercise_targets_by_phase() {
    let contract = jin_route_contract();
    let phase = jin_contract_phase(&contract);
    assert!(contract.contains("- Target phase: `wave-2-complete`."));
    for field in JIN_CARD_FIELDS {
        assert!(
            contract.contains(&format!("`{field}`")),
            "Jin contract omits theorem-card field {field}"
        );
    }

    let coverage = read_json(&contracts_root().join("coverage.json"));
    let coverage_rows = array(&coverage, "items", "coverage")
        .iter()
        .filter(|row| matches!(integer(row, "chapter", "coverage row"), 30..=32))
        .collect::<Vec<_>>();
    assert_eq!(coverage_rows.len(), 18, "Jin route must retain 18 CFT rows");
    let expected_theorems = JIN_CFT_SPECS
        .iter()
        .filter(|spec| {
            let kind = if phase.completes(spec.chapter) {
                spec.kind
            } else {
                jin_baseline_kind(spec)
            };
            kind == "theorem"
        })
        .count();
    assert_eq!(
        coverage_rows
            .iter()
            .filter(|row| string(row, "kind", "coverage row") == "theorem")
            .count(),
        expected_theorems,
        "the active Jin phase has the wrong theorem roster"
    );
    assert_eq!(
        coverage_rows
            .iter()
            .filter(|row| string(row, "kind", "coverage row") == "definition")
            .count(),
        18 - expected_theorems,
        "the active Jin phase has the wrong definition roster"
    );

    let exercises = read_json(&contracts_root().join("exercises.json"));
    let exercise_rows = array(&exercises, "exercises", "exercises")
        .iter()
        .filter(|row| matches!(integer(row, "chapter", "exercise row"), 30..=32))
        .collect::<Vec<_>>();
    assert_eq!(
        exercise_rows.len(),
        18,
        "Jin route must retain 18 exercises"
    );
    assert!(
        jin_phase_metadata_errors(&coverage, &exercises, phase).is_empty(),
        "active Jin phase metadata differs from its exact per-row contract: {:#?}",
        jin_phase_metadata_errors(&coverage, &exercises, phase)
    );

    let exporter = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/ExportReceipt.lean"),
    )
    .expect("textbook receipt exporter");
    assert!(exporter.contains("checkedExerciseTheoremNames"));
    assert!(exporter.contains("declarationKind info == \"theorem\""));

    for chapter in 30..=32 {
        if !phase.completes(chapter) {
            assert!(
                !workspace_root()
                    .join(format!(
                        "formalization/lean/CrouzeixTextbook/Part06/Exercises/Chapter{chapter}.lean"
                    ))
                    .exists(),
                "pending Chapter {chapter} must not hide an unregistered exercise namespace"
            );
        }
    }

    let mut solution_names = BTreeSet::new();
    for chapter in 30..=32 {
        let markdown = jin_chapter_markdown(chapter);
        assert_eq!(
            markdown
                .lines()
                .filter(|line| {
                    line.starts_with(&format!("### CFT-{chapter:02}-"))
                        && !line.starts_with(&format!("### CFT-{chapter:02}-E"))
                })
                .count(),
            6,
            "Chapter {chapter} must expose exactly six theorem-card headings"
        );
        for index in 1..=6 {
            let item_id = format!("CFT-{chapter:02}-{index:03}");
            let anchor = format!("{{#cft-{chapter:02}-{index:03}}}");
            assert_eq!(
                markdown.matches(&anchor).count(),
                1,
                "{item_id} must have exactly one prose anchor"
            );
            assert_eq!(
                coverage_rows
                    .iter()
                    .filter(|row| string(row, "item_id", "coverage row") == item_id)
                    .count(),
                1,
                "{item_id} must have exactly one coverage row"
            );
        }

        let card_errors = jin_complete_card_errors(&markdown, chapter);
        if phase.completes(chapter) {
            assert!(
                card_errors.is_empty(),
                "completed Chapter {chapter} violates the theorem-card contract: {card_errors:#?}"
            );
            let required_locators: &[&str] = match chapter {
                30 => &["the_numerical_range_is_a_2_spectral_set_v4.tex#L319-L386"],
                31 => &["the_numerical_range_is_a_2_spectral_set_v4.tex#L387-L463"],
                32 => &[
                    "the_numerical_range_is_a_2_spectral_set_v4.tex#L464-L530",
                    "the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910",
                ],
                _ => unreachable!(),
            };
            for locator in required_locators {
                assert!(
                    markdown.contains(locator),
                    "completed Chapter {chapter} omits pinned source locator {locator}"
                );
            }
        } else {
            assert!(
                jin_pending_card_errors(&markdown, chapter).is_empty(),
                "pending Chapter {chapter} no longer matches the frozen baseline: {card_errors:#?}"
            );
        }

        for index in 1..=6 {
            let item_id = format!("CFT-{chapter:02}-{index:03}");
            let row = coverage_rows
                .iter()
                .find(|row| string(row, "item_id", "coverage row") == item_id)
                .expect("Jin coverage row");
            let exercise_id = format!("CFT-{chapter:02}-E{index:02}");
            let exercise = exercise_rows
                .iter()
                .find(|row| string(row, "exercise_id", "exercise row") == exercise_id)
                .expect("Jin exercise row");
            if phase.completes(chapter) {
                assert_eq!(
                    string(row, "prose_proof_status", &item_id),
                    "reconstructible"
                );
                assert_eq!(string(row, "lean_correspondence_status", &item_id), "exact");
                let expected = format!(
                    "CrouzeixTextbook.Part06.Exercises.Chapter{chapter}.exercise_{index:02}_solution"
                );
                let actual = string(&exercise["lean_solution"], "declaration", &exercise_id);
                assert_eq!(actual, expected);
                assert_eq!(
                    string(&exercise["lean_solution"], "source_path", &exercise_id),
                    format!("formalization/lean/CrouzeixTextbook/Part06/Chapter{chapter}.lean")
                );
                assert_eq!(
                    string(
                        &exercise["lean_solution"],
                        "verification_target",
                        &exercise_id
                    ),
                    "CrouzeixTextbook"
                );
                assert!(
                    solution_names.insert(actual.to_owned()),
                    "duplicate solution {actual}"
                );
                assert_ne!(
                    actual,
                    string(&row["lean_declaration"], "name", &item_id),
                    "{exercise_id} aliases its public checkpoint by name"
                );
                assert_ne!(
                    string(&exercise["lean_solution"], "type_sha256", &exercise_id),
                    string(&row["lean_declaration"], "type_sha256", &item_id),
                    "{exercise_id} reuses the checkpoint statement instead of a prompt-matched task"
                );
                assert!(
                    jin_solution_is_compiler_guarded(&exporter, actual),
                    "{exercise_id} is outside the compiler-level direct-alias guard"
                );
            } else {
                assert_eq!(string(row, "prose_proof_status", &item_id), "summary");
                assert!(
                    exercise["lean_solution"].is_null(),
                    "pending {exercise_id} must not claim a fake solution"
                );
            }
        }
    }

    let deliberately_thin = generated_theorem_card("cft-30-001").replacen(
        "### Generated theorem",
        "### CFT-30-001 — deliberately thin",
        1,
    );
    let errors = jin_complete_card_errors(&deliberately_thin, 30);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("`Proof` omits or misorders scoped obligations")),
        "the target-card check accepts a keyword-only template: {errors:#?}"
    );
}

#[test]
fn jin_route_phase_model_rejects_per_row_correspondence_swaps() {
    let (mut coverage, exercises) = jin_baseline_fixture();
    assert!(
        jin_phase_metadata_errors(&coverage, &exercises, JinPhase::BaselineFrozen).is_empty(),
        "immutable baseline fixture must pass before mutation"
    );
    let rows = coverage["items"].as_array_mut().expect("coverage items");
    let checkpoint = rows
        .iter()
        .position(|row| row["item_id"] == "CFT-30-001")
        .expect("checkpoint row");
    let unmapped = rows
        .iter()
        .position(|row| row["item_id"] == "CFT-30-003")
        .expect("unmapped row");
    rows[checkpoint]["lean_correspondence_status"] = json!("unmapped");
    rows[unmapped]["lean_correspondence_status"] = json!("checkpoint");
    assert!(
        !jin_phase_metadata_errors(&coverage, &exercises, JinPhase::BaselineFrozen).is_empty(),
        "immutable per-row baseline accepted a correspondence swap"
    );
}

#[test]
fn jin_route_phase_model_accepts_and_mutation_tests_all_four_phases() {
    let (baseline_coverage, baseline_exercises) = jin_baseline_fixture();
    for phase in JinPhase::ALL {
        let mut coverage = baseline_coverage.clone();
        let mut exercises = baseline_exercises.clone();
        jin_promote_phase_fixture(&mut coverage, &mut exercises, phase);
        assert!(
            jin_phase_metadata_errors(&coverage, &exercises, phase).is_empty(),
            "valid `{}` fixture was rejected",
            phase.label()
        );
        for spec in JIN_CFT_SPECS {
            let observed = [phase.completes(spec.chapter); 10];
            assert!(
                jin_card_field_presence_errors(spec.item_id, phase, &observed).is_empty(),
                "valid {} card-field fixture was rejected in `{}`",
                spec.item_id,
                phase.label()
            );
        }

        let mutation_spec = JIN_CFT_SPECS
            .iter()
            .find(|spec| phase.completes(spec.chapter))
            .unwrap_or(&JIN_CFT_SPECS[0]);
        let row = coverage["items"]
            .as_array_mut()
            .expect("coverage fixture")
            .iter_mut()
            .find(|row| row["item_id"] == mutation_spec.item_id)
            .expect("mutation row");
        row["lean_correspondence_status"] = json!("wrong-phase-state");
        assert!(
            !jin_phase_metadata_errors(&coverage, &exercises, phase).is_empty(),
            "`{}` fixture accepted a per-row correspondence mutation",
            phase.label()
        );

        let mut fields = [phase.completes(mutation_spec.chapter); 10];
        fields[7] = !fields[7];
        assert!(
            !jin_card_field_presence_errors(mutation_spec.item_id, phase, &fields).is_empty(),
            "`{}` fixture accepted a Lean correspondence field-presence mutation",
            phase.label()
        );
    }
}

#[test]
fn jin_route_chapter30_completion_does_not_mutate_the_frozen_baseline_fixture() {
    let (baseline_coverage, baseline_exercises) = jin_baseline_fixture();
    assert!(
        jin_phase_metadata_errors(
            &baseline_coverage,
            &baseline_exercises,
            JinPhase::BaselineFrozen,
        )
        .is_empty(),
        "immutable baseline fixture must pass before Chapter30 promotion"
    );
    let mut completed_coverage = baseline_coverage.clone();
    let mut completed_exercises = baseline_exercises.clone();
    jin_promote_phase_fixture(
        &mut completed_coverage,
        &mut completed_exercises,
        JinPhase::Chapter30Complete,
    );
    assert!(
        jin_phase_metadata_errors(
            &completed_coverage,
            &completed_exercises,
            JinPhase::Chapter30Complete,
        )
        .is_empty(),
        "synthetic real Chapter30 completion must satisfy its phase"
    );
    assert!(
        jin_phase_metadata_errors(
            &baseline_coverage,
            &baseline_exercises,
            JinPhase::BaselineFrozen,
        )
        .is_empty(),
        "Chapter30 promotion mutated the immutable baseline fixture"
    );
}

#[test]
fn jin_route_chapter30_completion_preserves_the_baseline_provider_identity() {
    let spec = jin_cft_spec("CFT-30-001");
    assert_eq!(
        jin_baseline_provider_hash(spec),
        "484e9737cde90a145a1db689cb649f22f3ee2f1cf3e299bd52d3a2374b9686a5"
    );
    assert_eq!(
        jin_target_provider_hash(spec),
        "d72a6d691c4c807c0b20b620c2e030345c8d923a1c9b31d1a2a2e9fb07732523"
    );

    let baseline = jin_mapping_receipt_fixture(JinPhase::BaselineFrozen);
    let completed = jin_mapping_receipt_fixture(JinPhase::Chapter30Complete);
    let provider_hash = |receipt: &Value| {
        receipt["declarations"]
            .as_array()
            .expect("mapping declarations")
            .iter()
            .find(|row| row["name"] == "CrouzeixConjecture.IsPositiveRealCompletion")
            .expect("completion provider")["type_sha256"]
            .as_str()
            .expect("provider hash")
            .to_owned()
    };
    let provider_kind = |receipt: &Value| {
        receipt["declarations"]
            .as_array()
            .expect("mapping declarations")
            .iter()
            .find(|row| row["name"] == "CrouzeixConjecture.IsPositiveRealCompletion")
            .expect("completion provider")["kind"]
            .as_str()
            .expect("provider kind")
            .to_owned()
    };
    assert_eq!(
        provider_hash(&baseline),
        "484e9737cde90a145a1db689cb649f22f3ee2f1cf3e299bd52d3a2374b9686a5"
    );
    assert_eq!(
        provider_hash(&completed),
        "d72a6d691c4c807c0b20b620c2e030345c8d923a1c9b31d1a2a2e9fb07732523"
    );
    assert_eq!(provider_kind(&baseline), "definition");
    assert_eq!(provider_kind(&completed), "definition");

    let mut impossible_baseline = baseline.clone();
    impossible_baseline["declarations"]
        .as_array_mut()
        .expect("mapping declarations")
        .iter_mut()
        .find(|row| row["name"] == "CrouzeixConjecture.IsPositiveRealCompletion")
        .expect("completion provider")["kind"] = json!("theorem");
    assert!(
        jin_lean_mapping_errors(
            &jin_baseline_fixture().0,
            &impossible_baseline,
            JinPhase::BaselineFrozen,
        )
        .iter()
        .any(|error| error.contains("definition-kind provider")),
        "immutable baseline accepted an impossible theorem-kind definition provider"
    );
}

#[test]
fn jin_route_pending_cards_reject_every_unexpected_contract_field() {
    let markdown = jin_chapter_markdown(30);
    let mutated = markdown.replacen(
        "### CFT-30-001 —",
        "### CFT-30-001 —\n\n#### Statement\n\nAn unexpected partial promotion.\n\n### CFT-30-001 continuation —",
        1,
    );
    assert!(
        !jin_pending_card_errors(&mutated, 30).is_empty(),
        "pending-card validation accepted an unexpected Statement field"
    );
}

#[test]
fn chapter_30_exact_completion_interface_is_visible_and_reconstructible() {
    let markdown = jin_chapter_markdown(30);
    let visible = visible_markdown_text(&markdown);
    for required in [
        "IsPositiveRealCompletion(B,T,H)",
        "H analytic on 𝔻",
        "H(0)=I",
        "IsPositiveMatrix(Re H(z)) for z∈𝔻",
        "H(z)-(I-zT)⁻¹∈alg(Bᴴ) for z∈𝔻",
        "B=S diag(μ)S⁻¹ with μ_i distinct",
        "T=S diag(λ)S⁻¹",
        "simple spectrum belongs to B, not T",
        "‖T‖≤2",
        "completionPullbackFunction(S,H)(z)=SᴴH(z)S",
        "G=completionGramMatrix(S)=SᴴS",
        "completionKernelModel(G,λ,d)(z)",
        "G diag(j↦(1-zλ_j)⁻¹)+diag(d(z))G",
        "exists_completionKernelModel_of_isPositiveRealCompletion",
        "matrixHerglotzKernel_isPositiveMatrixKernelOn",
        "matrixHerglotzKernel_positive_congr_on",
        "norm_completionDiagonalizableMatrix_le_two_of_positiveKernelModel",
        "P_ij=G_ij/(1-conj(λ_i)λ_j/4)",
        "R_ij=G_ij/(1-conj(λ_i)λ_j/2)",
        "X=R-P",
        "K⁻¹PK⁻¹=Gramian_4(C)",
        "K⁻¹RK⁻¹=Gramian_2(C)",
        "K⁻¹XK⁻¹=Gramian_2(C)-Gramian_4(C)",
        "Y=4X-XG⁻¹P-PG⁻¹X⪰0",
        "K⁻¹YK⁻¹⪰0",
        "the_numerical_range_is_a_2_spectral_set_v4.tex#L319-L386",
        "SOURCE CLAIM.",
        "INSPECTED EVIDENCE.",
        "PEER-REVIEW / PUBLICATION STATE.",
        "HARP REPRODUCTION / FORMALIZATION STATE.",
        "2×2 worked instance.",
    ] {
        assert!(
            visible.contains(required),
            "completed Chapter 30 is missing visible exact content {required}"
        );
    }
    for index in 1..=6 {
        let item_id = format!("CFT-30-{index:03}");
        let card = jin_card(&markdown, &item_id, 30).expect("Chapter 30 theorem card");
        let history = visible_markdown_text(
            jin_card_field(card, "Historical context").expect("historical context"),
        );
        for required in [
            "SOURCE CLAIM.",
            "INSPECTED EVIDENCE.",
            "PEER-REVIEW / PUBLICATION STATE.",
            "HARP REPRODUCTION / FORMALIZATION STATE.",
        ] {
            assert!(history.contains(required), "{item_id} omits {required}");
        }
        let ml = visible_markdown_text(jin_card_field(card, "ML analogy").expect("ML analogy"));
        for required in [
            "Mathematical object / ML counterpart.",
            "Exact transfer.",
            "Non-transfer.",
            "Diagnostic.",
        ] {
            assert!(
                ml.contains(required),
                "{item_id} omits ML contract {required}"
            );
        }
        if matches!(index, 3 | 4) {
            assert!(ml.contains("operator or Frobenius norm"));
            assert!(!ml.contains("smallest eigenvalue of the Hermitian residual"));
        }
    }
}

#[test]
fn chapter_30_exact_completion_interface_rejects_comment_only_equations() {
    let markdown = jin_chapter_markdown(30);
    let card = jin_card(&markdown, "CFT-30-003", 30).expect("CFT-30-003 card");
    let statement = jin_card_field(card, "Statement").expect("CFT-30-003 statement");
    let mutated = statement.replace("    K⁻¹PK⁻¹=Gramian_4(C)", "<!-- K⁻¹PK⁻¹=Gramian_4(C) -->");
    assert_ne!(
        mutated, statement,
        "negative mutation must replace the equation"
    );
    assert!(
        !visible_markdown_text(&mutated).contains("K⁻¹PK⁻¹=Gramian_4(C)"),
        "HTML-comment equation was accepted as visible Chapter 30 mathematics"
    );
}

#[test]
fn chapter_30_completion_predicate_is_a_definition_not_a_proof() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let row = coverage["items"]
        .as_array()
        .expect("coverage rows")
        .iter()
        .find(|row| row["item_id"] == "CFT-30-001")
        .expect("CFT-30-001 coverage row");
    assert_eq!(row["kind"], "definition");
    assert_eq!(row["formal_mode"], "definition");
    assert_eq!(row["prose_proof_status"], "reconstructible");
    assert_eq!(row["lean_correspondence_status"], "exact");

    let receipt = fresh_textbook_receipt();
    let declarations = receipt["declarations"]
        .as_array()
        .expect("fresh receipt declarations");
    let public = declarations
        .iter()
        .find(|entry| entry["name"] == "CrouzeixTextbook.Part06.positive_real_completion")
        .expect("CFT-30-001 public declaration");
    let provider = declarations
        .iter()
        .find(|entry| entry["name"] == "CrouzeixConjecture.IsPositiveRealCompletion")
        .expect("CFT-30-001 provider declaration");
    assert_eq!(public["kind"], "direct-alias");
    assert_eq!(provider["kind"], "definition");

    let markdown = jin_chapter_markdown(30);
    let card = jin_card(&markdown, "CFT-30-001", 30).expect("CFT-30-001 card");
    let visible = visible_markdown_text(card);
    assert!(visible.contains("This is a definition, not a theorem"));
    assert!(!visible.contains("defining equivalence"));
    assert!(!visible.contains("theorem proof"));
}

#[test]
fn chapter_30_completion_endpoint_reexports_the_proved_theorem() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let row = coverage["items"]
        .as_array()
        .expect("coverage rows")
        .iter()
        .find(|row| row["item_id"] == "CFT-30-002")
        .expect("CFT-30-002 coverage row");
    assert_eq!(row["kind"], "theorem");
    assert_eq!(row["formal_mode"], "reexported-proof");
    assert_eq!(
        row["lean_declaration"]["underlying_declaration"],
        "CrouzeixConjecture.positiveRealCompletionStatement"
    );

    let source = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean"),
    )
    .expect("Chapter30 Lean source");
    assert!(source.contains("import CrouzeixConjecture.CompletionDiagonalization"));
    assert!(source.contains("theorem positive_real_completion_statement"));

    let receipt = fresh_textbook_receipt();
    let declarations = receipt["declarations"]
        .as_array()
        .expect("fresh receipt declarations");
    let public = declarations
        .iter()
        .find(|entry| entry["name"] == "CrouzeixTextbook.Part06.positive_real_completion_statement")
        .expect("CFT-30-002 public declaration");
    let provider = declarations
        .iter()
        .find(|entry| entry["name"] == "CrouzeixConjecture.positiveRealCompletionStatement")
        .expect("CFT-30-002 provider theorem");
    assert_eq!(public["kind"], "direct-alias");
    assert_eq!(provider["kind"], "theorem");
    assert_eq!(public["normalized_type"], provider["normalized_type"]);
    assert!(public["normalized_type"]
        .as_str()
        .expect("normalized CFT-30-002 type")
        .contains("Nonempty"));
    assert!(public["direct_dependencies"]
        .as_array()
        .expect("CFT-30-002 dependencies")
        .iter()
        .any(|dependency| dependency == "CrouzeixConjecture.positiveRealCompletionStatement"));
}

#[test]
fn chapter_30_endpoint_dependency_ledger_matches_the_compiled_proof_term() {
    let markdown = jin_chapter_markdown(30);
    let endpoint = jin_card(&markdown, "CFT-30-002", 30).expect("CFT-30-002 card");
    let ledger = visible_markdown_text(
        jin_card_field(endpoint, "Hypothesis ledger").expect("CFT-30-002 hypothesis ledger"),
    );
    for required in [
        "SimpleDiagonalization B",
        "eigenvalues",
        "changeBasis",
        "eq_conjugate",
        "does not use eigenvalues_injective",
        "suggesting that this interface may admit a future weakening",
    ] {
        assert!(
            ledger.contains(required),
            "CFT-30-002 dependency ledger omits {required}"
        );
    }

    let proof = visible_markdown_text(jin_card_field(endpoint, "Proof").expect("CFT-30-002 proof"));
    for false_consumption_claim in [
        "distinct eigenvalues μ_i are used by",
        "This is where simplicity is consumed",
    ] {
        assert!(
            !proof.contains(false_consumption_claim),
            "CFT-30-002 retains false consumption claim {false_consumption_claim}"
        );
    }
    assert!(proof.contains("No call to hB.eigenvalues_injective occurs"));

    let ml = visible_markdown_text(
        jin_card_field(endpoint, "ML analogy").expect("CFT-30-002 ML analogy"),
    );
    assert!(ml.contains("Source intuition only, not a compiled dependency"));
    assert!(ml.contains("distinct probe coordinates may identify basis components"));
    assert!(ml.contains("The exact compiled transfer is shared-basis diagonalization"));

    let provider = fs::read_to_string(
        workspace_root()
            .join("formalization/lean/CrouzeixConjecture/CompletionDiagonalization.lean"),
    )
    .expect("completion-diagonalization provider");
    let route = provider
        .split_once("theorem exists_diagonal_correction_of_mem_generatedAlgebra_conjTranspose")
        .expect("local algebra-to-diagonal-correction route")
        .1
        .split_once("/-- Pointwise generated-algebra membership")
        .expect("end of local algebra-to-diagonal-correction route")
        .0;
    for field in ["hB.eigenvalues", "hB.changeBasis", "hB.eq_conjugate"] {
        assert!(route.contains(field), "compiled route omits {field}");
    }
    assert!(
        !route.contains("eigenvalues_injective"),
        "compiled Chapter 30 route unexpectedly started consuming injectivity"
    );

    let gramian = jin_card(&markdown, "CFT-30-003", 30).expect("CFT-30-003 card");
    let gramian_ml = visible_markdown_text(
        jin_card_field(gramian, "ML analogy").expect("CFT-30-003 ML analogy"),
    );
    assert!(gramian_ml.contains("G=K² and K⁻¹K=KK⁻¹=I"));
    assert!(gramian_ml.contains("K⁻¹ G K⁻¹ = K⁻¹ K² K⁻¹ = I"));
    assert!(gramian_ml.contains("Only associativity and the two inverse identities are used"));
    assert!(gramian_ml.contains("no commutation hypothesis"));
}

#[test]
fn chapter_30_difference_and_positivity_cards_match_their_lean_statements() {
    let markdown = jin_chapter_markdown(30);
    let difference = jin_card(&markdown, "CFT-30-005", 30).expect("CFT-30-005 card");
    let difference_statement = visible_markdown_text(
        jin_card_field(difference, "Statement").expect("CFT-30-005 statement"),
    );
    assert!(difference_statement.contains("Define X:=R-P"));
    assert!(difference_statement.contains("The conclusion is the single congruence"));
    assert!(!difference_statement.contains("X=R-P,"));

    let coverage = read_json(&contracts_root().join("coverage.json"));
    let source_positive = coverage["items"]
        .as_array()
        .expect("coverage rows")
        .iter()
        .find(|row| row["item_id"] == "CFT-30-006")
        .expect("CFT-30-006 row");
    assert_eq!(source_positive["formal_mode"], "proved-here");
    assert!(source_positive["lean_declaration"]["underlying_declaration"].is_null());

    let source = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean"),
    )
    .expect("Chapter30 Lean source");
    assert!(source.contains("theorem completion_gramian_source_positive"));

    let receipt = fresh_textbook_receipt();
    let declaration = receipt["declarations"]
        .as_array()
        .expect("fresh receipt declarations")
        .iter()
        .find(|row| row["name"] == "CrouzeixTextbook.Part06.completion_gramian_source_positive")
        .expect("CFT-30-006 public theorem");
    assert_eq!(declaration["kind"], "theorem");
    assert!(declaration["normalized_type"]
        .as_str()
        .expect("CFT-30-006 normalized type")
        .contains("And"));
    assert!(declaration["direct_dependencies"]
        .as_array()
        .expect("CFT-30-006 dependencies")
        .iter()
        .any(|dependency| {
            dependency == "CrouzeixConjecture.completion_gramian_expression_posSemidef_of_source"
        }));

    let source_audit = jin_receipt_audit(
        jin_card_field(
            jin_card(&markdown, "CFT-30-006", 30).expect("CFT-30-006 card"),
            "Lean correspondence",
        )
        .expect("CFT-30-006 Lean correspondence"),
    )
    .expect("CFT-30-006 receipt audit");
    assert_eq!(
        source_audit.get("provider-type-sha256").map(String::as_str),
        Some("54e0b92ee477409a1b741eb87362322c7a801bb9c9ecdb4b61a1320140728baa")
    );
}

#[test]
fn chapter_30_two_by_two_example_notation_and_diagnostics_are_exact() {
    let markdown = jin_chapter_markdown(30);
    let visible = visible_markdown_text(&markdown);
    assert_math_steps_in_order(
        &markdown,
        "Chapter30 exact two-by-two calculation",
        &[
            "K=\\begin{pmatrix}2&1\\\\1&2\\end{pmatrix}",
            "G=K^2=\\begin{pmatrix}5&4\\\\4&5\\end{pmatrix}",
            "K^{-1}=\\frac13\\begin{pmatrix}2&-1\\\\-1&2\\end{pmatrix}",
            "\\Lambda=\\operatorname{diag}(1,0)",
            "C=K\\Lambda K^{-1}",
            "\\frac13\\begin{pmatrix}4&-2\\\\2&-1\\end{pmatrix}",
            "P=\\begin{pmatrix}20/3&4\\\\4&5\\end{pmatrix}",
            "R=\\begin{pmatrix}10&4\\\\4&5\\end{pmatrix}",
            "X=\\begin{pmatrix}10/3&0\\\\0&0\\end{pmatrix}",
            "K^{-1}PK^{-1}",
            "\\begin{pmatrix}47/27&-10/27\\\\-10/27&32/27\\end{pmatrix}",
            "\\operatorname{Gramian}_4(C)",
            "K^{-1}RK^{-1}",
            "\\begin{pmatrix}29/9&-10/9\\\\-10/9&14/9\\end{pmatrix}",
            "\\operatorname{Gramian}_2(C)",
            "K^{-1}XK^{-1}",
            "\\begin{pmatrix}40/27&-20/27\\\\-20/27&10/27\\end{pmatrix}",
            "\\operatorname{Gramian}_2(C)-\\operatorname{Gramian}_4(C)",
        ],
    );
    for notation in ["G=SᴴS", "S∈GL_n(ℂ)", "λ:n→ℂ", "q∈ℝ", "q>1"] {
        assert!(
            visible.contains(notation),
            "Chapter30 omits notation {notation}"
        );
    }
    assert!(!visible.contains("G=S*S"));
    assert!(visible.contains("operator or Frobenius norm"));
    assert!(visible.contains("smallest eigenvalue only for PSD"));
    assert!(visible.contains("K⁻¹ is the exact whitening map"));
}

#[test]
fn chapter_30_exercises_expose_exact_signatures_links_and_written_derivations() {
    let markdown = jin_chapter_markdown(30);
    let exercises = markdown
        .split_once("## Exercises")
        .expect("Chapter30 exercises")
        .1;
    for index in 1..=6 {
        let marker = format!("### CFT-30-E{index:02} --");
        let end_marker = if index < 6 {
            format!("### CFT-30-E{:02} --", index + 1)
        } else {
            "### Written solutions".to_owned()
        };
        let section = exercises
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {marker}"))
            .1
            .split_once(&end_marker)
            .map(|(body, _)| body)
            .expect("exercise boundary");
        let declaration =
            format!("CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_{index:02}_solution");
        assert!(
            section.contains("Lean declaration:"),
            "E{index:02} lacks link label"
        );
        assert!(section.contains("[[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|"));
        assert!(section.contains(&declaration));
        assert!(section.contains("theorem exercise_"));
    }
    for required in [
        "(B T : SquareMatrix n) (H : ℂ → SquareMatrix n)",
        "(hcompletion : IsPositiveRealCompletion B T H) : Commute B T",
        "{G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)",
        "(lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1) (i j : n)",
        "completionX G lambda = completionR G lambda - completionP G lambda ∧",
        "(hsource : (4 * completionX G lambda -",
    ] {
        assert!(
            exercises.contains(required),
            "exercise prompts omit `{required}`"
        );
    }

    let solutions = exercises
        .split_once("### Written solutions")
        .expect("written solutions")
        .1;
    for (exercise_id, equations) in [
        (
            "CFT-30-E02",
            &["diag(μ)diag(λ)=diag(λ)diag(μ)", "BT=TB"][..],
        ),
        ("CFT-30-E03", &["P_ij=Σ_{k≥0}", "K⁻¹PK⁻¹=Gramian_4(C)"][..]),
        ("CFT-30-E04", &["R_ij=Σ_{k≥0}", "K⁻¹RK⁻¹=Gramian_2(C)"][..]),
        (
            "CFT-30-E05",
            &["K⁻¹(R-P)K⁻¹", "Gramian_2(C)-Gramian_4(C)"][..],
        ),
        (
            "CFT-30-E06",
            &["K⁻¹YK⁻¹⪰0", "K⁻¹YK⁻¹=4(G₂-G₄)-(G₂-G₄)G₄-G₄(G₂-G₄)"][..],
        ),
    ] {
        let marker = format!("#### {exercise_id} solution");
        let body = solutions
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {marker}"))
            .1
            .split_once("\n#### CFT-30-")
            .map_or_else(
                || solutions.split_once(&marker).expect("solution marker").1,
                |(body, _)| body,
            );
        assert!(
            body.split_whitespace().count() >= 65,
            "{exercise_id} solution is too thin"
        );
        for equation in equations {
            assert!(body.contains(equation), "{exercise_id} omits `{equation}`");
        }
    }

    let exercise_contract = read_json(&contracts_root().join("exercises.json"));
    let e05 = exercise_contract["exercises"]
        .as_array()
        .expect("exercise rows")
        .iter()
        .find(|row| row["exercise_id"] == "CFT-30-E05")
        .expect("CFT-30-E05 row");
    assert_eq!(e05["kind"], "written-proof");
    assert_eq!(e05["difficulty"], "proof");
}

#[test]
fn chapter_30_e06_builds_the_ordered_identity_from_smaller_bridges() {
    let source = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean"),
    )
    .expect("Chapter30 Lean source");
    let proof = source
        .split_once("theorem exercise_06_solution")
        .expect("Chapter30 E06 theorem")
        .1
        .split_once("\nend Exercises.Chapter30")
        .expect("Chapter30 exercise namespace boundary")
        .0;
    for dependency in [
        "completion_congruence_identity",
        "completionGram_inverse_eq",
        "completionX_congruence_eq_gramian_difference",
        "completionP_congruence_eq_gramian_four",
    ] {
        assert!(
            proof.contains(dependency),
            "Chapter30 E06 omits the smaller bridge `{dependency}`"
        );
    }
    assert!(
        !proof.contains("completion_PRX_congruence_eq_gramian_expression"),
        "Chapter30 E06 must not call the provider that proves its entire equality field"
    );
}

#[test]
fn chapter_30_atlas_render_preserves_adjoint_and_matrix_notation() {
    let corpus = read_json(&workspace_root().join("atlas/src/content/generated/corpus.json"));
    let chapter = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| document["title"] == "Chapter 30: Jin’s positive-real completion")
        .expect("Chapter 30 Atlas document");
    let corpus = chapter["html"].as_str().expect("Chapter 30 Atlas HTML");
    for rendered in [
        "G=SᴴS",
        "class=\"math math-display\"",
        "K=\\begin{pmatrix}2&amp;1\\\\1&amp;2\\end{pmatrix}",
        "P=\\begin{pmatrix}20/3&amp;4\\\\4&amp;5\\end{pmatrix}",
        "K^{-1}PK^{-1}",
        "47/27&amp;-10/27",
        "Written solutions",
    ] {
        assert!(corpus.contains(rendered), "Atlas corpus omits `{rendered}`");
    }
    for broken in ["G=S*S", "H(z)<em>", "v</em>Re H(z)v", "(H⁻¹)*YH⁻¹"] {
        assert!(
            !corpus.contains(broken),
            "Atlas contains broken adjoint render `{broken}`"
        );
    }
}

#[test]
fn chapter_30_lean_source_exports_exact_six_solutions_without_ls() {
    let source = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean"),
    )
    .expect("Chapter 30 Lean source");
    assert!(source.contains("namespace Exercises.Chapter30"));
    for index in 1..=6 {
        assert_eq!(
            source
                .matches(&format!("theorem exercise_{index:02}_solution"))
                .count(),
            1,
            "Chapter 30 must define exercise_{index:02}_solution exactly once"
        );
    }
    assert!(!source.contains("LoristSchwenninger"));
    assert!(!source.contains("Crouzeix.LoristSchwenninger"));
}

#[test]
fn chapter_31_kernel_samples_and_cancellation_are_visible_and_reconstructible() {
    let markdown = jin_chapter_markdown(31);
    let visible = visible_markdown_text(&markdown);
    for required in [
        "G∈M_n(ℂ)",
        "Λ=diag(λ)",
        "D(z)=diag(d_i(z))",
        "Q(z)=diag((1-zλ_j)⁻¹)",
        "K(z)=GQ(z)+D(z)G",
        "K(z)_ij=G_ij(1-zλ_j)⁻¹+d_i(z)G_ij",
        "z_0=0",
        "z_i=conj(λ_i)/2",
        "x_0=v=-G⁻¹Pu",
        "x_i=u_i e_i∈ℂⁿ",
        "sample/sample=4R-2P",
        "sample/origin=G+R",
        "origin/sample=G+R",
        "origin/origin=2G",
        "Θ=Σ_i conj(u_i)d_i(conj(λ_i)/2)(Gv+Pu)_i",
        "Gv+Pu=-GG⁻¹Pu+Pu=0",
        "Θ+conj(Θ)=0",
        "Scalar two-sample calculation.",
        "SOURCE CLAIM.",
        "INSPECTED EVIDENCE.",
        "PEER-REVIEW / PUBLICATION STATE.",
        "HARP REPRODUCTION / FORMALIZATION STATE.",
    ] {
        assert!(
            visible.contains(required),
            "completed Chapter 31 is missing visible exact content `{required}`"
        );
    }
    assert_math_steps_in_order(
        &visible,
        "Chapter31 positivity transitions",
        &[
            "pointwise positive-real",
            "sampled-kernel PSD",
            "block-matrix PSD",
            "scalar quadratic inequality",
            "final X PSD",
        ],
    );
}

#[test]
fn chapter_31_defines_domains_and_the_two_variable_kernel_before_use() {
    let markdown = jin_chapter_markdown(31);
    let visible = visible_markdown_text(&markdown);
    let definitions = [
        "P_ij=G_ij/(1-conj(λ_i)λ_j/4)",
        "R_ij=G_ij/(1-conj(λ_i)λ_j/2)",
        "X=R-P",
        "L_K(z,w)=(1-zconj(w))⁻¹(K(z)+K(w)ᴴ)",
        "Q(z)=diag((1-zλ_j)⁻¹)",
    ];
    for definition in definitions {
        assert!(
            visible.contains(definition),
            "Chapter31 omits the defining formula `{definition}`"
        );
    }
    let first_p_use = visible.find("matrices P, R").expect("opening P/R use");
    assert!(
        definitions[..3].iter().all(|definition| visible
            .find(definition)
            .is_some_and(|offset| offset < first_p_use)),
        "P, R, and X must be defined before the opening problem uses them"
    );
    for ambiguous in ["K(z_i,z_j)", "K_H"] {
        assert!(
            !visible.contains(ambiguous),
            "Chapter31 retains ambiguous kernel notation `{ambiguous}`"
        );
    }
    assert_math_steps_in_order(
        &visible,
        "Chapter31 diagonal inverse boundary",
        &[
            "Q(z)=diag((1-zλ_j)⁻¹)",
            "1-zλ_j≠0 for every j",
            "Q(z)=(I-zΛ)⁻¹",
        ],
    );
}

#[test]
fn chapter_31_derives_each_ordered_resolvent_and_correction_block() {
    let markdown = jin_chapter_markdown(31);
    let visible = visible_markdown_text(&markdown);
    for required in [
        "a_ij=conj(λ_i)λ_j",
        "2/((1-a_ij/4)(1-a_ij/2))=4/(1-a_ij/2)-2/(1-a_ij/4)",
        "L_0(z_i,z_j)_ij=(4R-2P)_ij",
        "L_0(z_i,0)_ij=(G+R)_ij",
        "L_0(0,z_j)_ij=(G+R)_ij",
        "L_0(0,0)_ij=(2G)_ij",
        "conj(u_i)L_0(z_i,z_j)_ij u_j=conj(u_i)(4R-2P)_ij u_j",
        "Σ_i Σ_j conj(u_i)(4R-2P)_ij u_j=uᴴ(4R-2P)u",
        "L_D(z_i,z_j)_ij=(ΔP+PΔᴴ)_ij",
        "L_D(z_i,0)_ij=(ΔG)_ij",
        "L_D(0,z_j)_ij=(GΔᴴ)_ij",
        "L_D(0,0)=0",
        "P=Pᴴ",
        "correction sampling sum=Θ+conj(Θ)",
        "Gv+Pu=0",
        "correction sampling sum=0",
        "completionCorrectionKernel_sample_sample_apply",
        "completionCorrectionKernel_sample_zero_apply",
        "completionCorrectionKernel_zero_sample_apply",
        "completionCorrectionKernel_sampling_eq_unknownContribution",
        "Exercises.Chapter31.exercise_04_solution",
    ] {
        assert!(
            visible.contains(required),
            "Chapter31 omits ordered block derivation `{required}`"
        );
    }
    assert_math_steps_in_order(
        &visible,
        "Chapter31 correction derivation",
        &[
            "L_D(z_i,z_j)_ij=(ΔP+PΔᴴ)_ij",
            "L_D(z_i,0)_ij=(ΔG)_ij",
            "L_D(0,z_j)_ij=(GΔᴴ)_ij",
            "L_D(0,0)=0",
            "correction sampling sum=vᴴ(GΔᴴ)u+uᴴ(ΔG)v+uᴴ(ΔP+PΔᴴ)u",
            "correction sampling sum=Θ+conj(Θ)",
            "Gv+Pu=0",
            "correction sampling sum=0",
        ],
    );
}

#[test]
fn chapter_31_proves_the_analytic_and_hermitian_positivity_boundaries() {
    let markdown = jin_chapter_markdown(31);
    let visible = visible_markdown_text(&markdown);
    assert_math_steps_in_order(
        &visible,
        "Chapter31 analytic boundary",
        &[
            "analyticity of K on 𝔻",
            "pointwise positive-real",
            "L_K positive as a matrix kernel on 𝔻",
            "compiled CFT-31-006 begins with kernel positivity",
        ],
    );
    assert_math_steps_in_order(
        &visible,
        "Chapter31 positivity-kind transitions",
        &[
            "analyticity + pointwise positive-real ⇒ L_K kernel-positive",
            "L_K kernel-positive ⇒ sampled-kernel PSD",
            "sampled-kernel PSD ⇒ block-matrix PSD",
            "block-matrix PSD ⇒ finite quadratic sum≥0",
            "correction sum=0",
            "finite quadratic sum=u*(4X-XG⁻¹P-PG⁻¹X)u",
            "quadratic nonnegativity ⇒ matrix PSD",
        ],
    );
    for required in [
        "P=Pᴴ",
        "R=Rᴴ",
        "X=Xᴴ",
        "(G⁻¹)ᴴ=G⁻¹",
        "(XG⁻¹P)ᴴ=PG⁻¹X",
        "(PG⁻¹X)ᴴ=XG⁻¹P",
        "(4X-XG⁻¹P-PG⁻¹X)ᴴ=4X-XG⁻¹P-PG⁻¹X",
    ] {
        assert!(
            visible.contains(required),
            "Chapter31 omits Hermiticity step `{required}`"
        );
    }
}

#[test]
fn chapter_31_scalar_pairing_and_copyable_exercise_signatures_are_exact() {
    let markdown = jin_chapter_markdown(31);
    let visible = visible_markdown_text(&markdown);
    for required in [
        "p=g/(1-|λ|²/4)",
        "r=g/(1-|λ|²/2)",
        "z_0=0",
        "z_1=conj(λ)/2",
        "conj(v)gconj(δ)u+conj(u)δgv+|u|²(δp+pconj(δ))",
        "conj(u_a)(4R-2P)_ab u_b",
    ] {
        assert!(
            visible.contains(required),
            "scalar example omits `{required}`"
        );
    }
    assert!(
        !visible.contains("[ 2G      G+R      G+R   ]"),
        "the misleading matrix-of-matrix-blocks display remains"
    );
    for index in 1..=6 {
        let signature = format!(
            "theorem exercise_{index:02}_solution\n    {{n : Type*}} [Fintype n] [DecidableEq n]"
        );
        assert!(
            markdown.contains(&signature),
            "CFT-31-E{index:02} does not publish a copyable full signature"
        );
    }
    assert!(
        !visible.contains("Before using it, show that multiplication"),
        "E01 asks for a sparse-column proposition absent from its checked conclusion"
    );
    assert!(
        visible.contains(
            "(completionKernelModel G lambda d z *ᵥ completionSparseVector (fun _ ↦ (1 : ℂ)) j) i = completionKernelModel G lambda d z i j ∧"
        ),
        "E01 checked proposition omits sparse-column selection"
    );
    assert!(
        visible.contains(
            "With E04's Hermiticity hypothesis and d(0)=0, this becomes the complete correction contribution"
        ),
        "E05 does not state the hypotheses needed to identify its scalar with the full correction sum"
    );
}

#[test]
fn chapter_31_reverse_normalization_is_proved_here_with_invertibility() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let row = coverage["items"]
        .as_array()
        .expect("coverage rows")
        .iter()
        .find(|row| row["item_id"] == "CFT-31-002")
        .expect("CFT-31-002 row");
    assert_eq!(row["formal_mode"], "proved-here");
    assert_eq!(row["lean_correspondence_status"], "exact");
    assert!(row["lean_declaration"]["underlying_declaration"].is_null());
    assert_eq!(
        row["lean_declaration"]["name"],
        "CrouzeixTextbook.Part06.completion_kernel_normalization_forces_correction_zero"
    );

    let source = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean"),
    )
    .expect("Chapter31 Lean source");
    assert!(
        source.contains("theorem completion_kernel_normalization_forces_correction_zero_bridge")
    );
    assert!(source.contains("theorem completion_kernel_normalization_forces_correction_zero"));
    let public_proof = source
        .split_once("theorem completion_kernel_normalization_forces_correction_zero\n")
        .expect("CFT-31-002 public theorem")
        .1
        .split_once("namespace Exercises.Chapter31")
        .expect("Chapter31 exercise boundary")
        .0;
    assert!(public_proof.contains("completion_kernel_normalization_forces_correction_zero_bridge"));
    assert!(!public_proof.contains("completionKernelModel_zero"));
}

#[test]
fn chapter_31_exercises_are_exact_distinct_proofs_without_endpoint_shortcuts() {
    let source = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean"),
    )
    .expect("Chapter31 Lean source");
    assert!(source.contains("namespace Exercises.Chapter31"));
    for index in 1..=6 {
        assert_eq!(
            source
                .matches(&format!("theorem exercise_{index:02}_solution"))
                .count(),
            1,
            "Chapter31 must define exercise_{index:02}_solution exactly once"
        );
    }
    let exercise_body = source
        .split_once("namespace Exercises.Chapter31")
        .expect("Chapter31 exercise namespace")
        .1;
    for forbidden in [
        "kernel_positivity_implies_X",
        "completion_X_inequality_of_positiveKernel",
        "jinFinalCrouzeixConjecture",
    ] {
        assert!(
            !exercise_body.contains(forbidden),
            "Chapter31 exercises use forbidden endpoint `{forbidden}`"
        );
    }
    assert!(exercise_body.contains("mulVec_completionSparseVector"));
    assert!(exercise_body.contains("completionUnknownHalfContribution_eq_zero"));

    let markdown = jin_chapter_markdown(31);
    for index in 1..=6 {
        let declaration =
            format!("CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_{index:02}_solution");
        assert!(markdown.contains(&declaration));
        assert!(markdown.contains("[[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|"));
        assert!(markdown.contains(&format!("#### CFT-31-E{index:02} solution")));
    }
}

#[test]
fn chapter_31_ml_contract_names_exact_transfer_limits_and_diagnostic() {
    let markdown = jin_chapter_markdown(31);
    let diagnostics = [
        ("CFT-31-001", "entrywise model residual"),
        ("CFT-31-002", "‖D(0)‖≤‖K(0)-G‖‖G⁻¹‖"),
        ("CFT-31-003", "four ordered block residuals"),
        ("CFT-31-004", "raw correction sum against 2Re(Θ)"),
        ("CFT-31-005", "‖Gv+Pu‖ and the cancellation residual"),
        ("CFT-31-006", "two smallest-eigenvalue comparisons"),
    ];
    for index in 1..=6 {
        let item_id = format!("CFT-31-{index:03}");
        let card = jin_card(&markdown, &item_id, 31).expect("Chapter31 theorem card");
        let ml = visible_markdown_text(
            jin_card_field(card, "ML analogy").expect("Chapter31 ML analogy"),
        );
        for required in [
            "Mathematical object / ML counterpart.",
            "feature geometry",
            "Exact transfer.",
            "finite certificate",
            "Non-transfer.",
            "noisy empirical kernels",
            "floating-point PSD",
            "Diagnostic.",
        ] {
            assert!(
                ml.contains(required),
                "{item_id} omits ML contract `{required}`"
            );
        }
        let diagnostic = diagnostics
            .iter()
            .find_map(|(id, diagnostic)| (*id == item_id).then_some(*diagnostic))
            .expect("Chapter31 theorem-specific diagnostic");
        assert!(
            ml.contains(diagnostic),
            "{item_id} omits theorem-specific diagnostic `{diagnostic}`"
        );
    }
}

#[test]
fn chapter_31_atlas_preserves_kernel_and_block_math() {
    let corpus = read_json(&workspace_root().join("atlas/src/content/generated/corpus.json"));
    let chapter = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| document["title"] == "Chapter 31: Jin’s correction cancellation")
        .expect("Chapter 31 Atlas document");
    let html = chapter["html"].as_str().expect("Chapter31 Atlas HTML");
    for rendered in [
        "K(z)=GQ(z)+D(z)G",
        "L_K(z,w)=(1-zconj(w))⁻¹(K(z)+K(w)ᴴ)",
        "sample/sample=4R-2P",
        "origin/origin=2G",
        "Scalar two-sample calculation.",
        "Written solutions",
    ] {
        assert!(html.contains(rendered), "Atlas corpus omits `{rendered}`");
    }
}

#[test]
fn chapter_32_norm_extraction_is_ordered_and_reconstructible() {
    let markdown = jin_chapter_markdown(32);
    let visible = visible_markdown_text(&markdown);
    for required in [
        "G=SᴴS",
        "K=G^{1/2}",
        "K⁻¹K=KK⁻¹=I",
        "C=KΛK⁻¹",
        "G₂=Gramian_2(C)",
        "G₄=Gramian_4(C)",
        "X=G₂-G₄",
        "4X-XG₄-G₄X⪰0",
        "choose e with G₄e=pe",
        "assume p>2",
        "e*Xe=0",
        "Xe=0",
        "Y=(1/4)C*C",
        "X-Y⪰0 and Y⪰0",
        "e*Ye=0",
        "Ce=0",
        "G₄e=e",
        "G₄e=pe gives p=1, contradiction",
        "G₄≤2I",
        "G₄=I+Σ_{k≥1}4⁻ᵏ(Cᵏ)*Cᵏ≥I",
        "C*C≤4(G₄-I)≤4I",
        "‖C‖≤2",
        "S=UK",
        "‖SΛS⁻¹‖=‖UCU⁻¹‖=‖C‖≤2",
    ] {
        assert!(visible.contains(required), "Chapter32 omits `{required}`");
    }
    assert_math_steps_in_order(
        &visible,
        "Chapter32 eigenvector contradiction",
        &[
            "choose e with G₄e=pe",
            "assume p>2",
            "e*Xe=0",
            "Xe=0",
            "Y=(1/4)C*C",
            "e*Ye=0",
            "Ce=0",
            "G₄e=e",
            "p=1",
            "G₄≤2I",
            "C*C≤4(G₄-I)≤4I",
            "‖C‖≤2",
            "‖SΛS⁻¹‖=‖C‖≤2",
        ],
    );
}

#[test]
fn chapter_32_normalization_rational_and_limit_passages_are_distinct() {
    let markdown = jin_chapter_markdown(32);
    let visible = visible_markdown_text(&markdown);
    let polynomial_card =
        visible_markdown_text(jin_card(&markdown, "CFT-32-002", 32).expect("CFT-32-002 card"));
    for required in [
        "M=max_{z∈W(A)}|p(z)|",
        "M=0 ⇒ p(A)=0",
        "q=p/M",
        "max_{W(A)}|q|≤1",
        "p(A)=M q(A)",
        "poles(r)∩W(A)=∅",
        "U_r=ℂ\\poles(r)",
        "r(A)=num(r)(A)den(r)(A)⁻¹",
        "M_r=max_{z∈W(A)}|r(z)|",
        "s=r/M_r",
        "polynomial normalization and rational normalization are different operations",
        "A_j→A",
        "f(A_j)→f(A)",
        "W(A_j)⊆Ω_m eventually",
        "max_{closure(Ω_m)}|f|→max_{W(A)}|f|",
        "first j→∞ with m fixed",
        "then m→∞",
        "holomorphicMatrixEval(A,p)=polynomialEval(p,A)",
        "maxFunctionModulusOnSet(W(A),p)=maxPolynomialModulusOnNumericalRange(A,p)",
    ] {
        assert!(visible.contains(required), "Chapter32 omits `{required}`");
    }
    for required in [
        "those eigenvalues lie in W(A_j), not necessarily W(A)",
        "M_m=max_{z∈closure(Ω_m)}|p(z)|",
        "q_m=p/M_m",
        "σ(A_j)⊆W(A_j)⊆Ω_m",
        "CFT-32-001 therefore gives",
        "First let j→∞ with m fixed",
        "Only then let m→∞",
        "M_m→M",
        "|q_m(z)|≤1 for every z∈closure(Ω_m)",
        "μ_{j,i}=q_m(λ_{j,i})",
        "q_m(A_j)=S_j diag(μ_j) S_j⁻¹",
        "outer-boundary Cauchy identities",
        "construct the positive-real completion",
        "d_j(0)=0",
        "positive Herglotz kernel",
        "the eigenvalue bound is necessary but does not itself supply the completion",
    ] {
        assert!(
            polynomial_card.contains(required),
            "CFT-32-002 omits outer-domain normalization step `{required}`"
        );
    }
    assert_math_steps_in_order(
        &polynomial_card,
        "CFT-32-002 outer-domain normalization",
        &[
            "M_m=max_{z∈closure(Ω_m)}|p(z)|",
            "q_m=p/M_m",
            "A_j→A",
            "σ(A_j)⊆W(A_j)⊆Ω_m",
            "μ_{j,i}=q_m(λ_{j,i})",
            "q_m(A_j)=S_j diag(μ_j) S_j⁻¹",
            "outer-boundary Cauchy identities",
            "construct the positive-real completion",
            "d_j(0)=0",
            "positive Herglotz kernel",
            "CFT-32-001 therefore gives",
            "‖q_m(A_j)‖≤2",
            "First let j→∞ with m fixed",
            "Only then let m→∞",
            "M_m→M",
            "‖p(A)‖≤2M",
        ],
    );
    assert_math_steps_in_order(
        &visible,
        "Chapter32 ordered limits",
        &[
            "fix m",
            "A_j→A",
            "W(A_j)⊆Ω_m eventually",
            "f(A_j)→f(A)",
            "first j→∞ with m fixed",
            "then m→∞",
            "max_{closure(Ω_m)}|f|→max_{W(A)}|f|",
            "f=p",
            "holomorphicMatrixEval(A,p)=polynomialEval(p,A)",
        ],
    );
}

#[test]
fn chapter_32_rational_spectral_set_names_its_local_provider_and_proof_dependency() {
    let markdown = jin_chapter_markdown(32);
    let card =
        visible_markdown_text(jin_card(&markdown, "CFT-32-003", 32).expect("CFT-32-003 card"));
    for required in [
        "Provider: Chapter32.lean",
        "CrouzeixTextbook.Part06.jin_rational_spectral_set_provider",
        "CrouzeixConjecture.holomorphicCrouzeixRationalBound is its proof dependency, not the direct provider",
    ] {
        assert!(card.contains(required), "CFT-32-003 omits `{required}`");
    }
    assert!(
        !card.contains("Provider: HolomorphicConsequences.lean"),
        "CFT-32-003 mislabels the dependency as its direct provider"
    );
}

#[test]
fn chapter_32_exercises_publish_exact_signatures_and_full_solutions() {
    let markdown = jin_chapter_markdown(32);
    let exercises = markdown
        .split_once("## Exercises")
        .expect("Chapter32 exercises")
        .1;
    for index in 1..=6 {
        let declaration =
            format!("CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_{index:02}_solution");
        assert!(exercises.contains(&declaration));
        assert!(exercises.contains("[[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|"));
        assert!(exercises.contains(&format!("#### CFT-32-E{index:02} solution")));
        assert!(exercises.contains(&format!("theorem exercise_{index:02}_solution")));
    }
    for required in [
        "(hbound : ∀ k : ℕ, ‖C ^ k‖ ≤ M)",
        "four_sub_conjTranspose_mul_self_posSemidef_of_gramian_inequality",
        "(M = 0 → polynomialEval p A = 0) ∧",
        "IsOpen (rationalPoleSet r)ᶜ ∧ s ⊆ (rationalPoleSet r)ᶜ",
        "(M = 0 → rationalMatrixEval r A = 0) ∧",
        "Tendsto (simpleSpectrumHolomorphicEval A f) atTop",
        "holomorphicMatrixEval A (fun z ↦ Polynomial.eval z p) = polynomialEval p A ∧",
    ] {
        assert!(
            exercises.contains(required),
            "Chapter32 exercises omit `{required}`"
        );
    }
    for exercise_id in ["CFT-32-E01", "CFT-32-E02", "CFT-32-E04", "CFT-32-E05"] {
        let marker = format!("#### {exercise_id} solution");
        let body = exercises
            .split_once(&marker)
            .unwrap_or_else(|| panic!("missing {marker}"))
            .1
            .split_once("\n#### CFT-32-")
            .map_or_else(
                || exercises.split_once(&marker).unwrap().1,
                |(body, _)| body,
            );
        assert!(
            body.split_whitespace().count() >= 75,
            "{exercise_id} solution is too thin"
        );
    }
}

#[test]
fn chapter_32_lean_solutions_are_distinct_and_avoid_terminal_shortcuts() {
    let source = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean"),
    )
    .expect("Chapter32 Lean source");
    assert!(source.contains("theorem completion_implies_norm_two"));
    assert!(source.contains("namespace Exercises.Chapter32"));
    for index in 1..=6 {
        assert_eq!(
            source
                .matches(&format!("theorem exercise_{index:02}_solution"))
                .count(),
            1,
            "Chapter32 must define exercise_{index:02}_solution exactly once"
        );
    }
    let exercise_body = source
        .split_once("namespace Exercises.Chapter32")
        .unwrap()
        .1;
    for forbidden in [
        "jinFinalCrouzeixConjecture",
        "completion_implies_norm_two",
        "norm_completionDiagonalizableMatrix_le_two_of_positiveKernelModel",
        "crouzeixRationalBound",
        "holomorphicCrouzeixBound",
        "polynomialCrouzeixBound_of_holomorphicCrouzeixBound",
    ] {
        assert!(
            !exercise_body.contains(forbidden),
            "Chapter32 exercise uses `{forbidden}`"
        );
    }
    for required in [
        "four_sub_conjTranspose_mul_self_posSemidef_of_gramian_inequality",
        "matrix_norm_le_two_of_four_sub_conjTranspose_mul_self_posSemidef",
        "differentiableOn_rationalScalarEval",
        "tendsto_simpleSpectrumHolomorphicEval_of_differentiableOn_neighborhood",
        "tendsto_maxFunctionModulusOnSet_of_outerApproximation",
        "holomorphicMatrixEval_polynomial",
    ] {
        assert!(
            exercise_body.contains(required),
            "Chapter32 Lean omits `{required}`"
        );
    }
}

#[test]
fn chapter_32_history_ml_and_running_example_are_concrete() {
    let markdown = jin_chapter_markdown(32);
    let visible = visible_markdown_text(&markdown);
    for required in [
        "SOURCE CLAIM.",
        "INSPECTED EVIDENCE.",
        "PEER-REVIEW / PUBLICATION STATE.",
        "HARP REPRODUCTION / FORMALIZATION STATE.",
        "the_numerical_range_is_a_2_spectral_set_v4.tex#L464-L530",
        "the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910",
        r"A_{λ,α}=\begin{pmatrix}λ&α\\0&-λ\end{pmatrix}",
        "Mathematical object / ML counterpart.",
        "feature geometry",
        "Exact transfer.",
        "finite certificate",
        "Non-transfer.",
        "noisy empirical kernels",
        "floating-point PSD",
        "Diagnostic.",
        "smallest eigenvalue of 4I-C*C",
        "ordered-limit residual",
    ] {
        assert!(visible.contains(required), "Chapter32 omits `{required}`");
    }
    for index in 1..=6 {
        let item_id = format!("CFT-32-{index:03}");
        let card = jin_card(&markdown, &item_id, 32).expect("Chapter32 theorem card");
        let ml = visible_markdown_text(jin_card_field(card, "ML analogy").unwrap());
        for label in [
            "Mathematical object / ML counterpart.",
            "Exact transfer.",
            "Non-transfer.",
            "Diagnostic.",
        ] {
            assert!(ml.contains(label), "{item_id} omits `{label}`");
        }
    }
}

#[test]
fn chapter_32_atlas_preserves_endpoint_and_limit_math() {
    let corpus = read_json(&workspace_root().join("atlas/src/content/generated/corpus.json"));
    let chapter = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| document["title"] == "Chapter 32: Jin’s constant-two endpoint")
        .expect("Chapter32 Atlas document");
    let html = chapter["html"].as_str().expect("Chapter32 Atlas HTML");
    for rendered in [
        "4X-XG₄-G₄X⪰0",
        "Y=(1/4)C*C",
        "G₄e=e",
        "first j→∞ with m fixed",
        "then m→∞",
        "Written solutions",
    ] {
        assert!(html.contains(rendered), "Atlas corpus omits `{rendered}`");
    }
}

#[test]
fn jin_route_complete_cards_reject_filler_without_scoped_equations() {
    let words = "filler ".repeat(90);
    let mut markdown = String::new();
    for index in 1..=6 {
        markdown.push_str(&format!(
            "### CFT-30-{index:03} — filler {{#cft-30-{index:03}}}\n\n\
             #### Purpose\n\n**Motivation.** {words}\n\n\
             #### Statement\n\n{words}\n\n\
             #### Hypothesis ledger\n\n{words}\n\n\
             #### Proof roadmap\n\n{words}\n\n\
             #### Proof\n\n{words}\n\n\
             #### Boundary case\n\n{words}\n\n\
             #### Pedagogical prerequisites\n\n{words}\n\n\
             #### Lean correspondence\n\n[[formalization/lean/fake|fake]] type fingerprint axioms verification target {words}\n\n\
             #### Historical context\n\n{words}\n\n\
             #### ML analogy\n\n**Mathematical object / ML counterpart.** {words} **Exact transfer.** {words} **Non-transfer.** {words} **Diagnostic.** {words}\n\n"
        ));
    }
    assert!(
        !jin_complete_card_errors(&markdown, 30).is_empty(),
        "word-count validation accepted cards with no Jin mathematics"
    );
}

#[test]
fn jin_route_exercise_guard_rejects_name_only_exporter_mentions() {
    let declaration = "CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_01_solution";
    let exporter = format!("-- `{declaration}` appears only in a comment\n");
    assert!(
        !jin_solution_is_compiler_guarded(&exporter, declaration),
        "substring membership accepted a comment-only exporter mention"
    );
}

#[test]
fn jin_route_contract_probe_elaborates_exact_types_and_rejects_pseudotypes() {
    let receipt = jin_contract_probe_receipt();
    let errors = jin_contract_probe_errors(receipt);
    assert!(
        errors.is_empty(),
        "the canonical test-only Lean signatures failed compiler validation: {errors:#?}"
    );
    assert_eq!(
        receipt.as_array().expect("probe rows").len(),
        20,
        "the probe must contain nineteen canonical targets plus one negative mutation"
    );
    assert_eq!(
        JIN_EXERCISE_SPECS
            .iter()
            .map(|spec| spec.probe_declaration)
            .collect::<BTreeSet<_>>()
            .len(),
        18,
        "all eighteen exercises need distinct compiler probe declarations"
    );

    let mut omitted_hypothesis = receipt.clone();
    let mutation = jin_contract_probe_row(JIN_CFT_31_002_OMITTED_HYPOTHESIS_MUTATION).clone();
    let public = omitted_hypothesis
        .as_array_mut()
        .expect("probe rows")
        .iter_mut()
        .find(|row| row["name"] == JIN_CFT_31_002_PROBE_DECLARATION)
        .expect("CFT-31-002 probe row");
    public["normalized_type"] = mutation["normalized_type"].clone();
    public["type_sha256"] = mutation["type_sha256"].clone();
    assert!(
        jin_contract_probe_errors(&omitted_hypothesis)
            .iter()
            .any(|error| error.contains("differs from its canonical elaborated signature")),
        "a compiler-elaborated CFT-31-002 signature without invertibility was accepted"
    );

    let mut pseudo = receipt.clone();
    let exercise = pseudo
        .as_array_mut()
        .expect("probe rows")
        .iter_mut()
        .find(|row| row["name"] == JIN_EXERCISE_SPECS[0].probe_declaration)
        .expect("exercise probe row");
    exercise["normalized_type"] =
        json!("IsPositiveRealCompletion(B,T,H) → four display-only conjuncts");
    exercise["type_sha256"] = json!(receipt_type_hash(
        exercise["normalized_type"].as_str().expect("pseudo type")
    ));
    assert!(
        jin_contract_probe_errors(&pseudo)
            .iter()
            .any(|error| error.contains("differs from its canonical elaborated signature")),
        "a hand-injected display pseudotype was accepted as compiler metadata"
    );
}

#[test]
fn jin_route_completed_exercises_have_a_compiler_derived_transitive_dependency_audit() {
    let phase = jin_contract_phase(&jin_route_contract());
    let mut declarations = jin_exercise_dependency_receipt()
        .as_array()
        .expect("exercise dependency rows")
        .clone();
    for spec in JIN_EXERCISE_SPECS.iter().filter(|spec| {
        let chapter = spec.exercise_id[4..6]
            .parse::<u64>()
            .expect("exercise chapter");
        phase.completes(chapter)
    }) {
        let row = declarations
            .iter_mut()
            .find(|row| row["name"].as_str() == Some(spec.solution))
            .expect("live exercise dependency root");
        row["type_sha256"] = json!(jin_exercise_receipt_hash(spec));
    }
    let receipt = json!({
        "declarations": declarations
    });
    let exporter = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/ExportReceipt.lean"),
    )
    .expect("exercise exporter source");
    let errors = jin_exercise_receipt_errors(&receipt, &exporter, phase);
    assert!(
        errors.is_empty(),
        "live completed exercise dependency closure violates the contract: {errors:#?}"
    );
}

#[test]
fn jin_route_exercise_parent_guard_uses_phase_active_chapter31_names() {
    let phase = JinPhase::Chapter31Complete;
    let e02 = JIN_EXERCISE_SPECS
        .iter()
        .find(|spec| spec.exercise_id == "CFT-31-E02")
        .expect("CFT-31-E02 contract");
    let parent = jin_cft_spec(e02.parent);
    let active_public = jin_completed_public_declaration(parent);
    let active_bridge = jin_completed_provider_declaration(parent);
    let (_, exporter) = jin_exercise_receipt_fixture(phase);

    for forbidden in [active_public, active_bridge] {
        let mut direct = jin_exercise_receipt_fixture(phase).0;
        direct["declarations"]
            .as_array_mut()
            .expect("fixture declarations")
            .iter_mut()
            .find(|row| row["name"] == e02.solution)
            .expect("CFT-31-E02 row")["direct_dependencies"] = json!([forbidden]);
        assert!(
            jin_exercise_receipt_errors(&direct, &exporter, phase)
                .iter()
                .any(|error| error.contains("parent checkpoint/provider")),
            "CFT-31-E02 directly reused active parent `{forbidden}`"
        );

        let helper = format!(
            "CrouzeixTextbook.Part06.Exercises.Chapter31.hidden_{}",
            forbidden.rsplit('.').next().expect("name suffix")
        );
        let mut hidden = jin_exercise_receipt_fixture(phase).0;
        hidden["declarations"]
            .as_array_mut()
            .expect("fixture declarations")
            .iter_mut()
            .find(|row| row["name"] == e02.solution)
            .expect("CFT-31-E02 row")["direct_dependencies"] = json!([helper]);
        hidden["declarations"]
            .as_array_mut()
            .expect("fixture declarations")
            .push(json!({
                "name": helper,
                "kind": "theorem",
                "normalized_type": "Unit",
                "type_sha256": receipt_type_hash("Unit"),
                "direct_dependencies": [forbidden],
            }));
        assert!(
            jin_exercise_receipt_errors(&hidden, &exporter, phase)
                .iter()
                .any(|error| error.contains("parent checkpoint/provider")),
            "CFT-31-E02 hid active parent `{forbidden}` behind a helper"
        );
    }
}

#[test]
fn jin_route_required_exercise_dependencies_come_only_from_compiler_closure() {
    let phase = JinPhase::Chapter31Complete;
    let (mut receipt, exporter) = jin_exercise_receipt_fixture(phase);
    for exercise_id in ["CFT-31-E01", "CFT-31-E04", "CFT-31-E05"] {
        let spec = JIN_EXERCISE_SPECS
            .iter()
            .find(|spec| spec.exercise_id == exercise_id)
            .expect("required-dependency exercise");
        receipt["declarations"]
            .as_array_mut()
            .expect("fixture declarations")
            .iter_mut()
            .find(|row| row["name"] == spec.solution)
            .expect("exercise row")["direct_dependencies"] = json!([]);
    }
    let comment_only = format!(
        "{exporter}\n-- mulVec_completionSparseVector\n-- completionUnknownHalfContribution_eq_matrixPairing\n-- completionUnknownHalfContribution_eq_zero\n"
    );
    let errors = jin_exercise_receipt_errors(&receipt, &comment_only, phase);
    for exercise_id in ["CFT-31-E01", "CFT-31-E04", "CFT-31-E05"] {
        assert!(
            errors.iter().any(|error| error.contains(exercise_id)
                && error.contains("required compiler dependency")),
            "comment-only dependency names satisfied {exercise_id}"
        );
    }
}

#[test]
fn jin_route_cft_31_002_gate_freezes_the_reverse_cancellation_chain() {
    let spec = jin_cft_spec("CFT-31-002");
    assert_eq!(spec.statement_steps, &["K(0)=G+D(0)G", "D(0)G=0", "D(0)=0"]);
    assert_eq!(
        spec.proof_steps,
        &["G+D(0)G=G", "D(0)G=0", "D(0)=D(0)(GG⁻¹)=(D(0)G)G⁻¹=0",]
    );
}

#[test]
fn jin_route_exercise_contract_accepts_all_phase_fixtures() {
    for phase in JinPhase::ALL {
        let (receipt, exporter) = jin_exercise_receipt_fixture(phase);
        let errors = jin_exercise_receipt_errors(&receipt, &exporter, phase);
        assert!(
            errors.is_empty(),
            "valid exercise fixture for `{}` was rejected: {errors:#?}",
            phase.label()
        );
        let expected = JIN_EXERCISE_SPECS
            .iter()
            .filter(|spec| {
                let chapter = spec.exercise_id[4..6].parse::<u64>().expect("chapter");
                phase.completes(chapter)
            })
            .count();
        assert_eq!(
            jin_checked_exercise_names(&exporter)
                .expect("fixture exporter")
                .len(),
            expected,
            "phase `{}` must export exactly six solutions per completed chapter",
            phase.label()
        );
    }
}

#[test]
fn jin_route_exercise_contract_rejects_true_assumptions_aliases_and_terminals() {
    let phase = JinPhase::Wave2Complete;
    let (receipt, exporter) = jin_exercise_receipt_fixture(phase);
    let solution = JIN_EXERCISE_SPECS[0].solution;
    let mutate = |mut receipt: Value, field: &str, value: Value| {
        let row = receipt["declarations"]
            .as_array_mut()
            .expect("receipt declarations")
            .iter_mut()
            .find(|row| row["name"] == solution)
            .expect("exercise receipt row");
        row[field] = value;
        receipt
    };

    let true_receipt = mutate(receipt.clone(), "normalized_type", json!("True"));
    assert!(
        jin_exercise_receipt_errors(&true_receipt, &exporter, phase)
            .iter()
            .any(|error| error.contains("proves only True")),
        "exercise validator accepted `True := by trivial`"
    );

    let assumed = mutate(
        receipt.clone(),
        "normalized_type",
        json!("∀ (P : Prop), P → P"),
    );
    assert!(
        jin_exercise_receipt_errors(&assumed, &exporter, phase)
            .iter()
            .any(|error| error.contains("assumes its conclusion")),
        "exercise validator accepted assumed-conclusion plumbing"
    );

    let direct_alias = mutate(receipt.clone(), "kind", json!("direct-alias"));
    assert!(
        jin_exercise_receipt_errors(&direct_alias, &exporter, phase)
            .iter()
            .any(|error| error.contains("never direct/eta alias")),
        "exercise validator accepted a direct alias"
    );

    let eta_alias = mutate(
        direct_alias,
        "direct_dependencies",
        json!([jin_cft_spec("CFT-30-001").public_declaration]),
    );
    let errors = jin_exercise_receipt_errors(&eta_alias, &exporter, phase);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("parent checkpoint/provider")),
        "exercise validator accepted an eta alias of the parent checkpoint"
    );

    let terminal = mutate(
        receipt,
        "direct_dependencies",
        json!(["CrouzeixConjecture.jinFinalCrouzeixConjecture"]),
    );
    assert!(
        jin_exercise_receipt_errors(&terminal, &exporter, phase)
            .iter()
            .any(|error| error.contains("forbidden terminal dependency")),
        "exercise validator accepted a route terminal as a shortcut"
    );

    let mut extra_conclusion = jin_exercise_receipt_fixture(phase).0;
    let extra_type = format!(
        "{} → False",
        jin_contract_probe_row(JIN_EXERCISE_SPECS[0].probe_declaration)["normalized_type"]
            .as_str()
            .expect("exercise target type")
    );
    let extra_row = extra_conclusion["declarations"]
        .as_array_mut()
        .expect("receipt declarations")
        .iter_mut()
        .find(|row| row["name"] == solution)
        .expect("exercise receipt row");
    extra_row["normalized_type"] = json!(extra_type);
    extra_row["type_sha256"] = json!(receipt_type_hash(&extra_type));
    assert!(
        jin_exercise_receipt_errors(&extra_conclusion, &exporter, phase)
            .iter()
            .any(|error| error.contains("authoritative compiler-receipt type hash")),
        "token membership accepted a proposition with an extra conclusion"
    );

    let mut helper_terminal = jin_exercise_receipt_fixture(phase).0;
    let helper = "CrouzeixTextbook.Part06.Exercises.Chapter30.hidden_terminal_helper";
    helper_terminal["declarations"]
        .as_array_mut()
        .expect("fixture declarations")
        .iter_mut()
        .find(|row| row["name"] == solution)
        .expect("exercise row")["direct_dependencies"] = json!([helper]);
    helper_terminal["declarations"]
        .as_array_mut()
        .expect("fixture declarations")
        .push(json!({
            "name": helper,
            "kind": "theorem",
            "normalized_type": "Unit",
            "type_sha256": receipt_type_hash("Unit"),
            "direct_dependencies": ["CrouzeixConjecture.jinFinalCrouzeixConjecture"],
        }));
    assert!(
        jin_exercise_receipt_errors(&helper_terminal, &exporter, phase)
            .iter()
            .any(|error| error.contains("transitive forbidden terminal dependency")),
        "exercise validator accepted a helper that hides a terminal dependency"
    );

    let mut packaged_bridge = jin_exercise_receipt_fixture(phase).0;
    let e06 = JIN_EXERCISE_SPECS
        .iter()
        .find(|spec| spec.exercise_id == "CFT-30-E06")
        .expect("Chapter30 E06 contract");
    packaged_bridge["declarations"]
        .as_array_mut()
        .expect("fixture declarations")
        .iter_mut()
        .find(|row| row["name"] == e06.solution)
        .expect("Chapter30 E06 receipt row")["direct_dependencies"] =
        json!(["CrouzeixConjecture.completion_PRX_congruence_eq_gramian_expression"]);
    assert!(
        jin_exercise_receipt_errors(&packaged_bridge, &exporter, phase)
            .iter()
            .any(|error| error.contains("forbidden terminal dependency")),
        "CFT-30-E06 accepted the provider that packages the whole ordered identity"
    );

    let mut extra_namespace = jin_exercise_receipt_fixture(phase).0;
    extra_namespace["declarations"]
        .as_array_mut()
        .expect("fixture declarations")
        .push(json!({
            "name": "CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_07_solution",
            "kind": "theorem",
            "normalized_type": "Unit",
            "type_sha256": receipt_type_hash("Unit"),
            "direct_dependencies": [],
        }));
    assert!(
        jin_exercise_receipt_errors(&extra_namespace, &exporter, phase)
            .iter()
            .any(|error| error.contains("exactly six declarations")),
        "exercise validator accepted an extra declaration in the Chapter30 exercise namespace"
    );

    let mut private_namespace = jin_exercise_receipt_fixture(phase).0;
    private_namespace["declarations"]
        .as_array_mut()
        .expect("fixture declarations")
        .push(json!({
            "name": "_private.CrouzeixTextbook.Part06.Chapter32.0.CrouzeixTextbook.Part06.Exercises.Chapter32.hidden_helper",
            "kind": "theorem",
            "normalized_type": "Unit",
            "type_sha256": receipt_type_hash("Unit"),
            "direct_dependencies": [],
        }));
    assert!(
        jin_exercise_receipt_errors(&private_namespace, &exporter, phase)
            .iter()
            .any(|error| error.contains("exactly six declarations")),
        "exercise validator accepted a private helper in the Chapter32 exercise namespace"
    );
}

#[test]
fn jin_route_cft_32_001_completion_is_theorem_with_psd_eigenvector_order() {
    let spec = jin_cft_spec("CFT-32-001");
    assert_eq!(
        spec.kind, "theorem",
        "completed CFT-32-001 must be a theorem"
    );
    assert!(
        !spec
            .hypotheses
            .iter()
            .chain(spec.statement_steps)
            .chain(spec.proof_steps)
            .any(|step| step.contains('≻') || step.contains("IsPositiveDefinite")),
        "completion uses PSD, not strict positive definiteness"
    );
    assert_eq!(
        spec.proof_steps,
        &[
            "choose e with G₄e=pe",
            "assume p>2",
            "e*(G₂-G₄)e=0",
            "(G₂-G₄)e=0",
            "Y=(1/4)C*C",
            "G₂-G₄-Y⪰₀ and Y⪰₀",
            "e*Ye=0",
            "Ce=0",
            "G₄e=e",
            "G₄e=pe gives p=1, contradiction",
            "G₄≤2I",
            "G₄=I+Σ_{k≥1}4⁻ᵏ(Cᵏ)*Cᵏ≥I",
            "C*C≤4(G₄-I)≤4I",
            "‖C‖≤2",
            "‖SΛS⁻¹‖=‖C‖≤2",
        ]
    );
}

#[test]
fn jin_route_contract_pins_source_and_declaration_inventory() {
    let contract = jin_route_contract();
    for locator in [
        "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L319-L386",
        "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L387-L463",
        "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L464-L530",
        "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910",
        "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:Lean/CrouzeixConjecture/Statements.lean#L13-L18",
        "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:Lean/CrouzeixConjecture/Statements.lean#L21-L22",
        "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:Lean/CrouzeixConjecture/FinalTheorems.lean#L15-L23",
    ] {
        assert!(contract.contains(locator), "missing pinned Jin locator {locator}");
    }
    for spec in JIN_CFT_SPECS {
        for exact in [
            spec.item_id,
            spec.public_declaration,
            spec.provider_declaration,
            spec.type_sha256,
            jin_baseline_provider_hash(&spec),
            jin_completed_public_declaration(&spec),
            jin_completed_provider_declaration(&spec),
            jin_completed_mode(&spec),
        ] {
            assert!(
                contract.contains(exact),
                "contract does not bind {} to `{exact}`",
                spec.item_id
            );
        }
    }
    for spec in JIN_EXERCISE_SPECS {
        for exact in [spec.exercise_id, spec.parent, spec.probe_declaration] {
            assert!(
                contract.contains(exact),
                "contract does not bind {} to `{exact}`",
                spec.exercise_id
            );
        }
    }
}

#[test]
fn jin_route_lean_map_matches_coverage_and_a_fresh_compiler_receipt() {
    let phase = jin_contract_phase(&jin_route_contract());
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let coverage_rows = coverage["items"].as_array().expect("coverage rows");
    let receipt = fresh_textbook_receipt();
    assert_eq!(receipt["target"], "CrouzeixTextbook");
    assert_eq!(
        coverage_rows.len(),
        216,
        "coverage fixture must remain complete"
    );
    let mapping_errors = jin_lean_mapping_errors(&coverage, &receipt, phase);
    assert!(
        mapping_errors.is_empty(),
        "active Jin Lean map differs from the fresh compiler receipt: {mapping_errors:#?}"
    );
    let exporter = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/ExportReceipt.lean"),
    )
    .expect("textbook receipt exporter");
    let exercise_errors = jin_exercise_receipt_errors(&receipt, &exporter, phase);
    assert!(
        exercise_errors.is_empty(),
        "active Jin exercises differ from the exact compiler contract: {exercise_errors:#?}"
    );
}

#[test]
fn jin_route_lean_map_rejects_missing_provider_rows_and_accepts_completed_bridges() {
    let (baseline_coverage, baseline_exercises) = jin_baseline_fixture();
    assert!(jin_phase_metadata_errors(
        &baseline_coverage,
        &baseline_exercises,
        JinPhase::BaselineFrozen,
    )
    .is_empty());
    let mut baseline_receipt = jin_mapping_receipt_fixture(JinPhase::BaselineFrozen);
    assert!(
        jin_lean_mapping_errors(
            &baseline_coverage,
            &baseline_receipt,
            JinPhase::BaselineFrozen,
        )
        .is_empty(),
        "baseline mapping fixture must pass before provider mutation"
    );
    let missing = jin_cft_spec("CFT-30-003").provider_declaration;
    baseline_receipt["declarations"]
        .as_array_mut()
        .expect("mapping receipt declarations")
        .retain(|row| row["name"] != missing);
    assert!(
        jin_lean_mapping_errors(
            &baseline_coverage,
            &baseline_receipt,
            JinPhase::BaselineFrozen,
        )
        .iter()
        .any(|error| error.contains("exactly one provider row")),
        "optional provider lookup accepted a missing provider"
    );

    let (mut chapter31_coverage, mut chapter31_exercises) = jin_baseline_fixture();
    jin_promote_phase_fixture(
        &mut chapter31_coverage,
        &mut chapter31_exercises,
        JinPhase::Chapter31Complete,
    );
    let chapter31_receipt = jin_mapping_receipt_fixture(JinPhase::Chapter31Complete);
    assert!(
        jin_lean_mapping_errors(
            &chapter31_coverage,
            &chapter31_receipt,
            JinPhase::Chapter31Complete,
        )
        .is_empty(),
        "Chapter31 target mapping fixture must allow the local reverse-direction bridge"
    );
    let reverse = jin_cft_spec("CFT-31-002");
    assert_eq!(jin_completed_mode(reverse), "proved-here");
    assert_ne!(
        jin_completed_provider_declaration(reverse),
        reverse.provider_declaration,
        "the opposite-direction completionKernelModel_zero alias cannot satisfy CFT-31-002"
    );
}

#[test]
fn jin_route_lean_map_enforces_provider_declaration_kinds() {
    let (baseline_coverage, _) = jin_baseline_fixture();
    let mut wrong_baseline_theorem = jin_mapping_receipt_fixture(JinPhase::BaselineFrozen);
    wrong_baseline_theorem["declarations"]
        .as_array_mut()
        .expect("mapping declarations")
        .iter_mut()
        .find(|row| row["name"] == "CrouzeixConjecture.completionP_congruence_eq_gramian_four")
        .expect("baseline theorem provider")["kind"] = json!("definition");
    assert!(
        jin_lean_mapping_errors(
            &baseline_coverage,
            &wrong_baseline_theorem,
            JinPhase::BaselineFrozen,
        )
        .iter()
        .any(|error| error.contains("theorem-kind provider")),
        "pending theorem row accepted a definition-kind provider"
    );

    let mut wrong_baseline_alias = jin_mapping_receipt_fixture(JinPhase::BaselineFrozen);
    wrong_baseline_alias["declarations"]
        .as_array_mut()
        .expect("mapping declarations")
        .iter_mut()
        .find(|row| row["name"] == "CrouzeixConjecture.jinFinalCrouzeixConjecture")
        .expect("baseline alias provider")["kind"] = json!("theorem");
    assert!(
        jin_lean_mapping_errors(
            &baseline_coverage,
            &wrong_baseline_alias,
            JinPhase::BaselineFrozen,
        )
        .iter()
        .any(|error| error.contains("direct-alias-kind provider")),
        "pending alias row accepted a theorem-kind provider"
    );

    let (mut wave2_coverage, mut wave2_exercises) = jin_baseline_fixture();
    jin_promote_phase_fixture(
        &mut wave2_coverage,
        &mut wave2_exercises,
        JinPhase::Wave2Complete,
    );
    let wave2_receipt = jin_mapping_receipt_fixture(JinPhase::Wave2Complete);
    assert!(
        jin_lean_mapping_errors(&wave2_coverage, &wave2_receipt, JinPhase::Wave2Complete)
            .is_empty(),
        "wave-2 fixture must bind completed endpoints to theorem-kind providers"
    );
    let mut wrong_completed_theorem = wave2_receipt;
    wrong_completed_theorem["declarations"]
        .as_array_mut()
        .expect("mapping declarations")
        .iter_mut()
        .find(|row| {
            row["name"] == "CrouzeixConjecture.polynomialCrouzeixBound_of_holomorphicCrouzeixBound"
        })
        .expect("completed theorem provider")["kind"] = json!("direct-alias");
    assert!(
        jin_lean_mapping_errors(
            &wave2_coverage,
            &wrong_completed_theorem,
            JinPhase::Wave2Complete,
        )
        .iter()
        .any(|error| error.contains("theorem-kind provider")),
        "completed theorem row accepted a direct-alias-kind provider"
    );

    let (mut coverage, mut exercises) = jin_baseline_fixture();
    jin_promote_phase_fixture(&mut coverage, &mut exercises, JinPhase::Chapter30Complete);

    let mut wrong_definition_provider = jin_mapping_receipt_fixture(JinPhase::Chapter30Complete);
    let definition_provider = wrong_definition_provider["declarations"]
        .as_array_mut()
        .expect("mapping declarations")
        .iter_mut()
        .find(|row| row["name"] == "CrouzeixConjecture.IsPositiveRealCompletion")
        .expect("completion definition provider");
    definition_provider["kind"] = json!("theorem");
    assert!(
        jin_lean_mapping_errors(
            &coverage,
            &wrong_definition_provider,
            JinPhase::Chapter30Complete,
        )
        .iter()
        .any(|error| error.contains("definition-kind provider")),
        "definition row accepted a theorem-kind provider"
    );

    let mut wrong_theorem_provider = jin_mapping_receipt_fixture(JinPhase::Chapter30Complete);
    let theorem_provider = wrong_theorem_provider["declarations"]
        .as_array_mut()
        .expect("mapping declarations")
        .iter_mut()
        .find(|row| row["name"] == "CrouzeixConjecture.positiveRealCompletionStatement")
        .expect("proved completion theorem provider");
    theorem_provider["kind"] = json!("definition");
    assert!(
        jin_lean_mapping_errors(
            &coverage,
            &wrong_theorem_provider,
            JinPhase::Chapter30Complete,
        )
        .iter()
        .any(|error| error.contains("theorem-kind provider")),
        "theorem row accepted a definition-kind provider"
    );
}

#[test]
fn jin_route_pedagogical_branch_runs_from_29_through_30_31_32_without_ls() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let rows = array(&coverage, "items", "coverage");
    let graph = rows
        .iter()
        .map(|row| {
            let item_id = string(row, "item_id", "coverage row").to_owned();
            let prerequisites = array(row, "pedagogical_prerequisites", &item_id)
                .iter()
                .map(|value| value.as_str().expect("prerequisite string").to_owned())
                .collect::<BTreeSet<_>>();
            (item_id, prerequisites)
        })
        .collect::<BTreeMap<_, _>>();
    assert!(jin_graph_reaches(
        &graph,
        "CFT-30-001",
        "CFT-29-002",
        &mut BTreeSet::new()
    ));
    assert!(jin_graph_reaches(
        &graph,
        "CFT-30-006",
        "CFT-30-001",
        &mut BTreeSet::new()
    ));
    assert!(jin_graph_reaches(
        &graph,
        "CFT-31-006",
        "CFT-30-006",
        &mut BTreeSet::new()
    ));
    assert!(jin_graph_reaches(
        &graph,
        "CFT-32-006",
        "CFT-31-006",
        &mut BTreeSet::new()
    ));
    assert!(jin_graph_reaches(
        &graph,
        "CFT-32-006",
        "CFT-29-002",
        &mut BTreeSet::new()
    ));
    assert!(
        jin_routes_are_pedagogically_independent(&graph),
        "Jin teaching nodes must not cross into the LS branch"
    );
    assert!(
        (1..=6).any(|index| {
            let comparison = format!("CFT-35-{index:03}");
            jin_graph_reaches(&graph, &comparison, "CFT-32-006", &mut BTreeSet::new())
                && jin_graph_reaches(&graph, &comparison, "CFT-34-006", &mut BTreeSet::new())
        }),
        "the first cross-route comparison must be explicit in Chapter 35"
    );
}

#[test]
fn jin_route_pedagogical_independence_rejects_ls_to_jin_crossings() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let mut graph = array(&coverage, "items", "coverage")
        .iter()
        .map(|row| {
            let item_id = string(row, "item_id", "coverage row").to_owned();
            let prerequisites = array(row, "pedagogical_prerequisites", &item_id)
                .iter()
                .map(|value| value.as_str().expect("prerequisite string").to_owned())
                .collect::<BTreeSet<_>>();
            (item_id, prerequisites)
        })
        .collect::<BTreeMap<_, _>>();
    graph
        .get_mut("CFT-33-001")
        .expect("LS node")
        .insert("CFT-30-001".to_owned());
    assert!(
        !jin_routes_are_pedagogically_independent(&graph),
        "one-direction validation accepted an LS-to-Jin prerequisite"
    );
}

#[test]
fn jin_route_import_closure_uses_the_canonical_parser_and_rejects_ls_transitively() {
    let lakefile = fs::read_to_string(workspace_root().join("formalization/lean/lakefile.toml"))
        .expect("Lean lakefile");
    assert!(lakefile.contains("[[lean_lib]]\nname = \"CrouzeixJin\""));
    let mise = fs::read_to_string(workspace_root().join("mise.toml")).expect("mise config");
    let task = mise
        .split_once("[tasks.lean-crouzeix-jin]")
        .expect("Jin build task")
        .1
        .split_once("\n[")
        .expect("Jin build task boundary")
        .0;
    assert!(task.contains("scripts/check_lean_library.sh CrouzeixJin"));

    let actual = run_jin_closure(&workspace_root().join("formalization/lean"));
    assert!(
        actual.status.success(),
        "canonical Jin closure failed: {}",
        String::from_utf8_lossy(&actual.stderr)
    );
    let closure = String::from_utf8(actual.stdout).expect("UTF-8 closure");
    assert!(closure.lines().any(|module| module == "CrouzeixJin"));
    assert!(closure
        .lines()
        .any(|module| module == "Crouzeix.Jin.Terminal"));
    assert!(
        !closure.lines().any(|module| {
            module == "CrouzeixLoristSchwenninger"
                || module.starts_with("Crouzeix.LoristSchwenninger")
        }),
        "LS provider entered the Jin closure: {closure}"
    );

    for (label, root_import, neutral_import) in [
        (
            "direct",
            "import Crouzeix.LoristSchwenninger.Consequences\n",
            None,
        ),
        (
            "transitive",
            "import CrouzeixConjecture.NeutralDependency\n",
            Some("import Crouzeix.LoristSchwenninger.Consequences\n"),
        ),
    ] {
        let fixture = tempfile::tempdir().expect("Jin closure fixture");
        fs::write(fixture.path().join("CrouzeixJin.lean"), root_import)
            .expect("write fixture root");
        if let Some(import) = neutral_import {
            let path = fixture
                .path()
                .join("CrouzeixConjecture/NeutralDependency.lean");
            fs::create_dir_all(path.parent().expect("neutral parent"))
                .expect("create neutral parent");
            fs::write(path, import).expect("write neutral dependency");
        }
        let rejected = run_jin_closure(fixture.path());
        assert_eq!(
            rejected.status.code(),
            Some(7),
            "{label} fixture was accepted"
        );
        assert!(
            String::from_utf8_lossy(&rejected.stderr)
                .contains("rejected provider import Crouzeix.LoristSchwenninger.Consequences"),
            "{label} rejection did not identify the LS provider: {}",
            String::from_utf8_lossy(&rejected.stderr)
        );
    }
}

#[test]
fn jin_route_history_separates_source_evidence_review_and_local_formalization() {
    let locators = [(30, "#L319-L386"), (31, "#L346-L373"), (32, "#L464-L530")];
    for (chapter, source_lines) in locators {
        let markdown = jin_chapter_markdown(chapter);
        let visible = visible_markdown_text(&markdown);
        for required in [
            "SOURCE CLAIM.",
            "INSPECTED EVIDENCE.",
            "PEER-REVIEW / PUBLICATION STATE.",
            "HARP REPRODUCTION / FORMALIZATION STATE.",
            "JIN-565-V4-TEX",
            "source_manifest.tsv#L11",
            source_lines,
            "no peer-review, acceptance, or journal-publication receipt",
            "no priority claim",
        ] {
            assert!(
                visible.contains(required) || markdown.contains(required),
                "Chapter {chapter} omits cumulative history field `{required}`"
            );
        }
        assert!(
            markdown
                .contains("[[evidence/crouzeix_conjecture/source_manifest.tsv|JIN-565-V4-TEX]]"),
            "Chapter {chapter} omits the native evidence locator"
        );
        for index in 1..=6 {
            let item_id = format!("CFT-{chapter:02}-{index:03}");
            let card = jin_card(&markdown, &item_id, chapter).expect("Jin theorem card");
            let history = visible_markdown_text(
                jin_card_field(card, "Historical context").expect("historical context"),
            );
            for label in [
                "SOURCE CLAIM.",
                "INSPECTED EVIDENCE.",
                "PEER-REVIEW / PUBLICATION STATE.",
                "HARP REPRODUCTION / FORMALIZATION STATE.",
            ] {
                assert!(history.contains(label), "{item_id} omits `{label}`");
            }
        }
        assert!(
            !markdown.contains("<!--"),
            "Chapter {chapter} hides route claims in comments"
        );
    }
}

#[test]
fn jin_route_history_binds_each_cft_to_its_exact_source_ranges() {
    const SOURCE_PATH: &str = "the_numerical_range_is_a_2_spectral_set_v4.tex";
    let expected = [
        (30, 1, &["#L319-L386"] as &[_]),
        (30, 2, &["#L319-L386"] as &[_]),
        (30, 3, &["#L464-L498"] as &[_]),
        (30, 4, &["#L464-L498"] as &[_]),
        (30, 5, &["#L464-L498"] as &[_]),
        (30, 6, &["#L438-L460", "#L464-L498"] as &[_]),
        (31, 1, &["#L346-L373"] as &[_]),
        (31, 2, &["#L346-L373"] as &[_]),
        (31, 3, &["#L387-L463"] as &[_]),
        (31, 4, &["#L387-L463"] as &[_]),
        (31, 5, &["#L387-L463"] as &[_]),
        (31, 6, &["#L387-L463"] as &[_]),
        (32, 1, &["#L464-L530"] as &[_]),
        (32, 2, &["#L464-L530", "#L652-L774", "#L854-L910"] as &[_]),
        (32, 3, &["#L854-L910"] as &[_]),
        (32, 4, &["#L854-L910"] as &[_]),
        (32, 5, &["#L854-L910"] as &[_]),
        (32, 6, &["#L854-L910"] as &[_]),
    ];
    let all_ranges = [
        "#L319-L386",
        "#L346-L373",
        "#L387-L463",
        "#L438-L460",
        "#L464-L498",
        "#L464-L530",
        "#L652-L774",
        "#L854-L910",
    ];

    for (chapter, index, expected_ranges) in expected {
        let markdown = jin_chapter_markdown(chapter);
        let item_id = format!("CFT-{chapter:02}-{index:03}");
        let card = jin_card(&markdown, &item_id, chapter).expect("Jin theorem card");
        let history = visible_markdown_text(
            jin_card_field(card, "Historical context").expect("historical context"),
        );
        for source_range in expected_ranges {
            let locator = format!("{SOURCE_PATH}{source_range}");
            assert!(
                history.contains(&locator),
                "{item_id} omits exact source locator `{locator}`"
            );
        }
        for source_range in all_ranges {
            if !expected_ranges.contains(&source_range) {
                assert!(
                    !history.contains(source_range),
                    "{item_id} retains unrelated source range `{source_range}`"
                );
            }
        }
    }
}

#[test]
fn jin_route_running_nonnormal_family_has_exact_computable_specializations() {
    let chapter30 = visible_markdown_text(&jin_chapter_markdown(30));
    for required in [
        r"A_{λ,α}=\begin{pmatrix}λ&α\\0&-λ\end{pmatrix}",
        r"S_{λ,α}=\begin{pmatrix}1&-α/(2λ)\\0&1\end{pmatrix}",
        r"A_{λ,α}=S_{λ,α}\operatorname{diag}(λ,-λ)S_{λ,α}^{-1}",
        r"A_{1/2,1}=\begin{pmatrix}1/2&1\\0&-1/2\end{pmatrix}",
        r"G=S^{\mathrm H}S=\begin{pmatrix}1&-1\\-1&2\end{pmatrix}",
        "illustrative completion data",
        "does not prove that the analytic completion H exists",
    ] {
        assert!(chapter30.contains(required), "Chapter30 omits `{required}`");
    }

    let chapter31 = visible_markdown_text(&jin_chapter_markdown(31));
    for required in [
        "z_1=1/4, z_2=-1/4",
        r"P=\begin{pmatrix}16/15&-16/17\\-16/17&32/15\end{pmatrix}",
        r"R=\begin{pmatrix}8/7&-8/9\\-8/9&16/7\end{pmatrix}",
        r"X=\begin{pmatrix}8/105&8/153\\8/153&16/105\end{pmatrix}",
        r"v=-\frac1{255}\begin{pmatrix}304&64\\32&304\end{pmatrix}u",
        r"Gv+Pu=0",
        "specified diagonalizable member",
    ] {
        assert!(chapter31.contains(required), "Chapter31 omits `{required}`");
    }

    let chapter32 = visible_markdown_text(&jin_chapter_markdown(32));
    for required in [
        r"A_{0,2}=\begin{pmatrix}0&2\\0&0\end{pmatrix}",
        r"A_{0,2}^{\mathrm H}A_{0,2}=\operatorname{diag}(0,4)",
        r"\lVert A_{0,2}\rVert=2",
        r"W(A_{0,2})=\{z\in\mathbb C:|z|\le1\}",
        r"\sup_{z\in W(A_{0,2})}|z|=1",
        "equality in the factor-two estimate",
        "not diagonalizable",
        "does not itself satisfy the shared-basis completion hypothesis",
    ] {
        assert!(chapter32.contains(required), "Chapter32 omits `{required}`");
    }
}

#[test]
fn jin_route_ml_cards_are_four_field_specific_and_nonempirical() {
    let diagnostics = [
        (
            30,
            [
                "four-clause completion residual",
                "two diagonalization residuals",
                "denominator-four congruence residual",
                "denominator-two tail residual",
                "Gramian-difference eigenvalue",
                "source-to-Gramian congruence residual",
            ],
        ),
        (
            31,
            [
                "entrywise model residual",
                "right-cancellation estimate",
                "four ordered block residuals",
                "raw correction sum",
                "compensating-equation residual",
                "two smallest-eigenvalue comparisons",
            ],
        ),
        (
            32,
            [
                "three endpoint eigenvalue margins",
                "polynomial normalization ratio",
                "pole-distance and denominator-inverse residual",
                "rational normalization ratio",
                "ordered-limit residual",
                "calculus-interface residual",
            ],
        ),
    ];
    let mut bodies = BTreeSet::new();
    for (chapter, chapter_diagnostics) in diagnostics {
        let markdown = jin_chapter_markdown(chapter);
        for (offset, diagnostic) in chapter_diagnostics.iter().enumerate() {
            let item_id = format!("CFT-{chapter:02}-{:03}", offset + 1);
            let card = jin_card(&markdown, &item_id, chapter).expect("Jin theorem card");
            let ml =
                visible_markdown_text(jin_card_field(card, "ML analogy").expect("Jin ML analogy"));
            for required in [
                "Mathematical object / ML counterpart.",
                "Exact transfer.",
                "Non-transfer.",
                "Diagnostic.",
            ] {
                assert!(ml.contains(required), "{item_id} omits `{required}`");
            }
            assert!(ml.contains(diagnostic), "{item_id} omits `{diagnostic}`");
            let ml_lower = ml.to_lowercase();
            assert!(
                ml_lower.contains("noisy empirical") || ml_lower.contains("empirical covariance"),
                "{item_id} does not deny empirical-kernel transfer"
            );
            assert!(
                ml_lower.contains("floating-point") || ml_lower.contains("finite precision"),
                "{item_id} does not state the numerical PSD boundary"
            );
            assert!(bodies.insert(ml), "{item_id} duplicates another ML card");
        }
    }
    let route = visible_markdown_text(&format!(
        "{}\n{}\n{}",
        jin_chapter_markdown(30),
        jin_chapter_markdown(31),
        jin_chapter_markdown(32)
    ));
    assert!(route.contains("G is feature geometry"));
    assert!(route.contains("sampled-kernel PSD is a finite certificate"));
    assert!(route.contains("eigenvalue-sampling diagnostic"));
}

#[test]
fn jin_route_review_exposes_handoffs_independence_and_lean_audit() {
    let markdown = jin_chapter_markdown(32);
    let visible = visible_markdown_text(&markdown);
    for required in [
        "Jin route review",
        "Chapter 29 → Chapter 30 → Chapter 31 → Chapter 32",
        "independent of the Lorist-Schwenninger route",
        "Notation continuity",
        "Chapter 30 handoff",
        "Chapter 31 handoff",
        "Chapter 32 endpoint",
        "Formalization boundary",
        "18 theorem cards",
        "18 exercise solutions",
        "CrouzeixJin",
        "CrouzeixTextbook",
        "No theorem in Chapters 30-32 imports a Lorist-Schwenninger module",
    ] {
        assert!(
            visible.contains(required),
            "Jin route review omits `{required}`"
        );
    }
    for path in [
        "formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean",
        "formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean",
        "formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean",
        "formalization/lean/CrouzeixConjecture/CompletionStatement.lean",
        "formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean",
        "formalization/lean/CrouzeixConjecture/PositiveRealCompletion.lean",
    ] {
        assert!(
            markdown.contains(path),
            "Jin route review omits Lean link `{path}`"
        );
    }
}

#[test]
fn chapter_25_projection_cards_show_the_full_lax_style_argument() {
    let markdown = jin_chapter_markdown(25);
    let checks = [
        (
            "CFT-25-001",
            ["Closedness gives", "midpoint", "parallelogram identity"],
        ),
        ("CFT-25-002", ["0<t≤1", "Division by", "letting t↓0"]),
        (
            "CFT-25-003",
            ["both variational inequalities", "Cauchy", "cancelled"],
        ),
    ];
    for (item_id, steps) in checks {
        let card = jin_card(&markdown, item_id, 25).expect("Chapter 25 theorem card");
        let proof = jin_card_field(card, "Proof").expect("Chapter 25 proof field");
        let visible = visible_markdown_text(proof);
        for step in steps {
            assert!(
                visible.contains(step),
                "{item_id} proof omits the visible reconstruction step `{step}`"
            );
        }
    }
    let endpoint = jin_card(&markdown, "CFT-25-002", 25).expect("variational card");
    assert!(
        visible_markdown_text(endpoint).contains("endpoint derivative equals zero would be wrong")
    );
    let nonexpansive = jin_card(&markdown, "CFT-25-003", 25).expect("nonexpansive card");
    assert!(nonexpansive.contains("If `p=q`, the claim is immediate"));
    let existence = jin_card(&markdown, "CFT-25-001", 25).expect("existence card");
    assert!(existence.contains("\\tfrac14\\|p-q\\|^2"));
}

#[test]
fn chapter_25_projection_theorem_is_closed_convex_not_compact_only() {
    let source = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean"),
    )
    .expect("Chapter 25 Lean source");
    let theorem = source
        .split("theorem convex_projection")
        .nth(1)
        .expect("convex projection theorem")
        .split("theorem convex_projection_variational")
        .next()
        .expect("convex projection theorem boundary");
    assert!(theorem.contains("(hKclosed : IsClosed K)"));
    assert!(!theorem.contains("(hKcompact : IsCompact K)"));
    assert!(theorem.contains("exists_norm_eq_iInf_of_complete_convex"));

    let markdown = jin_chapter_markdown(25);
    let card = jin_card(&markdown, "CFT-25-001", 25).expect("projection card");
    for required in [
        "closed convex",
        "proper-space compact sublevel",
        "closedness keeps the limit in K",
        "existence and uniqueness use different hypotheses",
    ] {
        assert!(
            visible_markdown_text(card).contains(required),
            "closed-convex projection card omits `{required}`"
        );
    }
}

#[test]
fn chapter_25_radius_and_ellipse_derivations_are_explicit() {
    let markdown = jin_chapter_markdown(25);
    for required in [
        "\\varepsilon_k=\\operatorname{outerApproximationRadius}(k)=\\frac1{k+1}",
        "0<\\varepsilon_k\\le1",
        "\\varepsilon_k\\longrightarrow0",
        "\\lambda_{\\max}(\\operatorname{Re}(e^{-i\\theta}A_{\\lambda,\\alpha}))",
        "z=x(\\theta)+t\\,n(\\theta)",
        "t\\ge0",
        "semiaxes",
        "degenerate segment",
    ] {
        assert!(markdown.contains(required), "Chapter 25 omits `{required}`");
    }
    assert!(
        !markdown.contains("4Re("),
        "raw `Re` TeX survived rendering source"
    );
}

#[test]
fn chapter_25_radial_package_and_polygon_scope_are_field_by_field() {
    let markdown = jin_chapter_markdown(25);
    let radial = jin_card(&markdown, "CFT-25-006", 25).expect("radial card");
    for required in [
        "inverse gauge",
        "projection residual",
        "ν(θ)",
        "s(θ)=‖γ′(θ)‖",
        "supporting normal",
        "positive speed",
        "γ′(θ)=iν(θ)s(θ)",
        "provider fields",
    ] {
        assert!(
            visible_markdown_text(radial).contains(required),
            "radial card omits `{required}`"
        );
    }
    let exercise = markdown
        .split("### CFT-25-E05")
        .nth(1)
        .expect("E05")
        .split("### CFT-25-E06")
        .next()
        .expect("E05 boundary");
    for required in [
        "constant-speed polygonal parametrization",
        "degenerate C¹ parametrizations",
        "pause",
        "positive-speed regular radial C¹ package",
    ] {
        assert!(
            visible_markdown_text(exercise).contains(required),
            "E05 omits `{required}`"
        );
    }
}

#[test]
fn chapter_25_each_exercise_has_compiler_bound_lean_correspondence() {
    let markdown = jin_chapter_markdown(25);
    for index in 1..=6 {
        let start = format!("### CFT-25-E{index:02}");
        let exercise = markdown
            .split(&start)
            .nth(1)
            .unwrap_or_else(|| panic!("missing {start}"));
        let exercise = if index < 6 {
            let next = format!("### CFT-25-E{:02}", index + 1);
            exercise.split(&next).next().expect("exercise boundary")
        } else {
            exercise
                .split("## Synthesis and forward dependencies")
                .next()
                .expect("last exercise boundary")
        };
        for required in [
            "#### Lean correspondence",
            "Public declaration:",
            "Formal mode: `proved-here`.",
            "Provider boundary:",
            "Code: [Lean proof]",
            "Type SHA-256:",
            "Axioms: `Classical.choice, Quot.sound, propext`.",
            "Compiler receipt:",
            "Receipt identity:",
        ] {
            assert!(
                exercise.contains(required),
                "CFT-25-E{index:02} omits `{required}`"
            );
        }
    }
}

#[test]
fn chapter_25_outer_geometry_is_uniform_exactly_c1_and_advances_the_ellipse() {
    let markdown = jin_chapter_markdown(25);
    let visible = visible_markdown_text(&markdown);
    for required in [
        "A_{λ,α}",
        "Hausdorff error vanishes uniformly",
        "fixed compact neighborhood",
        "positively oriented radial C¹ boundary package",
        "never claims the raw body has C∞ boundary",
        "Mathematical object:",
        "ML counterpart:",
        "Exact transfer:",
        "Non-transfer:",
        "Diagnostic:",
    ] {
        assert!(visible.contains(required), "Chapter 25 omits `{required}`");
    }
    for equation in ["h_{W(A)}", "h_k(θ)=h_W(θ)+ε_k"] {
        assert!(
            markdown.contains(equation),
            "Chapter 25 omits the displayed running-example equation `{equation}`"
        );
    }
    assert!(
        visible.contains("stadium-like domain with the certified C¹ contour"),
        "the degenerate elliptical numerical range is not carried through the boundary construction"
    );
    for required in [
        "0\\le d_H(\\overline{\\Omega_k},K)\\le\\varepsilon_k",
        "\\operatorname{cthickening}(\\varepsilon_k,K)",
        "\\max_{\\overline{\\Omega_k}}|f|\\le\\max_K|f|+\\eta",
        "\\gamma(\\theta)=c+R(\\theta)e^{i\\theta}\\in\\operatorname{frontier}(\\Omega_k)",
    ] {
        assert!(markdown.contains(required), "Chapter 25 omits `{required}`");
    }
    assert!(!markdown.contains("\\gamma(\\mathbb R)=\\partial\\Omega_k"));
    for (item_id, mode, underlying) in [
        ("CFT-25-001", "proved-here", None),
        (
            "CFT-25-002",
            "reexported-proof",
            Some("CrouzeixConjecture.convexProjection_variational"),
        ),
        (
            "CFT-25-003",
            "reexported-proof",
            Some("CrouzeixConjecture.norm_convexProjection_sub_le"),
        ),
        ("CFT-25-004", "proved-here", None),
        ("CFT-25-005", "proved-here", None),
        (
            "CFT-25-006",
            "reexported-proof",
            Some("CrouzeixConjecture.canonicalParallelOrientedRadialBoundaryStatement"),
        ),
    ] {
        let card = jin_card(&markdown, item_id, 25).expect("Chapter 25 theorem card");
        let lean = jin_card_field(card, "Lean correspondence").expect("Lean correspondence");
        assert!(lean.contains(&format!("Formal mode: `{mode}`.")));
        if let Some(underlying) = underlying {
            assert!(lean.contains(&format!("Underlying declaration: `{underlying}`.")));
        }
    }
}

#[test]
fn chapter_25_lean_exercises_are_distinct_reconstructions_without_shortcuts() {
    let source = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean"),
    )
    .expect("Chapter 25 Lean source");
    for index in 1..=6 {
        assert!(
            source.contains(&format!("theorem exercise_{index:02}_solution")),
            "Chapter 25 omits exercise {index:02}"
        );
    }
    let exercise_02 = source
        .split("theorem exercise_02_solution")
        .nth(1)
        .expect("E02 body")
        .split("theorem exercise_03_solution")
        .next()
        .expect("E02 boundary");
    assert!(!exercise_02.contains("convexProjection_variational"));
    let exercise_04 = source
        .split("theorem exercise_04_solution")
        .nth(1)
        .expect("E04 body")
        .split("theorem exercise_05_solution")
        .next()
        .expect("E04 boundary");
    assert!(!exercise_04.contains("norm_convexProjection_sub_le"));
    let exercise_06 = source
        .split("theorem exercise_06_solution")
        .nth(1)
        .expect("E06 body");
    assert!(!exercise_06.contains("canonicalParallelOrientedRadialBoundaryStatement"));
    assert!(exercise_06.contains("parallelPositivePeriodicRadialData"));
    assert!(exercise_06.contains("orientedRadialConvexBoundary_thickening"));
}

#[test]
fn chapter_25_atlas_contains_the_equations_and_compiler_validated_links() {
    let corpus =
        fs::read_to_string(workspace_root().join("atlas/src/content/generated/corpus.json"))
            .expect("generated Atlas corpus");
    for required in [
        "CFT-25-001",
        "CFT-25-006",
        "Chapter25.lean#L14",
        "Chapter25.lean#L114",
        "4ffef37c950b39279846aeb028511a0d8f18a945e1d8888379d849d8096a6100",
        "e0c090a879dce8bda6dbcd5297ac2597904ac6646e0fff4ebdae2e33e4f846b9",
        "h_{W(A)}",
        "d_H",
    ] {
        assert!(corpus.contains(required), "Atlas corpus omits `{required}`");
    }

    for line in [14, 47, 56, 65, 71, 114, 122, 136, 204, 219, 243, 282] {
        let href =
            format!("../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L{line}");
        assert!(
            corpus.contains(&href),
            "rendered Chapter 25 omits exact Lean href `{href}`"
        );
    }
    for exercise in 1..=6 {
        assert!(
            corpus.contains(&format!("CFT-25-E{exercise:02}")),
            "rendered Chapter 25 omits exercise E{exercise:02}"
        );
    }
}

#[path = "support/crouzeix_textbook_wave3_contract.rs"]
mod wave3_contract;

#[path = "support/crouzeix_textbook_wave4_contract.rs"]
mod wave4_contract;

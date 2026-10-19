use super::contracts::{normalize_link_path, validate_local_links};
use super::registration::{KnowledgeRegistration, RegisteredDocument};
use super::render::render_markdown;
use super::*;
use std::fs;
use tempfile::TempDir;
fn fixture() -> TempDir {
    let target = workspace_root().join("target");
    fs::create_dir_all(&target).unwrap();
    let repo = tempfile::Builder::new()
        .prefix("knowledge-")
        .tempdir_in(target)
        .unwrap();
    fs::create_dir_all(repo.path().join("content")).unwrap();
    fs::create_dir_all(repo.path().join("content/generated")).unwrap();
    repo
}
fn write_complete_fixture(repo: &Path) {
    for &(id, path) in AUXILIARY_DOCUMENTS {
        fs::create_dir_all(repo.join(path).parent().unwrap()).unwrap();
        fs::write(
            repo.join(path),
            format!("# {}\n\nFixture.\n", id.replace('-', " ")),
        )
        .unwrap();
    }
    for &(_, _, path) in READER_ROUTES {
        if !repo.join(path).exists() {
            fs::create_dir_all(repo.join(path).parent().unwrap()).unwrap();
            fs::write(repo.join(path), "# Route\n\nFixture.\n").unwrap();
        }
    }
    let routes = "content/crouzeix_textbook/compatibility_routes.json";
    let bytes = fs::read(workspace_root().join(routes)).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    fs::create_dir_all(repo.join(routes).parent().unwrap()).unwrap();
    fs::write(repo.join(routes), bytes).unwrap();
    for route in value["routes"].as_array().unwrap() {
        let id = route["canonical_id"].as_str().unwrap();
        let path = route["canonical_path"].as_str().unwrap();
        fs::create_dir_all(repo.join(path).parent().unwrap()).unwrap();
        fs::write(repo.join(path), format!("---\nid: {id}\ntype: technical-document\nstatus: active\ntags: [crouzeix-textbook]\nconfidence: high\n---\n# {id}\n\n## Mechanism {{#fixture-mechanism}}\n\nFixture.\n")).unwrap();
    }
}
#[test]
fn registration_derives_reader_and_auxiliary_views() {
    let registration = KnowledgeRegistration::new(vec![
        RegisteredDocument::reader("reader", "Reader", "knowledge/reader.md"),
        RegisteredDocument::auxiliary("reference", "knowledge/reference.md"),
    ])
    .unwrap();

    assert_eq!(registration.reader_routes().count(), 1);
    assert_eq!(registration.auxiliary_documents().count(), 1);
}

#[test]
fn registration_rejects_duplicate_id_path_and_reader_label() {
    let duplicate_id = vec![
        RegisteredDocument::auxiliary("same", "knowledge/a.md"),
        RegisteredDocument::auxiliary("same", "knowledge/b.md"),
    ];
    assert!(KnowledgeRegistration::new(duplicate_id).is_err());

    let duplicate_path = vec![
        RegisteredDocument::auxiliary("a", "knowledge/same.md"),
        RegisteredDocument::auxiliary("b", "knowledge/same.md"),
    ];
    assert!(KnowledgeRegistration::new(duplicate_path).is_err());

    let duplicate_label = vec![
        RegisteredDocument::reader("a", "Same", "knowledge/a.md"),
        RegisteredDocument::reader("b", "Same", "knowledge/b.md"),
    ];
    assert!(KnowledgeRegistration::new(duplicate_label).is_err());
}

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
                b' ' | b'\n'
                    | b'\r'
                    | b'\t'
                    | b'\''
                    | b'"'
                    | b'('
                    | b'['
                    | b'{'
                    | b'='
                    | b'`'
                    | b'<'
            );
        let starts_component = source
            .as_bytes()
            .get(index + 1)
            .is_some_and(u8::is_ascii_alphanumeric);

        starts_token && starts_component
    }) || source.contains("~/")
        || source.contains("~\\")
        || source.contains("file://")
        || source
            .as_bytes()
            .windows(3)
            .enumerate()
            .any(|(index, window)| {
                let starts_token = index == 0
                    || matches!(
                        source.as_bytes()[index - 1],
                        b' ' | b'\n'
                            | b'\r'
                            | b'\t'
                            | b'\''
                            | b'"'
                            | b'('
                            | b'['
                            | b'{'
                            | b'='
                            | b'`'
                            | b'<'
                    );
                starts_token
                    && window[0].is_ascii_alphabetic()
                    && window[1] == b':'
                    && matches!(window[2], b'\\' | b'/')
            })
        || source.match_indices("\\\\").any(|(index, _)| {
            index == 0
                || matches!(
                    source.as_bytes()[index - 1],
                    b' ' | b'\n'
                        | b'\r'
                        | b'\t'
                        | b'\''
                        | b'"'
                        | b'('
                        | b'['
                        | b'{'
                        | b'='
                        | b'`'
                        | b'<'
                )
        })
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
        "See C:/input",
        "[input](C:/input)",
        "`C:/input`",
        "<C:/input>",
        "`/opt/harp/input`",
        "</opt/harp/input>",
        "\\\\server\\share",
        "See \\\\server\\share",
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
        "https://www.dpmms.cam.ac.uk/~wtg10/csineq.html",
        "[source](http://example.org/notes)",
        r"$(s,t)\begin{pmatrix}A&C\\C&B\end{pmatrix}(s,t)^T$",
    ] {
        assert!(
            !contains_nonportable_path_reference(source),
            "{source} is syntax, not a nonportable path reference"
        );
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
fn compiles_the_crouzeix_textbook_route_and_current_packet() {
    let corpus = compile(workspace_root()).unwrap();
    assert!(corpus.reader_routes.iter().any(|route| {
        route.route_id == "crouzeix-textbook"
            && route.label == "Crouzeix textbook"
            && route.canonical_markdown_path
                == "knowledge/crouzeix_textbook/crouzeix_textbook_index.md"
    }));

    let routes: serde_json::Value = serde_json::from_slice(
        &fs::read(workspace_root().join("content/crouzeix_textbook/compatibility_routes.json"))
            .unwrap(),
    )
    .unwrap();
    let routes = routes["routes"].as_array().unwrap();
    assert_eq!(routes.len(), 47);
    let documents = corpus
        .documents
        .iter()
        .filter(|document| {
            document
                .canonical_markdown_path
                .starts_with("knowledge/crouzeix_textbook/")
        })
        .map(|document| (document.canonical_markdown_path.as_str(), document))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(documents.len(), 47);

    let mut canonical_ids = BTreeSet::new();
    let mut legacy_ids = BTreeSet::new();
    let aliases = corpus
        .document_aliases
        .iter()
        .map(|alias| {
            (
                alias.alias_id.as_str(),
                alias.canonical_document_id.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(aliases.len(), 47);
    for route in routes {
        let canonical_id = route["canonical_id"].as_str().unwrap();
        let legacy_id = route["legacy_concept_id"].as_str().unwrap();
        let path = route["canonical_path"].as_str().unwrap();
        let document = documents
            .get(path)
            .unwrap_or_else(|| panic!("legacy route {legacy_id} lost canonical path {path}"));
        assert_eq!(document.concept_id, canonical_id);
        assert_eq!(document.metadata.id, canonical_id);
        assert_eq!(aliases.get(legacy_id), Some(&canonical_id));
        assert!(canonical_ids.insert(canonical_id));
        assert!(legacy_ids.insert(legacy_id));
    }
}

#[test]
fn compiles_harp_mathematics_supplements_without_new_chapters() {
    let corpus = compile(workspace_root()).unwrap();
    for (id, path) in [
        (
            "cft-harp-mathematical-audit",
            "knowledge/crouzeix_textbook/harp_mathematical_audit.md",
        ),
        (
            "cft-harp-finite-horizon-remainder",
            "knowledge/crouzeix_textbook/harp_finite_horizon_remainder.md",
        ),
    ] {
        let document = corpus
            .documents
            .iter()
            .find(|document| document.concept_id == id)
            .unwrap_or_else(|| panic!("missing canonical mathematics supplement {id}"));
        assert_eq!(document.canonical_markdown_path, path);
        assert_eq!(document.metadata.id, id);
    }
    let chapters = corpus
        .documents
        .iter()
        .filter(|document| document.concept_id.starts_with("cft-chapter-"))
        .count();
    assert_eq!(
        chapters, 36,
        "supplements must not become numbered chapters"
    );
}

#[test]
fn crouzeix_textbook_packet_rejects_symlinked_and_hardlinked_markdown() {
    let canonical =
        "knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations.md";

    let symlink_fixture = fixture();
    write_complete_fixture(symlink_fixture.path());
    let symlink_target = symlink_fixture.path().join("textbook-symlink-target.md");
    fs::write(&symlink_target, "# replacement\n").unwrap();
    fs::remove_file(symlink_fixture.path().join(canonical)).unwrap();
    std::os::unix::fs::symlink(&symlink_target, symlink_fixture.path().join(canonical)).unwrap();
    let error = compile(symlink_fixture.path()).expect_err("textbook symlink must fail closed");
    assert_eq!(error.code, "knowledge.crouzeix_textbook.registration");

    let hardlink_fixture = fixture();
    write_complete_fixture(hardlink_fixture.path());
    fs::hard_link(
        hardlink_fixture.path().join(canonical),
        hardlink_fixture.path().join("textbook-hardlink.md"),
    )
    .unwrap();
    let error = compile(hardlink_fixture.path()).expect_err("textbook hardlink must fail closed");
    assert_eq!(error.code, "knowledge.crouzeix_textbook.registration");
}

#[test]
fn crouzeix_textbook_markdown_enables_math_rendering() {
    let rendered = render_markdown(
        "Inline $Ax$ and display $$A^2x$$.",
        "knowledge/crouzeix_textbook/example.md",
    );
    assert!(rendered.contains("class=\"math math-inline\""));
    assert!(rendered.contains("class=\"math math-display\""));
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
    const MANIFEST_PATH: &str = "lab/math/lean/MathematicalFoundations/PublicTheorems.lean";
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
    let root_module =
        fs::read_to_string(workspace_root().join("lab/math/lean/MathematicalFoundations.lean"))
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
    let formalization_root = workspace_root().join("lab/math/lean/MathematicalFoundations");
    let lean_sources = formalization_lean_sources(&formalization_root);
    let manifest_suffix =
        Path::new("MathematicalFoundations").join(["Public", "Theorems.lean"].concat());
    assert!(
        lean_sources
            .iter()
            .any(|path| path.ends_with(&manifest_suffix)),
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
fn preserves_native_details_in_rendered_html() {
    let rendered = render_markdown(
        &format!(
            "# Test\n\n## Mechanism\n\nText.\n\n<details>\n{SOURCE_SUMMARY}\n\n- Source.\n\n</details>\n"
        ),
        "knowledge/rsi/chapters/test.md",
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
fn renders_chapter_33_formalization_wikilinks_from_the_vault_root() {
    let rendered = render::render_markdown_with_targets(
        "[[lab/math/lean/CrouzeixTextbook/Part06/Chapter33.lean|Chapter33.lean]]",
        "knowledge/crouzeix_textbook/part_06_constant_two_routes/33_lorist_schwenninger_perturbation_lemma.md",
        &BTreeMap::new(),
    );

    assert!(
        rendered.contains("href=\"../../lab/math/lean/CrouzeixTextbook/Part06/Chapter33.lean\""),
        "rendered Chapter 33 link: {rendered}"
    );
    assert!(
        !rendered.contains("knowledge/crouzeix_textbook/part_06_constant_two_routes/formalization")
    );
}

#[test]
fn wraps_tables_in_a_keyboard_accessible_scroll_region() {
    let rendered = render_markdown(
        "# Test\n\n| Field | Value |\n|---|---|\n| state | durable |\n",
        "knowledge/rsi/chapters/test.md",
    );

    assert!(rendered.contains(
        "<div class=\"canonical-table-scroll\" role=\"region\" aria-label=\"Scrollable data table\" tabindex=\"0\"><table>"
    ));
    assert!(rendered.contains("</table></div>"));
}

#[test]
fn math_and_heading_attributes_are_scoped_to_mathematics_packets() {
    let packet = render_markdown(
        "## Bound {#bound}\n\n$\\|T\\|\\le2$",
        "knowledge/crouzeix_conjecture/example.md",
    );
    assert!(packet.contains("id=\"bound\""));
    assert!(packet.contains("class=\"math math-inline\""));
    assert!(packet.contains("data-tex=\""));

    let foundations = render_markdown(
        "## Gradient {#gradient}\n\n$\\nabla f(x)$",
        "knowledge/mathematical_foundations/example.md",
    );
    assert!(foundations.contains("id=\"gradient\""));
    assert!(foundations.contains("class=\"math math-inline\""));

    let legacy = render_markdown(
        "Revenue moved from $0.0291 to $0.6371.",
        "knowledge/rsi/systems/aflow.md",
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

    let rendered = render_markdown("$x<&>\"'$", "knowledge/crouzeix_conjecture/example.md");
    assert!(rendered.contains("data-tex=\"x&lt;&amp;&gt;&quot;&#39;\""));
    assert!(rendered.contains(">x&lt;&amp;&gt;\"'</span>"));
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
    fs::create_dir_all(repo.path().join("lab/sicp")).unwrap();
    let source = repo.path().join("crates/harp/src/sources.rs");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(source, b"// materializer\n").unwrap();
    let repository = HeldDirectory::open(repo.path(), "test repository").unwrap();

    validate_local_links(
        "knowledge/rsi/sicp/course/capstone/guide.md",
        "[source](../../../../../evidence/sicp/sicp.pdf)\n[lab](../../../../../lab/sicp/)\n[materializer](../../../../../crates/harp/src/sources.rs)\n",
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
    let lean_target = repo.path().join("lab/math/lean/Example.lean");
    fs::create_dir_all(lean_target.parent().unwrap()).unwrap();
    fs::write(lean_target, "theorem example : True := by trivial\n").unwrap();
    let repository = HeldDirectory::open(repo.path(), "test repository").unwrap();

    validate_local_links(
        "knowledge/rsi/chapters/guide.md",
        "[[knowledge/rsi/target#Result boundary|Result]]",
        &repository,
    )
    .unwrap();

    validate_local_links(
        "knowledge/rsi/chapters/guide.md",
        "[[lab/math/lean/Example.lean|Lean proof]]",
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

#[cfg(unix)]
#[test]
fn build_rejects_a_symlinked_output_without_changing_its_target() {
    use std::os::unix::fs::symlink;

    let repo = fixture();
    write_complete_fixture(repo.path());
    let victim = repo.path().join("victim.json");
    fs::write(&victim, b"do not overwrite\n").unwrap();
    let output = PathBuf::from("content/generated/corpus.json");
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
    let generated = repo.path().join("content/generated");
    fs::remove_dir(&generated).unwrap();
    let outside = tempfile::tempdir().unwrap();
    symlink(outside.path(), &generated).unwrap();
    let output = PathBuf::from("content/generated/corpus.json");

    let error =
        crate::build_corpus(repo.path(), Some(&output), crate::BuildMode::Write).unwrap_err();

    assert_eq!(error.code(), "fs.symlink");
    assert!(fs::read_dir(outside.path()).unwrap().next().is_none());
}

#[test]
fn canonical_frontmatter_rejects_duplicate_and_nested_identity_fields() {
    for markdown in [
        "---\nid: first\nid: second\n---\n# Title\n",
        "---\n  id: nested\n---\n# Title\n",
        "---\nid: invalid identity\n---\n# Title\n",
        "---\ntitle: Missing identity\n---\n# Title\n",
    ] {
        let error = canonical_frontmatter_identity(markdown, "knowledge/example.md")
            .expect_err("canonical identity must be unambiguous");
        assert_eq!(error.code(), "knowledge.rsi.frontmatter");
    }
}

#[test]
fn textbook_alias_cannot_capture_a_surviving_document_identity() {
    let repo = fixture();
    write_complete_fixture(repo.path());
    let path = repo
        .path()
        .join("content/crouzeix_textbook/compatibility_routes.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["routes"][0]["legacy_concept_id"] = serde_json::json!("article-survey");
    fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(
        compile(repo.path()).unwrap_err().code(),
        "knowledge.rsi.document_identity"
    );
}

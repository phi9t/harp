use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::{check, FormalMode, TextbookContracts, TextbookDiagnostic};
use crate::fs::HeldDirectory;

const RECEIPT_SCHEMA: &str = "crouzeix-textbook-lean-receipt/v1";
const TOOLCHAIN: &str = "leanprover/lean4:v4.32.1";
const TARGET: &str = "CrouzeixTextbook";
const RECEIPT_PATH: &str = "Lean receipt";
const RECEIPT_CODE: &str = "crouzeix-textbook.lean-receipt";
const MAX_RECEIPT_BYTES: u64 = 4 * 1024 * 1024;
const MAX_DECLARATIONS: usize = 4096;
const MAX_RELATIONS: usize = 4096;
const MAX_STRING_BYTES: usize = 64 * 1024;
const MAX_LEAN_SOURCE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LeanReceipt {
    schema_version: String,
    toolchain: String,
    target: String,
    declarations: Vec<ReceiptDeclaration>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptDeclaration {
    name: String,
    kind: String,
    source_path: PathBuf,
    line: u32,
    column: u32,
    normalized_type: String,
    type_sha256: String,
    direct_dependencies: Vec<String>,
    axioms: Vec<String>,
}

struct CorrespondenceInput {
    modules: BTreeSet<String>,
    declarations: BTreeSet<String>,
}

struct ExpectedDeclaration<'a> {
    identity: &'a str,
    name: &'a str,
    source_path: &'a Path,
    line: u32,
    column: u32,
    type_sha256: &'a str,
    verification_target: &'a str,
    mode: ExpectedMode<'a>,
    axioms: &'a [String],
}

enum ExpectedMode<'a> {
    Theorem(FormalMode, Option<&'a str>),
    Exercise,
}

pub fn check_lean_receipt(
    repo_root: &Path,
    receipt_path: &Path,
) -> Result<(), Vec<TextbookDiagnostic>> {
    let contracts = check(repo_root)?;
    validate_receipt(&contracts, receipt_path).map(|_| ())
}

#[derive(Clone, Debug)]
pub(super) struct KernelDependency {
    pub(super) declaration: String,
    pub(super) direct_dependencies: Vec<String>,
}

#[derive(Clone, Debug)]
pub(super) struct CompilerReceiptIdentity {
    pub(super) sha256: String,
    pub(super) bytes: usize,
    pub(super) declaration_count: usize,
}

pub(super) struct ValidatedReceipt {
    pub(super) kernel_dependencies: Vec<KernelDependency>,
    pub(super) identity: CompilerReceiptIdentity,
}

pub fn generate_correspondence(repo_root: &Path) -> Result<String, Vec<TextbookDiagnostic>> {
    let contracts = check(repo_root)?;
    render_correspondence(repo_root, &contracts)
}

fn render_correspondence(
    repo_root: &Path,
    contracts: &TextbookContracts,
) -> Result<String, Vec<TextbookDiagnostic>> {
    let repository = HeldDirectory::open(repo_root, "Crouzeix textbook repository")
        .map_err(|error| vec![correspondence_diagnostic(None, "repository", error.message)])?;
    let mut modules = BTreeSet::new();
    let mut declarations = BTreeSet::new();
    for row in contracts.theorems() {
        if let Some(declaration) = &row.lean_declaration {
            if declaration.verification_target == TARGET {
                modules.insert(validated_source_module(
                    &repository,
                    &declaration.source_path,
                    &row.item_id,
                )?);
                declarations.insert(validated_declaration(
                    &declaration.name,
                    &row.item_id,
                    "lean_declaration.name",
                )?);
                if let Some(underlying) = declaration.underlying_declaration.as_deref() {
                    declarations.insert(validated_declaration(
                        underlying,
                        &row.item_id,
                        "lean_declaration.underlying_declaration",
                    )?);
                }
            }
        }
    }
    for exercise in contracts.exercises() {
        if let Some(solution) = &exercise.lean_solution {
            if solution.verification_target == TARGET {
                modules.insert(validated_source_module(
                    &repository,
                    &solution.source_path,
                    &exercise.exercise_id,
                )?);
                declarations.insert(validated_declaration(
                    &solution.declaration,
                    &exercise.exercise_id,
                    "lean_solution.declaration",
                )?);
            }
        }
    }

    Ok(render_correspondence_sets(CorrespondenceInput {
        modules,
        declarations,
    }))
}

fn validated_declaration(
    name: &str,
    identity: &str,
    field: &str,
) -> Result<String, Vec<TextbookDiagnostic>> {
    if is_safe_lean_name(name) {
        Ok(name.to_owned())
    } else {
        Err(vec![TextbookDiagnostic {
            code: "crouzeix-textbook.correspondence",
            identity: Some(identity.to_owned()),
            field: field.to_owned(),
            expected: "maintained qualified Lean declaration name".to_owned(),
            observed: name.to_owned(),
            path: None,
            line: None,
            column: None,
        }])
    }
}

fn validated_source_module(
    repository: &HeldDirectory,
    path: &Path,
    identity: &str,
) -> Result<String, Vec<TextbookDiagnostic>> {
    let module = module_from_path(path).ok_or_else(|| {
        vec![TextbookDiagnostic {
            code: "crouzeix-textbook.correspondence",
            identity: Some(identity.to_owned()),
            field: "source_path".to_owned(),
            expected: "relative maintained formalization/lean/**/*.lean module route".to_owned(),
            observed: path.display().to_string(),
            path: Some(path.to_path_buf()),
            line: None,
            column: None,
        }]
    })?;
    match repository.read_optional_regular_single_link_file_bounded(
        path,
        "Lean correspondence source",
        MAX_LEAN_SOURCE_BYTES,
    ) {
        Ok(Some(_)) => Ok(module),
        Ok(None) => Err(vec![correspondence_diagnostic(
            Some(path),
            "source_path",
            "Lean source is missing",
        )]),
        Err(error) => Err(vec![correspondence_diagnostic(
            Some(path),
            "source_path",
            error.message,
        )]),
    }
}

fn render_correspondence_sets(input: CorrespondenceInput) -> String {
    let mut output =
        String::from("-- GENERATED from the Crouzeix textbook contracts. DO NOT EDIT.\n\n");
    for module in input.modules {
        output.push_str("import ");
        output.push_str(&module);
        output.push('\n');
    }
    output.push_str("\nset_option linter.defProp false\n\n");
    for (index, declaration) in input.declarations.into_iter().enumerate() {
        output.push_str("#check ");
        output.push_str(&declaration);
        output.push('\n');
        output.push_str(&format!(
            "noncomputable def CrouzeixTextbook.ReceiptMarker.declaration{:04} :=\n  @",
            index + 1
        ));
        output.push_str(&declaration);
        output.push('\n');
    }
    output
}

fn correspondence_diagnostic(
    path: Option<&Path>,
    field: &str,
    observed: impl Into<String>,
) -> TextbookDiagnostic {
    TextbookDiagnostic {
        code: "crouzeix-textbook.correspondence",
        identity: None,
        field: field.to_owned(),
        expected: "validated v2 contracts".to_owned(),
        observed: observed.into(),
        path: path.map(Path::to_path_buf),
        line: None,
        column: None,
    }
}

fn module_from_path(path: &Path) -> Option<String> {
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(first)), Some(Component::Normal(second)))
            if first == "formalization" && second == "lean" => {}
        _ => return None,
    }
    let relative = components.collect::<PathBuf>();
    let relative = relative.to_str()?.strip_suffix(".lean")?;
    let module = relative.replace('/', ".");
    (!module.is_empty()
        && module.split('.').all(is_safe_lean_component)
        && is_maintained_name(&format!("{module}.__source"))
        && !module.starts_with("CrouzeixTextbook.ReceiptMarker."))
    .then_some(module)
}

pub(super) fn validate_receipt(
    contracts: &TextbookContracts,
    receipt_path: &Path,
) -> Result<ValidatedReceipt, Vec<TextbookDiagnostic>> {
    let bytes = read_receipt(receipt_path)?;
    let receipt: LeanReceipt = serde_json::from_slice(&bytes).map_err(|error| {
        vec![diagnostic(
            None,
            "$",
            "strict Lean receipt JSON",
            error.to_string(),
            Some(receipt_path.to_path_buf()),
        )]
    })?;
    let mut diagnostics = Vec::new();
    compare(
        receipt.schema_version.as_str(),
        RECEIPT_SCHEMA,
        None,
        "schema_version",
        receipt_path,
        &mut diagnostics,
    );
    compare(
        receipt.toolchain.as_str(),
        TOOLCHAIN,
        None,
        "toolchain",
        receipt_path,
        &mut diagnostics,
    );
    compare(
        receipt.target.as_str(),
        TARGET,
        None,
        "target",
        receipt_path,
        &mut diagnostics,
    );
    if receipt.declarations.len() > MAX_DECLARATIONS {
        diagnostics.push(diagnostic(
            None,
            "declarations",
            format!("at most {MAX_DECLARATIONS} rows"),
            receipt.declarations.len().to_string(),
            Some(receipt_path.to_path_buf()),
        ));
        return Err(diagnostics);
    }

    let mut rows = BTreeMap::new();
    let mut previous_name: Option<&str> = None;
    for row in &receipt.declarations {
        validate_row_shape(row, receipt_path, &mut diagnostics);
        if previous_name.is_some_and(|previous| previous >= row.name.as_str()) {
            diagnostics.push(diagnostic(
                Some(&row.name),
                "name",
                "strictly increasing receipt declaration names",
                previous_name.unwrap_or_default(),
                Some(receipt_path.to_path_buf()),
            ));
        }
        previous_name = Some(&row.name);
        if rows.insert(row.name.as_str(), row).is_some() {
            diagnostics.push(diagnostic(
                Some(&row.name),
                "name",
                "unique receipt declaration",
                "duplicate",
                Some(receipt_path.to_path_buf()),
            ));
        }
    }

    let expected = expected_declarations(contracts);
    let mut allowed_names = expected
        .iter()
        .map(|declaration| declaration.name)
        .collect::<BTreeSet<_>>();
    for theorem in contracts.theorems() {
        if let Some(underlying) = theorem
            .lean_declaration
            .as_ref()
            .and_then(|declaration| declaration.underlying_declaration.as_deref())
        {
            allowed_names.insert(underlying);
        }
    }
    for name in rows.keys() {
        if !allowed_names.contains(name) {
            diagnostics.push(diagnostic(
                Some(name),
                "name",
                "public theorem, exercise solution, or declared underlying proof",
                "unexpected receipt declaration",
                Some(receipt_path.to_path_buf()),
            ));
        }
    }
    for declaration in &expected {
        let Some(row) = rows.get(declaration.name).copied() else {
            diagnostics.push(diagnostic(
                Some(declaration.identity),
                "name",
                declaration.name,
                "missing",
                Some(receipt_path.to_path_buf()),
            ));
            continue;
        };
        validate_expected(declaration, row, &rows, receipt_path, &mut diagnostics);
    }
    validate_provider_dependencies(contracts, &rows, receipt_path, &mut diagnostics);
    validate_exercise_distinctness(contracts, &rows, receipt_path, &mut diagnostics);

    if diagnostics.is_empty() {
        let identity = CompilerReceiptIdentity {
            sha256: hex_sha256(&bytes),
            bytes: bytes.len(),
            declaration_count: receipt.declarations.len(),
        };
        Ok(ValidatedReceipt {
            kernel_dependencies: receipt
                .declarations
                .into_iter()
                .map(|row| KernelDependency {
                    declaration: row.name,
                    direct_dependencies: row.direct_dependencies,
                })
                .collect(),
            identity,
        })
    } else {
        Err(diagnostics)
    }
}

fn expected_declarations(contracts: &TextbookContracts) -> Vec<ExpectedDeclaration<'_>> {
    let mut expected = Vec::new();
    for theorem in contracts.theorems() {
        if let Some(declaration) = &theorem.lean_declaration {
            expected.push(ExpectedDeclaration {
                identity: &theorem.item_id,
                name: &declaration.name,
                source_path: &declaration.source_path,
                line: declaration.line,
                column: declaration.column,
                type_sha256: &declaration.type_sha256,
                verification_target: &declaration.verification_target,
                mode: ExpectedMode::Theorem(
                    theorem.formal_mode,
                    declaration.underlying_declaration.as_deref(),
                ),
                axioms: &declaration.axioms,
            });
        }
    }
    for exercise in contracts.exercises() {
        if let Some(solution) = &exercise.lean_solution {
            expected.push(ExpectedDeclaration {
                identity: &exercise.exercise_id,
                name: &solution.declaration,
                source_path: &solution.source_path,
                line: solution.line,
                column: solution.column,
                type_sha256: &solution.type_sha256,
                verification_target: &solution.verification_target,
                mode: ExpectedMode::Exercise,
                axioms: &solution.axioms,
            });
        }
    }
    expected
}

fn validate_expected(
    expected: &ExpectedDeclaration<'_>,
    row: &ReceiptDeclaration,
    rows: &BTreeMap<&str, &ReceiptDeclaration>,
    receipt_path: &Path,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    let identity = Some(expected.identity);
    compare_path(
        &row.source_path,
        expected.source_path,
        identity,
        "source_path",
        receipt_path,
        diagnostics,
    );
    compare(
        &row.line,
        &expected.line,
        identity,
        "line",
        receipt_path,
        diagnostics,
    );
    compare(
        &row.column,
        &expected.column,
        identity,
        "column",
        receipt_path,
        diagnostics,
    );
    compare(
        row.type_sha256.as_str(),
        expected.type_sha256,
        identity,
        "type_sha256",
        receipt_path,
        diagnostics,
    );
    compare(
        expected.verification_target,
        TARGET,
        identity,
        "verification_target",
        receipt_path,
        diagnostics,
    );
    compare(
        row.axioms.as_slice(),
        expected.axioms,
        identity,
        "axioms",
        receipt_path,
        diagnostics,
    );

    match expected.mode {
        ExpectedMode::Exercise => {
            compare(
                row.kind.as_str(),
                "theorem",
                identity,
                "kind",
                receipt_path,
                diagnostics,
            );
        }
        ExpectedMode::Theorem(FormalMode::ProvedHere, _) => {
            compare(
                row.kind.as_str(),
                "theorem",
                identity,
                "kind",
                receipt_path,
                diagnostics,
            );
        }
        ExpectedMode::Theorem(FormalMode::ReexportedProof, underlying) => {
            compare(
                row.kind.as_str(),
                "direct-alias",
                identity,
                "kind",
                receipt_path,
                diagnostics,
            );
            if let Some(underlying) = underlying {
                if row.direct_dependencies.as_slice() != [underlying] {
                    diagnostics.push(diagnostic(
                        identity,
                        "underlying_declaration",
                        format!("compiler-detected direct alias target {underlying}"),
                        format!("{:?}", row.direct_dependencies),
                        Some(receipt_path.to_path_buf()),
                    ));
                }
                match rows.get(underlying).copied() {
                    Some(underlying_row) if underlying_row.kind == "theorem" => {
                        if !alpha_equivalent_normalized_types(
                            &row.normalized_type,
                            &underlying_row.normalized_type,
                        ) {
                            diagnostics.push(diagnostic(
                                identity,
                                "underlying_declaration",
                                "exact normalized type equality with the direct alias target",
                                format!(
                                    "alias={:?}; underlying={:?}",
                                    row.normalized_type, underlying_row.normalized_type
                                ),
                                Some(receipt_path.to_path_buf()),
                            ));
                        }
                    }
                    Some(underlying_row) => diagnostics.push(diagnostic(
                        identity,
                        "underlying_declaration",
                        "theorem-kind underlying declaration",
                        &underlying_row.kind,
                        Some(receipt_path.to_path_buf()),
                    )),
                    None => diagnostics.push(diagnostic(
                        identity,
                        "underlying_declaration",
                        "underlying declaration present in compiled receipt",
                        "missing",
                        Some(receipt_path.to_path_buf()),
                    )),
                }
            }
        }
        ExpectedMode::Theorem(FormalMode::Definition, underlying) => {
            if let Some(underlying) = underlying {
                compare(
                    row.kind.as_str(),
                    "direct-alias",
                    identity,
                    "kind",
                    receipt_path,
                    diagnostics,
                );
                if row.direct_dependencies.as_slice() != [underlying] {
                    diagnostics.push(diagnostic(
                        identity,
                        "underlying_declaration",
                        format!("compiler-detected direct definition alias target {underlying}"),
                        format!("{:?}", row.direct_dependencies),
                        Some(receipt_path.to_path_buf()),
                    ));
                }
                match rows.get(underlying).copied() {
                    Some(underlying_row) if underlying_row.kind == "definition" => {
                        if !alpha_equivalent_normalized_types(
                            &row.normalized_type,
                            &underlying_row.normalized_type,
                        ) {
                            diagnostics.push(diagnostic(
                                identity,
                                "underlying_declaration",
                                "exact normalized type equality with the definition alias target",
                                format!(
                                    "alias={:?}; underlying={:?}",
                                    row.normalized_type, underlying_row.normalized_type
                                ),
                                Some(receipt_path.to_path_buf()),
                            ));
                        }
                    }
                    Some(underlying_row) => diagnostics.push(diagnostic(
                        identity,
                        "underlying_declaration",
                        "definition-kind underlying declaration",
                        &underlying_row.kind,
                        Some(receipt_path.to_path_buf()),
                    )),
                    None => diagnostics.push(diagnostic(
                        identity,
                        "underlying_declaration",
                        "underlying definition present in compiled receipt",
                        "missing",
                        Some(receipt_path.to_path_buf()),
                    )),
                }
            } else if !matches!(row.kind.as_str(), "definition" | "opaque") {
                diagnostics.push(diagnostic(
                    identity,
                    "kind",
                    "definition or opaque body",
                    &row.kind,
                    Some(receipt_path.to_path_buf()),
                ));
            }
        }
        ExpectedMode::Theorem(FormalMode::Checkpoint, _) => {
            if row.kind == "axiom" {
                diagnostics.push(diagnostic(
                    identity,
                    "kind",
                    "compiled non-axiom checkpoint",
                    &row.kind,
                    Some(receipt_path.to_path_buf()),
                ));
            }
        }
        ExpectedMode::Theorem(FormalMode::Informal, _) => {
            diagnostics.push(diagnostic(
                identity,
                "formal_mode",
                "informal rows without Lean declaration",
                expected.name,
                Some(receipt_path.to_path_buf()),
            ));
        }
    }

    for dependency in &row.direct_dependencies {
        if !is_maintained_name(dependency) {
            diagnostics.push(diagnostic(
                identity,
                "direct_dependencies",
                "maintained Crouzeix namespace dependency",
                dependency,
                Some(receipt_path.to_path_buf()),
            ));
        }
    }
}

fn validate_row_shape(
    row: &ReceiptDeclaration,
    receipt_path: &Path,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    for (field, value) in [
        ("name", row.name.as_str()),
        ("kind", row.kind.as_str()),
        ("normalized_type", row.normalized_type.as_str()),
        ("type_sha256", row.type_sha256.as_str()),
    ] {
        if value.len() > MAX_STRING_BYTES {
            diagnostics.push(diagnostic(
                Some(&row.name),
                field,
                format!("at most {MAX_STRING_BYTES} bytes"),
                value.len().to_string(),
                Some(receipt_path.to_path_buf()),
            ));
        }
    }
    if !matches!(
        row.kind.as_str(),
        "theorem"
            | "definition"
            | "opaque"
            | "axiom"
            | "direct-alias"
            | "quotient"
            | "inductive"
            | "constructor"
            | "recursor"
    ) {
        diagnostics.push(diagnostic(
            Some(&row.name),
            "kind",
            "known compiled declaration kind",
            &row.kind,
            Some(receipt_path.to_path_buf()),
        ));
    }
    let valid_hash = row.type_sha256.len() == 64
        && row
            .type_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid_hash {
        diagnostics.push(diagnostic(
            Some(&row.name),
            "type_sha256",
            "64 lowercase hexadecimal characters",
            &row.type_sha256,
            Some(receipt_path.to_path_buf()),
        ));
    }
    let computed = hex_sha256(row.normalized_type.as_bytes());
    compare(
        row.type_sha256.as_str(),
        computed.as_str(),
        Some(&row.name),
        "normalized_type",
        receipt_path,
        diagnostics,
    );
    if row.direct_dependencies.len() > MAX_RELATIONS || row.axioms.len() > MAX_RELATIONS {
        diagnostics.push(diagnostic(
            Some(&row.name),
            "direct_dependencies",
            format!("at most {MAX_RELATIONS} dependencies and axioms"),
            format!(
                "{} dependencies, {} axioms",
                row.direct_dependencies.len(),
                row.axioms.len()
            ),
            Some(receipt_path.to_path_buf()),
        ));
    }
    for (field, values) in [
        ("direct_dependencies", &row.direct_dependencies),
        ("axioms", &row.axioms),
    ] {
        if let Some(value) = values.iter().find(|value| value.len() > MAX_STRING_BYTES) {
            diagnostics.push(diagnostic(
                Some(&row.name),
                field,
                format!("names no larger than {MAX_STRING_BYTES} bytes"),
                value.len().to_string(),
                Some(receipt_path.to_path_buf()),
            ));
        }
    }
    if !is_safe_relative_lean_path(&row.source_path) {
        diagnostics.push(diagnostic(
            Some(&row.name),
            "source_path",
            "relative normalized formalization/lean/*.lean path",
            row.source_path.display().to_string(),
            Some(receipt_path.to_path_buf()),
        ));
    }
    if row.line == 0 || row.column == 0 {
        diagnostics.push(diagnostic(
            Some(&row.name),
            "line",
            "one-based source position",
            format!("{}:{}", row.line, row.column),
            Some(receipt_path.to_path_buf()),
        ));
    }
    let mut dependencies = row.direct_dependencies.clone();
    dependencies.sort();
    dependencies.dedup();
    if dependencies.len() != row.direct_dependencies.len()
        || dependencies != row.direct_dependencies
    {
        diagnostics.push(diagnostic(
            Some(&row.name),
            "direct_dependencies",
            "sorted unique names",
            format!("{:?}", row.direct_dependencies),
            Some(receipt_path.to_path_buf()),
        ));
    }
    let mut axioms = row.axioms.clone();
    axioms.sort();
    axioms.dedup();
    if axioms.len() != row.axioms.len() || axioms != row.axioms {
        diagnostics.push(diagnostic(
            Some(&row.name),
            "axioms",
            "sorted unique names",
            format!("{:?}", row.axioms),
            Some(receipt_path.to_path_buf()),
        ));
    }
}

fn validate_provider_dependencies(
    contracts: &TextbookContracts,
    rows: &BTreeMap<&str, &ReceiptDeclaration>,
    receipt_path: &Path,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    let mut owners = BTreeMap::new();
    for theorem in contracts.theorems() {
        let owner = match theorem.chapter {
            30..=32 => Some("jin"),
            33..=34 => Some("ls"),
            _ => None,
        };
        let (Some(owner), Some(declaration)) = (owner, theorem.lean_declaration.as_ref()) else {
            continue;
        };
        for name in std::iter::once(declaration.name.as_str())
            .chain(declaration.underlying_declaration.as_deref())
        {
            if let Some(previous) = owners.insert(name, owner) {
                if previous != owner {
                    diagnostics.push(diagnostic(
                        Some(name),
                        "provider",
                        "one Chapter 30-34 provider owner",
                        format!("{previous} and {owner}"),
                        Some(receipt_path.to_path_buf()),
                    ));
                }
            }
        }
    }
    for (&start, &owner) in &owners {
        let mut pending = vec![start];
        let mut visited = BTreeSet::new();
        while let Some(name) = pending.pop() {
            if !visited.insert(name) {
                continue;
            }
            if let Some(&dependency_owner) = owners.get(name) {
                if dependency_owner != owner {
                    diagnostics.push(diagnostic(
                        Some(start),
                        "direct_dependencies",
                        format!("receipt-visible {owner} dependency closure"),
                        format!("crosses into {dependency_owner} at {name}"),
                        Some(receipt_path.to_path_buf()),
                    ));
                    break;
                }
            }
            if let Some(row) = rows.get(name) {
                pending.extend(row.direct_dependencies.iter().map(String::as_str));
            }
        }
    }
}

fn validate_exercise_distinctness(
    contracts: &TextbookContracts,
    rows: &BTreeMap<&str, &ReceiptDeclaration>,
    receipt_path: &Path,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    let mut checkpoint_declarations = BTreeSet::new();
    let mut checkpoints_by_type = BTreeMap::<String, Vec<(&str, &str)>>::new();
    for theorem in contracts.theorems() {
        let Some(checkpoint) = &theorem.lean_declaration else {
            continue;
        };
        checkpoint_declarations.insert(checkpoint.name.as_str());
        if let Some(checkpoint_row) = rows.get(checkpoint.name.as_str()).copied() {
            checkpoints_by_type
                .entry(canonicalize_universe_binders(
                    &checkpoint_row.normalized_type,
                ))
                .or_default()
                .push((theorem.item_id.as_str(), checkpoint.name.as_str()));
        }
    }

    for exercise in contracts.exercises() {
        let Some(solution) = &exercise.lean_solution else {
            continue;
        };
        if checkpoint_declarations.contains(solution.declaration.as_str()) {
            diagnostics.push(diagnostic(
                Some(&exercise.exercise_id),
                "lean_solution.declaration",
                "declaration distinct from every studied theorem checkpoint",
                &solution.declaration,
                Some(receipt_path.to_path_buf()),
            ));
        }
        let Some(solution_row) = rows.get(solution.declaration.as_str()).copied() else {
            continue;
        };
        let canonical_type = canonicalize_universe_binders(&solution_row.normalized_type);
        if let Some(matches) = checkpoints_by_type.get(&canonical_type) {
            for (item_id, checkpoint_name) in matches {
                diagnostics.push(diagnostic(
                    Some(&exercise.exercise_id),
                    "lean_solution.type_sha256",
                    "statement fingerprint distinct from every public theorem checkpoint",
                    format!("matches {item_id} ({checkpoint_name})"),
                    Some(receipt_path.to_path_buf()),
                ));
            }
        }
    }
}

fn is_maintained_name(name: &str) -> bool {
    [
        "CrouzeixTextbook.",
        "CrouzeixConjecture.",
        "Crouzeix.Jin.",
        "CrouzeixJin.",
        "Crouzeix.LoristSchwenninger.",
        "CrouzeixLoristSchwenninger.",
        "Crouzeix.Harp.",
        "CrouzeixHarp.",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
}

/// Lean's pretty-printer preserves internal universe parameter names. Two
/// separately declared, definitionally identical polymorphic theorems can
/// therefore render with `u_1` and `u_2` even though the names differ only by
/// alpha-renaming. Canonicalize those binders by first occurrence before the
/// receipt validator compares normalized types.
fn alpha_equivalent_normalized_types(left: &str, right: &str) -> bool {
    canonicalize_universe_binders(left) == canonicalize_universe_binders(right)
}

fn canonicalize_universe_binders(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut binders = BTreeMap::<&str, usize>::new();
    let mut canonical = String::with_capacity(value.len());
    let mut index = 0;

    while index < bytes.len() {
        let starts_binder = bytes[index] == b'u'
            && bytes.get(index + 1) == Some(&b'_')
            && bytes.get(index + 2).is_some_and(u8::is_ascii_digit)
            && (index == 0
                || !bytes[index - 1].is_ascii_alphanumeric() && bytes[index - 1] != b'_');
        if starts_binder {
            let mut end = index + 2;
            while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                end += 1;
            }
            if bytes
                .get(end)
                .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
            {
                let binder = &value[index..end];
                let next = binders.len() + 1;
                let canonical_index = *binders.entry(binder).or_insert(next);
                canonical.push_str("u_");
                canonical.push_str(&canonical_index.to_string());
                index = end;
                continue;
            }
        }

        let character = value[index..]
            .chars()
            .next()
            .expect("index remains on a UTF-8 character boundary");
        canonical.push(character);
        index += character.len_utf8();
    }

    canonical
}

fn is_safe_lean_component(component: &str) -> bool {
    !component.is_empty()
        && component.len() <= 128
        && component.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric() || byte == b'_' || (index > 0 && byte == b'\'')
        })
        && !component.as_bytes()[0].is_ascii_digit()
}

fn is_safe_lean_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 512
        && is_maintained_name(name)
        && name.split('.').all(is_safe_lean_component)
}

fn is_safe_relative_lean_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .extension()
            .is_some_and(|extension| extension == "lean")
        && path.starts_with("formalization/lean")
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn read_receipt(path: &Path) -> Result<Vec<u8>, Vec<TextbookDiagnostic>> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        vec![diagnostic(
            None,
            "file",
            RECEIPT_PATH,
            error.to_string(),
            Some(path.to_path_buf()),
        )]
    })?;
    if !metadata.file_type().is_file()
        || metadata.nlink() != 1
        || metadata.len() > MAX_RECEIPT_BYTES
    {
        return Err(vec![diagnostic(
            None,
            "file",
            format!("single-linked regular file no larger than {MAX_RECEIPT_BYTES} bytes"),
            format!(
                "type={:?}, links={}, bytes={}",
                metadata.file_type(),
                metadata.nlink(),
                metadata.len()
            ),
            Some(path.to_path_buf()),
        )]);
    }
    let mut file: File = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| {
            vec![diagnostic(
                None,
                "file",
                RECEIPT_PATH,
                error.to_string(),
                Some(path.to_path_buf()),
            )]
        })?;
    let opened = file.metadata().map_err(|error| {
        vec![diagnostic(
            None,
            "file",
            RECEIPT_PATH,
            error.to_string(),
            Some(path.to_path_buf()),
        )]
    })?;
    if !opened.is_file()
        || opened.nlink() != 1
        || opened.len() > MAX_RECEIPT_BYTES
        || opened.dev() != metadata.dev()
        || opened.ino() != metadata.ino()
        || opened.len() != metadata.len()
    {
        return Err(vec![diagnostic(
            None,
            "file",
            "stable receipt inode",
            "changed during open",
            Some(path.to_path_buf()),
        )]);
    }
    let mut bytes = Vec::with_capacity(usize::try_from(opened.len()).unwrap_or(0));
    (&mut file)
        .take(MAX_RECEIPT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            vec![diagnostic(
                None,
                "file",
                RECEIPT_PATH,
                error.to_string(),
                Some(path.to_path_buf()),
            )]
        })?;
    if bytes.len() as u64 > MAX_RECEIPT_BYTES {
        return Err(vec![diagnostic(
            None,
            "file",
            format!("at most {MAX_RECEIPT_BYTES} bytes"),
            bytes.len().to_string(),
            Some(path.to_path_buf()),
        )]);
    }
    let after = file.metadata().map_err(|error| {
        vec![diagnostic(
            None,
            "file",
            RECEIPT_PATH,
            error.to_string(),
            Some(path.to_path_buf()),
        )]
    })?;
    if !after.is_file()
        || after.nlink() != 1
        || after.dev() != opened.dev()
        || after.ino() != opened.ino()
        || after.len() != opened.len()
        || bytes.len() as u64 != opened.len()
    {
        return Err(vec![diagnostic(
            None,
            "file",
            "stable single-linked receipt descriptor and exact bytes",
            "changed while reading",
            Some(path.to_path_buf()),
        )]);
    }
    Ok(bytes)
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn compare<T: std::fmt::Debug + PartialEq + ?Sized>(
    observed: &T,
    expected: &T,
    identity: Option<&str>,
    field: &str,
    path: &Path,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    if observed != expected {
        diagnostics.push(diagnostic(
            identity,
            field,
            format!("{expected:?}"),
            format!("{observed:?}"),
            Some(path.to_path_buf()),
        ));
    }
}

fn compare_path(
    observed: &Path,
    expected: &Path,
    identity: Option<&str>,
    field: &str,
    path: &Path,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    if observed != expected {
        diagnostics.push(diagnostic(
            identity,
            field,
            expected.display().to_string(),
            observed.display().to_string(),
            Some(path.to_path_buf()),
        ));
    }
}

fn diagnostic(
    identity: Option<&str>,
    field: impl Into<String>,
    expected: impl Into<String>,
    observed: impl Into<String>,
    path: Option<PathBuf>,
) -> TextbookDiagnostic {
    TextbookDiagnostic {
        code: RECEIPT_CODE,
        identity: identity.map(str::to_owned),
        field: field.into(),
        expected: expected.into(),
        observed: observed.into(),
        path,
        line: None,
        column: None,
    }
}

#[cfg(test)]
mod tests {
    use super::alpha_equivalent_normalized_types;

    #[test]
    fn normalized_types_compare_universe_binders_up_to_alpha_renaming() {
        assert!(alpha_equivalent_normalized_types(
            "forall (H : Type u_1), H -> H",
            "forall (H : Type u_2), H -> H",
        ));
        assert!(alpha_equivalent_normalized_types(
            "forall (A : Type u_7) (B : Type u_3), A -> B",
            "forall (A : Type u_2) (B : Type u_9), A -> B",
        ));
        assert!(!alpha_equivalent_normalized_types(
            "forall (A : Type u_7) (B : Type u_7), A -> B",
            "forall (A : Type u_2) (B : Type u_9), A -> B",
        ));
    }
}

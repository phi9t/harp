use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::lean::{self, CompilerReceiptIdentity, KernelDependency};
use super::markdown::{self, CompatibilityRoute};
use super::{check, LeanCorrespondenceStatus, TextbookContracts, TextbookDiagnostic};
use crate::fs::descriptor::{AnchoredDirectory, FilePolicy, HeldFileLock};
use crate::fs::{
    FileSnapshot, HeldDirectory, PublicFileMutation, PublicMutationEvent, PublicMutationOperation,
    PublicMutationPhase,
};
use crate::AppError;

const PUBLICATION_ROOT: &str = "atlas/src/content/generated/crouzeix_textbook/publication";
const GENERATIONS_ROOT: &str =
    "atlas/src/content/generated/crouzeix_textbook/publication/generations";
const CURRENT_PATH: &str = "atlas/src/content/generated/crouzeix_textbook/publication/current.json";
#[cfg(test)]
const LOCK_PATH: &str =
    "atlas/src/content/generated/crouzeix_textbook/publication/publication.lock";
const CURRENT_NAME: &str = "current.json";
const LOCK_NAME: &str = "publication.lock";
const MANIFEST_SCHEMA: &str = "crouzeix-textbook-publication/v1";
const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
const MAX_MANIFEST_BYTES: usize = 64 * 1024;
const PUBLICATION_DIRECTORY_MODE: u32 = 0o755;
const GENERATION_DIRECTORY_MODE: u32 = 0o755;
const LEDGER_FILE_MODE: u32 = 0o444;
const POINTER_FILE_MODE: u32 = 0o600;
const OUTPUT_NAMES: [&str; 6] = [
    "compatibility_ledger.md",
    "exercise_ledger.md",
    "kernel_dependency_ledger.md",
    "pedagogical_dependency_ledger.md",
    "status_ledger.md",
    "theorem_coverage_ledger.md",
];

#[derive(Clone, Debug, Eq, PartialEq)]
enum PublicationEvent {
    LedgerFileSync(String),
    GenerationDirectorySync,
    GenerationRename,
    GenerationPostRenameReopen,
    GenerationsParentSync,
    CreationCleanupSync,
    StagedGenerationCleanupSync,
    PointerFileSync,
    PointerRename,
    PointerParentSync,
    PointerCleanupSync,
    PointerRollbackSync(PublicMutationOperation),
    PointerExchangeValidation,
    PointerInstallValidation,
    Readback,
}

#[cfg(test)]
thread_local! {
    static PUBLICATION_TRACE: std::cell::RefCell<Vec<PublicationEvent>> = const {
        std::cell::RefCell::new(Vec::new())
    };
    static PUBLICATION_FAULTS: std::cell::RefCell<Vec<PublicationEvent>> = const {
        std::cell::RefCell::new(Vec::new())
    };
}

fn before_publication_event(event: &PublicationEvent) -> Result<(), AppError> {
    #[cfg(test)]
    {
        let injected = PUBLICATION_FAULTS.with(|faults| {
            let mut faults = faults.borrow_mut();
            if faults.first() == Some(event) {
                faults.remove(0);
                true
            } else {
                false
            }
        });
        if injected {
            return Err(AppError::external(
                "crouzeix-textbook.publication.sync",
                format!("injected publication fault at {event:?}"),
            ));
        }
    }
    #[cfg(not(test))]
    let _ = event;
    Ok(())
}

fn record_publication_event(event: PublicationEvent) {
    #[cfg(test)]
    PUBLICATION_TRACE.with(|trace| trace.borrow_mut().push(event));
    #[cfg(not(test))]
    let _ = event;
}

fn publication_step<T>(
    event: PublicationEvent,
    operation: impl FnOnce() -> Result<T, AppError>,
) -> Result<T, AppError> {
    before_publication_event(&event)?;
    let value = operation()?;
    record_publication_event(event);
    Ok(value)
}

#[cfg(test)]
fn take_publication_trace() -> Vec<PublicationEvent> {
    PUBLICATION_TRACE.with(|trace| std::mem::take(&mut *trace.borrow_mut()))
}

#[cfg(test)]
fn with_publication_fault<T>(event: PublicationEvent, operation: impl FnOnce() -> T) -> T {
    with_publication_faults(vec![event], operation)
}

#[cfg(test)]
fn with_publication_faults<T>(events: Vec<PublicationEvent>, operation: impl FnOnce() -> T) -> T {
    PUBLICATION_FAULTS.with(|faults| {
        let previous = std::mem::replace(&mut *faults.borrow_mut(), events);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation));
        let remaining = std::mem::replace(&mut *faults.borrow_mut(), previous);
        let value = match result {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        };
        assert!(
            remaining.is_empty(),
            "requested publication fault sequence was not fully consumed"
        );
        value
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextbookPublishMode {
    Check,
    Write,
}

#[derive(Debug, Serialize)]
pub struct TextbookPublishResult {
    pub theorem_count: usize,
    pub exercise_count: usize,
    pub exact_theorem_correspondence_count: usize,
    pub checked_exercise_solution_count: usize,
    pub outputs: Vec<PathBuf>,
    pub matched: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PublicationManifest {
    schema_version: String,
    generation: String,
    outputs: Vec<ManifestOutput>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ManifestOutput {
    name: String,
    path: PathBuf,
    bytes: usize,
    sha256: String,
}

struct PreparedPublication {
    generation: String,
    files: BTreeMap<&'static str, Vec<u8>>,
    manifest: PublicationManifest,
    manifest_bytes: Vec<u8>,
}

#[derive(Debug)]
struct CurrentPublication {
    bytes: Option<Vec<u8>>,
    snapshot: Option<FileSnapshot>,
}

struct PublicationDirectories {
    publication: AnchoredDirectory,
    generations: Option<AnchoredDirectory>,
}

pub fn publish_crouzeix_textbook(
    repo_root: &Path,
    receipt_path: &Path,
    mode: TextbookPublishMode,
) -> Result<TextbookPublishResult, AppError> {
    let contracts = check(repo_root).map_err(validation_error)?;
    let validated_receipt =
        lean::validate_receipt(&contracts, receipt_path).map_err(validation_error)?;
    let repository = HeldDirectory::open(repo_root, "Crouzeix textbook repository")?;
    let compatibility = markdown::compatibility_routes(&repository).map_err(validation_error)?;
    let prepared = prepare(
        &contracts,
        &validated_receipt.kernel_dependencies,
        &validated_receipt.identity,
        &compatibility.routes,
    )?;
    validate_in_held_temporary_storage(&prepared.files)?;
    let matched = match mode {
        TextbookPublishMode::Check => {
            let Some(directories) = open_publication_directories(&repository, false)? else {
                return Ok(result(&contracts, &prepared, false));
            };
            let lock = acquire_publication_lock(&directories, false)?;
            let current = inspect_current_publication(&directories)?;
            validate_expected_generation(&directories, &prepared)?;
            if let Some(lock) = &lock {
                lock.verify("Crouzeix textbook publication lock")?;
            } else if current.bytes.is_some() {
                return Err(AppError::invalid_input(
                    "crouzeix-textbook.publication.lock",
                    "a published textbook generation requires its persistent lock",
                ));
            }
            current.bytes.as_deref() == Some(prepared.manifest_bytes.as_slice())
        }
        TextbookPublishMode::Write => {
            let directories =
                open_publication_directories(&repository, true)?.ok_or_else(|| {
                    AppError::external(
                        "crouzeix-textbook.publication.directory",
                        "publication directories disappeared during creation",
                    )
                })?;
            let lock = acquire_publication_lock(&directories, true)?.ok_or_else(|| {
                AppError::external(
                    "crouzeix-textbook.publication.lock",
                    "publication lock was not created",
                )
            })?;
            let current = inspect_current_publication(&directories)?;
            let matched = current.bytes.as_deref() == Some(prepared.manifest_bytes.as_slice());
            if !matched {
                if !validate_expected_generation(&directories, &prepared)? {
                    lock.verify("Crouzeix textbook publication lock")?;
                    install_generation(&directories, &prepared)?;
                    lock.verify("Crouzeix textbook publication lock")?;
                    validate_expected_generation(&directories, &prepared)?;
                }
                lock.verify("Crouzeix textbook publication lock")?;
                switch_and_validate(&repository, &directories, &prepared, &current, |_| Ok(()))?;
            }
            lock.verify("Crouzeix textbook publication lock")?;
            matched
        }
    };

    Ok(result(&contracts, &prepared, matched))
}

fn result(
    contracts: &TextbookContracts,
    prepared: &PreparedPublication,
    matched: bool,
) -> TextbookPublishResult {
    TextbookPublishResult {
        theorem_count: contracts.theorems().len(),
        exercise_count: contracts.exercises().len(),
        exact_theorem_correspondence_count: exact_theorem_correspondence_count(contracts),
        checked_exercise_solution_count: checked_exercise_solution_count(contracts),
        outputs: prepared
            .manifest
            .outputs
            .iter()
            .map(|output| output.path.clone())
            .collect(),
        matched,
    }
}

fn inspect_current_publication(
    directories: &PublicationDirectories,
) -> Result<CurrentPublication, AppError> {
    inspect_current_publication_with_hook(directories, || {})
}

fn inspect_current_publication_with_hook(
    directories: &PublicationDirectories,
    after_pointer_read: impl FnOnce(),
) -> Result<CurrentPublication, AppError> {
    let current = directories.publication.read_optional_bounded(
        &AnchoredDirectory::cstring(CURRENT_NAME, "publication pointer")?,
        "Crouzeix textbook publication pointer",
        MAX_MANIFEST_BYTES,
        FilePolicy::exact(effective_uid(), POINTER_FILE_MODE, 1),
    )?;
    after_pointer_read();
    if let Some((bytes, snapshot)) = &current {
        validate_current_generation(directories, bytes)?;
        verify_pointer_unchanged(directories, bytes, snapshot)?;
    }
    Ok(CurrentPublication {
        bytes: current.as_ref().map(|(bytes, _)| bytes.clone()),
        snapshot: current.map(|(_, snapshot)| snapshot),
    })
}

fn switch_and_validate(
    repository: &HeldDirectory,
    directories: &PublicationDirectories,
    prepared: &PreparedPublication,
    previous: &CurrentPublication,
    after_switch: impl FnOnce(&PreparedPublication) -> Result<(), AppError>,
) -> Result<(), AppError> {
    let installed = repository.compare_and_replace_public_regular_file_with_mode_and_events(
        PublicFileMutation {
            relative: Path::new(CURRENT_PATH),
            bytes: &prepared.manifest_bytes,
            expected: previous.snapshot.as_ref(),
            label: "Crouzeix textbook publication pointer",
            max_bytes: MAX_MANIFEST_BYTES,
            required_mode: Some(POINTER_FILE_MODE),
        },
        |event| {
            let PublicMutationEvent { operation, phase } = event;
            let publication = match operation {
                PublicMutationOperation::FileSync => PublicationEvent::PointerFileSync,
                PublicMutationOperation::Rename => PublicationEvent::PointerRename,
                PublicMutationOperation::ParentSync => PublicationEvent::PointerParentSync,
                PublicMutationOperation::CleanupSync => PublicationEvent::PointerCleanupSync,
                PublicMutationOperation::RollbackSync => {
                    PublicationEvent::PointerRollbackSync(PublicMutationOperation::RollbackSync)
                }
                PublicMutationOperation::ExchangeValidation => {
                    PublicationEvent::PointerExchangeValidation
                }
                PublicMutationOperation::InstallValidation => {
                    PublicationEvent::PointerInstallValidation
                }
            };
            match phase {
                PublicMutationPhase::Before => before_publication_event(&publication),
                PublicMutationPhase::After => {
                    record_publication_event(publication);
                    Ok(())
                }
            }
        },
    )?;
    let validation = after_switch(prepared).and_then(|()| {
        before_publication_event(&PublicationEvent::Readback)?;
        let (current, current_snapshot) = directories
            .publication
            .read_optional_bounded(
                &AnchoredDirectory::cstring(CURRENT_NAME, "publication pointer")?,
                "Crouzeix textbook publication pointer",
                MAX_MANIFEST_BYTES,
                FilePolicy::exact(effective_uid(), POINTER_FILE_MODE, 1),
            )?
            .ok_or_else(|| {
                AppError::invalid_input(
                    "crouzeix-textbook.publication.post-switch",
                    "publication pointer disappeared after the switch",
                )
            })?;
        if current != prepared.manifest_bytes || current_snapshot != installed {
            return Err(AppError::invalid_input(
                "crouzeix-textbook.publication.post-switch",
                "publication pointer changed after the switch",
            ));
        }
        validate_current_generation(directories, &current)?;
        verify_pointer_unchanged(directories, &current, &current_snapshot)?;
        record_publication_event(PublicationEvent::Readback);
        Ok(())
    });
    if let Err(validation_error) = validation {
        rollback_pointer(repository, previous, &installed, |event| {
            match event.phase {
                PublicMutationPhase::Before => before_publication_event(
                    &PublicationEvent::PointerRollbackSync(event.operation),
                ),
                PublicMutationPhase::After => {
                    record_publication_event(PublicationEvent::PointerRollbackSync(
                        event.operation,
                    ));
                    Ok(())
                }
            }
        })
        .map_err(|rollback| {
            AppError::external(
                "crouzeix-textbook.publication.rollback",
                format!(
                    "post-switch validation failed: {}; rollback failed: {}",
                    validation_error.message, rollback.message
                ),
            )
        })?;
        return Err(validation_error);
    }
    Ok(())
}

fn verify_pointer_unchanged(
    directories: &PublicationDirectories,
    expected_bytes: &[u8],
    expected_snapshot: &FileSnapshot,
) -> Result<(), AppError> {
    let observed = directories.publication.read_optional_bounded(
        &AnchoredDirectory::cstring(CURRENT_NAME, "publication pointer")?,
        "Crouzeix textbook publication pointer",
        MAX_MANIFEST_BYTES,
        FilePolicy::exact(effective_uid(), POINTER_FILE_MODE, 1),
    )?;
    match observed {
        Some((bytes, snapshot)) if bytes == expected_bytes && &snapshot == expected_snapshot => {
            Ok(())
        }
        _ => Err(AppError::invalid_input(
            "crouzeix-textbook.publication.concurrent-change",
            "publication pointer changed while its generation was validated",
        )),
    }
}

fn rollback_pointer(
    repository: &HeldDirectory,
    previous: &CurrentPublication,
    installed: &FileSnapshot,
    events: impl FnMut(PublicMutationEvent) -> Result<(), AppError>,
) -> Result<(), AppError> {
    if let Some(previous_bytes) = previous.bytes.as_deref() {
        repository
            .compare_and_replace_public_regular_file_with_mode_and_events(
                PublicFileMutation {
                    relative: Path::new(CURRENT_PATH),
                    bytes: previous_bytes,
                    expected: Some(installed),
                    label: "Crouzeix textbook publication pointer rollback",
                    max_bytes: MAX_MANIFEST_BYTES,
                    required_mode: Some(POINTER_FILE_MODE),
                },
                events,
            )
            .map(|_| ())
    } else {
        repository.compare_and_remove_public_regular_file_with_events(
            Path::new(CURRENT_PATH),
            installed,
            "Crouzeix textbook publication pointer rollback",
            MAX_MANIFEST_BYTES,
            events,
        )
    }
}

fn prepare(
    contracts: &TextbookContracts,
    kernel_dependencies: &[KernelDependency],
    receipt_identity: &CompilerReceiptIdentity,
    compatibility: &[CompatibilityRoute],
) -> Result<PreparedPublication, AppError> {
    let files = BTreeMap::from([
        (
            "compatibility_ledger.md",
            render_compatibility(compatibility).into_bytes(),
        ),
        (
            "exercise_ledger.md",
            render_exercises(contracts).into_bytes(),
        ),
        (
            "kernel_dependency_ledger.md",
            render_kernel_dependencies(kernel_dependencies).into_bytes(),
        ),
        (
            "pedagogical_dependency_ledger.md",
            render_pedagogical_dependencies(contracts).into_bytes(),
        ),
        (
            "status_ledger.md",
            render_status(contracts, receipt_identity).into_bytes(),
        ),
        (
            "theorem_coverage_ledger.md",
            render_theorems(contracts).into_bytes(),
        ),
    ]);
    prepare_files(files)
}

fn prepare_files(files: BTreeMap<&'static str, Vec<u8>>) -> Result<PreparedPublication, AppError> {
    validate_rendered_files(&files)?;
    let generation = generation_digest(&files);
    let outputs = files
        .iter()
        .map(|(name, bytes)| ManifestOutput {
            name: (*name).to_owned(),
            path: generation_output_path(&generation, name),
            bytes: bytes.len(),
            sha256: sha256(bytes),
        })
        .collect();
    let manifest = PublicationManifest {
        schema_version: MANIFEST_SCHEMA.to_owned(),
        generation: generation.clone(),
        outputs,
    };
    let mut manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(|error| {
        AppError::external(
            "crouzeix-textbook.publication.serialize",
            format!("could not serialize publication pointer: {error}"),
        )
    })?;
    manifest_bytes.push(b'\n');
    if manifest_bytes.len() > MAX_MANIFEST_BYTES {
        return Err(AppError::external(
            "crouzeix-textbook.publication.size",
            "publication pointer exceeds its bounded size",
        ));
    }
    Ok(PreparedPublication {
        generation,
        files,
        manifest,
        manifest_bytes,
    })
}

fn render_theorems(contracts: &TextbookContracts) -> String {
    let mut rows = contracts.theorems().iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| left.item_id.cmp(&right.item_id));
    let mut output = generated_heading("Theorem coverage ledger");
    output.push_str("| Item | Chapter | Kind | Publication | Prose proof | Lean | Mode | Declaration | Prose |\n");
    output.push_str("|---|---:|---|---|---|---|---|---|---|\n");
    for row in rows {
        let declaration = row
            .lean_declaration
            .as_ref()
            .map(|declaration| {
                format!(
                    "{} ({}:{}:{})",
                    declaration.name,
                    declaration.source_path.display(),
                    declaration.line,
                    declaration.column
                )
            })
            .unwrap_or_else(|| "—".to_owned());
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {}#{} |\n",
            cell(&row.item_id),
            row.chapter,
            cell(&row.kind),
            row.publication_status,
            row.prose_proof_status,
            row.lean_correspondence_status,
            row.formal_mode,
            cell(&declaration),
            cell(&row.prose_path.display().to_string()),
            cell(&row.anchor)
        ));
    }
    output
}

fn render_exercises(contracts: &TextbookContracts) -> String {
    let mut rows = contracts.exercises().iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| left.exercise_id.cmp(&right.exercise_id));
    let mut output = generated_heading("Exercise ledger");
    output.push_str(
        "| Exercise | Chapter | Kind | Difficulty | Skills | Starter | Checked solution |\n",
    );
    output.push_str("|---|---:|---|---|---|---|---|\n");
    for row in rows {
        let starter = row
            .starter
            .as_ref()
            .map(|starter| {
                format!(
                    "{}:{}:{}",
                    starter.source_path.display(),
                    starter.line,
                    starter.column
                )
            })
            .unwrap_or_else(|| "—".to_owned());
        let solution = row
            .lean_solution
            .as_ref()
            .map(|solution| {
                format!(
                    "{} ({}:{}:{})",
                    solution.declaration,
                    solution.source_path.display(),
                    solution.line,
                    solution.column
                )
            })
            .unwrap_or_else(|| "—".to_owned());
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            cell(&row.exercise_id),
            row.chapter,
            cell(&row.kind),
            cell(&row.difficulty),
            list_cell(&row.skills),
            cell(&starter),
            cell(&solution)
        ));
    }
    output
}

fn render_pedagogical_dependencies(contracts: &TextbookContracts) -> String {
    let mut rows = contracts.theorems().iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| left.item_id.cmp(&right.item_id));
    let mut output = generated_heading("Pedagogical dependency ledger");
    output.push_str("| Item | Pedagogical prerequisites |\n|---|---|\n");
    for row in rows {
        output.push_str(&format!(
            "| {} | {} |\n",
            cell(&row.item_id),
            list_cell(&row.pedagogical_prerequisites)
        ));
    }
    output
}

fn render_kernel_dependencies(rows: &[KernelDependency]) -> String {
    let mut rows = rows.iter().collect::<Vec<_>>();
    rows.sort_by(|left, right| left.declaration.cmp(&right.declaration));
    let mut output = generated_heading("Kernel dependency ledger");
    output.push_str("| Lean declaration | Direct kernel dependencies |\n|---|---|\n");
    for row in rows {
        output.push_str(&format!(
            "| {} | {} |\n",
            cell(&row.declaration),
            list_cell(&row.direct_dependencies)
        ));
    }
    output
}

fn render_status(
    contracts: &TextbookContracts,
    receipt_identity: &CompilerReceiptIdentity,
) -> String {
    let theorem_statuses = counted(
        contracts
            .theorems()
            .iter()
            .map(|row| row.lean_correspondence_status.to_string()),
    );
    let publication_statuses = counted(
        contracts
            .theorems()
            .iter()
            .map(|row| row.publication_status.to_string()),
    );
    let formal_modes = counted(
        contracts
            .theorems()
            .iter()
            .map(|row| row.formal_mode.to_string()),
    );
    let checked_solutions = contracts
        .exercises()
        .iter()
        .filter(|row| row.lean_solution.is_some())
        .count();
    let mut output = generated_heading("Publication status ledger");
    output.push_str(&format!(
        "- Theorems: {}\n- Exercises: {}\n- Exact theorem correspondences: {}\n- Checked Lean exercise solutions: {}\n",
        contracts.theorems().len(),
        contracts.exercises().len(),
        exact_theorem_correspondence_count(contracts),
        checked_exercise_solution_count(contracts)
    ));
    output.push_str(&format!(
        "- Compiler receipt SHA-256: `{}`\n- Compiler receipt bytes: `{}`\n- Compiler receipt declarations: `{}`\n",
        receipt_identity.sha256, receipt_identity.bytes, receipt_identity.declaration_count
    ));
    output.push_str(
        "\nThe compiler receipt identity hashes the exact validated receipt bytes. \
The six-ledger publication generation is a separate digest over the rendered \
ledger files and is recorded by `current.json`; it is not the compiler receipt identity.\n",
    );
    output.push_str("\n| Axis | Status | Count |\n|---|---|---:|\n");
    for (status, count) in publication_statuses {
        output.push_str(&format!(
            "| publication | {} | {} |\n",
            cell(&status),
            count
        ));
    }
    for (status, count) in theorem_statuses {
        output.push_str(&format!(
            "| Lean correspondence | {} | {} |\n",
            cell(&status),
            count
        ));
    }
    for (status, count) in formal_modes {
        output.push_str(&format!(
            "| Formal mode | {} | {} |\n",
            cell(&status),
            count
        ));
    }
    output.push_str(&format!(
        "| Exercise solution | checked | {checked_solutions} |\n"
    ));
    output.push_str(&format!(
        "| Exercise solution | incomplete | {} |\n",
        contracts.exercises().len() - checked_solutions
    ));
    output
}

fn render_compatibility(routes: &[CompatibilityRoute]) -> String {
    let mut routes = routes.iter().collect::<Vec<_>>();
    routes.sort_by(|left, right| {
        left.canonical_path
            .cmp(&right.canonical_path)
            .then_with(|| left.canonical_id.cmp(&right.canonical_id))
    });
    let mut output = generated_heading("Compatibility route ledger");
    output.push_str("| Canonical identity | Legacy identity | Canonical path |\n|---|---|---|\n");
    for route in routes {
        output.push_str(&format!(
            "| {} | {} | {} |\n",
            cell(&route.canonical_id),
            cell(&route.legacy_concept_id),
            cell(&route.canonical_path.display().to_string())
        ));
    }
    output
}

fn generated_heading(title: &str) -> String {
    format!("<!-- GENERATED by `harp crouzeix-textbook publish`. DO NOT EDIT. -->\n\n# {title}\n\n")
}

fn cell(value: &str) -> String {
    value.replace('|', "\\|").replace(['\r', '\n'], " ")
}

fn list_cell(values: &[String]) -> String {
    if values.is_empty() {
        "—".to_owned()
    } else {
        cell(&values.join(", "))
    }
}

fn counted(values: impl Iterator<Item = String>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_insert(0) += 1;
    }
    counts
}

fn exact_theorem_correspondence_count(contracts: &TextbookContracts) -> usize {
    contracts
        .theorems()
        .iter()
        .filter(|row| row.lean_correspondence_status == LeanCorrespondenceStatus::Exact)
        .count()
}

fn checked_exercise_solution_count(contracts: &TextbookContracts) -> usize {
    contracts
        .exercises()
        .iter()
        .filter(|row| row.lean_solution.is_some())
        .count()
}

fn validate_rendered_files(files: &BTreeMap<&'static str, Vec<u8>>) -> Result<(), AppError> {
    if files.len() != OUTPUT_NAMES.len()
        || files.keys().copied().collect::<Vec<_>>() != OUTPUT_NAMES
    {
        return Err(AppError::external(
            "crouzeix-textbook.publication.outputs",
            "prepared publication must contain the exact six sorted ledgers",
        ));
    }
    for (name, bytes) in files {
        if bytes.is_empty()
            || bytes.len() > MAX_OUTPUT_BYTES
            || bytes.last() != Some(&b'\n')
            || bytes.contains(&b'\0')
            || bytes.contains(&b'\r')
        {
            return Err(AppError::external(
                "crouzeix-textbook.publication.output",
                format!("{name} is not bounded canonical LF-terminated Markdown"),
            ));
        }
    }
    Ok(())
}

fn validate_in_held_temporary_storage(
    files: &BTreeMap<&'static str, Vec<u8>>,
) -> Result<(), AppError> {
    let temporary = tempfile::tempdir().map_err(|error| {
        AppError::io(
            "crouzeix-textbook.publication.temp",
            "create held publication preparation directory",
            error,
        )
    })?;
    let held = HeldDirectory::open(temporary.path(), "prepared textbook publication")?;
    let directory = held.anchored_directory("prepared textbook publication")?;
    for (name, bytes) in files {
        directory.create_file_noreplace(
            &AnchoredDirectory::cstring(name, "prepared textbook ledger")?,
            bytes,
            0o600,
            "prepared textbook ledger",
        )?;
    }
    for (name, expected) in files {
        let observed = directory
            .read_optional_bounded(
                &AnchoredDirectory::cstring(name, "prepared textbook ledger")?,
                "prepared textbook ledger",
                MAX_OUTPUT_BYTES,
                FilePolicy::exact(effective_uid(), 0o600, 1),
            )?
            .ok_or_else(|| {
                AppError::external(
                    "crouzeix-textbook.publication.prepare",
                    format!("prepared ledger {name} disappeared"),
                )
            })?
            .0;
        if observed != *expected {
            return Err(AppError::external(
                "crouzeix-textbook.publication.prepare",
                format!("prepared ledger {name} changed during validation"),
            ));
        }
    }
    Ok(())
}

fn validate_current_generation(
    directories: &PublicationDirectories,
    bytes: &[u8],
) -> Result<(), AppError> {
    let manifest: PublicationManifest = serde_json::from_slice(bytes).map_err(|error| {
        AppError::invalid_input(
            "crouzeix-textbook.publication.manifest",
            format!("current publication pointer is invalid: {error}"),
        )
    })?;
    validate_manifest(directories, &manifest).map(|_| ())
}

fn validate_manifest(
    directories: &PublicationDirectories,
    manifest: &PublicationManifest,
) -> Result<BTreeMap<&'static str, Vec<u8>>, AppError> {
    validate_manifest_with_hooks(directories, manifest, || {}, |_| {})
}

fn validate_manifest_with_hooks(
    directories: &PublicationDirectories,
    manifest: &PublicationManifest,
    after_generation_open: impl FnOnce(),
    mut after_ledger_open: impl FnMut(&str),
) -> Result<BTreeMap<&'static str, Vec<u8>>, AppError> {
    if manifest.schema_version != MANIFEST_SCHEMA || !is_digest(&manifest.generation) {
        return Err(AppError::invalid_input(
            "crouzeix-textbook.publication.manifest",
            "current publication pointer has an unsupported schema or generation",
        ));
    }
    if manifest.outputs.len() != OUTPUT_NAMES.len() {
        return Err(AppError::invalid_input(
            "crouzeix-textbook.publication.manifest",
            "current publication pointer must name exactly six ledgers",
        ));
    }
    if manifest
        .outputs
        .iter()
        .map(|output| output.name.as_str())
        .ne(OUTPUT_NAMES)
    {
        return Err(AppError::invalid_input(
            "crouzeix-textbook.publication.manifest",
            "current publication pointer must list ledgers in canonical sorted order",
        ));
    }
    let generations = directories.generations.as_ref().ok_or_else(|| {
        AppError::invalid_input(
            "crouzeix-textbook.publication.missing",
            "publication generations directory is missing",
        )
    })?;
    let generation_name = AnchoredDirectory::cstring(
        &manifest.generation,
        "Crouzeix textbook publication generation",
    )?;
    let generation = generations
        .open_optional_directory(&generation_name, "Crouzeix textbook publication generation")?
        .ok_or_else(|| {
            AppError::invalid_input(
                "crouzeix-textbook.publication.missing",
                "publication generation is missing",
            )
        })?;
    generation.verify_owner_mode(
        GENERATION_DIRECTORY_MODE,
        "Crouzeix textbook publication generation",
    )?;
    let generation_identity = generation.identity("Crouzeix textbook publication generation")?;
    after_generation_open();
    let observed_names = generation
        .entry_names_bounded(
            "Crouzeix textbook publication generation",
            OUTPUT_NAMES.len(),
        )
        .map_err(publication_generation_error)?;
    let expected_names = OUTPUT_NAMES
        .iter()
        .map(|name| AnchoredDirectory::cstring(name, "Crouzeix textbook ledger"))
        .collect::<Result<Vec<_>, _>>()?;
    if observed_names != expected_names {
        return Err(AppError::invalid_input(
            "crouzeix-textbook.publication.generation",
            "publication generation must contain exactly the six declared ledgers",
        ));
    }

    let mut names = BTreeSet::new();
    let mut generation_files = BTreeMap::new();
    let mut generation_snapshots = BTreeMap::new();
    for output in &manifest.outputs {
        if !OUTPUT_NAMES.contains(&output.name.as_str()) || !names.insert(output.name.as_str()) {
            return Err(AppError::invalid_input(
                "crouzeix-textbook.publication.manifest",
                "current publication pointer contains an unknown or duplicate ledger",
            ));
        }
        if output.path != generation_output_path(&manifest.generation, &output.name)
            || !is_digest(&output.sha256)
            || output.bytes > MAX_OUTPUT_BYTES
        {
            return Err(AppError::invalid_input(
                "crouzeix-textbook.publication.manifest",
                format!("current publication route for {} is invalid", output.name),
            ));
        }
        let output_name = AnchoredDirectory::cstring(&output.name, "published textbook ledger")?;
        let (observed, snapshot) = generation
            .read_optional_bounded_with_hook(
                &output_name,
                "published Crouzeix textbook ledger",
                MAX_OUTPUT_BYTES,
                FilePolicy::exact(effective_uid(), LEDGER_FILE_MODE, 1),
                || after_ledger_open(&output.name),
            )?
            .ok_or_else(|| {
                AppError::invalid_input(
                    "crouzeix-textbook.publication.missing",
                    format!("published ledger {} is missing", output.path.display()),
                )
            })?;
        if snapshot.length as usize != output.bytes
            || observed.len() != output.bytes
            || hex_digest(snapshot.digest) != output.sha256
        {
            return Err(AppError::invalid_input(
                "crouzeix-textbook.publication.digest",
                format!(
                    "published ledger {} does not match its pointer",
                    output.name
                ),
            ));
        }
        let name = OUTPUT_NAMES
            .iter()
            .copied()
            .find(|name| *name == output.name)
            .expect("allowlisted output name was checked above");
        generation_files.insert(name, observed);
        generation_snapshots.insert(name, snapshot);
    }
    if generation_digest(&generation_files) != manifest.generation {
        return Err(AppError::invalid_input(
            "crouzeix-textbook.publication.generation",
            "publication generation digest does not match the six ledger bytes",
        ));
    }
    let final_names = generation
        .entry_names_bounded(
            "Crouzeix textbook publication generation",
            OUTPUT_NAMES.len(),
        )
        .map_err(publication_generation_error)?;
    if final_names != expected_names {
        return Err(AppError::invalid_input(
            "crouzeix-textbook.publication.generation",
            "publication generation changed while it was validated",
        ));
    }
    for name in OUTPUT_NAMES {
        let output_name = AnchoredDirectory::cstring(name, "published textbook ledger")?;
        let (final_bytes, final_snapshot) = generation
            .read_optional_bounded(
                &output_name,
                "published Crouzeix textbook ledger",
                MAX_OUTPUT_BYTES,
                FilePolicy::exact(effective_uid(), LEDGER_FILE_MODE, 1),
            )?
            .ok_or_else(|| {
                AppError::invalid_input(
                    "crouzeix-textbook.publication.missing",
                    format!("published ledger {name} disappeared during validation"),
                )
            })?;
        if generation_snapshots.get(name) != Some(&final_snapshot)
            || generation_files.get(name) != Some(&final_bytes)
        {
            return Err(AppError::invalid_input(
                "crouzeix-textbook.publication.generation",
                format!("published ledger {name} changed during validation"),
            ));
        }
    }
    generation.verify_namespace("Crouzeix textbook publication generation")?;
    generations.verify_entry_identity(
        &generation_name,
        generation_identity,
        true,
        "Crouzeix textbook publication generation",
    )?;
    generations.verify_namespace("Crouzeix textbook generations parent")?;
    Ok(generation_files)
}

fn validate_expected_generation(
    directories: &PublicationDirectories,
    prepared: &PreparedPublication,
) -> Result<bool, AppError> {
    let Some(generations) = &directories.generations else {
        return Ok(false);
    };
    let generation_name = AnchoredDirectory::cstring(
        &prepared.generation,
        "expected Crouzeix textbook generation",
    )?;
    if generations
        .open_optional_directory(&generation_name, "expected Crouzeix textbook generation")?
        .is_none()
    {
        return Ok(false);
    }
    let observed = validate_manifest(directories, &prepared.manifest)?;
    for (name, expected) in &prepared.files {
        if observed.get(name) != Some(expected) {
            return Err(AppError::invalid_input(
                "crouzeix-textbook.publication.digest",
                format!("expected publication ledger {name} has conflicting bytes"),
            ));
        }
    }
    Ok(true)
}

fn install_generation(
    directories: &PublicationDirectories,
    prepared: &PreparedPublication,
) -> Result<(), AppError> {
    let generations = directories.generations.as_ref().ok_or_else(|| {
        AppError::external(
            "crouzeix-textbook.publication.directory",
            "publication generations directory is missing",
        )
    })?;
    let mut staged = generations.create_staged_directory_with_cleanup_sync_hook(
        ".prepared-generation",
        GENERATION_DIRECTORY_MODE,
        "prepared Crouzeix textbook generation",
        |before| {
            if before {
                before_publication_event(&PublicationEvent::CreationCleanupSync)
            } else {
                record_publication_event(PublicationEvent::CreationCleanupSync);
                Ok(())
            }
        },
    )?;
    let installation = (|| -> Result<(), AppError> {
        for (name, bytes) in &prepared.files {
            publication_step(PublicationEvent::LedgerFileSync((*name).to_owned()), || {
                staged.create_file_noreplace_with_cleanup_sync_hook(
                    &AnchoredDirectory::cstring(name, "prepared textbook ledger")?,
                    bytes,
                    LEDGER_FILE_MODE,
                    "prepared textbook ledger",
                    |before| {
                        if before {
                            before_publication_event(&PublicationEvent::CreationCleanupSync)
                        } else {
                            record_publication_event(PublicationEvent::CreationCleanupSync);
                            Ok(())
                        }
                    },
                )
            })?;
        }
        publication_step(PublicationEvent::GenerationDirectorySync, || {
            staged
                .directory()
                .sync("prepared Crouzeix textbook generation")
        })?;
        let destination = AnchoredDirectory::cstring(
            &prepared.generation,
            "prepared Crouzeix textbook generation",
        )?;
        let published = publication_step(PublicationEvent::GenerationRename, || {
            generations.publish_staged_directory_noreplace_with_hook(
                &mut staged,
                &destination,
                "prepared Crouzeix textbook generation",
                || before_publication_event(&PublicationEvent::GenerationPostRenameReopen),
            )
        })?;
        record_publication_event(PublicationEvent::GenerationPostRenameReopen);
        published.verify_owner_mode(
            GENERATION_DIRECTORY_MODE,
            "published Crouzeix textbook generation",
        )?;
        publication_step(PublicationEvent::GenerationsParentSync, || {
            generations.sync("Crouzeix textbook generations parent")
        })?;
        Ok(())
    })();
    match installation {
        Ok(()) => Ok(()),
        Err(error) => {
            staged.cleanup_with_parent_sync(
                "prepared Crouzeix textbook generation cleanup",
                |before| {
                    if before {
                        before_publication_event(&PublicationEvent::StagedGenerationCleanupSync)
                    } else {
                        record_publication_event(PublicationEvent::StagedGenerationCleanupSync);
                        Ok(())
                    }
                },
            )?;
            Err(error)
        }
    }
}

fn open_publication_directories(
    repository: &HeldDirectory,
    create: bool,
) -> Result<Option<PublicationDirectories>, AppError> {
    let root = repository.anchored_directory("Crouzeix textbook repository")?;
    let publication = if create {
        root.walk_or_create(
            Path::new(PUBLICATION_ROOT),
            PUBLICATION_DIRECTORY_MODE,
            "Crouzeix textbook publication directory",
        )?
    } else {
        match root.walk(
            Path::new(PUBLICATION_ROOT),
            "Crouzeix textbook publication directory",
        ) {
            Ok(directory) => directory,
            Err(error) if error.code() == "fs.missing" => return Ok(None),
            Err(error) => return Err(error),
        }
    };
    publication.verify_owner_mode(
        PUBLICATION_DIRECTORY_MODE,
        "Crouzeix textbook publication directory",
    )?;
    let generations = if create {
        Some(root.walk_or_create(
            Path::new(GENERATIONS_ROOT),
            PUBLICATION_DIRECTORY_MODE,
            "Crouzeix textbook generations directory",
        )?)
    } else {
        match root.walk(
            Path::new(GENERATIONS_ROOT),
            "Crouzeix textbook generations directory",
        ) {
            Ok(directory) => Some(directory),
            Err(error) if error.code() == "fs.missing" => None,
            Err(error) => return Err(error),
        }
    };
    if let Some(generations) = &generations {
        generations.verify_owner_mode(
            PUBLICATION_DIRECTORY_MODE,
            "Crouzeix textbook generations directory",
        )?;
    }
    Ok(Some(PublicationDirectories {
        publication,
        generations,
    }))
}

fn acquire_publication_lock(
    directories: &PublicationDirectories,
    exclusive: bool,
) -> Result<Option<HeldFileLock>, AppError> {
    let name = AnchoredDirectory::cstring(LOCK_NAME, "Crouzeix textbook publication lock")?;
    directories
        .publication
        .acquire_file_lock(
            &name,
            exclusive,
            exclusive,
            "Crouzeix textbook publication lock",
        )
        .map_err(|error| {
            if error.code() == "fs.lock_busy" {
                AppError::invalid_input(
                    "crouzeix-textbook.publication.lock-busy",
                    "another Crouzeix textbook publication is in progress",
                )
            } else {
                error
            }
        })
}

fn publication_generation_error(error: AppError) -> AppError {
    if error.code() == "fs.limit" {
        AppError::invalid_input(
            "crouzeix-textbook.publication.generation",
            "publication generation exceeds six entries",
        )
    } else {
        error
    }
}

fn effective_uid() -> u32 {
    unsafe { libc::geteuid() }
}

fn hex_digest(digest: [u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

fn generation_output_path(generation: &str, name: &str) -> PathBuf {
    Path::new(GENERATIONS_ROOT).join(generation).join(name)
}

fn generation_digest(files: &BTreeMap<&str, Vec<u8>>) -> String {
    let mut digest = Sha256::new();
    digest.update(MANIFEST_SCHEMA.as_bytes());
    for (name, bytes) in files {
        digest.update((name.len() as u64).to_be_bytes());
        digest.update(name.as_bytes());
        digest.update((bytes.len() as u64).to_be_bytes());
        digest.update(bytes);
    }
    format!("{:x}", digest.finalize())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validation_error(mut diagnostics: Vec<TextbookDiagnostic>) -> AppError {
    diagnostics.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.column.cmp(&right.column))
            .then_with(|| left.identity.cmp(&right.identity))
            .then_with(|| left.field.cmp(&right.field))
            .then_with(|| left.code.cmp(right.code))
    });
    let rendered = serde_json::to_string(&diagnostics)
        .unwrap_or_else(|error| format!("diagnostic serialization failed: {error}"));
    AppError::invalid_input(
        "crouzeix-textbook.publication.validation",
        format!("textbook publication validation failed: {rendered}"),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    use crate::fs::descriptor::{with_creation_fault, CreationStage};
    use crate::fs::PublicMutationOperation;

    use super::{
        acquire_publication_lock, inspect_current_publication,
        inspect_current_publication_with_hook, install_generation, open_publication_directories,
        prepare_files, switch_and_validate, take_publication_trace, validate_expected_generation,
        validate_manifest_with_hooks, with_publication_fault, with_publication_faults, AppError,
        HeldDirectory, PublicationEvent, OUTPUT_NAMES,
    };

    fn prepared_files(marker: &str) -> BTreeMap<&'static str, Vec<u8>> {
        OUTPUT_NAMES
            .into_iter()
            .map(|name| {
                (
                    name,
                    format!("<!-- generated -->\n\n# {name}\n\n{marker}\n").into_bytes(),
                )
            })
            .collect()
    }

    fn setup_publication_root() -> (
        tempfile::TempDir,
        HeldDirectory,
        super::PublicationDirectories,
        super::HeldFileLock,
    ) {
        let root = tempfile::tempdir().expect("publication transaction root");
        let repository =
            HeldDirectory::open(root.path(), "publication transaction root").expect("held root");
        let directories = open_publication_directories(&repository, true)
            .expect("publication directories")
            .expect("created publication directories");
        let lock = acquire_publication_lock(&directories, true)
            .expect("publication lock")
            .expect("created publication lock");
        (root, repository, directories, lock)
    }

    #[test]
    fn post_switch_replacement_is_detected_and_previous_pointer_is_restored() {
        let (root, repository, directories, _lock) = setup_publication_root();

        let first = prepare_files(prepared_files("first")).expect("first preparation");
        install_generation(&directories, &first).expect("install first generation");
        let empty = inspect_current_publication(&directories).expect("empty pointer");
        switch_and_validate(&repository, &directories, &first, &empty, |_| Ok(()))
            .expect("publish first generation");
        let first_current = inspect_current_publication(&directories).expect("first pointer");

        let second = prepare_files(prepared_files("second")).expect("second preparation");
        install_generation(&directories, &second).expect("install second generation");
        let replacement = root.path().join(&second.manifest.outputs[0].path);
        let error = switch_and_validate(&repository, &directories, &second, &first_current, |_| {
            std::thread::scope(|scope| {
                scope
                    .spawn(|| {
                        let mut permissions = fs::metadata(&replacement)
                            .expect("replacement metadata")
                            .permissions();
                        permissions.set_mode(0o644);
                        fs::set_permissions(&replacement, permissions)
                            .expect("make replacement writable");
                        fs::write(&replacement, b"concurrent replacement\n")
                            .expect("replace switched output");
                        let mut permissions = fs::metadata(&replacement)
                            .expect("replacement metadata")
                            .permissions();
                        permissions.set_mode(0o444);
                        fs::set_permissions(&replacement, permissions)
                            .expect("restore immutable mode");
                    })
                    .join()
                    .expect("replacement thread");
            });
            Ok(())
        })
        .expect_err("post-switch replacement must fail");
        assert_eq!(error.code(), "crouzeix-textbook.publication.digest");

        let restored = inspect_current_publication(&directories).expect("restored pointer");
        assert_eq!(restored.bytes, first_current.bytes);
        assert!(validate_expected_generation(&directories, &first)
            .expect("first generation remains valid"));
    }

    #[test]
    fn pointer_inode_swap_during_generation_validation_is_rejected() {
        let (root, repository, directories, _lock) = setup_publication_root();
        let prepared = prepare_files(prepared_files("pointer read set")).expect("preparation");
        install_generation(&directories, &prepared).expect("install generation");
        let empty = inspect_current_publication(&directories).expect("empty pointer");
        switch_and_validate(&repository, &directories, &prepared, &empty, |_| Ok(()))
            .expect("initial publication");
        let current_path = root.path().join(super::CURRENT_PATH);

        let error = inspect_current_publication_with_hook(&directories, || {
            let replacement = current_path.with_extension("validation-replacement");
            fs::write(&replacement, &prepared.manifest_bytes).expect("replacement pointer");
            let mut permissions = fs::metadata(&replacement)
                .expect("replacement pointer metadata")
                .permissions();
            permissions.set_mode(0o600);
            fs::set_permissions(&replacement, permissions).expect("replacement pointer mode");
            fs::rename(replacement, &current_path).expect("swap pointer during validation");
        })
        .expect_err("pointer read set must remain stable");
        assert_eq!(
            error.code(),
            "crouzeix-textbook.publication.concurrent-change"
        );
    }

    #[test]
    fn failed_initial_post_switch_validation_removes_the_new_pointer() {
        let (root, repository, directories, _lock) = setup_publication_root();
        let prepared = prepare_files(prepared_files("initial")).expect("preparation");
        install_generation(&directories, &prepared).expect("install generation");
        let empty = inspect_current_publication(&directories).expect("empty pointer");
        let replacement = root.path().join(&prepared.manifest.outputs[0].path);
        switch_and_validate(&repository, &directories, &prepared, &empty, |_| {
            let mut permissions = fs::metadata(&replacement)
                .expect("replacement metadata")
                .permissions();
            permissions.set_mode(0o644);
            fs::set_permissions(&replacement, permissions).expect("make replacement writable");
            fs::write(&replacement, b"invalid initial generation\n")
                .expect("replace switched output");
            Ok(())
        })
        .expect_err("invalid initial generation must fail");
        let current = inspect_current_publication(&directories).expect("pointer after rollback");
        assert!(current.bytes.is_none(), "initial pointer must be removed");
    }

    #[test]
    fn generation_directory_inode_swap_is_rejected() {
        let (root, _repository, directories, _lock) = setup_publication_root();
        let prepared = prepare_files(prepared_files("generation swap")).expect("preparation");
        install_generation(&directories, &prepared).expect("install generation");
        let generation = root
            .path()
            .join("atlas/src/content/generated/crouzeix_textbook/publication/generations")
            .join(&prepared.generation);
        let displaced = generation.with_extension("displaced");

        let error = validate_manifest_with_hooks(
            &directories,
            &prepared.manifest,
            || {
                fs::rename(&generation, &displaced).expect("displace generation");
                fs::create_dir(&generation).expect("replacement generation");
                for name in OUTPUT_NAMES {
                    let destination = generation.join(name);
                    fs::copy(displaced.join(name), &destination).expect("copy replacement ledger");
                    let mut permissions = fs::metadata(&destination)
                        .expect("replacement ledger metadata")
                        .permissions();
                    permissions.set_mode(0o444);
                    fs::set_permissions(destination, permissions).expect("replacement ledger mode");
                }
            },
            |_| {},
        )
        .expect_err("same-byte generation on a different inode must fail");
        assert_eq!(error.code(), "fs.concurrent_change");
    }

    #[test]
    fn individual_ledger_inode_swap_is_rejected() {
        let (root, _repository, directories, _lock) = setup_publication_root();
        let prepared = prepare_files(prepared_files("ledger swap")).expect("preparation");
        install_generation(&directories, &prepared).expect("install generation");
        let ledger = root.path().join(&prepared.manifest.outputs[0].path);
        let replacement = ledger.with_extension("replacement");
        let mut swapped = false;

        let error = validate_manifest_with_hooks(
            &directories,
            &prepared.manifest,
            || {},
            |_| {
                if swapped {
                    return;
                }
                swapped = true;
                fs::copy(&ledger, &replacement).expect("copy replacement ledger");
                let mut permissions = fs::metadata(&replacement)
                    .expect("replacement ledger metadata")
                    .permissions();
                permissions.set_mode(0o444);
                fs::set_permissions(&replacement, permissions).expect("replacement ledger mode");
                fs::rename(&replacement, &ledger).expect("replace ledger inode");
            },
        )
        .expect_err("same-byte ledger on a different inode must fail");
        assert_eq!(error.code(), "fs.concurrent_change");
    }

    #[test]
    fn already_read_ledger_swap_is_rejected_by_final_read_set_validation() {
        let (root, _repository, directories, _lock) = setup_publication_root();
        let prepared = prepare_files(prepared_files("late ledger swap")).expect("preparation");
        install_generation(&directories, &prepared).expect("install generation");
        let first_ledger = root.path().join(&prepared.manifest.outputs[0].path);
        let replacement = first_ledger.with_extension("late-replacement");
        let mut swapped = false;

        let error = validate_manifest_with_hooks(
            &directories,
            &prepared.manifest,
            || {},
            |name| {
                if swapped || name != OUTPUT_NAMES[1] {
                    return;
                }
                swapped = true;
                fs::copy(&first_ledger, &replacement).expect("copy late replacement ledger");
                let mut permissions = fs::metadata(&replacement)
                    .expect("late replacement metadata")
                    .permissions();
                permissions.set_mode(0o444);
                fs::set_permissions(&replacement, permissions)
                    .expect("late replacement ledger mode");
                fs::rename(&replacement, &first_ledger).expect("replace already-read ledger");
            },
        )
        .expect_err("already-read ledger replacement must fail");
        assert_eq!(error.code(), "crouzeix-textbook.publication.generation");
    }

    #[test]
    fn late_generation_entry_is_rejected_by_final_enumeration() {
        let (root, _repository, directories, _lock) = setup_publication_root();
        let prepared = prepare_files(prepared_files("late entry")).expect("preparation");
        install_generation(&directories, &prepared).expect("install generation");
        let generation = root
            .path()
            .join("atlas/src/content/generated/crouzeix_textbook/publication/generations")
            .join(&prepared.generation);
        let mut inserted = false;

        let error = validate_manifest_with_hooks(
            &directories,
            &prepared.manifest,
            || {},
            |_| {
                if inserted {
                    return;
                }
                inserted = true;
                let extra = generation.join("unexpected.md");
                fs::write(&extra, b"unexpected\n").expect("late generation entry");
                let mut permissions = fs::metadata(&extra)
                    .expect("late entry metadata")
                    .permissions();
                permissions.set_mode(0o444);
                fs::set_permissions(extra, permissions).expect("late entry mode");
            },
        )
        .expect_err("late generation entry must fail");
        assert_eq!(error.code(), "crouzeix-textbook.publication.generation");
    }

    #[test]
    fn lock_path_replacement_cannot_split_a_successful_publication() {
        let (root, _repository, directories, lock) = setup_publication_root();
        let lock_path = root.path().join(super::LOCK_PATH);
        fs::remove_file(&lock_path).expect("unlink held lock path");
        fs::write(&lock_path, b"").expect("replacement lock path");
        let mut permissions = fs::metadata(&lock_path)
            .expect("replacement lock metadata")
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&lock_path, permissions).expect("replacement lock mode");
        let second = acquire_publication_lock(&directories, true)
            .expect("replacement lock acquisition")
            .expect("replacement lock exists");

        let error = lock
            .verify("Crouzeix textbook publication lock")
            .expect_err("original writer must detect lock replacement");
        assert_eq!(error.code(), "fs.concurrent_change");
        second
            .verify("replacement Crouzeix textbook publication lock")
            .expect("replacement lock remains internally coherent");
    }

    #[test]
    fn initial_rollback_preserves_a_same_bytes_pointer_replacement() {
        let (root, repository, directories, _lock) = setup_publication_root();
        let prepared = prepare_files(prepared_files("rollback replacement")).expect("preparation");
        install_generation(&directories, &prepared).expect("install generation");
        let empty = inspect_current_publication(&directories).expect("empty pointer");
        let current_path = root.path().join(super::CURRENT_PATH);
        let replacement_inode = std::cell::Cell::new(0);

        let error = switch_and_validate(&repository, &directories, &prepared, &empty, |_| {
            let replacement = current_path.with_extension("replacement");
            fs::write(&replacement, &prepared.manifest_bytes)
                .expect("same-byte replacement pointer");
            let mut permissions = fs::metadata(&replacement)
                .expect("replacement pointer metadata")
                .permissions();
            permissions.set_mode(0o600);
            fs::set_permissions(&replacement, permissions).expect("replacement pointer mode");
            replacement_inode.set(
                fs::metadata(&replacement)
                    .expect("replacement pointer metadata")
                    .ino(),
            );
            fs::rename(replacement, &current_path).expect("replace installed pointer");
            Err(AppError::external("test.post-switch", "force rollback"))
        })
        .expect_err("ambiguous rollback must fail");
        assert_eq!(error.code(), "crouzeix-textbook.publication.rollback");
        assert_eq!(
            fs::metadata(&current_path)
                .expect("preserved replacement pointer")
                .ino(),
            replacement_inode.get()
        );
        assert_eq!(
            fs::read(current_path).expect("replacement pointer bytes"),
            prepared.manifest_bytes
        );
    }

    #[test]
    fn publication_sync_order_precedes_the_pointer_readback() {
        let (_root, repository, directories, _lock) = setup_publication_root();
        let prepared = prepare_files(prepared_files("sync order")).expect("preparation");
        take_publication_trace();
        install_generation(&directories, &prepared).expect("install generation");
        let empty = inspect_current_publication(&directories).expect("empty pointer");
        switch_and_validate(&repository, &directories, &prepared, &empty, |_| Ok(()))
            .expect("switch generation");

        let mut expected = OUTPUT_NAMES
            .into_iter()
            .map(|name| PublicationEvent::LedgerFileSync(name.to_owned()))
            .collect::<Vec<_>>();
        expected.extend([
            PublicationEvent::GenerationDirectorySync,
            PublicationEvent::GenerationRename,
            PublicationEvent::GenerationPostRenameReopen,
            PublicationEvent::GenerationsParentSync,
            PublicationEvent::PointerFileSync,
            PublicationEvent::PointerRename,
            PublicationEvent::PointerInstallValidation,
            PublicationEvent::PointerParentSync,
            PublicationEvent::Readback,
        ]);
        assert_eq!(take_publication_trace(), expected);
    }

    #[test]
    fn sequenced_precommit_and_rollback_sync_faults_preserve_the_prior_pointer() {
        let (_root, repository, directories, _lock) = setup_publication_root();
        let first =
            prepare_files(prepared_files("prior rollback pointer")).expect("first preparation");
        install_generation(&directories, &first).expect("install first generation");
        let empty = inspect_current_publication(&directories).expect("empty pointer");
        switch_and_validate(&repository, &directories, &first, &empty, |_| Ok(()))
            .expect("publish first generation");
        let prior = inspect_current_publication(&directories).expect("prior pointer");
        let second =
            prepare_files(prepared_files("next rollback pointer")).expect("second preparation");
        install_generation(&directories, &second).expect("install second generation");

        with_publication_faults(
            vec![
                PublicationEvent::PointerParentSync,
                PublicationEvent::PointerRollbackSync(PublicMutationOperation::RollbackSync),
            ],
            || switch_and_validate(&repository, &directories, &second, &prior, |_| Ok(())),
        )
        .expect_err("rollback synchronization fault must be observable");
        let current = inspect_current_publication(&directories).expect("restored pointer");
        assert_eq!(current.bytes, prior.bytes);
        assert!(current
            .snapshot
            .as_ref()
            .zip(prior.snapshot.as_ref())
            .is_some_and(|(current, prior)| current.matches_moved_object(prior)));
    }

    #[test]
    fn initial_rename_fault_then_cleanup_sync_fault_leaves_no_pointer_or_staging() {
        let (root, repository, directories, _lock) = setup_publication_root();
        let prepared =
            prepare_files(prepared_files("initial rename cleanup")).expect("preparation");
        install_generation(&directories, &prepared).expect("install generation");
        let empty = inspect_current_publication(&directories).expect("empty pointer");

        with_publication_faults(
            vec![
                PublicationEvent::PointerRename,
                PublicationEvent::PointerCleanupSync,
            ],
            || switch_and_validate(&repository, &directories, &prepared, &empty, |_| Ok(())),
        )
        .expect_err("rename and cleanup faults must be observed");
        let current = inspect_current_publication(&directories).expect("pointer after cleanup");
        assert!(current.bytes.is_none(), "initial pointer remains absent");
        assert!(
            fs::read_dir(
                root.path()
                    .join("atlas/src/content/generated/crouzeix_textbook/publication")
            )
            .expect("publication directory")
            .all(|entry| !entry
                .expect("publication entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".harp-public-")),
            "rename failure must not leave pointer staging"
        );
        switch_and_validate(&repository, &directories, &prepared, &empty, |_| Ok(()))
            .expect("allocator remains reusable after initial cleanup");
    }

    #[test]
    fn readback_rollback_sync_fault_is_traced_after_removing_initial_pointer() {
        let (_root, repository, directories, _lock) = setup_publication_root();
        let prepared = prepare_files(prepared_files("rollback trace")).expect("preparation");
        install_generation(&directories, &prepared).expect("install generation");
        let empty = inspect_current_publication(&directories).expect("empty pointer");
        let error = with_publication_faults(
            vec![
                PublicationEvent::Readback,
                PublicationEvent::PointerRollbackSync(PublicMutationOperation::CleanupSync),
            ],
            || switch_and_validate(&repository, &directories, &prepared, &empty, |_| Ok(())),
        )
        .expect_err("readback and rollback fault must be consumed");
        assert_eq!(error.code(), "crouzeix-textbook.publication.rollback");
        let current = inspect_current_publication(&directories).expect("rolled-back pointer");
        assert!(current.bytes.is_none());
    }

    #[test]
    fn readback_rollback_file_and_parent_faults_are_operation_specific() {
        for operation in [
            PublicMutationOperation::FileSync,
            PublicMutationOperation::ParentSync,
        ] {
            let (_root, repository, directories, _lock) = setup_publication_root();
            let first = prepare_files(prepared_files("rollback prior")).expect("first preparation");
            install_generation(&directories, &first).expect("install first");
            let empty = inspect_current_publication(&directories).expect("empty pointer");
            switch_and_validate(&repository, &directories, &first, &empty, |_| Ok(()))
                .expect("publish first");
            let prior = inspect_current_publication(&directories).expect("prior pointer");
            let second =
                prepare_files(prepared_files("rollback second")).expect("second preparation");
            install_generation(&directories, &second).expect("install second");

            let error = with_publication_faults(
                vec![
                    PublicationEvent::Readback,
                    PublicationEvent::PointerRollbackSync(operation),
                ],
                || switch_and_validate(&repository, &directories, &second, &prior, |_| Ok(())),
            )
            .expect_err("targeted rollback fault");
            assert_eq!(error.code(), "crouzeix-textbook.publication.rollback");
            let current = inspect_current_publication(&directories).expect("current pointer");
            assert_eq!(current.bytes, Some(second.manifest_bytes), "{operation:?}");
        }
    }

    #[test]
    fn successful_prior_rollback_traces_file_parent_and_cleanup_syncs() {
        let (_root, repository, directories, _lock) = setup_publication_root();
        let first = prepare_files(prepared_files("trace prior")).expect("first preparation");
        install_generation(&directories, &first).expect("install first");
        let empty = inspect_current_publication(&directories).expect("empty pointer");
        switch_and_validate(&repository, &directories, &first, &empty, |_| Ok(()))
            .expect("publish first");
        let prior = inspect_current_publication(&directories).expect("prior pointer");
        let second = prepare_files(prepared_files("trace second")).expect("second preparation");
        install_generation(&directories, &second).expect("install second");
        take_publication_trace();

        with_publication_fault(PublicationEvent::Readback, || {
            switch_and_validate(&repository, &directories, &second, &prior, |_| Ok(()))
        })
        .expect_err("readback fault must roll back");
        let current = inspect_current_publication(&directories).expect("restored pointer");
        assert_eq!(current.bytes, prior.bytes);
        let trace = take_publication_trace();
        for operation in [
            PublicMutationOperation::FileSync,
            PublicMutationOperation::ParentSync,
            PublicMutationOperation::CleanupSync,
        ] {
            assert!(
                trace.contains(&PublicationEvent::PointerRollbackSync(operation)),
                "successful rollback must trace {operation:?}"
            );
        }
    }

    #[test]
    fn later_writer_during_readback_is_preserved_without_rollback_sync() {
        let (root, repository, directories, _lock) = setup_publication_root();
        let first = prepare_files(prepared_files("later writer prior")).expect("first preparation");
        install_generation(&directories, &first).expect("install first");
        let empty = inspect_current_publication(&directories).expect("empty pointer");
        switch_and_validate(&repository, &directories, &first, &empty, |_| Ok(()))
            .expect("publish first");
        let prior = inspect_current_publication(&directories).expect("prior pointer");
        let second =
            prepare_files(prepared_files("later writer second")).expect("second preparation");
        install_generation(&directories, &second).expect("install second");
        let current_path = root.path().join(super::CURRENT_PATH);
        let replacement_inode = std::cell::Cell::new(0);
        take_publication_trace();

        let error = switch_and_validate(&repository, &directories, &second, &prior, |_| {
            let replacement = current_path.with_extension("later-writer");
            fs::write(&replacement, &second.manifest_bytes).expect("later pointer");
            let mut permissions = fs::metadata(&replacement)
                .expect("later pointer metadata")
                .permissions();
            permissions.set_mode(0o600);
            fs::set_permissions(&replacement, permissions).expect("later pointer mode");
            replacement_inode.set(
                fs::metadata(&replacement)
                    .expect("later pointer metadata")
                    .ino(),
            );
            fs::rename(replacement, &current_path).expect("substitute later pointer");
            Ok(())
        })
        .expect_err("later pointer makes rollback ambiguous");
        assert_eq!(error.code(), "crouzeix-textbook.publication.rollback");
        assert_eq!(
            fs::metadata(&current_path)
                .expect("later pointer remains")
                .ino(),
            replacement_inode.get()
        );
        assert!(
            !take_publication_trace()
                .iter()
                .any(|event| matches!(event, PublicationEvent::PointerRollbackSync(_))),
            "identity mismatch before rollback mutation must not emit a sync"
        );
    }

    #[test]
    fn pre_rename_generation_faults_remove_staging_and_leave_the_allocator_reusable() {
        let faults = [
            PublicationEvent::LedgerFileSync(OUTPUT_NAMES[0].to_owned()),
            PublicationEvent::GenerationDirectorySync,
            PublicationEvent::GenerationRename,
        ];
        for fault in faults {
            let (root, _repository, directories, _lock) = setup_publication_root();
            let prepared = prepare_files(prepared_files("staging cleanup")).expect("preparation");

            let error = with_publication_fault(fault.clone(), || {
                install_generation(&directories, &prepared)
            })
            .expect_err("pre-rename fault must fail publication");
            assert_eq!(error.code(), "crouzeix-textbook.publication.sync");

            let generations = root
                .path()
                .join("atlas/src/content/generated/crouzeix_textbook/publication/generations");
            assert!(
                fs::read_dir(&generations)
                    .expect("enumerate generations")
                    .all(|entry| !entry
                        .expect("generation entry")
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".prepared-generation-")),
                "{fault:?} must not leave a staged generation"
            );
            install_generation(&directories, &prepared)
                .expect("allocator remains reusable after staged cleanup");
        }
    }

    #[test]
    fn every_ledger_boundary_cleans_prior_children_even_when_cleanup_sync_faults() {
        for ledger in OUTPUT_NAMES {
            let (root, _repository, directories, _lock) = setup_publication_root();
            let prepared =
                prepare_files(prepared_files("sequenced cleanup fault")).expect("preparation");
            let result = with_publication_faults(
                vec![
                    PublicationEvent::LedgerFileSync(ledger.to_owned()),
                    PublicationEvent::StagedGenerationCleanupSync,
                ],
                || install_generation(&directories, &prepared),
            );
            assert!(result.is_err(), "{ledger} fault must be observable");
            let generations = root
                .path()
                .join("atlas/src/content/generated/crouzeix_textbook/publication/generations");
            assert!(
                fs::read_dir(&generations)
                    .expect("enumerate generations")
                    .all(|entry| !entry
                        .expect("generation entry")
                        .file_name()
                        .to_string_lossy()
                        .starts_with(".prepared-generation-")),
                "{ledger} fault must not leave staging residue"
            );
            install_generation(&directories, &prepared)
                .expect("allocator remains reusable after cleanup-sync fault");
        }
    }

    #[test]
    fn ledger_setup_cleanup_sync_is_traced_and_a_sequenced_fault_leaves_no_residue() {
        let (root, _repository, directories, _lock) = setup_publication_root();
        let prepared = prepare_files(prepared_files("ledger setup cleanup")).expect("preparation");
        let result = with_creation_fault(CreationStage::FileWrite, || {
            with_publication_faults(
                vec![
                    PublicationEvent::CreationCleanupSync,
                    PublicationEvent::StagedGenerationCleanupSync,
                ],
                || install_generation(&directories, &prepared),
            )
        });
        assert!(
            result.is_err(),
            "creation cleanup sync fault must be observable"
        );
        let generations = root
            .path()
            .join("atlas/src/content/generated/crouzeix_textbook/publication/generations");
        assert!(
            fs::read_dir(&generations)
                .expect("enumerate generations")
                .all(|entry| !entry
                    .expect("generation entry")
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".prepared-generation-")),
            "faulted setup cleanup must not leave a staged generation"
        );
        install_generation(&directories, &prepared)
            .expect("allocator remains reusable after setup cleanup fault");
    }

    #[test]
    fn post_rename_reopen_fault_leaves_a_complete_digest_generation_orphan() {
        let (root, _repository, directories, _lock) = setup_publication_root();
        let prepared =
            prepare_files(prepared_files("post-rename reopen fault")).expect("preparation");

        with_publication_fault(PublicationEvent::GenerationPostRenameReopen, || {
            install_generation(&directories, &prepared)
        })
        .expect_err("post-rename reopen fault must be observable");

        let generation = root
            .path()
            .join("atlas/src/content/generated/crouzeix_textbook/publication/generations")
            .join(&prepared.generation);
        assert!(
            generation.is_dir(),
            "the digest generation is an honest orphan"
        );
        for name in OUTPUT_NAMES {
            assert_eq!(
                fs::read(generation.join(name)).expect("complete orphan ledger"),
                prepared.files[name]
            );
        }
    }

    #[test]
    fn post_commit_pointer_cleanup_sync_failure_keeps_the_new_pointer_successful() {
        let (_root, repository, directories, _lock) = setup_publication_root();
        let first = prepare_files(prepared_files("first pointer")).expect("first preparation");
        install_generation(&directories, &first).expect("install first generation");
        let empty = inspect_current_publication(&directories).expect("empty pointer");
        switch_and_validate(&repository, &directories, &first, &empty, |_| Ok(()))
            .expect("publish first generation");
        let previous = inspect_current_publication(&directories).expect("first pointer");

        let second = prepare_files(prepared_files("second pointer")).expect("second preparation");
        install_generation(&directories, &second).expect("install second generation");
        with_publication_fault(PublicationEvent::PointerCleanupSync, || {
            switch_and_validate(&repository, &directories, &second, &previous, |_| Ok(()))
        })
        .expect("cleanup synchronization after the commit point is best effort");

        let current = inspect_current_publication(&directories).expect("current pointer");
        assert_eq!(current.bytes, Some(second.manifest_bytes));
    }

    #[test]
    fn every_precommit_pointer_fault_preserves_the_exact_prior_pointer() {
        let faults = [
            PublicationEvent::PointerFileSync,
            PublicationEvent::PointerRename,
            PublicationEvent::PointerExchangeValidation,
            PublicationEvent::PointerParentSync,
        ];
        for fault in faults {
            let (_root, repository, directories, _lock) = setup_publication_root();
            let first = prepare_files(prepared_files("prior pointer")).expect("first preparation");
            install_generation(&directories, &first).expect("install first generation");
            let empty = inspect_current_publication(&directories).expect("empty pointer");
            switch_and_validate(&repository, &directories, &first, &empty, |_| Ok(()))
                .expect("publish first generation");
            let prior = inspect_current_publication(&directories).expect("prior pointer");

            let second = prepare_files(prepared_files("next pointer")).expect("second preparation");
            install_generation(&directories, &second).expect("install second generation");
            let error = with_publication_fault(fault.clone(), || {
                switch_and_validate(&repository, &directories, &second, &prior, |_| Ok(()))
            })
            .expect_err("precommit pointer fault must fail");
            assert_eq!(error.code(), "crouzeix-textbook.publication.sync");
            let current = inspect_current_publication(&directories).expect("preserved pointer");
            assert_eq!(current.bytes, prior.bytes, "{fault:?}");
            assert!(
                current
                    .snapshot
                    .as_ref()
                    .zip(prior.snapshot.as_ref())
                    .is_some_and(|(current, prior)| current.matches_moved_object(prior)),
                "{fault:?} must preserve the prior pointer identity and bytes"
            );
        }
    }

    #[test]
    fn publication_accepts_secure_repository_ancestors_and_keeps_owned_directories_exact() {
        let root = tempfile::tempdir().expect("publication transaction root");
        let secure_ancestor = root.path().join("atlas");
        fs::create_dir(&secure_ancestor).expect("secure repository ancestor");
        let mut permissions = fs::metadata(&secure_ancestor)
            .expect("secure ancestor metadata")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&secure_ancestor, permissions).expect("secure ancestor mode");
        let repository =
            HeldDirectory::open(root.path(), "publication transaction root").expect("held root");

        let directories = open_publication_directories(&repository, true)
            .expect("publication directories")
            .expect("created publication directories");
        directories
            .publication
            .verify_owner_mode(super::PUBLICATION_DIRECTORY_MODE, "publication directory")
            .expect("owned publication directory mode");
        directories
            .generations
            .as_ref()
            .expect("owned generations directory")
            .verify_owner_mode(super::PUBLICATION_DIRECTORY_MODE, "generations directory")
            .expect("owned generations directory mode");
    }

    #[test]
    fn every_publication_sync_boundary_fails_before_returning_success() {
        let faults = [
            PublicationEvent::LedgerFileSync(OUTPUT_NAMES[0].to_owned()),
            PublicationEvent::GenerationDirectorySync,
            PublicationEvent::GenerationRename,
            PublicationEvent::GenerationsParentSync,
            PublicationEvent::PointerFileSync,
            PublicationEvent::PointerRename,
            PublicationEvent::PointerParentSync,
            PublicationEvent::Readback,
        ];
        for fault in faults {
            let (_root, repository, directories, _lock) = setup_publication_root();
            let prepared = prepare_files(prepared_files("sync fault")).expect("preparation");
            let install_fault = matches!(
                fault,
                PublicationEvent::LedgerFileSync(_)
                    | PublicationEvent::GenerationDirectorySync
                    | PublicationEvent::GenerationRename
                    | PublicationEvent::GenerationsParentSync
            );
            let result = if install_fault {
                with_publication_fault(fault.clone(), || {
                    install_generation(&directories, &prepared)
                })
            } else {
                install_generation(&directories, &prepared).expect("install generation");
                let empty = inspect_current_publication(&directories).expect("empty pointer");
                with_publication_fault(fault.clone(), || {
                    switch_and_validate(&repository, &directories, &prepared, &empty, |_| Ok(()))
                })
            };
            assert!(result.is_err(), "{fault:?} must be observable");
            let current = inspect_current_publication(&directories).expect("pointer after fault");
            assert!(
                current.bytes.is_none(),
                "{fault:?} must not leave an invalid current pointer"
            );
        }
    }
}

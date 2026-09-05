use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::diagnostic::TextbookDiagnostic;

pub(super) const COVERAGE_PATH: &str = "content/crouzeix_textbook/coverage.json";
pub(super) const EXERCISES_PATH: &str = "content/crouzeix_textbook/exercises.json";
pub(super) const COVERAGE_SCHEMA: &str = "crouzeix-textbook-coverage/v2";
pub(super) const EXERCISES_SCHEMA: &str = "crouzeix-textbook-exercises/v2";
pub(super) const SCHEMA_CODE: &str = "crouzeix-textbook.contract.schema";
pub(super) const DUPLICATE_CODE: &str = "crouzeix-textbook.contract.duplicate";
pub(super) const COMBINATION_CODE: &str = "crouzeix-textbook.contract.invalid-combination";
const GRAPH_UNKNOWN_CODE: &str = "crouzeix-textbook.pedagogical-graph.unknown-prerequisite";
const GRAPH_DUPLICATE_CODE: &str = "crouzeix-textbook.pedagogical-graph.duplicate-edge";
const GRAPH_SELF_CODE: &str = "crouzeix-textbook.pedagogical-graph.self-edge";
const GRAPH_CYCLE_CODE: &str = "crouzeix-textbook.pedagogical-graph.cycle";
const GRAPH_ORDER_CODE: &str = "crouzeix-textbook.pedagogical-graph.edge-order";

macro_rules! status_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
        #[serde(rename_all = "kebab-case")]
        pub enum $name {
            $($variant),+
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                let value = serde_json::to_value(self).map_err(|_| fmt::Error)?;
                formatter.write_str(value.as_str().ok_or(fmt::Error)?)
            }
        }
    };
}

status_enum!(PublicationStatus { Draft, Active });
status_enum!(ProseProofStatus {
    Summary,
    Reconstructible,
    NotApplicable,
});
status_enum!(LeanCorrespondenceStatus {
    Unmapped,
    Checkpoint,
    Exact,
    NotApplicable,
});
status_enum!(FormalMode {
    ProvedHere,
    ReexportedProof,
    Definition,
    Checkpoint,
    Informal,
});

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewStatus {
    source_id: String,
    status: String,
}

impl ReviewStatus {
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn status(&self) -> &str {
        &self.status
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LeanDeclaration {
    pub(super) name: String,
    pub(super) underlying_declaration: Option<String>,
    pub(super) source_path: PathBuf,
    pub(super) line: u32,
    pub(super) column: u32,
    pub(super) verification_target: String,
    pub(super) type_sha256: String,
    pub(super) assumptions: Vec<String>,
    pub(super) axioms: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TheoremRow {
    pub(super) item_id: String,
    pub(super) chapter: u32,
    pub(super) kind: String,
    pub(super) prose_path: PathBuf,
    pub(super) anchor: String,
    pub(super) source_ids: Vec<String>,
    pub(super) pedagogical_prerequisites: Vec<String>,
    pub(super) publication_status: PublicationStatus,
    pub(super) prose_proof_status: ProseProofStatus,
    pub(super) lean_correspondence_status: LeanCorrespondenceStatus,
    pub(super) review_status: ReviewStatus,
    pub(super) formal_mode: FormalMode,
    pub(super) lean_declaration: Option<LeanDeclaration>,
}

impl TheoremRow {
    pub fn item_id(&self) -> &str {
        &self.item_id
    }

    pub fn publication_status(&self) -> PublicationStatus {
        self.publication_status
    }

    pub fn prose_proof_status(&self) -> ProseProofStatus {
        self.prose_proof_status
    }

    pub fn lean_correspondence_status(&self) -> LeanCorrespondenceStatus {
        self.lean_correspondence_status
    }

    pub fn review_status(&self) -> &ReviewStatus {
        &self.review_status
    }

    pub fn formal_mode(&self) -> FormalMode {
        self.formal_mode
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LeanStarter {
    pub(super) source_path: PathBuf,
    pub(super) line: u32,
    pub(super) column: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LeanSolution {
    pub(super) declaration: String,
    pub(super) source_path: PathBuf,
    pub(super) line: u32,
    pub(super) column: u32,
    pub(super) type_sha256: String,
    pub(super) verification_target: String,
    pub(super) axioms: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExerciseRow {
    pub(super) exercise_id: String,
    pub(super) chapter: u32,
    pub(super) anchor: String,
    pub(super) kind: String,
    pub(super) difficulty: String,
    pub(super) skills: Vec<String>,
    pub(super) starter: Option<LeanStarter>,
    pub(super) lean_solution: Option<LeanSolution>,
}

impl ExerciseRow {
    pub fn exercise_id(&self) -> &str {
        &self.exercise_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextbookContracts {
    pub(super) theorems: Vec<TheoremRow>,
    pub(super) exercises: Vec<ExerciseRow>,
}

impl TextbookContracts {
    pub fn theorems(&self) -> &[TheoremRow] {
        &self.theorems
    }

    pub fn exercises(&self) -> &[ExerciseRow] {
        &self.exercises
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawCoverage {
    pub(super) schema_version: String,
    pub(super) items: Vec<TheoremRow>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawExercises {
    pub(super) schema_version: String,
    pub(super) exercises: Vec<ExerciseRow>,
}

pub(super) struct RawContracts {
    pub(super) coverage: RawCoverage,
    pub(super) exercises: RawExercises,
}

impl TryFrom<RawContracts> for TextbookContracts {
    type Error = Vec<TextbookDiagnostic>;

    fn try_from(raw: RawContracts) -> Result<Self, Self::Error> {
        let (contracts, diagnostics) = validate(raw);
        if diagnostics.is_empty() {
            Ok(contracts)
        } else {
            Err(diagnostics)
        }
    }
}

pub(super) fn validate(raw: RawContracts) -> (TextbookContracts, Vec<TextbookDiagnostic>) {
    let mut diagnostics = Vec::new();
    validate_schema(
        &raw.coverage.schema_version,
        COVERAGE_SCHEMA,
        COVERAGE_PATH,
        &mut diagnostics,
    );
    validate_schema(
        &raw.exercises.schema_version,
        EXERCISES_SCHEMA,
        EXERCISES_PATH,
        &mut diagnostics,
    );
    validate_theorems(&raw.coverage.items, &mut diagnostics);
    validate_exercises(
        &raw.exercises.exercises,
        &raw.coverage.items,
        &mut diagnostics,
    );

    (
        TextbookContracts {
            theorems: raw.coverage.items,
            exercises: raw.exercises.exercises,
        },
        diagnostics,
    )
}

fn validate_schema(
    observed: &str,
    expected: &'static str,
    path: &'static str,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    if observed != expected {
        diagnostics.push(TextbookDiagnostic::new(
            SCHEMA_CODE,
            None,
            "schema_version",
            expected,
            observed,
            PathBuf::from(path),
        ));
    }
}

fn validate_theorems(rows: &[TheoremRow], diagnostics: &mut Vec<TextbookDiagnostic>) {
    let path = PathBuf::from(COVERAGE_PATH);
    let mut identities = BTreeSet::new();
    for row in rows {
        let identity = Some(row.item_id.clone());
        if !identities.insert(row.item_id.as_str()) {
            diagnostics.push(TextbookDiagnostic::new(
                DUPLICATE_CODE,
                identity.clone(),
                "item_id",
                "unique theorem identity",
                &row.item_id,
                path.clone(),
            ));
        }
        validate_chapter_identity(&row.item_id, row.chapter, false, path.clone(), diagnostics);
        if row.source_ids.is_empty() {
            diagnostics.push(TextbookDiagnostic::new(
                COMBINATION_CODE,
                identity.clone(),
                "source_ids",
                "at least one source identity",
                "empty",
                path.clone(),
            ));
        }
        if !row
            .source_ids
            .iter()
            .any(|id| id == &row.review_status.source_id)
        {
            diagnostics.push(TextbookDiagnostic::new(
                COMBINATION_CODE,
                identity.clone(),
                "review_status.source_id",
                "one of source_ids",
                &row.review_status.source_id,
                path.clone(),
            ));
        }
        if row.review_status.status.trim().is_empty() {
            diagnostics.push(TextbookDiagnostic::new(
                COMBINATION_CODE,
                identity.clone(),
                "review_status.status",
                "non-empty source review state",
                "empty",
                path.clone(),
            ));
        }

        if let Some(error) = validate_formal_combination(row) {
            diagnostics.push(TextbookDiagnostic::new(
                COMBINATION_CODE,
                identity,
                error.field,
                error.expected,
                error.observed,
                path.clone(),
            ));
        }
    }
    validate_pedagogical_graph(rows, diagnostics);
}

fn validate_pedagogical_graph(rows: &[TheoremRow], diagnostics: &mut Vec<TextbookDiagnostic>) {
    let path = PathBuf::from(COVERAGE_PATH);
    let rows_by_id = rows
        .iter()
        .map(|row| (row.item_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let known = rows_by_id.keys().copied().collect::<BTreeSet<_>>();
    let mut graph = BTreeMap::<&str, Vec<&str>>::new();
    let mut structurally_valid = true;

    for (item_id, row) in &rows_by_id {
        let mut seen = BTreeSet::new();
        let mut prerequisites = Vec::new();
        for prerequisite in &row.pedagogical_prerequisites {
            let identity = Some((*item_id).to_owned());
            if !seen.insert(prerequisite.as_str()) {
                structurally_valid = false;
                diagnostics.push(TextbookDiagnostic::new(
                    GRAPH_DUPLICATE_CODE,
                    identity,
                    "pedagogical_prerequisites",
                    "each prerequisite listed once",
                    prerequisite,
                    path.clone(),
                ));
                continue;
            }
            if prerequisite == item_id {
                structurally_valid = false;
                diagnostics.push(TextbookDiagnostic::new(
                    GRAPH_SELF_CODE,
                    identity,
                    "pedagogical_prerequisites",
                    "prerequisite other than the item itself",
                    prerequisite,
                    path.clone(),
                ));
                continue;
            }
            if !known.contains(prerequisite.as_str()) {
                structurally_valid = false;
                diagnostics.push(TextbookDiagnostic::new(
                    GRAPH_UNKNOWN_CODE,
                    identity,
                    "pedagogical_prerequisites",
                    "registered theorem item_id",
                    prerequisite,
                    path.clone(),
                ));
                continue;
            }
            prerequisites.push(prerequisite.as_str());
        }
        prerequisites.sort_unstable();
        graph.insert(item_id, prerequisites);
    }

    if let Some(cycle) = deterministic_cycle(&graph) {
        structurally_valid = false;
        diagnostics.push(TextbookDiagnostic::new(
            GRAPH_CYCLE_CODE,
            cycle.first().map(|identity| (*identity).to_owned()),
            "pedagogical_prerequisites",
            "acyclic pedagogical graph",
            cycle.join(" -> "),
            path.clone(),
        ));
    }

    if !structurally_valid {
        return;
    }

    validate_pedagogical_topology(&graph, &rows_by_id, diagnostics);
}

fn validate_pedagogical_topology<'a>(
    graph: &BTreeMap<&'a str, Vec<&'a str>>,
    rows_by_id: &BTreeMap<&'a str, &'a TheoremRow>,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    let path = PathBuf::from(COVERAGE_PATH);

    for (item_id, prerequisites) in graph {
        let row = rows_by_id[item_id];
        for prerequisite in prerequisites {
            let prerequisite_row = rows_by_id[prerequisite];
            if prerequisite_row.chapter > row.chapter
                && row.prose_proof_status != ProseProofStatus::Summary
            {
                diagnostics.push(TextbookDiagnostic::new(
                    GRAPH_ORDER_CODE,
                    Some((*item_id).to_owned()),
                    "pedagogical_prerequisites",
                    "the same chapter, an earlier chapter, or a forward reader link from a summary preview",
                    *prerequisite,
                    path.clone(),
                ));
            }
        }
    }
}

fn deterministic_cycle<'a>(graph: &BTreeMap<&'a str, Vec<&'a str>>) -> Option<Vec<&'a str>> {
    fn visit<'a>(
        node: &'a str,
        graph: &BTreeMap<&'a str, Vec<&'a str>>,
        states: &mut BTreeMap<&'a str, u8>,
        stack: &mut Vec<&'a str>,
    ) -> Option<Vec<&'a str>> {
        states.insert(node, 1);
        stack.push(node);
        if let Some(neighbors) = graph.get(node) {
            for &neighbor in neighbors {
                match states.get(neighbor).copied().unwrap_or(0) {
                    0 => {
                        if let Some(cycle) = visit(neighbor, graph, states, stack) {
                            return Some(cycle);
                        }
                    }
                    1 => {
                        let start = stack.iter().position(|candidate| *candidate == neighbor)?;
                        let mut cycle = stack[start..].to_vec();
                        cycle.push(neighbor);
                        return Some(cycle);
                    }
                    _ => {}
                }
            }
        }
        stack.pop();
        states.insert(node, 2);
        None
    }

    let mut states = BTreeMap::new();
    let mut stack = Vec::new();
    for &node in graph.keys() {
        if states.get(node).copied().unwrap_or(0) == 0 {
            if let Some(cycle) = visit(node, graph, &mut states, &mut stack) {
                return Some(cycle);
            }
        }
    }
    None
}

fn validate_exercises(
    rows: &[ExerciseRow],
    theorems: &[TheoremRow],
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    let path = PathBuf::from(EXERCISES_PATH);
    let mut identities = BTreeSet::new();
    let theorem_declarations = theorems
        .iter()
        .filter_map(|row| row.lean_declaration.as_ref())
        .map(|declaration| declaration.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut solution_declarations = BTreeSet::new();
    for row in rows {
        if !identities.insert(row.exercise_id.as_str()) {
            diagnostics.push(TextbookDiagnostic::new(
                DUPLICATE_CODE,
                Some(row.exercise_id.clone()),
                "exercise_id",
                "unique exercise identity",
                &row.exercise_id,
                path.clone(),
            ));
        }
        validate_chapter_identity(
            &row.exercise_id,
            row.chapter,
            true,
            path.clone(),
            diagnostics,
        );
        if row.skills.is_empty() {
            diagnostics.push(TextbookDiagnostic::new(
                COMBINATION_CODE,
                Some(row.exercise_id.clone()),
                "skills",
                "at least one skill or CFT item",
                "empty",
                path.clone(),
            ));
        }
        if let Some(solution) = &row.lean_solution {
            if solution.axioms.windows(2).any(|pair| pair[0] >= pair[1]) {
                diagnostics.push(TextbookDiagnostic::new(
                    COMBINATION_CODE,
                    Some(row.exercise_id.clone()),
                    "lean_solution.axioms",
                    "strictly sorted unique axiom names",
                    format!("{:?}", solution.axioms),
                    path.clone(),
                ));
            }
            if theorem_declarations.contains(solution.declaration.as_str()) {
                diagnostics.push(TextbookDiagnostic::new(
                    COMBINATION_CODE,
                    Some(row.exercise_id.clone()),
                    "lean_solution.declaration",
                    "distinct exercise solution declaration",
                    &solution.declaration,
                    path.clone(),
                ));
            } else if !solution_declarations.insert(solution.declaration.as_str()) {
                diagnostics.push(TextbookDiagnostic::new(
                    DUPLICATE_CODE,
                    Some(row.exercise_id.clone()),
                    "lean_solution.declaration",
                    "unique exercise solution declaration",
                    &solution.declaration,
                    path.clone(),
                ));
            }
        }
    }
}

fn validate_chapter_identity(
    identity: &str,
    chapter: u32,
    exercise: bool,
    path: PathBuf,
    diagnostics: &mut Vec<TextbookDiagnostic>,
) {
    if !(1..=35).contains(&chapter) {
        diagnostics.push(TextbookDiagnostic::new(
            COMBINATION_CODE,
            Some(identity.to_owned()),
            "chapter",
            "integer from 1 through 35",
            chapter.to_string(),
            path.clone(),
        ));
    }
    let prefix = if exercise {
        format!("CFT-{chapter:02}-E")
    } else {
        format!("CFT-{chapter:02}-")
    };
    if !identity.starts_with(&prefix) {
        diagnostics.push(TextbookDiagnostic::new(
            COMBINATION_CODE,
            Some(identity.to_owned()),
            if exercise { "exercise_id" } else { "item_id" },
            format!("identity beginning with {prefix}"),
            identity,
            path,
        ));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeclarationShape {
    Absent,
    Local,
    Reexport,
    InvalidUnderlying,
}

#[derive(Clone, Copy)]
struct AllowedCombination {
    mode: FormalMode,
    lean: LeanCorrespondenceStatus,
    prose: ProseProofStatus,
    declaration: DeclarationShape,
}

const ALLOWED_COMBINATIONS: &[AllowedCombination] = &[
    // A declaration may exist before its correspondence receipt is validated.
    AllowedCombination {
        mode: FormalMode::ProvedHere,
        lean: LeanCorrespondenceStatus::Unmapped,
        prose: ProseProofStatus::Summary,
        declaration: DeclarationShape::Local,
    },
    AllowedCombination {
        mode: FormalMode::ProvedHere,
        lean: LeanCorrespondenceStatus::Unmapped,
        prose: ProseProofStatus::Reconstructible,
        declaration: DeclarationShape::Local,
    },
    AllowedCombination {
        mode: FormalMode::ProvedHere,
        lean: LeanCorrespondenceStatus::Exact,
        prose: ProseProofStatus::Summary,
        declaration: DeclarationShape::Local,
    },
    AllowedCombination {
        mode: FormalMode::ProvedHere,
        lean: LeanCorrespondenceStatus::Exact,
        prose: ProseProofStatus::Reconstructible,
        declaration: DeclarationShape::Local,
    },
    AllowedCombination {
        mode: FormalMode::ReexportedProof,
        lean: LeanCorrespondenceStatus::Unmapped,
        prose: ProseProofStatus::Summary,
        declaration: DeclarationShape::Reexport,
    },
    AllowedCombination {
        mode: FormalMode::ReexportedProof,
        lean: LeanCorrespondenceStatus::Unmapped,
        prose: ProseProofStatus::Reconstructible,
        declaration: DeclarationShape::Reexport,
    },
    AllowedCombination {
        mode: FormalMode::ReexportedProof,
        lean: LeanCorrespondenceStatus::Exact,
        prose: ProseProofStatus::Summary,
        declaration: DeclarationShape::Reexport,
    },
    AllowedCombination {
        mode: FormalMode::ReexportedProof,
        lean: LeanCorrespondenceStatus::Exact,
        prose: ProseProofStatus::Reconstructible,
        declaration: DeclarationShape::Reexport,
    },
    AllowedCombination {
        mode: FormalMode::Definition,
        lean: LeanCorrespondenceStatus::Unmapped,
        prose: ProseProofStatus::NotApplicable,
        declaration: DeclarationShape::Absent,
    },
    AllowedCombination {
        mode: FormalMode::Definition,
        lean: LeanCorrespondenceStatus::Checkpoint,
        prose: ProseProofStatus::NotApplicable,
        declaration: DeclarationShape::Local,
    },
    AllowedCombination {
        mode: FormalMode::Definition,
        lean: LeanCorrespondenceStatus::Exact,
        prose: ProseProofStatus::NotApplicable,
        declaration: DeclarationShape::Local,
    },
    // A reviewed mathematical definition can have reconstructible exposition
    // even though it does not carry a theorem proof.
    AllowedCombination {
        mode: FormalMode::Definition,
        lean: LeanCorrespondenceStatus::Exact,
        prose: ProseProofStatus::Reconstructible,
        declaration: DeclarationShape::Local,
    },
    AllowedCombination {
        mode: FormalMode::Definition,
        lean: LeanCorrespondenceStatus::Exact,
        prose: ProseProofStatus::Reconstructible,
        declaration: DeclarationShape::Reexport,
    },
    // This is the intentional pre-formal orientation state.
    AllowedCombination {
        mode: FormalMode::Checkpoint,
        lean: LeanCorrespondenceStatus::Unmapped,
        prose: ProseProofStatus::Summary,
        declaration: DeclarationShape::Absent,
    },
    AllowedCombination {
        mode: FormalMode::Checkpoint,
        lean: LeanCorrespondenceStatus::Checkpoint,
        prose: ProseProofStatus::Summary,
        declaration: DeclarationShape::Local,
    },
    AllowedCombination {
        mode: FormalMode::Informal,
        lean: LeanCorrespondenceStatus::NotApplicable,
        prose: ProseProofStatus::NotApplicable,
        declaration: DeclarationShape::Absent,
    },
];

struct CombinationError {
    field: &'static str,
    expected: String,
    observed: String,
}

fn validate_formal_combination(row: &TheoremRow) -> Option<CombinationError> {
    let declaration = declaration_shape(row);
    if ALLOWED_COMBINATIONS.iter().any(|allowed| {
        allowed.mode == row.formal_mode
            && allowed.lean == row.lean_correspondence_status
            && allowed.prose == row.prose_proof_status
            && allowed.declaration == declaration
    }) {
        return None;
    }

    let for_mode = ALLOWED_COMBINATIONS
        .iter()
        .filter(|allowed| allowed.mode == row.formal_mode)
        .collect::<Vec<_>>();
    let for_lean = for_mode
        .iter()
        .copied()
        .filter(|allowed| allowed.lean == row.lean_correspondence_status)
        .collect::<Vec<_>>();
    if for_lean.is_empty() {
        return Some(CombinationError {
            field: "lean_correspondence_status",
            expected: joined_statuses(for_mode.iter().map(|allowed| allowed.lean)),
            observed: row.lean_correspondence_status.to_string(),
        });
    }

    let for_prose = for_lean
        .iter()
        .copied()
        .filter(|allowed| allowed.prose == row.prose_proof_status)
        .collect::<Vec<_>>();
    if for_prose.is_empty() {
        return Some(CombinationError {
            field: "prose_proof_status",
            expected: joined_statuses(for_lean.iter().map(|allowed| allowed.prose)),
            observed: row.prose_proof_status.to_string(),
        });
    }

    let field = if row.formal_mode == FormalMode::ReexportedProof
        || declaration == DeclarationShape::InvalidUnderlying
    {
        "lean_declaration.underlying_declaration"
    } else {
        "lean_declaration"
    };
    Some(CombinationError {
        field,
        expected: joined_declaration_shapes(for_prose.iter().map(|allowed| allowed.declaration)),
        observed: declaration_label(declaration).to_owned(),
    })
}

fn declaration_shape(row: &TheoremRow) -> DeclarationShape {
    let Some(declaration) = &row.lean_declaration else {
        return DeclarationShape::Absent;
    };
    match declaration.underlying_declaration.as_deref() {
        None => DeclarationShape::Local,
        Some(underlying)
            if !underlying.trim().is_empty() && underlying != declaration.name.as_str() =>
        {
            DeclarationShape::Reexport
        }
        Some(_) => DeclarationShape::InvalidUnderlying,
    }
}

fn joined_statuses<T: fmt::Display + Copy + Eq>(values: impl Iterator<Item = T>) -> String {
    let mut distinct = Vec::new();
    for value in values {
        if !distinct.contains(&value) {
            distinct.push(value);
        }
    }
    distinct
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" or ")
}

fn joined_declaration_shapes(values: impl Iterator<Item = DeclarationShape>) -> String {
    joined_statuses(values.map(declaration_label))
}

fn declaration_label(shape: DeclarationShape) -> &'static str {
    match shape {
        DeclarationShape::Absent => "absent",
        DeclarationShape::Local => "declaration data",
        DeclarationShape::Reexport => "declaration with substantive underlying proof",
        DeclarationShape::InvalidUnderlying => "invalid underlying declaration",
    }
}

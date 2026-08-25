use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CString;
use std::fmt;
use std::fs::{self, File};
use std::io::Read;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path};
use std::process::Command;

use serde::de::{self, DeserializeOwned, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};

const MANIFEST_SCHEMA: &str = "crouzeix-route-proof-manifest/v1";
const RECEIPT_SCHEMA: &str = "crouzeix-route-proof-receipt/v1";
const REVIEW_SCHEMA: &str = "crouzeix-proof-review/v1";
const ALLOWED_AXIOMS: [&str; 3] = ["Classical.choice", "Quot.sound", "propext"];
const MAX_ARTIFACT_BYTES: u64 = 2 * 1024 * 1024;
const MAX_CACHE_ARTIFACT_BYTES: u64 = 8 * 1024 * 1024;
const ROUTE_MANIFESTS: [&str; 3] = [
    "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json",
    "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/route-manifest.json",
    "labs/crouzeix_proof_reproduction/formal_targets/harp/route-manifest.json",
];
const LS_ARTIFACT_MANIFEST_PATH: &str =
    "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/artifact-manifest.json";
const SOURCE_MANIFEST_PATH: &str = "evidence/crouzeix_conjecture/source_manifest.tsv";
const LS_SOURCE_ID: &str = "LS-ARXIV-V1";
const LS_SOURCE_IDENTITY: &str = "arxiv:2608.03841v1";
const LS_SOURCE_ARCHIVE_SHA256: &str =
    "b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9";
const LS_SOURCE_FILE_SHA256: &str =
    "20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a";
const LS_SOURCE_PATH: &str = "CrouzeixConjecturev2.tex";
const LS_SOURCE_BYTES: u64 = 18_783;
const LS_SOURCE_LINES: u64 = 281;
const LS_SOURCE_IDENTITIES: [&str; 3] = [
    "arxiv:2608.03841v1",
    "sha256:20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a",
    "sha256:b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9",
];
const HARP_ALLOWED_LS_SUPPORT: [&str; 11] = [
    "Crouzeix.LoristSchwenninger.BoundaryEmbedding",
    "Crouzeix.LoristSchwenninger.BoundaryMultiplier",
    "Crouzeix.LoristSchwenninger.BoundarySquareRoot",
    "Crouzeix.LoristSchwenninger.CompanionAlgebra",
    "Crouzeix.LoristSchwenninger.CompletedSquare",
    "Crouzeix.LoristSchwenninger.CompressionMoments",
    "Crouzeix.LoristSchwenninger.Dilation",
    "Crouzeix.LoristSchwenninger.NormAttainment",
    "Crouzeix.LoristSchwenninger.PolynomialPowerCauchy",
    "Crouzeix.LoristSchwenninger.Recurrence",
    "Crouzeix.LoristSchwenninger.Scalar",
];

struct StrictValue(Value);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceLocatorKind<'a> {
    Local,
    Git(&'a str),
    Arxiv(&'a str),
}

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictValue;
            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("bounded JSON without duplicate keys")
            }
            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Bool(value)))
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Number(value.into())))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Number(value.into())))
            }
            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(value)
                    .map(Value::Number)
                    .map(StrictValue)
                    .ok_or_else(|| E::custom("non-finite number"))
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(value.to_owned())))
            }
            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(value)))
            }
            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(StrictValue(value)) = sequence.next_element()? {
                    if values.len() >= 4096 {
                        return Err(de::Error::custom("JSON array exceeds item bound"));
                    }
                    values.push(value);
                }
                Ok(StrictValue(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!("duplicate JSON key: {key}")));
                    }
                    let StrictValue(value) = map.next_value()?;
                    values.insert(key, value);
                }
                Ok(StrictValue(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(StrictVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum RouteId {
    Jin,
    LoristSchwenninger,
    Harp,
}

impl RouteId {
    fn aggregate(self) -> &'static str {
        match self {
            Self::Jin => "CrouzeixJin",
            Self::LoristSchwenninger => "CrouzeixLoristSchwenninger",
            Self::Harp => "CrouzeixHarp",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ClaimKind {
    SourceFaithful,
    Derived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ProvenanceKind {
    Source,
    SharedFoundation,
    ReusedRoute,
    Derived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum CorrespondenceKind {
    DirectSource,
    CompatibilityPort,
    StructuralRefactor,
    DerivedExtraction,
    SharedFoundation,
    ReusedRoute,
}

fn correspondence_matches(provenance: ProvenanceKind, correspondence: CorrespondenceKind) -> bool {
    matches!(
        (provenance, correspondence),
        (ProvenanceKind::Source, CorrespondenceKind::DirectSource)
            | (
                ProvenanceKind::Source,
                CorrespondenceKind::CompatibilityPort
            )
            | (
                ProvenanceKind::Source,
                CorrespondenceKind::StructuralRefactor
            )
            | (
                ProvenanceKind::Derived,
                CorrespondenceKind::DerivedExtraction
            )
            | (
                ProvenanceKind::SharedFoundation,
                CorrespondenceKind::SharedFoundation
            )
            | (ProvenanceKind::ReusedRoute, CorrespondenceKind::ReusedRoute)
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum NodeRole {
    LoadBearing,
    Terminal,
    Consequence,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RouteNode {
    node_id: String,
    role: NodeRole,
    declaration: String,
    module_path: String,
    dependency_ids: Vec<String>,
    provenance_kind: ProvenanceKind,
    correspondence_kind: CorrespondenceKind,
    source_locator: Option<String>,
    source_archive_sha256: Option<String>,
    source_file_sha256: Option<String>,
    source_excerpt_sha256: Option<String>,
    source_line_count: Option<u64>,
    reused_from_route: Option<RouteId>,
    reused_node_id: Option<String>,
    declaration_type_path: String,
    statement_sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RouteManifest {
    schema_version: String,
    route_id: RouteId,
    claim_kind: ClaimKind,
    aggregate_module: String,
    build_target: String,
    terminal_declaration: String,
    terminal_type_sha256: String,
    consequence_declarations: Vec<String>,
    source_identities: Vec<String>,
    shared_foundation_modules: Vec<String>,
    module_closure: Vec<String>,
    module_closure_sha256: String,
    allowed_axioms: Vec<String>,
    review_path: String,
    review_sha256: Option<String>,
    receipt_path: String,
    receipt_sha256: Option<String>,
    nodes: Vec<RouteNode>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileDigest {
    module: String,
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteProviderReport {
    schema_version: String,
    route_id: RouteId,
    aggregate_module: String,
    modules: Vec<String>,
    module_closure_sha256: String,
    status: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteCommand {
    schema_version: String,
    argv: Vec<String>,
    working_directory: String,
    aggregate_module: String,
    build_target: String,
    cache_identity: String,
    toolchain: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AxiomAudit {
    schema_version: String,
    route_id: RouteId,
    allowed_axioms: Vec<String>,
    results: Vec<AxiomResult>,
    status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DeclarationType {
    declaration: String,
    type_artifact_path: String,
    type_artifact_sha256: String,
    statement_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AxiomResult {
    declaration: String,
    axioms: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RouteReceipt {
    schema_version: String,
    route_id: RouteId,
    aggregate_module: String,
    build_target: String,
    manifest_path: String,
    manifest_sha256: String,
    candidate_commit: String,
    candidate_tree: String,
    command_artifact_path: String,
    command_artifact_sha256: String,
    argv: Vec<String>,
    working_directory: String,
    cache_identity: String,
    toolchain: String,
    local_closure_modules: Vec<String>,
    local_closure_sha256: String,
    mathlib_artifacts: Vec<FileDigest>,
    mathlib_artifacts_sha256: String,
    declaration_types: Vec<DeclarationType>,
    allowed_axioms: Vec<String>,
    axiom_audit_path: String,
    axiom_audit_sha256: String,
    axiom_results: Vec<AxiomResult>,
    provider_report_path: String,
    provider_report_sha256: String,
    stdout_path: String,
    stdout_sha256: String,
    stderr_path: String,
    stderr_sha256: String,
    exit_code: u8,
    status: String,
    receipt_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
enum Severity {
    Critical,
    Important,
    Minor,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Finding {
    finding_id: String,
    severity: Severity,
    resolved: bool,
    locator: String,
    statement: String,
    falsifying_test_or_gap: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProofReview {
    schema_version: String,
    route_id: RouteId,
    reviewed_commit: String,
    reviewed_tree: String,
    manifest_path: String,
    manifest_sha256: String,
    terminal_type_sha256: String,
    review_id: String,
    reviewer_identity: String,
    reviewer_model: String,
    reviewer_run_id: String,
    verdict: String,
    outcome: String,
    source_fidelity_check: String,
    derivation_reuse_check: String,
    findings: Vec<Finding>,
    review_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LsArtifactManifest {
    archive_sha256: String,
    manuscript_bytes: u64,
    manuscript_path: String,
    manuscript_sha256: String,
    line_count: u64,
    schema_version: String,
    source_id: String,
    source_identity: String,
}

#[derive(Debug)]
struct SourceManifestRow<'a> {
    schema_version: &'a str,
    receipt_id: &'a str,
    source_id: &'a str,
    source_class: &'a str,
    role: &'a str,
    immutable_identity: &'a str,
    source_url: &'a str,
    upstream_path: &'a str,
    bytes: &'a str,
    sha256: &'a str,
    local_path: &'a str,
    observed: &'a str,
    license_status: &'a str,
    redistribution_status: &'a str,
}

fn canonical_digest(value: &Value) -> Result<String, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn self_digest(value: &Value, field: &str) -> Result<String, String> {
    let mut normalized = value.clone();
    normalized[field] = Value::Null;
    canonical_digest(&normalized)
}

fn manifest_contract_digest(value: &Value) -> Result<String, String> {
    let mut normalized = value.clone();
    normalized["receipt_sha256"] = Value::Null;
    normalized["review_sha256"] = Value::Null;
    canonical_digest(&normalized)
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_git_id(value: &str) -> bool {
    (40..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_arxiv_identity(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("arxiv:") else {
        return false;
    };
    let Some((year_month, version)) = rest.rsplit_once('v') else {
        return false;
    };
    let Some((year, number)) = year_month.split_once('.') else {
        return false;
    };
    year.len() == 4
        && year.bytes().all(|byte| byte.is_ascii_digit())
        && (4..=5).contains(&number.len())
        && number.bytes().all(|byte| byte.is_ascii_digit())
        && !version.is_empty()
        && version.bytes().all(|byte| byte.is_ascii_digit())
        && !version.starts_with('0')
}

fn safe_path(value: &str) -> bool {
    if value.is_empty() || value.len() > 4096 || value.contains('\\') {
        return false;
    }
    let path = value.split_once('#').map_or(value, |(path, _)| path);
    if path.starts_with('/') || path.starts_with('\\') {
        return false;
    }
    let mut parts = path.split('/');
    let Some(first) = parts.next() else {
        return false;
    };
    if first.is_empty() || first == "." || first == ".." || first.contains(':') {
        return false;
    }
    parts.all(|part| !part.is_empty() && part != "." && part != "..")
}

fn parse_source_locator(value: &str) -> Option<(SourceLocatorKind<'_>, &str, u64, u64)> {
    let (head, span) = value.rsplit_once("#L")?;
    let (start, end) = span.split_once("-L")?;
    let start = start.parse::<u64>().ok()?;
    let end = end.parse::<u64>().ok()?;
    if start == 0 || end == 0 {
        return None;
    }
    if let Some(rest) = head.strip_prefix("git:") {
        let (commit, path) = rest.split_once(':')?;
        if commit.len() != 40 || !valid_git_id(commit) || path.contains('#') || !safe_path(path) {
            return None;
        }
        Some((SourceLocatorKind::Git(commit), path, start, end))
    } else if let Some(rest) = head.strip_prefix("arxiv:") {
        let (version, path) = rest.split_once(':')?;
        let identity = head.strip_suffix(&format!(":{path}"))?;
        if path.contains('#') || !safe_path(path) || !valid_arxiv_identity(identity) {
            return None;
        }
        debug_assert_eq!(identity, format!("arxiv:{version}"));
        Some((SourceLocatorKind::Arxiv(identity), path, start, end))
    } else if safe_path(head) {
        Some((SourceLocatorKind::Local, head, start, end))
    } else {
        None
    }
}

fn validate_source_manifest_row(
    repo_root: &Path,
    expected: SourceManifestRow<'_>,
) -> Result<(), String> {
    let bytes = read_bounded_regular(repo_root, SOURCE_MANIFEST_PATH, "source manifest")?;
    let text = String::from_utf8(bytes).map_err(|_| "source manifest is not UTF-8")?;
    let mut lines = text.lines();
    let header = lines.next().ok_or("source manifest is empty")?;
    let expected_header = "schema_version\treceipt_id\tsource_id\tsource_class\trole\timmutable_identity\tsource_url\tupstream_path\tbytes\tsha256\tlocal_path\tobserved\tlicense_status\tredistribution_status";
    if header != expected_header {
        return Err("source manifest header mismatch".into());
    }
    let rows = lines
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let matching = rows
        .iter()
        .filter(|row| row.len() == 14 && row[1] == expected.receipt_id)
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err(format!(
            "expected exactly one source manifest row for {}",
            expected.receipt_id
        ));
    }
    let row = matching[0];
    let actual = SourceManifestRow {
        schema_version: row[0],
        receipt_id: row[1],
        source_id: row[2],
        source_class: row[3],
        role: row[4],
        immutable_identity: row[5],
        source_url: row[6],
        upstream_path: row[7],
        bytes: row[8],
        sha256: row[9],
        local_path: row[10],
        observed: row[11],
        license_status: row[12],
        redistribution_status: row[13],
    };
    if actual.schema_version != expected.schema_version
        || actual.receipt_id != expected.receipt_id
        || actual.source_id != expected.source_id
        || actual.source_class != expected.source_class
        || actual.role != expected.role
        || actual.immutable_identity != expected.immutable_identity
        || actual.source_url != expected.source_url
        || actual.upstream_path != expected.upstream_path
        || actual.bytes != expected.bytes
        || actual.sha256 != expected.sha256
        || actual.local_path != expected.local_path
        || actual.observed != expected.observed
        || actual.license_status != expected.license_status
        || actual.redistribution_status != expected.redistribution_status
    {
        return Err(format!(
            "source manifest {} row mismatch",
            expected.receipt_id
        ));
    }
    Ok(())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|items| items[0] < items[1])
}

fn module_from_path(value: &str) -> Option<String> {
    let stem = value.strip_suffix(".lean")?;
    Some(stem.replace('/', "."))
}

fn parse<T: for<'de> Deserialize<'de>>(value: &Value, label: &str) -> Result<T, String> {
    serde_json::from_value(value.clone()).map_err(|error| format!("invalid {label}: {error}"))
}

fn validate_manifest(manifest: &RouteManifest) -> Result<(), String> {
    if manifest.schema_version != MANIFEST_SCHEMA {
        return Err("invalid manifest schema".into());
    }
    if manifest.aggregate_module != manifest.route_id.aggregate()
        || manifest.build_target != manifest.aggregate_module
    {
        return Err("route aggregate mismatch".into());
    }
    if !valid_digest(&manifest.terminal_type_sha256)
        || !valid_digest(&manifest.module_closure_sha256)
        || manifest
            .review_sha256
            .as_deref()
            .is_some_and(|value| !valid_digest(value))
        || manifest
            .receipt_sha256
            .as_deref()
            .is_some_and(|value| !valid_digest(value))
    {
        return Err("invalid manifest digest".into());
    }
    if !safe_path(&manifest.review_path) || !safe_path(&manifest.receipt_path) {
        return Err("unsafe manifest path".into());
    }
    if !sorted_unique(&manifest.module_closure)
        || !sorted_unique(&manifest.allowed_axioms)
        || manifest
            .allowed_axioms
            .iter()
            .any(|item| !ALLOWED_AXIOMS.contains(&item.as_str()))
    {
        return Err("manifest rosters must be sorted, unique, and allowed".into());
    }
    let closure_value = serde_json::to_value(&manifest.module_closure).unwrap();
    if canonical_digest(&closure_value)? != manifest.module_closure_sha256 {
        return Err("module closure digest mismatch".into());
    }
    match manifest.route_id {
        RouteId::Jin | RouteId::LoristSchwenninger => {
            if manifest.claim_kind != ClaimKind::SourceFaithful
                || manifest.source_identities.is_empty()
            {
                return Err("source-faithful route requires source identities".into());
            }
            if manifest.source_identities.iter().any(|item| {
                !(item.starts_with("sha256:") && valid_digest(&item[7..]))
                    && !valid_arxiv_identity(item)
            }) {
                return Err("invalid source identity".into());
            }
        }
        RouteId::Harp if manifest.claim_kind != ClaimKind::Derived => {
            return Err("Harp route must be derived".into());
        }
        RouteId::Harp => {}
    }
    let mut ids = BTreeSet::new();
    let mut modules = BTreeSet::new();
    let mut declarations = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut terminal = None;
    let mut harp_reuse = false;
    for node in &manifest.nodes {
        if !ids.insert(node.node_id.as_str()) {
            return Err("duplicate node id".into());
        }
        let module = module_from_path(&node.module_path).ok_or("invalid node module path")?;
        if !safe_path(&node.module_path) || !safe_path(&node.declaration_type_path) {
            return Err("unsafe node path".into());
        }
        if !modules.insert(module) || !declarations.insert(node.declaration.as_str()) {
            return Err("duplicate node module or declaration".into());
        }
        if !valid_digest(&node.statement_sha256) {
            return Err("invalid statement digest".into());
        }
        let source_metadata = [
            node.source_archive_sha256.as_deref(),
            node.source_file_sha256.as_deref(),
            node.source_excerpt_sha256.as_deref(),
        ];
        if source_metadata
            .iter()
            .flatten()
            .any(|digest| !valid_digest(digest))
            || node.source_line_count == Some(0)
        {
            return Err("invalid source metadata".into());
        }
        for dependency in &node.dependency_ids {
            if dependency == &node.node_id {
                return Err("self dependency".into());
            }
            if !seen.contains(dependency.as_str()) {
                return Err("dependency is unknown or violates stable topological order".into());
            }
        }
        match node.provenance_kind {
            ProvenanceKind::Source => {
                if node
                    .source_locator
                    .as_deref()
                    .and_then(parse_source_locator)
                    .is_none()
                    || node.source_file_sha256.is_none()
                    || node.source_excerpt_sha256.is_none()
                    || node.source_line_count.is_none()
                    || node.reused_from_route.is_some()
                    || node.reused_node_id.is_some()
                {
                    return Err("invalid source provenance".into());
                }
            }
            ProvenanceKind::ReusedRoute => {
                if node.source_locator.is_some()
                    || node.source_archive_sha256.is_some()
                    || node.source_file_sha256.is_some()
                    || node.source_excerpt_sha256.is_some()
                    || node.source_line_count.is_some()
                    || node.reused_from_route.is_none()
                    || node.reused_node_id.is_none()
                {
                    return Err("invalid reused-route provenance".into());
                }
                if manifest.route_id == RouteId::Harp {
                    if node.reused_from_route != Some(RouteId::LoristSchwenninger) {
                        return Err("Harp may reuse only Lorist-Schwenninger".into());
                    }
                    harp_reuse = true;
                }
            }
            ProvenanceKind::SharedFoundation | ProvenanceKind::Derived => {
                if node.source_locator.is_some()
                    || node.source_archive_sha256.is_some()
                    || node.source_file_sha256.is_some()
                    || node.source_excerpt_sha256.is_some()
                    || node.source_line_count.is_some()
                    || node.reused_from_route.is_some()
                    || node.reused_node_id.is_some()
                {
                    return Err("invalid non-source provenance".into());
                }
            }
        }
        if !correspondence_matches(node.provenance_kind, node.correspondence_kind) {
            return Err("correspondence kind is incompatible with provenance kind".into());
        }
        if manifest.claim_kind == ClaimKind::SourceFaithful
            && !matches!(
                node.provenance_kind,
                ProvenanceKind::Source | ProvenanceKind::Derived
            )
        {
            return Err("source-faithful route cannot reuse another route".into());
        }
        if node.role == NodeRole::Terminal && terminal.replace(node).is_some() {
            return Err("multiple terminal nodes".into());
        }
        seen.insert(node.node_id.as_str());
    }
    let terminal = terminal.ok_or("missing terminal node")?;
    if manifest.claim_kind == ClaimKind::SourceFaithful
        && !manifest
            .nodes
            .iter()
            .any(|node| node.provenance_kind == ProvenanceKind::Source)
    {
        return Err("source-faithful route requires at least one source node".into());
    }
    if terminal.declaration != manifest.terminal_declaration
        || terminal.statement_sha256 != manifest.terminal_type_sha256
    {
        return Err("terminal declaration/type mismatch".into());
    }
    if manifest
        .consequence_declarations
        .iter()
        .any(|item| !declarations.contains(item.as_str()))
    {
        return Err("unrepresented consequence declaration".into());
    }
    if manifest.route_id == RouteId::Harp && !harp_reuse {
        return Err("Harp route omits explicit LS reuse".into());
    }
    let shared = manifest
        .shared_foundation_modules
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if modules.iter().any(|module| shared.contains(module)) {
        return Err("duplicate shared/node module coverage".into());
    }
    if manifest
        .module_closure
        .iter()
        .any(|module| !modules.contains(module) && !shared.contains(module))
    {
        return Err("unmapped closure module".into());
    }
    Ok(())
}

#[cfg(test)]
fn validate_records(
    manifest_value: &Value,
    receipt_value: &Value,
    review_value: &Value,
) -> Result<(), String> {
    let manifest: RouteManifest = parse(manifest_value, "manifest")?;
    let receipt: RouteReceipt = parse(receipt_value, "receipt")?;
    let review: ProofReview = parse(review_value, "review")?;
    validate_manifest(&manifest)?;
    validate_receipt(manifest_value, &manifest, receipt_value, &receipt)?;
    validate_review(&manifest, &receipt, review_value, &review)
}

fn validate_receipt(
    manifest_value: &Value,
    manifest: &RouteManifest,
    receipt_value: &Value,
    receipt: &RouteReceipt,
) -> Result<(), String> {
    if receipt.schema_version != RECEIPT_SCHEMA {
        return Err("invalid receipt schema".into());
    }
    if receipt.route_id != manifest.route_id
        || receipt.aggregate_module != manifest.aggregate_module
        || receipt.build_target != manifest.build_target
    {
        return Err("receipt aggregate/route mismatch".into());
    }
    if receipt.manifest_sha256 != manifest_contract_digest(manifest_value)? {
        return Err("receipt manifest digest mismatch".into());
    }
    if !valid_git_id(&receipt.candidate_commit) || !valid_git_id(&receipt.candidate_tree) {
        return Err("invalid candidate identity".into());
    }
    if receipt.argv
        != [
            "scripts/check_lean_library.sh",
            manifest.build_target.as_str(),
        ]
        || receipt.working_directory != "."
    {
        return Err("receipt command mismatch".into());
    }
    for path in [
        &receipt.manifest_path,
        &receipt.command_artifact_path,
        &receipt.axiom_audit_path,
        &receipt.provider_report_path,
        &receipt.stdout_path,
        &receipt.stderr_path,
    ] {
        if !safe_path(path) {
            return Err("unsafe receipt path".into());
        }
    }
    if receipt.local_closure_modules != manifest.module_closure
        || receipt.local_closure_sha256 != manifest.module_closure_sha256
    {
        return Err("receipt local closure mismatch".into());
    }
    if receipt.allowed_axioms != manifest.allowed_axioms {
        return Err("receipt allowed axioms mismatch".into());
    }
    let mathlib_value = serde_json::to_value(&receipt.mathlib_artifacts).unwrap();
    if canonical_digest(&mathlib_value)? != receipt.mathlib_artifacts_sha256
        || receipt
            .mathlib_artifacts
            .iter()
            .any(|item| !safe_path(&item.path) || !valid_digest(&item.sha256))
    {
        return Err("Mathlib roster/digest mismatch".into());
    }
    validate_declaration_roster(manifest, receipt)?;
    let nodes = manifest
        .nodes
        .iter()
        .map(|node| (node.declaration.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    for item in &receipt.declaration_types {
        let node = nodes
            .get(item.declaration.as_str())
            .ok_or("unknown declaration type")?;
        if item.type_artifact_path != node.declaration_type_path
            || item.statement_sha256 != node.statement_sha256
            || !safe_path(&item.type_artifact_path)
            || !valid_digest(&item.type_artifact_sha256)
        {
            return Err("declaration type mismatch".into());
        }
    }
    let mut audited = BTreeSet::new();
    for result in &receipt.axiom_results {
        if !audited.insert(result.declaration.as_str())
            || !nodes.contains_key(result.declaration.as_str())
            || !sorted_unique(&result.axioms)
            || result
                .axioms
                .iter()
                .any(|axiom| !manifest.allowed_axioms.contains(axiom))
        {
            return Err("invalid or forbidden axiom result".into());
        }
    }
    if audited.len() != nodes.len() {
        return Err("incomplete axiom audit".into());
    }
    for digest in [
        &receipt.command_artifact_sha256,
        &receipt.axiom_audit_sha256,
        &receipt.provider_report_sha256,
        &receipt.stdout_sha256,
        &receipt.stderr_sha256,
    ] {
        if !valid_digest(digest) {
            return Err("invalid receipt artifact digest".into());
        }
    }
    if receipt.status != "passed" || receipt.exit_code != 0 {
        return Err("receipt status/exit mismatch".into());
    }
    if receipt.receipt_sha256 != self_digest(receipt_value, "receipt_sha256")? {
        return Err("receipt identity digest mismatch".into());
    }
    if !receipt.cache_identity.starts_with("sha256:")
        || !valid_digest(&receipt.cache_identity[7..])
        || receipt.toolchain != "leanprover/lean4:v4.32.1"
    {
        return Err("invalid cache/toolchain identity".into());
    }
    Ok(())
}

fn validate_declaration_roster(
    manifest: &RouteManifest,
    receipt: &RouteReceipt,
) -> Result<(), String> {
    let expected = manifest
        .nodes
        .iter()
        .map(|node| node.declaration.as_str())
        .collect::<BTreeSet<_>>();
    let actual = receipt
        .declaration_types
        .iter()
        .map(|item| item.declaration.as_str())
        .collect::<Vec<_>>();
    if actual.len() != expected.len() || actual.iter().copied().collect::<BTreeSet<_>>() != expected
    {
        return Err("declaration type roster mismatch".into());
    }
    Ok(())
}

fn validate_review(
    manifest: &RouteManifest,
    receipt: &RouteReceipt,
    review_value: &Value,
    review: &ProofReview,
) -> Result<(), String> {
    if review.schema_version != REVIEW_SCHEMA || review.route_id != manifest.route_id {
        return Err("review route/schema mismatch".into());
    }
    if review.reviewed_commit != receipt.candidate_commit
        || review.reviewed_tree != receipt.candidate_tree
    {
        return Err("review commit/tree mismatch".into());
    }
    if review.manifest_path != receipt.manifest_path
        || review.manifest_sha256 != receipt.manifest_sha256
        || review.terminal_type_sha256 != manifest.terminal_type_sha256
    {
        return Err("review manifest/type mismatch".into());
    }
    if !safe_path(&review.manifest_path)
        || [
            &review.review_id,
            &review.reviewer_identity,
            &review.reviewer_model,
            &review.reviewer_run_id,
        ]
        .iter()
        .any(|value| value.is_empty() || value.len() > 16_384)
    {
        return Err("invalid review identity/path".into());
    }
    let checks_match = match manifest.claim_kind {
        ClaimKind::SourceFaithful => {
            review.source_fidelity_check == "passed"
                && review.derivation_reuse_check == "not-applicable"
        }
        ClaimKind::Derived => {
            review.source_fidelity_check == "not-applicable"
                && review.derivation_reuse_check == "passed"
        }
    };
    if !checks_match {
        return Err("review provenance checks mismatch".into());
    }
    let mut finding_ids = BTreeSet::new();
    for finding in &review.findings {
        if !finding_ids.insert(finding.finding_id.as_str())
            || !safe_path(&finding.locator)
            || finding.statement.is_empty()
            || finding.falsifying_test_or_gap.is_empty()
        {
            return Err("invalid review finding".into());
        }
    }
    let unresolved = review.findings.iter().any(|finding| {
        matches!(finding.severity, Severity::Critical | Severity::Important) && !finding.resolved
    });
    if review.verdict == "complete" && unresolved {
        return Err("complete review has unresolved Critical/Important finding".into());
    }
    if review.verdict != "complete" || review.outcome != "approved" {
        return Err("review is not complete and approved".into());
    }
    if review.review_sha256 != self_digest(review_value, "review_sha256")? {
        return Err("review identity digest mismatch".into());
    }
    Ok(())
}

pub(super) fn verify_published_routes(repo_root: &Path) -> Result<BTreeSet<String>, String> {
    let mut route_evidence = BTreeSet::new();
    for manifest_path in ROUTE_MANIFESTS {
        match fs::symlink_metadata(repo_root.join(manifest_path)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("cannot inspect route manifest: {error}")),
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(format!(
                    "route manifest is not a regular file: {manifest_path}"
                ));
            }
            Ok(_) => {}
        }
        let manifest_value = read_json_value(repo_root, manifest_path, "route manifest")?;
        let manifest: RouteManifest = parse(&manifest_value, "manifest")?;
        validate_manifest(&manifest)?;
        let closure =
            active_route_closure(repo_root, manifest.route_id, &manifest.aggregate_module)?;
        if closure != manifest.module_closure
            || canonical_digest(&serde_json::to_value(&closure).unwrap())?
                != manifest.module_closure_sha256
        {
            return Err("route manifest does not match active closure".into());
        }
        validate_manifest_against_closure(&manifest, &closure)?;
        if manifest.route_id == RouteId::Harp {
            let ls_value = read_json_value(repo_root, ROUTE_MANIFESTS[1], "LS route manifest")
                .map_err(|error| format!("missing or invalid LS route manifest: {error}"))?;
            let ls_manifest: RouteManifest = parse(&ls_value, "LS route manifest")?;
            validate_harp_reuse(&manifest, &closure, &ls_manifest)?;
        }
        validate_manifest_files(repo_root, &manifest)?;
        for node in &manifest.nodes {
            if let Some(relative) = crouzeix_evidence_member(&node.declaration_type_path) {
                route_evidence.insert(relative);
            }
            if let Some(locator) = node.source_locator.as_deref() {
                let (kind, path, _, _) = parse_source_locator(locator)
                    .expect("source locator was validated with the route manifest");
                if !matches!(kind, SourceLocatorKind::Git(_)) {
                    if let Some(relative) = crouzeix_evidence_member(path) {
                        route_evidence.insert(relative);
                    }
                }
            }
        }
        let receipt_exists = artifact_state(repo_root, &manifest.receipt_path, "route receipt")?;
        let review_exists = artifact_state(repo_root, &manifest.review_path, "proof review")?;
        if review_exists && !receipt_exists {
            return Err("review publication requires a receipt".into());
        }
        if receipt_exists != manifest.receipt_sha256.is_some() {
            return Err("receipt publication presence/digest mismatch".into());
        }
        if review_exists != manifest.review_sha256.is_some() {
            return Err("review publication presence/digest mismatch".into());
        }
        if receipt_exists {
            verify_file_digest(
                repo_root,
                &manifest.receipt_path,
                manifest.receipt_sha256.as_deref().unwrap(),
                "route receipt",
            )?;
            let receipt_value =
                read_json_value(repo_root, &manifest.receipt_path, "route receipt")?;
            let receipt: RouteReceipt = parse(&receipt_value, "receipt")?;
            validate_receipt(&manifest_value, &manifest, &receipt_value, &receipt)?;
            validate_git_identity(
                repo_root,
                &receipt.candidate_commit,
                &receipt.candidate_tree,
                &closure,
            )?;
            verify_bound_files(repo_root, &manifest, &receipt, &closure)?;
            for path in [
                &manifest.receipt_path,
                &receipt.command_artifact_path,
                &receipt.axiom_audit_path,
                &receipt.provider_report_path,
                &receipt.stdout_path,
                &receipt.stderr_path,
            ] {
                route_evidence.insert(crouzeix_evidence_relative(path)?);
            }
            if review_exists {
                verify_file_digest(
                    repo_root,
                    &manifest.review_path,
                    manifest.review_sha256.as_deref().unwrap(),
                    "proof review",
                )?;
                let review_value =
                    read_json_value(repo_root, &manifest.review_path, "proof review")?;
                let review: ProofReview = parse(&review_value, "review")?;
                validate_review(&manifest, &receipt, &review_value, &review)?;
                route_evidence.insert(crouzeix_evidence_relative(&manifest.review_path)?);
            }
        }
    }
    Ok(route_evidence)
}

fn crouzeix_evidence_relative(path: &str) -> Result<String, String> {
    crouzeix_evidence_member(path)
        .ok_or_else(|| "route publication path must stay beneath Crouzeix evidence root".to_owned())
}

fn crouzeix_evidence_member(path: &str) -> Option<String> {
    Path::new(path)
        .strip_prefix(super::ROOT)
        .ok()
        .map(super::slash_path)
}

fn artifact_state(repo_root: &Path, relative: &str, label: &str) -> Result<bool, String> {
    match fs::symlink_metadata(repo_root.join(relative)) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!("cannot inspect {label}: {error}")),
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(format!("{label} must be a regular non-symlink file"))
        }
        Ok(_) => Ok(true),
    }
}

fn validate_manifest_files(repo_root: &Path, manifest: &RouteManifest) -> Result<(), String> {
    if manifest.route_id == RouteId::Harp && !manifest.source_identities.is_empty() {
        return Err("Harp source identities must be empty".into());
    }
    for node in &manifest.nodes {
        validate_source_provenance(repo_root, manifest, node)?;
        let bytes = read_bounded_regular(
            repo_root,
            &node.declaration_type_path,
            "declaration type artifact",
        )?;
        let text =
            String::from_utf8(bytes).map_err(|_| "declaration type artifact is not UTF-8")?;
        let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.is_empty()
            || format!("{:x}", Sha256::digest(normalized.as_bytes())) != node.statement_sha256
        {
            return Err("declaration type digest mismatch".into());
        }
    }
    Ok(())
}

fn verify_bound_files(
    repo_root: &Path,
    manifest: &RouteManifest,
    receipt: &RouteReceipt,
    closure: &[String],
) -> Result<(), String> {
    let command: RouteCommand = read_digest_bound_json(
        repo_root,
        &receipt.command_artifact_path,
        &receipt.command_artifact_sha256,
        "command artifact",
    )?;
    let expected_cache = cache_contract_identity(repo_root)?;
    if command.schema_version != "crouzeix-route-command/v1"
        || command.argv != receipt.argv
        || command.working_directory != "."
        || command.working_directory != receipt.working_directory
        || command.aggregate_module != manifest.aggregate_module
        || command.aggregate_module != receipt.aggregate_module
        || command.build_target != manifest.build_target
        || command.build_target != receipt.build_target
        || command.cache_identity != expected_cache
        || command.cache_identity != receipt.cache_identity
        || command.toolchain != "leanprover/lean4:v4.32.1"
        || command.toolchain != receipt.toolchain
    {
        return Err("command artifact invocation contract mismatch".into());
    }
    let audit: AxiomAudit = read_digest_bound_json(
        repo_root,
        &receipt.axiom_audit_path,
        &receipt.axiom_audit_sha256,
        "axiom audit",
    )?;
    if audit.schema_version != "crouzeix-route-axiom-audit/v1"
        || audit.route_id != manifest.route_id
        || audit.allowed_axioms != receipt.allowed_axioms
        || audit.results != receipt.axiom_results
        || audit.status != "passed"
    {
        return Err("axiom audit semantic mismatch".into());
    }
    let provider: RouteProviderReport = read_digest_bound_json(
        repo_root,
        &receipt.provider_report_path,
        &receipt.provider_report_sha256,
        "provider report",
    )?;
    if provider.schema_version != "crouzeix-route-provider-report/v1"
        || provider.route_id != manifest.route_id
        || provider.aggregate_module != manifest.aggregate_module
        || provider.modules != closure
        || provider.module_closure_sha256 != manifest.module_closure_sha256
        || provider.status != "passed"
    {
        return Err("provider report does not match active closure".into());
    }
    for node in &manifest.nodes {
        validate_source_provenance(repo_root, manifest, node)?;
        let bytes = read_bounded_regular(
            repo_root,
            &node.declaration_type_path,
            "declaration type artifact",
        )?;
        let text =
            String::from_utf8(bytes).map_err(|_| "declaration type artifact is not UTF-8")?;
        let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.is_empty()
            || format!("{:x}", Sha256::digest(normalized.as_bytes())) != node.statement_sha256
        {
            return Err(format!(
                "declaration type digest mismatch: {}",
                node.declaration
            ));
        }
    }
    for (path, digest, label) in [
        (&receipt.stdout_path, &receipt.stdout_sha256, "stdout"),
        (&receipt.stderr_path, &receipt.stderr_sha256, "stderr"),
    ] {
        verify_file_digest(repo_root, path, digest, label)?;
    }
    let expected_mathlib = active_mathlib_closure(repo_root, closure)?;
    validate_mathlib_roster(repo_root, receipt, &expected_mathlib)?;
    for item in &receipt.declaration_types {
        verify_file_digest(
            repo_root,
            &item.type_artifact_path,
            &item.type_artifact_sha256,
            "declaration type artifact",
        )?;
    }
    Ok(())
}

fn validate_mathlib_roster(
    repo_root: &Path,
    receipt: &RouteReceipt,
    expected_modules: &[String],
) -> Result<(), String> {
    if !sorted_unique(expected_modules) {
        return Err("Mathlib closure is not sorted and unique".into());
    }
    let expected = expected_modules
        .iter()
        .map(|module| {
            (
                module.as_str(),
                format!(
                    "formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean/{}.olean",
                    module.replace('.', "/")
                ),
            )
        })
        .collect::<Vec<_>>();
    if receipt.mathlib_artifacts.len() != expected.len() {
        return Err("Mathlib artifact roster does not match transitive closure".into());
    }
    let cache_root = resolve_approved_cache_root(repo_root)?;
    for (item, (module, path)) in receipt.mathlib_artifacts.iter().zip(expected) {
        if item.module != module || item.path != path {
            return Err("Mathlib artifact module/path mismatch".into());
        }
        let actual = format!(
            "{:x}",
            Sha256::digest(read_cache_mathlib_artifact(&cache_root, module)?)
        );
        if actual != item.sha256 {
            return Err(format!("Mathlib artifact digest mismatch: {module}"));
        }
    }
    Ok(())
}

fn validate_git_identity(
    repo_root: &Path,
    commit: &str,
    expected_tree: &str,
    closure: &[String],
) -> Result<(), String> {
    let exists = Command::new("git")
        .args([
            "-C",
            repo_root.to_str().ok_or("repository path is not UTF-8")?,
            "cat-file",
            "-e",
            &format!("{commit}^{{commit}}"),
        ])
        .output()
        .map_err(|error| format!("Git identity validation unavailable: {error}"))?;
    if !exists.status.success() {
        return Err("candidate Git commit is unreachable".into());
    }
    let ancestor = Command::new("git")
        .args([
            "-C",
            repo_root.to_str().ok_or("repository path is not UTF-8")?,
            "merge-base",
            "--is-ancestor",
            commit,
            "HEAD",
        ])
        .output()
        .map_err(|error| format!("Git identity validation unavailable: {error}"))?;
    if !ancestor.status.success() {
        return Err("candidate Git commit is not an ancestor of HEAD".into());
    }
    let tree = Command::new("git")
        .args([
            "-C",
            repo_root.to_str().ok_or("repository path is not UTF-8")?,
            "rev-parse",
            &format!("{commit}^{{tree}}"),
        ])
        .output()
        .map_err(|error| format!("Git identity validation unavailable: {error}"))?;
    if !tree.status.success() || String::from_utf8_lossy(&tree.stdout).trim() != expected_tree {
        return Err("candidate Git tree mismatch".into());
    }
    for module in closure {
        let relative = format!("formalization/lean/{}.lean", module.replace('.', "/"));
        let current = read_bounded_regular(repo_root, &relative, "route source")?;
        let candidate = Command::new("git")
            .args([
                "-C",
                repo_root.to_str().ok_or("repository path is not UTF-8")?,
                "show",
                &format!("{commit}:{relative}"),
            ])
            .output()
            .map_err(|error| format!("Git source binding unavailable: {error}"))?;
        if !candidate.status.success() || candidate.stdout != current {
            return Err(format!("candidate route source blob mismatch: {relative}"));
        }
    }
    Ok(())
}

fn active_route_closure(
    repo_root: &Path,
    route: RouteId,
    aggregate: &str,
) -> Result<Vec<String>, String> {
    let root = format!("formalization/lean/{}.lean", aggregate.replace('.', "/"));
    let closure = super::local_source_closure(
        repo_root,
        &[root.as_str()],
        Path::new("route-manifest.json"),
        1,
    )
    .map_err(|error| error.message)?;
    let modules = closure
        .sources
        .into_iter()
        .map(|(module, _)| module)
        .collect::<Vec<_>>();
    if let Some(forbidden) = modules
        .iter()
        .find(|module| forbidden_provider(route, module))
    {
        return Err(format!("forbidden provider module: {forbidden}"));
    }
    Ok(modules)
}

fn forbidden_provider(route: RouteId, module: &str) -> bool {
    match route {
        RouteId::Jin => {
            matches!(
                module,
                "Crouzeix" | "CrouzeixLoristSchwenninger" | "CrouzeixHarp"
            ) || module.starts_with("Crouzeix.LoristSchwenninger")
                || module.starts_with("Crouzeix.Harp")
        }
        RouteId::LoristSchwenninger => {
            matches!(module, "Crouzeix" | "CrouzeixJin" | "CrouzeixHarp")
                || module.starts_with("Crouzeix.Jin")
                || module.starts_with("Crouzeix.Harp")
        }
        RouteId::Harp => {
            matches!(
                module,
                "Crouzeix" | "CrouzeixJin" | "CrouzeixLoristSchwenninger"
            ) || module.starts_with("Crouzeix.Jin")
                || (module.starts_with("Crouzeix.LoristSchwenninger")
                    && !HARP_ALLOWED_LS_SUPPORT.contains(&module))
        }
    }
}

fn active_mathlib_closure(
    repo_root: &Path,
    local_modules: &[String],
) -> Result<Vec<String>, String> {
    let manifest = Path::new("route-manifest.json");
    let cache_root = resolve_approved_cache_root(repo_root)?;
    let mut pending = Vec::new();
    for module in local_modules {
        let relative = format!("formalization/lean/{}.lean", module.replace('.', "/"));
        let source = String::from_utf8(read_bounded_regular(
            repo_root,
            &relative,
            "route Lean module",
        )?)
        .map_err(|_| "route Lean module is not UTF-8")?;
        let active =
            super::mask_lean_source(&source, module, manifest, 1).map_err(|error| error.message)?;
        pending.extend(
            super::lean_header_imports(&active, module, manifest, 1)
                .map_err(|error| error.message)?
                .into_iter()
                .filter(|item| item == "Mathlib" || item.starts_with("Mathlib.")),
        );
    }
    let mut visited = BTreeSet::new();
    while let Some(module) = pending.pop() {
        if !visited.insert(module.clone()) {
            continue;
        }
        let source = String::from_utf8(read_cache_mathlib_source(&cache_root, &module)?)
            .map_err(|_| "Mathlib source is not UTF-8")?;
        let active = super::mask_lean_source(&source, &module, manifest, 1)
            .map_err(|error| error.message)?;
        pending.extend(
            super::lean_header_imports(&active, &module, manifest, 1)
                .map_err(|error| error.message)?
                .into_iter()
                .filter(|item| item == "Mathlib" || item.starts_with("Mathlib.")),
        );
    }
    Ok(visited.into_iter().collect())
}

fn resolve_approved_cache_root(repo_root: &Path) -> Result<std::path::PathBuf, String> {
    let repo_text = repo_root.to_str().ok_or("repository path is not UTF-8")?;
    let output = Command::new("git")
        .args([
            "-C",
            repo_text,
            "rev-parse",
            "--path-format=absolute",
            "--git-common-dir",
        ])
        .output()
        .map_err(|error| format!("approved primary cache lookup unavailable: {error}"))?;
    if !output.status.success() {
        return Err("approved primary cache lookup failed".into());
    }
    let common = fs::canonicalize(String::from_utf8_lossy(&output.stdout).trim())
        .map_err(|error| format!("cannot resolve Git common-dir: {error}"))?;
    if common.file_name().and_then(|name| name.to_str()) != Some(".git") {
        return Err("Git common-dir is not .git".into());
    }
    let primary = common.parent().ok_or("Git common-dir lacks parent")?;
    let approved = primary.join("formalization/lean/.lake");
    let worktree = repo_root.join("formalization/lean/.lake");
    if repo_root
        .canonicalize()
        .map_err(|error| error.to_string())?
        == primary.canonicalize().map_err(|error| error.to_string())?
    {
        let metadata = fs::symlink_metadata(&worktree).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err("primary cache must be a real directory".into());
        }
    } else {
        let metadata = fs::symlink_metadata(&worktree).map_err(|error| error.to_string())?;
        if !metadata.file_type().is_symlink()
            || worktree.canonicalize().map_err(|error| error.to_string())?
                != approved.canonicalize().map_err(|error| error.to_string())?
        {
            return Err("worktree .lake does not target approved primary cache".into());
        }
    }
    let approved = approved.canonicalize().map_err(|error| error.to_string())?;
    for relative in ["packages", "packages/mathlib"] {
        if fs::symlink_metadata(approved.join(relative))
            .map_err(|error| error.to_string())?
            .file_type()
            .is_symlink()
        {
            return Err(format!(
                "approved cache contains nested symlink: {relative}"
            ));
        }
    }
    Ok(approved)
}

fn cache_contract_identity(repo_root: &Path) -> Result<String, String> {
    let toolchain = String::from_utf8(read_bounded_regular(
        repo_root,
        "formalization/lean/lean-toolchain",
        "lean-toolchain",
    )?)
    .map_err(|_| "lean-toolchain is not UTF-8")?;
    if toolchain.trim() != "leanprover/lean4:v4.32.1" {
        return Err("toolchain does not match pinned route toolchain".into());
    }
    let manifest = read_json_value(
        repo_root,
        "formalization/lean/lake-manifest.json",
        "lake manifest",
    )?;
    let payload = serde_json::json!({
        "schema_version": "crouzeix-route-cache-identity/v1",
        "toolchain": "leanprover/lean4:v4.32.1",
        "lake_manifest_sha256": canonical_digest(&manifest)?,
    });
    Ok(format!("sha256:{}", canonical_digest(&payload)?))
}

fn mathlib_source_cache_relative(module: &str) -> Result<String, String> {
    if module.is_empty()
        || module.split('.').any(|part| {
            part.is_empty()
                || !part
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '\'')
        })
    {
        return Err(format!("invalid Mathlib module: {module}"));
    }
    Ok(if module == "Mathlib" {
        "packages/mathlib/Mathlib.lean".to_owned()
    } else {
        format!("packages/mathlib/{}.lean", module.replace('.', "/"))
    })
}

fn mathlib_artifact_cache_relative(module: &str) -> Result<String, String> {
    if module.is_empty()
        || module.split('.').any(|part| {
            part.is_empty()
                || !part
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '\'')
        })
    {
        return Err(format!("invalid Mathlib module: {module}"));
    }
    Ok(format!(
        "packages/mathlib/.lake/build/lib/lean/{}.olean",
        module.replace('.', "/")
    ))
}

fn read_cache_mathlib_source(cache_root: &Path, module: &str) -> Result<Vec<u8>, String> {
    let relative = mathlib_source_cache_relative(module)?;
    read_openat(
        cache_root,
        &relative,
        &format!("Mathlib source {module}"),
        MAX_ARTIFACT_BYTES,
    )
}

fn read_cache_regular(cache_root: &Path, relative: &str, label: &str) -> Result<Vec<u8>, String> {
    let expected_prefix = Path::new("packages/mathlib/.lake/build/lib/lean");
    let path = Path::new(relative);
    if !path.starts_with(expected_prefix)
        || path.extension().and_then(|ext| ext.to_str()) != Some("olean")
    {
        return Err(format!(
            "{label} must use a validated Mathlib artifact path"
        ));
    }
    read_openat(cache_root, relative, label, MAX_CACHE_ARTIFACT_BYTES)
}

fn read_cache_mathlib_artifact(cache_root: &Path, module: &str) -> Result<Vec<u8>, String> {
    let relative = mathlib_artifact_cache_relative(module)?;
    read_cache_regular(cache_root, &relative, "Mathlib artifact")
}

fn validate_source_provenance(
    repo_root: &Path,
    manifest: &RouteManifest,
    node: &RouteNode,
) -> Result<(), String> {
    let metadata_present = node.source_archive_sha256.is_some()
        || node.source_file_sha256.is_some()
        || node.source_excerpt_sha256.is_some()
        || node.source_line_count.is_some();
    if node.provenance_kind != ProvenanceKind::Source {
        return if node.source_locator.is_none() && !metadata_present {
            Ok(())
        } else {
            Err("non-source provenance carries source metadata".into())
        };
    }
    let locator = node
        .source_locator
        .as_deref()
        .ok_or("source locator is missing")?;
    let (kind, path, start, end) = parse_source_locator(locator).ok_or("invalid source locator")?;
    if start > end {
        return Err("reversed source locator span".into());
    }
    let file_sha = node
        .source_file_sha256
        .as_deref()
        .ok_or("source file digest is missing")?;
    let excerpt_sha = node
        .source_excerpt_sha256
        .as_deref()
        .ok_or("source excerpt digest is missing")?;
    let line_count = node
        .source_line_count
        .ok_or("source line count is missing")?;
    if end > line_count {
        return Err("source locator exceeds declared line count".into());
    }
    match kind {
        SourceLocatorKind::Git(commit) => {
            let archive_sha = node
                .source_archive_sha256
                .as_deref()
                .ok_or("Git source locator requires archive digest")?;
            if !manifest
                .source_identities
                .contains(&format!("sha256:{archive_sha}"))
            {
                return Err("source archive identity digest mismatch".into());
            }
            if manifest.route_id != RouteId::Jin {
                return Ok(());
            }
            let artifact_path =
                "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/artifact-manifest.json";
            let artifact = read_json_value(repo_root, artifact_path, "Jin artifact manifest")?;
            if artifact["source_commit"].as_str() != Some(commit)
                || artifact["archive"]["sha256"].as_str() != Some(archive_sha)
            {
                return Err("Jin source commit/archive identity mismatch".into());
            }
            if let Some(artifacts) = artifact["artifacts"].as_object() {
                let matching = artifacts
                    .values()
                    .filter(|record| record["path"].as_str() == Some(path))
                    .collect::<Vec<_>>();
                if matching.len() != 1 {
                    return Err("exactly one artifact record must match Git source path".into());
                }
                if matching[0]["sha256"].as_str() != Some(file_sha) {
                    return Err("Jin source file digest mismatch".into());
                }
            } else {
                return Err("Jin artifact manifest artifacts must be an object".into());
            }
            return Ok(());
        }
        SourceLocatorKind::Arxiv(identity) => {
            let archive_sha = node
                .source_archive_sha256
                .as_deref()
                .ok_or("arXiv source locator requires archive digest")?;
            if !manifest
                .source_identities
                .contains(&format!("sha256:{archive_sha}"))
            {
                return Err("source archive identity digest mismatch".into());
            }
            if !manifest
                .source_identities
                .contains(&format!("sha256:{file_sha}"))
            {
                return Err("source identity digest mismatch".into());
            }
            if !manifest.source_identities.contains(&identity.to_string()) {
                return Err("arXiv source identity mismatch".into());
            }
            if manifest.route_id != RouteId::LoristSchwenninger {
                return Ok(());
            }
            if manifest.source_identities
                != LS_SOURCE_IDENTITIES
                    .iter()
                    .map(|item| item.to_string())
                    .collect::<Vec<_>>()
            {
                return Err("LS route manifest source identities must be exact".into());
            }
            let artifact: LsArtifactManifest = read_digest_bound_json_with(
                "LS artifact manifest",
                &format!(
                    "{:x}",
                    Sha256::digest(read_bounded_regular(
                        repo_root,
                        LS_ARTIFACT_MANIFEST_PATH,
                        "LS artifact manifest",
                    )?)
                ),
                || {
                    read_bounded_regular(
                        repo_root,
                        LS_ARTIFACT_MANIFEST_PATH,
                        "LS artifact manifest",
                    )
                },
            )?;
            if artifact.schema_version != "crouzeix-arxiv-artifact-manifest/v1"
                || artifact.source_id != LS_SOURCE_ID
                || artifact.source_identity != LS_SOURCE_IDENTITY
                || artifact.archive_sha256 != LS_SOURCE_ARCHIVE_SHA256
                || artifact.manuscript_sha256 != LS_SOURCE_FILE_SHA256
                || artifact.manuscript_path != LS_SOURCE_PATH
                || artifact.manuscript_bytes != LS_SOURCE_BYTES
                || artifact.line_count != LS_SOURCE_LINES
            {
                return Err("LS artifact manifest mismatch".into());
            }
            if identity != artifact.source_identity
                || archive_sha != artifact.archive_sha256
                || file_sha != artifact.manuscript_sha256
                || path != artifact.manuscript_path
                || line_count != artifact.line_count
            {
                return Err("LS source provenance does not match artifact manifest".into());
            }
            validate_source_manifest_row(
                repo_root,
                SourceManifestRow {
                    schema_version: "crouzeix-source-receipt/v1",
                    receipt_id: "LS-ARXIV-V1-SOURCE-ARCHIVE",
                    source_id: LS_SOURCE_ID,
                    source_class: "arxiv-artifact",
                    role: "source",
                    immutable_identity: LS_SOURCE_IDENTITY,
                    source_url: "https://export.arxiv.org/e-print/2608.03841v1",
                    upstream_path: "source.tar.gz",
                    bytes: "7330",
                    sha256: LS_SOURCE_ARCHIVE_SHA256,
                    local_path: "-",
                    observed: "2026-08-14",
                    license_status: "arxiv-nonexclusive",
                    redistribution_status: "quotation-only",
                },
            )?;
            validate_source_manifest_row(
                repo_root,
                SourceManifestRow {
                    schema_version: "crouzeix-source-receipt/v1",
                    receipt_id: "LS-ARXIV-V1-TEX",
                    source_id: LS_SOURCE_ID,
                    source_class: "arxiv-artifact",
                    role: "manuscript",
                    immutable_identity: LS_SOURCE_IDENTITY,
                    source_url: "https://export.arxiv.org/e-print/2608.03841v1",
                    upstream_path: LS_SOURCE_PATH,
                    bytes: "18783",
                    sha256: LS_SOURCE_FILE_SHA256,
                    local_path: "-",
                    observed: "2026-08-14",
                    license_status: "arxiv-nonexclusive",
                    redistribution_status: "quotation-only",
                },
            )?;
            return Ok(());
        }
        SourceLocatorKind::Local => {}
    }
    if node.source_archive_sha256.is_some() {
        return Err("local source locator cannot declare an archive digest".into());
    }
    let bytes = read_bounded_regular(repo_root, path, "source locator")?;
    let text = String::from_utf8(bytes.clone()).map_err(|_| "source locator is not UTF-8")?;
    let lines = text.split_inclusive('\n').collect::<Vec<_>>();
    let observed_line_count = text.lines().count() as u64;
    if observed_line_count != line_count {
        return Err("source line count mismatch".into());
    }
    if format!("{:x}", Sha256::digest(&bytes)) != file_sha {
        return Err("source file digest mismatch".into());
    }
    let excerpt = lines[(start - 1) as usize..end as usize].concat();
    if format!("{:x}", Sha256::digest(excerpt.as_bytes())) != excerpt_sha {
        return Err("source excerpt digest mismatch".into());
    }
    let identity = format!("sha256:{file_sha}");
    if !manifest.source_identities.contains(&identity) {
        return Err("source identity digest mismatch".into());
    }
    Ok(())
}

fn validate_manifest_against_closure(
    manifest: &RouteManifest,
    closure: &[String],
) -> Result<(), String> {
    let node_modules = manifest
        .nodes
        .iter()
        .map(|node| {
            let module =
                module_from_path(&node.module_path).ok_or("invalid canonical node module path")?;
            if node.module_path != format!("{}.lean", module.replace('.', "/")) {
                return Err("noncanonical node module path");
            }
            Ok(module)
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let shared = manifest
        .shared_foundation_modules
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if !node_modules.is_disjoint(&shared) {
        return Err("duplicate node/shared closure coverage".into());
    }
    let covered = node_modules
        .union(&shared)
        .cloned()
        .collect::<BTreeSet<_>>();
    if covered != closure.iter().cloned().collect() {
        return Err("node/shared coverage differs from active closure".into());
    }
    let consequences = manifest
        .nodes
        .iter()
        .filter(|node| node.role == NodeRole::Consequence)
        .map(|node| node.declaration.as_str())
        .collect::<BTreeSet<_>>();
    let declared = manifest
        .consequence_declarations
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if consequences != declared {
        return Err("consequence role/declaration mismatch".into());
    }
    Ok(())
}

fn validate_harp_reuse(
    manifest: &RouteManifest,
    closure: &[String],
    ls: &RouteManifest,
) -> Result<(), String> {
    let active = closure
        .iter()
        .filter(|module| HARP_ALLOWED_LS_SUPPORT.contains(&module.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>();
    let reused = manifest
        .nodes
        .iter()
        .filter(|node| node.provenance_kind == ProvenanceKind::ReusedRoute)
        .collect::<Vec<_>>();
    let reused_modules = reused
        .iter()
        .map(|node| module_from_path(&node.module_path).ok_or("invalid reuse module path"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if active != reused_modules {
        return Err("Harp LS reuse coverage mismatch".into());
    }
    let ls_nodes = ls
        .nodes
        .iter()
        .map(|node| (node.node_id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    for node in reused {
        if node.reused_from_route != Some(RouteId::LoristSchwenninger) {
            return Err("Harp reuse names wrong route".into());
        }
        let referenced = ls_nodes
            .get(node.reused_node_id.as_deref().unwrap_or(""))
            .ok_or("dangling Harp LS reuse")?;
        if referenced.module_path != node.module_path
            || referenced.declaration != node.declaration
            || referenced.statement_sha256 != node.statement_sha256
        {
            return Err("Harp LS reuse mismatch".into());
        }
    }
    Ok(())
}

fn verify_file_digest(
    repo_root: &Path,
    path: &str,
    expected: &str,
    label: &str,
) -> Result<(), String> {
    let actual = format!(
        "{:x}",
        Sha256::digest(read_bounded_regular(repo_root, path, label)?)
    );
    if actual != expected {
        return Err(format!("{label} digest mismatch: {path}"));
    }
    Ok(())
}

fn read_digest_bound_json<T: DeserializeOwned>(
    repo_root: &Path,
    path: &str,
    expected: &str,
    label: &str,
) -> Result<T, String> {
    read_digest_bound_json_with(label, expected, || {
        read_bounded_regular(repo_root, path, label)
    })
}

fn read_digest_bound_json_with<T, F>(label: &str, expected: &str, reader: F) -> Result<T, String>
where
    T: DeserializeOwned,
    F: FnOnce() -> Result<Vec<u8>, String>,
{
    let bytes = reader()?;
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual != expected {
        return Err(format!("{label} digest mismatch"));
    }
    let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
    let value =
        T::deserialize(&mut deserializer).map_err(|error| format!("invalid {label}: {error}"))?;
    deserializer
        .end()
        .map_err(|error| format!("invalid {label}: {error}"))?;
    Ok(value)
}

fn read_json_value(repo_root: &Path, path: &str, label: &str) -> Result<Value, String> {
    let bytes = read_bounded_regular(repo_root, path, label)?;
    let mut deserializer = serde_json::Deserializer::from_slice(&bytes);
    let StrictValue(value) = StrictValue::deserialize(&mut deserializer)
        .map_err(|error| format!("invalid {label} JSON: {error}"))?;
    deserializer
        .end()
        .map_err(|error| format!("invalid {label} JSON: {error}"))?;
    Ok(value)
}

fn read_bounded_regular(repo_root: &Path, relative: &str, label: &str) -> Result<Vec<u8>, String> {
    if !safe_path(relative) {
        return Err(format!("unsafe {label} path: {relative}"));
    }
    read_openat(repo_root, relative, label, MAX_ARTIFACT_BYTES)
}

fn read_openat(
    root: &Path,
    relative: &str,
    label: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("cannot resolve {label} root: {error}"))?;
    let mut directory: OwnedFd = File::open(&root)
        .map_err(|error| format!("cannot open {label} root: {error}"))?
        .into();
    let parts = Path::new(relative).components().collect::<Vec<_>>();
    if parts.is_empty()
        || !parts
            .iter()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(format!("unsafe {label} path: {relative}"));
    }
    for part in &parts[..parts.len() - 1] {
        let Component::Normal(name) = part else {
            unreachable!()
        };
        let name = CString::new(name.as_bytes()).map_err(|_| format!("invalid {label} path"))?;
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(format!(
                "cannot open {label} without following symlinks: {}",
                std::io::Error::last_os_error()
            ));
        }
        directory = unsafe { OwnedFd::from_raw_fd(fd) };
    }
    let Component::Normal(leaf) = parts.last().unwrap() else {
        unreachable!()
    };
    let leaf = CString::new(leaf.as_bytes()).map_err(|_| format!("invalid {label} path"))?;
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            leaf.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(format!(
            "cannot open {label} without following symlinks: {}",
            std::io::Error::last_os_error()
        ));
    }
    let file = unsafe { File::from_raw_fd(fd) };
    let metadata = file
        .metadata()
        .map_err(|error| format!("cannot inspect opened {label}: {error}"))?;
    use std::os::unix::fs::MetadataExt;
    if metadata.nlink() != 1 {
        return Err(format!("{label} cannot be a hardlink alias"));
    }
    if !metadata.is_file() || metadata.len() > max_bytes {
        return Err(format!("{label} is not a bounded unique regular file"));
    }
    let mut bytes = Vec::new();
    file.take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read {label}: {error}"))?;
    if bytes.len() as u64 > max_bytes {
        return Err(format!("{label} exceeds byte bound"));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use serde_json::{json, Value};
    use sha2::{Digest as _, Sha256};
    use tempfile::TempDir;

    use super::{
        active_mathlib_closure, active_route_closure, cache_contract_identity,
        correspondence_matches, parse, parse_source_locator, read_bounded_regular, read_json_value,
        validate_declaration_roster, validate_harp_reuse, validate_manifest_against_closure,
        validate_mathlib_roster, validate_records, validate_source_provenance,
        verify_published_routes, ClaimKind, CorrespondenceKind, NodeRole, ProvenanceKind,
        RouteCommand, RouteId, RouteManifest, RouteNode, RouteReceipt, SourceLocatorKind,
        LS_ARTIFACT_MANIFEST_PATH, LS_SOURCE_ARCHIVE_SHA256, LS_SOURCE_BYTES,
        LS_SOURCE_FILE_SHA256, LS_SOURCE_ID, LS_SOURCE_IDENTITY, LS_SOURCE_LINES, LS_SOURCE_PATH,
        MANIFEST_SCHEMA, SOURCE_MANIFEST_PATH,
    };

    fn digest(value: &Value) -> String {
        let mut bytes = serde_json::to_vec(value).unwrap();
        bytes.push(b'\n');
        format!("{:x}", Sha256::digest(bytes))
    }

    fn self_digest(value: &Value, field: &str) -> String {
        let mut normalized = value.clone();
        normalized[field] = Value::Null;
        digest(&normalized)
    }

    fn manifest_digest(value: &Value) -> String {
        let mut normalized = value.clone();
        normalized["receipt_sha256"] = Value::Null;
        normalized["review_sha256"] = Value::Null;
        digest(&normalized)
    }

    fn fixture() -> (Value, Value, Value) {
        let closure = json!([
            "Crouzeix.Jin.Base",
            "Crouzeix.Jin.Consequence",
            "Crouzeix.Jin.Main",
            "CrouzeixConjecture.Foundation",
            "CrouzeixJin"
        ]);
        let closure_sha = digest(&closure);
        let manifest = json!({
            "schema_version": "crouzeix-route-proof-manifest/v1",
            "route_id": "jin",
            "claim_kind": "source-faithful",
            "aggregate_module": "CrouzeixJin",
            "build_target": "CrouzeixJin",
            "terminal_declaration": "CrouzeixConjecture.jinMain",
            "terminal_type_sha256": "a".repeat(64),
            "consequence_declarations": ["CrouzeixConjecture.jinConsequence"],
            "source_identities": [format!("sha256:{}", "b".repeat(64))],
            "shared_foundation_modules": ["CrouzeixConjecture.Foundation", "CrouzeixJin"],
            "module_closure": closure,
            "module_closure_sha256": closure_sha,
            "allowed_axioms": ["Classical.choice", "Quot.sound", "propext"],
            "review_path": "evidence/crouzeix_conjecture/reviews/jin.json",
            "review_sha256": "c".repeat(64),
            "receipt_path": "evidence/crouzeix_conjecture/routes/jin/receipt.json",
            "receipt_sha256": "d".repeat(64),
            "nodes": [
                {
                    "node_id": "base", "role": "load-bearing",
                    "declaration": "CrouzeixConjecture.jinBase",
                    "module_path": "Crouzeix/Jin/Base.lean", "dependency_ids": [],
                    "provenance_kind": "source",
                    "correspondence_kind": "direct-source",
                    "source_locator": "evidence/crouzeix_conjecture/source.md#L1-L1",
                    "source_archive_sha256": null,
                    "source_file_sha256": "b".repeat(64),
                    "source_excerpt_sha256": "b".repeat(64),
                    "source_line_count": 1,
                    "reused_from_route": null, "reused_node_id": null,
                    "declaration_type_path": "evidence/crouzeix_conjecture/types/base.txt",
                    "statement_sha256": "e".repeat(64)
                },
                {
                    "node_id": "main", "role": "terminal",
                    "declaration": "CrouzeixConjecture.jinMain",
                    "module_path": "Crouzeix/Jin/Main.lean", "dependency_ids": ["base"],
                    "provenance_kind": "source",
                    "correspondence_kind": "direct-source",
                    "source_locator": "evidence/crouzeix_conjecture/source.md#L2-L2",
                    "source_archive_sha256": null,
                    "source_file_sha256": "b".repeat(64),
                    "source_excerpt_sha256": "b".repeat(64),
                    "source_line_count": 2,
                    "reused_from_route": null, "reused_node_id": null,
                    "declaration_type_path": "evidence/crouzeix_conjecture/types/main.txt",
                    "statement_sha256": "a".repeat(64)
                },
                {
                    "node_id": "consequence", "role": "consequence",
                    "declaration": "CrouzeixConjecture.jinConsequence",
                    "module_path": "Crouzeix/Jin/Consequence.lean", "dependency_ids": ["main"],
                    "provenance_kind": "source",
                    "correspondence_kind": "direct-source",
                    "source_locator": "evidence/crouzeix_conjecture/source.md#L3-L3",
                    "source_archive_sha256": null,
                    "source_file_sha256": "b".repeat(64),
                    "source_excerpt_sha256": "b".repeat(64),
                    "source_line_count": 3,
                    "reused_from_route": null, "reused_node_id": null,
                    "declaration_type_path": "evidence/crouzeix_conjecture/types/consequence.txt",
                    "statement_sha256": "f".repeat(64)
                }
            ]
        });
        let mut receipt = json!({
            "schema_version": "crouzeix-route-proof-receipt/v1",
            "route_id": "jin", "aggregate_module": "CrouzeixJin",
            "build_target": "CrouzeixJin",
            "manifest_path": "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json",
            "manifest_sha256": "1".repeat(64),
            "candidate_commit": "1".repeat(40), "candidate_tree": "2".repeat(40),
            "command_artifact_path": "evidence/crouzeix_conjecture/routes/jin/command.json",
            "command_artifact_sha256": "2".repeat(64),
            "argv": ["scripts/check_lean_library.sh", "CrouzeixJin"],
            "working_directory": ".",
            "cache_identity": format!("sha256:{}", "3".repeat(64)),
            "toolchain": "leanprover/lean4:v4.32.1",
            "local_closure_modules": manifest["module_closure"].clone(),
            "local_closure_sha256": manifest["module_closure_sha256"].clone(),
            "mathlib_artifacts": [{"module": "Mathlib.Data.Matrix.Basic", "path": "evidence/crouzeix_conjecture/routes/jin/Mathlib.olean", "sha256": "4".repeat(64)}],
            "mathlib_artifacts_sha256": "5".repeat(64),
            "declaration_types": [
                {"declaration": "CrouzeixConjecture.jinBase", "type_artifact_path": "evidence/crouzeix_conjecture/types/base.txt", "type_artifact_sha256": "6".repeat(64), "statement_sha256": "e".repeat(64)},
                {"declaration": "CrouzeixConjecture.jinMain", "type_artifact_path": "evidence/crouzeix_conjecture/types/main.txt", "type_artifact_sha256": "7".repeat(64), "statement_sha256": "a".repeat(64)},
                {"declaration": "CrouzeixConjecture.jinConsequence", "type_artifact_path": "evidence/crouzeix_conjecture/types/consequence.txt", "type_artifact_sha256": "8".repeat(64), "statement_sha256": "f".repeat(64)}
            ],
            "allowed_axioms": manifest["allowed_axioms"].clone(),
            "axiom_audit_path": "evidence/crouzeix_conjecture/routes/jin/axioms.json",
            "axiom_audit_sha256": "9".repeat(64),
            "axiom_results": [
                {"declaration": "CrouzeixConjecture.jinBase", "axioms": ["propext"]},
                {"declaration": "CrouzeixConjecture.jinMain", "axioms": ["propext"]},
                {"declaration": "CrouzeixConjecture.jinConsequence", "axioms": ["propext"]}
            ],
            "provider_report_path": "evidence/crouzeix_conjecture/routes/jin/provider.json",
            "provider_report_sha256": "a".repeat(64),
            "stdout_path": "evidence/crouzeix_conjecture/routes/jin/stdout.log", "stdout_sha256": "b".repeat(64),
            "stderr_path": "evidence/crouzeix_conjecture/routes/jin/stderr.log", "stderr_sha256": "c".repeat(64),
            "exit_code": 0, "status": "passed", "receipt_sha256": "d".repeat(64)
        });
        receipt["mathlib_artifacts_sha256"] = json!(digest(&receipt["mathlib_artifacts"]));
        receipt["manifest_sha256"] = json!(manifest_digest(&manifest));
        receipt["receipt_sha256"] = json!(self_digest(&receipt, "receipt_sha256"));
        let mut review = json!({
            "schema_version": "crouzeix-proof-review/v1", "route_id": "jin",
            "reviewed_commit": receipt["candidate_commit"].clone(),
            "reviewed_tree": receipt["candidate_tree"].clone(),
            "manifest_path": receipt["manifest_path"].clone(),
            "manifest_sha256": receipt["manifest_sha256"].clone(),
            "terminal_type_sha256": manifest["terminal_type_sha256"].clone(),
            "review_id": "review-jin-001", "reviewer_identity": "reviewer",
            "reviewer_model": "review-model", "reviewer_run_id": "run-001",
            "verdict": "complete", "outcome": "approved",
            "source_fidelity_check": "passed", "derivation_reuse_check": "not-applicable",
            "findings": [{"finding_id": "F-001", "severity": "Minor", "resolved": true, "locator": "formalization/lean/CrouzeixJin.lean", "statement": "note", "falsifying_test_or_gap": "none"}],
            "review_sha256": "e".repeat(64)
        });
        review["manifest_sha256"] = receipt["manifest_sha256"].clone();
        review["review_sha256"] = json!(self_digest(&review, "review_sha256"));
        (manifest, receipt, review)
    }

    #[test]
    fn accepts_concrete_bound_records() {
        let (manifest, receipt, review) = fixture();
        validate_records(&manifest, &receipt, &review).unwrap();
    }

    #[test]
    fn rejects_unknown_manifest_field() {
        let (mut manifest, receipt, review) = fixture();
        manifest["surprise"] = json!(true);
        assert!(validate_records(&manifest, &receipt, &review)
            .unwrap_err()
            .contains("unknown field"));
    }

    #[test]
    fn rejects_cycle_and_cross_artifact_mismatch() {
        let (mut manifest, receipt, review) = fixture();
        manifest["nodes"][0]["dependency_ids"] = json!(["main"]);
        assert!(validate_records(&manifest, &receipt, &review)
            .unwrap_err()
            .contains("topological"));

        let (manifest, mut receipt, review) = fixture();
        receipt["aggregate_module"] = json!("CrouzeixHarp");
        assert!(validate_records(&manifest, &receipt, &review)
            .unwrap_err()
            .contains("aggregate"));
    }

    #[test]
    fn rejects_unsafe_path_forbidden_axiom_and_blocking_review() {
        let (manifest, mut receipt, review) = fixture();
        receipt["stdout_path"] = json!("../stdout.log");
        assert!(validate_records(&manifest, &receipt, &review)
            .unwrap_err()
            .contains("path"));

        let (manifest, mut receipt, review) = fixture();
        receipt["axiom_results"][0]["axioms"] = json!(["False.elim"]);
        assert!(validate_records(&manifest, &receipt, &review)
            .unwrap_err()
            .contains("axiom"));

        let (manifest, receipt, mut review) = fixture();
        review["findings"][0]["severity"] = json!("Critical");
        review["findings"][0]["resolved"] = json!(false);
        assert!(validate_records(&manifest, &receipt, &review)
            .unwrap_err()
            .contains("unresolved"));
    }

    #[test]
    fn repository_loader_is_optional_and_rejects_duplicate_keys() {
        let root = TempDir::new().unwrap();
        verify_published_routes(root.path()).unwrap();

        let relative =
            "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json";
        let path = root.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"{\"route_id\":\"jin\",\"route_id\":\"harp\"}\n").unwrap();

        assert!(read_json_value(root.path(), relative, "route manifest")
            .unwrap_err()
            .contains("duplicate JSON key"));
    }

    #[cfg(unix)]
    #[test]
    fn repository_loader_rejects_symlinks_and_hardlinks() {
        use std::os::unix::fs::symlink;

        let root = TempDir::new().unwrap();
        let original = root.path().join("artifact.json");
        fs::write(&original, b"{}\n").unwrap();
        let symlink_path = root.path().join("linked.json");
        symlink("artifact.json", &symlink_path).unwrap();
        assert!(read_bounded_regular(root.path(), "linked.json", "artifact")
            .unwrap_err()
            .contains("symlink"));

        let hardlink_path = root.path().join("hardlinked.json");
        fs::hard_link(&original, &hardlink_path).unwrap();
        assert!(
            read_bounded_regular(root.path(), "hardlinked.json", "artifact")
                .unwrap_err()
                .contains("hardlink")
        );
    }

    #[test]
    fn repository_loader_rejects_active_closure_and_provider_report_drift() {
        let (root, manifest_path) = repository_fixture();
        let mut manifest = read_json_value(root.path(), &manifest_path, "manifest").unwrap();
        manifest["module_closure"] = json!(["CrouzeixJin"]);
        manifest["module_closure_sha256"] = json!(digest(&manifest["module_closure"]));
        fs::write(
            root.path().join(&manifest_path),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        refresh_manifest_receipt_digest(root.path(), &manifest_path);
        let error = verify_published_routes(root.path()).unwrap_err();
        assert!(error.contains("active closure"), "{error}");

        let (root, _) = repository_fixture();
        let provider = root
            .path()
            .join("evidence/crouzeix_conjecture/routes/jin/provider.json");
        let mut value: Value = serde_json::from_slice(&fs::read(&provider).unwrap()).unwrap();
        value["modules"] = json!([]);
        fs::write(&provider, serde_json::to_vec(&value).unwrap()).unwrap();
        let error = verify_published_routes(root.path()).unwrap_err();
        assert!(error.contains("provider report"), "{error}");
    }

    #[test]
    fn repository_loader_rejects_fake_git_identity() {
        let (root, manifest_path) = repository_fixture();
        let manifest = read_json_value(root.path(), &manifest_path, "manifest").unwrap();
        let receipt_path = manifest["receipt_path"].as_str().unwrap();
        let mut receipt = read_json_value(root.path(), receipt_path, "receipt").unwrap();
        receipt["candidate_commit"] = json!("1".repeat(40));
        receipt["receipt_sha256"] = json!(self_digest(&receipt, "receipt_sha256"));
        fs::write(
            root.path().join(receipt_path),
            serde_json::to_vec(&receipt).unwrap(),
        )
        .unwrap();
        refresh_manifest_receipt_digest(root.path(), &manifest_path);
        let error = verify_published_routes(root.path()).unwrap_err();
        assert!(error.contains("Git commit"), "{error}");
    }

    #[test]
    fn repository_loader_enforces_exact_publication_states() {
        let (root, manifest_path) = repository_fixture();
        let mut manifest = read_json_value(root.path(), &manifest_path, "manifest").unwrap();
        let review_path = manifest["review_path"].as_str().unwrap().to_owned();
        manifest["receipt_sha256"] = Value::Null;
        fs::write(
            root.path().join(&manifest_path),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let error = verify_published_routes(root.path()).unwrap_err();
        assert!(
            error.contains("receipt") && error.contains("digest"),
            "{error}"
        );

        let (root, manifest_path) = repository_fixture();
        let mut manifest = read_json_value(root.path(), &manifest_path, "manifest").unwrap();
        let receipt_path = manifest["receipt_path"].as_str().unwrap().to_owned();
        fs::remove_file(root.path().join(&receipt_path)).unwrap();
        manifest["receipt_sha256"] = Value::Null;
        fs::write(
            root.path().join(&manifest_path),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let error = verify_published_routes(root.path()).unwrap_err();
        assert!(
            error.contains("review") && error.contains("receipt"),
            "{error}"
        );

        let (root, manifest_path) = repository_fixture();
        let manifest = read_json_value(root.path(), &manifest_path, "manifest").unwrap();
        fs::remove_file(root.path().join(manifest["review_path"].as_str().unwrap())).unwrap();
        let error = verify_published_routes(root.path()).unwrap_err();
        assert!(
            error.contains("review")
                && (error.contains("missing") || error.contains("presence/digest")),
            "{error}"
        );
        let _ = review_path;
    }

    #[test]
    fn manifest_only_rejects_source_and_type_drift() {
        let (root, manifest_path) = repository_fixture();
        let mut manifest = read_json_value(root.path(), &manifest_path, "manifest").unwrap();
        let receipt = manifest["receipt_path"].as_str().unwrap().to_owned();
        let review = manifest["review_path"].as_str().unwrap().to_owned();
        fs::remove_file(root.path().join(receipt)).unwrap();
        fs::remove_file(root.path().join(review)).unwrap();
        manifest["receipt_sha256"] = Value::Null;
        manifest["review_sha256"] = Value::Null;
        manifest["nodes"][0]["source_locator"] =
            json!("evidence/crouzeix_conjecture/source.md#L3-L2");
        fs::write(
            root.path().join(&manifest_path),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(verify_published_routes(root.path())
            .unwrap_err()
            .contains("reversed"));
    }

    #[test]
    fn repository_loader_semantically_rejects_command_and_axiom_mutations() {
        for artifact in ["command", "axiom"] {
            let (root, manifest_path) = repository_fixture();
            let manifest = read_json_value(root.path(), &manifest_path, "manifest").unwrap();
            let receipt_path = manifest["receipt_path"].as_str().unwrap();
            let mut receipt = read_json_value(root.path(), receipt_path, "receipt").unwrap();
            let (path_field, digest_field) = if artifact == "command" {
                ("command_artifact_path", "command_artifact_sha256")
            } else {
                ("axiom_audit_path", "axiom_audit_sha256")
            };
            let path = receipt[path_field].as_str().unwrap();
            let mut value = json!({"schema_version": "wrong/v1", "surprise": true});
            fs::write(root.path().join(path), serde_json::to_vec(&value).unwrap()).unwrap();
            receipt[digest_field] = json!(format!(
                "{:x}",
                Sha256::digest(fs::read(root.path().join(path)).unwrap())
            ));
            receipt["receipt_sha256"] = json!(self_digest(&receipt, "receipt_sha256"));
            fs::write(
                root.path().join(receipt_path),
                serde_json::to_vec(&receipt).unwrap(),
            )
            .unwrap();
            refresh_manifest_receipt_digest(root.path(), &manifest_path);
            let error = verify_published_routes(root.path()).unwrap_err();
            assert!(error.contains(artifact), "{artifact}: {error}");
            value = Value::Null;
            let _ = value;
        }
    }

    fn refresh_manifest_receipt_digest(root: &Path, manifest_path: &str) {
        let mut manifest = read_json_value(root, manifest_path, "manifest").unwrap();
        let receipt_path = manifest["receipt_path"].as_str().unwrap();
        manifest["receipt_sha256"] = json!(format!(
            "{:x}",
            Sha256::digest(fs::read(root.join(receipt_path)).unwrap())
        ));
        fs::write(
            root.join(manifest_path),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn production_reader_is_bounded_and_does_not_use_fs_read() {
        let source = include_str!("route.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        assert!(!production.contains("fs::read("));
        let root = TempDir::new().unwrap();
        fs::write(
            root.path().join("oversize"),
            vec![0u8; super::MAX_ARTIFACT_BYTES as usize + 1],
        )
        .unwrap();
        assert!(read_bounded_regular(root.path(), "oversize", "artifact")
            .unwrap_err()
            .contains("bounded"));
    }

    #[test]
    fn cache_artifact_reader_rejects_cached_lean_source_above_two_mebibytes() {
        let root = TempDir::new().unwrap();
        let relative = "packages/mathlib/Mathlib/Data/Matrix/Basic.lean";
        let path = root.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, vec![b'x'; super::MAX_ARTIFACT_BYTES as usize + 1]).unwrap();

        let error = super::read_cache_regular(root.path(), relative, "Mathlib source").unwrap_err();
        assert!(
            error.contains("Mathlib source")
                || error.contains("bounded")
                || error.contains("artifact path")
        );
    }

    #[test]
    fn cache_artifact_reader_rejects_dot_segment_escape() {
        let root = TempDir::new().unwrap();
        let escaped = "packages/mathlib/escaped.olean";
        let escaped_path = root.path().join(escaped);
        fs::create_dir_all(escaped_path.parent().unwrap()).unwrap();
        fs::write(
            &escaped_path,
            vec![b'x'; super::MAX_ARTIFACT_BYTES as usize + 1],
        )
        .unwrap();

        let error = super::read_cache_regular(
            root.path(),
            "packages/mathlib/.lake/build/lib/lean/../../../../escaped.olean",
            "Mathlib artifact",
        )
        .unwrap_err();
        assert!(error.contains("unsafe") || error.contains("validated Mathlib artifact path"));
    }

    #[test]
    fn cache_source_reader_rejects_cached_lean_source_above_two_mebibytes() {
        let root = TempDir::new().unwrap();
        let relative = "packages/mathlib/Mathlib/Data/Matrix/Basic.lean";
        let path = root.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, vec![b'x'; super::MAX_ARTIFACT_BYTES as usize + 1]).unwrap();

        let error =
            super::read_cache_mathlib_source(root.path(), "Mathlib.Data.Matrix.Basic").unwrap_err();
        assert!(
            error.contains("Mathlib source Mathlib.Data.Matrix.Basic") && error.contains("bounded")
        );
    }

    #[test]
    fn cache_artifact_reader_accepts_large_olean_within_cache_specific_bound() {
        let root = TempDir::new().unwrap();
        let relative = "packages/mathlib/.lake/build/lib/lean/Mathlib/Data/Matrix/Basic.olean";
        let path = root.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let payload = vec![b'x'; 3_365_968];
        fs::write(&path, &payload).unwrap();

        assert_eq!(
            super::read_cache_regular(root.path(), relative, "cache artifact").unwrap(),
            payload
        );
    }

    #[test]
    fn cache_artifact_reader_rejects_artifacts_above_eight_mebibytes() {
        let root = TempDir::new().unwrap();
        let relative = "packages/mathlib/.lake/build/lib/lean/Mathlib/Data/Matrix/TooLarge.olean";
        let path = root.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, vec![b'x'; 8 * 1024 * 1024 + 1]).unwrap();

        assert!(
            super::read_cache_regular(root.path(), relative, "cache artifact")
                .unwrap_err()
                .contains("bounded")
        );
    }

    #[test]
    fn structured_reader_keeps_two_mebibyte_bound() {
        let root = TempDir::new().unwrap();
        fs::write(
            root.path().join("oversize-structured"),
            vec![0u8; 2 * 1024 * 1024 + 1],
        )
        .unwrap();

        assert!(
            read_bounded_regular(root.path(), "oversize-structured", "structured artifact")
                .unwrap_err()
                .contains("bounded")
        );
    }

    #[test]
    fn structured_artifact_parses_the_exact_digest_verified_single_read() {
        let bytes = br#"{"schema_version":"crouzeix-route-command/v1","argv":["scripts/check_lean_library.sh","CrouzeixJin"],"working_directory":".","aggregate_module":"CrouzeixJin","build_target":"CrouzeixJin","cache_identity":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","toolchain":"leanprover/lean4:v4.32.1"}"#.to_vec();
        let expected = format!("{:x}", Sha256::digest(&bytes));
        let reads = std::cell::Cell::new(0);
        let command: RouteCommand =
            super::read_digest_bound_json_with("command artifact", &expected, || {
                reads.set(reads.get() + 1);
                Ok(bytes)
            })
            .unwrap();
        assert_eq!(command.schema_version, "crouzeix-route-command/v1");
        assert_eq!(reads.get(), 1);
    }

    #[test]
    fn hardened_parent_lexer_drives_route_and_mathlib_closures() {
        let root = TempDir::new().unwrap();
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(root.path())
            .status()
            .unwrap();
        fs::create_dir_all(
            root.path()
                .join("formalization/lean/.lake/packages/mathlib/Mathlib"),
        )
        .unwrap();
        fs::create_dir_all(root.path().join("formalization/lean/Crouzeix/Jin")).unwrap();
        fs::write(root.path().join("formalization/lean/CrouzeixJin.lean"), b"module\nprelude\n/- nested /- import Crouzeix.Harp.Hidden -/ comment -/\npublic import Crouzeix.Jin.Main\n").unwrap();
        fs::write(
            root.path()
                .join("formalization/lean/Crouzeix/Jin/Main.lean"),
            b"-- import Crouzeix.Harp.Hidden\nimport Mathlib.A\n",
        )
        .unwrap();
        fs::write(
            root.path()
                .join("formalization/lean/.lake/packages/mathlib/Mathlib/A.lean"),
            b"module\npublic import Mathlib.B\n",
        )
        .unwrap();
        fs::write(
            root.path()
                .join("formalization/lean/.lake/packages/mathlib/Mathlib/B.lean"),
            b"\n",
        )
        .unwrap();
        let closure = active_route_closure(root.path(), RouteId::Jin, "CrouzeixJin").unwrap();
        assert_eq!(closure, vec!["Crouzeix.Jin.Main", "CrouzeixJin"]);
        assert_eq!(
            active_mathlib_closure(root.path(), &closure).unwrap(),
            vec!["Mathlib.A", "Mathlib.B"]
        );
        fs::write(
            root.path().join("formalization/lean/CrouzeixJin.lean"),
            b"module\nprelude\npublic import Crouzeix.Harp.Hidden\n",
        )
        .unwrap();
        fs::create_dir_all(root.path().join("formalization/lean/Crouzeix/Harp")).unwrap();
        fs::write(
            root.path()
                .join("formalization/lean/Crouzeix/Harp/Hidden.lean"),
            b"\n",
        )
        .unwrap();
        assert!(
            active_route_closure(root.path(), RouteId::Jin, "CrouzeixJin")
                .unwrap_err()
                .contains("forbidden provider")
        );
    }

    #[test]
    fn route_policy_rejects_top_level_cross_route_aggregates() {
        let cases = [
            (
                RouteId::Jin,
                ["CrouzeixLoristSchwenninger", "CrouzeixHarp", "Crouzeix"],
            ),
            (
                RouteId::LoristSchwenninger,
                ["CrouzeixJin", "CrouzeixHarp", "Crouzeix"],
            ),
            (
                RouteId::Harp,
                ["CrouzeixJin", "CrouzeixLoristSchwenninger", "Crouzeix"],
            ),
        ];
        for (route, modules) in cases {
            for module in modules {
                assert!(
                    super::forbidden_provider(route, module),
                    "{route:?} accepted {module}"
                );
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn linked_worktree_cache_accepts_only_primary_cache_and_rejects_nested_links() {
        use std::os::unix::fs::symlink;

        let primary = TempDir::new().unwrap();
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(primary.path())
            .status()
            .unwrap();
        fs::create_dir_all(
            primary
                .path()
                .join("formalization/lean/.lake/packages/mathlib"),
        )
        .unwrap();
        fs::write(primary.path().join("tracked"), b"x").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(primary.path())
            .status()
            .unwrap();
        Command::new("git")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-q",
                "-m",
                "fixture",
            ])
            .current_dir(primary.path())
            .status()
            .unwrap();
        let linked_root = primary.path().join("linked");
        Command::new("git")
            .args([
                "worktree",
                "add",
                "-q",
                "--detach",
                linked_root.to_str().unwrap(),
                "HEAD",
            ])
            .current_dir(primary.path())
            .status()
            .unwrap();
        fs::create_dir_all(linked_root.join("formalization/lean")).unwrap();
        symlink(
            primary.path().join("formalization/lean/.lake"),
            linked_root.join("formalization/lean/.lake"),
        )
        .unwrap();
        assert_eq!(
            super::resolve_approved_cache_root(&linked_root).unwrap(),
            primary
                .path()
                .join("formalization/lean/.lake")
                .canonicalize()
                .unwrap()
        );
        fs::rename(
            primary
                .path()
                .join("formalization/lean/.lake/packages/mathlib"),
            primary
                .path()
                .join("formalization/lean/.lake/packages/mathlib-real"),
        )
        .unwrap();
        symlink(
            "mathlib-real",
            primary
                .path()
                .join("formalization/lean/.lake/packages/mathlib"),
        )
        .unwrap();
        assert!(super::resolve_approved_cache_root(&linked_root)
            .unwrap_err()
            .contains("nested symlink"));
    }

    #[cfg(unix)]
    #[test]
    fn published_route_validates_mathlib_artifacts_through_approved_cache_link() {
        let (_primary, _linked_parent, linked, _basic, _transitive) = linked_published_fixture();
        verify_published_routes(&linked).unwrap();
    }

    #[test]
    fn published_route_reports_every_validated_in_root_evidence_member() {
        let (_primary, _linked_parent, linked, _basic, _transitive) = linked_published_fixture();

        let evidence = verify_published_routes(&linked).unwrap();

        assert!(evidence.contains("routes/jin/receipt.json"));
        assert!(evidence.contains("routes/jin/command.json"));
        assert!(evidence.contains("routes/jin/axioms.json"));
        assert!(evidence.contains("routes/jin/provider.json"));
        assert!(evidence.contains("routes/jin/stdout.log"));
        assert!(evidence.contains("routes/jin/stderr.log"));
        assert!(evidence.contains("reviews/jin.json"));
        assert!(evidence.contains("types/main.txt"));
        assert!(evidence.contains("source.md"));
    }

    #[cfg(unix)]
    #[test]
    fn published_route_rejects_cache_and_mathlib_artifact_mutations() {
        use std::os::unix::fs::symlink;

        let (_primary, _linked_parent, linked, _, _) = linked_published_fixture();
        let link = linked.join("formalization/lean/.lake");
        fs::remove_file(&link).unwrap();
        let hostile = linked.join("hostile-cache");
        fs::create_dir(&hostile).unwrap();
        symlink(&hostile, &link).unwrap();
        let error = verify_published_routes(&linked).unwrap_err();
        assert!(
            error.contains("approved primary cache") || error.contains("worktree .lake"),
            "{error}"
        );

        let (_primary, _linked_parent, linked, _, transitive) = linked_published_fixture();
        let saved = transitive.with_extension("real");
        fs::rename(&transitive, &saved).unwrap();
        symlink(&saved, &transitive).unwrap();
        assert!(verify_published_routes(&linked)
            .unwrap_err()
            .contains("symlink"));

        let (_primary, _linked_parent, linked, _, transitive) = linked_published_fixture();
        fs::remove_file(&transitive).unwrap();
        assert!(verify_published_routes(&linked)
            .unwrap_err()
            .contains("Mathlib artifact"));

        let (_primary, _linked_parent, linked, basic, transitive) = linked_published_fixture();
        rewrite_linked_receipt_mathlib(&linked, &[("Mathlib.Data.Matrix.Basic", &basic)]);
        assert!(verify_published_routes(&linked)
            .unwrap_err()
            .contains("roster"));
        rewrite_linked_receipt_mathlib(
            &linked,
            &[
                ("Mathlib.Data.Matrix.Basic", &basic),
                ("Mathlib.Data.Matrix.Transitive", &transitive),
                ("Mathlib.Extra", &basic),
            ],
        );
        assert!(verify_published_routes(&linked)
            .unwrap_err()
            .contains("roster"));

        let (_primary, _linked_parent, linked, _basic, _transitive) = linked_published_fixture();
        mutate_linked_receipt(&linked, |receipt| {
            receipt["mathlib_artifacts"][0]["path"] = json!("formalization/lean/.lake/wrong.olean");
        });
        assert!(verify_published_routes(&linked)
            .unwrap_err()
            .contains("module/path"));

        let (_primary, _linked_parent, linked, _basic, _transitive) = linked_published_fixture();
        mutate_linked_receipt(&linked, |receipt| {
            receipt["mathlib_artifacts"][0]["sha256"] = json!("0".repeat(64));
        });
        assert!(verify_published_routes(&linked)
            .unwrap_err()
            .contains("digest mismatch"));
    }

    #[cfg(unix)]
    fn linked_published_fixture() -> (
        TempDir,
        TempDir,
        std::path::PathBuf,
        std::path::PathBuf,
        std::path::PathBuf,
    ) {
        use std::os::unix::fs::symlink;
        let (primary, _) = repository_fixture();
        let cache = primary.path().join("formalization/lean/.lake");
        let mathlib_source = cache.join("packages/mathlib/Mathlib/Data/Matrix/Basic.lean");
        let mathlib_transitive = cache.join("packages/mathlib/Mathlib/Data/Matrix/Transitive.lean");
        fs::create_dir_all(mathlib_source.parent().unwrap()).unwrap();
        fs::write(&mathlib_source, b"import Mathlib.Data.Matrix.Transitive\n").unwrap();
        fs::write(&mathlib_transitive, b"\n").unwrap();
        let artifact_root = cache.join("packages/mathlib/.lake/build/lib/lean/Mathlib/Data/Matrix");
        fs::create_dir_all(&artifact_root).unwrap();
        let basic = artifact_root.join("Basic.olean");
        let transitive = artifact_root.join("Transitive.olean");
        fs::write(&basic, b"olean-basic").unwrap();
        fs::write(&transitive, b"olean-transitive").unwrap();

        let linked_parent = TempDir::new().unwrap();
        let linked = linked_parent.path().join("linked");
        Command::new("git")
            .args([
                "worktree",
                "add",
                "-q",
                "--detach",
                linked.to_str().unwrap(),
                "HEAD",
            ])
            .current_dir(primary.path())
            .status()
            .unwrap();
        fs::create_dir_all(linked.join("formalization/lean")).unwrap();
        let linked_cache = linked.join("formalization/lean/.lake");
        if linked_cache.exists() {
            fs::remove_dir_all(&linked_cache).unwrap();
        }
        symlink(&cache, &linked_cache).unwrap();
        copy_untracked_fixture_files(primary.path(), &linked);
        rewrite_linked_receipt_mathlib(
            &linked,
            &[
                ("Mathlib.Data.Matrix.Basic", &basic),
                ("Mathlib.Data.Matrix.Transitive", &transitive),
            ],
        );
        (primary, linked_parent, linked, basic, transitive)
    }

    fn copy_untracked_fixture_files(primary: &Path, linked: &Path) {
        for relative in [
            "formalization/lean/lean-toolchain",
            "formalization/lean/lake-manifest.json",
            "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json",
            "evidence/crouzeix_conjecture/routes/jin/receipt.json",
            "evidence/crouzeix_conjecture/routes/jin/provider.json",
            "evidence/crouzeix_conjecture/routes/jin/command.json",
            "evidence/crouzeix_conjecture/routes/jin/axioms.json",
            "evidence/crouzeix_conjecture/routes/jin/stdout.log",
            "evidence/crouzeix_conjecture/routes/jin/stderr.log",
            "evidence/crouzeix_conjecture/reviews/jin.json",
            "evidence/crouzeix_conjecture/types/main.txt",
            "evidence/crouzeix_conjecture/source.md",
        ] {
            let from = primary.join(relative);
            if !from.exists() {
                continue;
            }
            let to = linked.join(relative);
            fs::create_dir_all(to.parent().unwrap()).unwrap();
            fs::copy(from, to).unwrap();
        }
    }

    fn rewrite_linked_receipt_mathlib(linked: &Path, entries: &[(&str, &Path)]) {
        let manifest_path =
            "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json";
        let mut manifest = read_json_value(linked, manifest_path, "manifest").unwrap();
        let receipt_path = manifest["receipt_path"].as_str().unwrap();
        let mut receipt = read_json_value(linked, receipt_path, "receipt").unwrap();
        let roster = entries.iter().map(|(module, path)| json!({
            "module": module,
            "path": format!("formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean/{}.olean", module.replace('.', "/")),
            "sha256": format!("{:x}", Sha256::digest(fs::read(path).unwrap())),
        })).collect::<Vec<_>>();
        receipt["mathlib_artifacts"] = json!(roster);
        receipt["mathlib_artifacts_sha256"] = json!(digest(&receipt["mathlib_artifacts"]));
        receipt["receipt_sha256"] = json!(self_digest(&receipt, "receipt_sha256"));
        fs::write(
            linked.join(receipt_path),
            serde_json::to_vec(&receipt).unwrap(),
        )
        .unwrap();
        manifest["receipt_sha256"] = json!(format!(
            "{:x}",
            Sha256::digest(fs::read(linked.join(receipt_path)).unwrap())
        ));
        fs::write(
            linked.join(manifest_path),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
    }

    fn mutate_linked_receipt(linked: &Path, mutate: impl FnOnce(&mut Value)) {
        let manifest_path =
            "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json";
        let mut manifest = read_json_value(linked, manifest_path, "manifest").unwrap();
        let receipt_path = manifest["receipt_path"].as_str().unwrap().to_owned();
        let mut receipt = read_json_value(linked, &receipt_path, "receipt").unwrap();
        mutate(&mut receipt);
        receipt["mathlib_artifacts_sha256"] = json!(digest(&receipt["mathlib_artifacts"]));
        receipt["receipt_sha256"] = json!(self_digest(&receipt, "receipt_sha256"));
        fs::write(
            linked.join(&receipt_path),
            serde_json::to_vec(&receipt).unwrap(),
        )
        .unwrap();
        manifest["receipt_sha256"] = json!(format!(
            "{:x}",
            Sha256::digest(fs::read(linked.join(&receipt_path)).unwrap())
        ));
        fs::write(
            linked.join(manifest_path),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn source_locator_enforces_span_and_identity() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join("evidence")).unwrap();
        fs::write(root.path().join("evidence/source.md"), b"one\ntwo\n").unwrap();
        let file_sha = format!("{:x}", Sha256::digest(b"one\ntwo\n"));
        let excerpt_sha = file_sha.clone();
        let (manifest_value, _, _) = fixture();
        let mut manifest: RouteManifest = parse(&manifest_value, "manifest").unwrap();
        manifest.source_identities = vec![format!("sha256:{file_sha}")];
        let mut node = manifest.nodes[0].clone();
        node.source_locator = Some("evidence/source.md#L1-L2".into());
        node.source_file_sha256 = Some(file_sha);
        node.source_excerpt_sha256 = Some(excerpt_sha);
        node.source_line_count = Some(2);
        validate_source_provenance(root.path(), &manifest, &node).unwrap();

        node.source_locator = Some("evidence/source.md#L2-L1".into());
        assert!(validate_source_provenance(root.path(), &manifest, &node)
            .unwrap_err()
            .contains("reversed"));
        node.source_locator = Some("evidence/source.md#L1-L3".into());
        assert!(validate_source_provenance(root.path(), &manifest, &node)
            .unwrap_err()
            .contains("line count"));
        node.source_locator = Some("evidence/source.md#L1-L2".into());
        manifest.source_identities = vec![format!("sha256:{}", "0".repeat(64))];
        assert!(validate_source_provenance(root.path(), &manifest, &node)
            .unwrap_err()
            .contains("source identity"));
    }

    #[test]
    fn jin_git_locator_requires_exactly_one_registered_artifact_path() {
        let root = TempDir::new().unwrap();
        let artifact_path = root.path().join(
            "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/artifact-manifest.json",
        );
        fs::create_dir_all(artifact_path.parent().unwrap()).unwrap();
        let commit = "a".repeat(40);
        let archive = "b".repeat(64);
        fs::write(
            artifact_path,
            serde_json::to_vec(&json!({
                "schema_version": "crouzeix-formal-artifact-manifest/v1",
                "source_commit": commit,
                "archive": {"sha256": archive},
                "artifacts": {
                    "registered": {
                        "path": "Lean/CrouzeixConjecture/Registered.lean",
                        "sha256": "c".repeat(64),
                    }
                },
            }))
            .unwrap(),
        )
        .unwrap();
        let (manifest_value, _, _) = fixture();
        let mut manifest: RouteManifest = parse(&manifest_value, "manifest").unwrap();
        manifest.source_identities = vec![format!("sha256:{archive}")];
        let mut node = manifest.nodes[0].clone();
        node.source_locator = Some(format!(
            "git:{commit}:Lean/CrouzeixConjecture/Unregistered.lean#L1-L1"
        ));
        node.source_archive_sha256 = Some(archive.clone());
        node.source_file_sha256 = Some("d".repeat(64));
        node.source_excerpt_sha256 = Some("e".repeat(64));
        node.source_line_count = Some(1);

        assert!(validate_source_provenance(root.path(), &manifest, &node)
            .unwrap_err()
            .contains("exactly one artifact record must match Git source path"));

        let artifact_path = root.path().join(
            "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/artifact-manifest.json",
        );
        fs::write(
            artifact_path,
            serde_json::to_vec(&json!({
                "schema_version": "crouzeix-formal-artifact-manifest/v1",
                "source_commit": commit,
                "archive": {"sha256": archive},
                "artifacts": {
                    "first": {
                        "path": "Lean/CrouzeixConjecture/Unregistered.lean",
                        "sha256": "d".repeat(64),
                    },
                    "duplicate": {
                        "path": "Lean/CrouzeixConjecture/Unregistered.lean",
                        "sha256": "d".repeat(64),
                    }
                },
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(validate_source_provenance(root.path(), &manifest, &node)
            .unwrap_err()
            .contains("exactly one artifact record must match Git source path"));
    }

    #[test]
    fn git_locator_rejects_opaque_path_fragment() {
        let locator = format!("git:{}:Lean/Foo.lean#opaque#L1-L1", "a".repeat(40));
        assert!(parse_source_locator(&locator).is_none());
    }

    #[test]
    fn arxiv_locator_preserves_full_identity_and_accepts_generic_safe_path() {
        let locator = "arxiv:2608.03841v2:appendix.tex#L1-L2";
        let (kind, path, start, end) = parse_source_locator(locator).unwrap();
        assert_eq!(path, "appendix.tex");
        assert_eq!((start, end), (1, 2));
        match kind {
            SourceLocatorKind::Arxiv(identity) => {
                assert_eq!(identity, "arxiv:2608.03841v2");
            }
            other => panic!("expected arxiv locator, got {other:?}"),
        }
    }

    #[test]
    fn source_locator_rejects_aliasing_path_forms_for_all_locator_kinds() {
        for locator in [
            "dir//file.tex#L1-L2",
            "./dir/file.tex#L1-L2",
            "git:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:dir//file.tex#L1-L2",
            "git:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:./dir/file.tex#L1-L2",
            "arxiv:2608.03841v1:dir//file.tex#L1-L2",
            "arxiv:2608.03841v1:./dir/file.tex#L1-L2",
        ] {
            assert!(parse_source_locator(locator).is_none(), "{locator}");
        }
    }

    #[test]
    fn ls_arxiv_validation_rejects_wrong_identity_and_path_after_generic_parse() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(
            root.path()
                .join("labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger"),
        )
        .unwrap();
        fs::write(
            root.path().join(LS_ARTIFACT_MANIFEST_PATH),
            serde_json::to_vec(&json!({
                "archive_sha256": LS_SOURCE_ARCHIVE_SHA256,
                "manuscript_bytes": LS_SOURCE_BYTES,
                "manuscript_path": LS_SOURCE_PATH,
                "manuscript_sha256": LS_SOURCE_FILE_SHA256,
                "line_count": LS_SOURCE_LINES,
                "schema_version": "crouzeix-arxiv-artifact-manifest/v1",
                "source_id": LS_SOURCE_ID,
                "source_identity": LS_SOURCE_IDENTITY,
            }))
            .unwrap(),
        )
        .unwrap();
        fs::create_dir_all(root.path().join("evidence/crouzeix_conjecture")).unwrap();
        fs::write(
            root.path().join(SOURCE_MANIFEST_PATH),
            concat!(
                "schema_version\treceipt_id\tsource_id\tsource_class\trole\timmutable_identity\t",
                "source_url\tupstream_path\tbytes\tsha256\tlocal_path\tobserved\tlicense_status\tredistribution_status\n",
                "crouzeix-source-receipt/v1\tLS-ARXIV-V1-SOURCE-ARCHIVE\tLS-ARXIV-V1\tarxiv-artifact\tsource\tarxiv:2608.03841v1\thttps://export.arxiv.org/e-print/2608.03841v1\tsource.tar.gz\t7330\tb4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9\t-\t2026-08-14\tarxiv-nonexclusive\tquotation-only\n",
                "crouzeix-source-receipt/v1\tLS-ARXIV-V1-TEX\tLS-ARXIV-V1\tarxiv-artifact\tmanuscript\tarxiv:2608.03841v1\thttps://export.arxiv.org/e-print/2608.03841v1\tCrouzeixConjecturev2.tex\t18783\t20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a\t-\t2026-08-14\tarxiv-nonexclusive\tquotation-only\n"
            ),
        )
        .unwrap();

        let manifest = RouteManifest {
            schema_version: MANIFEST_SCHEMA.into(),
            route_id: RouteId::LoristSchwenninger,
            claim_kind: ClaimKind::SourceFaithful,
            aggregate_module: "CrouzeixLoristSchwenninger".into(),
            build_target: "CrouzeixLoristSchwenninger".into(),
            terminal_declaration: "CrouzeixConjecture.loristSchwenningerMainTheorem".into(),
            terminal_type_sha256: "c".repeat(64),
            consequence_declarations: vec![
                "CrouzeixConjecture.loristSchwenningerMainTheorem".into()
            ],
            source_identities: vec![
                "arxiv:2608.03841v2".into(),
                format!("sha256:{LS_SOURCE_FILE_SHA256}"),
                format!("sha256:{LS_SOURCE_ARCHIVE_SHA256}"),
            ],
            shared_foundation_modules: vec!["CrouzeixLoristSchwenninger".into()],
            module_closure: vec!["CrouzeixLoristSchwenninger".into()],
            module_closure_sha256: "d".repeat(64),
            allowed_axioms: vec![
                "Classical.choice".into(),
                "Quot.sound".into(),
                "propext".into(),
            ],
            review_path: "evidence/crouzeix_conjecture/reviews/lorist-schwenninger.json".into(),
            review_sha256: None,
            receipt_path: "evidence/crouzeix_conjecture/routes/lorist-schwenninger/receipt.json"
                .into(),
            receipt_sha256: None,
            nodes: vec![RouteNode {
                node_id: "ls-terminal-crouzeix".into(),
                role: NodeRole::Terminal,
                declaration: "CrouzeixConjecture.loristSchwenningerMainTheorem".into(),
                module_path: "CrouzeixLoristSchwenninger.lean".into(),
                dependency_ids: Vec::new(),
                provenance_kind: ProvenanceKind::Source,
                correspondence_kind: CorrespondenceKind::DirectSource,
                source_locator: Some("arxiv:2608.03841v2:appendix.tex#L1-L2".into()),
                source_archive_sha256: Some(LS_SOURCE_ARCHIVE_SHA256.into()),
                source_file_sha256: Some(LS_SOURCE_FILE_SHA256.into()),
                source_excerpt_sha256: Some("e".repeat(64)),
                source_line_count: Some(2),
                reused_from_route: None,
                reused_node_id: None,
                declaration_type_path:
                    "evidence/crouzeix_conjecture/routes/lorist-schwenninger/types/main.txt".into(),
                statement_sha256: "c".repeat(64),
            }],
        };

        assert!(
            validate_source_provenance(root.path(), &manifest, &manifest.nodes[0])
                .unwrap_err()
                .contains("source identities must be exact")
        );
    }

    #[test]
    fn correspondence_kind_matrix_is_exact() {
        let allowed = [
            (ProvenanceKind::Source, CorrespondenceKind::DirectSource),
            (
                ProvenanceKind::Source,
                CorrespondenceKind::CompatibilityPort,
            ),
            (
                ProvenanceKind::Source,
                CorrespondenceKind::StructuralRefactor,
            ),
            (
                ProvenanceKind::Derived,
                CorrespondenceKind::DerivedExtraction,
            ),
            (
                ProvenanceKind::SharedFoundation,
                CorrespondenceKind::SharedFoundation,
            ),
            (ProvenanceKind::ReusedRoute, CorrespondenceKind::ReusedRoute),
        ];
        let provenances = [
            ProvenanceKind::Source,
            ProvenanceKind::Derived,
            ProvenanceKind::SharedFoundation,
            ProvenanceKind::ReusedRoute,
        ];
        let correspondences = [
            CorrespondenceKind::DirectSource,
            CorrespondenceKind::CompatibilityPort,
            CorrespondenceKind::StructuralRefactor,
            CorrespondenceKind::DerivedExtraction,
            CorrespondenceKind::SharedFoundation,
            CorrespondenceKind::ReusedRoute,
        ];
        for provenance in provenances {
            for correspondence in correspondences {
                assert_eq!(
                    correspondence_matches(provenance, correspondence),
                    allowed.contains(&(provenance, correspondence)),
                    "unexpected matrix result for {provenance:?}/{correspondence:?}"
                );
            }
        }
    }

    #[test]
    fn exact_mathlib_roster_rejects_transitive_omission() {
        let root = TempDir::new().unwrap();
        let (_, receipt_value, _) = fixture();
        let mut receipt: RouteReceipt = parse(&receipt_value, "receipt").unwrap();
        receipt.mathlib_artifacts.clear();
        let expected = vec!["Mathlib.A".to_owned(), "Mathlib.B".to_owned()];
        assert!(validate_mathlib_roster(root.path(), &receipt, &expected)
            .unwrap_err()
            .contains("transitive closure"));
    }

    #[test]
    fn exact_roles_closure_and_declaration_rosters_reject_drift() {
        let (manifest_value, mut receipt_value, _) = fixture();
        let mut manifest: RouteManifest = parse(&manifest_value, "manifest").unwrap();
        let closure = manifest.module_closure.clone();
        validate_manifest_against_closure(&manifest, &closure).unwrap();
        manifest.nodes[0].role = NodeRole::Consequence;
        assert!(validate_manifest_against_closure(&manifest, &closure)
            .unwrap_err()
            .contains("consequence"));

        receipt_value["declaration_types"][1] = receipt_value["declaration_types"][0].clone();
        let receipt: RouteReceipt = parse(&receipt_value, "receipt").unwrap();
        let manifest: RouteManifest = parse(&manifest_value, "manifest").unwrap();
        assert!(validate_declaration_roster(&manifest, &receipt)
            .unwrap_err()
            .contains("roster"));
    }

    #[test]
    fn harp_reuse_resolves_all_eleven_and_rejects_mutations() {
        let (manifest, ls, closure) = harp_reuse_fixture();
        validate_harp_reuse(&manifest, &closure, &ls).unwrap();
        let mut omitted = manifest.clone();
        omitted.nodes.remove(0);
        assert!(validate_harp_reuse(&omitted, &closure, &ls)
            .unwrap_err()
            .contains("coverage"));
        let mut dangling = manifest.clone();
        dangling.nodes[0].reused_node_id = Some("missing".into());
        assert!(validate_harp_reuse(&dangling, &closure, &ls)
            .unwrap_err()
            .contains("dangling"));
        let mut mismatch = ls.clone();
        mismatch.nodes[0].statement_sha256 = "0".repeat(64);
        assert!(validate_harp_reuse(&manifest, &closure, &mismatch)
            .unwrap_err()
            .contains("mismatch"));
    }

    fn harp_reuse_fixture() -> (RouteManifest, RouteManifest, Vec<String>) {
        let closure = super::HARP_ALLOWED_LS_SUPPORT
            .iter()
            .map(|item| (*item).to_owned())
            .collect::<Vec<_>>();
        let mut harp_nodes = Vec::new();
        let mut ls_nodes = Vec::new();
        for (index, module) in closure.iter().enumerate() {
            let mut node = RouteNode {
                node_id: format!("reuse-{index}"),
                role: NodeRole::LoadBearing,
                declaration: format!("CrouzeixConjecture.support{index}"),
                module_path: format!("{}.lean", module.replace('.', "/")),
                dependency_ids: Vec::new(),
                provenance_kind: ProvenanceKind::ReusedRoute,
                correspondence_kind: CorrespondenceKind::ReusedRoute,
                source_locator: None,
                source_archive_sha256: None,
                source_file_sha256: None,
                source_excerpt_sha256: None,
                source_line_count: None,
                reused_from_route: Some(RouteId::LoristSchwenninger),
                reused_node_id: Some(format!("reuse-{index}")),
                declaration_type_path: format!("evidence/type-{index}.txt"),
                statement_sha256: format!("{:064x}", index + 1),
            };
            harp_nodes.push(node.clone());
            node.provenance_kind = ProvenanceKind::Source;
            node.correspondence_kind = CorrespondenceKind::DirectSource;
            node.source_locator = Some("evidence/source.md#L1-L1".into());
            node.source_file_sha256 = Some("b".repeat(64));
            node.source_excerpt_sha256 = Some("b".repeat(64));
            node.source_line_count = Some(1);
            node.reused_from_route = None;
            node.reused_node_id = None;
            ls_nodes.push(node);
        }
        let make = |route_id, claim_kind, nodes| RouteManifest {
            schema_version: MANIFEST_SCHEMA.into(),
            route_id,
            claim_kind,
            aggregate_module: route_id.aggregate().into(),
            build_target: route_id.aggregate().into(),
            terminal_declaration: "CrouzeixConjecture.terminal".into(),
            terminal_type_sha256: "f".repeat(64),
            consequence_declarations: Vec::new(),
            source_identities: Vec::new(),
            shared_foundation_modules: Vec::new(),
            module_closure: closure.clone(),
            module_closure_sha256: digest(&json!(closure)),
            allowed_axioms: Vec::new(),
            review_path: "evidence/review.json".into(),
            review_sha256: None,
            receipt_path: "evidence/receipt.json".into(),
            receipt_sha256: None,
            nodes,
        };
        (
            make(RouteId::Harp, ClaimKind::Derived, harp_nodes),
            make(
                RouteId::LoristSchwenninger,
                ClaimKind::SourceFaithful,
                ls_nodes,
            ),
            closure,
        )
    }

    fn repository_fixture() -> (TempDir, String) {
        let root = TempDir::new().unwrap();
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(root.path())
            .status()
            .unwrap();
        fs::create_dir_all(root.path().join("formalization/lean/Crouzeix/Jin")).unwrap();
        fs::create_dir_all(root.path().join("formalization/lean/CrouzeixConjecture")).unwrap();
        fs::write(
            root.path().join("formalization/lean/CrouzeixJin.lean"),
            b"import Crouzeix.Jin.Main\n",
        )
        .unwrap();
        fs::write(
            root.path()
                .join("formalization/lean/Crouzeix/Jin/Main.lean"),
            b"import CrouzeixConjecture.Foundation\n",
        )
        .unwrap();
        fs::write(
            root.path()
                .join("formalization/lean/CrouzeixConjecture/Foundation.lean"),
            b"import Mathlib.Data.Matrix.Basic\n",
        )
        .unwrap();
        let (mut manifest, mut receipt, mut review) = fixture();
        manifest["nodes"] = json!([manifest["nodes"][1].clone()]);
        manifest["nodes"][0]["dependency_ids"] = json!([]);
        manifest["nodes"][0]["module_path"] = json!("Crouzeix/Jin/Main.lean");
        manifest["consequence_declarations"] = json!([]);
        manifest["nodes"][0]["role"] = json!("terminal");
        manifest["module_closure"] = json!([
            "Crouzeix.Jin.Main",
            "CrouzeixConjecture.Foundation",
            "CrouzeixJin"
        ]);
        manifest["module_closure_sha256"] = json!(digest(&manifest["module_closure"]));
        manifest["shared_foundation_modules"] =
            json!(["CrouzeixConjecture.Foundation", "CrouzeixJin"]);
        let manifest_path = manifest["receipt_path"].as_str().unwrap().replace(
            "evidence/crouzeix_conjecture/routes/jin/receipt.json",
            "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json",
        );
        fs::create_dir_all(root.path().join(&manifest_path).parent().unwrap()).unwrap();
        fs::write(
            root.path().join(&manifest_path),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(root.path())
            .status()
            .unwrap();
        Command::new("git")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-q",
                "-m",
                "fixture",
            ])
            .current_dir(root.path())
            .status()
            .unwrap();
        let commit = String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(root.path())
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_owned();
        let tree = String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "HEAD^{tree}"])
                .current_dir(root.path())
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim()
        .to_owned();
        receipt["candidate_commit"] = json!(commit);
        receipt["candidate_tree"] = json!(tree);
        receipt["local_closure_modules"] = manifest["module_closure"].clone();
        receipt["local_closure_sha256"] = manifest["module_closure_sha256"].clone();
        receipt["declaration_types"] = json!([receipt["declaration_types"][1].clone()]);
        receipt["axiom_results"] = json!([receipt["axiom_results"][1].clone()]);
        receipt["manifest_sha256"] = json!(manifest_digest(&manifest));
        fs::write(
            root.path().join("formalization/lean/lean-toolchain"),
            b"leanprover/lean4:v4.32.1\n",
        )
        .unwrap();
        fs::write(
            root.path().join("formalization/lean/lake-manifest.json"),
            b"{\"name\":\"fixture\",\"packages\":[]}\n",
        )
        .unwrap();
        let cache_identity = cache_contract_identity(root.path()).unwrap();
        receipt["cache_identity"] = json!(cache_identity.clone());
        receipt["working_directory"] = json!(".");
        receipt["receipt_sha256"] = json!(self_digest(&receipt, "receipt_sha256"));
        review["reviewed_commit"] = receipt["candidate_commit"].clone();
        review["reviewed_tree"] = receipt["candidate_tree"].clone();
        review["manifest_sha256"] = receipt["manifest_sha256"].clone();
        review["review_sha256"] = json!(self_digest(&review, "review_sha256"));
        let receipt_path = manifest["receipt_path"].as_str().unwrap().to_owned();
        let review_path = manifest["review_path"].as_str().unwrap().to_owned();
        fs::create_dir_all(root.path().join(&receipt_path).parent().unwrap()).unwrap();
        fs::create_dir_all(root.path().join(&review_path).parent().unwrap()).unwrap();
        let provider = json!({"schema_version": "crouzeix-route-provider-report/v1", "route_id": "jin", "aggregate_module": "CrouzeixJin", "modules": manifest["module_closure"], "module_closure_sha256": manifest["module_closure_sha256"], "status": "passed"});
        let provider_path = receipt["provider_report_path"].as_str().unwrap();
        fs::write(
            root.path().join(provider_path),
            serde_json::to_vec(&provider).unwrap(),
        )
        .unwrap();
        receipt["provider_report_sha256"] = json!(format!(
            "{:x}",
            Sha256::digest(fs::read(root.path().join(provider_path)).unwrap())
        ));
        let command = json!({
            "schema_version": "crouzeix-route-command/v1",
            "argv": receipt["argv"], "working_directory": ".",
            "aggregate_module": receipt["aggregate_module"], "build_target": receipt["build_target"],
            "cache_identity": cache_identity, "toolchain": "leanprover/lean4:v4.32.1",
        });
        let audit = json!({
            "schema_version": "crouzeix-route-axiom-audit/v1", "route_id": "jin",
            "allowed_axioms": receipt["allowed_axioms"], "results": receipt["axiom_results"], "status": "passed",
        });
        for (path_key, digest_key, bytes) in [
            (
                "command_artifact_path",
                "command_artifact_sha256",
                serde_json::to_vec(&command).unwrap(),
            ),
            (
                "axiom_audit_path",
                "axiom_audit_sha256",
                serde_json::to_vec(&audit).unwrap(),
            ),
            ("stdout_path", "stdout_sha256", b"ok\n".to_vec()),
            ("stderr_path", "stderr_sha256", Vec::new()),
        ] {
            let path = receipt[path_key].as_str().unwrap();
            fs::create_dir_all(root.path().join(path).parent().unwrap()).unwrap();
            fs::write(root.path().join(path), &bytes).unwrap();
            receipt[digest_key] = json!(format!("{:x}", Sha256::digest(&bytes)));
        }
        for item in receipt["declaration_types"].as_array().unwrap() {
            let path = item["type_artifact_path"].as_str().unwrap();
            fs::create_dir_all(root.path().join(path).parent().unwrap()).unwrap();
            fs::write(root.path().join(path), b"forall n, n = n\n").unwrap();
        }
        let source_path = manifest["nodes"][0]["source_locator"]
            .as_str()
            .unwrap()
            .split('#')
            .next()
            .unwrap();
        fs::create_dir_all(root.path().join(source_path).parent().unwrap()).unwrap();
        let source_bytes = b"source line one\nsource line two\nsource line three\n";
        fs::write(root.path().join(source_path), source_bytes).unwrap();
        let source_sha = format!("{:x}", Sha256::digest(source_bytes));
        manifest["source_identities"] = json!([format!("sha256:{source_sha}")]);
        manifest["nodes"][0]["source_file_sha256"] = json!(source_sha);
        manifest["nodes"][0]["source_excerpt_sha256"] =
            json!(format!("{:x}", Sha256::digest(b"source line two\n")));
        manifest["nodes"][0]["source_line_count"] = json!(3);
        let type_bytes = b"forall n, n = n\n";
        let type_digest = format!("{:x}", Sha256::digest(b"forall n, n = n"));
        manifest["nodes"][0]["statement_sha256"] = json!(type_digest.clone());
        manifest["terminal_type_sha256"] = json!(type_digest.clone());
        receipt["declaration_types"][0]["statement_sha256"] = json!(type_digest);
        receipt["declaration_types"][0]["type_artifact_sha256"] =
            json!(format!("{:x}", Sha256::digest(type_bytes)));
        receipt["manifest_sha256"] = json!(manifest_digest(&manifest));
        review["manifest_sha256"] = receipt["manifest_sha256"].clone();
        review["terminal_type_sha256"] = manifest["terminal_type_sha256"].clone();
        receipt["receipt_sha256"] = json!(self_digest(&receipt, "receipt_sha256"));
        review["review_sha256"] = json!(self_digest(&review, "review_sha256"));
        fs::write(
            root.path().join(&receipt_path),
            serde_json::to_vec(&receipt).unwrap(),
        )
        .unwrap();
        fs::write(
            root.path().join(&review_path),
            serde_json::to_vec(&review).unwrap(),
        )
        .unwrap();
        manifest["receipt_sha256"] = json!(format!(
            "{:x}",
            Sha256::digest(fs::read(root.path().join(&receipt_path)).unwrap())
        ));
        manifest["review_sha256"] = json!(format!(
            "{:x}",
            Sha256::digest(fs::read(root.path().join(&review_path)).unwrap())
        ));
        fs::write(
            root.path().join(&manifest_path),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        (root, manifest_path)
    }
}

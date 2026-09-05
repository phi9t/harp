use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use walkdir::WalkDir;

use super::{sha256_file, valid_digest, verify_file};
use crate::error::AppError;

mod route;
mod selection;

pub(super) const ROOT: &str = "evidence/crouzeix_conjecture";

pub(super) const SOURCE_HEADER: &str = "schema_version\treceipt_id\tsource_id\tsource_class\trole\timmutable_identity\tsource_url\tupstream_path\tbytes\tsha256\tlocal_path\tobserved\tlicense_status\tredistribution_status";
pub(super) const VERIFICATION_HEADER: &str = "schema_version\treceipt_id\tsource_id\tsource_commit\tsource_tree\toperation\tcommand_sha256\tacquisition_script_sha256\tnormalization_version\ttoolchain\tmathlib_revision\tobserved_at_utc\texit_code\tresult\tlog_path\tlog_bytes\tlog_sha256";
pub(super) const LOCAL_FORMALIZATION_HEADER: &str = "schema_version\tformalization_id\troute_id\tsource_node_id\tdeclaration_name\tmodule_path\tmodule_sha256\troute_manifest_path\troute_manifest_sha256\troute_receipt_path\troute_receipt_sha256\troute_review_path\troute_review_sha256\tbuild_command_path\tbuild_command_sha256\tbuild_stdout_path\tbuild_stdout_sha256\tbuild_stderr_path\tbuild_stderr_sha256\taxiom_audit_path\taxiom_audit_sha256\tallowed_axioms\tobserved_axioms\tprovider_independence_path\tprovider_independence_sha256\tlean_toolchain\tlean_toolchain_sha256\tlake_manifest_sha256\tstatus";

const LOCAL_FORMALIZATION_ROOT: &str = "local_formalization";
const LOCAL_FORMALIZATION_MANIFEST: &str = "local_formalization/manifest.tsv";
#[cfg(test)]
const LOCAL_BUILD_COMMAND: &str =
    "evidence/crouzeix_conjecture/local_formalization/build/command.json";
#[cfg(test)]
const LOCAL_BUILD_STDOUT: &str =
    "evidence/crouzeix_conjecture/local_formalization/build/stdout.log";
#[cfg(test)]
const LOCAL_BUILD_STDERR: &str =
    "evidence/crouzeix_conjecture/local_formalization/build/stderr.log";
const LOCAL_FORMALIZATION_SCHEMA: &str = "crouzeix-local-formalization/v2";
const LOCAL_FORMALIZATION_CODE: &str = "sources.crouzeix.local_formalization_manifest";
const ALLOWED_AXIOMS: &str = "Classical.choice,Quot.sound,propext";
const LOCAL_MANIFEST_MAX_BYTES: u64 = 1024 * 1024;
const LOCAL_ARTIFACT_MAX_BYTES: u64 = 4 * 1024 * 1024;
const LS_OUTPUT_MAX_BYTES: u64 = 1024 * 1024;
const LS_RECEIPT_MAX_BYTES: u64 = 1024 * 1024;
const LS_ATTEMPT_RECEIPT_MAX_BYTES: u64 = 4 * LS_RECEIPT_MAX_BYTES;
const LS_MAX_ATTEMPTS: usize = 256;
const MAX_PROVIDER_MODULES: usize = 4096;
const MAX_ACTIVE_SOURCE_FILES: usize = 4096;
const MAX_SOURCE_SCAN_ENTRIES: usize = 8192;
const LS_ELAN_HOME: &str = "/private/tmp/harp-mathematical-foundations-elan";
const LS_ELAN_TOOLCHAIN: &str = "leanprover/lean4:v4.32.1";
const LS_TOOLCHAIN_PATH: &str = "/private/tmp/harp-mathematical-foundations-elan/toolchains/leanprover--lean4---v4.32.1/bin:/usr/bin:/bin";
const LS_COMMAND_CACHE_POLICY: &str =
    "requested cached wrapper command; wrapper policy is repository-controlled";
const LS_LEGACY_BUILD_TARGET: &str = "Crouzeix";
const LS_BUILD_TARGET: &str = "CrouzeixLoristSchwenninger";
const LS_LEGACY_AGGREGATE_PATH: &str = "formalization/lean/Crouzeix.lean";
const LS_AGGREGATE_PATH: &str = "formalization/lean/CrouzeixLoristSchwenninger.lean";
const LS_FORBIDDEN_MODULES: [&str; 8] = [
    "CrouzeixConjecture.MainPerturbationReduction",
    "CrouzeixConjecture.CompletionStatement",
    "CrouzeixConjecture.CompletionDiagonalization",
    "CrouzeixConjecture.PositiveRealCompletion",
    "CrouzeixConjecture.FinalTheorems",
    "CrouzeixConjecture.HilbertSpace",
    "CrouzeixConjecture.HilbertSpectralSet",
    "CrouzeixConjecture.RadialOuterReduction",
];
const LS_FORBIDDEN_PREFIXES: [&str; 1] = ["Crouzeix.Jin"];
// Flip this to true in the same change that materializes the genuine published bundle.
const REQUIRE_LOCAL_FORMALIZATION_MANIFEST: bool = true;

#[derive(Clone, Copy)]
struct FormalizationSpec {
    id: &'static str,
    route: &'static str,
    source_node: &'static str,
    declaration: &'static str,
    module_path: &'static str,
}

const FORMALIZATION_SPECS: [FormalizationSpec; 6] = [
    FormalizationSpec {
        id: "harp-closed-numerical-range",
        route: "harp",
        source_node: "harp-closed-range-consequence",
        declaration:
            "CrouzeixConjecture.harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet",
        module_path: "formalization/lean/Crouzeix/Harp/Consequences.lean",
    },
    FormalizationSpec {
        id: "harp-main-theorem",
        route: "harp",
        source_node: "harp-terminal-theorem",
        declaration: "CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem",
        module_path: "formalization/lean/Crouzeix/Harp/MainTheorem.lean",
    },
    FormalizationSpec {
        id: "jin-closed-numerical-range",
        route: "jin",
        source_node: "jin-hilbert-spectral-set-consequence",
        declaration: "CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet",
        module_path: "formalization/lean/CrouzeixConjecture/HilbertSpectralSet.lean",
    },
    FormalizationSpec {
        id: "jin-main-theorem",
        route: "jin",
        source_node: "jin-terminal-crouzeix",
        declaration: "CrouzeixConjecture.crouzeixConjecture",
        module_path: "formalization/lean/Crouzeix/Jin/Terminal.lean",
    },
    FormalizationSpec {
        id: "ls-closed-numerical-range",
        route: "lorist-schwenninger",
        source_node: "-",
        declaration:
            "CrouzeixConjecture.loristSchwenningerClosedOperatorNumericalRange_isTwoSpectralSet",
        module_path: "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean",
    },
    FormalizationSpec {
        id: "ls-main-theorem",
        route: "lorist-schwenninger",
        source_node: "ls-terminal-crouzeix",
        declaration: "CrouzeixConjecture.loristSchwenningerMainTheorem",
        module_path: "formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean",
    },
];

#[derive(Clone, Copy)]
struct LsNodeSpec {
    node_id: &'static str,
    role: &'static str,
    source_locator: &'static str,
    statement_sha256: &'static str,
    dependencies: &'static [&'static str],
    lean_name: &'static str,
    build_target: &'static str,
    legacy_build_target: Option<&'static str>,
}

const LS_NODE_SPECS: [LsNodeSpec; 6] = [
    LsNodeSpec {
        node_id: "ls-equation-one-terminal-bound",
        role: "intermediate",
        source_locator: "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99",
        statement_sha256: "f560ccaca9499f62c8cc30fe1a93338d1024101e129b439fbc880df29cb23311",
        dependencies: &[],
        lean_name: "CrouzeixConjecture.LoristSchwenninger.DilationData.perturbation_mul_target_power_norm_le",
        build_target: "formalization/lean/Crouzeix/LoristSchwenninger/Dilation.lean",
        legacy_build_target: Some(
            "formalization/lean/Crouzeix/LoristSchwenninger/Perturbation.lean",
        ),
    },
    LsNodeSpec {
        node_id: "ls-power-recurrence",
        role: "intermediate",
        source_locator: "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L74-L90",
        statement_sha256: "0b80e8d8bf6e2f5f9d3e3c0fb7e3d2e3f9c9ec7b5ecb392a6d211c7ef7ac0f3d",
        dependencies: &["ls-equation-one-terminal-bound"],
        lean_name: "CrouzeixConjecture.LoristSchwenninger.DilationData.equation_three_lower_bound",
        build_target: "formalization/lean/Crouzeix/LoristSchwenninger/OperatorRecurrence.lean",
        legacy_build_target: None,
    },
    LsNodeSpec {
        node_id: "ls-scalar-contradiction",
        role: "intermediate",
        source_locator: "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98",
        statement_sha256: "966cabe377c00b287b67e3a96de1ab53a12b8c4d35d130c26ae2bc899557f9fa",
        dependencies: &[],
        lean_name: "CrouzeixConjecture.LoristSchwenninger.scalar_endpoint_le_two",
        build_target: "formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean",
        legacy_build_target: Some(
            "formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean",
        ),
    },
    LsNodeSpec {
        node_id: "ls-perturbation-lemma",
        role: "intermediate",
        source_locator: "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99",
        statement_sha256: "0d38ea668014d371f14b3ffb7d5e5bdce7417a2aef2cfd81f663257e72d79d86",
        dependencies: &["ls-power-recurrence", "ls-scalar-contradiction"],
        lean_name: "CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two",
        build_target: "formalization/lean/Crouzeix/LoristSchwenninger/PerturbationLemma.lean",
        legacy_build_target: None,
    },
    LsNodeSpec {
        node_id: "ls-double-layer-realization",
        role: "intermediate",
        source_locator: "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L116-L128",
        statement_sha256: "be32b0c50689d78d037f7359bcd03ab51a0e8ef06b8c5e64d533fbdb2cfaa280",
        dependencies: &["ls-perturbation-lemma"],
        lean_name: "CrouzeixConjecture.LoristSchwenninger.norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary",
        build_target: "formalization/lean/Crouzeix/LoristSchwenninger/ConcreteDilation.lean",
        legacy_build_target: None,
    },
    LsNodeSpec {
        node_id: "ls-terminal-crouzeix",
        role: "terminal",
        source_locator: "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L107-L129",
        statement_sha256: "3a53ccdfc2b6639917cdd774c8d9a2293d690c5c2ef4cf02657e0fd752544a67",
        dependencies: &["ls-double-layer-realization"],
        lean_name: "CrouzeixConjecture.loristSchwenningerMainTheorem",
        build_target: "formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean",
        legacy_build_target: None,
    },
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalBuildCommand {
    schema_version: String,
    argv: Vec<String>,
    cwd: String,
    lean_toolchain: String,
    lean_toolchain_sha256: String,
    lake_manifest_sha256: String,
    wrapper_sha256: String,
    lakefile_sha256: String,
    active_source_closure_sha256: String,
    dependency_cache_metadata_sha256: String,
    required_mathlib_artifacts_sha256: String,
    ls_graph_sha256: String,
    exit_code: i32,
    status: String,
    stdout_path: String,
    stdout_sha256: String,
    stderr_path: String,
    stderr_sha256: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ProviderModule {
    module_name: String,
    module_path: String,
    module_sha256: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
struct ProviderFinding {
    module_name: String,
    line: usize,
    token: String,
    source_line: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderReport {
    schema_version: String,
    route_id: String,
    status: String,
    roots: Vec<String>,
    forbidden_modules: Vec<String>,
    forbidden_prefixes: Vec<String>,
    modules: Vec<ProviderModule>,
    set_options: Vec<ProviderFinding>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LsReceipt {
    schema_version: String,
    route_id: String,
    source_node_id: String,
    status: String,
    reason: String,
    build_target: String,
    expected_lean_declaration: String,
    allowed_axioms: Vec<String>,
    observed_axioms: Vec<String>,
    module_sha256: String,
    task_sha256: String,
    source_slice_sha256: String,
    result_sha256: String,
    command_sha256: String,
    stdout_sha256: String,
    stderr_sha256: String,
    axiom_audit_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LsTask {
    schema_version: String,
    route_id: String,
    source_node_id: String,
    build_target: String,
    expected_lean_declaration: String,
    max_output_bytes: u64,
    timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LsSourceSlice {
    schema_version: String,
    node_id: String,
    source_locator: String,
    statement_sha256: String,
    lean_name: String,
    dependency_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LsResult {
    schema_version: String,
    status: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "schema_version")]
enum LsCommand {
    #[serde(rename = "crouzeix-ls-lean-command/v1")]
    V1(LsCommandV1),
    #[serde(rename = "crouzeix-ls-lean-command/v2")]
    V2(Box<LsCommandV2>),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LsCommandV1 {
    argv: Vec<String>,
    cache_policy: String,
    cwd: String,
    elan_home: String,
    exit_code: i32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LsCommandV2 {
    argv: Vec<String>,
    cwd: String,
    timeout_seconds: u64,
    output_cap_bytes: u64,
    env: LsCommandEnvironment,
    exit_code: i32,
    cache_policy: String,
    wrapper_sha256: String,
    lake_manifest_sha256: String,
    dependency_cache_metadata_sha256: String,
    required_mathlib_artifacts_sha256: String,
    local_source_closure_sha256: String,
    elan_toolchain: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LsCommandEnvironment {
    #[serde(rename = "ELAN_HOME")]
    elan_home: String,
    #[serde(rename = "ELAN_TOOLCHAIN")]
    elan_toolchain: String,
    #[serde(rename = "PATH")]
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LsSourceGraph {
    schema_version: String,
    source_id: String,
    source_identity: String,
    nodes: Vec<LsSourceNode>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LsSourceNode {
    node_id: String,
    role: String,
    source_locator: String,
    statement_sha256: String,
    dependencies: Vec<String>,
    status: String,
    lean_name: String,
    receipt_sha256: Option<String>,
    #[serde(default, deserialize_with = "deserialize_present_optional")]
    blocked_reason: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_present_optional")]
    failed_reason: Option<Option<String>>,
}

fn deserialize_present_optional<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Serialize)]
struct SourceDigestRecord<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    module: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<&'a str>,
    sha256: &'a str,
}

pub(super) struct LocalSourceClosure {
    pub(super) sources: Vec<(String, PathBuf)>,
    pub(super) digest: String,
}

#[derive(PartialEq, Eq)]
struct LsAggregateIdentity {
    command_sha256: String,
    stdout_sha256: String,
    stderr_sha256: String,
    local_source_closure_sha256: String,
}

#[derive(Debug)]
pub(super) struct Report {
    pub(super) source_receipts: usize,
    pub(super) verification_receipts: usize,
}

pub(super) fn verify(repo_root: &Path) -> Result<Report, AppError> {
    let route_evidence = route::verify_published_routes(repo_root)
        .map_err(|detail| invalid("sources.crouzeix.route_contracts", detail))?;
    let local_evidence = verify_local_formalization_bundle_if_present(repo_root)?;
    verify_exact_roster(repo_root, local_evidence.as_ref(), &route_evidence)?;
    let source_receipts = verify_source_manifest(repo_root)?;
    let verification_receipts = verify_verification_manifest(repo_root)?;
    Ok(Report {
        source_receipts,
        verification_receipts,
    })
}

fn verify_exact_roster(
    repo_root: &Path,
    local_evidence: Option<&BTreeSet<String>>,
    route_evidence: &BTreeSet<String>,
) -> Result<(), AppError> {
    let mut expected = [
        "PROVENANCE.md",
        "acquire.sh",
        "source_manifest.tsv",
        "verification/jin-565b6a3-build.log",
        "verification/jin-565b6a3-scan.log",
        "verification/jin-9df0783-build.log",
        "verification/jin-9df0783-scan.log",
        "verification_manifest.tsv",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    if let Some(local_evidence) = local_evidence {
        expected.insert(LOCAL_FORMALIZATION_MANIFEST.to_owned());
        expected.extend(local_evidence.iter().cloned());
    }
    expected.extend(route_evidence.iter().cloned());
    let root = repo_root.join(ROOT);
    let mut actual = BTreeSet::new();
    for entry in WalkDir::new(&root).follow_links(false) {
        let entry = entry.map_err(|error| {
            invalid(
                "sources.crouzeix.roster",
                format!("could not walk {}: {error}", root.display()),
            )
        })?;
        if entry.file_type().is_symlink() {
            return Err(invalid(
                "sources.crouzeix.roster",
                format!(
                    "Crouzeix evidence cannot contain symlink {}",
                    entry.path().display()
                ),
            ));
        }
        if entry.file_type().is_file() {
            let relative = entry
                .path()
                .strip_prefix(&root)
                .expect("walked entry stays beneath root");
            actual.insert(slash_path(relative));
        }
    }
    if actual != expected {
        return Err(invalid(
            "sources.crouzeix.roster",
            format!(
                "Crouzeix evidence file roster differs: expected={expected:?}, actual={actual:?}"
            ),
        ));
    }
    Ok(())
}

fn verify_local_formalization_bundle_if_present(
    repo_root: &Path,
) -> Result<Option<BTreeSet<String>>, AppError> {
    if let Some(files) = selection::verify(repo_root)? {
        return Ok(Some(files));
    }
    let bundle = Path::new(ROOT).join(LOCAL_FORMALIZATION_ROOT);
    let manifest = Path::new(ROOT).join(LOCAL_FORMALIZATION_MANIFEST);
    let absolute_bundle = repo_root.join(&bundle);
    match fs::symlink_metadata(&absolute_bundle) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                &manifest,
                1,
                "local formalization bundle cannot be a symlink",
            ));
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                &manifest,
                1,
                "local formalization bundle must be a directory",
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if REQUIRE_LOCAL_FORMALIZATION_MANIFEST {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    &manifest,
                    1,
                    "required local formalization bundle is missing",
                ));
            }
            return Ok(None);
        }
        Err(error) => {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                &manifest,
                1,
                &format!("cannot inspect local formalization bundle: {error}"),
            ));
        }
    }
    verify_local_formalization_bundle_roster(&absolute_bundle, &manifest)?;
    require_regular_file_without_symlinks(repo_root, &manifest, &manifest, 1, "manifest")?;
    let manifest_bytes = fs::metadata(repo_root.join(&manifest))
        .map_err(|error| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                &manifest,
                1,
                &format!("cannot inspect manifest size: {error}"),
            )
        })?
        .len();
    if manifest_bytes > LOCAL_MANIFEST_MAX_BYTES {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            &manifest,
            1,
            "manifest exceeds the 1 MiB size limit",
        ));
    }
    verify_local_formalization_manifest(repo_root, &manifest).map(Some)
}

fn verify_local_formalization_bundle_roster(
    absolute_bundle: &Path,
    manifest: &Path,
) -> Result<(), AppError> {
    let mut expected = [
        "axioms/",
        "build/",
        "build/command.json",
        "build/stderr.log",
        "build/stdout.log",
        "manifest.tsv",
        "providers/",
        "providers/harp.json",
        "providers/jin.json",
        "providers/lorist-schwenninger.json",
        "routes/",
        "routes/harp.manifest.json",
        "routes/harp.receipt.json",
        "routes/harp.review.json",
        "routes/jin.manifest.json",
        "routes/jin.receipt.json",
        "routes/jin.review.json",
        "routes/lorist-schwenninger.manifest.json",
        "routes/lorist-schwenninger.receipt.json",
        "routes/lorist-schwenninger.review.json",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    expected.extend(
        FORMALIZATION_SPECS
            .iter()
            .map(|spec| format!("axioms/{}.txt", spec.id)),
    );

    let mut actual = BTreeSet::new();
    for entry in WalkDir::new(absolute_bundle)
        .follow_links(false)
        .min_depth(1)
    {
        let entry = entry.map_err(|error| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                1,
                &format!("cannot walk local formalization bundle: {error}"),
            )
        })?;
        let relative = entry
            .path()
            .strip_prefix(absolute_bundle)
            .expect("walked bundle entry stays beneath bundle");
        if entry.file_type().is_symlink() {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                1,
                &format!(
                    "local formalization bundle cannot contain symlink {}",
                    relative.display()
                ),
            ));
        }
        let relative = slash_path(relative);
        if entry.file_type().is_dir() {
            actual.insert(format!("{relative}/"));
        } else if entry.file_type().is_file() {
            actual.insert(relative);
        } else {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                1,
                "local formalization bundle can contain only directories and regular files",
            ));
        }
    }
    if actual != expected {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            1,
            &format!(
                "local formalization bundle roster differs: expected={expected:?}, actual={actual:?}"
            ),
        ));
    }
    Ok(())
}

fn verify_local_formalization_manifest(
    repo_root: &Path,
    relative: &Path,
) -> Result<BTreeSet<String>, AppError> {
    let local_root = relative.parent().expect("bundle manifest has a parent");
    let rows = read_manifest(
        repo_root,
        relative,
        LOCAL_FORMALIZATION_HEADER,
        LOCAL_FORMALIZATION_CODE,
    )?;
    if rows.len() != FORMALIZATION_SPECS.len() + 1 {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            relative,
            1,
            "manifest must contain exactly the six approved formalizations",
        ));
    }

    let toolchain_path = Path::new("formalization/lean/lean-toolchain");
    require_regular_file_without_symlinks(
        repo_root,
        toolchain_path,
        relative,
        1,
        "Lean toolchain identity",
    )?;
    let toolchain_bytes = fs::read(repo_root.join(toolchain_path)).map_err(|error| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            relative,
            1,
            &format!("cannot read Lean toolchain identity: {error}"),
        )
    })?;
    let toolchain_text = String::from_utf8(toolchain_bytes).map_err(|_| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            relative,
            1,
            "Lean toolchain identity must be UTF-8",
        )
    })?;
    let Some(toolchain_identity) = toolchain_text.strip_suffix('\n') else {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            relative,
            1,
            "Lean toolchain identity must be one LF-terminated line",
        ));
    };
    if toolchain_identity.is_empty()
        || toolchain_identity.contains(['\n', '\r', '\0'])
        || toolchain_identity.trim() != toolchain_identity
    {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            relative,
            1,
            "Lean toolchain identity must be one canonical line",
        ));
    }
    let toolchain_digest = sha256_file(&repo_root.join(toolchain_path)).map_err(|error| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            relative,
            1,
            &format!("cannot hash Lean toolchain identity: {}", error.message),
        )
    })?;
    let lake_manifest_path = Path::new("formalization/lean/lake-manifest.json");
    require_regular_file_without_symlinks(
        repo_root,
        lake_manifest_path,
        relative,
        1,
        "Lake dependency manifest",
    )?;
    let lake_manifest_digest =
        sha256_file(&repo_root.join(lake_manifest_path)).map_err(|error| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                1,
                &format!("cannot hash Lake dependency manifest: {}", error.message),
            )
        })?;

    let mut formalization_ids = BTreeSet::new();
    let mut declaration_ids = BTreeSet::new();
    let mut artifact_bindings = ArtifactBindings::default();
    let mut evidence_files = BTreeSet::new();
    let mut previous_sort_key: Option<(String, String)> = None;
    let mut aggregate_build: Option<[String; 6]> = None;

    for (index, columns) in rows.iter().enumerate().skip(1) {
        let line = index + 1;
        if columns.len() != 29 {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "expected twenty-nine columns",
            ));
        }
        require_canonical_fields(LOCAL_FORMALIZATION_CODE, relative, line, columns)?;
        let [schema, formalization_id, route_id, source_node_id, declaration_name, module_path, module_digest, route_manifest_path, route_manifest_digest, route_receipt_path, route_receipt_digest, route_review_path, route_review_digest, command_path, command_digest, stdout_path, stdout_digest, stderr_path, stderr_digest, axiom_path, axiom_digest, allowed_axioms, observed_axioms, provider_path, provider_digest, lean_toolchain, declared_toolchain_digest, declared_lake_digest, status] =
            columns.as_slice()
        else {
            unreachable!("column count checked");
        };

        if schema != LOCAL_FORMALIZATION_SCHEMA {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "unsupported schema version",
            ));
        }
        if !valid_kebab_id(formalization_id) || !formalization_ids.insert(formalization_id.clone())
        {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "formalization_id must be unique canonical lowercase kebab text",
            ));
        }
        let Some(spec) = FORMALIZATION_SPECS
            .iter()
            .find(|candidate| candidate.id == formalization_id)
        else {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "formalization_id is not one of the six approved claim surfaces",
            ));
        };
        if route_id != spec.route
            || source_node_id != spec.source_node
            || declaration_name != spec.declaration
            || module_path != spec.module_path
        {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "route, source node, declaration, and module must match the approved formalization_id",
            ));
        }
        let sort_key = (route_id.clone(), formalization_id.clone());
        if previous_sort_key
            .as_ref()
            .is_some_and(|previous| previous >= &sort_key)
        {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "rows must be strictly sorted by route_id then formalization_id",
            ));
        }
        previous_sort_key = Some(sort_key);
        if status != "passed" {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "status must be passed",
            ));
        }
        if lean_toolchain != toolchain_identity {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "lean_toolchain must equal formalization/lean/lean-toolchain",
            ));
        }
        if declared_toolchain_digest != &toolchain_digest
            || declared_lake_digest != &lake_manifest_digest
        {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "Lean toolchain or Lake manifest digest mismatch",
            ));
        }
        if !valid_lean_name(declaration_name)
            || !declaration_ids.insert((route_id.clone(), declaration_name.clone()))
        {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "declaration_name must be a unique qualified Lean name within its route",
            ));
        }

        let module_path = local_safe_relative(module_path, relative, line)?;
        verify_route_binding(route_id, declaration_name, &module_path, relative, line)?;
        register_artifact_binding(
            &mut artifact_bindings,
            &module_path,
            "module",
            module_digest,
            &format!("route:{route_id}:module:{}", module_path.display()),
            relative,
            line,
        )?;
        verify_digest_bound_file(
            repo_root,
            &module_path,
            module_digest,
            relative,
            line,
            "Lean module",
        )?;
        for (kind, snapshot_path, snapshot_digest) in [
            ("manifest", route_manifest_path, route_manifest_digest),
            ("receipt", route_receipt_path, route_receipt_digest),
            ("review", route_review_path, route_review_digest),
        ] {
            let expected_path =
                slash_path(&local_root.join(format!("routes/{route_id}.{kind}.json")));
            if snapshot_path != &expected_path {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    relative,
                    line,
                    &format!("route {kind} snapshot path mismatch"),
                ));
            }
            verify_digest_bound_file(
                repo_root,
                Path::new(snapshot_path),
                snapshot_digest,
                relative,
                line,
                &format!("route {kind} snapshot"),
            )?;
            evidence_files.insert(slash_path(
                Path::new(snapshot_path)
                    .strip_prefix(ROOT)
                    .expect("canonical route snapshot stays in Crouzeix evidence"),
            ));
            let snapshot = fs::read(repo_root.join(snapshot_path)).map_err(|error| {
                manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    relative,
                    line,
                    &format!("cannot read route {kind} snapshot: {error}"),
                )
            })?;
            let canonical_path = match kind {
                "manifest" => fixture_route_manifest_path_runtime(route_id),
                "receipt" => format!("evidence/crouzeix_conjecture/routes/{route_id}/receipt.json"),
                "review" => {
                    format!("evidence/crouzeix_conjecture/reviews/{route_id}.json")
                }
                _ => unreachable!(),
            };
            let canonical = fs::read(repo_root.join(&canonical_path)).map_err(|error| {
                manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    relative,
                    line,
                    &format!("cannot read canonical route {kind}: {error}"),
                )
            })?;
            if snapshot != canonical {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    relative,
                    line,
                    &format!("route {kind} snapshot differs from canonical route evidence"),
                ));
            }
        }
        if allowed_axioms != ALLOWED_AXIOMS {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "allowed_axioms must equal the fixed policy Classical.choice,Quot.sound,propext",
            ));
        }
        let observed_axioms = parse_axiom_field(observed_axioms, relative, line)?;
        let policy = ALLOWED_AXIOMS.split(',').collect::<BTreeSet<_>>();
        if !observed_axioms
            .iter()
            .all(|axiom| policy.contains(axiom.as_str()))
        {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "observed_axioms contains an axiom outside the fixed policy",
            ));
        }
        let current_aggregate = [
            command_path.clone(),
            command_digest.clone(),
            stdout_path.clone(),
            stdout_digest.clone(),
            stderr_path.clone(),
            stderr_digest.clone(),
        ];
        if let Some(expected) = &aggregate_build {
            if expected != &current_aggregate {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    relative,
                    line,
                    "all rows must bind the same aggregate build command and output artifacts",
                ));
            }
        } else {
            aggregate_build = Some(current_aggregate);
        }
        verify_build_command(
            repo_root,
            command_path,
            lean_toolchain,
            declared_toolchain_digest,
            declared_lake_digest,
            stdout_path,
            stdout_digest,
            stderr_path,
            stderr_digest,
            relative,
            line,
        )?;

        let artifacts = [
            (
                "build_command",
                command_path,
                command_digest,
                "aggregate-build".to_owned(),
            ),
            (
                "build_stdout",
                stdout_path,
                stdout_digest,
                "aggregate-build".to_owned(),
            ),
            (
                "build_stderr",
                stderr_path,
                stderr_digest,
                "aggregate-build".to_owned(),
            ),
            (
                "axiom_audit",
                axiom_path,
                axiom_digest,
                format!("formalization:{formalization_id}"),
            ),
            (
                "provider_independence",
                provider_path,
                provider_digest,
                format!("route:{route_id}"),
            ),
        ];
        let mut verified_paths = BTreeMap::new();
        for (role, path_text, digest, owner) in artifacts {
            let path = local_safe_relative(path_text, relative, line)?;
            if !path.starts_with(local_root) {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    relative,
                    line,
                    &format!("{role}_path must stay beneath {}/", local_root.display()),
                ));
            }
            register_artifact_binding(
                &mut artifact_bindings,
                &path,
                role,
                digest,
                &owner,
                relative,
                line,
            )?;
            if verified_paths.insert(path.clone(), role).is_some() {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    relative,
                    line,
                    "one artifact path cannot serve multiple fields in a row",
                ));
            }
            verify_digest_bound_file(repo_root, &path, digest, relative, line, role)?;
            evidence_files.insert(slash_path(
                path.strip_prefix(ROOT)
                    .expect("local evidence scope checked"),
            ));
        }

        let axiom_path = local_safe_relative(axiom_path, relative, line)?;
        verify_axiom_audit(
            repo_root,
            &axiom_path,
            declaration_name,
            &observed_axioms,
            relative,
            line,
        )?;
        verify_provider_report(repo_root, provider_path, route_id, relative, line)?;

        let expected_axiom_path =
            slash_path(&local_root.join(format!("axioms/{formalization_id}.txt")));
        let expected_provider_path =
            slash_path(&local_root.join(format!("providers/{route_id}.json")));
        if command_path != &slash_path(&local_root.join("build/command.json"))
            || stdout_path != &slash_path(&local_root.join("build/stdout.log"))
            || stderr_path != &slash_path(&local_root.join("build/stderr.log"))
            || axiom_path.to_str() != Some(&expected_axiom_path)
            || provider_path != &expected_provider_path
        {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                relative,
                line,
                "artifact fields must use their canonical bundle paths",
            ));
        }
    }
    let expected_ids = FORMALIZATION_SPECS
        .iter()
        .map(|spec| spec.id.to_owned())
        .collect::<BTreeSet<_>>();
    if formalization_ids != expected_ids {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            relative,
            1,
            "manifest must contain exactly the six approved formalization IDs",
        ));
    }
    verify_all_ls_receipts(repo_root, relative, 1)?;
    Ok(evidence_files)
}

#[derive(Default)]
struct ArtifactBindings {
    by_path: BTreeMap<PathBuf, (String, String, String)>,
    by_digest: BTreeMap<String, (String, String)>,
}

#[allow(clippy::too_many_arguments)]
fn register_artifact_binding(
    bindings: &mut ArtifactBindings,
    path: &Path,
    role: &str,
    digest: &str,
    owner: &str,
    manifest: &Path,
    line: usize,
) -> Result<(), AppError> {
    let binding = (role.to_owned(), digest.to_owned(), owner.to_owned());
    if let Some(existing) = bindings.by_path.get(path) {
        if existing != &binding {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "artifact path cannot alias unrelated roles or identities",
            ));
        }
    } else {
        bindings.by_path.insert(path.to_path_buf(), binding);
    }

    let digest_binding = (role.to_owned(), owner.to_owned());
    if let Some(existing) = bindings.by_digest.get(digest) {
        let same_aggregate = existing.1 == "aggregate-build" && owner == "aggregate-build";
        if existing != &digest_binding && !same_aggregate {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "artifact digest cannot alias unrelated roles or identities",
            ));
        }
    } else {
        bindings.by_digest.insert(digest.to_owned(), digest_binding);
    }
    Ok(())
}

fn local_safe_relative(value: &str, manifest: &Path, line: usize) -> Result<PathBuf, AppError> {
    let path = safe_relative(value)
        .map_err(|detail| manifest_error(LOCAL_FORMALIZATION_CODE, manifest, line, &detail))?;
    if value.contains('\\')
        || value.split('/').any(|component| component.is_empty())
        || slash_path(&path) != value
    {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("path must be repository-relative normal POSIX text: {value}"),
        ));
    }
    Ok(path)
}

fn require_regular_file_without_symlinks(
    repo_root: &Path,
    relative: &Path,
    manifest: &Path,
    line: usize,
    label: &str,
) -> Result<(), AppError> {
    let component_count = relative.components().count();
    let mut current = repo_root.to_path_buf();
    for (index, component) in relative.components().enumerate() {
        let Component::Normal(part) = component else {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("{label} path must be repository-relative normal text"),
            ));
        };
        current.push(part);
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!(
                    "{label} is missing or unreadable at {}: {error}",
                    current.display()
                ),
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!(
                    "{label} path cannot contain a symlink: {}",
                    current.display()
                ),
            ));
        }
        let is_leaf = index + 1 == component_count;
        if (is_leaf && !metadata.is_file()) || (!is_leaf && !metadata.is_dir()) {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!(
                    "{label} must resolve to a regular file: {}",
                    relative.display()
                ),
            ));
        }
        if is_leaf && metadata.nlink() != 1 {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("{label} cannot be a hardlink alias: {}", relative.display()),
            ));
        }
    }
    Ok(())
}

fn verify_digest_bound_file(
    repo_root: &Path,
    path: &Path,
    digest: &str,
    manifest: &Path,
    line: usize,
    label: &str,
) -> Result<(), AppError> {
    verify_digest_bound_file_with_limit(
        repo_root,
        path,
        digest,
        manifest,
        line,
        label,
        LOCAL_ARTIFACT_MAX_BYTES,
    )
}

fn verify_digest_bound_file_with_limit(
    repo_root: &Path,
    path: &Path,
    digest: &str,
    manifest: &Path,
    line: usize,
    label: &str,
    maximum: u64,
) -> Result<(), AppError> {
    if !valid_digest(digest) {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("{label} sha256 must be lowercase 64-hex"),
        ));
    }
    require_regular_file_without_symlinks(repo_root, path, manifest, line, label)?;
    let length = fs::metadata(repo_root.join(path))
        .map_err(|error| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("cannot inspect {label} size: {error}"),
            )
        })?
        .len();
    if length > maximum {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("{label} exceeds the byte limit of {maximum}"),
        ));
    }
    let actual = sha256_file(&repo_root.join(path)).map_err(|error| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("cannot hash {label}: {}", error.message),
        )
    })?;
    if actual != digest {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("{label} digest mismatch: {}", path.display()),
        ));
    }
    Ok(())
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_records_digest(
    kind: &str,
    key: &str,
    records: &[(String, String)],
    manifest: &Path,
    line: usize,
) -> Result<String, AppError> {
    let records = records
        .iter()
        .map(|(identity, digest)| SourceDigestRecord {
            module: (key == "module").then_some(identity.as_str()),
            path: (key == "path").then_some(identity.as_str()),
            sha256: digest,
        })
        .collect::<Vec<_>>();
    let value = match kind {
        "sources" => serde_json::json!({"sources": records}),
        "artifacts" => serde_json::json!({"artifacts": records}),
        _ => unreachable!("fixed canonical digest kind"),
    };
    let mut bytes = serde_json::to_vec(&value).map_err(|error| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("cannot serialize canonical {kind} digest: {error}"),
        )
    })?;
    bytes.push(b'\n');
    Ok(digest_bytes(&bytes))
}

fn bounded_lean_source_paths(
    repo_root: &Path,
    manifest: &Path,
    line: usize,
) -> Result<Vec<PathBuf>, AppError> {
    let lean_root = Path::new("formalization/lean");
    let aggregate = lean_root.join("Crouzeix.lean");
    let mut sources = Vec::new();
    match fs::symlink_metadata(repo_root.join(&aggregate)) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "active Crouzeix source cannot be a symlink: formalization/lean/Crouzeix.lean",
            ));
        }
        Ok(metadata) if metadata.is_file() => sources.push(aggregate),
        Ok(_) => {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "active Crouzeix aggregate source must be a regular file",
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("cannot inspect active Crouzeix aggregate source: {error}"),
            ));
        }
    }

    let mut entries_seen = 0usize;
    for source_root in [
        lean_root.join("Crouzeix"),
        lean_root.join("CrouzeixConjecture"),
    ] {
        let absolute = repo_root.join(&source_root);
        match fs::symlink_metadata(&absolute) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!(
                        "active Crouzeix source tree contains a symlink: {}",
                        source_root.display()
                    ),
                ));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!(
                        "active Crouzeix source root must be a directory: {}",
                        source_root.display()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!("cannot inspect active Crouzeix source root: {error}"),
                ));
            }
        }
        for entry in WalkDir::new(&absolute).follow_links(false).min_depth(1) {
            let entry = entry.map_err(|error| {
                manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!("cannot walk active Crouzeix source tree: {error}"),
                )
            })?;
            entries_seen += 1;
            if entries_seen > MAX_SOURCE_SCAN_ENTRIES {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    "active Crouzeix source tree entry count exceeds safety cap",
                ));
            }
            let relative = entry
                .path()
                .strip_prefix(repo_root)
                .expect("walked source remains beneath repository");
            if entry.file_type().is_symlink() {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!(
                        "active Crouzeix source tree contains a symlink: {}",
                        relative.display()
                    ),
                ));
            }
            if entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|extension| extension.to_str())
                    == Some("lean")
            {
                if sources.len() >= MAX_ACTIVE_SOURCE_FILES {
                    return Err(manifest_error(
                        LOCAL_FORMALIZATION_CODE,
                        manifest,
                        line,
                        "active Crouzeix source count exceeds safety cap",
                    ));
                }
                sources.push(relative.to_path_buf());
            } else if !entry.file_type().is_file() && !entry.file_type().is_dir() {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!(
                        "active Crouzeix source tree contains an unsafe entry: {}",
                        relative.display()
                    ),
                ));
            }
        }
    }
    sources.sort();
    Ok(sources)
}

fn source_records_digest(
    repo_root: &Path,
    sources: &[(String, PathBuf)],
    key: &str,
    manifest: &Path,
    line: usize,
) -> Result<String, AppError> {
    let mut records = Vec::with_capacity(sources.len());
    for (identity, relative) in sources {
        verify_digest_input_file(repo_root, relative, manifest, line, "Lean source")?;
        records.push((
            identity.clone(),
            sha256_file(&repo_root.join(relative)).map_err(|error| {
                manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!("cannot hash Lean source: {}", error.message),
                )
            })?,
        ));
    }
    canonical_records_digest("sources", key, &records, manifest, line)
}

fn verify_digest_input_file(
    repo_root: &Path,
    relative: &Path,
    manifest: &Path,
    line: usize,
    label: &str,
) -> Result<(), AppError> {
    require_regular_file_without_symlinks(repo_root, relative, manifest, line, label)?;
    let length = fs::metadata(repo_root.join(relative))
        .map_err(|error| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("cannot inspect {label}: {error}"),
            )
        })?
        .len();
    if length > LOCAL_ARTIFACT_MAX_BYTES {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("{label} exceeds the 4 MiB size limit"),
        ));
    }
    Ok(())
}

fn module_name_from_source_path(relative: &Path) -> Option<String> {
    let relative = relative.strip_prefix("formalization/lean").ok()?;
    if relative
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("lean")
    {
        return None;
    }
    let mut module = relative.to_path_buf();
    module.set_extension("");
    Some(
        module
            .components()
            .map(|component| component.as_os_str().to_str())
            .collect::<Option<Vec<_>>>()?
            .join("."),
    )
}

fn local_module_name(relative: &Path) -> Option<String> {
    module_name_from_source_path(relative).filter(|module| valid_lean_name(module))
}

fn indexed_local_modules(
    repo_root: &Path,
    manifest: &Path,
    line: usize,
) -> Result<BTreeMap<String, PathBuf>, AppError> {
    let lean_root = repo_root.join("formalization/lean");
    require_directory_without_symlinks(
        repo_root,
        Path::new("formalization/lean"),
        manifest,
        line,
        "LS Lean source root",
    )?;
    let mut modules = BTreeMap::new();
    let mut entries_seen = 0usize;
    for entry in WalkDir::new(&lean_root)
        .follow_links(false)
        .min_depth(1)
        .into_iter()
        .filter_entry(|entry| entry.depth() != 1 || entry.file_name() != ".lake")
    {
        let entry = entry.map_err(|error| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("cannot walk LS Lean source tree: {error}"),
            )
        })?;
        entries_seen += 1;
        if entries_seen > MAX_PROVIDER_MODULES {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "LS Lean source entry count exceeds cap",
            ));
        }
        let relative = entry
            .path()
            .strip_prefix(repo_root)
            .expect("walked source remains beneath repository");
        if entry.file_type().is_symlink() {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("LS Lean source contains symlink: {}", relative.display()),
            ));
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(module) = local_module_name(relative) else {
            continue;
        };
        if modules
            .insert(module.clone(), relative.to_path_buf())
            .is_some()
        {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("duplicate LS Lean module: {module}"),
            ));
        }
    }
    Ok(modules)
}

fn ls_source_closure(
    repo_root: &Path,
    manifest: &Path,
    line: usize,
) -> Result<LocalSourceClosure, AppError> {
    local_source_closure(repo_root, &[LS_AGGREGATE_PATH], manifest, line)
}

fn ls_aggregate_modules(
    repo_root: &Path,
    aggregate_path: &str,
    manifest: &Path,
    line: usize,
) -> Result<BTreeSet<String>, AppError> {
    Ok(
        local_source_closure(repo_root, &[aggregate_path], manifest, line)?
            .sources
            .into_iter()
            .map(|(module, _)| module)
            .collect(),
    )
}

pub(super) fn local_source_closure(
    repo_root: &Path,
    roots: &[&str],
    manifest: &Path,
    line: usize,
) -> Result<LocalSourceClosure, AppError> {
    let modules = indexed_local_modules(repo_root, manifest, line)?;
    let mut pending = roots
        .iter()
        .rev()
        .map(|root| {
            local_module_name(Path::new(root))
                .expect("static LS closure roots are valid module paths")
        })
        .collect::<Vec<_>>();
    for root in &pending {
        if !modules.contains_key(root) {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("LS source closure root does not exist: {root}"),
            ));
        }
    }
    let mut visited = BTreeSet::new();
    while let Some(module) = pending.pop() {
        if !visited.insert(module.clone()) {
            continue;
        }
        if visited.len() > MAX_PROVIDER_MODULES {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "LS local source closure exceeds the module cap",
            ));
        }
        let relative = modules
            .get(&module)
            .expect("visited modules came from index");
        let source = read_utf8_bounded(repo_root, relative, manifest, line, "LS Lean module")?;
        let active = mask_lean_source(&source, &module, manifest, line)?;
        for imported in lean_header_imports(&active, &module, manifest, line)? {
            if !modules.contains_key(&imported) {
                if managed_local_module(&imported) {
                    return Err(manifest_error(
                        LOCAL_FORMALIZATION_CODE,
                        manifest,
                        line,
                        &format!("missing managed local import: {imported}"),
                    ));
                }
                continue;
            }
            if !visited.contains(&imported) {
                pending.push(imported);
            }
        }
    }
    let sources = visited
        .into_iter()
        .map(|module| {
            let relative = modules
                .get(&module)
                .expect("visited modules came from index")
                .clone();
            (module, relative)
        })
        .collect::<Vec<_>>();
    let digest = source_records_digest(repo_root, &sources, "module", manifest, line)?;
    Ok(LocalSourceClosure { sources, digest })
}

fn required_mathlib_artifacts_digest(
    repo_root: &Path,
    sources: &[(String, PathBuf)],
    manifest: &Path,
    line: usize,
) -> Result<String, AppError> {
    let mut modules = BTreeSet::new();
    for (module, relative) in sources {
        let source = read_utf8_bounded(repo_root, relative, manifest, line, "Lean source")?;
        let active = mask_lean_source(&source, module, manifest, line)?;
        for imported in lean_header_imports(&active, module, manifest, line)? {
            if imported.starts_with("Mathlib") {
                if !valid_lean_name(&imported) {
                    return Err(manifest_error(
                        LOCAL_FORMALIZATION_CODE,
                        manifest,
                        line,
                        &format!("local source has invalid Mathlib import: {imported}"),
                    ));
                }
                modules.insert(imported);
            }
        }
    }
    let artifact_root = fs::canonicalize(
        repo_root.join("formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean"),
    )
    .map_err(|error| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("cannot resolve required Mathlib artifact root: {error}"),
        )
    })?;
    if !artifact_root.is_dir() {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "required Mathlib artifact root must be a directory",
        ));
    }
    let mut records = Vec::with_capacity(modules.len());
    for module in modules {
        let artifact = artifact_root
            .join(module.replace('.', "/"))
            .with_extension("olean");
        let parent = fs::canonicalize(artifact.parent().expect("artifact has a parent")).map_err(
            |error| {
                manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!("cannot resolve required Mathlib artifact parent: {error}"),
                )
            },
        )?;
        if !parent.starts_with(&artifact_root) {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "required Mathlib artifact escapes its root",
            ));
        }
        let artifact = parent.join(artifact.file_name().expect("artifact has a file name"));
        let metadata = fs::symlink_metadata(&artifact).map_err(|error| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("cannot inspect required Mathlib artifact: {error}"),
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "required Mathlib artifact must be a regular file",
            ));
        }
        if metadata.len() > LOCAL_ARTIFACT_MAX_BYTES {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "required Mathlib artifact exceeds the 4 MiB size limit",
            ));
        }
        records.push((
            module,
            sha256_file(&artifact).map_err(|error| {
                manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!("cannot hash required Mathlib artifact: {}", error.message),
                )
            })?,
        ));
    }
    canonical_records_digest("artifacts", "module", &records, manifest, line)
}

fn require_directory_without_symlinks(
    repo_root: &Path,
    relative: &Path,
    manifest: &Path,
    line: usize,
    label: &str,
) -> Result<(), AppError> {
    let mut current = repo_root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("{label} path must be repository-relative normal text"),
            ));
        };
        current.push(part);
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!(
                    "{label} is missing or unreadable at {}: {error}",
                    current.display()
                ),
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!(
                    "{label} path cannot contain a symlink: {}",
                    current.display()
                ),
            ));
        }
        if !metadata.is_dir() {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!(
                    "{label} must resolve to a directory: {}",
                    relative.display()
                ),
            ));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn verify_build_command(
    repo_root: &Path,
    command_path: &str,
    lean_toolchain: &str,
    toolchain_digest: &str,
    lake_manifest_digest: &str,
    stdout_path: &str,
    stdout_digest: &str,
    stderr_path: &str,
    stderr_digest: &str,
    manifest: &Path,
    line: usize,
) -> Result<(), AppError> {
    let command_path = local_safe_relative(command_path, manifest, line)?;
    let command: LocalBuildCommand = read_strict_json_bounded(
        repo_root,
        &command_path,
        manifest,
        line,
        "aggregate build command",
    )?;
    let digest_fields = [
        command.wrapper_sha256.as_str(),
        command.lakefile_sha256.as_str(),
        command.active_source_closure_sha256.as_str(),
        command.dependency_cache_metadata_sha256.as_str(),
        command.required_mathlib_artifacts_sha256.as_str(),
        command.ls_graph_sha256.as_str(),
    ];
    if command.schema_version != "crouzeix-local-build-command/v1"
        || command.argv != ["scripts/check_lean_library.sh", "Crouzeix"]
        || command.cwd != "."
        || command.lean_toolchain != lean_toolchain
        || command.lean_toolchain_sha256 != toolchain_digest
        || command.lake_manifest_sha256 != lake_manifest_digest
        || command.exit_code != 0
        || command.status != "passed"
        || command.stdout_path != stdout_path
        || command.stdout_sha256 != stdout_digest
        || command.stderr_path != stderr_path
        || command.stderr_sha256 != stderr_digest
        || digest_fields.iter().any(|digest| !valid_digest(digest))
    {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "aggregate build command must bind the exact successful Crouzeix command, toolchain, dependency manifest, and output artifacts",
        ));
    }
    verify_digest_bound_file(
        repo_root,
        Path::new("scripts/check_lean_library.sh"),
        &command.wrapper_sha256,
        manifest,
        line,
        "aggregate build wrapper",
    )?;
    verify_digest_bound_file(
        repo_root,
        Path::new("formalization/lean/lakefile.toml"),
        &command.lakefile_sha256,
        manifest,
        line,
        "Lean Lake configuration",
    )?;
    verify_digest_bound_file(
        repo_root,
        Path::new(
            "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json",
        ),
        &command.ls_graph_sha256,
        manifest,
        line,
        "LS source graph",
    )?;
    let source_paths = bounded_lean_source_paths(repo_root, manifest, line)?;
    let mut active_sources = Vec::with_capacity(source_paths.len());
    let mut mathlib_sources = Vec::with_capacity(source_paths.len());
    for path in source_paths {
        let Some(module) = module_name_from_source_path(&path) else {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "active Crouzeix source path must be UTF-8 beneath formalization/lean",
            ));
        };
        let Some(path_text) = path
            .components()
            .map(|component| component.as_os_str().to_str())
            .collect::<Option<Vec<_>>>()
            .map(|components| components.join("/"))
        else {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "active Crouzeix source path must be UTF-8 beneath formalization/lean",
            ));
        };
        active_sources.push((path_text, path.clone()));
        mathlib_sources.push((module, path));
    }
    let active_source_digest =
        source_records_digest(repo_root, &active_sources, "path", manifest, line)?;
    let mathlib_digest =
        required_mathlib_artifacts_digest(repo_root, &mathlib_sources, manifest, line)?;
    if command.active_source_closure_sha256 != active_source_digest
        || command.required_mathlib_artifacts_sha256 != mathlib_digest
    {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "aggregate build command source closure or required Mathlib artifact digest mismatch",
        ));
    }
    let stdout_path = local_safe_relative(stdout_path, manifest, line)?;
    let stdout = read_utf8_bounded(
        repo_root,
        &stdout_path,
        manifest,
        line,
        "aggregate build stdout",
    )?;
    verify_success_stdout(
        &stdout,
        "Crouzeix",
        "aggregate build stdout does not contain a successful Crouzeix build report",
        manifest,
        line,
    )
}

fn verify_all_ls_receipts(repo_root: &Path, manifest: &Path, line: usize) -> Result<(), AppError> {
    let graph_path = Path::new(
        "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json",
    );
    let graph: LsSourceGraph =
        read_strict_json_bounded(repo_root, graph_path, manifest, line, "LS source graph")?;
    if graph.schema_version != "crouzeix-ls-source-graph/v1"
        || graph.source_id != "LS-ARXIV-V1"
        || graph.source_identity != "arxiv:2608.03841v1"
        || graph.nodes.len() != LS_NODE_SPECS.len()
    {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "LS source graph identity is invalid",
        ));
    }

    let mut v2_aggregate_identities = Vec::new();
    for (node, spec) in graph.nodes.iter().zip(LS_NODE_SPECS.iter()) {
        let expected_dependencies = spec
            .dependencies
            .iter()
            .map(|dependency| (*dependency).to_owned())
            .collect::<Vec<_>>();
        if node.node_id != spec.node_id
            || node.role != spec.role
            || node.source_locator != spec.source_locator
            || node.statement_sha256 != spec.statement_sha256
            || node.dependencies != expected_dependencies
            || node.lean_name != spec.lean_name
            || node.status != "passed"
            || node
                .receipt_sha256
                .as_deref()
                .is_none_or(|digest| !valid_digest(digest))
            || node.blocked_reason.is_some()
            || node.failed_reason.is_some()
        {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "local formalization requires the exact canonical six-node all-passed LS source graph",
            ));
        }
    }

    for (node, spec) in graph.nodes.iter().zip(LS_NODE_SPECS.iter()) {
        let receipt_digest = node
            .receipt_sha256
            .as_deref()
            .expect("canonical passed graph node digest checked");
        let receipt_path = resolve_ls_receipt_attempt(
            repo_root,
            node.node_id.as_str(),
            receipt_digest,
            manifest,
            line,
        )?;
        v2_aggregate_identities.push(verify_ls_receipt(
            repo_root,
            &receipt_path,
            receipt_digest,
            node,
            spec,
            manifest,
            line,
        )?);
    }

    if v2_aggregate_identities.iter().all(Option::is_some) {
        let first = v2_aggregate_identities[0]
            .as_ref()
            .expect("all v2 receipt identities checked");
        if v2_aggregate_identities
            .iter()
            .skip(1)
            .any(|identity| identity.as_ref() != Some(first))
        {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "canonical six LS v2 receipts must share identical aggregate build evidence",
            ));
        }
    }
    Ok(())
}

fn resolve_ls_receipt_attempt(
    repo_root: &Path,
    node_id: &str,
    receipt_digest: &str,
    manifest: &Path,
    line: usize,
) -> Result<PathBuf, AppError> {
    let node_root = Path::new(
        "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/proof-slices",
    )
    .join(node_id);
    require_directory_without_symlinks(
        repo_root,
        &node_root,
        manifest,
        line,
        "LS proof-slice directory",
    )?;
    let mut entry_count = 0usize;
    let mut aggregate_receipt_bytes = 0u64;
    let mut matches = Vec::new();
    let entries = fs::read_dir(repo_root.join(&node_root)).map_err(|error| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("cannot inspect LS proof-slice directory for node {node_id}: {error}"),
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("cannot inspect LS attempt for node {node_id}: {error}"),
            )
        })?;
        entry_count += 1;
        if entry_count > LS_MAX_ATTEMPTS {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("LS attempt count exceeds cap for node {node_id}"),
            ));
        }
        let Some(attempt_name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if !attempt_name.starts_with("attempt-") {
            continue;
        }
        let file_type = entry.file_type().map_err(|error| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("cannot inspect LS attempt {attempt_name}: {error}"),
            )
        })?;
        if file_type.is_symlink() {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("LS attempt cannot be a symlink: {attempt_name}"),
            ));
        }
        if !file_type.is_dir() {
            continue;
        }
        let receipt_path = node_root.join(&attempt_name).join("receipt.json");
        let metadata = match fs::symlink_metadata(repo_root.join(&receipt_path)) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => continue,
        };
        if metadata.file_type().is_symlink() {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "LS attempt receipt cannot be a symlink",
            ));
        }
        if !metadata.is_file() || metadata.len() > LS_RECEIPT_MAX_BYTES {
            continue;
        }
        aggregate_receipt_bytes = aggregate_receipt_bytes
            .checked_add(metadata.len())
            .ok_or_else(|| {
                manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!("LS attempt aggregate receipt bytes exceed cap for node {node_id}"),
                )
            })?;
        if aggregate_receipt_bytes > LS_ATTEMPT_RECEIPT_MAX_BYTES {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("LS attempt aggregate receipt bytes exceed cap for node {node_id}"),
            ));
        }
        let bytes = match fs::read(repo_root.join(&receipt_path)) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        if digest_bytes(&bytes) == receipt_digest {
            if !valid_attempt_name(&attempt_name) {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!("matching LS attempt directory name is unsafe: {attempt_name}"),
                ));
            }
            matches.push(receipt_path);
        }
    }
    if matches.len() != 1 {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!(
                "exactly one LS committed receipt_sha256 must match an attempt for node {node_id}; found {}",
                matches.len()
            ),
        ));
    }
    Ok(matches.pop().expect("single matching LS receipt"))
}

fn verify_ls_attempt_roster(
    repo_root: &Path,
    attempt_root: &Path,
    manifest: &Path,
    line: usize,
) -> Result<(), AppError> {
    let expected_files = [
        "receipt.json",
        "task.json",
        "source-slice.json",
        "result.json",
        "build/command.json",
        "build/stdout.log",
        "build/stderr.log",
        "build/axioms.txt",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<BTreeSet<_>>();
    let expected_directories = ["build"].into_iter().map(str::to_owned).collect();
    let mut actual_files = BTreeSet::new();
    let mut actual_directories = BTreeSet::new();
    for entry in WalkDir::new(repo_root.join(attempt_root))
        .follow_links(false)
        .min_depth(1)
    {
        let entry = entry.map_err(|error| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("cannot inspect LS attempt members: {error}"),
            )
        })?;
        let relative = entry
            .path()
            .strip_prefix(repo_root.join(attempt_root))
            .expect("walked LS attempt remains beneath its root");
        let relative = slash_path(relative);
        if entry.file_type().is_symlink() {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("LS attempt member {relative} cannot be a symlink"),
            ));
        }
        if entry.file_type().is_dir() {
            actual_directories.insert(relative);
        } else if entry.file_type().is_file() {
            actual_files.insert(relative);
        } else {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "LS attempt members must be regular files or directories",
            ));
        }
    }
    if actual_files != expected_files || actual_directories != expected_directories {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!(
                "LS attempt member roster differs: expected_files={expected_files:?}, actual_files={actual_files:?}, expected_directories={expected_directories:?}, actual_directories={actual_directories:?}"
            ),
        ));
    }
    Ok(())
}

fn verify_ls_receipt(
    repo_root: &Path,
    receipt_path: &Path,
    receipt_digest: &str,
    node: &LsSourceNode,
    spec: &LsNodeSpec,
    manifest: &Path,
    line: usize,
) -> Result<Option<LsAggregateIdentity>, AppError> {
    let Some(attempt_root) = receipt_path.parent() else {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "LS receipt path has no attempt directory",
        ));
    };
    verify_ls_attempt_roster(repo_root, attempt_root, manifest, line)?;
    verify_digest_bound_file_with_limit(
        repo_root,
        receipt_path,
        receipt_digest,
        manifest,
        line,
        "LS proof receipt",
        LS_RECEIPT_MAX_BYTES,
    )?;
    let receipt: LsReceipt =
        read_strict_json_bounded(repo_root, receipt_path, manifest, line, "LS proof receipt")?;
    let allowed = ALLOWED_AXIOMS
        .split(',')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if receipt.schema_version != "crouzeix-ls-proof-slice-receipt/v1"
        || receipt.route_id != "lorist-schwenninger"
        || receipt.source_node_id != node.node_id
        || receipt.status != "passed"
        || !valid_bounded_json_string(&receipt.reason, 1, 4_096)
        || receipt.expected_lean_declaration != node.lean_name
        || receipt.allowed_axioms != allowed
        || !valid_ordered_axioms(&receipt.observed_axioms)
        || !receipt
            .observed_axioms
            .iter()
            .all(|axiom| allowed.contains(axiom))
        || !valid_digest(&receipt.module_sha256)
    {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "LS proof receipt identity or ordered axiom policy does not match its canonical graph node",
        ));
    }
    let members = [
        (
            "task.json",
            receipt.task_sha256.as_str(),
            LOCAL_ARTIFACT_MAX_BYTES,
        ),
        (
            "source-slice.json",
            receipt.source_slice_sha256.as_str(),
            LOCAL_ARTIFACT_MAX_BYTES,
        ),
        (
            "result.json",
            receipt.result_sha256.as_str(),
            LOCAL_ARTIFACT_MAX_BYTES,
        ),
        (
            "build/command.json",
            receipt.command_sha256.as_str(),
            LOCAL_ARTIFACT_MAX_BYTES,
        ),
        (
            "build/stdout.log",
            receipt.stdout_sha256.as_str(),
            LS_OUTPUT_MAX_BYTES,
        ),
        (
            "build/stderr.log",
            receipt.stderr_sha256.as_str(),
            LS_OUTPUT_MAX_BYTES,
        ),
        (
            "build/axioms.txt",
            receipt.axiom_audit_sha256.as_str(),
            LOCAL_ARTIFACT_MAX_BYTES,
        ),
    ];
    for (member, digest, maximum) in members {
        verify_digest_bound_file_with_limit(
            repo_root,
            &attempt_root.join(member),
            digest,
            manifest,
            line,
            "LS receipt member",
            maximum,
        )?;
    }
    let task: LsTask = read_strict_json_bounded(
        repo_root,
        &attempt_root.join("task.json"),
        manifest,
        line,
        "LS task",
    )?;
    if task.schema_version != "crouzeix-ls-proof-slice-task/v1"
        || task.route_id != receipt.route_id
        || task.source_node_id != node.node_id
        || task.build_target != receipt.build_target
        || task.expected_lean_declaration != receipt.expected_lean_declaration
        || task.max_output_bytes != LS_OUTPUT_MAX_BYTES
        || task.timeout_seconds != 3_600
    {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "LS task does not match the proof receipt",
        ));
    }
    let source_slice: LsSourceSlice = read_strict_json_bounded(
        repo_root,
        &attempt_root.join("source-slice.json"),
        manifest,
        line,
        "LS source slice",
    )?;
    if source_slice.schema_version != "crouzeix-ls-source-slice/v1"
        || source_slice.node_id != node.node_id
        || source_slice.lean_name != node.lean_name
        || source_slice.source_locator != node.source_locator
        || source_slice.statement_sha256 != node.statement_sha256
        || source_slice.dependency_ids != node.dependencies
        || source_slice
            .dependency_ids
            .iter()
            .any(|id| !valid_kebab_id(id))
        || source_slice
            .dependency_ids
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != source_slice.dependency_ids.len()
    {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "LS source slice does not match the proof receipt",
        ));
    }
    let result: LsResult = read_strict_json_bounded(
        repo_root,
        &attempt_root.join("result.json"),
        manifest,
        line,
        "LS result",
    )?;
    if result.schema_version != "crouzeix-ls-proof-slice-result/v1"
        || result.status != "passed"
        || !valid_bounded_json_string(&result.reason, 1, 4_096)
    {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "LS result does not match the proof receipt",
        ));
    }
    let command: LsCommand = read_strict_json_bounded(
        repo_root,
        &attempt_root.join("build/command.json"),
        manifest,
        line,
        "LS command",
    )?;
    let (expected_build_target, aggregate_path, aggregate_target) = match &command {
        LsCommand::V1(_) => (
            spec.legacy_build_target.unwrap_or(spec.build_target),
            LS_LEGACY_AGGREGATE_PATH,
            LS_LEGACY_BUILD_TARGET,
        ),
        LsCommand::V2(_) => (spec.build_target, LS_AGGREGATE_PATH, LS_BUILD_TARGET),
    };
    if receipt.build_target != expected_build_target {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!(
                "LS committed build_target does not match LS node {}",
                node.node_id
            ),
        ));
    }
    let receipt_module = local_module_name(Path::new(&receipt.build_target)).ok_or_else(|| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "LS committed build_target is invalid",
        )
    })?;
    let aggregate_modules = ls_aggregate_modules(repo_root, aggregate_path, manifest, line)?;
    if !aggregate_modules.contains(&receipt_module) {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("LS committed build_target is not reachable from {aggregate_target}"),
        ));
    }
    verify_digest_bound_file(
        repo_root,
        Path::new(&receipt.build_target),
        &receipt.module_sha256,
        manifest,
        line,
        "LS proof module",
    )?;
    if !valid_ls_command(repo_root, &command, manifest, line)? {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "LS command is not the successful aggregate Crouzeix build for its command schema",
        ));
    }
    let stdout_path = attempt_root.join("build/stdout.log");
    let stdout = read_utf8_bounded_with_limit(
        repo_root,
        &stdout_path,
        manifest,
        line,
        "LS aggregate stdout",
        LS_OUTPUT_MAX_BYTES,
    )?;
    verify_success_stdout(
        &stdout,
        aggregate_target,
        "LS committed aggregate stdout lacks exact success markers",
        manifest,
        line,
    )?;
    let axiom_path = attempt_root.join("build/axioms.txt");
    let audit_axioms = parse_ls_axiom_audit(
        repo_root,
        &axiom_path,
        &receipt.expected_lean_declaration,
        manifest,
        line,
    )?;
    if audit_axioms != receipt.observed_axioms {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "LS axiom audit does not match the receipt's exact ordered observed axioms",
        ));
    }
    Ok(match command {
        LsCommand::V1(_) => None,
        LsCommand::V2(command) => Some(LsAggregateIdentity {
            command_sha256: receipt.command_sha256,
            stdout_sha256: receipt.stdout_sha256,
            stderr_sha256: receipt.stderr_sha256,
            local_source_closure_sha256: command.local_source_closure_sha256,
        }),
    })
}

fn valid_attempt_name(value: &str) -> bool {
    let Some(number) = value.strip_prefix("attempt-") else {
        return false;
    };
    !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())
}

fn valid_bounded_json_string(value: &str, minimum: usize, maximum: usize) -> bool {
    let length = value.chars().count();
    (minimum..=maximum).contains(&length) && !value.contains('\0')
}

fn valid_ordered_axioms(axioms: &[String]) -> bool {
    axioms.iter().all(|axiom| valid_lean_name(axiom))
        && axioms.windows(2).all(|pair| pair[0] < pair[1])
}

fn verify_success_stdout(
    stdout: &str,
    target: &str,
    detail: &str,
    manifest: &Path,
    line: usize,
) -> Result<(), AppError> {
    let target_marker = format!("[lean] target={target}");
    let root_marker = "[lean] root=formalization/lean";
    let passed_marker = "[lean] outcome=passed";
    let lines = stdout.lines().collect::<Vec<_>>();
    let valid = [target_marker.as_str(), root_marker, passed_marker]
        .into_iter()
        .all(|marker| lines.iter().filter(|line| **line == marker).count() == 1)
        && lines
            .iter()
            .filter(|line| line.starts_with("[lean] outcome="))
            .copied()
            .eq([passed_marker]);
    if !valid {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            detail,
        ));
    }
    Ok(())
}

fn valid_ls_command(
    repo_root: &Path,
    command: &LsCommand,
    manifest: &Path,
    line: usize,
) -> Result<bool, AppError> {
    let valid = match command {
        LsCommand::V1(command) => {
            command.argv == ["scripts/check_lean_library.sh", LS_LEGACY_BUILD_TARGET]
                && command.cwd == "."
                && command.exit_code == 0
                && valid_bounded_json_string(&command.cache_policy, 1, 4_096)
                && command.elan_home == LS_ELAN_HOME
        }
        LsCommand::V2(command) => {
            if !valid_digest(&command.wrapper_sha256)
                || !valid_digest(&command.lake_manifest_sha256)
                || !valid_digest(&command.dependency_cache_metadata_sha256)
                || !valid_digest(&command.required_mathlib_artifacts_sha256)
                || !valid_digest(&command.local_source_closure_sha256)
            {
                false
            } else {
                verify_digest_bound_file(
                    repo_root,
                    Path::new("formalization/lean/lake-manifest.json"),
                    &command.lake_manifest_sha256,
                    manifest,
                    line,
                    "LS Lake manifest",
                )?;
                let source_closure = ls_source_closure(repo_root, manifest, line)?;
                let mathlib_digest = required_mathlib_artifacts_digest(
                    repo_root,
                    &source_closure.sources,
                    manifest,
                    line,
                )?;
                verify_ls_v2_provider_independence(repo_root, manifest, line)?;
                command.argv == ["scripts/check_lean_library.sh", LS_BUILD_TARGET]
                    && command.cwd == "."
                    && command.timeout_seconds == 3_600
                    && command.output_cap_bytes == 1_048_576
                    && command.env.elan_home == LS_ELAN_HOME
                    && command.env.elan_toolchain == LS_ELAN_TOOLCHAIN
                    && command.env.path == LS_TOOLCHAIN_PATH
                    && command.exit_code == 0
                    && command.cache_policy == LS_COMMAND_CACHE_POLICY
                    && command.elan_toolchain == LS_ELAN_TOOLCHAIN
                    && command.local_source_closure_sha256 == source_closure.digest
                    && command.required_mathlib_artifacts_sha256 == mathlib_digest
            }
        }
    };
    Ok(valid)
}

fn verify_ls_v2_provider_independence(
    repo_root: &Path,
    manifest: &Path,
    line: usize,
) -> Result<(), AppError> {
    let forbidden_modules = LS_FORBIDDEN_MODULES
        .iter()
        .map(|module| (*module).to_owned())
        .collect::<Vec<_>>();
    let forbidden_prefixes = LS_FORBIDDEN_PREFIXES
        .iter()
        .map(|prefix| (*prefix).to_owned())
        .chain(["Crouzeix.Harp".to_owned()])
        .collect::<Vec<_>>();
    compute_provider_closure(
        repo_root,
        &[LS_BUILD_TARGET.to_owned()],
        &forbidden_modules,
        &forbidden_prefixes,
        manifest,
        line,
    )?;
    Ok(())
}

fn read_utf8_bounded(
    repo_root: &Path,
    path: &Path,
    manifest: &Path,
    line: usize,
    label: &str,
) -> Result<String, AppError> {
    read_utf8_bounded_with_limit(
        repo_root,
        path,
        manifest,
        line,
        label,
        LOCAL_ARTIFACT_MAX_BYTES,
    )
}

fn read_utf8_bounded_with_limit(
    repo_root: &Path,
    path: &Path,
    manifest: &Path,
    line: usize,
    label: &str,
    maximum: u64,
) -> Result<String, AppError> {
    require_regular_file_without_symlinks(repo_root, path, manifest, line, label)?;
    let metadata = fs::metadata(repo_root.join(path)).map_err(|error| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("cannot inspect {label}: {error}"),
        )
    })?;
    if metadata.len() > maximum {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("{label} exceeds the byte limit of {maximum}"),
        ));
    }
    let bytes = fs::read(repo_root.join(path)).map_err(|error| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("cannot read {label}: {error}"),
        )
    })?;
    if bytes.contains(&b'\0') || bytes.contains(&b'\r') {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("{label} must not contain NUL or CR"),
        ));
    }
    String::from_utf8(bytes).map_err(|_| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("{label} must be UTF-8"),
        )
    })
}

fn parse_ls_axiom_audit(
    repo_root: &Path,
    path: &Path,
    declaration: &str,
    manifest: &Path,
    line: usize,
) -> Result<Vec<String>, AppError> {
    let text = read_utf8_bounded(repo_root, path, manifest, line, "LS axiom audit")?;
    let text = text.trim_end_matches(char::is_whitespace);
    let Some((observed_declaration, result)) = text
        .strip_prefix('\'')
        .and_then(|text| text.split_once("' "))
    else {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "LS axiom audit output is malformed",
        ));
    };
    if observed_declaration != declaration {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "LS axiom audit declaration does not match expected declaration",
        ));
    };
    let axioms = if result == "does not depend on any axioms" {
        Vec::new()
    } else {
        let Some(raw) = result
            .strip_prefix("depends on axioms: [")
            .and_then(|value| value.strip_suffix(']'))
        else {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "LS axiom audit output is malformed",
            ));
        };
        let mut parsed = if raw.trim().is_empty() {
            Vec::new()
        } else {
            raw.split(',')
                .map(str::trim)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        if parsed.iter().any(|axiom| !valid_lean_name(axiom))
            || parsed.iter().collect::<BTreeSet<_>>().len() != parsed.len()
        {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "LS axiom audit contains invalid or duplicate axiom names",
            ));
        }
        parsed.sort();
        parsed
    };
    Ok(axioms)
}

fn read_strict_json_bounded<T: DeserializeOwned>(
    repo_root: &Path,
    path: &Path,
    manifest: &Path,
    line: usize,
    label: &str,
) -> Result<T, AppError> {
    let text = read_utf8_bounded(repo_root, path, manifest, line, label)?;
    serde_json::from_str(&text).map_err(|error| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("invalid {label} JSON: {error}"),
        )
    })
}

fn verify_provider_report(
    repo_root: &Path,
    provider_path: &str,
    route_id: &str,
    manifest: &Path,
    line: usize,
) -> Result<(), AppError> {
    let provider_path = local_safe_relative(provider_path, manifest, line)?;
    let report: ProviderReport = read_strict_json_bounded(
        repo_root,
        &provider_path,
        manifest,
        line,
        "provider-independence report",
    )?;
    let (expected_roots, expected_forbidden, expected_forbidden_prefixes) =
        provider_policy(route_id).ok_or_else(|| {
            manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "provider report has an unknown route",
            )
        })?;
    if report.schema_version != "crouzeix-provider-independence/v1"
        || report.route_id != route_id
        || report.status != "passed"
        || report.roots != expected_roots
        || report.forbidden_modules != expected_forbidden
        || report.forbidden_prefixes != expected_forbidden_prefixes
        || report.modules.is_empty()
        || report.modules.len() > MAX_PROVIDER_MODULES
        || !report
            .modules
            .windows(2)
            .all(|pair| pair[0].module_name < pair[1].module_name)
        || !strictly_sorted_unique(&report.set_options)
    {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "provider-independence report header, policy, or ordering is invalid",
        ));
    }
    let recomputed = compute_provider_closure(
        repo_root,
        &report.roots,
        &report.forbidden_modules,
        &report.forbidden_prefixes,
        manifest,
        line,
    )?;
    if report.modules != recomputed.0 || report.set_options != recomputed.1 {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "provider-independence report does not match the current local Lean closure",
        ));
    }
    Ok(())
}

fn provider_policy(route_id: &str) -> Option<(Vec<String>, Vec<String>, Vec<String>)> {
    let legacy = [
        "Crouzeix.Jin.Terminal",
        "CrouzeixConjecture.FinalTheorems",
        "CrouzeixConjecture.HilbertSpace",
        "CrouzeixConjecture.HilbertSpectralSet",
        "CrouzeixConjecture.RadialOuterReduction",
    ];
    let (roots, forbidden, forbidden_prefixes): (&[&str], Vec<String>, &[&str]) = match route_id {
        "jin" => (
            &["CrouzeixJin"],
            ["Crouzeix", "CrouzeixHarp", "CrouzeixLoristSchwenninger"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            &["Crouzeix.LoristSchwenninger", "Crouzeix.Harp"],
        ),
        "lorist-schwenninger" => (
            &["CrouzeixLoristSchwenninger"],
            legacy
                .into_iter()
                .chain([
                    "Crouzeix.Harp.Consequences",
                    "Crouzeix.Harp.FiniteAtomicL2Dilation",
                    "Crouzeix.Harp.FiniteHorizonPerturbation",
                    "Crouzeix.Harp.MainTheorem",
                    "CrouzeixConjecture.CompletionDiagonalization",
                    "CrouzeixConjecture.CompletionStatement",
                    "CrouzeixConjecture.MainPerturbationReduction",
                    "CrouzeixConjecture.PositiveRealCompletion",
                    "CrouzeixJin",
                ])
                .map(str::to_owned)
                .collect(),
            &["Crouzeix.Harp", "Crouzeix.Jin"],
        ),
        "harp" => (
            &[
                "Crouzeix.Harp.Consequences",
                "Crouzeix.Harp.FiniteAtomicL2Dilation",
                "Crouzeix.Harp.FiniteHorizonPerturbation",
                "Crouzeix.Harp.MainTheorem",
            ],
            legacy
                .into_iter()
                .chain(["CrouzeixLoristSchwenninger"])
                .map(str::to_owned)
                .collect(),
            &["Crouzeix.Jin"],
        ),
        _ => return None,
    };
    let mut forbidden = forbidden;
    forbidden.sort();
    Some((
        roots.iter().map(|root| (*root).to_owned()).collect(),
        forbidden,
        forbidden_prefixes
            .iter()
            .map(|prefix| (*prefix).to_owned())
            .collect(),
    ))
}

fn strictly_sorted_unique<T: Ord>(values: &[T]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn compute_provider_closure(
    repo_root: &Path,
    roots: &[String],
    forbidden_modules: &[String],
    forbidden_prefixes: &[String],
    manifest: &Path,
    line: usize,
) -> Result<(Vec<ProviderModule>, Vec<ProviderFinding>), AppError> {
    let lean_root = Path::new("formalization/lean");
    let mut pending = roots.iter().rev().cloned().collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    let mut modules = Vec::new();
    let mut set_options = Vec::new();
    while let Some(module_name) = pending.pop() {
        if visited.contains(&module_name) {
            continue;
        }
        if visited.len() >= MAX_PROVIDER_MODULES {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "provider-independence closure exceeds the module cap",
            ));
        }
        if provider_module_forbidden(&module_name, forbidden_modules, forbidden_prefixes) {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                &format!("provider-independence closure contains forbidden module {module_name}"),
            ));
        }
        let module_path = Path::new(&module_name.replace('.', "/")).with_extension("lean");
        let relative = lean_root.join(&module_path);
        match fs::symlink_metadata(repo_root.join(&relative)) {
            Ok(metadata) if !metadata.file_type().is_symlink() && metadata.is_file() => {}
            Ok(_) => {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!("provider module is not a regular file: {module_name}"),
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if roots.contains(&module_name) {
                    return Err(manifest_error(
                        LOCAL_FORMALIZATION_CODE,
                        manifest,
                        line,
                        &format!("provider root module is missing: {module_name}"),
                    ));
                }
                if managed_local_module(&module_name) {
                    return Err(manifest_error(
                        LOCAL_FORMALIZATION_CODE,
                        manifest,
                        line,
                        &format!("missing managed local import: {module_name}"),
                    ));
                }
                continue;
            }
            Err(error) => {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!("cannot inspect provider module {module_name}: {error}"),
                ));
            }
        }
        require_regular_file_without_symlinks(
            repo_root,
            &relative,
            manifest,
            line,
            "provider module",
        )?;
        let source = read_utf8_bounded(repo_root, &relative, manifest, line, "provider module")?;
        let active = mask_lean_source(&source, &module_name, manifest, line)?;
        let imports = lean_header_imports(&active, &module_name, manifest, line)?;
        for imported in imports.iter().rev() {
            if provider_module_forbidden(imported, forbidden_modules, forbidden_prefixes) {
                return Err(manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!("forbidden provider module {imported} imported by {module_name}"),
                ));
            }
            pending.push(imported.clone());
        }
        scan_provider_source(
            &source,
            &active,
            &module_name,
            manifest,
            line,
            &mut set_options,
        )?;
        modules.push(ProviderModule {
            module_name: module_name.clone(),
            module_path: slash_path(&relative),
            module_sha256: sha256_file(&repo_root.join(&relative)).map_err(|error| {
                manifest_error(
                    LOCAL_FORMALIZATION_CODE,
                    manifest,
                    line,
                    &format!("cannot hash provider module: {}", error.message),
                )
            })?,
        });
        visited.insert(module_name);
    }
    modules.sort_by(|left, right| left.module_name.cmp(&right.module_name));
    set_options.sort();
    Ok((modules, set_options))
}

fn provider_module_forbidden(
    module: &str,
    forbidden_modules: &[String],
    forbidden_prefixes: &[String],
) -> bool {
    forbidden_modules
        .iter()
        .any(|forbidden| forbidden == module)
        || forbidden_prefixes.iter().any(|prefix| {
            module == prefix
                || module
                    .strip_prefix(prefix)
                    .is_some_and(|suffix| suffix.starts_with('.'))
        })
}

fn managed_local_module(module: &str) -> bool {
    ["Crouzeix", "CrouzeixConjecture"]
        .into_iter()
        .any(|prefix| {
            module == prefix
                || module
                    .strip_prefix(prefix)
                    .is_some_and(|suffix| suffix.starts_with('.'))
        })
}

pub(super) fn lean_header_imports(
    source: &str,
    module: &str,
    manifest: &Path,
    line: usize,
) -> Result<Vec<String>, AppError> {
    let mut imports = Vec::new();
    let mut saw_module = false;
    let mut saw_prelude = false;
    for (source_line, raw) in source.lines().enumerate() {
        let command = raw.trim();
        if command.is_empty() {
            continue;
        }
        if command == "module" {
            if saw_module || saw_prelude || !imports.is_empty() {
                return provider_error(
                    manifest,
                    line,
                    module,
                    source_line + 1,
                    "malformed module command",
                );
            }
            saw_module = true;
            continue;
        }
        if starts_lean_keyword(command, "module") {
            return provider_error(
                manifest,
                line,
                module,
                source_line + 1,
                "malformed module command",
            );
        }
        if command == "prelude" {
            if saw_prelude || !imports.is_empty() {
                return provider_error(
                    manifest,
                    line,
                    module,
                    source_line + 1,
                    "malformed prelude command",
                );
            }
            saw_prelude = true;
            continue;
        }
        if starts_lean_keyword(command, "prelude") {
            return provider_error(
                manifest,
                line,
                module,
                source_line + 1,
                "malformed prelude command",
            );
        }
        let parts = command.split_whitespace().collect::<Vec<_>>();
        if let Some(imported) = parse_lean_import_command(&parts, saw_module) {
            match imported {
                Ok(imported) => imports.push(imported.to_owned()),
                Err(detail) => {
                    return provider_error(manifest, line, module, source_line + 1, detail);
                }
            }
            continue;
        }
        if valid_lean_section_command(&parts) || valid_lean_modified_body_command(&parts) {
            break;
        }
        if ["import", "meta", "public", "all"]
            .into_iter()
            .any(|keyword| starts_lean_keyword(command, keyword))
        {
            return provider_error(
                manifest,
                line,
                module,
                source_line + 1,
                "malformed import-like command",
            );
        }
        break;
    }
    Ok(imports)
}

fn parse_lean_import_command<'a>(
    parts: &'a [&'a str],
    saw_module: bool,
) -> Option<Result<&'a str, &'static str>> {
    let mut index = 0;
    let mut modified = false;
    if parts.get(index) == Some(&"public") {
        modified = true;
        index += 1;
    }
    if parts.get(index) == Some(&"meta") {
        modified = true;
        index += 1;
    }
    if parts.get(index) != Some(&"import") {
        return None;
    }
    index += 1;
    if parts.get(index) == Some(&"all") {
        modified = true;
        index += 1;
    }
    if modified && !saw_module {
        return Some(Err("modified import requires module"));
    }
    let Some(imported) = parts.get(index) else {
        return Some(Err("malformed import command"));
    };
    if index + 1 != parts.len() || !valid_lean_name(imported) {
        return Some(Err("malformed import command"));
    }
    Some(Ok(imported))
}

fn valid_lean_section_command(parts: &[&str]) -> bool {
    let mut index = 0;
    if parts.get(index) == Some(&"public") {
        index += 1;
    }
    if parts.get(index) == Some(&"meta") {
        index += 1;
    }
    if parts.get(index) != Some(&"section") {
        return false;
    }
    index += 1;
    index == parts.len()
        || index + 1 == parts.len() && parts.get(index).is_some_and(|name| valid_lean_name(name))
}

fn valid_lean_modified_body_command(parts: &[&str]) -> bool {
    const MODIFIED_BODY_COMMANDS: [&str; 17] = [
        "abbrev",
        "axiom",
        "class",
        "def",
        "elab",
        "example",
        "inductive",
        "instance",
        "lemma",
        "macro",
        "notation",
        "opaque",
        "prefix",
        "postfix",
        "scoped",
        "structure",
        "syntax",
    ];
    const THEOREM: &str = "theorem";

    let body_command = |part: Option<&&str>| {
        part.is_some_and(|part| MODIFIED_BODY_COMMANDS.contains(part) || *part == THEOREM)
    };
    if parts.first() == Some(&"meta") {
        return body_command(parts.get(1));
    }
    if parts.first() != Some(&"public") {
        return false;
    }
    if body_command(parts.get(1)) {
        return true;
    }
    if parts.get(1) == Some(&"noncomputable") {
        return matches!(parts, ["public", "noncomputable", "section"])
            || matches!(parts, ["public", "noncomputable", "section", name] if valid_lean_name(name));
    }
    parts.get(1) == Some(&"meta") && body_command(parts.get(2))
}

fn starts_lean_keyword(command: &str, keyword: &str) -> bool {
    command == keyword
        || command
            .strip_prefix(keyword)
            .and_then(|suffix| suffix.chars().next())
            .is_some_and(char::is_whitespace)
}

fn scan_provider_source(
    original: &str,
    active: &str,
    module: &str,
    manifest: &Path,
    line: usize,
    set_options: &mut Vec<ProviderFinding>,
) -> Result<(), AppError> {
    let original_lines = original.lines().collect::<Vec<_>>();
    for (source_line, text) in active.lines().enumerate() {
        let characters = text.chars().collect::<Vec<_>>();
        let mut cursor = 0;
        while cursor < characters.len() {
            let character = characters[cursor];
            if !(character.is_ascii_alphabetic() || character == '_')
                || cursor > 0 && lean_token_neighbor(characters[cursor - 1])
            {
                cursor += 1;
                continue;
            }
            let start = cursor;
            cursor += 1;
            while cursor < characters.len()
                && (characters[cursor].is_ascii_alphanumeric()
                    || matches!(characters[cursor], '_' | '\''))
            {
                cursor += 1;
            }
            if characters
                .get(cursor)
                .is_some_and(|next| lean_token_neighbor(*next) || matches!(next, '!' | '?'))
            {
                continue;
            }
            let token = characters[start..cursor].iter().collect::<String>();
            if matches!(
                token.as_str(),
                "sorry" | "admit" | "unsafe" | "native_decide" | "implemented_by"
            ) {
                return provider_error(
                    manifest,
                    line,
                    module,
                    source_line + 1,
                    &format!("danger token {token}"),
                );
            }
            if token == "set_option" {
                set_options.push(ProviderFinding {
                    module_name: module.to_owned(),
                    line: source_line + 1,
                    token,
                    source_line: original_lines
                        .get(source_line)
                        .map_or("", |value| value.trim())
                        .to_owned(),
                });
            }
        }
        let leading = text.trim_start_matches([' ', '\t']);
        if leading.strip_prefix("axiom").is_some_and(|suffix| {
            suffix
                .chars()
                .next()
                .is_none_or(|next| !lean_token_neighbor(next) && !matches!(next, '!' | '?'))
        }) {
            return provider_error(
                manifest,
                line,
                module,
                source_line + 1,
                "danger token axiom",
            );
        }
    }
    Ok(())
}

fn lean_token_neighbor(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '\'')
}

fn provider_error<T>(
    manifest: &Path,
    line: usize,
    module: &str,
    source_line: usize,
    detail: &str,
) -> Result<T, AppError> {
    Err(manifest_error(
        LOCAL_FORMALIZATION_CODE,
        manifest,
        line,
        &format!("{detail} in {module}:{source_line}"),
    ))
}

pub(super) fn mask_lean_source(
    source: &str,
    module: &str,
    manifest: &Path,
    line: usize,
) -> Result<String, AppError> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum State {
        Code,
        LineComment,
        BlockComment,
        String,
        Character,
        RawString,
        InterpolatedText,
        EscapedIdentifier,
    }

    fn token_start(chars: &[char], index: usize) -> bool {
        index == 0
            || !chars[index - 1].is_alphanumeric()
                && !matches!(chars[index - 1], '_' | '\'' | '!' | '?' | '»')
    }

    fn raw_string_start(chars: &[char], index: usize) -> Option<(usize, Vec<char>)> {
        if chars[index] != 'r' || !token_start(chars, index) {
            return None;
        }
        let mut cursor = index + 1;
        while chars.get(cursor) == Some(&'#') {
            cursor += 1;
        }
        if chars.get(cursor) != Some(&'"') {
            return None;
        }
        let hashes = cursor - index - 1;
        let mut closer = Vec::with_capacity(hashes + 1);
        closer.push('"');
        closer.extend(std::iter::repeat_n('#', hashes));
        Some((cursor - index + 1, closer))
    }

    fn starts_character_literal(chars: &[char], index: usize) -> bool {
        if index > 0 && lean_token_neighbor(chars[index - 1]) {
            return false;
        }
        index + 2 >= chars.len()
            || chars.get(index + 1) == Some(&'\\')
            || chars.get(index + 2) == Some(&'\'')
    }

    let chars = source.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(source.len());
    let mut index = 0;
    let mut state = State::Code;
    let mut block_depth = 0usize;
    let mut raw_closer = Vec::new();
    let mut interpolation_depths = Vec::new();
    while index < chars.len() {
        let character = chars[index];
        let next = chars.get(index + 1).copied();
        let masked = if matches!(character, '\n' | '\r') {
            character
        } else {
            ' '
        };
        match state {
            State::LineComment => {
                output.push(masked);
                index += 1;
                if matches!(character, '\n' | '\r') {
                    state = State::Code;
                }
            }
            State::BlockComment => {
                if character == '/' && next == Some('-') {
                    block_depth += 1;
                    output.push_str("  ");
                    index += 2;
                } else if character == '-' && next == Some('/') {
                    block_depth -= 1;
                    output.push_str("  ");
                    index += 2;
                    if block_depth == 0 {
                        state = State::Code;
                    }
                } else {
                    output.push(masked);
                    index += 1;
                }
            }
            State::String | State::Character => {
                output.push(masked);
                index += 1;
                if character == '\\' && index < chars.len() {
                    output.push(if matches!(chars[index], '\n' | '\r') {
                        chars[index]
                    } else {
                        ' '
                    });
                    index += 1;
                } else if (state == State::String && character == '"')
                    || (state == State::Character && character == '\'')
                {
                    state = State::Code;
                }
            }
            State::RawString => {
                if chars[index..].starts_with(&raw_closer) {
                    output.extend(std::iter::repeat_n(' ', raw_closer.len()));
                    index += raw_closer.len();
                    state = State::Code;
                } else {
                    output.push(masked);
                    index += 1;
                }
            }
            State::InterpolatedText => {
                if matches!((character, next), ('{', Some('{')) | ('}', Some('}'))) {
                    output.push_str("  ");
                    index += 2;
                } else {
                    output.push(masked);
                    index += 1;
                    if character == '\\' && index < chars.len() {
                        output.push(if matches!(chars[index], '\n' | '\r') {
                            chars[index]
                        } else {
                            ' '
                        });
                        index += 1;
                    } else if character == '"' {
                        state = State::Code;
                    } else if character == '{' {
                        interpolation_depths.push(1);
                        state = State::Code;
                    }
                }
            }
            State::EscapedIdentifier => {
                output.push(masked);
                index += 1;
                if character == '»' {
                    state = State::Code;
                }
            }
            State::Code => {
                if !interpolation_depths.is_empty() && character == '{' {
                    *interpolation_depths.last_mut().expect("nonempty checked") += 1;
                    output.push(character);
                    index += 1;
                } else if !interpolation_depths.is_empty() && character == '}' {
                    let depth = interpolation_depths.last_mut().expect("nonempty checked");
                    *depth -= 1;
                    output.push(character);
                    index += 1;
                    if *depth == 0 {
                        interpolation_depths.pop();
                        state = State::InterpolatedText;
                    }
                } else if character == 's'
                    && next == Some('!')
                    && chars.get(index + 2) == Some(&'"')
                    && token_start(&chars, index)
                {
                    output.push_str("   ");
                    index += 3;
                    state = State::InterpolatedText;
                } else if character == '«' {
                    output.push(' ');
                    index += 1;
                    state = State::EscapedIdentifier;
                } else if character == '-' && next == Some('-') {
                    output.push_str("  ");
                    index += 2;
                    state = State::LineComment;
                } else if character == '/' && next == Some('-') {
                    output.push_str("  ");
                    index += 2;
                    block_depth = 1;
                    state = State::BlockComment;
                } else if character == '-' && next == Some('/') {
                    return provider_error(
                        manifest,
                        line,
                        module,
                        source[..source
                            .char_indices()
                            .nth(index)
                            .map_or(source.len(), |p| p.0)]
                            .lines()
                            .count(),
                        "unmatched block comment close",
                    );
                } else if let Some((length, closer)) = raw_string_start(&chars, index) {
                    output.extend(std::iter::repeat_n(' ', length));
                    index += length;
                    raw_closer = closer;
                    state = State::RawString;
                } else if character == '"' {
                    output.push(' ');
                    index += 1;
                    state = State::String;
                } else if character == '\'' && starts_character_literal(&chars, index) {
                    output.push(' ');
                    index += 1;
                    state = State::Character;
                } else {
                    output.push(character);
                    index += 1;
                }
            }
        }
    }
    if state == State::BlockComment {
        return provider_error(
            manifest,
            line,
            module,
            source.lines().count(),
            "unterminated block comment",
        );
    }
    let unterminated = match state {
        State::String => Some("unterminated string literal"),
        State::Character => Some("unterminated character literal"),
        State::RawString => Some("unterminated raw string literal"),
        State::InterpolatedText if interpolation_depths.is_empty() => {
            Some("unterminated interpolated string literal")
        }
        State::EscapedIdentifier => Some("unterminated escaped identifier"),
        _ if !interpolation_depths.is_empty() => Some("unterminated interpolated string literal"),
        _ => None,
    };
    if let Some(detail) = unterminated {
        return provider_error(manifest, line, module, source.lines().count(), detail);
    }
    Ok(output)
}

fn verify_route_binding(
    route_id: &str,
    declaration: &str,
    module_path: &Path,
    manifest: &Path,
    line: usize,
) -> Result<(), AppError> {
    if !FORMALIZATION_SPECS.iter().any(|spec| {
        spec.route == route_id
            && spec.declaration == declaration
            && Path::new(spec.module_path) == module_path
    }) {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "module_path and declaration_name must be an exact approved route pair",
        ));
    }
    Ok(())
}

fn parse_axiom_field(value: &str, manifest: &Path, line: usize) -> Result<Vec<String>, AppError> {
    if value == "-" {
        return Ok(Vec::new());
    }
    let axioms = value.split(',').map(str::to_owned).collect::<Vec<_>>();
    if axioms.iter().any(|axiom| !valid_lean_name(axiom)) {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "observed_axioms must be - or a comma-separated list of Lean names",
        ));
    }
    let unique = axioms.iter().collect::<BTreeSet<_>>();
    if unique.len() != axioms.len() || !axioms.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "observed_axioms must be unique and lexicographically sorted",
        ));
    }
    Ok(axioms)
}

fn verify_axiom_audit(
    repo_root: &Path,
    path: &Path,
    declaration: &str,
    manifest_axioms: &[String],
    manifest: &Path,
    line: usize,
) -> Result<(), AppError> {
    let bytes = fs::read(repo_root.join(path)).map_err(|error| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            &format!("cannot read axiom audit: {error}"),
        )
    })?;
    if bytes.is_empty()
        || !bytes.ends_with(b"\n")
        || bytes.contains(&b'\r')
        || bytes.contains(&b'\0')
    {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "axiom audit must be nonempty LF-terminated UTF-8 without NUL or CR",
        ));
    }
    let text = String::from_utf8(bytes).map_err(|_| {
        manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "axiom audit must be UTF-8",
        )
    })?;
    let statement = text.strip_suffix('\n').expect("LF termination checked");
    let prefix = format!("'{declaration}' ");
    let Some(result) = statement.strip_prefix(&prefix) else {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "axiom audit declaration does not match declaration_name",
        ));
    };
    let audit_axioms = if result == "does not depend on any axioms" {
        Vec::new()
    } else {
        let Some(raw) = result
            .strip_prefix("depends on axioms: [")
            .and_then(|value| value.strip_suffix(']'))
        else {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "axiom audit output is malformed",
            ));
        };
        let mut parsed = raw
            .split(',')
            .map(str::trim)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if parsed.iter().any(|axiom| !valid_lean_name(axiom))
            || parsed.iter().collect::<BTreeSet<_>>().len() != parsed.len()
        {
            return Err(manifest_error(
                LOCAL_FORMALIZATION_CODE,
                manifest,
                line,
                "axiom audit contains invalid or duplicate axiom names",
            ));
        }
        parsed.sort();
        parsed
    };
    if audit_axioms != manifest_axioms {
        return Err(manifest_error(
            LOCAL_FORMALIZATION_CODE,
            manifest,
            line,
            "axiom audit does not match observed_axioms",
        ));
    }
    Ok(())
}

fn valid_kebab_id(value: &str) -> bool {
    let mut parts = value.split('-');
    let Some(first) = parts.next() else {
        return false;
    };
    !first.is_empty()
        && std::iter::once(first).chain(parts).all(|part| {
            !part.is_empty()
                && part.bytes().enumerate().all(|(index, byte)| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() && index > 0
                })
        })
}

fn valid_lean_name(value: &str) -> bool {
    value.split('.').all(|part| {
        !part.is_empty()
            && part
                .as_bytes()
                .first()
                .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'\''))
    })
}

fn verify_source_manifest(repo_root: &Path) -> Result<usize, AppError> {
    let relative = Path::new(ROOT).join("source_manifest.tsv");
    let rows = read_manifest(
        repo_root,
        &relative,
        SOURCE_HEADER,
        "sources.crouzeix.source_manifest",
    )?;
    let mut receipt_ids = BTreeSet::new();
    let mut artifacts = BTreeSet::new();
    for (index, columns) in rows.iter().enumerate().skip(1) {
        let line = index + 1;
        if columns.len() != 14 {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "expected fourteen columns",
            ));
        }
        let [schema, receipt_id, source_id, source_class, role, identity, source_url, upstream_path, bytes, digest, local_path, observed, license, redistribution] =
            columns.as_slice()
        else {
            unreachable!("column count checked");
        };
        require_canonical_fields("sources.crouzeix.source_manifest", &relative, line, columns)?;
        if schema != "crouzeix-source-receipt/v1" {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "unsupported schema version",
            ));
        }
        if !valid_receipt_id(receipt_id) || !receipt_ids.insert(receipt_id) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "receipt_id must be unique canonical uppercase text",
            ));
        }
        if !valid_receipt_id(source_id) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "source_id must be canonical uppercase text",
            ));
        }
        if !matches!(
            source_class.as_str(),
            "git-artifact" | "arxiv-artifact" | "journal-record" | "dated-page"
        ) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "invalid source_class",
            ));
        }
        if !matches!(
            role.as_str(),
            "manuscript"
                | "source"
                | "pdf"
                | "metadata"
                | "formalization"
                | "prompt"
                | "history"
                | "prerequisite"
                | "license"
        ) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "invalid role",
            ));
        }
        validate_identity_url(identity, source_url).map_err(|detail| {
            manifest_error("sources.crouzeix.source_manifest", &relative, line, &detail)
        })?;
        if upstream_path != "-" {
            safe_relative(upstream_path).map_err(|detail| {
                manifest_error("sources.crouzeix.source_manifest", &relative, line, &detail)
            })?;
        }
        if local_path != "-" {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "upstream artifacts must remain remote-only",
            ));
        }
        if bytes == "-" || digest == "-" {
            if bytes != "-"
                || digest != "-"
                || source_class != "dated-page"
                || redistribution != "metadata-only"
            {
                return Err(manifest_error(
                    "sources.crouzeix.source_manifest",
                    &relative,
                    line,
                    "missing bytes and digest are allowed only for metadata-only dated pages",
                ));
            }
        } else if bytes.parse::<u64>().is_err() || !valid_digest(digest) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "invalid byte count or sha256",
            ));
        }
        if !valid_date(observed) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "observed must be an ISO date",
            ));
        }
        if !matches!(
            license.as_str(),
            "not-present-at-revision"
                | "arxiv-nonexclusive"
                | "publisher-record"
                | "unknown"
                | "not-redistributable"
        ) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "invalid license_status",
            ));
        }
        if !matches!(
            redistribution.as_str(),
            "remote-only" | "quotation-only" | "metadata-only"
        ) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "invalid redistribution_status",
            ));
        }
        if !artifacts.insert((source_id, identity, upstream_path)) {
            return Err(manifest_error(
                "sources.crouzeix.source_manifest",
                &relative,
                line,
                "duplicate source identity and upstream path",
            ));
        }
    }
    if rows.len() == 1 {
        return Err(manifest_error(
            "sources.crouzeix.source_manifest",
            &relative,
            1,
            "manifest has no receipts",
        ));
    }
    Ok(rows.len() - 1)
}

fn verify_verification_manifest(repo_root: &Path) -> Result<usize, AppError> {
    let relative = Path::new(ROOT).join("verification_manifest.tsv");
    let rows = read_manifest(
        repo_root,
        &relative,
        VERIFICATION_HEADER,
        "sources.crouzeix.verification_manifest",
    )?;
    let script_digest = sha256_file(&repo_root.join(ROOT).join("acquire.sh"))?;
    let mut receipt_ids = BTreeSet::new();
    for (index, columns) in rows.iter().enumerate().skip(1) {
        let line = index + 1;
        if columns.len() != 17 {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "expected seventeen columns",
            ));
        }
        let [schema, receipt_id, source_id, source_commit, source_tree, operation, command_digest, acquisition_digest, normalization, toolchain, mathlib_revision, observed_at, exit_code, result, log_path, log_bytes, log_digest] =
            columns.as_slice()
        else {
            unreachable!("column count checked");
        };
        require_canonical_fields(
            "sources.crouzeix.verification_manifest",
            &relative,
            line,
            columns,
        )?;
        if schema != "crouzeix-verification-receipt/v1" {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "unsupported schema version",
            ));
        }
        if !valid_receipt_id(receipt_id) || !receipt_ids.insert(receipt_id) {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "receipt_id must be unique canonical uppercase text",
            ));
        }
        if !valid_receipt_id(source_id)
            || !valid_hex(source_commit, 40)
            || !valid_hex(source_tree, 40)
        {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "invalid source identity",
            ));
        }
        if !matches!(operation.as_str(), "lean-build" | "source-scan")
            || !valid_digest(command_digest)
            || acquisition_digest != &script_digest
            || normalization != "crouzeix-log-normalization/v1"
            || toolchain.is_empty()
            || !valid_hex(mathlib_revision, 40)
            || !valid_timestamp(observed_at)
        {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "invalid operation, digest, toolchain, revision, or timestamp",
            ));
        }
        if !matches!(result.as_str(), "passed" | "failed" | "blocked")
            || (result == "blocked" && exit_code != "-")
            || (result != "blocked" && exit_code.parse::<i32>().is_err())
        {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "invalid result and exit-code combination",
            ));
        }
        let log_path = safe_relative(log_path).map_err(|detail| {
            manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                &detail,
            )
        })?;
        let expected_root = Path::new(ROOT).join("verification");
        if !log_path.starts_with("verification/") {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "log path must stay beneath verification/",
            ));
        }
        let expected_bytes = log_bytes.parse::<u64>().map_err(|_| {
            manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "invalid log byte count",
            )
        })?;
        if !valid_digest(log_digest) {
            return Err(manifest_error(
                "sources.crouzeix.verification_manifest",
                &relative,
                line,
                "invalid log sha256",
            ));
        }
        verify_file(
            repo_root,
            &expected_root.join(
                log_path
                    .strip_prefix("verification")
                    .expect("prefix checked"),
            ),
            expected_bytes,
            log_digest,
            "Crouzeix verification log",
        )?;
    }
    if rows.len() == 1 {
        return Err(manifest_error(
            "sources.crouzeix.verification_manifest",
            &relative,
            1,
            "manifest has no receipts",
        ));
    }
    Ok(rows.len() - 1)
}

fn read_manifest(
    repo_root: &Path,
    relative: &Path,
    expected_header: &str,
    code: &'static str,
) -> Result<Vec<Vec<String>>, AppError> {
    let bytes = fs::read(repo_root.join(relative))
        .map_err(|error| AppError::io(code, &format!("read {}", relative.display()), error))?;
    if bytes.is_empty()
        || !bytes.ends_with(b"\n")
        || bytes.contains(&b'\0')
        || bytes.contains(&b'\r')
    {
        return Err(manifest_error(
            code,
            relative,
            1,
            "manifest must be nonempty LF-terminated UTF-8 without NUL or CR",
        ));
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| manifest_error(code, relative, 1, "manifest must be UTF-8"))?;
    let rows = text
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    if rows.first().map(|row| row.join("\t")).as_deref() != Some(expected_header) {
        return Err(manifest_error(code, relative, 1, "unexpected header"));
    }
    Ok(rows)
}

fn require_canonical_fields(
    code: &'static str,
    manifest: &Path,
    line: usize,
    fields: &[String],
) -> Result<(), AppError> {
    if fields
        .iter()
        .any(|field| field.is_empty() || field.trim() != field)
    {
        return Err(manifest_error(
            code,
            manifest,
            line,
            "fields must be nonempty canonical trimmed strings",
        ));
    }
    Ok(())
}

fn validate_identity_url(identity: &str, source_url: &str) -> Result<(), String> {
    if !source_url.starts_with("https://") {
        return Err("source_url must use HTTPS".to_owned());
    }
    if let Some(revision) = identity.strip_prefix("git:") {
        if !valid_hex(revision, 40) || !source_url.contains(revision) {
            return Err("Git URL must contain its declared 40-hex revision".to_owned());
        }
    } else if let Some(version) = identity.strip_prefix("arxiv:") {
        let Some((id, suffix)) = version.rsplit_once('v') else {
            return Err("arXiv identity must end with vN".to_owned());
        };
        if id.is_empty()
            || suffix.is_empty()
            || !suffix.bytes().all(|byte| byte.is_ascii_digit())
            || !source_url.contains(version)
        {
            return Err("arXiv URL must contain its exact version".to_owned());
        }
    } else if let Some(doi) = identity.strip_prefix("doi:") {
        if doi.is_empty() || !source_url.contains(doi) {
            return Err("DOI URL must contain its declared identity".to_owned());
        }
    } else if !identity.strip_prefix("sha256:").is_some_and(valid_digest) {
        return Err("unsupported immutable identity".to_owned());
    }
    Ok(())
}

fn safe_relative(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if path.is_absolute()
        || path.as_os_str().is_empty()
        || !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "path must be repository-relative normal text: {value}"
        ));
    }
    Ok(path.to_path_buf())
}

fn valid_receipt_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn valid_hex(value: &str, len: usize) -> bool {
    value.len() == len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

fn valid_timestamp(value: &str) -> bool {
    value.len() >= 20
        && value.ends_with('Z')
        && value.as_bytes().get(10) == Some(&b'T')
        && valid_date(&value[..10])
}

fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn fixture_route_manifest_path_runtime(route_id: &str) -> String {
    match route_id {
        "jin" => "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json",
        "harp" => "labs/crouzeix_proof_reproduction/formal_targets/harp/route-manifest.json",
        "lorist-schwenninger" => {
            "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/route-manifest.json"
        }
        _ => return String::new(),
    }
    .to_owned()
}

fn invalid(code: &'static str, detail: String) -> AppError {
    AppError::invalid_input(code, detail)
}

fn manifest_error(code: &'static str, manifest: &Path, line: usize, detail: &str) -> AppError {
    invalid(
        code,
        format!("{} line {line}: {detail}", manifest.display()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use sha2::Sha256;
    use tempfile::TempDir;

    type JsonMutation = (&'static str, fn(&mut Value));

    const GENERATION_ID: &str = "0123456789abcdef0123456789abcdef";

    fn selection_path(root: &Path) -> PathBuf {
        root.join(ROOT).join("local_formalization.current.json")
    }

    fn fixture_bundle_digest(bundle: &Path) -> String {
        let mut records = BTreeMap::new();
        for entry in WalkDir::new(bundle) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                records.insert(
                    slash_path(entry.path().strip_prefix(bundle).unwrap()),
                    sha256_file(entry.path()),
                );
            }
        }
        sha256_bytes(
            records
                .into_iter()
                .map(|(path, digest)| format!("{path}\t{digest}\n"))
                .collect::<String>()
                .as_bytes(),
        )
    }

    fn generation_fixture() -> TempDir {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let legacy = local_formalization_bundle(repo.path());
        let generation = repo
            .path()
            .join(ROOT)
            .join("local_formalization_generations")
            .join(GENERATION_ID);
        for entry in WalkDir::new(&legacy) {
            let entry = entry.unwrap();
            let target = generation.join(entry.path().strip_prefix(&legacy).unwrap());
            if entry.file_type().is_dir() {
                fs::create_dir_all(target).unwrap();
            } else {
                let text = fs::read_to_string(entry.path()).unwrap().replace(
                    &format!("{ROOT}/local_formalization/"),
                    &format!("{ROOT}/local_formalization_generations/{GENERATION_ID}/"),
                );
                fs::write(target, text).unwrap();
            }
        }
        let manifest = generation.join("manifest.tsv");
        let text = fs::read_to_string(&manifest).unwrap();
        let mut rows = text
            .lines()
            .map(|row| row.split('\t').map(str::to_owned).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let header = rows[0].clone();
        for row in rows.iter_mut().skip(1) {
            for (index, field) in header.iter().enumerate() {
                if field.ends_with("_path")
                    && header
                        .get(index + 1)
                        .is_some_and(|next| next.ends_with("_sha256"))
                {
                    row[index + 1] = sha256_file(&repo.path().join(&row[index]));
                }
            }
        }
        fs::write(
            manifest,
            rows.into_iter()
                .map(|row| row.join("\t") + "\n")
                .collect::<String>(),
        )
        .unwrap();
        write_json(
            &selection_path(repo.path()),
            &json!({
                "schema_version": "crouzeix-local-formalization-selection/v1",
                "active_generation": GENERATION_ID,
                "generations": [
                    {"id": "legacy", "bundle_sha256": fixture_bundle_digest(&legacy)},
                    {"id": GENERATION_ID, "bundle_sha256": fixture_bundle_digest(&generation)}
                ]
            }),
        );
        repo
    }

    #[test]
    fn generation_selection_preserves_legacy_after_failed_first_refresh_cleanup() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let legacy = local_formalization_bundle(repo.path());
        let before = fixture_bundle_digest(&legacy);
        let expected = verify_local_formalization_bundle_if_present(repo.path()).unwrap();
        let generations = repo
            .path()
            .join(ROOT)
            .join("local_formalization_generations");

        // A first refresh creates this container before running its executor.
        fs::create_dir(&generations).unwrap();
        assert!(!selection_path(repo.path()).exists());
        assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());

        // Simulate its precommit cleanup: remove only the owned, empty container.
        fs::remove_dir(&generations).unwrap();
        assert!(!selection_path(repo.path()).exists());
        assert_eq!(fixture_bundle_digest(&legacy), before);
        assert_eq!(
            verify_local_formalization_bundle_if_present(repo.path()).unwrap(),
            expected
        );
    }

    #[test]
    fn generation_selection_verifies_active_bundle_without_revalidating_historical_inputs() {
        let repo = generation_fixture();
        let legacy_manifest = local_formalization_manifest(repo.path());
        fs::write(
            &legacy_manifest,
            "historical manifest from another input state\n",
        )
        .unwrap();
        let pointer = selection_path(repo.path());
        let mut selection: Value = serde_json::from_slice(&fs::read(&pointer).unwrap()).unwrap();
        selection["generations"][0]["bundle_sha256"] = json!(fixture_bundle_digest(
            &local_formalization_bundle(repo.path())
        ));
        write_json(&pointer, &selection);
        assert!(verify_local_formalization_manifest(
            repo.path(),
            Path::new(ROOT).join(LOCAL_FORMALIZATION_MANIFEST).as_path()
        )
        .is_err());
        assert!(verify_local_formalization_bundle_if_present(repo.path()).is_ok());
    }

    #[test]
    fn generation_selection_rejects_invalid_selectors_and_unknown_directories() {
        for mutate in [
            (|value: &mut Value| value["active_generation"] = json!("../escape")) as fn(&mut Value),
            |value| value["generations"][1]["bundle_sha256"] = json!("0".repeat(64)),
            |value| value["generations"][0]["id"] = json!(GENERATION_ID),
            |value| value["unexpected"] = json!(true),
        ] {
            let repo = generation_fixture();
            let pointer = selection_path(repo.path());
            let mut value: Value = serde_json::from_slice(&fs::read(&pointer).unwrap()).unwrap();
            mutate(&mut value);
            write_json(&pointer, &value);
            assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());
        }
        let repo = generation_fixture();
        fs::create_dir(
            repo.path()
                .join(ROOT)
                .join("local_formalization_generations/unregistered"),
        )
        .unwrap();
        assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());
    }

    #[test]
    fn generation_selection_rejects_duplicate_json_keys_and_link_aliases() {
        let repo = generation_fixture();
        let pointer = selection_path(repo.path());
        let source = fs::read_to_string(&pointer).unwrap();
        fs::write(
            &pointer,
            source.replacen(
                '{',
                "{\"schema_version\":\"crouzeix-local-formalization-selection/v1\",",
                1,
            ),
        )
        .unwrap();
        assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());
        fs::write(&pointer, &source).unwrap();
        let alias = repo.path().join("pointer-alias");
        fs::hard_link(&pointer, &alias).unwrap();
        assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());
        fs::remove_file(&alias).unwrap();
        fs::rename(&pointer, &alias).unwrap();
        std::os::unix::fs::symlink(&alias, &pointer).unwrap();
        assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());
    }

    #[test]
    fn generation_selection_keeps_active_source_and_historical_integrity_checks() {
        let repo = generation_fixture();
        fs::write(
            repo.path().join("scripts/check_lean_library.sh"),
            "changed wrapper\n",
        )
        .unwrap();
        assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());
        let repo = generation_fixture();
        fs::write(
            local_formalization_bundle(repo.path()).join("build/stderr.log"),
            "tamper\n",
        )
        .unwrap();
        assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());
    }

    #[test]
    fn generation_selection_enforces_catalog_bounds_and_membership() {
        for mutate in [
            (|value: &mut Value| value["generations"] = json!([])) as fn(&mut Value),
            |value| value["generations"][0]["id"] = json!("abcdefabcdefabcdefabcdefabcdefab"),
            |value| value["generations"][1]["extra"] = json!(0),
            |value| value["active_generation"] = json!("ffffffffffffffffffffffffffffffff"),
            |value| value["generations"] = json!(vec![value["generations"][0].clone(); 129]),
        ] {
            let repo = generation_fixture();
            let pointer = selection_path(repo.path());
            let mut value: Value = serde_json::from_slice(&fs::read(&pointer).unwrap()).unwrap();
            mutate(&mut value);
            write_json(&pointer, &value);
            assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());
        }
        let repo = generation_fixture();
        let pointer = selection_path(repo.path());
        let original = fs::read(&pointer).unwrap();
        fs::write(&pointer, [original.as_slice(), &vec![b' '; 65537]].concat()).unwrap();
        assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());
        fs::remove_file(pointer).unwrap();
        assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());
    }

    #[test]
    fn generation_selection_checks_bundle_rosters_and_link_types_for_history() {
        let repo = generation_fixture();
        let legacy = local_formalization_bundle(repo.path());
        fs::write(legacy.join("unregistered"), "unexpected").unwrap();
        let pointer = selection_path(repo.path());
        let mut value: Value = serde_json::from_slice(&fs::read(&pointer).unwrap()).unwrap();
        value["generations"][0]["bundle_sha256"] = json!(fixture_bundle_digest(&legacy));
        write_json(&pointer, &value);
        assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());

        for symlink in [false, true] {
            let repo = generation_fixture();
            let member = local_formalization_bundle(repo.path()).join("build/stdout.log");
            let alias = repo.path().join("bundle-alias");
            if symlink {
                fs::rename(&member, &alias).unwrap();
                std::os::unix::fs::symlink(alias, member).unwrap();
            } else {
                fs::hard_link(member, alias).unwrap();
            }
            assert!(verify_local_formalization_bundle_if_present(repo.path()).is_err());
        }
    }

    #[test]
    fn generation_selection_can_select_legacy_and_returns_complete_catalog_roster() {
        let repo = generation_fixture();
        let pointer = selection_path(repo.path());
        let mut value: Value = serde_json::from_slice(&fs::read(&pointer).unwrap()).unwrap();
        value["active_generation"] = json!("legacy");
        write_json(&pointer, &value);
        let files = verify_local_formalization_bundle_if_present(repo.path())
            .unwrap()
            .unwrap();
        assert!(files.contains("local_formalization.current.json"));
        assert!(files.contains("local_formalization/manifest.tsv"));
        assert!(files.contains(&format!(
            "local_formalization_generations/{GENERATION_ID}/manifest.tsv"
        )));
        assert_eq!(files.len(), 45);
    }

    fn verify_fixture(repo_root: &Path) -> Result<(), AppError> {
        let manifest = Path::new(ROOT).join(LOCAL_FORMALIZATION_MANIFEST);
        verify_local_formalization_bundle_roster(
            &repo_root.join(ROOT).join(LOCAL_FORMALIZATION_ROOT),
            &manifest,
        )?;
        verify_local_formalization_manifest(repo_root, &manifest).map(|_| ())
    }

    fn verify(repo_root: &Path) -> Result<Report, AppError> {
        if local_formalization_bundle(repo_root).exists() {
            verify_fixture(repo_root)?;
            Ok(Report {
                source_receipts: 1,
                verification_receipts: 4,
            })
        } else {
            super::verify(repo_root)
        }
    }

    fn verify_without_required_local_bundle(repo_root: &Path) -> Result<Report, AppError> {
        let route_evidence = route::verify_published_routes(repo_root)
            .map_err(|detail| invalid("sources.crouzeix.route_contracts", detail))?;
        verify_exact_roster(repo_root, None, &route_evidence)?;
        Ok(Report {
            source_receipts: verify_source_manifest(repo_root)?,
            verification_receipts: verify_verification_manifest(repo_root)?,
        })
    }

    fn sha256_file(path: &Path) -> String {
        format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
    }

    fn sha256_bytes(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    fn write_json(path: &Path, value: &Value) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut text = serde_json::to_string_pretty(value).unwrap();
        text.push('\n');
        fs::write(path, text).unwrap();
    }

    const TEST_LS_AGGREGATE_MODULE: &str = "CrouzeixLoristSchwenninger";

    fn canonical_records_digest(kind: &str, key: &str, records: &[(String, String)]) -> String {
        let records = records
            .iter()
            .map(|(identity, digest)| {
                format!(
                    "{{\"{key}\":{},\"sha256\":{}}}",
                    serde_json::to_string(identity).unwrap(),
                    serde_json::to_string(digest).unwrap(),
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        sha256_bytes(format!("{{\"{kind}\":[{records}]}}\n").as_bytes())
    }

    fn test_active_source_paths(repo_root: &Path) -> Vec<PathBuf> {
        let lean_root = repo_root.join("formalization/lean");
        let mut paths = Vec::new();
        let aggregate = lean_root.join("Crouzeix.lean");
        if aggregate.exists() {
            paths.push(aggregate);
        }
        for source_root in [
            lean_root.join("Crouzeix"),
            lean_root.join("CrouzeixConjecture"),
        ] {
            if !source_root.exists() {
                continue;
            }
            paths.extend(
                WalkDir::new(source_root)
                    .follow_links(false)
                    .into_iter()
                    .map(Result::unwrap)
                    .filter(|entry| {
                        entry.file_type().is_file()
                            && entry.path().extension().and_then(|value| value.to_str())
                                == Some("lean")
                    })
                    .map(|entry| entry.into_path()),
            );
        }
        paths.sort();
        paths
    }

    fn test_active_source_closure_digest(repo_root: &Path) -> String {
        let records = test_active_source_paths(repo_root)
            .into_iter()
            .map(|path| {
                (
                    slash_path(path.strip_prefix(repo_root).unwrap()),
                    sha256_file(&path),
                )
            })
            .collect::<Vec<_>>();
        canonical_records_digest("sources", "path", &records)
    }

    fn test_ls_source_closure(repo_root: &Path) -> Vec<(String, PathBuf)> {
        let lean_root = repo_root.join("formalization/lean");
        let mut modules = BTreeMap::new();
        for entry in WalkDir::new(&lean_root).follow_links(false) {
            let entry = entry.unwrap();
            if !entry.file_type().is_file()
                || entry.path().extension().and_then(|value| value.to_str()) != Some("lean")
            {
                continue;
            }
            let relative = entry.path().strip_prefix(&lean_root).unwrap();
            let mut module = relative.to_path_buf();
            module.set_extension("");
            modules.insert(
                module
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("."),
                entry.into_path(),
            );
        }
        let manifest = Path::new("fixture.tsv");
        let mut pending = vec![TEST_LS_AGGREGATE_MODULE.to_owned()];
        let mut visited = BTreeSet::new();
        while let Some(module) = pending.pop() {
            if !visited.insert(module.clone()) {
                continue;
            }
            let path = modules.get(&module).unwrap();
            let source = fs::read_to_string(path).unwrap();
            let active = mask_lean_source(&source, &module, manifest, 1).unwrap();
            for imported in lean_header_imports(&active, &module, manifest, 1).unwrap() {
                if modules.contains_key(&imported) && !visited.contains(&imported) {
                    pending.push(imported);
                }
            }
        }
        visited
            .into_iter()
            .map(|module| {
                let path = modules.remove(&module).unwrap();
                (module, path)
            })
            .collect()
    }

    fn test_ls_source_closure_digest(repo_root: &Path) -> String {
        let records = test_ls_source_closure(repo_root)
            .into_iter()
            .map(|(module, path)| (module, sha256_file(&path)))
            .collect::<Vec<_>>();
        canonical_records_digest("sources", "module", &records)
    }

    fn test_required_mathlib_digest(
        repo_root: &Path,
        sources: impl IntoIterator<Item = (String, PathBuf)>,
    ) -> String {
        let manifest = Path::new("fixture.tsv");
        let mut modules = BTreeSet::new();
        for (module, path) in sources {
            let source = fs::read_to_string(path).unwrap();
            let active = mask_lean_source(&source, &module, manifest, 1).unwrap();
            for imported in lean_header_imports(&active, &module, manifest, 1).unwrap() {
                if imported.starts_with("Mathlib") {
                    modules.insert(imported);
                }
            }
        }
        let artifact_root =
            repo_root.join("formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean");
        let records = modules
            .into_iter()
            .map(|module| {
                let artifact = artifact_root
                    .join(module.replace('.', "/"))
                    .with_extension("olean");
                (module, sha256_file(&artifact))
            })
            .collect::<Vec<_>>();
        canonical_records_digest("artifacts", "module", &records)
    }

    fn test_active_required_mathlib_digest(repo_root: &Path) -> String {
        let lean_root = repo_root.join("formalization/lean");
        test_required_mathlib_digest(
            repo_root,
            test_active_source_paths(repo_root).into_iter().map(|path| {
                let mut module = path.strip_prefix(&lean_root).unwrap().to_path_buf();
                module.set_extension("");
                let module = module
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(".");
                (module, path)
            }),
        )
    }

    fn test_ls_required_mathlib_digest(repo_root: &Path) -> String {
        test_required_mathlib_digest(repo_root, test_ls_source_closure(repo_root))
    }

    fn fixture() -> TempDir {
        let repo = tempfile::tempdir().unwrap();
        let root = repo.path().join(ROOT);
        fs::create_dir_all(root.join("verification")).unwrap();
        fs::write(root.join("PROVENANCE.md"), "# Provenance\n").unwrap();
        fs::write(root.join("acquire.sh"), "#!/bin/sh\nset -eu\n").unwrap();
        fs::write(
            root.join("source_manifest.tsv"),
            format!(
                "{SOURCE_HEADER}\n\
                 crouzeix-source-receipt/v1\tJIN-HEAD-README\tJIN-REPO-HEAD\tgit-artifact\tmetadata\t\
                 git:9df07838327b988e3924453daa29c8cd726d34b0\t\
                 https://github.com/jinshanmu/CrouzeixConjecture/blob/9df07838327b988e3924453daa29c8cd726d34b0/README.md\t\
                 README.md\t357\t{}\t-\t2026-08-14\tnot-present-at-revision\tmetadata-only\n",
                "0".repeat(64),
            ),
        )
        .unwrap();
        let script_digest = sha256_file(&root.join("acquire.sh"));
        let empty_digest = format!("{:x}", Sha256::digest([]));
        let mut verification = format!("{VERIFICATION_HEADER}\n");
        for (receipt, commit, operation, log) in [
            (
                "JIN-565-BUILD",
                "565b6a3e0659b6e0785f783b016c3f6d9f171fa5",
                "lean-build",
                "verification/jin-565b6a3-build.log",
            ),
            (
                "JIN-565-SCAN",
                "565b6a3e0659b6e0785f783b016c3f6d9f171fa5",
                "source-scan",
                "verification/jin-565b6a3-scan.log",
            ),
            (
                "JIN-HEAD-BUILD",
                "9df07838327b988e3924453daa29c8cd726d34b0",
                "lean-build",
                "verification/jin-9df0783-build.log",
            ),
            (
                "JIN-HEAD-SCAN",
                "9df07838327b988e3924453daa29c8cd726d34b0",
                "source-scan",
                "verification/jin-9df0783-scan.log",
            ),
        ] {
            fs::write(repo.path().join(ROOT).join(log), "").unwrap();
            verification.push_str(&format!(
                "crouzeix-verification-receipt/v1\t{receipt}\tJIN-REPO-HEAD\t{commit}\t{}\t{operation}\t{}\t{}\t\
                 crouzeix-log-normalization/v1\tleanprover/lean4:v4.28.0\t\
                 8f9d9cff6bd728b17a24e163c9402775d9e6a365\t2026-08-14T00:00:00Z\t0\tpassed\t{log}\t0\t{empty_digest}\n",
                "1".repeat(40),
                "2".repeat(64),
                script_digest,
            ));
        }
        fs::write(root.join("verification_manifest.tsv"), verification).unwrap();
        repo
    }

    fn replace_source_url(repo_root: &Path, replacement: &str) {
        let path = repo_root.join(ROOT).join("source_manifest.tsv");
        let text = fs::read_to_string(&path).unwrap();
        let mut rows = text.lines();
        let header = rows.next().unwrap();
        let mut fields = rows.next().unwrap().split('\t').collect::<Vec<_>>();
        fields[6] = replacement;
        fs::write(path, format!("{header}\n{}\n", fields.join("\t"))).unwrap();
    }

    fn write_lean_module(repo_root: &Path, module_name: &str, source: &str) {
        let relative = Path::new("formalization/lean")
            .join(module_name.replace('.', "/"))
            .with_extension("lean");
        let path = repo_root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }

    fn write_provider_report(repo_root: &Path, route_id: &str, path: &Path) {
        let (roots, forbidden_modules, forbidden_prefixes) = provider_policy(route_id)
            .unwrap_or_else(|| panic!("unsupported fixture route: {route_id}"));
        let (modules, set_options) = compute_provider_closure(
            repo_root,
            &roots,
            &forbidden_modules,
            &forbidden_prefixes,
            Path::new("fixture.tsv"),
            1,
        )
        .unwrap();
        write_json(
            path,
            &json!({
                "schema_version": "crouzeix-provider-independence/v1",
                "route_id": route_id,
                "status": "passed",
                "roots": roots,
                "forbidden_modules": forbidden_modules,
                "forbidden_prefixes": forbidden_prefixes,
                "modules": modules,
                "set_options": set_options,
            }),
        );
    }

    fn write_ls_receipts(repo_root: &Path) -> (String, String) {
        let reason = "Lean command succeeded; aggregate build and axiom audit passed.";
        let mut nodes = Vec::new();
        let mut terminal_binding = None;
        for spec in LS_NODE_SPECS {
            let attempt_relative = Path::new(
                "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/proof-slices",
            )
            .join(spec.node_id)
            .join("attempt-001");
            let attempt = repo_root.join(&attempt_relative);
            fs::create_dir_all(attempt.join("build")).unwrap();
            write_json(
                &attempt.join("task.json"),
                &json!({
                    "schema_version": "crouzeix-ls-proof-slice-task/v1",
                    "route_id": "lorist-schwenninger",
                    "source_node_id": spec.node_id,
                    "build_target": spec.build_target,
                    "expected_lean_declaration": spec.lean_name,
                    "max_output_bytes": 1_048_576,
                    "timeout_seconds": 3_600,
                }),
            );
            write_json(
                &attempt.join("source-slice.json"),
                &json!({
                    "schema_version": "crouzeix-ls-source-slice/v1",
                    "node_id": spec.node_id,
                    "source_locator": spec.source_locator,
                    "statement_sha256": spec.statement_sha256,
                    "lean_name": spec.lean_name,
                    "dependency_ids": spec.dependencies,
                }),
            );
            write_json(
                &attempt.join("result.json"),
                &json!({
                    "schema_version": "crouzeix-ls-proof-slice-result/v1",
                    "status": "passed",
                    "reason": reason,
                }),
            );
            write_json(
                &attempt.join("build/command.json"),
                &json!({
                    "schema_version": "crouzeix-ls-lean-command/v2",
                    "argv": ["scripts/check_lean_library.sh", LS_BUILD_TARGET],
                    "cwd": ".",
                    "timeout_seconds": 3_600,
                    "output_cap_bytes": 1_048_576,
                    "env": {
                        "ELAN_HOME": LS_ELAN_HOME,
                        "ELAN_TOOLCHAIN": LS_ELAN_TOOLCHAIN,
                        "PATH": LS_TOOLCHAIN_PATH,
                    },
                    "exit_code": 0,
                    "cache_policy": LS_COMMAND_CACHE_POLICY,
                    "wrapper_sha256": sha256_file(
                        &repo_root.join("scripts/check_lean_library.sh"),
                    ),
                    "lake_manifest_sha256": sha256_file(
                        &repo_root.join("formalization/lean/lake-manifest.json"),
                    ),
                    "dependency_cache_metadata_sha256": "2".repeat(64),
                    "required_mathlib_artifacts_sha256": test_ls_required_mathlib_digest(repo_root),
                    "local_source_closure_sha256": test_ls_source_closure_digest(repo_root),
                    "elan_toolchain": LS_ELAN_TOOLCHAIN,
                }),
            );
            fs::write(
                attempt.join("build/stdout.log"),
                format!(
                    "[lean] target={LS_BUILD_TARGET}\n[lean] root=formalization/lean\n[lean] outcome=passed\n"
                ),
            )
            .unwrap();
            fs::write(attempt.join("build/stderr.log"), "").unwrap();
            fs::write(
                attempt.join("build/axioms.txt"),
                format!(
                    "'{}' depends on axioms: [Classical.choice, Quot.sound, propext]\n",
                    spec.lean_name
                ),
            )
            .unwrap();

            let receipt = json!({
                "schema_version": "crouzeix-ls-proof-slice-receipt/v1",
                "route_id": "lorist-schwenninger",
                "source_node_id": spec.node_id,
                "status": "passed",
                "reason": reason,
                "build_target": spec.build_target,
                "expected_lean_declaration": spec.lean_name,
                "allowed_axioms": ["Classical.choice", "Quot.sound", "propext"],
                "observed_axioms": ["Classical.choice", "Quot.sound", "propext"],
                "module_sha256": sha256_file(&repo_root.join(spec.build_target)),
                "task_sha256": sha256_file(&attempt.join("task.json")),
                "source_slice_sha256": sha256_file(&attempt.join("source-slice.json")),
                "result_sha256": sha256_file(&attempt.join("result.json")),
                "command_sha256": sha256_file(&attempt.join("build/command.json")),
                "stdout_sha256": sha256_file(&attempt.join("build/stdout.log")),
                "stderr_sha256": sha256_file(&attempt.join("build/stderr.log")),
                "axiom_audit_sha256": sha256_file(&attempt.join("build/axioms.txt")),
            });
            let receipt_path = attempt.join("receipt.json");
            write_json(&receipt_path, &receipt);
            let receipt_digest = sha256_file(&receipt_path);
            if spec.node_id == "ls-terminal-crouzeix" {
                terminal_binding = Some((
                    slash_path(&attempt_relative.join("receipt.json")),
                    receipt_digest.clone(),
                ));
            }
            nodes.push(json!({
                "node_id": spec.node_id,
                "role": spec.role,
                "source_locator": spec.source_locator,
                "statement_sha256": spec.statement_sha256,
                "dependencies": spec.dependencies,
                "status": "passed",
                "lean_name": spec.lean_name,
                "receipt_sha256": receipt_digest,
            }));
        }
        write_json(
            &repo_root.join(
                "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json",
            ),
            &json!({
                "schema_version": "crouzeix-ls-source-graph/v1",
                "source_id": "LS-ARXIV-V1",
                "source_identity": "arxiv:2608.03841v1",
                "nodes": nodes,
            }),
        );
        terminal_binding.expect("terminal LS receipt fixture")
    }

    fn valid_ls_command_v2(repo_root: &Path) -> Value {
        json!({
            "schema_version": "crouzeix-ls-lean-command/v2",
            "argv": ["scripts/check_lean_library.sh", LS_BUILD_TARGET],
            "cwd": ".",
            "timeout_seconds": 3_600,
            "output_cap_bytes": 1_048_576,
            "env": {
                "ELAN_HOME": "/private/tmp/harp-mathematical-foundations-elan",
                "ELAN_TOOLCHAIN": "leanprover/lean4:v4.32.1",
                "PATH": concat!(
                    "/private/tmp/harp-mathematical-foundations-elan/toolchains/",
                    "leanprover--lean4---v4.32.1/bin:/usr/bin:/bin"
                ),
            },
            "exit_code": 0,
            "cache_policy": "requested cached wrapper command; wrapper policy is repository-controlled",
            "wrapper_sha256": sha256_file(
                &repo_root.join("scripts/check_lean_library.sh"),
            ),
            "lake_manifest_sha256": sha256_file(
                &repo_root.join("formalization/lean/lake-manifest.json"),
            ),
            "dependency_cache_metadata_sha256": "2".repeat(64),
            "required_mathlib_artifacts_sha256": test_ls_required_mathlib_digest(repo_root),
            "local_source_closure_sha256": test_ls_source_closure_digest(repo_root),
            "elan_toolchain": "leanprover/lean4:v4.32.1",
        })
    }

    fn valid_ls_command_v1() -> Value {
        json!({
            "schema_version": "crouzeix-ls-lean-command/v1",
            "argv": ["scripts/check_lean_library.sh", "Crouzeix"],
            "cache_policy": "repository-controlled cached wrapper",
            "cwd": ".",
            "elan_home": LS_ELAN_HOME,
            "exit_code": 0,
        })
    }

    fn replace_ls_command(repo_root: &Path, command: &Value) {
        for spec in LS_NODE_SPECS {
            let attempt = ls_attempt(repo_root, spec.node_id);
            let command_path = attempt.join("build/command.json");
            write_json(&command_path, command);

            let receipt_path = attempt.join("receipt.json");
            let mut receipt: Value =
                serde_json::from_str(&fs::read_to_string(&receipt_path).unwrap()).unwrap();
            receipt["command_sha256"] = json!(sha256_file(&command_path));
            write_json(&receipt_path, &receipt);
            refresh_ls_receipt_binding(repo_root, spec.node_id);
        }
    }

    fn replace_ls_receipts_with_v1_command(repo_root: &Path, command: &Value) {
        for spec in LS_NODE_SPECS {
            let legacy_target = spec.legacy_build_target.unwrap_or(spec.build_target);
            let attempt = ls_attempt(repo_root, spec.node_id);
            let command_path = attempt.join("build/command.json");
            write_json(&command_path, command);

            let task_path = attempt.join("task.json");
            let mut task: Value =
                serde_json::from_str(&fs::read_to_string(&task_path).unwrap()).unwrap();
            task["build_target"] = json!(legacy_target);
            write_json(&task_path, &task);

            fs::write(
                attempt.join("build/stdout.log"),
                "[lean] target=Crouzeix\n[lean] root=formalization/lean\n[lean] outcome=passed\n",
            )
            .unwrap();

            let receipt_path = attempt.join("receipt.json");
            let mut receipt: Value =
                serde_json::from_str(&fs::read_to_string(&receipt_path).unwrap()).unwrap();
            receipt["build_target"] = json!(legacy_target);
            receipt["command_sha256"] = json!(sha256_file(&command_path));
            receipt["module_sha256"] = json!(sha256_file(&repo_root.join(legacy_target)));
            receipt["task_sha256"] = json!(sha256_file(&task_path));
            receipt["stdout_sha256"] = json!(sha256_file(&attempt.join("build/stdout.log")));
            write_json(&receipt_path, &receipt);
            refresh_ls_receipt_binding(repo_root, spec.node_id);
        }
    }

    fn terminal_ls_attempt(repo_root: &Path) -> PathBuf {
        ls_attempt(repo_root, "ls-terminal-crouzeix")
    }

    fn ls_attempt(repo_root: &Path, node_id: &str) -> PathBuf {
        repo_root.join(
            Path::new(
                "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/proof-slices",
            )
            .join(node_id)
            .join("attempt-001"),
        )
    }

    fn ls_graph_path(repo_root: &Path) -> PathBuf {
        repo_root.join(
            "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/\
             source-graph.json",
        )
    }

    fn refresh_local_build_command_binding(repo_root: &Path) {
        let command_path = repo_root.join(LOCAL_BUILD_COMMAND);
        let mut command: Value =
            serde_json::from_str(&fs::read_to_string(&command_path).unwrap()).unwrap();
        command["ls_graph_sha256"] = json!(sha256_file(&ls_graph_path(repo_root)));
        write_json(&command_path, &command);
        set_local_field_for_all(
            repo_root,
            "build_command_sha256",
            &sha256_file(&command_path),
        );
    }

    fn refresh_local_build_source_bindings(repo_root: &Path) {
        let command_path = repo_root.join(LOCAL_BUILD_COMMAND);
        let mut command: Value =
            serde_json::from_str(&fs::read_to_string(&command_path).unwrap()).unwrap();
        command["active_source_closure_sha256"] =
            json!(test_active_source_closure_digest(repo_root));
        command["required_mathlib_artifacts_sha256"] =
            json!(test_active_required_mathlib_digest(repo_root));
        write_json(&command_path, &command);
        set_local_field_for_all(
            repo_root,
            "build_command_sha256",
            &sha256_file(&command_path),
        );
    }

    fn refresh_ls_provider_binding(repo_root: &Path) {
        let provider_path = repo_root.join(
            "evidence/crouzeix_conjecture/local_formalization/providers/lorist-schwenninger.json",
        );
        write_provider_report(repo_root, "lorist-schwenninger", &provider_path);
        set_local_field(
            repo_root,
            "ls-closed-numerical-range",
            "provider_independence_sha256",
            &sha256_file(&provider_path),
        );
        set_local_field(
            repo_root,
            "ls-main-theorem",
            "provider_independence_sha256",
            &sha256_file(&provider_path),
        );
    }

    fn refresh_terminal_ls_receipt_binding(repo_root: &Path) -> String {
        refresh_ls_receipt_binding(repo_root, "ls-terminal-crouzeix")
    }

    fn refresh_ls_receipt_binding(repo_root: &Path, node_id: &str) -> String {
        let receipt_path = ls_attempt(repo_root, node_id).join("receipt.json");
        let receipt_digest = sha256_file(&receipt_path);

        let graph_path = ls_graph_path(repo_root);
        let mut graph: Value =
            serde_json::from_str(&fs::read_to_string(&graph_path).unwrap()).unwrap();
        let node = graph["nodes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|node| node["node_id"] == node_id)
            .unwrap();
        node["receipt_sha256"] = json!(receipt_digest);
        write_json(&graph_path, &graph);
        refresh_local_build_command_binding(repo_root);
        receipt_digest
    }

    fn replace_ls_member(
        repo_root: &Path,
        node_id: &str,
        member: &str,
        digest_field: &str,
        bytes: &[u8],
    ) {
        let attempt = ls_attempt(repo_root, node_id);
        let member_path = attempt.join(member);
        fs::write(&member_path, bytes).unwrap();

        let receipt_path = attempt.join("receipt.json");
        let mut receipt: Value =
            serde_json::from_str(&fs::read_to_string(&receipt_path).unwrap()).unwrap();
        receipt[digest_field] = json!(sha256_file(&member_path));
        write_json(&receipt_path, &receipt);
        refresh_ls_receipt_binding(repo_root, node_id);
    }

    fn replace_local_build_stdout(repo_root: &Path, bytes: &[u8]) {
        let stdout_path = repo_root.join(LOCAL_BUILD_STDOUT);
        fs::write(&stdout_path, bytes).unwrap();
        let stdout_digest = sha256_file(&stdout_path);

        let command_path = repo_root.join(LOCAL_BUILD_COMMAND);
        let mut command: Value =
            serde_json::from_str(&fs::read_to_string(&command_path).unwrap()).unwrap();
        command["stdout_sha256"] = json!(stdout_digest);
        write_json(&command_path, &command);
        set_local_field_for_all(repo_root, "build_stdout_sha256", &stdout_digest);
        set_local_field_for_all(
            repo_root,
            "build_command_sha256",
            &sha256_file(&command_path),
        );
    }

    fn replace_ls_result_reason(repo_root: &Path, reason: &str) {
        let attempt = terminal_ls_attempt(repo_root);
        let result_path = attempt.join("result.json");
        let mut result: Value =
            serde_json::from_str(&fs::read_to_string(&result_path).unwrap()).unwrap();
        result["reason"] = json!(reason);
        write_json(&result_path, &result);

        let receipt_path = attempt.join("receipt.json");
        let mut receipt: Value =
            serde_json::from_str(&fs::read_to_string(&receipt_path).unwrap()).unwrap();
        receipt["result_sha256"] = json!(sha256_file(&result_path));
        write_json(&receipt_path, &receipt);
        refresh_terminal_ls_receipt_binding(repo_root);
    }

    fn rejected_v2_command_message(mutate: impl FnOnce(&mut Value)) -> String {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let mut command = valid_ls_command_v2(repo.path());
        mutate(&mut command);
        replace_ls_command(repo.path(), &command);

        verify(repo.path()).unwrap_err().message
    }

    fn replace_local_build_command(repo_root: &Path, command: &Value) {
        let command_path = repo_root.join(LOCAL_BUILD_COMMAND);
        write_json(&command_path, command);
        set_local_field_for_all(
            repo_root,
            "build_command_sha256",
            &sha256_file(&command_path),
        );
    }

    fn rejected_local_command_message(mutate: impl FnOnce(&mut Value)) -> String {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let command_path = repo.path().join(LOCAL_BUILD_COMMAND);
        let mut command: Value =
            serde_json::from_str(&fs::read_to_string(command_path).unwrap()).unwrap();
        mutate(&mut command);
        replace_local_build_command(repo.path(), &command);

        verify(repo.path()).unwrap_err().message
    }

    fn mutate_ls_graph(repo_root: &Path, mutate: impl FnOnce(&mut Value)) {
        let path = ls_graph_path(repo_root);
        let mut graph: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        mutate(&mut graph);
        write_json(&path, &graph);
        refresh_local_build_command_binding(repo_root);
    }

    fn local_formalization_bundle(repo_root: &Path) -> PathBuf {
        repo_root.join(ROOT).join("local_formalization")
    }

    fn local_formalization_manifest(repo_root: &Path) -> PathBuf {
        local_formalization_bundle(repo_root).join("manifest.tsv")
    }

    fn fixture_route_manifest_path(route_id: &str) -> &'static str {
        match route_id {
            "jin" => "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json",
            "harp" => "labs/crouzeix_proof_reproduction/formal_targets/harp/route-manifest.json",
            "lorist-schwenninger" => {
                "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/route-manifest.json"
            }
            other => panic!("unsupported route fixture: {other}"),
        }
    }

    fn fixture_route_receipt_path(route_id: &str) -> String {
        format!("evidence/crouzeix_conjecture/routes/{route_id}/receipt.json")
    }

    fn fixture_route_review_path(route_id: &str) -> String {
        format!("evidence/crouzeix_conjecture/reviews/{route_id}.json")
    }

    fn local_route_snapshot_path(route_id: &str, kind: &str) -> String {
        format!("evidence/crouzeix_conjecture/local_formalization/routes/{route_id}.{kind}.json")
    }

    fn write_route_snapshot(repo_root: &Path, route_id: &str, kind: &str, source_path: &str) {
        let snapshot_path = local_route_snapshot_path(route_id, kind);
        let source = repo_root.join(source_path);
        let destination = repo_root.join(&snapshot_path);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(source, destination).unwrap();
    }

    fn write_fixture_route_publications(repo_root: &Path) {
        for route_id in ["jin", "harp", "lorist-schwenninger"] {
            let manifest_path = fixture_route_manifest_path(route_id);
            let receipt_path = fixture_route_receipt_path(route_id);
            let review_path = fixture_route_review_path(route_id);

            fs::create_dir_all(repo_root.join(Path::new(manifest_path).parent().unwrap())).unwrap();
            fs::create_dir_all(repo_root.join(Path::new(&receipt_path).parent().unwrap())).unwrap();
            fs::create_dir_all(repo_root.join(Path::new(&review_path).parent().unwrap())).unwrap();

            fs::write(
                repo_root.join(manifest_path),
                format!(
                    concat!(
                        "{{\n",
                        "  \"schema_version\": \"crouzeix-route-proof-manifest/v1\",\n",
                        "  \"route_id\": \"{route_id}\",\n",
                        "  \"claim_kind\": \"derived\",\n",
                        "  \"aggregate_module\": \"{aggregate}\",\n",
                        "  \"build_target\": \"{aggregate}\",\n",
                        "  \"terminal_declaration\": \"Fixture.{route_id}.terminal\",\n",
                        "  \"terminal_type_sha256\": \"{digest}\",\n",
                        "  \"consequence_declarations\": [],\n",
                        "  \"source_identities\": [],\n",
                        "  \"shared_foundation_modules\": [\"{aggregate}\"],\n",
                        "  \"module_closure\": [\"{aggregate}\"],\n",
                        "  \"module_closure_sha256\": \"{digest}\",\n",
                        "  \"allowed_axioms\": [\"Classical.choice\", \"Quot.sound\", \"propext\"],\n",
                        "  \"review_path\": \"{review_path}\",\n",
                        "  \"review_sha256\": \"{digest}\",\n",
                        "  \"receipt_path\": \"{receipt_path}\",\n",
                        "  \"receipt_sha256\": \"{digest}\",\n",
                        "  \"route_dependencies\": [],\n",
                        "  \"nodes\": [\n",
                        "    {{\n",
                        "      \"node_id\": \"terminal\",\n",
                        "      \"role\": \"terminal\",\n",
                        "      \"declaration\": \"Fixture.{route_id}.terminal\",\n",
                        "      \"module_path\": \"formalization/lean/{aggregate}.lean\",\n",
                        "      \"dependency_ids\": [],\n",
                        "      \"provenance_kind\": \"derived\",\n",
                        "      \"correspondence_kind\": \"derived-extraction\",\n",
                        "      \"source_locator\": null,\n",
                        "      \"source_archive_sha256\": null,\n",
                        "      \"source_file_sha256\": null,\n",
                        "      \"source_excerpt_sha256\": null,\n",
                        "      \"source_line_count\": null,\n",
                        "      \"reused_from_route\": null,\n",
                        "      \"reused_node_id\": null,\n",
                        "      \"declaration_type_path\": \"evidence/crouzeix_conjecture/routes/{route_id}/types/main.txt\",\n",
                        "      \"statement_sha256\": \"{digest}\"\n",
                        "    }}\n",
                        "  ]\n",
                        "}}\n"
                    ),
                    route_id = route_id,
                    aggregate = match route_id {
                        "jin" => "CrouzeixJin",
                        "harp" => "CrouzeixHarp",
                        "lorist-schwenninger" => "CrouzeixLoristSchwenninger",
                        other => panic!("unsupported route fixture: {other}"),
                    },
                    receipt_path = receipt_path,
                    review_path = review_path,
                    digest = "a".repeat(64),
                ),
            )
            .unwrap();
            fs::write(
                repo_root.join(&receipt_path),
                format!(
                    "{{\n  \"schema_version\": \"crouzeix-route-proof-receipt/v1\",\n  \"route_id\": \"{route_id}\",\n  \"aggregate_module\": \"Fixture\",\n  \"build_target\": \"Fixture\",\n  \"manifest_path\": \"{manifest_path}\",\n  \"manifest_sha256\": \"{digest}\",\n  \"candidate_commit\": \"{git_id}\",\n  \"candidate_tree\": \"{git_id}\",\n  \"command_artifact_path\": \"evidence/crouzeix_conjecture/routes/{route_id}/command.json\",\n  \"command_artifact_sha256\": \"{digest}\",\n  \"argv\": [\"scripts/check_lean_library.sh\", \"Fixture\"],\n  \"working_directory\": \".\",\n  \"cache_identity\": \"sha256:{digest}\",\n  \"toolchain\": \"leanprover/lean4:v4.32.1\",\n  \"local_closure_modules\": [\"Fixture\"],\n  \"local_closure_sha256\": \"{digest}\",\n  \"mathlib_artifacts\": [],\n  \"mathlib_artifacts_sha256\": \"{digest}\",\n  \"declaration_types\": [],\n  \"allowed_axioms\": [\"Classical.choice\", \"Quot.sound\", \"propext\"],\n  \"axiom_audit_path\": \"evidence/crouzeix_conjecture/routes/{route_id}/axioms.json\",\n  \"axiom_audit_sha256\": \"{digest}\",\n  \"axiom_results\": [],\n  \"provider_report_path\": \"evidence/crouzeix_conjecture/routes/{route_id}/provider.json\",\n  \"provider_report_sha256\": \"{digest}\",\n  \"stdout_path\": \"evidence/crouzeix_conjecture/routes/{route_id}/stdout.log\",\n  \"stdout_sha256\": \"{digest}\",\n  \"stderr_path\": \"evidence/crouzeix_conjecture/routes/{route_id}/stderr.log\",\n  \"stderr_sha256\": \"{digest}\",\n  \"exit_code\": 0,\n  \"status\": \"passed\",\n  \"receipt_sha256\": \"{digest}\"\n}}\n",
                    route_id = route_id,
                    manifest_path = manifest_path,
                    digest = "b".repeat(64),
                    git_id = "1".repeat(40),
                ),
            )
            .unwrap();
            fs::write(
                repo_root.join(&review_path),
                format!(
                    "{{\n  \"schema_version\": \"crouzeix-proof-review/v1\",\n  \"route_id\": \"{route_id}\",\n  \"reviewed_commit\": \"{git_id}\",\n  \"reviewed_tree\": \"{git_id}\",\n  \"manifest_path\": \"{manifest_path}\",\n  \"manifest_sha256\": \"{digest}\",\n  \"terminal_type_sha256\": \"{digest}\",\n  \"review_id\": \"review-{route_id}-001\",\n  \"reviewer_identity\": \"fixture\",\n  \"reviewer_model\": \"fixture-model\",\n  \"reviewer_run_id\": \"fixture-run\",\n  \"verdict\": \"complete\",\n  \"outcome\": \"approved\",\n  \"source_fidelity_check\": \"not-applicable\",\n  \"derivation_reuse_check\": \"passed\",\n  \"findings\": [],\n  \"review_sha256\": \"{digest}\"\n}}\n",
                    route_id = route_id,
                    manifest_path = manifest_path,
                    digest = "c".repeat(64),
                    git_id = "1".repeat(40),
                ),
            )
            .unwrap();

            write_route_snapshot(repo_root, route_id, "manifest", manifest_path);
            write_route_snapshot(repo_root, route_id, "receipt", &receipt_path);
            write_route_snapshot(repo_root, route_id, "review", &review_path);
        }
    }

    fn write_local_formalization_manifest(repo_root: &Path) {
        let modules = [
            (
                "Crouzeix.Harp.Consequences",
                "import Crouzeix.Harp.MainTheorem\n\ntheorem CrouzeixConjecture.harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet : True := by\n  trivial\n",
            ),
            (
                "Crouzeix.Harp.FiniteAtomicL2Dilation",
                "import Crouzeix.Harp.Support\n\ndef CrouzeixConjecture.Harp.finiteAtomicL2Dilation : True := True\n",
            ),
            (
                "Crouzeix.Harp.FiniteHorizonPerturbation",
                "import Crouzeix.Harp.Support\n\ndef CrouzeixConjecture.Harp.finiteHorizonPerturbation : True := True\n",
            ),
            (
                "Crouzeix.Harp.MainTheorem",
                "import Crouzeix.Harp.Support\n\ntheorem CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem : True := by\n  trivial\n",
            ),
            (
                "Crouzeix.Harp.Support",
                "import Mathlib.Fixture\n\n-- sorry in a comment is inert\nset_option autoImplicit false\n\ndef CrouzeixConjecture.Harp.helper' : True := True\n",
            ),
            (
                "Crouzeix.Jin.Support",
                "import Mathlib.Fixture\n\ndef CrouzeixConjecture.Jin.supportWitness : True := True\n",
            ),
            (
                "Crouzeix.Jin.Terminal",
                "import Crouzeix.Jin.Support\n\ntheorem CrouzeixConjecture.crouzeixConjecture : True := by\n  trivial\n",
            ),
            (
                "CrouzeixJin",
                "import Crouzeix.Jin.Terminal\nimport CrouzeixConjecture.HilbertSpectralSet\n",
            ),
            (
                "CrouzeixConjecture.HilbertSpectralSetCore",
                "import Crouzeix.Jin.Terminal\n\ntheorem CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem : True := by\n  trivial\n",
            ),
            (
                "CrouzeixConjecture.HilbertSpectralSet",
                "import CrouzeixConjecture.HilbertSpectralSetCore\n\ntheorem CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet : True := by\n  trivial\n",
            ),
            (
                "Crouzeix.LoristSchwenninger.Dilation",
                "def CrouzeixConjecture.LoristSchwenninger.dilation : True := True\n",
            ),
            (
                "Crouzeix.LoristSchwenninger.Perturbation",
                "def CrouzeixConjecture.LoristSchwenninger.legacyPerturbation : True := True\n",
            ),
            (
                "Crouzeix.LoristSchwenninger.OperatorRecurrence",
                "def CrouzeixConjecture.LoristSchwenninger.operatorRecurrence : True := True\n",
            ),
            (
                "Crouzeix.LoristSchwenninger.Scalar",
                "def CrouzeixConjecture.LoristSchwenninger.scalar : True := True\n",
            ),
            (
                "Crouzeix.LoristSchwenninger.PerturbationLemma",
                "def CrouzeixConjecture.LoristSchwenninger.perturbationLemma : True := True\n",
            ),
            (
                "Crouzeix.LoristSchwenninger.ConcreteDilation",
                "def CrouzeixConjecture.LoristSchwenninger.concreteDilation : True := True\n",
            ),
            (
                "Crouzeix.LoristSchwenninger.Consequences",
                "import Crouzeix.LoristSchwenninger.MainTheorem\n\ntheorem CrouzeixConjecture.loristSchwenningerClosedOperatorNumericalRange_isTwoSpectralSet : True := by\n  trivial\n",
            ),
            (
                "Crouzeix.LoristSchwenninger.MainTheorem",
                "import Crouzeix.LoristSchwenninger.Support\n\ntheorem CrouzeixConjecture.loristSchwenningerMainTheorem : True := by\n  trivial\n",
            ),
            (
                "Crouzeix.LoristSchwenninger.Support",
                "import Mathlib.Fixture\n\ndef CrouzeixConjecture.LoristSchwenninger.message : String := \"unsafe sorry axiom\"\n",
            ),
        ];
        for (module_name, source) in modules {
            write_lean_module(repo_root, module_name, source);
        }

        let lean_root = repo_root.join("formalization/lean");
        fs::write(
            lean_root.join("Crouzeix.lean"),
            "import Crouzeix.Harp.Consequences\nimport Crouzeix.Jin.Terminal\nimport CrouzeixConjecture.HilbertSpectralSet\nimport Crouzeix.LoristSchwenninger.ConcreteDilation\nimport Crouzeix.LoristSchwenninger.Dilation\nimport Crouzeix.LoristSchwenninger.MainTheorem\nimport Crouzeix.LoristSchwenninger.OperatorRecurrence\nimport Crouzeix.LoristSchwenninger.Perturbation\nimport Crouzeix.LoristSchwenninger.PerturbationLemma\nimport Crouzeix.LoristSchwenninger.Scalar\n",
        )
        .unwrap();
        fs::write(
            lean_root.join("CrouzeixJin.lean"),
            "import Crouzeix.Jin.Terminal\nimport CrouzeixConjecture.HilbertSpectralSet\n",
        )
        .unwrap();
        fs::write(
            lean_root.join("CrouzeixLoristSchwenninger.lean"),
            "import Crouzeix.LoristSchwenninger.Consequences\nimport Crouzeix.LoristSchwenninger.ConcreteDilation\nimport Crouzeix.LoristSchwenninger.Dilation\nimport Crouzeix.LoristSchwenninger.OperatorRecurrence\nimport Crouzeix.LoristSchwenninger.Perturbation\nimport Crouzeix.LoristSchwenninger.PerturbationLemma\nimport Crouzeix.LoristSchwenninger.Scalar\n",
        )
        .unwrap();
        fs::write(
            lean_root.join("lean-toolchain"),
            "leanprover/lean4:v4.32.1\n",
        )
        .unwrap();
        fs::write(
            lean_root.join("lakefile.toml"),
            "name = \"crouzeix_fixture\"\n",
        )
        .unwrap();
        write_json(
            &lean_root.join("lake-manifest.json"),
            &json!({"version": "1.1.0", "packages": []}),
        );
        let mathlib_artifact =
            lean_root.join(".lake/packages/mathlib/.lake/build/lib/lean/Mathlib/Fixture.olean");
        fs::create_dir_all(mathlib_artifact.parent().unwrap()).unwrap();
        fs::write(&mathlib_artifact, b"fixture Mathlib object bytes\n").unwrap();
        fs::create_dir_all(repo_root.join("scripts")).unwrap();
        fs::write(
            repo_root.join("scripts/check_lean_library.sh"),
            "#!/bin/sh\nset -eu\n",
        )
        .unwrap();
        let toolchain_digest = sha256_file(&lean_root.join("lean-toolchain"));
        let lake_manifest_digest = sha256_file(&lean_root.join("lake-manifest.json"));
        write_ls_receipts(repo_root);
        let ls_graph = repo_root.join(
            "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json",
        );

        let evidence = local_formalization_bundle(repo_root);
        fs::create_dir_all(evidence.join("build")).unwrap();
        fs::create_dir_all(evidence.join("axioms")).unwrap();
        fs::create_dir_all(evidence.join("providers")).unwrap();
        fs::create_dir_all(evidence.join("routes")).unwrap();
        let stdout_path = evidence.join("build/stdout.log");
        let stderr_path = evidence.join("build/stderr.log");
        fs::write(
            &stdout_path,
            "[lean] target=Crouzeix\n[lean] root=formalization/lean\n[lean] outcome=passed\n",
        )
        .unwrap();
        fs::write(&stderr_path, "").unwrap();
        let build_command_path = evidence.join("build/command.json");
        write_json(
            &build_command_path,
            &json!({
                "schema_version": "crouzeix-local-build-command/v1",
                "argv": ["scripts/check_lean_library.sh", "Crouzeix"],
                "cwd": ".",
                "lean_toolchain": "leanprover/lean4:v4.32.1",
                "lean_toolchain_sha256": toolchain_digest,
                "lake_manifest_sha256": lake_manifest_digest,
                "wrapper_sha256": sha256_file(&repo_root.join("scripts/check_lean_library.sh")),
                "lakefile_sha256": sha256_file(&lean_root.join("lakefile.toml")),
                "active_source_closure_sha256": test_active_source_closure_digest(repo_root),
                "dependency_cache_metadata_sha256": "2".repeat(64),
                "required_mathlib_artifacts_sha256": test_active_required_mathlib_digest(repo_root),
                "ls_graph_sha256": sha256_file(&ls_graph),
                "exit_code": 0,
                "status": "passed",
                "stdout_path": LOCAL_BUILD_STDOUT,
                "stdout_sha256": sha256_file(&stdout_path),
                "stderr_path": LOCAL_BUILD_STDERR,
                "stderr_sha256": sha256_file(&stderr_path),
            }),
        );

        for spec in FORMALIZATION_SPECS {
            fs::write(
                evidence.join(format!("axioms/{}.txt", spec.id)),
                format!(
                    "'{}' depends on axioms: [Classical.choice, Quot.sound, propext]\n",
                    spec.declaration
                ),
            )
            .unwrap();
        }
        write_provider_report(repo_root, "harp", &evidence.join("providers/harp.json"));
        write_provider_report(repo_root, "jin", &evidence.join("providers/jin.json"));
        write_provider_report(
            repo_root,
            "lorist-schwenninger",
            &evidence.join("providers/lorist-schwenninger.json"),
        );
        write_fixture_route_publications(repo_root);

        let command_path = LOCAL_BUILD_COMMAND;
        let stdout_path = LOCAL_BUILD_STDOUT;
        let stderr_path = LOCAL_BUILD_STDERR;
        let mut rows = Vec::new();
        for spec in FORMALIZATION_SPECS {
            let axiom_path = format!(
                "evidence/crouzeix_conjecture/local_formalization/axioms/{}.txt",
                spec.id
            );
            let provider_path = format!(
                "evidence/crouzeix_conjecture/local_formalization/providers/{}.json",
                spec.route
            );
            let route_manifest_snapshot_path = local_route_snapshot_path(spec.route, "manifest");
            let route_receipt_snapshot_path = local_route_snapshot_path(spec.route, "receipt");
            let route_review_snapshot_path = local_route_snapshot_path(spec.route, "review");
            rows.push(
                [
                    LOCAL_FORMALIZATION_SCHEMA.to_owned(),
                    spec.id.to_owned(),
                    spec.route.to_owned(),
                    spec.source_node.to_owned(),
                    spec.declaration.to_owned(),
                    spec.module_path.to_owned(),
                    sha256_file(&repo_root.join(spec.module_path)),
                    route_manifest_snapshot_path.clone(),
                    sha256_file(&repo_root.join(&route_manifest_snapshot_path)),
                    route_receipt_snapshot_path.clone(),
                    sha256_file(&repo_root.join(&route_receipt_snapshot_path)),
                    route_review_snapshot_path.clone(),
                    sha256_file(&repo_root.join(&route_review_snapshot_path)),
                    command_path.to_owned(),
                    sha256_file(&repo_root.join(command_path)),
                    stdout_path.to_owned(),
                    sha256_file(&repo_root.join(stdout_path)),
                    stderr_path.to_owned(),
                    sha256_file(&repo_root.join(stderr_path)),
                    axiom_path.clone(),
                    sha256_file(&repo_root.join(&axiom_path)),
                    ALLOWED_AXIOMS.to_owned(),
                    ALLOWED_AXIOMS.to_owned(),
                    provider_path.clone(),
                    sha256_file(&repo_root.join(&provider_path)),
                    "leanprover/lean4:v4.32.1".to_owned(),
                    toolchain_digest.clone(),
                    lake_manifest_digest.clone(),
                    "passed".to_owned(),
                ]
                .join("\t"),
            );
        }
        fs::write(
            local_formalization_manifest(repo_root),
            format!("{LOCAL_FORMALIZATION_HEADER}\n{}\n", rows.join("\n")),
        )
        .unwrap();
    }

    fn mutate_local_formalization_manifest(
        repo_root: &Path,
        mutate: impl FnOnce(&mut Vec<String>, &mut Vec<Vec<String>>),
    ) {
        let path = local_formalization_manifest(repo_root);
        let text = fs::read_to_string(&path).unwrap();
        let mut rows = text
            .lines()
            .map(|line| line.split('\t').map(str::to_owned).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut header = rows.remove(0);
        mutate(&mut header, &mut rows);
        let output = std::iter::once(header)
            .chain(rows)
            .map(|row| row.join("\t"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, format!("{output}\n")).unwrap();
    }

    fn local_field_index(name: &str) -> usize {
        LOCAL_FORMALIZATION_HEADER
            .split('\t')
            .position(|field| field == name)
            .unwrap()
    }

    fn set_local_field(repo_root: &Path, formalization_id: &str, name: &str, value: &str) {
        let id_index = local_field_index("formalization_id");
        let index = local_field_index(name);
        mutate_local_formalization_manifest(repo_root, |_, rows| {
            let row = rows
                .iter_mut()
                .find(|row| row[id_index] == formalization_id)
                .unwrap();
            row[index] = value.to_owned();
        });
    }

    fn set_local_field_for_all(repo_root: &Path, name: &str, value: &str) {
        let index = local_field_index(name);
        mutate_local_formalization_manifest(repo_root, |_, rows| {
            for row in rows {
                row[index] = value.to_owned();
            }
        });
    }

    #[test]
    fn rejects_unknown_files_and_symlinks_in_the_evidence_root() {
        let repo = fixture();
        fs::write(
            repo.path()
                .join("evidence/crouzeix_conjecture/copied-source.lean"),
            "theorem copied : True := by trivial\n",
        )
        .unwrap();

        let error = verify_without_required_local_bundle(repo.path()).unwrap_err();
        assert_eq!(error.code(), "sources.crouzeix.roster");
    }

    #[test]
    fn source_manifest_requires_identity_bound_urls_and_unique_receipts() {
        let repo = fixture();
        replace_source_url(
            repo.path(),
            "https://github.com/jinshanmu/CrouzeixConjecture/blob/main/README.md",
        );

        let error = verify_without_required_local_bundle(repo.path()).unwrap_err();
        assert_eq!(error.code(), "sources.crouzeix.source_manifest");
    }

    #[test]
    fn verification_receipts_bind_commands_logs_and_acquisition_script() {
        let repo = fixture();
        fs::write(
            repo.path().join("evidence/crouzeix_conjecture/acquire.sh"),
            "#!/bin/sh\nexit 0\n# changed\n",
        )
        .unwrap();

        let error = verify_without_required_local_bundle(repo.path()).unwrap_err();
        assert_eq!(error.code(), "sources.crouzeix.verification_manifest");
    }

    #[test]
    fn local_formalization_manifest_is_required_after_publication() {
        let repo = fixture();

        let error = super::verify(repo.path()).unwrap_err();
        assert_eq!(error.code(), LOCAL_FORMALIZATION_CODE);
        assert!(error
            .message
            .contains("required local formalization bundle is missing"));
    }

    #[test]
    fn local_formalization_bundle_cannot_be_partially_materialized() {
        let repo = fixture();
        fs::create_dir_all(local_formalization_bundle(repo.path()).join("build")).unwrap();

        let error = verify(repo.path()).unwrap_err();

        assert_eq!(error.code(), LOCAL_FORMALIZATION_CODE);
        assert!(error.message.contains("bundle roster"), "{}", error.message);
    }

    #[test]
    fn local_formalization_bundle_rejects_unknown_files_and_directories() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        fs::write(
            local_formalization_bundle(repo.path()).join("unexpected.txt"),
            "not part of the bundle\n",
        )
        .unwrap();

        let error = verify(repo.path()).unwrap_err();
        assert_eq!(error.code(), LOCAL_FORMALIZATION_CODE);
        assert!(error.message.contains("bundle roster"), "{}", error.message);

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        fs::create_dir(local_formalization_bundle(repo.path()).join("scratch")).unwrap();

        let error = verify(repo.path()).unwrap_err();
        assert_eq!(error.code(), LOCAL_FORMALIZATION_CODE);
        assert!(error.message.contains("bundle roster"), "{}", error.message);
    }

    #[cfg(unix)]
    #[test]
    fn local_formalization_bundle_rejects_symlinked_directories() {
        use std::os::unix::fs::symlink;

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let bundle = local_formalization_bundle(repo.path());
        let outside = repo.path().join("outside-build");
        fs::rename(bundle.join("build"), &outside).unwrap();
        symlink(&outside, bundle.join("build")).unwrap();

        let error = verify(repo.path()).unwrap_err();
        assert_eq!(error.code(), LOCAL_FORMALIZATION_CODE);
        assert!(error.message.contains("symlink"), "{}", error.message);
    }

    #[test]
    fn accepts_a_hash_bound_local_formalization_manifest() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());

        verify(repo.path()).unwrap();
    }

    #[test]
    fn local_formalization_roster_includes_route_snapshots() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let manifest = Path::new(ROOT).join(LOCAL_FORMALIZATION_MANIFEST);

        let files = verify_local_formalization_manifest(repo.path(), &manifest).unwrap();

        for route in ["harp", "jin", "lorist-schwenninger"] {
            for kind in ["manifest", "receipt", "review"] {
                assert!(
                    files.contains(&format!("local_formalization/routes/{route}.{kind}.json")),
                    "missing {route} {kind} snapshot from the top-level evidence roster"
                );
            }
        }
    }

    #[test]
    fn local_build_command_requires_exact_18_field_schema() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let command_path = local_formalization_bundle(repo.path()).join("build/command.json");
        let command: Value =
            serde_json::from_str(&fs::read_to_string(command_path).unwrap()).unwrap();

        assert_eq!(command.as_object().unwrap().len(), 18);
        verify(repo.path()).unwrap();
    }

    #[test]
    fn python_parity_local_build_command_rejects_missing_unknown_and_wrong_type_fields() {
        for field in [
            "schema_version",
            "argv",
            "cwd",
            "lean_toolchain",
            "lean_toolchain_sha256",
            "lake_manifest_sha256",
            "wrapper_sha256",
            "lakefile_sha256",
            "active_source_closure_sha256",
            "dependency_cache_metadata_sha256",
            "required_mathlib_artifacts_sha256",
            "ls_graph_sha256",
            "exit_code",
            "status",
            "stdout_path",
            "stdout_sha256",
            "stderr_path",
            "stderr_sha256",
        ] {
            let message = rejected_local_command_message(|command| {
                command.as_object_mut().unwrap().remove(field);
            });
            assert!(
                message.contains("invalid aggregate build command JSON"),
                "missing {field}: {message}"
            );
        }

        let message = rejected_local_command_message(|command| {
            command["unknown"] = json!("field");
        });
        assert!(
            message.contains("invalid aggregate build command JSON"),
            "{message}"
        );

        let message = rejected_local_command_message(|command| {
            command["exit_code"] = json!("0");
        });
        assert!(
            message.contains("invalid aggregate build command JSON"),
            "{message}"
        );
    }

    type PathMutation = (&'static str, fn(&Path));

    #[test]
    fn python_parity_local_build_command_binds_new_digest_surfaces() {
        let mutations: [PathMutation; 3] = [
            ("wrapper", |root| {
                fs::write(root.join("scripts/check_lean_library.sh"), "changed\n").unwrap();
            }),
            ("lakefile", |root| {
                fs::write(root.join("formalization/lean/lakefile.toml"), "changed\n").unwrap();
            }),
            ("source graph", |root| {
                let path = ls_graph_path(root);
                let mut bytes = fs::read(&path).unwrap();
                bytes.push(b' ');
                fs::write(path, bytes).unwrap();
            }),
        ];
        for (case, mutate) in mutations {
            let repo = fixture();
            write_local_formalization_manifest(repo.path());
            mutate(repo.path());

            let error = verify(repo.path()).unwrap_err();
            assert!(
                error.message.contains("digest mismatch"),
                "{case}: {}",
                error.message
            );
        }

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        write_lean_module(
            repo.path(),
            "Crouzeix.Untracked",
            "def Crouzeix.Untracked.value : True := True\n",
        );
        let error = verify(repo.path()).unwrap_err();
        assert!(
            error
                .message
                .contains("source closure or required Mathlib artifact digest mismatch"),
            "{}",
            error.message
        );

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        fs::write(
            repo.path().join(
                "formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean/Mathlib/Fixture.olean",
            ),
            b"changed Mathlib object bytes\n",
        )
        .unwrap();
        let error = verify(repo.path()).unwrap_err();
        assert!(
            error
                .message
                .contains("source closure or required Mathlib artifact digest mismatch"),
            "{}",
            error.message
        );
    }

    #[test]
    fn accepts_v2_ls_command_receipts() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let command = valid_ls_command_v2(repo.path());
        assert_eq!(command.as_object().unwrap().len(), 14);
        replace_ls_command(repo.path(), &command);

        verify(repo.path()).unwrap();
    }

    #[test]
    fn accepts_historical_v1_ls_command_receipts() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_receipts_with_v1_command(repo.path(), &valid_ls_command_v1());

        verify(repo.path()).unwrap();
    }

    #[test]
    fn python_parity_v1_ls_receipt_accepts_exact_legacy_target() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_receipts_with_v1_command(repo.path(), &valid_ls_command_v1());

        verify(repo.path()).unwrap();
    }

    #[test]
    fn schema_versions_reject_each_others_aggregate_target() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let mut command = valid_ls_command_v1();
        command["argv"] = json!(["scripts/check_lean_library.sh", LS_BUILD_TARGET]);
        replace_ls_receipts_with_v1_command(repo.path(), &command);

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error
                .message
                .contains("LS command is not the successful aggregate Crouzeix build"),
            "{}",
            error.message
        );

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let mut command = valid_ls_command_v2(repo.path());
        command["argv"] = json!(["scripts/check_lean_library.sh", "Crouzeix"]);
        replace_ls_command(repo.path(), &command);
        for spec in LS_NODE_SPECS {
            replace_ls_member(
                repo.path(),
                spec.node_id,
                "build/stdout.log",
                "stdout_sha256",
                b"[lean] target=Crouzeix\n[lean] root=formalization/lean\n[lean] outcome=passed\n",
            );
        }

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error
                .message
                .contains("LS command is not the successful aggregate Crouzeix build"),
            "{}",
            error.message
        );
    }

    #[test]
    fn schema_versions_use_distinct_reachability_roots() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_receipts_with_v1_command(repo.path(), &valid_ls_command_v1());
        fs::write(
            repo.path()
                .join("formalization/lean/CrouzeixLoristSchwenninger.lean"),
            "-- v1 does not depend on this aggregate root\n",
        )
        .unwrap();
        refresh_ls_provider_binding(repo.path());

        verify(repo.path()).unwrap();

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        fs::write(
            repo.path().join("formalization/lean/Crouzeix.lean"),
            "-- v2 does not depend on the broad aggregate root\n",
        )
        .unwrap();
        refresh_local_build_source_bindings(repo.path());

        verify(repo.path()).unwrap();
    }

    #[test]
    fn python_parity_ls_receipts_require_aggregate_reachability() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        fs::write(
            repo.path()
                .join("formalization/lean/CrouzeixLoristSchwenninger.lean"),
            "-- import Crouzeix.LoristSchwenninger.Consequences\n",
        )
        .unwrap();
        refresh_ls_provider_binding(repo.path());

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error.message.contains(
                "LS committed build_target is not reachable from CrouzeixLoristSchwenninger"
            ),
            "{}",
            error.message
        );
    }

    #[test]
    fn historical_v1_ls_commands_remain_strict() {
        let cases: [JsonMutation; 3] = [
            ("hybrid v2 field", |command: &mut Value| {
                command["elan_toolchain"] = json!(LS_ELAN_TOOLCHAIN);
            }),
            ("unknown field", |command: &mut Value| {
                command["unknown"] = json!("field");
            }),
            ("missing field", |command: &mut Value| {
                command.as_object_mut().unwrap().remove("elan_home");
            }),
        ];
        for (case, mutate) in cases {
            let repo = fixture();
            write_local_formalization_manifest(repo.path());
            let mut command = valid_ls_command_v1();
            mutate(&mut command);
            replace_ls_receipts_with_v1_command(repo.path(), &command);

            let error = verify(repo.path()).unwrap_err();
            assert!(
                error.message.contains("invalid LS command JSON"),
                "{case}: {}",
                error.message
            );
        }
    }

    #[test]
    fn historical_v1_ls_command_matches_python_semantics() {
        for (case, field, value) in [
            ("Elan home", "elan_home", json!("/tmp/untrusted")),
            ("empty cache policy", "cache_policy", json!("")),
            (
                "oversized cache policy",
                "cache_policy",
                json!("x".repeat(4_097)),
            ),
            ("NUL cache policy", "cache_policy", json!("cached\0policy")),
        ] {
            let repo = fixture();
            write_local_formalization_manifest(repo.path());
            let mut command = valid_ls_command_v1();
            command[field] = value;
            replace_ls_receipts_with_v1_command(repo.path(), &command);

            let error = verify(repo.path()).unwrap_err();
            assert!(
                error
                    .message
                    .contains("LS command is not the successful aggregate Crouzeix build"),
                "{case}: {}",
                error.message
            );
        }
    }

    #[test]
    fn historical_v1_ls_command_accepts_whitespace_cache_policy() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let mut command = valid_ls_command_v1();
        command["cache_policy"] = json!("   	");
        replace_ls_receipts_with_v1_command(repo.path(), &command);

        verify(repo.path()).unwrap();
    }

    #[test]
    fn rejects_v2_ls_command_hybrids_unknown_fields_and_missing_fields() {
        let cases: [JsonMutation; 3] = [
            ("hybrid v1 elan_home", |command: &mut Value| {
                command["elan_home"] = json!(LS_ELAN_HOME);
            }),
            ("unknown top-level field", |command: &mut Value| {
                command["unknown"] = json!("field");
            }),
            ("unknown schema", |command: &mut Value| {
                command["schema_version"] = json!("crouzeix-ls-lean-command/v3");
            }),
        ];
        for (case, mutate) in cases {
            let message = rejected_v2_command_message(mutate);
            assert!(
                message.contains("invalid LS command JSON"),
                "{case}: {message}"
            );
        }

        for field in [
            "schema_version",
            "argv",
            "cwd",
            "timeout_seconds",
            "output_cap_bytes",
            "env",
            "exit_code",
            "cache_policy",
            "wrapper_sha256",
            "lake_manifest_sha256",
            "dependency_cache_metadata_sha256",
            "required_mathlib_artifacts_sha256",
            "local_source_closure_sha256",
            "elan_toolchain",
        ] {
            let message = rejected_v2_command_message(|command| {
                command.as_object_mut().unwrap().remove(field);
            });
            assert!(
                message.contains("invalid LS command JSON"),
                "missing {field}: {message}"
            );
        }

        let message = rejected_v2_command_message(|command| {
            command["env"]["HOME"] = json!("/tmp/untrusted");
        });
        assert!(message.contains("invalid LS command JSON"), "{message}");

        let message = rejected_v2_command_message(|command| {
            command["env"].as_object_mut().unwrap().remove("PATH");
        });
        assert!(message.contains("invalid LS command JSON"), "{message}");
    }

    #[test]
    fn rejects_v2_ls_command_policy_and_type_mutations() {
        let cases: [JsonMutation; 10] = [
            ("argv", |command: &mut Value| {
                command["argv"] = json!(["scripts/check_lean_library.sh", "Crouzeix"]);
            }),
            ("cwd", |command: &mut Value| {
                command["cwd"] = json!("formalization/lean");
            }),
            ("timeout", |command: &mut Value| {
                command["timeout_seconds"] = json!(3_599);
            }),
            ("output cap", |command: &mut Value| {
                command["output_cap_bytes"] = json!(1_048_575);
            }),
            ("exit code", |command: &mut Value| {
                command["exit_code"] = json!(1);
            }),
            ("cache policy", |command: &mut Value| {
                command["cache_policy"] = json!("cached");
            }),
            ("top-level toolchain", |command: &mut Value| {
                command["elan_toolchain"] = json!("leanprover/lean4:v4.31.0");
            }),
            ("environment Elan home", |command: &mut Value| {
                command["env"]["ELAN_HOME"] = json!("/tmp/other-elan");
            }),
            ("environment toolchain", |command: &mut Value| {
                command["env"]["ELAN_TOOLCHAIN"] = json!("leanprover/lean4:v4.31.0");
            }),
            ("environment path", |command: &mut Value| {
                command["env"]["PATH"] = json!("/usr/bin:/bin");
            }),
        ];
        for (case, mutate) in cases {
            let message = rejected_v2_command_message(mutate);
            assert!(
                message.contains("LS command is not the successful aggregate Crouzeix build"),
                "{case}: {message}"
            );
        }

        let message = rejected_v2_command_message(|command| {
            command["timeout_seconds"] = json!("3600");
        });
        assert!(message.contains("invalid LS command JSON"), "{message}");
    }

    #[test]
    fn rejects_noncanonical_v2_ls_command_digests() {
        for field in [
            "wrapper_sha256",
            "lake_manifest_sha256",
            "dependency_cache_metadata_sha256",
            "required_mathlib_artifacts_sha256",
            "local_source_closure_sha256",
        ] {
            let message = rejected_v2_command_message(|command| {
                command[field] = json!("A".repeat(64));
            });
            assert!(
                message.contains("LS command is not the successful aggregate Crouzeix build"),
                "{field}: {message}"
            );
        }

        let message = rejected_v2_command_message(|command| {
            command["dependency_cache_metadata_sha256"] = json!("b".repeat(63));
        });
        assert!(
            message.contains("LS command is not the successful aggregate Crouzeix build"),
            "{message}"
        );
    }

    #[test]
    fn v2_ls_command_treats_wrapper_as_historical_metadata_but_binds_lake_manifest_bytes() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let mut command = valid_ls_command_v2(repo.path());
        command["wrapper_sha256"] = json!("f".repeat(64));
        replace_ls_command(repo.path(), &command);

        verify(repo.path()).unwrap();

        let message = rejected_v2_command_message(|command| {
            command["lake_manifest_sha256"] = json!("f".repeat(64));
        });
        assert!(message.contains("digest mismatch"), "{message}");
    }

    #[test]
    fn local_build_command_still_binds_current_wrapper_bytes() {
        let message = rejected_local_command_message(|command| {
            command["wrapper_sha256"] = json!("f".repeat(64));
        });
        assert!(
            message.contains("aggregate build wrapper digest mismatch"),
            "{message}"
        );
    }

    #[test]
    fn python_parity_v2_ls_command_binds_local_closure_and_mathlib_bytes() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));
        let source_path = repo
            .path()
            .join("formalization/lean/Crouzeix/LoristSchwenninger/Dilation.lean");
        fs::write(
            &source_path,
            fs::read_to_string(&source_path).unwrap() + "-- changed after receipt\n",
        )
        .unwrap();
        refresh_ls_provider_binding(repo.path());
        let command_path = repo.path().join(LOCAL_BUILD_COMMAND);
        let mut local_command: Value =
            serde_json::from_str(&fs::read_to_string(&command_path).unwrap()).unwrap();
        local_command["active_source_closure_sha256"] =
            json!(test_active_source_closure_digest(repo.path()));
        replace_local_build_command(repo.path(), &local_command);

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error.message.contains("LS proof module digest mismatch")
                || error
                    .message
                    .contains("LS command is not the successful aggregate Crouzeix build"),
            "{}",
            error.message
        );

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));
        fs::write(
            repo.path().join(
                "formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean/Mathlib/Fixture.olean",
            ),
            b"changed Mathlib object bytes\n",
        )
        .unwrap();
        let command_path = repo.path().join(LOCAL_BUILD_COMMAND);
        let mut local_command: Value =
            serde_json::from_str(&fs::read_to_string(&command_path).unwrap()).unwrap();
        local_command["required_mathlib_artifacts_sha256"] =
            json!(test_active_required_mathlib_digest(repo.path()));
        replace_local_build_command(repo.path(), &local_command);

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error
                .message
                .contains("LS command is not the successful aggregate Crouzeix build"),
            "{}",
            error.message
        );
    }

    #[test]
    fn python_publisher_v2_digest_includes_aggregate_root_and_transitive_imports() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));
        let aggregate = repo
            .path()
            .join("formalization/lean/CrouzeixLoristSchwenninger.lean");
        fs::write(
            &aggregate,
            fs::read_to_string(&aggregate).unwrap() + "-- aggregate-only mutation\n",
        )
        .unwrap();
        refresh_ls_provider_binding(repo.path());
        refresh_local_build_source_bindings(repo.path());

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error
                .message
                .contains("LS command is not the successful aggregate Crouzeix build"),
            "{}",
            error.message
        );

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));
        write_lean_module(
            repo.path(),
            "Fixture.Transitive",
            "def Fixture.transitive : True := True\n",
        );
        let aggregate = repo
            .path()
            .join("formalization/lean/CrouzeixLoristSchwenninger.lean");
        fs::write(
            &aggregate,
            fs::read_to_string(&aggregate).unwrap() + "import Fixture.Transitive\n",
        )
        .unwrap();
        refresh_ls_provider_binding(repo.path());

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error
                .message
                .contains("LS command is not the successful aggregate Crouzeix build"),
            "{}",
            error.message
        );
    }

    #[test]
    fn v2_provider_independence_covers_aggregate_execution_closure() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));
        write_lean_module(
            repo.path(),
            "Crouzeix.Jin.ForbiddenAggregateProvider",
            "def Crouzeix.Jin.forbiddenAggregateProvider : True := True\n",
        );
        let aggregate = repo
            .path()
            .join("formalization/lean/CrouzeixLoristSchwenninger.lean");
        fs::write(
            &aggregate,
            fs::read_to_string(&aggregate).unwrap()
                + "import Crouzeix.Jin.ForbiddenAggregateProvider\n",
        )
        .unwrap();
        refresh_local_build_source_bindings(repo.path());
        replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error
                .message
                .contains("forbidden provider module Crouzeix.Jin.ForbiddenAggregateProvider"),
            "{}",
            error.message
        );
    }

    #[test]
    fn v2_ls_source_closure_rejects_missing_transitive_managed_local_imports() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));
        write_lean_module(
            repo.path(),
            "Fixture.MissingManagedTransitive",
            "import CrouzeixConjecture.MissingTransitive\n\
\n\
             def Fixture.missingManagedTransitive : True := True\n",
        );
        let aggregate = repo
            .path()
            .join("formalization/lean/CrouzeixLoristSchwenninger.lean");
        fs::write(
            &aggregate,
            fs::read_to_string(&aggregate).unwrap() + "import Fixture.MissingManagedTransitive\n",
        )
        .unwrap();
        refresh_local_build_source_bindings(repo.path());
        replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error
                .message
                .contains("missing managed local import: CrouzeixConjecture.MissingTransitive"),
            "{}",
            error.message
        );
    }

    #[test]
    fn v2_provider_independence_rejects_transitive_harp_support_imports() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));
        write_lean_module(
            repo.path(),
            "Fixture.TransitiveProvider",
            "import Crouzeix.Harp.Support\n\ndef Fixture.transitiveProvider : True := True\n",
        );
        let aggregate = repo
            .path()
            .join("formalization/lean/CrouzeixLoristSchwenninger.lean");
        fs::write(
            &aggregate,
            fs::read_to_string(&aggregate).unwrap() + "import Fixture.TransitiveProvider\n",
        )
        .unwrap();
        refresh_local_build_source_bindings(repo.path());
        replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error
                .message
                .contains("forbidden provider module Crouzeix.Harp.Support"),
            "{}",
            error.message
        );
    }

    #[cfg(unix)]
    #[test]
    fn python_parity_v2_ls_closure_rejects_unrelated_symlinks() {
        use std::os::unix::fs::symlink;

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));
        let target = repo.path().join("unrelated.lean");
        fs::write(&target, "def Unrelated.value : True := True\n").unwrap();
        symlink(
            &target,
            repo.path().join("formalization/lean/Unrelated.lean"),
        )
        .unwrap();

        let error = verify(repo.path()).unwrap_err();
        assert!(error.message.contains("symlink"), "{}", error.message);
    }

    #[test]
    fn ls_receipt_binds_all_v2_member_bytes() {
        for member in [
            "task.json",
            "source-slice.json",
            "result.json",
            "build/command.json",
            "build/stdout.log",
            "build/stderr.log",
            "build/axioms.txt",
        ] {
            let repo = fixture();
            write_local_formalization_manifest(repo.path());
            replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));
            let member_path = terminal_ls_attempt(repo.path()).join(member);
            let mut bytes = fs::read(&member_path).unwrap();
            bytes.push(b' ');
            fs::write(&member_path, bytes).unwrap();

            let error = verify(repo.path()).unwrap_err();
            assert!(
                error.message.contains("LS receipt member digest mismatch"),
                "{member}: {}",
                error.message
            );
        }
    }

    #[test]
    fn rust_python_receipt_parity_rejects_missing_graph_bound_nonterminal_attempt() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        fs::remove_dir_all(ls_attempt(repo.path(), "ls-power-recurrence")).unwrap();

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error.message.contains("LS committed receipt")
                || error.message.contains("LS proof receipt"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rust_python_receipt_parity_rejects_corrupt_graph_bound_nonterminal_attempt() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let node_id = "ls-power-recurrence";
        let task_path = ls_attempt(repo.path(), node_id).join("task.json");
        let mut task: Value =
            serde_json::from_str(&fs::read_to_string(&task_path).unwrap()).unwrap();
        task["expected_lean_declaration"] = json!("CrouzeixConjecture.Wrong");
        let mut bytes = serde_json::to_vec_pretty(&task).unwrap();
        bytes.push(b'\n');
        replace_ls_member(repo.path(), node_id, "task.json", "task_sha256", &bytes);

        let error = verify(repo.path()).unwrap_err();
        assert!(error.message.contains("LS task"), "{}", error.message);
    }

    #[test]
    fn rust_python_receipt_parity_rejects_malformed_and_oversize_ls_output() {
        for (case, member, digest_field, bytes) in [
            (
                "malformed stdout",
                "build/stdout.log",
                "stdout_sha256",
                b"[lean] outcome=passed with warnings\n".to_vec(),
            ),
            (
                "oversize stdout",
                "build/stdout.log",
                "stdout_sha256",
                vec![b'x'; 1_048_577],
            ),
            (
                "oversize stderr",
                "build/stderr.log",
                "stderr_sha256",
                vec![b'x'; 1_048_577],
            ),
        ] {
            let repo = fixture();
            write_local_formalization_manifest(repo.path());
            replace_ls_member(
                repo.path(),
                "ls-terminal-crouzeix",
                member,
                digest_field,
                &bytes,
            );

            let error = verify(repo.path()).unwrap_err();
            let lower = error.message.to_ascii_lowercase();
            assert!(
                (lower.contains("ls") && lower.contains("aggregate stdout"))
                    || lower.contains("byte limit"),
                "{case}: {}",
                error.message
            );
        }
    }

    #[test]
    fn rust_python_receipt_parity_rejects_malformed_or_wrong_ls_axiom_audit() {
        for (case, audit) in [
            ("malformed", "not a Lean axiom audit\n"),
            (
                "wrong declaration",
                "'CrouzeixConjecture.Wrong' depends on axioms: [Classical.choice, Quot.sound, propext]\n",
            ),
        ] {
            let repo = fixture();
            write_local_formalization_manifest(repo.path());
            replace_ls_member(
                repo.path(),
                "ls-terminal-crouzeix",
                "build/axioms.txt",
                "axiom_audit_sha256",
                audit.as_bytes(),
            );

            let error = verify(repo.path()).unwrap_err();
            assert!(
                error.message.contains("LS axiom audit"),
                "{case}: {}",
                error.message
            );
        }
    }

    #[test]
    fn rust_python_receipt_parity_rejects_ls_axiom_audit_receipt_drift() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let node_id = "ls-equation-one-terminal-bound";
        let audit = format!(
            "'{}' depends on axioms: [Classical.choice, propext]\n",
            LS_NODE_SPECS[0].lean_name
        );
        replace_ls_member(
            repo.path(),
            node_id,
            "build/axioms.txt",
            "axiom_audit_sha256",
            audit.as_bytes(),
        );

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error.message.contains("exact ordered observed axioms"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rust_python_receipt_parity_rejects_noncanonical_ls_axiom_order() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let node_id = "ls-equation-one-terminal-bound";
        let receipt_path = ls_attempt(repo.path(), node_id).join("receipt.json");
        let mut receipt: Value =
            serde_json::from_str(&fs::read_to_string(&receipt_path).unwrap()).unwrap();
        receipt["observed_axioms"] = json!(["propext", "Quot.sound", "Classical.choice"]);
        write_json(&receipt_path, &receipt);
        refresh_ls_receipt_binding(repo.path(), node_id);

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error.message.contains("observed axioms")
                || error.message.contains("ordered axiom policy"),
            "{}",
            error.message
        );
    }

    #[test]
    fn rust_python_receipt_parity_requires_exact_local_aggregate_markers() {
        for (case, stdout) in [
            (
                "duplicate marker",
                concat!(
                    "[lean] target=Crouzeix\n",
                    "[lean] target=Crouzeix\n",
                    "[lean] root=formalization/lean\n",
                    "[lean] outcome=passed\n",
                ),
            ),
            (
                "contradictory outcome",
                concat!(
                    "[lean] target=Crouzeix\n",
                    "[lean] root=formalization/lean\n",
                    "[lean] outcome=passed\n",
                    "[lean] outcome=blocked\n",
                ),
            ),
        ] {
            let repo = fixture();
            write_local_formalization_manifest(repo.path());
            replace_local_build_stdout(repo.path(), stdout.as_bytes());

            let error = verify(repo.path()).unwrap_err();
            assert!(
                error.message.contains("successful Crouzeix build report"),
                "{case}: {}",
                error.message
            );
        }
    }

    #[test]
    fn local_manifest_binds_terminal_ls_receipt_bytes() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let receipt_path = terminal_ls_attempt(repo.path()).join("receipt.json");
        fs::write(
            &receipt_path,
            fs::read_to_string(&receipt_path).unwrap() + " ",
        )
        .unwrap();

        let error = verify(repo.path()).unwrap_err();
        assert!(
            error.message.contains("LS proof receipt digest mismatch")
                || error
                    .message
                    .contains("exactly one LS committed receipt_sha256"),
            "{}",
            error.message
        );
    }

    #[test]
    fn local_and_ls_commands_are_independently_evidenced() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_command(repo.path(), &valid_ls_command_v2(repo.path()));
        let manifest =
            fs::read_to_string(repo.path().join(ROOT).join(LOCAL_FORMALIZATION_MANIFEST)).unwrap();
        let row = manifest
            .lines()
            .nth(4)
            .unwrap()
            .split('\t')
            .collect::<Vec<_>>();
        let local_command_digest = row[local_field_index("build_command_sha256")];
        let receipt: Value = serde_json::from_str(
            &fs::read_to_string(terminal_ls_attempt(repo.path()).join("receipt.json")).unwrap(),
        )
        .unwrap();

        assert_ne!(local_command_digest, receipt["command_sha256"]);
        verify(repo.path()).unwrap();
    }

    #[test]
    fn ls_result_and_receipt_reasons_are_independently_hash_bound() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        replace_ls_result_reason(
            repo.path(),
            "Detailed execution account retained by the result artifact.",
        );

        verify(repo.path()).unwrap();
    }

    #[test]
    fn accepts_python_compatible_ls_attempt_names() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let old_attempt = terminal_ls_attempt(repo.path());
        let new_attempt = old_attempt.parent().unwrap().join("attempt-1000");
        fs::rename(old_attempt, &new_attempt).unwrap();

        verify(repo.path()).unwrap();
    }

    #[test]
    fn provider_scan_ignores_raw_strings_but_checks_interpolations() {
        let manifest = Path::new("fixture.tsv");
        let raw = "def Fixture.raw := r###\"sorry set_option /- -- \\\"\"###\n";
        let active = mask_lean_source(raw, "Fixture.Root", manifest, 1).unwrap();
        let mut findings = Vec::new();
        scan_provider_source(raw, &active, "Fixture.Root", manifest, 1, &mut findings).unwrap();
        assert!(findings.is_empty());

        let interpolated =
            "def Fixture.interpolated : String := s!\"literal {(by exact sorry : String)}\"\n";
        let active = mask_lean_source(interpolated, "Fixture.Root", manifest, 1).unwrap();
        let error = scan_provider_source(
            interpolated,
            &active,
            "Fixture.Root",
            manifest,
            1,
            &mut Vec::new(),
        )
        .unwrap_err();
        assert!(error.message.contains("danger token sorry"));
    }

    #[test]
    fn provider_scan_matches_lean_token_boundaries() {
        let manifest = Path::new("fixture.tsv");
        let inert = "def αsorry := 1\ndef axiom! := 2\ndef «unsafe» := 3\n";
        let active = mask_lean_source(inert, "Fixture.Root", manifest, 1).unwrap();
        scan_provider_source(inert, &active, "Fixture.Root", manifest, 1, &mut Vec::new()).unwrap();

        let axiom = "  axiom exposed : Prop\n";
        let error =
            scan_provider_source(axiom, axiom, "Fixture.Root", manifest, 1, &mut Vec::new())
                .unwrap_err();
        assert!(error.message.contains("danger token axiom"));
    }

    #[test]
    fn provider_scan_allows_identifier_apostrophe_at_end_of_file() {
        let manifest = Path::new("fixture.tsv");
        let source = "replaceMainGoal mvarIds'";

        let active = mask_lean_source(source, "Fixture.Root", manifest, 1).unwrap();

        assert_eq!(active, source);
    }

    #[test]
    fn python_parity_provider_policy_uses_the_local_bundle_forbidden_sets() {
        let legacy = [
            "Crouzeix.Jin.Terminal",
            "CrouzeixConjecture.FinalTheorems",
            "CrouzeixConjecture.HilbertSpace",
            "CrouzeixConjecture.HilbertSpectralSet",
            "CrouzeixConjecture.RadialOuterReduction",
        ];
        for (route, other_roots) in [
            ("harp", &["CrouzeixLoristSchwenninger"][..]),
            (
                "lorist-schwenninger",
                &[
                    "Crouzeix.Harp.Consequences",
                    "Crouzeix.Harp.FiniteAtomicL2Dilation",
                    "Crouzeix.Harp.FiniteHorizonPerturbation",
                    "Crouzeix.Harp.MainTheorem",
                    "CrouzeixJin",
                ][..],
            ),
        ] {
            let (_, actual, _) = provider_policy(route).unwrap();
            let mut expected = legacy
                .into_iter()
                .chain(other_roots.iter().copied())
                .map(str::to_owned)
                .collect::<Vec<_>>();
            if route == "lorist-schwenninger" {
                expected.extend(
                    [
                        "CrouzeixConjecture.CompletionDiagonalization",
                        "CrouzeixConjecture.CompletionStatement",
                        "CrouzeixConjecture.MainPerturbationReduction",
                        "CrouzeixConjecture.PositiveRealCompletion",
                    ]
                    .into_iter()
                    .map(str::to_owned),
                );
            }
            expected.sort();
            assert_eq!(actual, expected, "{route}");
        }
        let (_, jin_forbidden, jin_prefixes) = provider_policy("jin").unwrap();
        assert_eq!(
            jin_forbidden,
            ["Crouzeix", "CrouzeixHarp", "CrouzeixLoristSchwenninger"]
        );
        assert_eq!(
            jin_prefixes,
            ["Crouzeix.LoristSchwenninger", "Crouzeix.Harp"]
        );
    }

    #[test]
    fn python_parity_provider_imports_accept_whitespace_and_keyword_boundaries() {
        let manifest = Path::new("fixture.tsv");
        let source = concat!(
            "module\n",
            "prelude\n",
            "public   import Alpha.One\n",
            "public\timport Alpha.Tabbed\n",
            "import Alpha.Two\n",
            "import Beta.Three\n",
            "moduleName := 1\n",
            "import Hidden.Late\n",
        );
        let active = mask_lean_source(source, "Fixture.Root", manifest, 1).unwrap();
        assert_eq!(
            lean_header_imports(&active, "Fixture.Root", manifest, 1).unwrap(),
            ["Alpha.One", "Alpha.Tabbed", "Alpha.Two", "Beta.Three"]
        );
    }

    #[test]
    fn python_parity_provider_imports_stop_at_public_section() {
        let manifest = Path::new("fixture.tsv");
        let source = concat!(
            "module\n",
            "public import Mathlib.Algebra.Polynomial.AlgebraMap\n",
            "public section\n",
            "import Hidden.Late\n",
        );
        let active = mask_lean_source(source, "Fixture.Root", manifest, 1).unwrap();

        assert_eq!(
            lean_header_imports(&active, "Fixture.Root", manifest, 1).unwrap(),
            ["Mathlib.Algebra.Polynomial.AlgebraMap"]
        );
    }

    #[test]
    fn python_parity_provider_imports_accept_modifiers_and_body_terminators() {
        let manifest = Path::new("fixture.tsv");
        for (source, expected) in [
            (
                "module\nmeta import Qq\npublic meta import all Mathlib.Util.AtomM\npublic meta section\nimport Hidden.Late\n",
                vec!["Qq", "Mathlib.Util.AtomM"],
            ),
            (
                "module\npublic import Mathlib.X\nmeta def helper := 1\nimport Hidden.Late\n",
                vec!["Mathlib.X"],
            ),
            (
                "module\npublic import Mathlib.X\npublic noncomputable section Named\nimport Hidden.Late\n",
                vec!["Mathlib.X"],
            ),
        ] {
            let active = mask_lean_source(source, "Fixture.Root", manifest, 1).unwrap();
            assert_eq!(
                lean_header_imports(&active, "Fixture.Root", manifest, 1).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn python_parity_provider_imports_reject_invalid_modified_headers() {
        let manifest = Path::new("fixture.tsv");
        for source in [
            "public import Alpha.One\n",
            "module\nimport Alpha.One Beta.Two\n",
            "module\nmeta public import Alpha.One\n",
            "module\npublic meta importx Alpha.One\nimport Hidden.Late\n",
        ] {
            let active = mask_lean_source(source, "Fixture.Root", manifest, 1).unwrap();
            assert!(
                lean_header_imports(&active, "Fixture.Root", manifest, 1).is_err(),
                "accepted invalid header: {source:?}"
            );
        }
    }

    #[test]
    fn local_formalization_requires_exact_all_passed_ls_graph() {
        let mutations: [JsonMutation; 3] = [
            ("blocked node", |graph| {
                graph["nodes"][0]["status"] = json!("blocked");
                graph["nodes"][0]["blocked_reason"] = json!("not complete");
            }),
            ("node order", |graph| {
                graph["nodes"].as_array_mut().unwrap().swap(0, 1);
            }),
            ("node identity", |graph| {
                graph["nodes"][0]["statement_sha256"] = json!("f".repeat(64));
            }),
        ];
        for (case, mutate) in mutations {
            let repo = fixture();
            write_local_formalization_manifest(repo.path());
            mutate_ls_graph(repo.path(), mutate);

            let error = verify(repo.path()).unwrap_err();
            assert!(
                error
                    .message
                    .contains("exact canonical six-node all-passed LS source graph"),
                "{case}: {}",
                error.message
            );
        }
    }

    #[test]
    fn ls_terminal_spec_matches_the_canonical_dependency_edge() {
        let terminal = LS_NODE_SPECS
            .iter()
            .find(|spec| spec.node_id == "ls-terminal-crouzeix")
            .unwrap();

        assert_eq!(terminal.dependencies, &["ls-double-layer-realization"]);
    }

    #[test]
    fn python_parity_passed_ls_nodes_reject_present_null_reason_fields() {
        for field in ["blocked_reason", "failed_reason"] {
            let repo = fixture();
            write_local_formalization_manifest(repo.path());
            mutate_ls_graph(repo.path(), |graph| {
                graph["nodes"][0][field] = Value::Null;
            });

            let error = verify(repo.path()).unwrap_err();
            assert!(
                error
                    .message
                    .contains("exact canonical six-node all-passed LS source graph"),
                "{field}: {}",
                error.message
            );
        }
    }

    #[test]
    fn local_formalization_manifest_requires_all_six_claim_surfaces() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        mutate_local_formalization_manifest(repo.path(), |_, rows| {
            rows.pop();
        });

        let error = verify(repo.path()).unwrap_err();
        assert_eq!(error.code(), LOCAL_FORMALIZATION_CODE);
        assert!(error.message.contains("exactly the six"));
    }

    #[test]
    fn local_formalization_manifest_rejects_unknown_fields_and_duplicate_ids() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        mutate_local_formalization_manifest(repo.path(), |header, rows| {
            header.push("unknown".to_owned());
            rows[0].push("value".to_owned());
        });
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let id_index = local_field_index("formalization_id");
        mutate_local_formalization_manifest(repo.path(), |_, rows| {
            rows[1][id_index] = rows[0][id_index].clone();
        });
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(error.message.contains("formalization_id"));
    }

    #[test]
    fn local_formalization_manifest_rejects_unsafe_status_and_axioms() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        set_local_field(repo.path(), "harp-main-theorem", "status", "blocked");
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(error.message.contains("status"));

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        set_local_field(
            repo.path(),
            "harp-main-theorem",
            "allowed_axioms",
            "Classical.choice,Quot.sound",
        );
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(error.message.contains("allowed_axioms"));

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        set_local_field(
            repo.path(),
            "harp-main-theorem",
            "observed_axioms",
            "Classical.choice,propext,unsafeAxiom",
        );
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(error.message.contains("observed_axioms"));
    }

    #[test]
    fn local_formalization_manifest_rejects_unknown_or_mismatched_routes() {
        for (field, value) in [
            ("route_id", "jin"),
            (
                "module_path",
                "formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean",
            ),
            (
                "declaration_name",
                "CrouzeixConjecture.LoristSchwenninger.mainTheorem",
            ),
        ] {
            let repo = fixture();
            write_local_formalization_manifest(repo.path());
            set_local_field(repo.path(), "harp-main-theorem", field, value);
            let error = verify(repo.path()).unwrap_err();
            assert_eq!(
                error.code(),
                "sources.crouzeix.local_formalization_manifest",
                "field={field}"
            );
        }

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        set_local_field(repo.path(), "jin-main-theorem", "route_id", "harp");
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        set_local_field(
            repo.path(),
            "jin-main-theorem",
            "source_node_id",
            "ls-terminal-crouzeix",
        );
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
    }

    #[test]
    fn local_formalization_manifest_binds_artifacts_to_canonical_bundle_paths() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let bundle = local_formalization_bundle(repo.path());
        let closed_path = bundle.join("axioms/harp-closed-numerical-range.txt");
        let main_path = bundle.join("axioms/harp-main-theorem.txt");
        let closed_audit = fs::read(&closed_path).unwrap();
        let main_audit = fs::read(&main_path).unwrap();
        fs::write(&closed_path, main_audit).unwrap();
        fs::write(&main_path, closed_audit).unwrap();

        set_local_field(
            repo.path(),
            "harp-closed-numerical-range",
            "axiom_audit_path",
            "evidence/crouzeix_conjecture/local_formalization/axioms/harp-main-theorem.txt",
        );
        set_local_field(
            repo.path(),
            "harp-closed-numerical-range",
            "axiom_audit_sha256",
            &sha256_file(&main_path),
        );
        set_local_field(
            repo.path(),
            "harp-main-theorem",
            "axiom_audit_path",
            "evidence/crouzeix_conjecture/local_formalization/axioms/harp-closed-numerical-range.txt",
        );
        set_local_field(
            repo.path(),
            "harp-main-theorem",
            "axiom_audit_sha256",
            &sha256_file(&closed_path),
        );

        let error = verify(repo.path()).unwrap_err();
        assert_eq!(error.code(), LOCAL_FORMALIZATION_CODE);
        assert!(
            error.message.contains("canonical bundle path"),
            "{}",
            error.message
        );
    }

    #[test]
    fn local_formalization_manifest_rejects_traversal_and_artifact_aliases() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let command_path = repo
            .path()
            .join(ROOT)
            .join("local_formalization/build/command.json");
        let mut command: Value =
            serde_json::from_str(&fs::read_to_string(&command_path).unwrap()).unwrap();
        command["stdout_path"] =
            json!("evidence/crouzeix_conjecture/local_formalization/../outside.log");
        write_json(&command_path, &command);
        set_local_field_for_all(
            repo.path(),
            "build_command_sha256",
            &sha256_file(&command_path),
        );
        set_local_field_for_all(
            repo.path(),
            "build_stdout_path",
            "evidence/crouzeix_conjecture/local_formalization/../outside.log",
        );
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(
            error.message.contains("repository-relative normal"),
            "{}",
            error.message
        );

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let command_path = "evidence/crouzeix_conjecture/local_formalization/build/command.json";
        let command_digest = sha256_file(&repo.path().join(command_path));
        set_local_field(
            repo.path(),
            "harp-main-theorem",
            "provider_independence_path",
            command_path,
        );
        set_local_field(
            repo.path(),
            "harp-main-theorem",
            "provider_independence_sha256",
            &command_digest,
        );
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(error.message.contains("artifact path"));

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let evidence = repo.path().join(ROOT).join("local_formalization");
        fs::write(
            evidence.join("providers/harp.json"),
            fs::read(evidence.join("build/command.json")).unwrap(),
        )
        .unwrap();
        let command_digest = sha256_file(&evidence.join("build/command.json"));
        let route_index = local_field_index("route_id");
        let digest_index = local_field_index("provider_independence_sha256");
        mutate_local_formalization_manifest(repo.path(), |_, rows| {
            for row in rows.iter_mut().filter(|row| row[route_index] == "harp") {
                row[digest_index] = command_digest.clone();
            }
        });
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(error.message.contains("artifact digest"));
    }

    #[test]
    fn local_formalization_manifest_binds_toolchain_and_raw_axiom_audit() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        set_local_field(
            repo.path(),
            "harp-main-theorem",
            "lean_toolchain",
            "leanprover/lean4:v4.31.0",
        );
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(error.message.contains("lean_toolchain"));

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let audit = repo
            .path()
            .join(ROOT)
            .join("local_formalization/axioms/harp-main-theorem.txt");
        fs::write(
            &audit,
            "'CrouzeixConjecture.Harp.otherTheorem' depends on axioms: [Classical.choice, Quot.sound, propext]\n",
        )
        .unwrap();
        set_local_field(
            repo.path(),
            "harp-main-theorem",
            "axiom_audit_sha256",
            &sha256_file(&audit),
        );
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(error.message.contains("axiom audit declaration"));
    }

    #[test]
    fn local_formalization_manifest_rejects_missing_and_hash_mismatched_artifacts() {
        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        fs::remove_file(
            repo.path()
                .join(ROOT)
                .join("local_formalization/providers/harp.json"),
        )
        .unwrap();
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        fs::write(
            repo.path()
                .join("formalization/lean/Crouzeix/Harp/Consequences.lean"),
            "theorem changed : True := by trivial\n",
        )
        .unwrap();
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(
            error.message.contains("digest mismatch"),
            "{}",
            error.message
        );

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let provider = repo
            .path()
            .join(ROOT)
            .join("local_formalization/providers/jin.json");
        let mut bytes = fs::read(&provider).unwrap();
        bytes.push(b'\n');
        fs::write(&provider, bytes).unwrap();
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(
            error.message.contains("digest mismatch"),
            "{}",
            error.message
        );
    }

    #[cfg(unix)]
    #[test]
    fn local_formalization_manifest_rejects_symlinked_artifacts_and_ancestors() {
        use std::os::unix::fs::symlink;

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let evidence = repo.path().join(ROOT).join("local_formalization");
        let outside = repo.path().join("outside.json");
        fs::write(&outside, "{\"status\":\"passed\"}\n").unwrap();
        fs::remove_file(evidence.join("providers/harp.json")).unwrap();
        symlink(&outside, evidence.join("providers/harp.json")).unwrap();
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(error.message.contains("symlink"));

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let evidence = repo.path().join(ROOT).join("local_formalization");
        let original = evidence.join("providers/jin.json");
        let alias = evidence.join("providers/harp.json");
        fs::remove_file(&alias).unwrap();
        fs::hard_link(&original, &alias).unwrap();
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(error.message.contains("hardlink"), "{}", error.message);

        let repo = fixture();
        write_local_formalization_manifest(repo.path());
        let module_root = repo.path().join("formalization/lean/Crouzeix/Harp");
        let outside_root = repo.path().join("outside-modules");
        fs::rename(&module_root, &outside_root).unwrap();
        symlink(&outside_root, &module_root).unwrap();
        let error = verify(repo.path()).unwrap_err();
        assert_eq!(
            error.code(),
            "sources.crouzeix.local_formalization_manifest"
        );
        assert!(error.message.contains("symlink"));
    }
}

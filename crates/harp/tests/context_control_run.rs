#[path = "support/run_context_control.rs"]
mod run_context_control;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Output;

use assert_cmd::Command;
use harp::context_control::canonical::{
    json_bytes, json_bytes_with_newline, sha256_hex, sha256_id,
};
use harp::context_control::provider::{execute, BoundProviderInvocation, ProviderExecution};
use harp::context_control::release::{
    compile_baseline_release, ReleaseIdentityV2, ReleaseManifest,
};
use harp::context_control::repository_state::RepositorySnapshot;
use harp::context_control::run::{run, RunReporter, RunRequest, RunResult};
use harp::context_control::state::{ReplacePolicy, StateRoot};
use harp::AppError;
use serde::Serialize;
use serde_json::{json, Value};

use run_context_control::{RunProviderFixtures, RunRepository};

const TASK: &str = "fix the failing test";

#[test]
fn provider_process_api_consumes_bound_invocation_authority() {
    let execute: fn(BoundProviderInvocation) -> Result<ProviderExecution, AppError> = execute;
    let _ = execute;
}

#[test]
fn harp_run_public_api_has_the_approved_shape() {
    let run: fn(RunRequest, &mut dyn RunReporter) -> Result<RunResult, AppError> = run;
    let _ = run;
}

#[test]
fn harp_run_trae_and_codex_share_context_but_publish_distinct_evidence() {
    let repository = RunRepository::new();
    let providers = RunProviderFixtures::supported();

    let trae = run_harp(&repository, &providers, "trae", "ci_repair", TASK, 0, false);
    let codex = run_harp(&repository, &providers, "codex", "ci_repair", TASK, 0, true);

    assert_eq!(
        trae.stdout,
        b"{\"provider\":\"trae\",\"event\":\"complete\"}\n"
    );
    assert_eq!(
        codex.stdout,
        b"{\"provider\":\"codex\",\"event\":\"complete\"}\n"
    );
    assert_lifecycle(&trae.stderr, "trae");
    assert_lifecycle(&codex.stderr, "codex");
    assert_eq!(providers.launches(), ["trae", "codex"]);
    assert_eq!(providers.prompt("trae"), providers.prompt("codex"));

    let episodes = episode_directories(&repository);
    assert_eq!(episodes.len(), 2);
    let mut manifests = episodes
        .iter()
        .map(|episode| read_json(&episode.join("manifest.json")))
        .collect::<Vec<_>>();
    manifests.sort_by_key(|manifest| manifest["provider"].as_str().unwrap().to_owned());
    let codex_manifest = &manifests[0];
    let trae_manifest = &manifests[1];
    assert_eq!(codex_manifest["provider"], "codex");
    assert_eq!(trae_manifest["provider"], "trae");
    assert_eq!(codex_manifest["release_id"], trae_manifest["release_id"]);
    assert_eq!(
        codex_manifest["context_bundle_sha256"],
        trae_manifest["context_bundle_sha256"]
    );
    assert_eq!(codex_manifest["workflow"], "ci_repair");
    assert_eq!(trae_manifest["workflow"], "ci_repair");
    assert_ne!(
        codex_manifest["provider_capabilities_sha256"],
        trae_manifest["provider_capabilities_sha256"]
    );
    assert_ne!(
        codex_manifest["invocation"]["command_sha256"],
        trae_manifest["invocation"]["command_sha256"]
    );

    for episode in &episodes {
        let manifest = read_json(&episode.join("manifest.json"));
        let provider = manifest["provider"].as_str().unwrap();
        assert!(episode.join("completion.json").is_file());
        assert!(episode.join("context.json").is_file());
        assert!(episode
            .join("raw")
            .join(provider)
            .join("raw_members.tsv")
            .is_file());
    }
}

#[test]
fn harp_run_nonzero_exit_completes_episode_and_preserves_stream_contract() {
    let repository = RunRepository::new();
    let providers = RunProviderFixtures::supported();

    let output = run_harp(
        &repository,
        &providers,
        "trae",
        "general_coding",
        "implement the requested change",
        7,
        true,
    );

    assert_eq!(output.status.code(), Some(7));
    assert_eq!(
        output.stdout,
        b"{\"provider\":\"trae\",\"event\":\"complete\"}\n"
    );
    assert_lifecycle(&output.stderr, "trae");
    let episodes = episode_directories(&repository);
    assert_eq!(episodes.len(), 1);
    let completion = read_json(&episodes[0].join("completion.json"));
    assert_eq!(completion["provider_exit_code"], 7);
    assert_eq!(completion["terminated_by_signal"], Value::Null);
    assert_eq!(completion["capture_complete"], true);
}

#[test]
fn harp_run_preflight_failures_publish_no_episode_or_launch() {
    struct Case {
        name: &'static str,
        providers: RunProviderFixtures,
        policy: Option<Value>,
        expected_code: &'static str,
    }

    let cases = [
        Case {
            name: "missing capability",
            providers: RunProviderFixtures::missing_required_capability("trae"),
            policy: None,
            expected_code: "provider.missing_capability",
        },
        Case {
            name: "disabled policy",
            providers: RunProviderFixtures::supported(),
            policy: Some(policy(false, None, None)),
            expected_code: "repository.policy_disabled",
        },
        Case {
            name: "malformed policy",
            providers: RunProviderFixtures::supported(),
            policy: Some(json!({
                "schema_version": "harp-repository-policy/v1",
                "enabled": true,
                "required_verification_labels": [],
            })),
            expected_code: "repository.policy",
        },
        Case {
            name: "disallowed workflow",
            providers: RunProviderFixtures::supported(),
            policy: Some(policy(true, Some(&["code_review"]), None)),
            expected_code: "repository.workflow_not_allowed",
        },
    ];

    for case in cases {
        let repository = RunRepository::new();
        if let Some(policy) = case.policy {
            repository.write_policy(policy);
        }

        let output = run_harp(
            &repository,
            &case.providers,
            "trae",
            "ci_repair",
            TASK,
            0,
            true,
        );

        assert_eq!(output.status.code(), Some(1), "{}", case.name);
        assert!(output.stdout.is_empty(), "{}", case.name);
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(case.expected_code),
            "{}: {}",
            case.name,
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(case.providers.launches().is_empty(), "{}", case.name);
        let repository_state = repository.repository_state_directory();
        assert!(!repository_state.join("episodes").exists(), "{}", case.name);
        let failures = directory_entries(&repository_state.join("preflight_failures"));
        assert_eq!(failures.len(), 1, "{}", case.name);
        let failure = read_json(&failures[0]);
        assert_eq!(failure["error_code"], case.expected_code, "{}", case.name);
    }
}

#[test]
fn disabled_policy_runs_no_provider_probe_and_publishes_no_release() {
    let repository = RunRepository::new();
    repository.write_policy(policy(false, None, None));
    let providers = RunProviderFixtures::supported();

    let output = run_harp(&repository, &providers, "trae", "ci_repair", TASK, 0, true);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("repository.policy_disabled"));
    assert!(providers.invocations().is_empty());
    assert!(!repository.state().join("releases").exists());
    let repository_state = repository.repository_state_directory();
    assert!(!repository_state.join("episodes").exists());
    let failures = directory_entries(&repository_state.join("preflight_failures"));
    assert_eq!(failures.len(), 1);
    assert_eq!(
        read_json(&failures[0])["error_code"],
        "repository.policy_disabled"
    );
}

#[test]
fn harp_run_rejects_a_provider_incompatible_pinned_release_before_launch() {
    let repository = RunRepository::new();
    let providers = RunProviderFixtures::supported();
    let release_id = publish_codex_only_release(repository.state());
    let mut repository_policy = policy(true, None, None);
    repository_policy["pinned_release"] = json!(release_id);
    repository.write_policy(repository_policy);

    let output = run_harp(&repository, &providers, "trae", "ci_repair", TASK, 0, true);

    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("release.provider_compatibility"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(providers.launches().is_empty());
    let repository_state = repository.repository_state_directory();
    assert!(!repository_state.join("episodes").exists());
    let failures = directory_entries(&repository_state.join("preflight_failures"));
    assert_eq!(failures.len(), 1);
    assert_eq!(
        read_json(&failures[0])["error_code"],
        "release.provider_compatibility"
    );
}

#[test]
fn harp_run_preserves_the_original_error_when_preflight_publication_fails() {
    let repository = RunRepository::new();
    repository.write_policy(policy(false, None, None));
    let providers = RunProviderFixtures::supported();
    let snapshot = RepositorySnapshot::capture(repository.path()).expect("repository snapshot");
    let state = StateRoot::open_or_create(repository.state()).expect("state root");
    state
        .write_private_atomic(
            &Path::new("repositories")
                .join(&snapshot.repository_id)
                .join("preflight_failures"),
            b"block preflight directory creation",
            ReplacePolicy::CreateOnly,
        )
        .expect("blocking private file");

    let output = run_harp(&repository, &providers, "trae", "ci_repair", TASK, 0, true);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("preflight publication failed"), "{stderr}");
    assert!(
        stderr.contains("original_code=repository.policy_disabled"),
        "{stderr}"
    );
    assert!(
        stderr.contains("\"code\":\"repository.policy_disabled\""),
        "{stderr}"
    );
    assert!(providers.launches().is_empty());
}

#[test]
fn harp_run_supports_all_workflows_auto_routing_and_restrictive_budget() {
    let explicit = [
        ("ci_repair", "repair the build"),
        ("code_review", "inspect the patch"),
        ("dependency_update", "refresh the dependency"),
        ("general_coding", "implement the feature"),
    ];

    for (workflow, task) in explicit {
        let repository = RunRepository::new();
        let providers = RunProviderFixtures::supported();
        let output = run_harp(&repository, &providers, "trae", workflow, task, 0, false);
        assert_eq!(output.status.code(), Some(0), "{workflow}");
        let manifest = read_json(&episode_directories(&repository)[0].join("manifest.json"));
        assert_eq!(manifest["workflow"], workflow, "{workflow}");
    }

    let repository = RunRepository::new();
    repository.write_policy(policy(true, Some(&["ci_repair"]), Some(120)));
    let providers = RunProviderFixtures::supported();
    let output = run_harp(
        &repository,
        &providers,
        "codex",
        "auto",
        "tests failing in CI",
        0,
        false,
    );
    assert_eq!(output.status.code(), Some(0));
    let episode = &episode_directories(&repository)[0];
    let manifest = read_json(&episode.join("manifest.json"));
    let context = read_json(&episode.join("context.json"));
    assert_eq!(manifest["workflow"], "ci_repair");
    assert!(context["estimated_tokens"].as_u64().unwrap() <= 120);
    assert!(!context["rejected_items"].as_array().unwrap().is_empty());

    let repository = RunRepository::new();
    repository.write_policy(policy(true, Some(&["ci_repair"]), Some(2_201)));
    let providers = RunProviderFixtures::supported();
    let output = run_harp(&repository, &providers, "trae", "ci_repair", TASK, 0, true);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("repository.policy_budget"));
    assert!(providers.launches().is_empty());
}

fn run_harp(
    repository: &RunRepository,
    providers: &RunProviderFixtures,
    provider: &str,
    workflow: &str,
    task: &str,
    exit_code: u8,
    json_format: bool,
) -> Output {
    let mut command = Command::cargo_bin("harp").expect("harp binary");
    command
        .current_dir(repository.path())
        .env("HARP_HOME", repository.state())
        .env("HARP_TEST_CAPTURE", providers.capture())
        .env("HARP_TEST_EXIT_CODE", exit_code.to_string())
        .env("PATH", providers.path_env());
    if json_format {
        command.args(["--format", "json"]);
    }
    command.args(["run", "--provider", provider, "--workflow", workflow, "--"]);
    command.args(task.split_whitespace());
    command.output().expect("run harp")
}

fn policy(enabled: bool, allowed: Option<&[&str]>, budget: Option<u32>) -> Value {
    json!({
        "schema_version": "harp-repository-policy/v1",
        "enabled": enabled,
        "allowed_workflows": allowed,
        "pinned_release": null,
        "max_context_tokens": budget,
        "required_verification_labels": [],
    })
}

fn assert_lifecycle(stderr: &[u8], provider: &str) {
    let stderr = String::from_utf8_lossy(stderr);
    assert!(stderr.contains("harp run"), "{stderr}");
    assert!(stderr.contains("episode_id="), "{stderr}");
    assert!(
        stderr.contains(&format!("provider={provider} lifecycle=complete")),
        "{stderr}"
    );
}

fn episode_directories(repository: &RunRepository) -> Vec<PathBuf> {
    directory_entries(&repository.repository_state_directory().join("episodes"))
}

fn directory_entries(path: &Path) -> Vec<PathBuf> {
    let mut entries = fs::read_dir(path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
        .map(|entry| entry.expect("directory entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(
        &fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

#[derive(Serialize)]
struct ReleaseAuthorizationFixture {
    schema_version: &'static str,
    release_id: String,
    members: BTreeMap<String, String>,
}

fn publish_codex_only_release(state_path: &Path) -> String {
    let baseline = compile_baseline_release().expect("baseline release");
    let mut identity: ReleaseIdentityV2 = baseline.identity_v2().clone();
    identity.compatible_providers = BTreeSet::from([harp::context_control::ProviderId::Codex]);
    let identity_bytes = json_bytes(&identity).expect("release identity");
    let release_id = sha256_id(&identity_bytes);
    let context_template_bytes =
        json_bytes_with_newline(baseline.context_template_v2()).expect("context template");
    let mut items_bytes = Vec::new();
    for item in baseline.items() {
        items_bytes.extend(json_bytes_with_newline(item).expect("release item"));
    }
    let manifest = ReleaseManifest {
        schema_version: "harp-context-release/v1".to_owned(),
        release_id: release_id.clone(),
        members: BTreeMap::from([
            (
                "context_template.json".to_owned(),
                sha256_hex(&context_template_bytes),
            ),
            ("identity.json".to_owned(), sha256_hex(&identity_bytes)),
            ("items.jsonl".to_owned(), sha256_hex(&items_bytes)),
        ]),
    };
    let manifest_bytes = json_bytes_with_newline(&manifest).expect("release manifest");
    let members = BTreeMap::from([
        (
            PathBuf::from("context_template.json"),
            context_template_bytes,
        ),
        (PathBuf::from("identity.json"), identity_bytes),
        (PathBuf::from("items.jsonl"), items_bytes),
        (PathBuf::from("manifest.json"), manifest_bytes),
    ]);
    let state = StateRoot::open_or_create(state_path).expect("state root");
    state
        .publish_immutable_tree(&Path::new("releases").join(&release_id), &members)
        .expect("release tree");

    let authorization = ReleaseAuthorizationFixture {
        schema_version: "harp-context-release-authorization/v1",
        release_id: release_id.clone(),
        members: members
            .iter()
            .map(|(name, bytes)| {
                (
                    name.to_string_lossy().into_owned(),
                    sha256_hex(bytes.as_slice()),
                )
            })
            .collect(),
    };
    let authorization_directory = state_path
        .join(".harp-internal/release-authorizations")
        .join(&release_id);
    create_private_directory(&authorization_directory);
    let authorization_path = authorization_directory.join("authorization.json");
    fs::write(
        &authorization_path,
        json_bytes_with_newline(&authorization).expect("release authorization"),
    )
    .expect("write release authorization");
    fs::set_permissions(&authorization_path, fs::Permissions::from_mode(0o600))
        .expect("secure release authorization");
    release_id
}

fn create_private_directory(path: &Path) {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            create_private_directory(parent);
        }
    }
    if !path.exists() {
        fs::create_dir(path).expect("create private directory");
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("secure private directory");
}

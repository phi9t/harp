use std::env;

use serde::Serialize;

use super::context::{resolve_routed, route_release, ContextRequest};
use super::episode::{
    generate_episode_id, generate_preflight_id, publish_preflight_failure, CompletionReceipt,
    EpisodeManifest, EpisodeWriter, PreflightFailure,
};
use super::policy::RepositoryPolicy;
use super::provider::{
    build_invocation_from_snapshot, execute, locate, probe_snapshot,
    MaterializedProviderInvocation, ProviderRunOptions,
};
use super::release::{
    compile_baseline_release, inspect_release_in_state, publish_release, ReleaseInspection,
};
use super::repository_state::resolve_worktree_root;
use super::state::StateRoot;
use super::{ProviderId, RepositorySnapshot, WorkflowChoice, WorkflowId, PREFLIGHT_FAILURE_SCHEMA};
use crate::AppError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunRequest {
    pub provider: ProviderId,
    pub workflow: WorkflowChoice,
    pub task: String,
    pub options: ProviderRunOptions,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct RunResult {
    pub episode_id: String,
    pub release_id: String,
    pub context_bundle_sha256: String,
    pub provider_exit_code: Option<i32>,
    pub terminated_by_signal: Option<i32>,
}

pub trait RunReporter {
    fn lifecycle(&mut self, message: &str);
    fn warning(&mut self, message: &str);
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn run(request: RunRequest, reporter: &mut dyn RunReporter) -> Result<RunResult, AppError> {
    reporter.lifecycle(&format!(
        "harp run: provider={} phase=preflight",
        request.provider
    ));
    let current_directory = env::current_dir().map_err(|error| {
        AppError::io(
            "run.current_directory",
            "could not resolve current directory",
            error,
        )
    })?;
    let repository_root = resolve_worktree_root(&current_directory)?;
    let state = StateRoot::open_from_environment()?;
    let repository = RepositorySnapshot::capture(&repository_root)?;

    let prepared = RepositoryPolicy::load(&repository_root).and_then(|policy| {
        enforce_enabled(&policy)?;
        prepare_run(
            &request,
            &repository_root,
            &state,
            &policy,
            &repository,
            reporter,
        )
    });
    let PreparedRun {
        writer,
        materialized,
        episode_id,
        release_id,
        context_bundle_sha256,
    } = match prepared {
        Ok(prepared) => prepared,
        Err(error) => {
            publish_preflight_preserving_error(
                &state,
                &repository.repository_id,
                request.provider,
                &error,
                reporter,
            );
            return Err(error);
        }
    };

    reporter.lifecycle(&format!(
        "harp run: episode_id={episode_id} release_id={release_id} phase=published"
    ));
    let bound = materialized.bind(writer.raw_directory_authority()?)?;
    let execution = execute(bound)?;
    let post_repository = RepositorySnapshot::capture(&repository_root)?;
    let completion =
        CompletionReceipt::from_execution(writer.manifest(), &execution, post_repository)?;
    writer.complete(&completion, &execution.evidence)?;
    reporter.lifecycle(&format!(
        "harp run: episode_id={episode_id} release_id={release_id} phase=completed"
    ));

    Ok(RunResult {
        episode_id,
        release_id,
        context_bundle_sha256,
        provider_exit_code: execution.result.exit_code,
        terminated_by_signal: execution.result.terminated_by_signal,
    })
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn run(_request: RunRequest, _reporter: &mut dyn RunReporter) -> Result<RunResult, AppError> {
    Err(AppError::external(
        "run.unsupported_platform",
        "context-control provider execution requires macOS or Linux",
    ))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
struct PreparedRun<'a> {
    writer: EpisodeWriter<'a>,
    materialized: MaterializedProviderInvocation,
    episode_id: String,
    release_id: String,
    context_bundle_sha256: String,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn prepare_run<'a>(
    request: &RunRequest,
    repository_root: &std::path::Path,
    state: &'a StateRoot,
    policy: &RepositoryPolicy,
    repository: &RepositorySnapshot,
    reporter: &mut dyn RunReporter,
) -> Result<PreparedRun<'a>, AppError> {
    let path = env::var_os("PATH").ok_or_else(|| {
        AppError::external("provider.path", "PATH is required for provider discovery")
    })?;
    let executable = locate(request.provider, &path)?;
    let capabilities = probe_snapshot(request.provider, &executable)?;

    let release = resolve_release(state, policy)?;
    ensure_provider_compatible(&release, request.provider)?;
    let route = route_release(&release, &request.task, request.workflow)?;
    enforce_policy(policy, route.selected())?;
    let budget = effective_budget(&release, policy, route.selected())?;
    reporter.lifecycle(&format!(
        "harp run: provider={} workflow={} release_id={} phase=context",
        request.provider,
        route.selected(),
        release.release_id()
    ));
    let context = resolve_routed(
        &ContextRequest {
            task: request.task.clone(),
            release,
            workflow: request.workflow,
            token_budget: budget,
        },
        route,
    )?;
    let invocation = build_invocation_from_snapshot(
        &capabilities,
        repository_root,
        repository,
        &context,
        &request.task,
        &request.options,
    )?;
    let episode_id = generate_episode_id()?;
    let raw_path = EpisodeWriter::plan_provider_raw_path(
        state,
        &repository.repository_id,
        &episode_id,
        request.provider,
    )?;
    let materialized = invocation.materialize(&raw_path)?;
    let manifest = EpisodeManifest::from_validated_inputs(
        episode_id.clone(),
        &context,
        Some(policy.digest()?),
        &request.task,
        &materialized,
    )?;
    let release_id = manifest.release_id.clone();
    let context_bundle_sha256 = manifest.context_bundle_sha256.clone();
    let writer = EpisodeWriter::begin(state, manifest, &context, &materialized)?;
    Ok(PreparedRun {
        writer,
        materialized,
        episode_id,
        release_id,
        context_bundle_sha256,
    })
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn resolve_release(
    state: &StateRoot,
    policy: &RepositoryPolicy,
) -> Result<ReleaseInspection, AppError> {
    match policy.pinned_release.as_deref() {
        Some(release_id) => inspect_release_in_state(state, release_id),
        None => {
            let release = compile_baseline_release()?;
            publish_release(state, &release)?;
            inspect_release_in_state(state, release.release_id())
        }
    }
}

fn ensure_provider_compatible(
    release: &ReleaseInspection,
    provider: ProviderId,
) -> Result<(), AppError> {
    if !release
        .identity_metadata()
        .compatible_providers
        .contains(&provider)
    {
        return Err(AppError::invalid_input(
            "release.provider_compatibility",
            format!(
                "context release {} is not compatible with provider {provider}",
                release.release_id()
            ),
        ));
    }
    Ok(())
}

fn enforce_policy(policy: &RepositoryPolicy, workflow: WorkflowId) -> Result<(), AppError> {
    enforce_enabled(policy)?;
    if policy
        .allowed_workflows
        .as_ref()
        .is_some_and(|allowed| !allowed.contains(&workflow))
    {
        return Err(AppError::invalid_input(
            "repository.workflow_not_allowed",
            format!("repository policy does not allow workflow {workflow}"),
        ));
    }
    Ok(())
}

fn enforce_enabled(policy: &RepositoryPolicy) -> Result<(), AppError> {
    if !policy.enabled {
        return Err(AppError::invalid_input(
            "repository.policy_disabled",
            "repository context control is disabled",
        ));
    }
    Ok(())
}

fn effective_budget(
    release: &ReleaseInspection,
    policy: &RepositoryPolicy,
    workflow: WorkflowId,
) -> Result<u32, AppError> {
    let release_budget = release
        .identity_metadata()
        .context_budgets
        .get(&workflow)
        .copied()
        .ok_or_else(|| {
            AppError::invalid_input(
                "release.workflow",
                format!(
                    "context release {} does not bind workflow {workflow}",
                    release.release_id()
                ),
            )
        })?;
    Ok(policy
        .max_context_tokens
        .map_or(release_budget, |maximum| maximum.min(release_budget)))
}

fn publish_preflight_preserving_error(
    state: &StateRoot,
    repository_id: &str,
    provider: ProviderId,
    original: &AppError,
    reporter: &mut dyn RunReporter,
) {
    let result = generate_preflight_id().and_then(|preflight_id| {
        publish_preflight_failure(
            state,
            &PreflightFailure {
                schema_version: PREFLIGHT_FAILURE_SCHEMA.to_owned(),
                preflight_id,
                repository_id: repository_id.to_owned(),
                provider,
                error_code: original.code().to_owned(),
                message: bounded_preflight_message(&original.message),
            },
        )
    });
    if let Err(secondary) = result {
        reporter.warning(&format!(
            "harp run warning: preflight publication failed; original_code={} secondary_code={}",
            original.code(),
            secondary.code()
        ));
    }
}

fn bounded_preflight_message(message: &str) -> String {
    let mut bounded = String::with_capacity(message.len().min(4096));
    for character in message.chars() {
        let character = if character.is_control() {
            ' '
        } else {
            character
        };
        if bounded.len() + character.len_utf8() > 4096 {
            break;
        }
        bounded.push(character);
    }
    if bounded.is_empty() {
        "preflight failed".to_owned()
    } else {
        bounded
    }
}

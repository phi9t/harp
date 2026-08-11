use std::time::Duration;

use harp_artifacts::{ArtifactStore, AttemptKey};
use harp_contracts::{OperationId, ResultEnvelope, RunId, ThreadHandle, TurnSpec, TurnStatus};
use harp_runtime::{ActivityHandle, ActivityRuntime, ActivitySpec};
use harp_state::{
    ActivityPreparation, ActivityRecoveryRecord, AttemptRecord, AttemptState, CliActivityKind,
    CliActivityState, CliTerminalFailureClass, InterruptPurpose, LeaseToken, OperationKind,
    OperationRecord, OperationState, StateStore, TaskClaim,
};

use crate::cancel::{complete_cli_activity_interrupt, runtime_interrupt_purpose};
use crate::scheduler::{
    activity_invocation_digest, pinned_output_schema, resolve_and_bind_scratch,
    semantic_failure_class, semantic_output_error, thread_spec, turn_spec,
    verify_and_register_result_artifacts, ActiveLeaseScope,
};
use crate::{Engine, EngineError, RunExecutionSpec, RunSummary};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryAction {
    DispatchThread,
    MarkIndeterminateAndRetry,
    StartTurn,
    ReadThreadForMarker,
    ReadThreadForTurn,
    AcceptPublishedResult,
    None,
}

pub fn recovery_action(attempt: &AttemptRecord) -> RecoveryAction {
    match attempt.state {
        AttemptState::Prepared => RecoveryAction::DispatchThread,
        AttemptState::DispatchingThread if attempt.thread_id.is_none() => {
            RecoveryAction::MarkIndeterminateAndRetry
        }
        AttemptState::ThreadStarted if attempt.latest_turn_id.is_none() => {
            RecoveryAction::StartTurn
        }
        AttemptState::DispatchingTurn => RecoveryAction::ReadThreadForMarker,
        AttemptState::TurnStarted | AttemptState::Reconciling => RecoveryAction::ReadThreadForTurn,
        AttemptState::ResultPublished => RecoveryAction::AcceptPublishedResult,
        AttemptState::Succeeded
        | AttemptState::Failed
        | AttemptState::Indeterminate
        | AttemptState::Cancelled => RecoveryAction::None,
        AttemptState::DispatchingThread | AttemptState::ThreadStarted => RecoveryAction::None,
    }
}

impl Engine {
    pub async fn resume_run(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        runtime: &mut dyn ActivityRuntime,
        run_id: &RunId,
    ) -> Result<RunSummary, EngineError> {
        state.set_lease_clock(std::sync::Arc::clone(&self.lease_clock))?;
        let execution = state
            .run_execution(run_id)?
            .ok_or_else(|| EngineError::MissingRun {
                run_id: run_id.clone(),
            })?;
        let spec = RunExecutionSpec::from_persisted(
            execution.graph,
            &execution.verified_graph_sha256,
            &execution.provenance,
            artifacts,
            &runtime.provenance()?,
        )?;
        if execution.budget != spec.run_budget()? {
            return Err(EngineError::ExecutionReceipt {
                context: "persisted run budget differs from the pinned execution receipt"
                    .to_owned(),
            });
        }
        if execution.run.state != harp_state::RunState::Active {
            return crate::scheduler::summarize(state, run_id);
        }
        if execution.run.cancellation_requested {
            return self
                .reconcile_run_cancellation(state, runtime, run_id)
                .await;
        }

        state.expire_leases(run_id, self.tick()?)?;
        let activities = state.recoverable_activities(run_id)?;
        if !activities.is_empty() {
            let lease_now = self.lease_clock.unix_seconds()?;
            if activities.iter().any(|activity| {
                state
                    .get_attempt(&activity.attempt_id)
                    .ok()
                    .flatten()
                    .and_then(|attempt| attempt.lease_expires_at)
                    .is_some_and(|expires_at| expires_at > lease_now)
            }) {
                return crate::scheduler::summarize(state, run_id);
            }
            for activity in activities {
                let mut lease = self.reclaim_activity_for_recovery(state, &activity)?;
                let _lease_scope = ActiveLeaseScope::register(
                    std::sync::Arc::clone(&self.active_leases),
                    lease.clone(),
                );
                self.recover_activity(state, artifacts, runtime, &spec, &activity, &mut lease)
                    .await?;
            }
            state.rebuild_ready_tasks(run_id, self.tick()?)?;
            return self
                .schedule_run(state, artifacts, runtime, &spec, run_id)
                .await;
        }
        let attempts = state.nonterminal_attempts(run_id)?;
        if attempts.iter().any(|attempt| {
            attempt.lease_expires_at.is_some_and(|expires_at| {
                self.lease_clock
                    .unix_seconds()
                    .is_ok_and(|now| expires_at > now)
            })
        }) {
            return crate::scheduler::summarize(state, run_id);
        }
        for attempt in attempts {
            if state.get_cli_attempt(&attempt.attempt_id)?.is_some() {
                let lease = self.reclaim_for_recovery(state, &attempt)?;
                let _lease_scope = ActiveLeaseScope::register(
                    std::sync::Arc::clone(&self.active_leases),
                    lease.clone(),
                );
                let claim = claim_from_attempt(state, &attempt, &lease)?;
                self.execute_claim(state, artifacts, runtime, &spec, &claim)
                    .await?;
                continue;
            }
            let lease = self.reclaim_for_recovery(state, &attempt)?;
            let _lease_scope = ActiveLeaseScope::register(
                std::sync::Arc::clone(&self.active_leases),
                lease.clone(),
            );
            return Err(EngineError::ExecutionReceipt {
                context: format!("attempt {} has no CLI recovery record", attempt.attempt_id),
            });
        }
        state.rebuild_ready_tasks(run_id, self.tick()?)?;
        self.schedule_run(state, artifacts, runtime, &spec, run_id)
            .await
    }

    fn reclaim_activity_for_recovery(
        &mut self,
        state: &mut StateStore,
        activity: &ActivityRecoveryRecord,
    ) -> Result<LeaseToken, EngineError> {
        let attempt = state.get_attempt(&activity.attempt_id)?.ok_or_else(|| {
            EngineError::ExecutionReceipt {
                context: "activity recovery attempt disappeared".to_owned(),
            }
        })?;
        self.reclaim_for_recovery(state, &attempt)
    }

    async fn recover_activity(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        runtime: &mut dyn ActivityRuntime,
        spec: &RunExecutionSpec,
        original_activity: &ActivityRecoveryRecord,
        lease: &mut LeaseToken,
    ) -> Result<(), EngineError> {
        let activity = state
            .recoverable_activities(&original_activity.run_id)?
            .into_iter()
            .find(|candidate| candidate.activity_id == original_activity.activity_id)
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "recoverable CLI activity disappeared".to_owned(),
            })?;
        if activity.kind == CliActivityKind::InterruptActivity {
            return self
                .recover_interrupt_activity(state, runtime, &activity, lease)
                .await;
        }
        let attempt = state.get_attempt(&activity.attempt_id)?.ok_or_else(|| {
            EngineError::ExecutionReceipt {
                context: "activity recovery attempt disappeared".to_owned(),
            }
        })?;
        if attempt.wall_started_at.is_none()
            && matches!(
                activity.activity_state,
                CliActivityState::Prepared
                    | CliActivityState::Dispatching
                    | CliActivityState::Running
                    | CliActivityState::Reconciling
            )
        {
            let claim = claim_from_attempt(state, &attempt, lease)?;
            resolve_and_bind_scratch(state, artifacts, &claim, lease, self.tick()?)?;
            state.start_wall_tracking(lease, self.wall_clock.unix_seconds()?, self.tick()?)?;
        }
        match activity.activity_state {
            CliActivityState::Prepared => {
                self.recover_prepared_activity(state, artifacts, runtime, spec, &activity, lease)
                    .await
            }
            CliActivityState::Dispatching if activity.process_record_sha256.is_none() => {
                state.mark_indeterminate(lease, "activity_dispatch_ambiguous", self.tick()?)?;
                Ok(())
            }
            CliActivityState::Dispatching => {
                state.mark_activity_running(lease, &activity.activity_id, self.tick()?)?;
                let refreshed = state
                    .recoverable_activities(&activity.run_id)?
                    .into_iter()
                    .find(|candidate| candidate.activity_id == activity.activity_id)
                    .ok_or_else(|| EngineError::ExecutionReceipt {
                        context: "running CLI activity disappeared".to_owned(),
                    })?;
                self.recover_running_activity(state, artifacts, runtime, spec, &refreshed, lease)
                    .await
            }
            CliActivityState::Running => {
                self.recover_running_activity(state, artifacts, runtime, spec, &activity, lease)
                    .await
            }
            CliActivityState::Reconciling => {
                self.recover_reconciling_activity(state, artifacts, runtime, spec, &activity, lease)
                    .await
            }
            CliActivityState::Completed => {
                self.recover_completed_activity(state, artifacts, runtime, &activity, lease)
                    .await
            }
            CliActivityState::Failed
            | CliActivityState::Indeterminate
            | CliActivityState::Cancelled => Ok(()),
        }
    }

    async fn recover_interrupt_activity(
        &mut self,
        state: &mut StateStore,
        runtime: &mut dyn ActivityRuntime,
        activity: &ActivityRecoveryRecord,
        lease: &mut LeaseToken,
    ) -> Result<(), EngineError> {
        let purpose = activity
            .interrupt_purpose
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "recoverable CLI interrupt activity omitted purpose".to_owned(),
            })?;
        let target_digest = activity
            .target_process_record_sha256
            .clone()
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "recoverable CLI interrupt activity omitted target process record"
                    .to_owned(),
            })?;
        let target = state
            .recoverable_activities(&activity.run_id)?
            .into_iter()
            .find(|candidate| {
                candidate.attempt_id == activity.attempt_id
                    && matches!(
                        candidate.kind,
                        CliActivityKind::StartActivity | CliActivityKind::ContinueActivity
                    )
                    && candidate.process_record_sha256.as_deref() == Some(target_digest.as_str())
            })
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "recoverable CLI interrupt target activity is missing".to_owned(),
            })?;
        let handle = ActivityHandle {
            logical_session_id: target.logical_session_id,
            logical_turn_id: target.logical_turn_id,
            process_record_sha256: target_digest,
            external_session_id: target.external_session_id,
        };
        let receipt = self
            .drive_runtime_future(
                state,
                lease,
                self.runtime_control_deadline(),
                runtime.interrupt(&handle, runtime_interrupt_purpose(purpose)),
            )
            .await?;
        complete_cli_activity_interrupt(
            state,
            lease,
            &handle,
            &activity.activity_id,
            receipt,
            || self.tick(),
        )?;
        match purpose {
            InterruptPurpose::Budget => {
                state.fail_terminal_budget_exhausted(lease, "token_limit", self.tick()?)?;
                Err(EngineError::BudgetExceeded {
                    task_id: activity.task_id.clone(),
                })
            }
            InterruptPurpose::ScannerIntegrity => {
                state.fail_terminal_semantic(lease, "storage_integrity", self.tick()?)?;
                Err(EngineError::RecoveredIntegrityInterrupt {
                    task_id: activity.task_id.clone(),
                    reason_code: "storage_integrity".to_owned(),
                })
            }
            InterruptPurpose::Cancellation => {
                state.record_activity_terminal_outcome(
                    lease,
                    &target.activity_id,
                    CliTerminalFailureClass::Cancelled,
                    self.tick()?,
                )?;
                Ok(())
            }
        }
    }

    async fn recover_prepared_activity(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        runtime: &mut dyn ActivityRuntime,
        spec: &RunExecutionSpec,
        activity: &ActivityRecoveryRecord,
        lease: &mut LeaseToken,
    ) -> Result<(), EngineError> {
        let (claim, node, activity_spec) =
            self.rebuild_activity_spec(state, artifacts, spec, activity, lease)?;
        state.mark_activity_dispatching(lease, &activity.activity_id, self.tick()?)?;
        artifacts.verify_runtime_path(&artifacts.resolve_attempt_scratch(&AttemptKey {
            run_id: claim.run_id.clone(),
            task_id: claim.task_id.clone(),
            attempt_id: claim.attempt_id.clone(),
        })?)?;
        let _logical_thread = self
            .drive_runtime_future(
                state,
                lease,
                Duration::from_secs(claim.budget.timeout_seconds),
                runtime.start_logical_session(activity_spec.thread_spec.clone()),
            )
            .await?;
        let handle = self
            .drive_runtime_future(
                state,
                lease,
                Duration::from_secs(claim.budget.timeout_seconds),
                runtime.start_activity(activity_spec.clone()),
            )
            .await?;
        if handle.logical_session_id != activity.logical_session_id
            || handle.logical_turn_id != activity.logical_turn_id
        {
            return Err(EngineError::ExecutionReceipt {
                context: "recovered activity returned a different identity".to_owned(),
            });
        }
        state.record_process(
            lease,
            &activity.activity_id,
            &handle.process_record_sha256,
            self.tick()?,
        )?;
        if let Some(external_session_id) = handle.external_session_id.clone() {
            state.record_cli_external_session(
                lease,
                &activity.activity_id,
                external_session_id,
                self.tick()?,
            )?;
        }
        state.mark_activity_running(lease, &activity.activity_id, self.tick()?)?;
        let result = self
            .consume_activity(
                state,
                artifacts,
                runtime,
                &claim,
                lease,
                &handle,
                &activity_spec.turn_spec.output_schema,
            )
            .await?;
        self.publish_and_accept_cli(
            state,
            artifacts,
            &claim,
            lease,
            &node,
            &activity.activity_id,
            &result,
        )
        .await
    }

    async fn recover_running_activity(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        runtime: &mut dyn ActivityRuntime,
        spec: &RunExecutionSpec,
        activity: &ActivityRecoveryRecord,
        lease: &mut LeaseToken,
    ) -> Result<(), EngineError> {
        let claim = claim_from_attempt_record(state, &activity.attempt_id, lease)?;
        let node = state
            .task_node(&activity.run_id, &activity.task_id)?
            .ok_or_else(|| EngineError::MissingTask {
                task_id: activity.task_id.clone(),
            })?;
        let handle = recovered_activity_handle(activity)?;
        let output_schema = pinned_output_schema(state, &activity.attempt_id)?;
        self.fail_recovered_wall_if_exceeded(state, lease, &claim)?;
        let result = match observe_terminal_activity(
            self,
            state,
            artifacts,
            runtime,
            &claim,
            lease,
            &handle,
            &output_schema,
        )
        .await
        {
            Ok(TerminalActivityObservation::Completed(result)) => result,
            Ok(TerminalActivityObservation::Incomplete) => {
                self.prepare_and_dispatch_continuation(
                    state, artifacts, runtime, spec, activity, lease,
                )
                .await?;
                return Ok(());
            }
            Err(EngineError::IncompleteTurn { .. }) => {
                self.consume_activity(
                    state,
                    artifacts,
                    runtime,
                    &claim,
                    lease,
                    &handle,
                    &output_schema,
                )
                .await?
            }
            Err(error) => return Err(error),
        };
        self.publish_and_accept_cli(
            state,
            artifacts,
            &claim,
            lease,
            &node,
            &activity.activity_id,
            &result,
        )
        .await
    }

    async fn recover_reconciling_activity(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        runtime: &mut dyn ActivityRuntime,
        spec: &RunExecutionSpec,
        activity: &ActivityRecoveryRecord,
        lease: &mut LeaseToken,
    ) -> Result<(), EngineError> {
        let claim = claim_from_attempt_record(state, &activity.attempt_id, lease)?;
        let node = state
            .task_node(&activity.run_id, &activity.task_id)?
            .ok_or_else(|| EngineError::MissingTask {
                task_id: activity.task_id.clone(),
            })?;
        let handle = recovered_activity_handle(activity)?;
        let output_schema = pinned_output_schema(state, &activity.attempt_id)?;
        self.fail_recovered_wall_if_exceeded(state, lease, &claim)?;
        let result = match observe_terminal_activity(
            self,
            state,
            artifacts,
            runtime,
            &claim,
            lease,
            &handle,
            &output_schema,
        )
        .await?
        {
            TerminalActivityObservation::Completed(result) => result,
            TerminalActivityObservation::Incomplete => {
                self.prepare_and_dispatch_continuation(
                    state, artifacts, runtime, spec, activity, lease,
                )
                .await?;
                return Ok(());
            }
        };
        self.publish_and_accept_cli(
            state,
            artifacts,
            &claim,
            lease,
            &node,
            &activity.activity_id,
            &result,
        )
        .await
    }

    fn fail_recovered_wall_if_exceeded(
        &mut self,
        state: &mut StateStore,
        lease: &LeaseToken,
        claim: &TaskClaim,
    ) -> Result<(), EngineError> {
        let attempt =
            state
                .get_attempt(&claim.attempt_id)?
                .ok_or_else(|| EngineError::ExecutionReceipt {
                    context: "recovered activity attempt disappeared before wall reconciliation"
                        .to_owned(),
                })?;
        let wall = self.reconcile_recovered_wall(state, lease, &attempt)?;
        if wall.attempt_limit_exceeded || wall.run_limit_exceeded {
            state.fail_terminal_budget_exhausted(
                lease,
                "recovered_terminal_wall_limit",
                self.tick()?,
            )?;
            return Err(EngineError::BudgetExceeded {
                task_id: claim.task_id.clone(),
            });
        }
        Ok(())
    }

    async fn prepare_and_dispatch_continuation(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        runtime: &mut dyn ActivityRuntime,
        spec: &RunExecutionSpec,
        activity: &ActivityRecoveryRecord,
        lease: &mut LeaseToken,
    ) -> Result<(), EngineError> {
        if activity.external_session_id.is_none() {
            state.record_activity_terminal_outcome(
                lease,
                &activity.activity_id,
                harp_state::CliTerminalFailureClass::NonResumableProtocolFailure,
                self.tick()?,
            )?;
            return Err(EngineError::IncompleteTurn {
                task_id: activity.task_id.clone(),
            });
        }
        state.mark_activity_reconciling(lease, &activity.activity_id, self.tick()?)?;
        let node = state
            .task_node(&activity.run_id, &activity.task_id)?
            .ok_or_else(|| EngineError::MissingTask {
                task_id: activity.task_id.clone(),
            })?;
        let continuation_id = OperationId::new();
        let continuation_turn = self.continuation_activity_turn_spec(
            state,
            artifacts,
            &node,
            activity,
            &continuation_id,
        )?;
        let invocation_sha256 = activity_invocation_digest(&continuation_turn)?;
        let preparation =
            ActivityPreparation::new(activity.activity_dir.clone(), invocation_sha256)?;
        let continuation = state.prepare_continuation_after_failure(
            lease,
            &activity.activity_id,
            CliTerminalFailureClass::ResumableInterrupted,
            continuation_id,
            &preparation,
            self.tick()?,
        )?;
        let refreshed = state
            .recoverable_activities(&activity.run_id)?
            .into_iter()
            .find(|candidate| candidate.activity_id == continuation.activity_id)
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "prepared continuation is not recoverable".to_owned(),
            })?;
        self.recover_prepared_activity(state, artifacts, runtime, spec, &refreshed, lease)
            .await
    }

    async fn recover_completed_activity(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        runtime: &mut dyn ActivityRuntime,
        activity: &ActivityRecoveryRecord,
        lease: &mut LeaseToken,
    ) -> Result<(), EngineError> {
        let attempt = state.get_attempt(&activity.attempt_id)?.ok_or_else(|| {
            EngineError::ExecutionReceipt {
                context: "completed activity attempt disappeared".to_owned(),
            }
        })?;
        match attempt.state {
            AttemptState::ResultPublished => {
                self.recover_published_result(state, artifacts, &attempt, lease)
            }
            AttemptState::Reconciling => {
                let claim = claim_from_attempt(state, &attempt, lease)?;
                let node = state
                    .task_node(&attempt.run_id, &attempt.task_id)?
                    .ok_or_else(|| EngineError::MissingTask {
                        task_id: attempt.task_id.clone(),
                    })?;
                let Some(digest) = attempt.result_sha256.as_deref() else {
                    let handle = recovered_activity_handle(activity)?;
                    let output_schema = pinned_output_schema(state, &attempt.attempt_id)?;
                    let result = match observe_terminal_activity(
                        self,
                        state,
                        artifacts,
                        runtime,
                        &claim,
                        lease,
                        &handle,
                        &output_schema,
                    )
                    .await?
                    {
                        TerminalActivityObservation::Completed(result) => result,
                        TerminalActivityObservation::Incomplete => {
                            return Err(EngineError::IncompleteTurn {
                                task_id: attempt.task_id,
                            });
                        }
                    };
                    return self
                        .publish_reconciling_result(state, artifacts, &claim, lease, &result);
                };
                self.finish_published_result(state, artifacts, &claim, lease, node.kind, digest)
            }
            AttemptState::Succeeded
            | AttemptState::Failed
            | AttemptState::Indeterminate
            | AttemptState::Cancelled => Ok(()),
            other => Err(EngineError::ExecutionReceipt {
                context: format!(
                    "completed CLI activity cannot recover legacy attempt state {}",
                    other.as_str()
                ),
            }),
        }
    }

    fn rebuild_activity_spec(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        spec: &RunExecutionSpec,
        activity: &ActivityRecoveryRecord,
        lease: &LeaseToken,
    ) -> Result<(TaskClaim, harp_contracts::TaskNode, ActivitySpec), EngineError> {
        let claim = claim_from_attempt_record(state, &activity.attempt_id, lease)?;
        let node = state
            .task_node(&activity.run_id, &activity.task_id)?
            .ok_or_else(|| EngineError::MissingTask {
                task_id: activity.task_id.clone(),
            })?;
        let scratch = resolve_and_bind_scratch(state, artifacts, &claim, lease, self.tick()?)?;
        let turn_spec = match activity.kind {
            CliActivityKind::StartActivity => {
                turn_spec(spec, &claim, &activity.activity_id, &scratch)?
            }
            CliActivityKind::ContinueActivity => self.continuation_activity_turn_spec(
                state,
                artifacts,
                &node,
                activity,
                &activity.activity_id,
            )?,
            CliActivityKind::InterruptActivity => {
                return Err(EngineError::ExecutionReceipt {
                    context: "interrupt activity cannot be rebuilt as semantic activity".to_owned(),
                });
            }
        };
        let invocation_sha256 = activity_invocation_digest(&turn_spec)?;
        if invocation_sha256 != activity.invocation_sha256 {
            return Err(EngineError::ExecutionReceipt {
                context: "recovered activity invocation digest changed".to_owned(),
            });
        }
        let activity_dir = scratch
            .canonical_path()
            .to_str()
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "resolved scratch path is not UTF-8".to_owned(),
            })?
            .to_owned();
        if activity_dir != activity.activity_dir {
            return Err(EngineError::ExecutionReceipt {
                context: "recovered activity directory changed".to_owned(),
            });
        }
        let activity_spec = ActivitySpec {
            logical_session_id: activity.logical_session_id.clone(),
            logical_turn_id: activity.logical_turn_id.clone(),
            thread_spec: thread_spec(&node, spec, &scratch)?,
            turn_spec,
            activity_dir,
            invocation_sha256,
            external_session_id: activity.external_session_id.clone(),
        };
        activity_spec.validate()?;
        Ok((claim, node, activity_spec))
    }

    fn continuation_activity_turn_spec(
        &mut self,
        _state: &mut StateStore,
        artifacts: &ArtifactStore,
        node: &harp_contracts::TaskNode,
        activity: &ActivityRecoveryRecord,
        operation_id: &OperationId,
    ) -> Result<TurnSpec, EngineError> {
        let checkpoint = artifacts
            .read_checkpoint(&AttemptKey {
                run_id: activity.run_id.clone(),
                task_id: activity.task_id.clone(),
                attempt_id: activity.attempt_id.clone(),
            })?
            .ok_or_else(|| EngineError::MissingCheckpoint {
                task_id: activity.task_id.clone(),
            })?;
        checkpoint.validate().map_err(EngineError::Contract)?;
        let instruction = serde_json::to_string(&serde_json::json!({
            "schemaVersion": 1,
            "taskId": activity.task_id,
            "continuation": true,
            "checkpoint": {
                "phase": checkpoint.phase,
                "completedUnits": checkpoint.completed_units,
                "pendingUnits": checkpoint.pending_units,
                "evidenceCount": checkpoint.evidence_count,
            },
            "instructions": [
                "Inspect existing thread history and durable scratch state.",
                "Continue from the validated checkpoint without repeating completed analysis.",
                "Retain existing evidence and satisfy the original output schema.",
                "Publish a final result envelope with cumulative token usage."
            ],
        }))
        .map_err(|source| EngineError::Serialization {
            context: "continuation prompt",
            source,
        })?;
        let output_schema = serde_json::from_str(&node.output_schema).map_err(|source| {
            EngineError::Serialization {
                context: "continuation output schema",
                source,
            }
        })?;
        let turn = TurnSpec {
            instruction,
            operation_marker: operation_id.clone(),
            output_schema,
            model: Some(node.model_policy.clone()),
            reasoning_effort: None,
        };
        turn.validate().map_err(EngineError::Contract)?;
        Ok(turn)
    }

    fn reclaim_for_recovery(
        &mut self,
        state: &mut StateStore,
        attempt: &AttemptRecord,
    ) -> Result<LeaseToken, EngineError> {
        let expired_at = attempt
            .lease_expires_at
            .or(attempt.last_lease_expires_at)
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: format!(
                    "attempt {} has no reclaimable lease history",
                    attempt.attempt_id
                ),
            })?;
        let event_now = self.tick()?;
        let lease_now = self.lease_clock.unix_seconds()?;
        let new_expiry = lease_now
            .checked_add(self.config.lease_seconds)
            .ok_or(EngineError::ClockOverflow)?;
        state
            .reclaim_attempt(
                &attempt.attempt_id,
                &self.config.worker_id,
                expired_at,
                event_now,
                new_expiry,
            )
            .map_err(EngineError::from)
    }

    fn publish_reconciling_result(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        claim: &TaskClaim,
        lease: &LeaseToken,
        result: &harp_contracts::ResultEnvelope,
    ) -> Result<(), EngineError> {
        let operation = state
            .operations_for_attempt(&claim.attempt_id)?
            .into_iter()
            .find(|operation| operation.kind == OperationKind::PublishResult)
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "reconciling attempt has no publication operation".to_owned(),
            })?;
        publish_existing_result(self, state, artifacts, claim, lease, result, &operation)
    }

    fn recover_published_result(
        &mut self,
        state: &mut StateStore,
        artifacts: &ArtifactStore,
        attempt: &AttemptRecord,
        lease: &LeaseToken,
    ) -> Result<(), EngineError> {
        let claim = claim_from_attempt(state, attempt, lease)?;
        let node = state
            .task_node(&attempt.run_id, &attempt.task_id)?
            .ok_or_else(|| EngineError::MissingTask {
                task_id: attempt.task_id.clone(),
            })?;
        let digest =
            attempt
                .result_sha256
                .as_deref()
                .ok_or_else(|| EngineError::ExecutionReceipt {
                    context: "published attempt has no result digest".to_owned(),
                })?;
        let artifact = state
            .artifact(digest)?
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "published result has no registered artifact metadata".to_owned(),
            })?;
        let result_bytes = artifacts.read_verified(&artifact)?;
        let output_schema = pinned_output_schema(state, &attempt.attempt_id)?;
        let result = match crate::output::decode_result_envelope(
            &output_schema,
            &result_bytes,
            &attempt.task_id,
            Some(attempt.observed_tokens),
        ) {
            Ok(result) => result,
            Err(error) if semantic_output_error(&error) => {
                return self.fail_semantic_result(
                    state,
                    artifacts,
                    &claim,
                    lease,
                    semantic_failure_class(&error),
                    error,
                );
            }
            Err(error) => return Err(error),
        };
        verify_nested_artifact(state, artifacts, result.answer_ref.as_ref())?;
        for evidence in &result.evidence {
            verify_nested_artifact(state, artifacts, Some(evidence))?;
        }
        verify_nested_artifact(state, artifacts, Some(&result.trace_ref))?;
        self.finish_published_result(state, artifacts, &claim, lease, node.kind, digest)
    }
}

fn verify_nested_artifact(
    state: &mut StateStore,
    artifacts: &ArtifactStore,
    expected: Option<&harp_contracts::ArtifactRef>,
) -> Result<(), EngineError> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let persisted =
        state
            .artifact(&expected.sha256)?
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: format!(
                    "result references unregistered artifact {}",
                    expected.sha256
                ),
            })?;
    if &persisted != expected {
        return Err(EngineError::ExecutionReceipt {
            context: format!("result artifact metadata differs for {}", expected.sha256),
        });
    }
    artifacts.read_verified(&persisted)?;
    Ok(())
}

fn publish_existing_result(
    engine: &mut Engine,
    state: &mut StateStore,
    artifacts: &ArtifactStore,
    claim: &TaskClaim,
    lease: &LeaseToken,
    result: &harp_contracts::ResultEnvelope,
    operation: &OperationRecord,
) -> Result<(), EngineError> {
    if !matches!(
        operation.state,
        OperationState::Dispatching | OperationState::Completed
    ) {
        return Err(EngineError::ExecutionReceipt {
            context: "existing result publication is not dispatching or completed".to_owned(),
        });
    }
    verify_and_register_result_artifacts(state, artifacts, result, engine.tick()?)?;
    let bytes = serde_json::to_vec(result).map_err(|source| EngineError::Serialization {
        context: "recovered result envelope",
        source,
    })?;
    let result_ref = artifacts.publish(&bytes, "application/vnd.harp.result+json")?;
    state.register_artifact(&result_ref, engine.tick()?)?;
    if operation.state == OperationState::Dispatching {
        state.complete_operation(lease, &operation.operation_id, None, engine.tick()?)?;
    }
    state.record_result_published(
        lease,
        &operation.operation_id,
        result,
        &result_ref,
        engine.tick()?,
    )?;
    let node = state
        .task_node(&claim.run_id, &claim.task_id)?
        .ok_or_else(|| EngineError::MissingTask {
            task_id: claim.task_id.clone(),
        })?;
    engine.finish_published_result(
        state,
        artifacts,
        claim,
        lease,
        node.kind,
        &result_ref.sha256,
    )
}

fn claim_from_attempt(
    state: &mut StateStore,
    _attempt: &AttemptRecord,
    lease: &LeaseToken,
) -> Result<TaskClaim, EngineError> {
    state
        .restore_task_claim(lease, lease.expires_at().saturating_sub(1))
        .map_err(EngineError::from)
}

fn claim_from_attempt_record(
    state: &mut StateStore,
    attempt_id: &harp_contracts::AttemptId,
    lease: &LeaseToken,
) -> Result<TaskClaim, EngineError> {
    let attempt = state
        .get_attempt(attempt_id)?
        .ok_or_else(|| EngineError::ExecutionReceipt {
            context: "activity recovery attempt disappeared".to_owned(),
        })?;
    claim_from_attempt(state, &attempt, lease)
}

fn recovered_activity_handle(
    activity: &ActivityRecoveryRecord,
) -> Result<ActivityHandle, EngineError> {
    Ok(ActivityHandle {
        logical_session_id: activity.logical_session_id.clone(),
        logical_turn_id: activity.logical_turn_id.clone(),
        process_record_sha256: activity.process_record_sha256.clone().ok_or_else(|| {
            EngineError::ExecutionReceipt {
                context: "recoverable CLI activity has no process record".to_owned(),
            }
        })?,
        external_session_id: activity.external_session_id.clone(),
    })
}

#[allow(clippy::large_enum_variant)]
enum TerminalActivityObservation {
    Completed(ResultEnvelope),
    Incomplete,
}

#[allow(clippy::too_many_arguments)]
async fn observe_terminal_activity(
    engine: &mut Engine,
    state: &mut StateStore,
    artifacts: &ArtifactStore,
    runtime: &mut dyn ActivityRuntime,
    claim: &TaskClaim,
    lease: &mut LeaseToken,
    activity: &ActivityHandle,
    output_schema: &serde_json::Value,
) -> Result<TerminalActivityObservation, EngineError> {
    let control = runtime.control_handle()?;
    let thread = ThreadHandle {
        thread_id: activity.logical_session_id.clone(),
    };
    let snapshot = engine
        .drive_runtime_future(
            state,
            lease,
            engine.runtime_control_deadline(),
            control.read_thread(&thread),
        )
        .await?;
    let turn = snapshot
        .turns
        .iter()
        .find(|turn| turn.turn_id == activity.logical_turn_id)
        .ok_or_else(|| EngineError::ExecutionReceipt {
            context: "recovered CLI activity turn is absent from runtime snapshot".to_owned(),
        })?;
    if turn.status != TurnStatus::Completed {
        return if matches!(turn.status, TurnStatus::Interrupted | TurnStatus::Failed) {
            Ok(TerminalActivityObservation::Incomplete)
        } else {
            Err(EngineError::IncompleteTurn {
                task_id: claim.task_id.clone(),
            })
        };
    }
    let message =
        turn.final_agent_message
            .as_deref()
            .ok_or_else(|| EngineError::InvalidResult {
                task_id: claim.task_id.clone(),
                context: "completed activity omitted final agent message".to_owned(),
            })?;
    let observed = state
        .reservation_usage(&claim.attempt_id)?
        .map(|usage| usage.0)
        .unwrap_or(0);
    match crate::output::decode_result_envelope(
        output_schema,
        message.as_bytes(),
        &claim.task_id,
        Some(observed),
    ) {
        Ok(result) => Ok(TerminalActivityObservation::Completed(result)),
        Err(error) if semantic_output_error(&error) => engine.fail_semantic_result(
            state,
            artifacts,
            claim,
            lease,
            semantic_failure_class(&error),
            error,
        ),
        Err(error) => Err(error),
    }
}

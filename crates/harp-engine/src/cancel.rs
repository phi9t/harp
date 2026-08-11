use std::str::FromStr;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use harp_contracts::{OperationId, RunId, ThreadHandle, TurnHandle};
use harp_runtime::{
    ActivityHandle, ActivityRuntime, InterruptPurpose as RuntimeInterruptPurpose, InterruptReceipt,
    RuntimeControl,
};
use harp_state::{
    ActivityPreparation, AttemptRecord, AttemptState, CliActivityKind, CliActivityState,
    CliSignalStage, InterruptIntent, InterruptPurpose, LeaseToken, OperationKind, OperationState,
    StateStore,
};

use crate::{Engine, EngineError, RunSummary};

#[derive(Clone)]
pub struct EngineControl {
    worker_id: String,
    lease_seconds: i64,
    lease_renewal_threshold_seconds: i64,
    runtime_operation_timeout: Duration,
    now: Arc<AtomicI64>,
    active_leases: Arc<Mutex<std::collections::BTreeMap<harp_contracts::AttemptId, LeaseToken>>>,
    lease_clock: Arc<dyn harp_state::LeaseClock>,
}

impl Engine {
    pub fn control_handle(&self) -> EngineControl {
        EngineControl {
            worker_id: self.config.worker_id.clone(),
            lease_seconds: self.config.lease_seconds,
            lease_renewal_threshold_seconds: self.config.lease_renewal_threshold_seconds,
            runtime_operation_timeout: self.runtime_control_deadline(),
            now: Arc::clone(&self.now),
            active_leases: Arc::clone(&self.active_leases),
            lease_clock: Arc::clone(&self.lease_clock),
        }
    }

    pub async fn cancel_run(
        &mut self,
        state: &mut StateStore,
        runtime: &mut dyn ActivityRuntime,
        run_id: &RunId,
    ) -> Result<RunSummary, EngineError> {
        let runtime_control = runtime.control_handle()?;
        self.control_handle()
            .cancel_run(state, runtime_control.as_ref(), run_id)
            .await
    }

    pub(crate) async fn reconcile_run_cancellation(
        &mut self,
        state: &mut StateStore,
        runtime: &mut dyn ActivityRuntime,
        run_id: &RunId,
    ) -> Result<RunSummary, EngineError> {
        state.expire_leases(run_id, self.tick()?)?;
        let lease_now = self.lease_clock.unix_seconds()?;
        for original in state.nonterminal_attempts(run_id)? {
            if original
                .lease_expires_at
                .is_some_and(|expires_at| expires_at > lease_now)
            {
                let Some(mut lease) = self
                    .active_leases
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .remove(&original.attempt_id)
                else {
                    continue;
                };
                self.cancel_cli_attempt(state, runtime, &original, &mut lease)
                    .await?;
                continue;
            }
            self.active_leases
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove(&original.attempt_id);
            let mut lease = self.reclaim_cancelled_attempt(state, &original)?;
            self.cancel_cli_attempt(state, runtime, &original, &mut lease)
                .await?;
        }
        crate::scheduler::summarize(state, run_id)
    }

    async fn cancel_cli_attempt(
        &mut self,
        state: &mut StateStore,
        runtime: &mut dyn ActivityRuntime,
        original: &AttemptRecord,
        lease: &mut LeaseToken,
    ) -> Result<(), EngineError> {
        if let Some(activity) = state
            .recoverable_activities(&original.run_id)?
            .into_iter()
            .find(|activity| {
                activity.attempt_id == original.attempt_id
                    && matches!(
                        activity.kind,
                        CliActivityKind::StartActivity | CliActivityKind::ContinueActivity
                    )
                    && matches!(
                        activity.activity_state,
                        CliActivityState::Dispatching
                            | CliActivityState::Running
                            | CliActivityState::Reconciling
                    )
            })
        {
            let Some(process_record_sha256) = activity.process_record_sha256.clone() else {
                state.mark_indeterminate(
                    lease,
                    "cancelled_activity_dispatch_ambiguous",
                    self.tick()?,
                )?;
                return Ok(());
            };
            let handle = ActivityHandle {
                logical_session_id: activity.logical_session_id,
                logical_turn_id: activity.logical_turn_id,
                process_record_sha256,
                external_session_id: activity.external_session_id,
            };
            let intent = InterruptIntent::cancellation("run_cancellation")?;
            self.interrupt_cli_activity(state, runtime, lease, &handle, &intent)
                .await?;
            state.record_activity_terminal_outcome(
                lease,
                &activity.activity_id,
                harp_state::CliTerminalFailureClass::Cancelled,
                self.tick()?,
            )?;
            return Ok(());
        }
        state.finalize_task_cancelled(lease, self.tick()?)?;
        Ok(())
    }

    async fn interrupt_cli_activity(
        &mut self,
        state: &mut StateStore,
        runtime: &mut dyn ActivityRuntime,
        lease: &mut LeaseToken,
        activity: &ActivityHandle,
        intent: &InterruptIntent,
    ) -> Result<(), EngineError> {
        let interrupt =
            prepare_cli_activity_interrupt(state, lease, activity, intent, self.tick()?)?;
        let receipt = self
            .drive_runtime_future(
                state,
                lease,
                self.runtime_control_deadline(),
                runtime.interrupt(activity, runtime_interrupt_purpose(intent.purpose)),
            )
            .await?;
        complete_cli_activity_interrupt(
            state,
            lease,
            activity,
            &interrupt.activity_id,
            receipt,
            || self.tick(),
        )?;
        Ok(())
    }

    fn reclaim_cancelled_attempt(
        &mut self,
        state: &mut StateStore,
        attempt: &AttemptRecord,
    ) -> Result<LeaseToken, EngineError> {
        let expired_at = attempt
            .lease_expires_at
            .or(attempt.last_lease_expires_at)
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: format!(
                    "cancelled attempt {} has no reclaimable lease history",
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
}

fn active_cli_activity(
    state: &mut StateStore,
    original: &AttemptRecord,
) -> Result<Option<harp_state::ActivityRecoveryRecord>, EngineError> {
    Ok(state
        .recoverable_activities(&original.run_id)?
        .into_iter()
        .find(|activity| {
            activity.attempt_id == original.attempt_id
                && matches!(
                    activity.kind,
                    CliActivityKind::StartActivity | CliActivityKind::ContinueActivity
                )
                && matches!(
                    activity.activity_state,
                    CliActivityState::Dispatching
                        | CliActivityState::Running
                        | CliActivityState::Reconciling
                )
        }))
}

pub(crate) fn prepare_cli_activity_interrupt(
    state: &mut StateStore,
    lease: &LeaseToken,
    activity: &ActivityHandle,
    intent: &InterruptIntent,
    now: i64,
) -> Result<harp_state::CliActivityRecord, EngineError> {
    let run_id = state
        .get_attempt(lease.attempt_id())?
        .ok_or_else(|| EngineError::ExecutionReceipt {
            context: "CLI interrupt attempt disappeared".to_owned(),
        })?
        .run_id;
    if let Some(existing) = state
        .recoverable_activities(&run_id)?
        .into_iter()
        .find(|candidate| {
            candidate.attempt_id == *lease.attempt_id()
                && candidate.kind == CliActivityKind::InterruptActivity
                && candidate.ordinal == 0
                && candidate.target_process_record_sha256.as_deref()
                    == Some(activity.process_record_sha256.as_str())
                && candidate.interrupt_purpose == Some(intent.purpose)
        })
    {
        return state
            .get_cli_activity(&existing.activity_id)?
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "recoverable CLI interrupt disappeared".to_owned(),
            });
    }
    let interrupt_id = OperationId::new();
    let preparation = ActivityPreparation::new(
        format!("/private/tmp/harp-interrupt-{}", interrupt_id),
        activity.process_record_sha256.clone(),
    )?;
    state
        .prepare_interrupt_activity(
            lease,
            interrupt_id,
            0,
            &preparation,
            &activity.process_record_sha256,
            intent.purpose,
            now,
        )
        .map_err(EngineError::from)
}

pub(crate) fn complete_cli_activity_interrupt(
    state: &mut StateStore,
    lease: &LeaseToken,
    activity: &ActivityHandle,
    interrupt_activity_id: &OperationId,
    receipt: InterruptReceipt,
    mut tick: impl FnMut() -> Result<i64, EngineError>,
) -> Result<(), EngineError> {
    if receipt.process_record_sha256 != activity.process_record_sha256 {
        return Err(EngineError::ExecutionReceipt {
            context: "interrupt receipt targets a different process record".to_owned(),
        });
    }
    state.advance_cli_signal_stage(
        lease,
        interrupt_activity_id,
        CliSignalStage::Prepared,
        CliSignalStage::SigintPrepared,
        tick()?,
    )?;
    state.advance_cli_signal_stage(
        lease,
        interrupt_activity_id,
        CliSignalStage::SigintPrepared,
        CliSignalStage::SigintSent,
        tick()?,
    )?;
    state.advance_cli_signal_stage(
        lease,
        interrupt_activity_id,
        CliSignalStage::SigintSent,
        if receipt.quiescent {
            CliSignalStage::Quiescent
        } else {
            CliSignalStage::Indeterminate
        },
        tick()?,
    )?;
    Ok(())
}

pub(crate) fn runtime_interrupt_purpose(purpose: InterruptPurpose) -> RuntimeInterruptPurpose {
    match purpose {
        InterruptPurpose::Budget => RuntimeInterruptPurpose::Budget,
        InterruptPurpose::ScannerIntegrity => RuntimeInterruptPurpose::ScannerIntegrity,
        InterruptPurpose::Cancellation => RuntimeInterruptPurpose::Cancellation,
    }
}

impl EngineControl {
    pub async fn cancel_run(
        &self,
        state: &mut StateStore,
        runtime: &dyn RuntimeControl,
        run_id: &RunId,
    ) -> Result<RunSummary, EngineError> {
        state.set_lease_clock(Arc::clone(&self.lease_clock))?;
        let execution = state
            .run_execution(run_id)?
            .ok_or_else(|| EngineError::MissingRun {
                run_id: run_id.clone(),
            })?;
        crate::scheduler::verify_persisted_runtime_control(
            &execution.verified_graph_sha256,
            &execution.provenance,
            &runtime.provenance()?,
        )?;
        state.request_run_cancellation(run_id, self.tick()?)?;
        state.expire_leases(run_id, self.tick()?)?;
        let lease_now = self.lease_clock.unix_seconds()?;
        for attempt in state.nonterminal_attempts(run_id)? {
            let mut lease = if attempt
                .lease_expires_at
                .is_some_and(|expires_at| expires_at > lease_now)
            {
                let Some(lease) = self
                    .active_leases
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .remove(&attempt.attempt_id)
                else {
                    continue;
                };
                lease
            } else {
                self.reclaim_cancelled_attempt(state, &attempt)?
            };
            self.cancel_attempt(state, runtime, &attempt, &mut lease)
                .await?;
        }
        crate::scheduler::summarize(state, run_id)
    }

    fn tick(&self) -> Result<i64, EngineError> {
        self.now
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |now| now.checked_add(1))
            .map(|previous| previous + 1)
            .map_err(|_| EngineError::ClockOverflow)
    }

    fn reclaim_cancelled_attempt(
        &self,
        state: &mut StateStore,
        attempt: &AttemptRecord,
    ) -> Result<LeaseToken, EngineError> {
        let expired_at = attempt
            .lease_expires_at
            .or(attempt.last_lease_expires_at)
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: format!(
                    "cancelled attempt {} has no reclaimable lease history",
                    attempt.attempt_id
                ),
            })?;
        let lease_now = self.lease_clock.unix_seconds()?;
        let new_expiry = lease_now
            .checked_add(self.lease_seconds)
            .ok_or(EngineError::ClockOverflow)?;
        state
            .reclaim_attempt(
                &attempt.attempt_id,
                &self.worker_id,
                expired_at,
                self.tick()?,
                new_expiry,
            )
            .map_err(EngineError::from)
    }

    async fn cancel_attempt(
        &self,
        state: &mut StateStore,
        runtime: &dyn RuntimeControl,
        original: &AttemptRecord,
        lease: &mut LeaseToken,
    ) -> Result<(), EngineError> {
        if let Some(activity) = active_cli_activity(state, original)? {
            let Some(process_record_sha256) = activity.process_record_sha256.clone() else {
                state.mark_indeterminate(
                    lease,
                    "cancelled_activity_dispatch_ambiguous",
                    self.tick()?,
                )?;
                return Ok(());
            };
            let handle = ActivityHandle {
                logical_session_id: activity.logical_session_id,
                logical_turn_id: activity.logical_turn_id,
                process_record_sha256,
                external_session_id: activity.external_session_id,
            };
            let intent = InterruptIntent::cancellation("run_cancellation")?;
            self.interrupt_cli_activity(state, runtime, lease, &handle, &intent)
                .await?;
            state.record_activity_terminal_outcome(
                lease,
                &activity.activity_id,
                harp_state::CliTerminalFailureClass::Cancelled,
                self.tick()?,
            )?;
            return Ok(());
        }

        let mut attempt = state.get_attempt(&original.attempt_id)?.ok_or_else(|| {
            EngineError::ExecutionReceipt {
                context: "cancellation attempt disappeared".to_owned(),
            }
        })?;
        if attempt.state == AttemptState::DispatchingThread && attempt.thread_id.is_none() {
            state.mark_indeterminate(lease, "cancelled_thread_start_ambiguous", self.tick()?)?;
            return Ok(());
        }
        if attempt.state == AttemptState::DispatchingTurn {
            let thread = ThreadHandle {
                thread_id: attempt.thread_id.clone().ok_or_else(|| {
                    EngineError::ExecutionReceipt {
                        context: "dispatching turn cancellation has no thread_id".to_owned(),
                    }
                })?,
            };
            let marker = attempt.latest_operation_marker.as_deref().ok_or_else(|| {
                EngineError::ExecutionReceipt {
                    context: "dispatching turn cancellation has no marker".to_owned(),
                }
            })?;
            let operation_id =
                OperationId::from_str(marker.strip_prefix("harp-operation:").ok_or_else(|| {
                    EngineError::ExecutionReceipt {
                        context: "dispatching turn cancellation marker is malformed".to_owned(),
                    }
                })?)
                .map_err(EngineError::Contract)?;
            let snapshot = self
                .drive_runtime_future(state, lease, runtime.read_thread(&thread))
                .await?;
            if let Some(turn) = snapshot
                .turn_by_operation_marker(&operation_id)
                .map_err(EngineError::Contract)?
            {
                state.record_turn_started(lease, &operation_id, &turn.turn_id, self.tick()?)?;
            } else {
                state.mark_indeterminate(
                    lease,
                    "cancelled_turn_dispatch_ambiguous",
                    self.tick()?,
                )?;
                return Ok(());
            }
            attempt = state.get_attempt(&original.attempt_id)?.ok_or_else(|| {
                EngineError::ExecutionReceipt {
                    context: "reconciled cancellation attempt disappeared".to_owned(),
                }
            })?;
        }
        if self
            .recover_pending_interrupt(state, runtime, &attempt, lease)
            .await?
        {
            return Ok(());
        }
        if let (Some(thread_id), Some(turn_id)) =
            (attempt.thread_id.clone(), attempt.latest_turn_id.clone())
        {
            let turn = TurnHandle { thread_id, turn_id };
            let intent = InterruptIntent::cancellation("run_cancellation")?;
            let interrupt = state.prepare_interrupt_operation(lease, 0, &intent, self.tick()?)?;
            if interrupt.state == OperationState::Prepared {
                state.mark_operation_dispatching(lease, &interrupt.operation_id, self.tick()?)?;
            }
            let snapshot = self
                .drive_runtime_future(
                    state,
                    lease,
                    runtime.read_thread(&ThreadHandle {
                        thread_id: turn.thread_id.clone(),
                    }),
                )
                .await?;
            let turn_state = snapshot
                .turns
                .iter()
                .find(|candidate| candidate.turn_id == turn.turn_id)
                .ok_or_else(|| EngineError::ExecutionReceipt {
                    context: "cancellation target turn is absent from runtime snapshot".to_owned(),
                })?;
            if turn_state.status == harp_contracts::TurnStatus::InProgress {
                self.drive_runtime_future(state, lease, runtime.interrupt(&turn))
                    .await?;
            }
            state.complete_operation(
                lease,
                &interrupt.operation_id,
                Some(&turn.turn_id.to_string()),
                self.tick()?,
            )?;
        }
        state.finalize_task_cancelled(lease, self.tick()?)?;
        Ok(())
    }

    async fn interrupt_cli_activity(
        &self,
        state: &mut StateStore,
        runtime: &dyn RuntimeControl,
        lease: &mut LeaseToken,
        activity: &ActivityHandle,
        intent: &InterruptIntent,
    ) -> Result<(), EngineError> {
        let interrupt =
            prepare_cli_activity_interrupt(state, lease, activity, intent, self.tick()?)?;
        let receipt = self
            .drive_runtime_future(
                state,
                lease,
                runtime.interrupt_activity(activity, runtime_interrupt_purpose(intent.purpose)),
            )
            .await?;
        complete_cli_activity_interrupt(
            state,
            lease,
            activity,
            &interrupt.activity_id,
            receipt,
            || self.tick(),
        )?;
        Ok(())
    }

    async fn recover_pending_interrupt(
        &self,
        state: &mut StateStore,
        runtime: &dyn RuntimeControl,
        attempt: &AttemptRecord,
        lease: &mut LeaseToken,
    ) -> Result<bool, EngineError> {
        let Some(operation) = state
            .operations_for_attempt(&attempt.attempt_id)?
            .into_iter()
            .filter(|operation| {
                operation.kind == OperationKind::InterruptTurn
                    && matches!(
                        operation.state,
                        OperationState::Prepared | OperationState::Dispatching
                    )
            })
            .max_by_key(|operation| (operation.ordinal, operation.operation_id.clone()))
        else {
            return Ok(false);
        };
        let intent =
            operation
                .interrupt_intent
                .clone()
                .ok_or_else(|| EngineError::ExecutionReceipt {
                    context: "nonterminal interrupt operation omitted pinned intent".to_owned(),
                })?;
        if operation.state == OperationState::Prepared {
            state.mark_operation_dispatching(lease, &operation.operation_id, self.tick()?)?;
        }
        let thread_id = attempt
            .thread_id
            .clone()
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "interrupt control recovery has no thread_id".to_owned(),
            })?;
        let turn_id =
            attempt
                .latest_turn_id
                .clone()
                .ok_or_else(|| EngineError::ExecutionReceipt {
                    context: "interrupt control recovery has no latest_turn_id".to_owned(),
                })?;
        if operation.target_external_id.as_deref() != Some(turn_id.to_string().as_str()) {
            return Err(EngineError::ExecutionReceipt {
                context: "interrupt control target differs from latest turn".to_owned(),
            });
        }
        let thread = ThreadHandle {
            thread_id: thread_id.clone(),
        };
        let turn = TurnHandle { thread_id, turn_id };
        let snapshot = self
            .drive_runtime_future(state, lease, runtime.read_thread(&thread))
            .await?;
        let turn_state = snapshot
            .turns
            .iter()
            .find(|candidate| candidate.turn_id == turn.turn_id)
            .ok_or_else(|| EngineError::ExecutionReceipt {
                context: "interrupt control target is absent from runtime snapshot".to_owned(),
            })?;
        if turn_state.status == harp_contracts::TurnStatus::InProgress {
            self.drive_runtime_future(state, lease, runtime.interrupt(&turn))
                .await?;
        }
        state.complete_operation(
            lease,
            &operation.operation_id,
            Some(&turn.turn_id.to_string()),
            self.tick()?,
        )?;
        match intent.purpose {
            InterruptPurpose::Cancellation => {
                state.finalize_task_cancelled(lease, self.tick()?)?;
                Ok(true)
            }
            InterruptPurpose::Budget => {
                state.fail_terminal_budget_exhausted(lease, &intent.reason_code, self.tick()?)?;
                Err(EngineError::BudgetExceeded {
                    task_id: attempt.task_id.clone(),
                })
            }
            InterruptPurpose::ScannerIntegrity => {
                state.fail_terminal_semantic(lease, &intent.reason_code, self.tick()?)?;
                Err(EngineError::RecoveredIntegrityInterrupt {
                    task_id: attempt.task_id.clone(),
                    reason_code: intent.reason_code,
                })
            }
        }
    }

    async fn drive_runtime_future<T, F>(
        &self,
        state: &mut StateStore,
        lease: &mut LeaseToken,
        future: F,
    ) -> Result<T, EngineError>
    where
        F: std::future::Future<Output = Result<T, harp_runtime::RuntimeError>>,
    {
        tokio::pin!(future);
        let renewal_period = Duration::from_secs(self.lease_renewal_threshold_seconds as u64);
        let renewal = tokio::time::sleep(renewal_period);
        let timeout = tokio::time::sleep(self.runtime_operation_timeout);
        tokio::pin!(renewal);
        tokio::pin!(timeout);
        loop {
            tokio::select! {
                result = &mut future => return result.map_err(EngineError::from),
                _ = &mut renewal => {
                    let lease_now = self.lease_clock.unix_seconds()?;
                    let new_expiry = lease_now
                        .checked_add(self.lease_seconds)
                        .ok_or(EngineError::ClockOverflow)?;
                    *lease = state.renew_lease(lease, new_expiry, self.tick()?)?;
                    renewal.as_mut().reset(tokio::time::Instant::now() + renewal_period);
                }
                _ = &mut timeout => {
                    return Err(EngineError::RuntimeOperationTimeout {
                        deadline: self.runtime_operation_timeout,
                    });
                }
            }
        }
    }
}

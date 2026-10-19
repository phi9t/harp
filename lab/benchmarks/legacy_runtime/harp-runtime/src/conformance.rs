use std::time::Duration;

use harp_contracts::{
    ExternalSessionId, RuntimeErrorKind, RuntimeEvent, ThreadSpec, TokenUsage, TurnId, TurnSpec,
    TurnStatus,
};

use crate::{
    collect_until_terminal, ActivityHandle, ActivityRuntime, ActivitySpec, CollectionLimits,
    InterruptPurpose, RuntimeError,
};

const CONFORMANCE_EVENT_LIMIT: usize = 1_024;

fn conformance_limits() -> CollectionLimits {
    CollectionLimits {
        max_events: CONFORMANCE_EVENT_LIMIT,
        max_serialized_bytes: 8 * 1024 * 1024,
        max_total_wait: Duration::from_secs(60),
        per_event_wait: Duration::from_secs(5),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeFixture {
    pub thread_spec: ThreadSpec,
    pub logical_turn_id: TurnId,
    pub turn_spec: TurnSpec,
    pub activity_dir: String,
    pub invocation_sha256: String,
    pub expected_process_record_sha256: String,
    pub expected_external_session_id: Option<ExternalSessionId>,
    pub expected_status: TurnStatus,
    pub expected_final_message: Option<String>,
    pub minimum_total_tokens: u64,
}

impl RuntimeFixture {
    fn validate(&self) -> Result<(), RuntimeError> {
        self.thread_spec
            .validate()
            .map_err(RuntimeError::from_contract)?;
        self.turn_spec
            .validate()
            .map_err(RuntimeError::from_contract)?;
        if self.expected_status == TurnStatus::InProgress {
            return Err(RuntimeError::protocol(
                "conformance fixture expected_status must be terminal",
            ));
        }
        Ok(())
    }
}

pub async fn assert_runtime_conformance(
    runtime: &mut dyn ActivityRuntime,
    fixture: &RuntimeFixture,
) -> Result<(), RuntimeError> {
    fixture.validate()?;
    let activity = start_fixture_activity(runtime, fixture).await?;
    let events = collect_until_terminal(runtime, &activity, &conformance_limits()).await?;
    let completion = validate_event_order(&events, &activity, fixture.minimum_total_tokens)?;
    validate_completion(fixture, completion)
}

async fn start_fixture_activity(
    runtime: &mut dyn ActivityRuntime,
    fixture: &RuntimeFixture,
) -> Result<ActivityHandle, RuntimeError> {
    let session = runtime
        .start_logical_session(fixture.thread_spec.clone())
        .await?;
    session.validate().map_err(RuntimeError::from_contract)?;
    let activity_spec = ActivitySpec {
        logical_session_id: session.thread_id.clone(),
        logical_turn_id: fixture.logical_turn_id.clone(),
        thread_spec: fixture.thread_spec.clone(),
        turn_spec: fixture.turn_spec.clone(),
        activity_dir: fixture.activity_dir.clone(),
        invocation_sha256: fixture.invocation_sha256.clone(),
        external_session_id: None,
    };
    activity_spec.validate()?;
    let activity = runtime.start_activity(activity_spec).await?;
    activity.validate()?;
    if activity.logical_session_id != session.thread_id
        || activity.logical_turn_id != fixture.logical_turn_id
    {
        return Err(RuntimeError::protocol(
            "start_activity returned a handle for a different logical activity",
        ));
    }
    if activity.process_record_sha256 != fixture.expected_process_record_sha256 {
        return Err(RuntimeError::protocol(
            "start_activity returned an unexpected process record digest",
        ));
    }
    if activity.external_session_id != fixture.expected_external_session_id {
        return Err(RuntimeError::protocol(
            "start_activity returned an unexpected external session identity",
        ));
    }
    Ok(activity)
}

fn validate_event_order<'a>(
    events: &'a [RuntimeEvent],
    activity: &ActivityHandle,
    minimum_total_tokens: u64,
) -> Result<&'a harp_contracts::TurnCompletedEvent, RuntimeError> {
    let mut started = false;
    let mut usage_seen = false;
    let mut completion = None;
    for event in events {
        match event {
            RuntimeEvent::TurnStarted(started_event)
                if started_event.thread_id == activity.logical_session_id
                    && started_event.turn_id == activity.logical_turn_id =>
            {
                started = true;
            }
            RuntimeEvent::TokenUsage(usage)
                if usage.thread_id == activity.logical_session_id
                    && usage.turn_id == activity.logical_turn_id =>
            {
                if !started {
                    return Err(RuntimeError::protocol(
                        "matching token usage preceded TurnStarted",
                    ));
                }
                validate_usage(&usage.usage, minimum_total_tokens)?;
                usage_seen = true;
            }
            RuntimeEvent::TurnCompleted(completed)
                if completed.thread_id == activity.logical_session_id
                    && completed.turn.turn_id == activity.logical_turn_id =>
            {
                if !started {
                    return Err(RuntimeError::protocol(
                        "matching TurnCompleted preceded TurnStarted",
                    ));
                }
                if !usage_seen {
                    return Err(RuntimeError::protocol(
                        "matching TurnCompleted preceded TokenUsage",
                    ));
                }
                completion = Some(completed);
            }
            _ => {}
        }
    }
    if !started {
        return Err(RuntimeError::protocol(
            "matching TurnStarted event was not observed",
        ));
    }
    if !usage_seen {
        return Err(RuntimeError::protocol(
            "matching TokenUsage event was not observed",
        ));
    }
    completion
        .ok_or_else(|| RuntimeError::protocol("matching TurnCompleted event was not observed"))
}

fn validate_usage(usage: &TokenUsage, minimum_total_tokens: u64) -> Result<(), RuntimeError> {
    if usage.total_tokens < minimum_total_tokens {
        return Err(RuntimeError::protocol(format!(
            "token usage {} was below expected minimum {minimum_total_tokens}",
            usage.total_tokens
        )));
    }
    Ok(())
}

fn validate_completion(
    fixture: &RuntimeFixture,
    completion: &harp_contracts::TurnCompletedEvent,
) -> Result<(), RuntimeError> {
    if completion.turn.status != fixture.expected_status {
        return Err(RuntimeError::protocol(
            "terminal event returned an unexpected status",
        ));
    }
    if completion.turn.final_agent_message != fixture.expected_final_message {
        return Err(RuntimeError::protocol(
            "terminal event returned an unexpected final message",
        ));
    }
    if completion.turn.operation_marker.as_ref() != Some(&fixture.turn_spec.operation_marker) {
        return Err(RuntimeError::protocol(
            "terminal event omitted or changed the operation marker",
        ));
    }
    Ok(())
}

pub async fn assert_interrupt_conformance(
    runtime: &mut dyn ActivityRuntime,
    fixture: &RuntimeFixture,
) -> Result<(), RuntimeError> {
    fixture.validate()?;
    let activity = start_fixture_activity(runtime, fixture).await?;
    let receipt = runtime
        .interrupt(&activity, InterruptPurpose::Cancellation)
        .await?;
    receipt.validate()?;
    if receipt.process_record_sha256 != activity.process_record_sha256 {
        return Err(RuntimeError::protocol(
            "interrupt receipt targeted a different process record",
        ));
    }
    if !receipt.quiescent {
        return Err(RuntimeError::protocol(
            "interrupt conformance requires a quiescent process",
        ));
    }
    Ok(())
}

pub fn expect_error_kind<T>(
    result: Result<T, RuntimeError>,
    expected: RuntimeErrorKind,
) -> Result<(), RuntimeError> {
    match result {
        Err(error) if error.kind() == expected => Ok(()),
        Err(error) => Err(RuntimeError::protocol(format!(
            "expected {expected:?}, received {:?}: {}",
            error.kind(),
            error.message()
        ))),
        Ok(_) => Err(RuntimeError::protocol(format!(
            "expected {expected:?}, received success"
        ))),
    }
}

use std::time::Duration;

use harp_contracts::{
    RuntimeErrorKind, RuntimeEvent, ThreadHandle, ThreadSpec, TokenUsage, TurnHandle, TurnSpec,
    TurnStatus,
};

use crate::{collect_until_terminal, CodexRuntime, CollectionLimits, RuntimeError};

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
    pub turn_spec: TurnSpec,
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
    runtime: &mut dyn CodexRuntime,
    fixture: &RuntimeFixture,
) -> Result<(), RuntimeError> {
    fixture.validate()?;
    let thread = runtime.start_thread(fixture.thread_spec.clone()).await?;
    thread.validate().map_err(RuntimeError::from_contract)?;
    let primary = run_conformance(runtime, fixture, &thread).await;
    finish_with_shutdown(runtime, thread, primary).await
}

async fn run_conformance(
    runtime: &mut dyn CodexRuntime,
    fixture: &RuntimeFixture,
    thread: &ThreadHandle,
) -> Result<(), RuntimeError> {
    let turn = runtime
        .start_turn(thread, fixture.turn_spec.clone())
        .await?;
    turn.validate().map_err(RuntimeError::from_contract)?;
    if turn.thread_id != thread.thread_id {
        return Err(RuntimeError::protocol(
            "start_turn returned a handle for a different thread",
        ));
    }

    let events = collect_until_terminal(runtime, &turn, &conformance_limits()).await?;
    let completion = validate_event_order(&events, &turn, fixture.minimum_total_tokens)?;
    validate_completion(fixture, completion)?;

    let snapshot = runtime.read_thread(thread).await?;
    snapshot.validate().map_err(RuntimeError::from_contract)?;
    if snapshot.thread_id != thread.thread_id {
        return Err(RuntimeError::protocol(
            "read_thread returned a snapshot for a different thread",
        ));
    }
    let Some(snapshot_turn) = snapshot
        .turns
        .iter()
        .find(|snapshot_turn| snapshot_turn.turn_id == turn.turn_id)
    else {
        return Err(RuntimeError::protocol(
            "read_thread snapshot omitted the completed turn",
        ));
    };
    if snapshot_turn != &completion.turn {
        return Err(RuntimeError::protocol(
            "read_thread turn does not equal the terminal event snapshot",
        ));
    }

    let resumed = runtime.resume_thread(thread).await?;
    resumed.validate().map_err(RuntimeError::from_contract)?;
    if resumed.thread_id != thread.thread_id || resumed != snapshot {
        return Err(RuntimeError::protocol(
            "resume_thread did not preserve the normalized thread history",
        ));
    }
    Ok(())
}

fn validate_event_order<'a>(
    events: &'a [RuntimeEvent],
    turn: &TurnHandle,
    minimum_total_tokens: u64,
) -> Result<&'a harp_contracts::TurnCompletedEvent, RuntimeError> {
    let mut started = false;
    let mut usage_seen = false;
    let mut completion = None;
    for event in events {
        match event {
            RuntimeEvent::TurnStarted(started_event)
                if started_event.thread_id == turn.thread_id
                    && started_event.turn_id == turn.turn_id =>
            {
                started = true;
            }
            RuntimeEvent::TokenUsage(usage)
                if usage.thread_id == turn.thread_id && usage.turn_id == turn.turn_id =>
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
                if completed.thread_id == turn.thread_id
                    && completed.turn.turn_id == turn.turn_id =>
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

async fn finish_with_shutdown(
    runtime: &mut dyn CodexRuntime,
    thread: ThreadHandle,
    primary: Result<(), RuntimeError>,
) -> Result<(), RuntimeError> {
    let cleanup = runtime.shutdown_thread(thread).await;
    match (primary, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(primary), Ok(())) => Err(primary),
        (Ok(()), Err(cleanup)) => Err(cleanup),
        (Err(primary), Err(cleanup)) => Err(RuntimeError::with_cleanup(primary, cleanup)),
    }
}

pub async fn assert_interrupt_conformance(
    runtime: &mut dyn CodexRuntime,
    fixture: &RuntimeFixture,
) -> Result<(), RuntimeError> {
    fixture.validate()?;
    let thread = runtime.start_thread(fixture.thread_spec.clone()).await?;
    let primary = async {
        let turn = runtime
            .start_turn(&thread, fixture.turn_spec.clone())
            .await?;
        if turn.thread_id != thread.thread_id {
            return Err(RuntimeError::protocol(
                "start_turn returned a handle for a different thread",
            ));
        }
        runtime.interrupt(&turn).await?;
        let snapshot = runtime.read_thread(&thread).await?;
        let Some(interrupted) = snapshot
            .turns
            .iter()
            .find(|candidate| candidate.turn_id == turn.turn_id)
        else {
            return Err(RuntimeError::protocol(
                "read_thread snapshot omitted the interrupted turn",
            ));
        };
        if interrupted.status != TurnStatus::Interrupted {
            return Err(RuntimeError::protocol(
                "interrupt did not produce an Interrupted turn snapshot",
            ));
        }
        Ok(())
    }
    .await;
    finish_with_shutdown(runtime, thread, primary).await
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

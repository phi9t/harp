use std::sync::{Arc, Mutex};

use harp_contracts::{
    OperationId, RuntimeEvent, ThreadHandle, ThreadSnapshot, ThreadStatus, TurnHandle,
    TurnSnapshot, TurnStatus,
};

use crate::RuntimeError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FakeExternalEffect {
    CreatedThread {
        thread: ThreadHandle,
    },
    CreatedTurn {
        turn: TurnHandle,
        operation_marker: OperationId,
    },
    Interrupted {
        turn: TurnHandle,
    },
    Shutdown {
        thread: ThreadHandle,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeTurnState {
    pub turn: TurnHandle,
    pub operation_marker: OperationId,
    pub status: TurnStatus,
    pub final_agent_message: Option<String>,
    pub interrupted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FakeThreadState {
    pub thread: ThreadHandle,
    pub turns: Vec<FakeTurnState>,
    pub shutdown: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FakeBackendSnapshot {
    pub threads: Vec<FakeThreadState>,
}

#[derive(Clone, Default)]
pub struct FakeBackend {
    inner: Arc<Mutex<FakeExternalState>>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FakeExternalState {
    pub(crate) threads: Vec<FakeThreadState>,
    pub(crate) effects: Vec<FakeExternalEffect>,
}

impl FakeBackend {
    pub fn snapshot(&self) -> FakeBackendSnapshot {
        let state = self.lock();
        FakeBackendSnapshot {
            threads: state.threads.clone(),
        }
    }

    pub fn effects(&self) -> Vec<FakeExternalEffect> {
        self.lock().effects.clone()
    }

    pub(crate) fn state(&self) -> FakeExternalState {
        self.lock().clone()
    }

    pub(crate) fn create_thread(&self, thread: ThreadHandle) -> Result<(), RuntimeError> {
        self.lock().create_thread(thread)
    }

    pub(crate) fn create_turn(
        &self,
        thread: &ThreadHandle,
        turn: TurnHandle,
        operation_marker: OperationId,
    ) -> Result<(), RuntimeError> {
        self.lock().create_turn(thread, turn, operation_marker)
    }

    pub(crate) fn interrupt(&self, turn: &TurnHandle) -> Result<(), RuntimeError> {
        self.lock().interrupt(turn)
    }

    pub(crate) fn shutdown(&self, thread: &ThreadHandle) -> Result<(), RuntimeError> {
        self.lock().shutdown(thread)
    }

    pub(crate) fn apply_event(&self, event: &RuntimeEvent) -> Result<(), RuntimeError> {
        self.lock().apply_event(event)
    }

    pub(crate) fn snapshot_thread(
        &self,
        thread: &ThreadHandle,
    ) -> Result<ThreadSnapshot, RuntimeError> {
        self.lock().snapshot_thread(thread)
    }

    pub(crate) fn has_turn(&self, turn: &TurnHandle) -> bool {
        self.lock().has_turn(turn)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, FakeExternalState> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl FakeExternalState {
    pub(crate) fn create_thread(&mut self, thread: ThreadHandle) -> Result<(), RuntimeError> {
        if self
            .threads
            .iter()
            .any(|state| state.thread.thread_id == thread.thread_id)
        {
            return Err(RuntimeError::protocol(format!(
                "fake backend already contains thread {}",
                thread.thread_id
            )));
        }
        self.effects.push(FakeExternalEffect::CreatedThread {
            thread: thread.clone(),
        });
        self.threads.push(FakeThreadState {
            thread,
            turns: Vec::new(),
            shutdown: false,
        });
        Ok(())
    }

    pub(crate) fn create_turn(
        &mut self,
        thread: &ThreadHandle,
        turn: TurnHandle,
        operation_marker: OperationId,
    ) -> Result<(), RuntimeError> {
        if turn.thread_id != thread.thread_id {
            return Err(RuntimeError::protocol(
                "fake backend turn belongs to a different thread",
            ));
        }
        let thread_state = self.thread_mut(thread)?;
        if thread_state
            .turns
            .iter()
            .any(|state| state.turn.turn_id == turn.turn_id)
        {
            return Err(RuntimeError::protocol(format!(
                "fake backend already contains turn {}",
                turn.turn_id
            )));
        }
        if thread_state
            .turns
            .iter()
            .any(|state| state.operation_marker == operation_marker)
        {
            return Err(RuntimeError::protocol(
                "fake backend already contains the operation marker",
            ));
        }
        thread_state.turns.push(FakeTurnState {
            turn: turn.clone(),
            operation_marker: operation_marker.clone(),
            status: TurnStatus::InProgress,
            final_agent_message: None,
            interrupted: false,
        });
        self.effects.push(FakeExternalEffect::CreatedTurn {
            turn,
            operation_marker,
        });
        Ok(())
    }

    pub(crate) fn interrupt(&mut self, turn: &TurnHandle) -> Result<(), RuntimeError> {
        let turn_state = self.turn_mut(turn)?;
        if turn_state.interrupted {
            return Ok(());
        }
        turn_state.status = TurnStatus::Interrupted;
        turn_state.interrupted = true;
        self.effects
            .push(FakeExternalEffect::Interrupted { turn: turn.clone() });
        Ok(())
    }

    pub(crate) fn shutdown(&mut self, thread: &ThreadHandle) -> Result<(), RuntimeError> {
        self.thread_mut(thread)?.shutdown = true;
        self.effects.push(FakeExternalEffect::Shutdown {
            thread: thread.clone(),
        });
        Ok(())
    }

    pub(crate) fn apply_event(&mut self, event: &RuntimeEvent) -> Result<(), RuntimeError> {
        match event {
            RuntimeEvent::TurnStarted(event) => {
                let handle = TurnHandle {
                    thread_id: event.thread_id.clone(),
                    turn_id: event.turn_id.clone(),
                };
                self.turn_mut(&handle)?.status = TurnStatus::InProgress;
            }
            RuntimeEvent::TurnCompleted(event) => {
                let handle = TurnHandle {
                    thread_id: event.thread_id.clone(),
                    turn_id: event.turn.turn_id.clone(),
                };
                let turn = self.turn_mut(&handle)?;
                turn.status = event.turn.status;
                turn.final_agent_message = event.turn.final_agent_message.clone();
                turn.interrupted = event.turn.status == TurnStatus::Interrupted;
            }
            RuntimeEvent::ThreadStarted(_)
            | RuntimeEvent::TokenUsage(_)
            | RuntimeEvent::ServerRequest(_)
            | RuntimeEvent::Lagged(_)
            | RuntimeEvent::Disconnected(_) => {}
        }
        Ok(())
    }

    pub(crate) fn snapshot_thread(
        &self,
        thread: &ThreadHandle,
    ) -> Result<ThreadSnapshot, RuntimeError> {
        let thread_state = self.thread(thread)?;
        let active = thread_state
            .turns
            .iter()
            .any(|turn| turn.status == TurnStatus::InProgress);
        Ok(ThreadSnapshot {
            thread_id: thread.thread_id.clone(),
            status: if active {
                ThreadStatus::Active
            } else {
                ThreadStatus::Idle
            },
            turns: thread_state
                .turns
                .iter()
                .map(|turn| TurnSnapshot {
                    turn_id: turn.turn.turn_id.clone(),
                    operation_marker: Some(turn.operation_marker.clone()),
                    status: turn.status,
                    final_agent_message: turn.final_agent_message.clone(),
                })
                .collect(),
        })
    }

    pub(crate) fn has_turn(&self, turn: &TurnHandle) -> bool {
        self.threads.iter().any(|thread| {
            thread.thread.thread_id == turn.thread_id
                && thread
                    .turns
                    .iter()
                    .any(|state| state.turn.turn_id == turn.turn_id)
        })
    }

    fn thread(&self, thread: &ThreadHandle) -> Result<&FakeThreadState, RuntimeError> {
        self.threads
            .iter()
            .find(|state| state.thread.thread_id == thread.thread_id)
            .ok_or_else(|| {
                RuntimeError::protocol(format!(
                    "fake backend does not contain thread {}",
                    thread.thread_id
                ))
            })
    }

    fn thread_mut(&mut self, thread: &ThreadHandle) -> Result<&mut FakeThreadState, RuntimeError> {
        self.threads
            .iter_mut()
            .find(|state| state.thread.thread_id == thread.thread_id)
            .ok_or_else(|| {
                RuntimeError::protocol(format!(
                    "fake backend does not contain thread {}",
                    thread.thread_id
                ))
            })
    }

    fn turn_mut(&mut self, turn: &TurnHandle) -> Result<&mut FakeTurnState, RuntimeError> {
        self.thread_mut(&ThreadHandle {
            thread_id: turn.thread_id.clone(),
        })?
        .turns
        .iter_mut()
        .find(|state| state.turn.turn_id == turn.turn_id)
        .ok_or_else(|| {
            RuntimeError::protocol(format!(
                "fake backend does not contain turn {}",
                turn.turn_id
            ))
        })
    }
}

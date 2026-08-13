#![allow(dead_code)]

use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use harp_contracts::{
    AttemptId, Checkpoint, RunId, RuntimeErrorKind, RuntimeEvent, TaskId, ThreadHandle, ThreadId,
    ThreadSnapshot, ThreadSpec, ThreadStatus, TokenUsage, TokenUsageEvent, TurnCompletedEvent,
    TurnHandle, TurnSnapshot, TurnStartedEvent, TurnStatus,
};
use harp_runtime::{
    ActivityHandle, ActivityRuntime, ActivitySpec, InterruptPurpose as RuntimeInterruptPurpose,
    InterruptReceipt, RuntimeControl, RuntimeError, RuntimeProvenance,
};
use serde::{Deserialize, Serialize};
use sha2::Digest;

#[derive(Clone)]
pub struct PersistentFakeBackend {
    inner: Arc<Mutex<FakeState>>,
    runtime_identity: String,
    adapter_kind: String,
    sidecar_path: Option<PathBuf>,
    next_event_hook: Option<Arc<dyn Fn() + Send + Sync>>,
    before_workspace_verify_hook: Option<Arc<dyn Fn() + Send + Sync>>,
}

#[derive(Default, Serialize, Deserialize)]
struct FakeState {
    start_logical_session_calls: usize,
    start_activity_calls: usize,
    interrupt_calls: usize,
    read_failures: VecDeque<bool>,
    duplicate_marker_on_read: bool,
    scratch_writes: bool,
    next_event_delay_millis: u64,
    start_turn_delay_millis: u64,
    interrupt_delay_millis: u64,
    interrupt_response_delay_millis: u64,
    outcomes: BTreeMap<String, VecDeque<FakeOutcome>>,
    threads: BTreeMap<String, FakeThread>,
}

#[derive(Serialize, Deserialize)]
struct FakeThread {
    handle: ThreadHandle,
    cwd: String,
    turns: Vec<FakeTurn>,
}

#[derive(Serialize, Deserialize)]
struct FakeTurn {
    handle: TurnHandle,
    marker: harp_contracts::OperationId,
    terminal_status: TurnStatus,
    status: TurnStatus,
    final_message: Option<String>,
    token_snapshots: VecDeque<u64>,
    started_emitted: bool,
    completed_emitted: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FakeOutcome {
    pub status: TurnStatus,
    pub final_message: Option<String>,
    pub total_tokens: u64,
    pub token_snapshots: Vec<u64>,
}

impl PersistentFakeBackend {
    pub fn new(outcomes: BTreeMap<String, VecDeque<FakeOutcome>>) -> Self {
        Self::with_runtime_identity(outcomes, "persistent-fake-runtime")
    }

    pub fn with_runtime_identity(
        outcomes: BTreeMap<String, VecDeque<FakeOutcome>>,
        runtime_identity: &str,
    ) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FakeState {
                outcomes,
                ..FakeState::default()
            })),
            runtime_identity: runtime_identity.to_owned(),
            adapter_kind: "persistent-fake".to_owned(),
            sidecar_path: None,
            next_event_hook: None,
            before_workspace_verify_hook: None,
        }
    }

    pub fn with_adapter_kind(mut self, adapter_kind: &str) -> Self {
        self.adapter_kind = adapter_kind.to_owned();
        self
    }

    pub fn open_file(
        path: &Path,
        outcomes: BTreeMap<String, VecDeque<FakeOutcome>>,
    ) -> io::Result<Self> {
        let state = if path.exists() {
            serde_json::from_slice(&fs::read(path)?)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
        } else {
            FakeState {
                outcomes,
                ..FakeState::default()
            }
        };
        let identity = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .canonicalize()?
            .join(path.file_name().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "sidecar path has no filename")
            })?)
            .display()
            .to_string();
        let backend = Self {
            inner: Arc::new(Mutex::new(state)),
            runtime_identity: identity,
            adapter_kind: "persistent-fake".to_owned(),
            sidecar_path: Some(path.to_path_buf()),
            next_event_hook: None,
            before_workspace_verify_hook: None,
        };
        {
            let state = backend.lock();
            backend.persist(&state)?;
        }
        Ok(backend)
    }

    pub fn runtime(&self) -> PersistentFakeRuntime {
        PersistentFakeRuntime {
            backend: self.clone(),
        }
    }

    pub fn start_logical_session_calls(&self) -> usize {
        self.lock().start_logical_session_calls
    }

    pub fn start_activity_calls(&self) -> usize {
        self.lock().start_activity_calls
    }

    pub fn interrupt_calls(&self) -> usize {
        self.lock().interrupt_calls
    }

    pub fn complete_all_turns(&self) {
        let mut state = self.lock();
        for thread in state.threads.values_mut() {
            for turn in &mut thread.turns {
                turn.status = turn.terminal_status;
                turn.started_emitted = true;
                turn.token_snapshots.clear();
                turn.completed_emitted = true;
            }
        }
    }

    pub fn fail_next_reads(&self, transient: bool, count: usize) {
        self.lock()
            .read_failures
            .extend(std::iter::repeat_n(transient, count));
    }

    pub fn duplicate_marker_on_read(&self) {
        self.lock().duplicate_marker_on_read = true;
    }

    pub fn enable_scratch_writes(&self) {
        self.lock().scratch_writes = true;
    }

    pub fn set_next_event_delay(&self, delay: Duration) {
        self.lock().next_event_delay_millis =
            u64::try_from(delay.as_millis()).expect("fake delay fits u64");
    }

    pub fn set_start_turn_delay(&self, delay: Duration) {
        self.lock().start_turn_delay_millis =
            u64::try_from(delay.as_millis()).expect("fake delay fits u64");
    }

    pub fn set_interrupt_delay(&self, delay: Duration) {
        self.lock().interrupt_delay_millis =
            u64::try_from(delay.as_millis()).expect("fake delay fits u64");
    }

    pub fn set_interrupt_response_delay(&self, delay: Duration) {
        self.lock().interrupt_response_delay_millis =
            u64::try_from(delay.as_millis()).expect("fake delay fits u64");
    }

    pub fn with_next_event_hook(mut self, hook: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.next_event_hook = Some(hook);
        self
    }

    pub fn with_before_workspace_verify_hook(mut self, hook: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.before_workspace_verify_hook = Some(hook);
        self
    }

    pub fn latest_thread(&self) -> ThreadHandle {
        self.lock()
            .threads
            .values()
            .last()
            .expect("fake backend has a thread")
            .handle
            .clone()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, FakeState> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn persist(&self, state: &FakeState) -> io::Result<()> {
        let Some(path) = &self.sidecar_path else {
            return Ok(());
        };
        let bytes = serde_json::to_vec(state)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let temporary = path.with_extension("tmp");
        {
            let mut file = fs::File::create(&temporary)?;
            std::io::Write::write_all(&mut file, &bytes)?;
            file.sync_all()?;
        }
        fs::rename(&temporary, path)?;
        fs::File::open(path.parent().unwrap_or_else(|| Path::new(".")))?.sync_all()
    }
}

pub struct PersistentFakeRuntime {
    backend: PersistentFakeBackend,
}

async fn next_backend_event(
    backend: &PersistentFakeBackend,
    turn: &TurnHandle,
) -> Result<RuntimeEvent, RuntimeError> {
    if let Some(hook) = &backend.next_event_hook {
        hook();
    }
    let delay_millis = backend.lock().next_event_delay_millis;
    if delay_millis > 0 {
        tokio::time::sleep(Duration::from_millis(delay_millis)).await;
    }
    let mut state = backend.lock();
    let turn_state = state
        .threads
        .get_mut(&turn.thread_id.to_string())
        .and_then(|thread| {
            thread
                .turns
                .iter_mut()
                .find(|candidate| candidate.handle.turn_id == turn.turn_id)
        })
        .ok_or_else(|| {
            RuntimeError::from_contract(harp_contracts::ContractError::new(
                "turn",
                "fake turn does not exist",
            ))
        })?;
    let event = if !turn_state.started_emitted {
        turn_state.started_emitted = true;
        RuntimeEvent::TurnStarted(TurnStartedEvent {
            thread_id: turn.thread_id.clone(),
            turn_id: turn.turn_id.clone(),
        })
    } else if let Some(total_tokens) = turn_state.token_snapshots.pop_front() {
        RuntimeEvent::TokenUsage(TokenUsageEvent {
            thread_id: turn.thread_id.clone(),
            turn_id: turn.turn_id.clone(),
            usage: TokenUsage {
                total_tokens,
                input_tokens: total_tokens,
                cached_input_tokens: 0,
                output_tokens: 0,
                reasoning_output_tokens: 0,
            },
        })
    } else if !turn_state.completed_emitted {
        turn_state.completed_emitted = true;
        turn_state.status = if turn_state.status == TurnStatus::Interrupted {
            TurnStatus::Interrupted
        } else {
            turn_state.terminal_status
        };
        RuntimeEvent::TurnCompleted(TurnCompletedEvent {
            thread_id: turn.thread_id.clone(),
            turn: TurnSnapshot {
                turn_id: turn.turn_id.clone(),
                operation_marker: Some(turn_state.marker.clone()),
                status: turn_state.status,
                final_agent_message: turn_state.final_message.clone(),
            },
        })
    } else {
        return Err(RuntimeError::from_contract(
            harp_contracts::ContractError::new("fakeTurn", "terminal event was already emitted"),
        ));
    };
    backend
        .persist(&state)
        .map_err(|error| runtime_io("persist fake event", error))?;
    Ok(event)
}

#[async_trait]
impl ActivityRuntime for PersistentFakeRuntime {
    fn provenance(&self) -> Result<RuntimeProvenance, RuntimeError> {
        RuntimeProvenance::new(
            &self.backend.adapter_kind,
            env!("CARGO_PKG_VERSION"),
            &self.backend.runtime_identity,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .map_err(RuntimeError::from_contract)
    }

    fn control_handle(&self) -> Result<Arc<dyn RuntimeControl>, RuntimeError> {
        Ok(Arc::new(PersistentFakeRuntimeControl {
            backend: self.backend.clone(),
        }))
    }

    async fn start_logical_session(
        &mut self,
        spec: ThreadSpec,
    ) -> Result<ThreadHandle, RuntimeError> {
        if let Some(hook) = &self.backend.before_workspace_verify_hook {
            hook();
        }
        if let Some(authority) = &spec.workspace_authority {
            harp_artifacts::verify_runtime_workspace_authority(authority).map_err(|error| {
                runtime_io("verify fake runtime workspace", io::Error::other(error))
            })?;
        }
        let mut state = self.backend.lock();
        state.start_logical_session_calls += 1;
        let handle = ThreadHandle {
            thread_id: ThreadId::from_str(&format!("logical-{}", state.threads.len() + 1))
                .map_err(RuntimeError::from_contract)?,
        };
        state.threads.insert(
            handle.thread_id.to_string(),
            FakeThread {
                handle: handle.clone(),
                cwd: spec.cwd.clone(),
                turns: Vec::new(),
            },
        );
        if state.scratch_writes {
            write_fake_scratch(&spec.cwd)?;
        }
        self.backend
            .persist(&state)
            .map_err(|error| runtime_io("persist fake logical session", error))?;
        Ok(handle)
    }

    async fn start_activity(&mut self, spec: ActivitySpec) -> Result<ActivityHandle, RuntimeError> {
        let delay_millis = self.backend.lock().start_turn_delay_millis;
        if delay_millis > 0 {
            tokio::time::sleep(Duration::from_millis(delay_millis)).await;
        }
        let task_id = task_id_from_instruction(&spec.turn_spec.instruction)?;
        let projected: serde_json::Value = serde_json::from_str(&spec.turn_spec.instruction)
            .map_err(|error| runtime_io("decode projected scratch", io::Error::other(error)))?;
        let mut state = self.backend.lock();
        state
            .threads
            .entry(spec.logical_session_id.to_string())
            .or_insert_with(|| FakeThread {
                handle: ThreadHandle {
                    thread_id: spec.logical_session_id.clone(),
                },
                cwd: spec.thread_spec.cwd.clone(),
                turns: Vec::new(),
            });
        if state.scratch_writes {
            let thread_state = state
                .threads
                .get(&spec.logical_session_id.to_string())
                .ok_or_else(|| {
                    RuntimeError::from_contract(harp_contracts::ContractError::new(
                        "thread",
                        "fake logical session does not exist",
                    ))
                })?;
            let projected_scratch = projected["scratchPath"].as_str().ok_or_else(|| {
                RuntimeError::from_contract(harp_contracts::ContractError::new(
                    "scratchPath",
                    "projected scratch path is missing",
                ))
            })?;
            if thread_state.cwd != projected_scratch {
                return Err(RuntimeError::from_contract(
                    harp_contracts::ContractError::new(
                        "scratchPath",
                        "logical session cwd differs from projected scratch path",
                    ),
                ));
            }
        }
        let outcome = state
            .outcomes
            .get_mut(&task_id)
            .and_then(VecDeque::pop_front)
            .ok_or_else(|| {
                RuntimeError::from_contract(harp_contracts::ContractError::new(
                    "fakeOutcome",
                    "missing scripted task outcome",
                ))
            })?;
        state.start_activity_calls += 1;
        let handle = TurnHandle {
            thread_id: spec.logical_session_id.clone(),
            turn_id: spec.logical_turn_id.clone(),
        };
        let thread_state = state
            .threads
            .get_mut(&spec.logical_session_id.to_string())
            .ok_or_else(|| {
                RuntimeError::from_contract(harp_contracts::ContractError::new(
                    "thread",
                    "fake logical session does not exist",
                ))
            })?;
        thread_state.turns.push(FakeTurn {
            handle,
            marker: spec.turn_spec.operation_marker,
            terminal_status: outcome.status,
            status: TurnStatus::InProgress,
            final_message: outcome.final_message,
            token_snapshots: if outcome.token_snapshots.is_empty() {
                VecDeque::from([outcome.total_tokens])
            } else {
                outcome.token_snapshots.into()
            },
            started_emitted: false,
            completed_emitted: false,
        });
        self.backend
            .persist(&state)
            .map_err(|error| runtime_io("persist fake activity", error))?;
        let logical_session_id = spec.logical_session_id;
        Ok(ActivityHandle {
            logical_session_id: logical_session_id.clone(),
            logical_turn_id: spec.logical_turn_id,
            process_record_sha256: spec.invocation_sha256,
            external_session_id: Some(
                format!("fake-session-{logical_session_id}")
                    .parse()
                    .unwrap(),
            ),
        })
    }

    async fn next_event(
        &mut self,
        activity: &ActivityHandle,
        max_wait: Duration,
    ) -> Result<RuntimeEvent, RuntimeError> {
        let turn = TurnHandle {
            thread_id: activity.logical_session_id.clone(),
            turn_id: activity.logical_turn_id.clone(),
        };
        let _ = max_wait;
        next_backend_event(&self.backend, &turn).await
    }

    async fn interrupt(
        &mut self,
        activity: &ActivityHandle,
        _purpose: RuntimeInterruptPurpose,
    ) -> Result<InterruptReceipt, RuntimeError> {
        let turn = TurnHandle {
            thread_id: activity.logical_session_id.clone(),
            turn_id: activity.logical_turn_id.clone(),
        };
        interrupt_backend(&self.backend, &turn).await?;
        Ok(InterruptReceipt {
            process_record_sha256: activity.process_record_sha256.clone(),
            quiescent: true,
        })
    }
}

struct PersistentFakeRuntimeControl {
    backend: PersistentFakeBackend,
}

#[async_trait]
impl RuntimeControl for PersistentFakeRuntimeControl {
    fn provenance(&self) -> Result<RuntimeProvenance, RuntimeError> {
        RuntimeProvenance::new(
            &self.backend.adapter_kind,
            env!("CARGO_PKG_VERSION"),
            &self.backend.runtime_identity,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .map_err(RuntimeError::from_contract)
    }

    async fn read_thread(&self, thread: &ThreadHandle) -> Result<ThreadSnapshot, RuntimeError> {
        snapshot(&self.backend, thread)
    }

    async fn interrupt(&self, turn: &TurnHandle) -> Result<(), RuntimeError> {
        interrupt_backend(&self.backend, turn).await
    }

    async fn interrupt_activity(
        &self,
        activity: &ActivityHandle,
        _purpose: harp_runtime::InterruptPurpose,
    ) -> Result<harp_runtime::InterruptReceipt, RuntimeError> {
        let turn = TurnHandle {
            thread_id: activity.logical_session_id.clone(),
            turn_id: activity.logical_turn_id.clone(),
        };
        interrupt_backend(&self.backend, &turn).await?;
        Ok(harp_runtime::InterruptReceipt {
            process_record_sha256: activity.process_record_sha256.clone(),
            quiescent: true,
        })
    }
}

async fn interrupt_backend(
    backend: &PersistentFakeBackend,
    turn: &TurnHandle,
) -> Result<(), RuntimeError> {
    let delay_millis = backend.lock().interrupt_delay_millis;
    if delay_millis > 0 {
        tokio::time::sleep(Duration::from_millis(delay_millis)).await;
    }
    let response_delay_millis = {
        let mut state = backend.lock();
        let already_interrupted = state
            .threads
            .get_mut(&turn.thread_id.to_string())
            .and_then(|thread| {
                thread
                    .turns
                    .iter_mut()
                    .find(|candidate| candidate.handle.turn_id == turn.turn_id)
            })
            .ok_or_else(|| {
                RuntimeError::from_contract(harp_contracts::ContractError::new(
                    "turn",
                    "fake turn does not exist",
                ))
            })?
            .status
            == TurnStatus::Interrupted;
        if !already_interrupted {
            state.interrupt_calls += 1;
            state
                .threads
                .get_mut(&turn.thread_id.to_string())
                .and_then(|thread| {
                    thread
                        .turns
                        .iter_mut()
                        .find(|candidate| candidate.handle.turn_id == turn.turn_id)
                })
                .expect("turn was resolved above")
                .status = TurnStatus::Interrupted;
            backend
                .persist(&state)
                .map_err(|error| runtime_io("persist fake interrupt", error))?;
        }
        state.interrupt_response_delay_millis
    };
    if response_delay_millis > 0 {
        tokio::time::sleep(Duration::from_millis(response_delay_millis)).await;
    }
    Ok(())
}

fn attempt_identity_from_cwd(cwd: &str) -> Result<(RunId, TaskId, AttemptId), RuntimeError> {
    let components = Path::new(cwd)
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    let runs = components
        .iter()
        .position(|component| *component == "runs")
        .ok_or_else(|| invalid_runtime_path("cwd has no runs component"))?;
    if components.get(runs + 2) != Some(&"tasks") || components.get(runs + 4) != Some(&"attempts") {
        return Err(invalid_runtime_path("cwd has invalid attempt layout"));
    }
    Ok((
        RunId::from_str(components[runs + 1]).map_err(RuntimeError::from_contract)?,
        TaskId::from_str(components[runs + 3]).map_err(RuntimeError::from_contract)?,
        AttemptId::from_str(components[runs + 5]).map_err(RuntimeError::from_contract)?,
    ))
}

fn invalid_runtime_path(message: &'static str) -> RuntimeError {
    RuntimeError::from_contract(harp_contracts::ContractError::new("cwd", message))
}

fn write_fake_scratch(cwd: &str) -> Result<(), RuntimeError> {
    fs::write(Path::new(cwd).join("notes.md"), b"runtime notes")
        .map_err(|error| runtime_io("write fake notes", error))?;
    fs::write(
        Path::new(cwd).join("result.json"),
        b"runtime scratch result",
    )
    .map_err(|error| runtime_io("write fake scratch result", error))?;
    fs::write(Path::new(cwd).join("tmp.bin"), b"runtime temporary bytes")
        .map_err(|error| runtime_io("write fake unknown scratch file", error))?;
    fs::write(
        Path::new(cwd).join("evidence.jsonl"),
        b"{\"claim\":\"runtime evidence\"}\n",
    )
    .map_err(|error| runtime_io("write fake evidence", error))?;
    let (run_id, task_id, attempt_id) = attempt_identity_from_cwd(cwd)?;
    let checkpoint = Checkpoint::new(run_id, task_id, attempt_id, "runtime-checkpoint", 1)
        .map_err(RuntimeError::from_contract)?;
    let checkpoint_bytes = serde_json::to_vec(&checkpoint)
        .map_err(|error| runtime_io("encode fake checkpoint", io::Error::other(error)))?;
    let checkpoint_digest = format!("{:x}", sha2::Sha256::digest(&checkpoint_bytes));
    let checkpoint_dir = Path::new(cwd).join("checkpoints");
    fs::create_dir(&checkpoint_dir)
        .map_err(|error| runtime_io("create fake checkpoint directory", error))?;
    fs::write(
        checkpoint_dir.join(format!("{:016x}-{checkpoint_digest}.json", 1_u64 << 63 | 1)),
        checkpoint_bytes,
    )
    .map_err(|error| runtime_io("write fake checkpoint", error))
}

fn runtime_io(context: &str, source: io::Error) -> RuntimeError {
    RuntimeError::with_source(RuntimeErrorKind::Transport, context, true, source)
        .unwrap_or_else(RuntimeError::from_contract)
}

fn task_id_from_instruction(instruction: &str) -> Result<String, RuntimeError> {
    let value: serde_json::Value = serde_json::from_str(instruction).map_err(|error| {
        RuntimeError::from_contract(harp_contracts::ContractError::new(
            "instruction",
            if error.is_syntax() {
                "projected instruction is invalid JSON"
            } else {
                "projected instruction could not be decoded"
            },
        ))
    })?;
    value
        .get("taskId")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            RuntimeError::from_contract(harp_contracts::ContractError::new(
                "instruction.taskId",
                "missing task identity",
            ))
        })
}

fn snapshot(
    backend: &PersistentFakeBackend,
    thread: &ThreadHandle,
) -> Result<ThreadSnapshot, RuntimeError> {
    let state = backend.lock();
    snapshot_state(&state, thread)
}

fn snapshot_state(
    state: &FakeState,
    thread: &ThreadHandle,
) -> Result<ThreadSnapshot, RuntimeError> {
    let thread_state = state
        .threads
        .get(&thread.thread_id.to_string())
        .ok_or_else(|| {
            RuntimeError::from_contract(harp_contracts::ContractError::new(
                "thread",
                "fake thread does not exist",
            ))
        })?;
    Ok(ThreadSnapshot {
        thread_id: thread_state.handle.thread_id.clone(),
        status: if thread_state
            .turns
            .iter()
            .any(|turn| turn.status == TurnStatus::InProgress)
        {
            ThreadStatus::Active
        } else {
            ThreadStatus::Idle
        },
        turns: thread_state
            .turns
            .iter()
            .map(|turn| TurnSnapshot {
                turn_id: turn.handle.turn_id.clone(),
                operation_marker: Some(turn.marker.clone()),
                status: turn.status,
                final_agent_message: turn.final_message.clone(),
            })
            .collect(),
    })
}

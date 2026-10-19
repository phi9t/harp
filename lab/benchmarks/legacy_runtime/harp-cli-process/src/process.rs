use std::collections::{BTreeMap, HashMap, VecDeque};
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use harp_contracts::{
    ExternalSessionId, RuntimeErrorKind, RuntimeEvent, ThreadHandle, ThreadId, ThreadSnapshot,
    ThreadSpec, ThreadStartedEvent, ThreadStatus, TokenUsage, TokenUsageEvent, TurnCompletedEvent,
    TurnHandle, TurnId, TurnSnapshot, TurnSpec, TurnStartedEvent, TurnStatus,
};
use harp_runtime::{
    ActivityHandle, ActivityRuntime, ActivitySpec, InterruptPurpose, InterruptReceipt,
    RuntimeControl, RuntimeError, RuntimeProvenance,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

const MAX_ARG_BYTES: usize = 64 * 1024;
const MAX_ARGV_BYTES: usize = 128 * 1024;
const MAX_ENV_VALUE_BYTES: usize = 16 * 1024;
const MAX_JSONL_LINE_BYTES: usize = 1024 * 1024;
const MAX_FINAL_MESSAGE_BYTES: usize = 1024 * 1024;
const MAX_VERSION_BYTES: usize = 64 * 1024;
const MAX_EXECUTABLE_HASH_BYTES: u64 = 512 * 1024 * 1024;
const THREAD_STARTED_TIMEOUT: Duration = Duration::from_secs(60);
const PROCESS_DIR: &str = "harp-cli-process";
const RECORDS_DIR: &str = "records";
const SPOOLS_DIR: &str = "spools";

#[derive(Clone, Debug)]
pub struct ProcessRuntimeConfig {
    executable: PathBuf,
    exec_args: Vec<OsString>,
    version_args: Vec<OsString>,
    codex_home: PathBuf,
    env: BTreeMap<OsString, OsString>,
}

impl ProcessRuntimeConfig {
    pub fn builder(
        executable: impl Into<PathBuf>,
        codex_home: impl Into<PathBuf>,
    ) -> ProcessRuntimeConfigBuilder {
        ProcessRuntimeConfigBuilder {
            config: Self {
                executable: executable.into(),
                exec_args: vec![OsString::from("exec"), OsString::from("--json")],
                version_args: vec![OsString::from("--version")],
                codex_home: codex_home.into(),
                env: BTreeMap::new(),
            },
        }
    }

    pub fn with_env(
        mut self,
        key: impl Into<OsString>,
        value: impl Into<OsString>,
    ) -> Result<Self, RuntimeError> {
        insert_env(&mut self.env, key.into(), value.into())?;
        Ok(self)
    }
}

pub struct ProcessRuntimeConfigBuilder {
    config: ProcessRuntimeConfig,
}

impl ProcessRuntimeConfigBuilder {
    pub fn exec_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.config.exec_args = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn version_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.config.version_args = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.config.env.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> Result<ProcessRuntimeConfig, RuntimeError> {
        validate_config(&self.config)?;
        Ok(self.config)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RuntimeManifest {
    schema_version: u32,
    executable_path: String,
    executable_sha256: String,
    exec_argv: Vec<String>,
    version_argv: Vec<String>,
    version_output: String,
    codex_home_path: String,
    environment_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ProcessRecord {
    schema_version: u32,
    pid: u32,
    started_unix_millis: u128,
    executable_path: String,
    argv: Vec<String>,
    codex_home_path: String,
    activity_dir: String,
    logical_session_id: ThreadId,
    logical_turn_id: TurnId,
    invocation_sha256: String,
    stdout_path: String,
    stderr_path: String,
    output_schema_path: String,
    final_message_path: String,
    environment_sha256: String,
}

#[derive(Debug)]
struct ActivityProcess {
    record: ProcessRecord,
    child: Option<Child>,
    offset: u64,
    final_message: Option<String>,
    queued: VecDeque<RuntimeEvent>,
    terminal_seen: bool,
}

#[derive(Debug)]
struct RuntimeInner {
    config: ProcessRuntimeConfig,
    provenance: RuntimeProvenance,
    codex_home: PathBuf,
    environment: BTreeMap<OsString, OsString>,
    activities: HashMap<String, ActivityProcess>,
}

pub struct CliProcessRuntime {
    inner: Arc<Mutex<RuntimeInner>>,
}

impl CliProcessRuntime {
    pub async fn spawn(config: ProcessRuntimeConfig) -> Result<Self, RuntimeError> {
        let prepared = prepare_runtime(config)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(prepared)),
        })
    }
}

#[async_trait]
impl ActivityRuntime for CliProcessRuntime {
    fn provenance(&self) -> Result<RuntimeProvenance, RuntimeError> {
        Ok(lock_inner(&self.inner)?.provenance.clone())
    }

    fn control_handle(&self) -> Result<Arc<dyn RuntimeControl>, RuntimeError> {
        Ok(Arc::new(CliProcessControl {
            inner: Arc::clone(&self.inner),
        }))
    }

    async fn start_logical_session(
        &mut self,
        spec: ThreadSpec,
    ) -> Result<ThreadHandle, RuntimeError> {
        spec.validate().map_err(RuntimeError::from_contract)?;
        if let Some(authority) = &spec.workspace_authority {
            harp_artifacts::verify_runtime_workspace_authority(authority).map_err(|error| {
                RuntimeError::with_source(
                    RuntimeErrorKind::Protocol,
                    "runtime workspace authority verification failed",
                    false,
                    error,
                )
                .unwrap_or_else(RuntimeError::from_contract)
            })?;
        }
        Ok(ThreadHandle {
            thread_id: ThreadId::from_str_like(&spec.cwd)?,
        })
    }

    async fn start_activity(&mut self, spec: ActivitySpec) -> Result<ActivityHandle, RuntimeError> {
        spec.validate()?;
        let (config, environment, codex_home) = {
            let inner = lock_inner(&self.inner)?;
            (
                inner.config.clone(),
                inner.environment.clone(),
                inner.codex_home.clone(),
            )
        };
        verify_activity_dir(&spec.activity_dir)?;
        let activity_dir = PathBuf::from(&spec.activity_dir);
        let spool_root = ensure_private_dir(&codex_home.join(PROCESS_DIR).join(SPOOLS_DIR))?;
        let spool_dir = unique_spool_dir(
            &spool_root,
            &spec.logical_session_id,
            &spec.logical_turn_id,
            &spec.invocation_sha256,
        )?;
        let stdout_path = spool_dir.join("stdout.jsonl");
        let stderr_path = spool_dir.join("stderr.log");
        let output_schema_path = spool_dir.join("output-schema.json");
        let final_message_path = spool_dir.join("final-message.txt");
        let output_schema_bytes =
            serde_json::to_vec(&spec.turn_spec.output_schema).map_err(|source| {
                runtime_with_source(
                    RuntimeErrorKind::Protocol,
                    "serialize output schema",
                    false,
                    source,
                )
            })?;
        write_new_file(&output_schema_path, &output_schema_bytes)?;
        write_new_file(&stdout_path, b"")?;
        write_new_file(&stderr_path, b"")?;

        let argv = build_activity_argv(
            &config.exec_args,
            &spec,
            &output_schema_path,
            &final_message_path,
        )?;
        let mut command = Command::new(&config.executable);
        command
            .args(&argv)
            .current_dir(&activity_dir)
            .env_clear()
            .envs(&environment)
            .stdin(Stdio::piped())
            .stdout(Stdio::from(open_append(&stdout_path)?))
            .stderr(Stdio::from(open_append(&stderr_path)?));
        unsafe {
            command.pre_exec(|| {
                if libc::setpgid(0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut child = command.spawn().map_err(|source| {
            runtime_with_source(
                RuntimeErrorKind::Startup,
                "spawn CLI activity",
                true,
                source,
            )
        })?;
        if let Some(mut stdin) = child.stdin.take() {
            let prompt = activity_prompt(&spec.thread_spec, &spec.turn_spec)?;
            stdin.write_all(prompt.as_bytes()).await.map_err(|source| {
                runtime_with_source(
                    RuntimeErrorKind::Transport,
                    "write CLI prompt",
                    true,
                    source,
                )
            })?;
            stdin.shutdown().await.map_err(|source| {
                runtime_with_source(
                    RuntimeErrorKind::Transport,
                    "close CLI prompt",
                    true,
                    source,
                )
            })?;
        }
        let pid = child.id().ok_or_else(|| {
            runtime_error(
                RuntimeErrorKind::Startup,
                "spawned CLI process omitted pid",
                true,
            )
        })?;
        let record = ProcessRecord {
            schema_version: 1,
            pid,
            started_unix_millis: unix_millis()?,
            executable_path: utf8_path(&config.executable, "executable")?,
            argv: utf8_args(&argv, "activity argv")?,
            codex_home_path: utf8_path(&codex_home, "CODEX_HOME")?,
            activity_dir: spec.activity_dir.clone(),
            logical_session_id: spec.logical_session_id.clone(),
            logical_turn_id: spec.logical_turn_id.clone(),
            invocation_sha256: spec.invocation_sha256.clone(),
            stdout_path: utf8_path(&stdout_path, "stdout spool")?,
            stderr_path: utf8_path(&stderr_path, "stderr spool")?,
            output_schema_path: utf8_path(&output_schema_path, "output schema")?,
            final_message_path: utf8_path(&final_message_path, "final message")?,
            environment_sha256: hash_json(&utf8_env(&environment)?)?,
        };
        let process_record_sha256 = persist_process_record(&codex_home, &record)?;
        let process = ActivityProcess {
            record,
            child: Some(child),
            offset: 0,
            final_message: None,
            queued: VecDeque::new(),
            terminal_seen: false,
        };
        {
            let mut inner = lock_inner(&self.inner)?;
            inner
                .activities
                .insert(process_record_sha256.clone(), process);
        }
        let external_session_id = {
            let mut inner = lock_inner(&self.inner)?;
            let process = inner
                .activities
                .get_mut(&process_record_sha256)
                .ok_or_else(|| {
                    runtime_error(
                        RuntimeErrorKind::NotFound,
                        "process record is not loaded",
                        false,
                    )
                })?;
            wait_for_thread_started(process, THREAD_STARTED_TIMEOUT)?
        };
        let handle = ActivityHandle {
            logical_session_id: spec.logical_session_id,
            logical_turn_id: spec.logical_turn_id,
            process_record_sha256,
            external_session_id: Some(external_session_id),
        };
        handle.validate()?;
        Ok(handle)
    }

    async fn next_event(
        &mut self,
        activity: &ActivityHandle,
        max_wait: Duration,
    ) -> Result<RuntimeEvent, RuntimeError> {
        activity.validate()?;
        if max_wait.is_zero() || max_wait > Duration::from_secs(10 * 60) {
            return Err(runtime_error(
                RuntimeErrorKind::Protocol,
                "max_wait must be nonzero and at most 10 minutes",
                false,
            ));
        }
        let deadline = std::time::Instant::now() + max_wait;
        loop {
            {
                let mut inner = lock_inner(&self.inner)?;
                ensure_activity_loaded(&mut inner, activity)?;
                let process = inner
                    .activities
                    .get_mut(&activity.process_record_sha256)
                    .ok_or_else(|| {
                        runtime_error(
                            RuntimeErrorKind::NotFound,
                            "process record is not loaded",
                            false,
                        )
                    })?;
                if let Some(event) = next_process_event(process, activity)? {
                    return Ok(event);
                }
                if process_exited(process)? {
                    return Err(runtime_error(
                        RuntimeErrorKind::Disconnected,
                        "CLI process exited before a terminal event was observed",
                        true,
                    ));
                }
            }
            if std::time::Instant::now() >= deadline {
                return Err(runtime_error(
                    RuntimeErrorKind::Transport,
                    "runtime event timeout",
                    true,
                ));
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    async fn interrupt(
        &mut self,
        activity: &ActivityHandle,
        purpose: InterruptPurpose,
    ) -> Result<InterruptReceipt, RuntimeError> {
        CliProcessControl {
            inner: Arc::clone(&self.inner),
        }
        .interrupt_activity(activity, purpose)
        .await
    }
}

struct CliProcessControl {
    inner: Arc<Mutex<RuntimeInner>>,
}

#[async_trait]
impl RuntimeControl for CliProcessControl {
    fn provenance(&self) -> Result<RuntimeProvenance, RuntimeError> {
        Ok(lock_inner(&self.inner)?.provenance.clone())
    }

    async fn read_thread(&self, thread: &ThreadHandle) -> Result<ThreadSnapshot, RuntimeError> {
        thread.validate().map_err(RuntimeError::from_contract)?;
        Ok(ThreadSnapshot {
            thread_id: thread.thread_id.clone(),
            status: ThreadStatus::NotLoaded,
            turns: Vec::new(),
        })
    }

    async fn interrupt(&self, turn: &TurnHandle) -> Result<(), RuntimeError> {
        turn.validate().map_err(RuntimeError::from_contract)?;
        Err(runtime_error(
            RuntimeErrorKind::NotFound,
            "legacy turn interruption is not supported by the CLI process runtime",
            false,
        ))
    }

    async fn interrupt_activity(
        &self,
        activity: &ActivityHandle,
        _purpose: InterruptPurpose,
    ) -> Result<InterruptReceipt, RuntimeError> {
        activity.validate()?;
        let pid = {
            let mut inner = lock_inner(&self.inner)?;
            ensure_activity_loaded(&mut inner, activity)?;
            inner
                .activities
                .get(&activity.process_record_sha256)
                .ok_or_else(|| {
                    runtime_error(
                        RuntimeErrorKind::NotFound,
                        "process record is not loaded",
                        false,
                    )
                })?
                .record
                .pid
        };
        signal_process_group(pid, libc::SIGINT)?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        let quiescent = if process_alive(pid) {
            signal_process_group(pid, libc::SIGTERM)?;
            tokio::time::sleep(Duration::from_millis(250)).await;
            if process_alive(pid) {
                signal_process_group(pid, libc::SIGKILL)?;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            !process_alive(pid)
        } else {
            true
        };
        let receipt = InterruptReceipt {
            process_record_sha256: activity.process_record_sha256.clone(),
            quiescent,
        };
        receipt.validate()?;
        Ok(receipt)
    }
}

fn prepare_runtime(config: ProcessRuntimeConfig) -> Result<RuntimeInner, RuntimeError> {
    validate_config(&config)?;
    let executable = canonical_regular_file(&config.executable, "executable")?;
    let executable_sha256 = hash_file(&executable, MAX_EXECUTABLE_HASH_BYTES)?;
    let codex_home = secure_codex_home(&config.codex_home)?;
    ensure_private_dir(&codex_home.join(PROCESS_DIR))?;
    ensure_private_dir(&codex_home.join(PROCESS_DIR).join(RECORDS_DIR))?;
    ensure_private_dir(&codex_home.join(PROCESS_DIR).join(SPOOLS_DIR))?;
    let mut environment = config.env.clone();
    environment.insert(
        OsString::from("CODEX_HOME"),
        codex_home.as_os_str().to_owned(),
    );
    environment
        .entry(OsString::from("HOME"))
        .or_insert_with(|| codex_home.as_os_str().to_owned());
    let version_output = bounded_command_output(&executable, &config.version_args, &environment)?;
    let manifest = RuntimeManifest {
        schema_version: 1,
        executable_path: utf8_path(&executable, "executable")?,
        executable_sha256,
        exec_argv: utf8_args(&config.exec_args, "exec argv")?,
        version_argv: utf8_args(&config.version_args, "version argv")?,
        version_output,
        codex_home_path: utf8_path(&codex_home, "CODEX_HOME")?,
        environment_sha256: hash_json(&utf8_env(&environment)?)?,
    };
    let config_sha256 = hash_json(&manifest)?;
    let metadata = fs::metadata(&codex_home).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            "inspect CODEX_HOME",
            false,
            source,
        )
    })?;
    let persistent_state_identity = format!(
        "codex-home:{}:{}:{}",
        metadata.dev(),
        metadata.ino(),
        &config_sha256[..16]
    );
    let provenance = RuntimeProvenance::new(
        "codex-cli-process",
        env!("CARGO_PKG_VERSION"),
        &persistent_state_identity,
        &config_sha256,
    )
    .map_err(RuntimeError::from_contract)?;
    Ok(RuntimeInner {
        config: ProcessRuntimeConfig {
            executable,
            exec_args: config.exec_args,
            version_args: config.version_args,
            codex_home: codex_home.clone(),
            env: config.env,
        },
        provenance,
        codex_home,
        environment,
        activities: HashMap::new(),
    })
}

fn wait_for_thread_started(
    process: &mut ActivityProcess,
    max_wait: Duration,
) -> Result<ExternalSessionId, RuntimeError> {
    let deadline = std::time::Instant::now() + max_wait;
    loop {
        if let Some(event) = next_process_event_internal(process)? {
            match event {
                ParsedProcessEvent::ThreadStarted(external) => {
                    process
                        .queued
                        .push_back(RuntimeEvent::ThreadStarted(ThreadStartedEvent {
                            thread_id: process.record.logical_session_id.clone(),
                        }));
                    return Ok(external);
                }
                ParsedProcessEvent::Runtime(event) => {
                    process.queued.push_back(event);
                    return Err(runtime_error(
                        RuntimeErrorKind::Protocol,
                        "CLI emitted activity events before thread.started",
                        false,
                    ));
                }
                ParsedProcessEvent::NoRuntimeEvent => {}
            }
        }
        if process_exited(process)? {
            return Err(runtime_error(
                RuntimeErrorKind::Protocol,
                "CLI exited before thread.started",
                false,
            ));
        }
        if std::time::Instant::now() >= deadline {
            return Err(runtime_error(
                RuntimeErrorKind::Transport,
                "timed out waiting for thread.started",
                true,
            ));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn next_process_event(
    process: &mut ActivityProcess,
    handle: &ActivityHandle,
) -> Result<Option<RuntimeEvent>, RuntimeError> {
    if process.record.logical_session_id != handle.logical_session_id
        || process.record.logical_turn_id != handle.logical_turn_id
    {
        return Err(runtime_error(
            RuntimeErrorKind::Protocol,
            "activity handle does not match process record identity",
            false,
        ));
    }
    if let Some(event) = process.queued.pop_front() {
        return Ok(Some(event));
    }
    loop {
        let Some(parsed) = next_process_event_internal(process)? else {
            return Ok(None);
        };
        match parsed {
            ParsedProcessEvent::ThreadStarted(_) => {
                return Ok(Some(RuntimeEvent::ThreadStarted(ThreadStartedEvent {
                    thread_id: process.record.logical_session_id.clone(),
                })));
            }
            ParsedProcessEvent::Runtime(event) => return Ok(Some(event)),
            ParsedProcessEvent::NoRuntimeEvent => continue,
        }
    }
}

enum ParsedProcessEvent {
    ThreadStarted(ExternalSessionId),
    Runtime(RuntimeEvent),
    NoRuntimeEvent,
}

fn next_process_event_internal(
    process: &mut ActivityProcess,
) -> Result<Option<ParsedProcessEvent>, RuntimeError> {
    let Some(line) = read_next_jsonl_line(process)? else {
        return Ok(None);
    };
    let value: Value = serde_json::from_str(&line).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Protocol,
            "parse CLI JSONL event",
            false,
            source,
        )
    })?;
    let event_type = value.get("type").and_then(Value::as_str).ok_or_else(|| {
        runtime_error(
            RuntimeErrorKind::Protocol,
            "CLI JSONL event omitted type",
            false,
        )
    })?;
    match event_type {
        "thread.started" => {
            let thread_id = value
                .get("thread_id")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    runtime_error(
                        RuntimeErrorKind::Protocol,
                        "thread.started omitted thread_id",
                        false,
                    )
                })?;
            Ok(Some(ParsedProcessEvent::ThreadStarted(
                thread_id.parse().map_err(RuntimeError::from_contract)?,
            )))
        }
        "turn.started" => Ok(Some(ParsedProcessEvent::Runtime(
            RuntimeEvent::TurnStarted(TurnStartedEvent {
                thread_id: process.record.logical_session_id.clone(),
                turn_id: process.record.logical_turn_id.clone(),
            }),
        ))),
        "item.completed" => {
            if let Some(item) = value.get("item") {
                if item.get("type").and_then(Value::as_str) == Some("agent_message") {
                    if let Some(text) = item.get("text").and_then(Value::as_str) {
                        if text.len() > MAX_FINAL_MESSAGE_BYTES {
                            return Err(runtime_error(
                                RuntimeErrorKind::OutputSchema,
                                "final agent message exceeds byte cap",
                                false,
                            ));
                        }
                        validate_final_message(text)?;
                        process.final_message = Some(text.to_owned());
                    }
                }
            }
            Ok(Some(ParsedProcessEvent::NoRuntimeEvent))
        }
        "turn.completed" => {
            process.terminal_seen = true;
            if let Some(usage) = parse_usage(&value)? {
                process
                    .queued
                    .push_back(RuntimeEvent::TurnCompleted(TurnCompletedEvent {
                        thread_id: process.record.logical_session_id.clone(),
                        turn: TurnSnapshot {
                            turn_id: process.record.logical_turn_id.clone(),
                            operation_marker: None,
                            status: TurnStatus::Completed,
                            final_agent_message: process.final_message.clone(),
                        },
                    }));
                Ok(Some(ParsedProcessEvent::Runtime(RuntimeEvent::TokenUsage(
                    TokenUsageEvent {
                        thread_id: process.record.logical_session_id.clone(),
                        turn_id: process.record.logical_turn_id.clone(),
                        usage,
                    },
                ))))
            } else {
                Ok(Some(ParsedProcessEvent::Runtime(
                    RuntimeEvent::TurnCompleted(TurnCompletedEvent {
                        thread_id: process.record.logical_session_id.clone(),
                        turn: TurnSnapshot {
                            turn_id: process.record.logical_turn_id.clone(),
                            operation_marker: None,
                            status: TurnStatus::Completed,
                            final_agent_message: process.final_message.clone(),
                        },
                    }),
                )))
            }
        }
        "error" => {
            let message = value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("CLI emitted error event");
            Err(runtime_error(RuntimeErrorKind::Transport, message, true))
        }
        _ => Ok(Some(ParsedProcessEvent::NoRuntimeEvent)),
    }
}

fn read_next_jsonl_line(process: &mut ActivityProcess) -> Result<Option<String>, RuntimeError> {
    let path = Path::new(&process.record.stdout_path);
    let mut file = OpenOptions::new().read(true).open(path).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Transport,
            "open stdout spool",
            true,
            source,
        )
    })?;
    file.seek(SeekFrom::Start(process.offset))
        .map_err(|source| {
            runtime_with_source(
                RuntimeErrorKind::Transport,
                "seek stdout spool",
                true,
                source,
            )
        })?;
    let mut bytes = Vec::new();
    let mut single = [0_u8; 1];
    while bytes.len() <= MAX_JSONL_LINE_BYTES {
        match file.read(&mut single) {
            Ok(0) => return Ok(None),
            Ok(_) => {
                process.offset += 1;
                if single[0] == b'\n' {
                    return String::from_utf8(bytes).map(Some).map_err(|source| {
                        runtime_with_source(
                            RuntimeErrorKind::Protocol,
                            "decode UTF-8 JSONL",
                            false,
                            source,
                        )
                    });
                }
                bytes.push(single[0]);
            }
            Err(source) => {
                return Err(runtime_with_source(
                    RuntimeErrorKind::Transport,
                    "read stdout spool",
                    true,
                    source,
                ))
            }
        }
    }
    Err(runtime_error(
        RuntimeErrorKind::Protocol,
        "CLI JSONL line exceeds byte cap",
        false,
    ))
}

fn parse_usage(value: &Value) -> Result<Option<TokenUsage>, RuntimeError> {
    let Some(usage) = value.get("usage") else {
        return Ok(None);
    };
    let input_tokens = usage
        .get("input_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let cached_input_tokens = usage
        .get("cached_input_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let output_tokens = usage
        .get("output_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let reasoning_output_tokens = usage
        .get("reasoning_output_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total_tokens = input_tokens.checked_add(output_tokens).ok_or_else(|| {
        runtime_error(RuntimeErrorKind::Protocol, "token usage overflowed", false)
    })?;
    let usage = TokenUsage {
        total_tokens,
        input_tokens,
        cached_input_tokens,
        output_tokens,
        reasoning_output_tokens,
    };
    usage.validate().map_err(RuntimeError::from_contract)?;
    Ok(Some(usage))
}

fn validate_final_message(text: &str) -> Result<(), RuntimeError> {
    let value: Value = serde_json::from_str(text).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::OutputSchema,
            "final agent message is not JSON",
            false,
            source,
        )
    })?;
    if !value.is_object() {
        return Err(runtime_error(
            RuntimeErrorKind::OutputSchema,
            "final agent message must be a JSON object",
            false,
        ));
    }
    Ok(())
}

fn process_exited(process: &mut ActivityProcess) -> Result<bool, RuntimeError> {
    if process.terminal_seen {
        return Ok(false);
    }
    if let Some(child) = process.child.as_mut() {
        if let Some(status) = child.try_wait().map_err(|source| {
            runtime_with_source(
                RuntimeErrorKind::Transport,
                "poll CLI process",
                true,
                source,
            )
        })? {
            if !status.success() {
                return Err(runtime_error(
                    RuntimeErrorKind::Transport,
                    format!("CLI process exited with status {status}"),
                    true,
                ));
            }
            return Ok(true);
        }
        return Ok(false);
    }
    Ok(!process_alive(process.record.pid))
}

fn ensure_activity_loaded(
    inner: &mut RuntimeInner,
    activity: &ActivityHandle,
) -> Result<(), RuntimeError> {
    if inner
        .activities
        .contains_key(&activity.process_record_sha256)
    {
        return Ok(());
    }
    let record = load_process_record(&inner.codex_home, &activity.process_record_sha256)?;
    if record.logical_session_id != activity.logical_session_id
        || record.logical_turn_id != activity.logical_turn_id
    {
        return Err(runtime_error(
            RuntimeErrorKind::Protocol,
            "process record identity does not match activity handle",
            false,
        ));
    }
    inner.activities.insert(
        activity.process_record_sha256.clone(),
        ActivityProcess {
            record,
            child: None,
            offset: 0,
            final_message: None,
            queued: VecDeque::new(),
            terminal_seen: false,
        },
    );
    Ok(())
}

fn build_activity_argv(
    base_args: &[OsString],
    spec: &ActivitySpec,
    output_schema_path: &Path,
    final_message_path: &Path,
) -> Result<Vec<OsString>, RuntimeError> {
    let mut args = base_args.to_vec();
    if let Some(external_session_id) = &spec.external_session_id {
        insert_resume_args(&mut args, external_session_id);
    }
    args.extend([
        OsString::from("--cd"),
        OsString::from(&spec.activity_dir),
        OsString::from("--sandbox"),
        OsString::from(&spec.thread_spec.sandbox_mode),
        OsString::from("--model"),
        OsString::from(
            spec.turn_spec
                .model
                .as_deref()
                .unwrap_or(&spec.thread_spec.model),
        ),
        OsString::from("--output-schema"),
        output_schema_path.as_os_str().to_owned(),
        OsString::from("--output-last-message"),
        final_message_path.as_os_str().to_owned(),
        OsString::from("--skip-git-repo-check"),
    ]);
    if spec.thread_spec.ephemeral {
        args.push(OsString::from("--ephemeral"));
    }
    validate_argv(&args, "activity argv")?;
    Ok(args)
}

fn insert_resume_args(args: &mut Vec<OsString>, external_session_id: &ExternalSessionId) {
    let resume = OsString::from("resume");
    let session = OsString::from(external_session_id.to_string());
    if args.first().and_then(|arg| arg.to_str()) == Some("exec") {
        args.insert(1, resume);
        args.insert(2, session);
    } else {
        args.insert(0, session);
        args.insert(0, resume);
    }
}

fn activity_prompt(thread: &ThreadSpec, turn: &TurnSpec) -> Result<String, RuntimeError> {
    let mut prompt = String::new();
    if !thread.base_instructions.is_empty() {
        prompt.push_str("Base instructions:\n");
        prompt.push_str(&thread.base_instructions);
        prompt.push_str("\n\n");
    }
    if !thread.developer_instructions.is_empty() {
        prompt.push_str("Developer instructions:\n");
        prompt.push_str(&thread.developer_instructions);
        prompt.push_str("\n\n");
    }
    prompt.push_str(&turn.instruction);
    if prompt.len() > 384 * 1024 {
        return Err(runtime_error(
            RuntimeErrorKind::Protocol,
            "composed CLI prompt exceeds byte cap",
            false,
        ));
    }
    Ok(prompt)
}

fn persist_process_record(
    codex_home: &Path,
    record: &ProcessRecord,
) -> Result<String, RuntimeError> {
    let bytes = serde_json::to_vec(record).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Protocol,
            "serialize process record",
            false,
            source,
        )
    })?;
    let digest = hash_bytes(&bytes);
    let records_dir = ensure_private_dir(&codex_home.join(PROCESS_DIR).join(RECORDS_DIR))?;
    let path = records_dir.join(format!("{digest}.json"));
    write_new_file(&path, &bytes)?;
    Ok(digest)
}

fn load_process_record(codex_home: &Path, digest: &str) -> Result<ProcessRecord, RuntimeError> {
    validate_sha256(digest)?;
    let path = codex_home
        .join(PROCESS_DIR)
        .join(RECORDS_DIR)
        .join(format!("{digest}.json"));
    let bytes = fs::read(&path).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::NotFound,
            "read process record",
            false,
            source,
        )
    })?;
    if hash_bytes(&bytes) != digest {
        return Err(runtime_error(
            RuntimeErrorKind::Protocol,
            "process record digest mismatch",
            false,
        ));
    }
    serde_json::from_slice(&bytes).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Protocol,
            "parse process record",
            false,
            source,
        )
    })
}

fn unique_spool_dir(
    spool_root: &Path,
    logical_session_id: &ThreadId,
    logical_turn_id: &TurnId,
    invocation_sha256: &str,
) -> Result<PathBuf, RuntimeError> {
    let prefix = format!(
        "{}-{}-{}",
        safe_component(&logical_session_id.to_string()),
        safe_component(&logical_turn_id.to_string()),
        &invocation_sha256[..16]
    );
    for index in 0..1000_u32 {
        let candidate = spool_root.join(format!("{prefix}-{index}"));
        match fs::create_dir(&candidate) {
            Ok(()) => {
                fs::set_permissions(&candidate, fs::Permissions::from_mode(0o700)).map_err(
                    |source| {
                        runtime_with_source(
                            RuntimeErrorKind::Startup,
                            "set spool mode",
                            false,
                            source,
                        )
                    },
                )?;
                return Ok(candidate);
            }
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(runtime_with_source(
                    RuntimeErrorKind::Startup,
                    "create activity spool",
                    false,
                    source,
                ))
            }
        }
    }
    Err(runtime_error(
        RuntimeErrorKind::Startup,
        "could not allocate activity spool directory",
        false,
    ))
}

fn safe_component(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_') {
                byte as char
            } else {
                '_'
            }
        })
        .collect()
}

fn validate_config(config: &ProcessRuntimeConfig) -> Result<(), RuntimeError> {
    if !config.executable.is_absolute() {
        return Err(runtime_error(
            RuntimeErrorKind::Startup,
            "executable path must be absolute",
            false,
        ));
    }
    if !config.codex_home.is_absolute() {
        return Err(runtime_error(
            RuntimeErrorKind::Startup,
            "CODEX_HOME path must be absolute",
            false,
        ));
    }
    validate_argv(&config.exec_args, "exec argv")?;
    validate_argv(&config.version_args, "version argv")?;
    for (key, value) in &config.env {
        validate_env(key, value)?;
    }
    Ok(())
}

fn validate_argv(args: &[OsString], label: &str) -> Result<(), RuntimeError> {
    let mut aggregate_bytes = 0_usize;
    for argument in args {
        let argument = argument.to_str().ok_or_else(|| {
            runtime_error(
                RuntimeErrorKind::Startup,
                format!("{label} must contain only UTF-8 arguments"),
                false,
            )
        })?;
        if argument.len() > MAX_ARG_BYTES {
            return Err(runtime_error(
                RuntimeErrorKind::Startup,
                format!("{label} argument exceeds {MAX_ARG_BYTES} bytes"),
                false,
            ));
        }
        if argument.chars().any(char::is_control) {
            return Err(runtime_error(
                RuntimeErrorKind::Startup,
                format!("{label} argument contains a control character"),
                false,
            ));
        }
        aggregate_bytes = aggregate_bytes.checked_add(argument.len()).ok_or_else(|| {
            runtime_error(
                RuntimeErrorKind::Startup,
                format!("{label} byte accounting overflowed"),
                false,
            )
        })?;
    }
    if aggregate_bytes > MAX_ARGV_BYTES {
        return Err(runtime_error(
            RuntimeErrorKind::Startup,
            format!("{label} exceeds {MAX_ARGV_BYTES} aggregate bytes"),
            false,
        ));
    }
    Ok(())
}

fn insert_env(
    env: &mut BTreeMap<OsString, OsString>,
    key: OsString,
    value: OsString,
) -> Result<(), RuntimeError> {
    validate_env(&key, &value)?;
    env.insert(key, value);
    Ok(())
}

fn validate_env(key: &OsStr, value: &OsStr) -> Result<(), RuntimeError> {
    let Some(key) = key.to_str() else {
        return Err(runtime_error(
            RuntimeErrorKind::Startup,
            "environment key must be UTF-8",
            false,
        ));
    };
    let Some(value) = value.to_str() else {
        return Err(runtime_error(
            RuntimeErrorKind::Startup,
            "environment value must be UTF-8",
            false,
        ));
    };
    if key.is_empty()
        || key.len() > 256
        || key.contains('=')
        || key == "CODEX_HOME"
        || key.chars().any(char::is_control)
        || value.len() > MAX_ENV_VALUE_BYTES
        || value.contains('\0')
    {
        return Err(runtime_error(
            RuntimeErrorKind::Startup,
            "invalid bounded process environment entry",
            false,
        ));
    }
    Ok(())
}

fn secure_codex_home(path: &Path) -> Result<PathBuf, RuntimeError> {
    if path.exists() {
        let metadata = fs::symlink_metadata(path).map_err(|source| {
            runtime_with_source(
                RuntimeErrorKind::Startup,
                "inspect CODEX_HOME",
                false,
                source,
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(runtime_error(
                RuntimeErrorKind::Startup,
                "CODEX_HOME must be a non-symlink directory",
                false,
            ));
        }
    } else {
        fs::create_dir_all(path).map_err(|source| {
            runtime_with_source(
                RuntimeErrorKind::Startup,
                "create CODEX_HOME",
                false,
                source,
            )
        })?;
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            "set CODEX_HOME mode",
            false,
            source,
        )
    })?;
    path.canonicalize().map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            "canonicalize CODEX_HOME",
            false,
            source,
        )
    })
}

fn ensure_private_dir(path: &Path) -> Result<PathBuf, RuntimeError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(runtime_error(
                    RuntimeErrorKind::Startup,
                    "runtime path must be a non-symlink directory",
                    false,
                ));
            }
        }
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(path).map_err(|source| {
                runtime_with_source(
                    RuntimeErrorKind::Startup,
                    "create runtime directory",
                    false,
                    source,
                )
            })?;
        }
        Err(source) => {
            return Err(runtime_with_source(
                RuntimeErrorKind::Startup,
                "inspect runtime directory",
                false,
                source,
            ))
        }
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            "set runtime directory mode",
            false,
            source,
        )
    })?;
    path.canonicalize().map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            "canonicalize runtime directory",
            false,
            source,
        )
    })
}

fn verify_activity_dir(path: &str) -> Result<(), RuntimeError> {
    let path = Path::new(path);
    if !path.is_absolute() {
        return Err(runtime_error(
            RuntimeErrorKind::Protocol,
            "activity directory must be absolute",
            false,
        ));
    }
    let metadata = fs::symlink_metadata(path).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Protocol,
            "inspect activity directory",
            false,
            source,
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(runtime_error(
            RuntimeErrorKind::Protocol,
            "activity directory must be a non-symlink directory",
            false,
        ));
    }
    Ok(())
}

fn canonical_regular_file(path: &Path, label: &str) -> Result<PathBuf, RuntimeError> {
    let canonical = path.canonicalize().map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            format!("canonicalize {label}"),
            false,
            source,
        )
    })?;
    let metadata = fs::symlink_metadata(&canonical).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            format!("inspect {label}"),
            false,
            source,
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(runtime_error(
            RuntimeErrorKind::Startup,
            format!("{label} must be a non-symlink regular file"),
            false,
        ));
    }
    Ok(canonical)
}

fn bounded_command_output(
    executable: &Path,
    args: &[OsString],
    environment: &BTreeMap<OsString, OsString>,
) -> Result<String, RuntimeError> {
    let output = std::process::Command::new(executable)
        .args(args)
        .env_clear()
        .envs(environment)
        .output()
        .map_err(|source| {
            runtime_with_source(
                RuntimeErrorKind::Startup,
                "run version command",
                false,
                source,
            )
        })?;
    if !output.status.success() {
        return Err(runtime_error(
            RuntimeErrorKind::Startup,
            format!("version command exited with status {}", output.status),
            false,
        ));
    }
    let mut bytes = output.stdout;
    bytes.extend_from_slice(&output.stderr);
    if bytes.len() > MAX_VERSION_BYTES {
        bytes.truncate(MAX_VERSION_BYTES);
    }
    String::from_utf8(bytes).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            "version output was not UTF-8",
            false,
            source,
        )
    })
}

fn open_append(path: &Path) -> Result<fs::File, RuntimeError> {
    OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|source| {
            runtime_with_source(
                RuntimeErrorKind::Startup,
                "open spool for append",
                false,
                source,
            )
        })
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), RuntimeError> {
    let mut options = OpenOptions::new();
    let mut file = options
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|source| {
            runtime_with_source(
                RuntimeErrorKind::Startup,
                "create runtime file",
                false,
                source,
            )
        })?;
    file.write_all(bytes).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            "write runtime file",
            false,
            source,
        )
    })?;
    file.sync_all().map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            "sync runtime file",
            false,
            source,
        )
    })
}

fn utf8_path(path: &Path, label: &str) -> Result<String, RuntimeError> {
    path.to_str().map(str::to_owned).ok_or_else(|| {
        runtime_error(
            RuntimeErrorKind::Startup,
            format!("{label} path must be UTF-8"),
            false,
        )
    })
}

fn utf8_args(args: &[OsString], label: &str) -> Result<Vec<String>, RuntimeError> {
    args.iter()
        .map(|arg| {
            arg.to_str().map(str::to_owned).ok_or_else(|| {
                runtime_error(
                    RuntimeErrorKind::Startup,
                    format!("{label} must contain only UTF-8 arguments"),
                    false,
                )
            })
        })
        .collect()
}

fn utf8_env(env: &BTreeMap<OsString, OsString>) -> Result<BTreeMap<String, String>, RuntimeError> {
    env.iter()
        .map(|(key, value)| {
            let key = key.to_str().ok_or_else(|| {
                runtime_error(
                    RuntimeErrorKind::Startup,
                    "environment key must be UTF-8",
                    false,
                )
            })?;
            let value = value.to_str().ok_or_else(|| {
                runtime_error(
                    RuntimeErrorKind::Startup,
                    "environment value must be UTF-8",
                    false,
                )
            })?;
            Ok((key.to_owned(), value.to_owned()))
        })
        .collect()
}

fn hash_file(path: &Path, max_bytes: u64) -> Result<String, RuntimeError> {
    let mut file = fs::File::open(path).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            "open file for hashing",
            false,
            source,
        )
    })?;
    let metadata = file.metadata().map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            "inspect file for hashing",
            false,
            source,
        )
    })?;
    if metadata.len() > max_bytes {
        return Err(runtime_error(
            RuntimeErrorKind::Startup,
            "hashed file exceeds byte cap",
            false,
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Startup,
            "read file for hashing",
            false,
            source,
        )
    })?;
    Ok(hash_bytes(&bytes))
}

fn hash_json(value: &impl Serialize) -> Result<String, RuntimeError> {
    let bytes = serde_json::to_vec(value).map_err(|source| {
        runtime_with_source(
            RuntimeErrorKind::Protocol,
            "serialize digest input",
            false,
            source,
        )
    })?;
    Ok(hash_bytes(&bytes))
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn validate_sha256(value: &str) -> Result<(), RuntimeError> {
    if value.len() != 64
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
    {
        return Err(runtime_error(
            RuntimeErrorKind::Protocol,
            "process record digest must be lowercase SHA-256",
            false,
        ));
    }
    Ok(())
}

fn signal_process_group(pid: u32, signal: libc::c_int) -> Result<(), RuntimeError> {
    let pgid = -(pid as libc::pid_t);
    let result = unsafe { libc::kill(pgid, signal) };
    if result == 0 {
        return Ok(());
    }
    let source = std::io::Error::last_os_error();
    if matches!(source.raw_os_error(), Some(libc::ESRCH | libc::EPERM)) {
        return Ok(());
    }
    Err(runtime_with_source(
        RuntimeErrorKind::Transport,
        "signal CLI process group",
        true,
        source,
    ))
}

fn process_alive(pid: u32) -> bool {
    let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

fn unix_millis() -> Result<u128, RuntimeError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|source| {
            runtime_with_source(
                RuntimeErrorKind::Startup,
                "system clock is before UNIX epoch",
                false,
                source,
            )
        })
}

fn lock_inner(
    inner: &Arc<Mutex<RuntimeInner>>,
) -> Result<std::sync::MutexGuard<'_, RuntimeInner>, RuntimeError> {
    inner.lock().map_err(|_| {
        runtime_error(
            RuntimeErrorKind::Protocol,
            "CLI runtime mutex was poisoned",
            false,
        )
    })
}

fn runtime_error(
    kind: RuntimeErrorKind,
    message: impl Into<String>,
    transient: bool,
) -> RuntimeError {
    RuntimeError::try_new(kind, message, transient).unwrap_or_else(RuntimeError::from_contract)
}

fn runtime_with_source<E>(
    kind: RuntimeErrorKind,
    message: impl Into<String>,
    transient: bool,
    source: E,
) -> RuntimeError
where
    E: std::error::Error + Send + Sync + 'static,
{
    RuntimeError::with_source(kind, message, transient, source)
        .unwrap_or_else(RuntimeError::from_contract)
}

trait ThreadIdExt {
    fn from_str_like(seed: &str) -> Result<ThreadId, RuntimeError>;
}

impl ThreadIdExt for ThreadId {
    fn from_str_like(seed: &str) -> Result<ThreadId, RuntimeError> {
        let digest = hash_bytes(seed.as_bytes());
        format!("harp-logical-{digest}")
            .parse()
            .map_err(RuntimeError::from_contract)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Seek, SeekFrom, Write};

    use super::*;

    #[test]
    fn executable_hash_cap_accepts_large_traecli_sized_binaries() {
        let root = tempfile::tempdir().expect("temp dir");
        let executable = root.path().join("large-executable");
        let mut file = fs::File::create(&executable).expect("create executable");
        file.seek(SeekFrom::Start(240 * 1024 * 1024))
            .expect("seek sparse executable");
        file.write_all(b"x").expect("write executable byte");
        drop(file);

        let digest =
            hash_file(&executable, MAX_EXECUTABLE_HASH_BYTES).expect("hash large executable");
        assert_eq!(digest.len(), 64);
    }

    #[test]
    fn executable_hash_cap_rejects_over_limit_binaries() {
        let root = tempfile::tempdir().expect("temp dir");
        let executable = root.path().join("too-large-executable");
        let mut file = fs::File::create(&executable).expect("create executable");
        file.seek(SeekFrom::Start(MAX_EXECUTABLE_HASH_BYTES))
            .expect("seek sparse executable");
        file.write_all(b"x").expect("write executable byte");
        drop(file);

        let error = hash_file(&executable, MAX_EXECUTABLE_HASH_BYTES).unwrap_err();
        assert_eq!(error.kind(), RuntimeErrorKind::Startup);
        assert!(error.message().contains("hashed file exceeds byte cap"));
    }

    #[test]
    fn thread_started_timeout_allows_slow_live_cli_startup() {
        assert!(
            THREAD_STARTED_TIMEOUT >= Duration::from_secs(60),
            "live CLI startup can exceed the old 10 second local adapter wait"
        );
    }

    #[test]
    fn reasoning_item_completed_does_not_replace_final_agent_message() {
        let root = tempfile::tempdir().expect("temp dir");
        let stdout_path = root.path().join("stdout.jsonl");
        fs::write(
            &stdout_path,
            concat!(
                "{\"type\":\"item.completed\",\"item\":{\"type\":\"reasoning\",\"text\":\"not json\"}}\n",
                "{\"type\":\"item.completed\",\"item\":{\"type\":\"agent_message\",\"text\":\"{\\\"ok\\\":true}\"}}\n",
            ),
        )
        .expect("stdout events");
        let mut process = ActivityProcess {
            record: ProcessRecord {
                schema_version: 1,
                pid: 1,
                started_unix_millis: 1,
                executable_path: "/bin/echo".to_owned(),
                argv: Vec::new(),
                codex_home_path: root.path().display().to_string(),
                activity_dir: root.path().display().to_string(),
                logical_session_id: "thread".parse().unwrap(),
                logical_turn_id: "turn".parse().unwrap(),
                invocation_sha256:
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
                stdout_path: stdout_path.display().to_string(),
                stderr_path: root.path().join("stderr.log").display().to_string(),
                output_schema_path: root.path().join("schema.json").display().to_string(),
                final_message_path: root.path().join("final.txt").display().to_string(),
                environment_sha256:
                    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
            },
            child: None,
            offset: 0,
            final_message: None,
            queued: VecDeque::new(),
            terminal_seen: false,
        };

        assert!(matches!(
            next_process_event_internal(&mut process).unwrap(),
            Some(ParsedProcessEvent::NoRuntimeEvent)
        ));
        assert_eq!(process.final_message, None);
        assert!(matches!(
            next_process_event_internal(&mut process).unwrap(),
            Some(ParsedProcessEvent::NoRuntimeEvent)
        ));
        assert_eq!(process.final_message.as_deref(), Some("{\"ok\":true}"));
    }
}

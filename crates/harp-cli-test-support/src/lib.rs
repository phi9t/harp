use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

const DEFAULT_SESSION_ID: &str = "fake-codex-session";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Success,
    MalformedJsonl,
    DelayedOutput,
    NonzeroExit,
    MissingThreadStarted,
    SchemaFailure,
    DurableBarrier,
    DescendantHeldDescriptors,
}

impl Mode {
    fn from_env() -> Self {
        match env::var("HARP_FAKE_CLI_MODE").as_deref() {
            Ok("malformed-jsonl") => Self::MalformedJsonl,
            Ok("delayed-output") => Self::DelayedOutput,
            Ok("nonzero-exit") => Self::NonzeroExit,
            Ok("missing-thread-started") => Self::MissingThreadStarted,
            Ok("schema-failure") => Self::SchemaFailure,
            Ok("durable-barrier") => Self::DurableBarrier,
            Ok("descendant-held-descriptors") => Self::DescendantHeldDescriptors,
            _ => Self::Success,
        }
    }
}

pub fn run_fake_cli() -> i32 {
    match run_fake_cli_inner() {
        Ok(code) => code,
        Err(error) => {
            let _ = writeln!(io::stderr(), "{error}");
            70
        }
    }
}

fn run_fake_cli_inner() -> Result<i32, Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--version") {
        println!("harp-fake-codex 0.1.0");
        return Ok(0);
    }
    let is_resume = if args.first().is_some_and(|arg| arg == "exec") {
        args.get(1).is_some_and(|arg| arg == "resume")
    } else {
        args.first().is_some_and(|arg| arg == "resume")
    };
    if args.first().is_some_and(|arg| arg == "exec") {
        args.remove(0);
    }
    if args.first().is_some_and(|arg| arg == "resume") {
        args.remove(0);
        if !args.is_empty() {
            args.remove(0);
        }
    }
    let mut stdin_prompt = String::new();
    let _ = io::stdin().read_to_string(&mut stdin_prompt);
    let mode = Mode::from_env();
    if mode == Mode::NonzeroExit {
        eprintln!("fake cli nonzero exit");
        return Ok(42);
    }
    if mode == Mode::MalformedJsonl {
        println!("{{not-json");
        return Ok(0);
    }
    if mode == Mode::DelayedOutput {
        thread::sleep(Duration::from_millis(
            env::var("HARP_FAKE_CLI_DELAY_MS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(200),
        ));
    }
    if mode == Mode::DescendantHeldDescriptors {
        spawn_descriptor_holder();
    }
    let session_id = env::var("HARP_FAKE_CLI_SESSION_ID")
        .ok()
        .unwrap_or_else(|| session_id_for_prompt(&stdin_prompt));
    let final_message =
        final_message_for_prompt(&stdin_prompt).unwrap_or_else(|| r#"{"answer":"ok"}"#.to_owned());
    let total_tokens = env::var("HARP_FAKE_CLI_TOTAL_TOKENS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(12);
    append_call_ledger(&session_id, is_resume)?;
    if mode != Mode::MissingThreadStarted {
        emit(&serde_json::json!({
            "type": "thread.started",
            "thread_id": session_id,
        }))?;
    }
    if mode == Mode::DurableBarrier {
        wait_for_barrier()?;
    }
    emit(&serde_json::json!({"type": "turn.started"}))?;
    if mode == Mode::SchemaFailure {
        emit(&serde_json::json!({
            "type": "item.completed",
            "item": {
                "id": "item_schema",
                "type": "agent_message",
                "text": "not-json"
            }
        }))?;
    } else {
        emit(&serde_json::json!({
            "type": "item.completed",
            "item": {
                "id": "item_final",
                "type": "agent_message",
                "text": final_message
            }
        }))?;
    }
    emit(&serde_json::json!({
        "type": "turn.completed",
        "usage": {
            "input_tokens": total_tokens,
            "cached_input_tokens": 0,
            "output_tokens": 0,
            "reasoning_output_tokens": 0
        }
    }))?;
    Ok(0)
}

fn final_message_for_prompt(prompt: &str) -> Option<String> {
    if let Some(map) = env::var("HARP_FAKE_CLI_FINAL_MESSAGES")
        .ok()
        .and_then(|value| serde_json::from_str::<BTreeMap<String, String>>(&value).ok())
    {
        for (task_id, message) in map {
            if prompt.contains(&format!(r#""taskId":"{task_id}""#))
                || prompt.contains(&format!(r#""task_id":"{task_id}""#))
                || prompt.contains(&format!("execute {task_id}"))
                || prompt.contains(&format!("produce {task_id}"))
            {
                return Some(message);
            }
        }
    }
    env::var("HARP_FAKE_CLI_FINAL_MESSAGE").ok()
}

fn session_id_for_prompt(prompt: &str) -> String {
    for task_id in ["alpha", "beta", "reduce"] {
        if prompt.contains(&format!(r#""taskId":"{task_id}""#))
            || prompt.contains(&format!(r#""task_id":"{task_id}""#))
            || prompt.contains(&format!("execute {task_id}"))
            || prompt.contains(&format!("produce {task_id}"))
        {
            return format!("{DEFAULT_SESSION_ID}-{task_id}");
        }
    }
    DEFAULT_SESSION_ID.to_owned()
}

fn emit(value: &serde_json::Value) -> io::Result<()> {
    println!(
        "{}",
        serde_json::to_string(value).expect("fake event serializes")
    );
    io::stdout().flush()
}

fn append_call_ledger(session_id: &str, is_resume: bool) -> io::Result<()> {
    let Some(path) = env::var_os("HARP_FAKE_CLI_LEDGER").map(PathBuf::from) else {
        return Ok(());
    };
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(
        file,
        "{}",
        serde_json::json!({
            "sessionId": session_id,
            "resume": is_resume,
            "pid": std::process::id(),
        })
    )
}

fn wait_for_barrier() -> io::Result<()> {
    let Some(path) = env::var_os("HARP_FAKE_CLI_BARRIER").map(PathBuf::from) else {
        return Ok(());
    };
    while path.exists() {
        thread::sleep(Duration::from_millis(25));
    }
    Ok(())
}

fn spawn_descriptor_holder() {
    #[cfg(unix)]
    unsafe {
        let pid = libc_fork();
        if pid == 0 {
            thread::sleep(Duration::from_secs(60));
            std::process::exit(0);
        }
    }
}

#[cfg(unix)]
unsafe fn libc_fork() -> i32 {
    extern "C" {
        fn fork() -> i32;
    }
    fork()
}

#[allow(dead_code)]
pub fn write_barrier(path: PathBuf) -> io::Result<()> {
    fs::write(path, b"hold")
}

//! Spike M3.1: what can corenius observe and gate of an ACP agent's side effects?
//!
//! The fixture agent asks its client to write a claimed path and to grant
//! permission, waits briefly for answers, then also writes a file directly
//! (the way a provider's own shell/edit tool would). The tests pin down:
//! (1) what corenius's shipped ACP client does with those requests today,
//! (2) that a client which does implement the methods can observe and deny them,
//! (3) that the direct write is invisible to the protocol in both cases.
//! See docs/spikes/collab/interception.md.

use std::fs;
use std::path::Path;
use std::process::Stdio;

use corenius::provider::acp::{AcpOptions, AcpSessionProfile, run_acp_prompt};
use serde_json::{Value, json};
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{Duration, timeout};

/// argv: <log.json> <workdir> <mode>. mode "normal" or "id_collision".
const AGENT: &str = r##"#!/usr/bin/env python3
import json, os, select, sys
log_path, workdir, mode = sys.argv[1], sys.argv[2], sys.argv[3]
log = {"initialize": None, "replies": {}}
buf = b""
def emit(obj): print(json.dumps(obj), flush=True)
def read_msg(wait):
    global buf
    while b"\n" not in buf:
        r, _, _ = select.select([0], [], [], wait)
        if not r: return None
        chunk = os.read(0, 65536)
        if not chunk: return None
        buf += chunk
    line, buf = buf.split(b"\n", 1)
    return json.loads(line)
def save(): json.dump(log, open(log_path, "w"))
while True:
    msg = read_msg(30)
    if msg is None: break
    m = msg.get("method")
    if m == "initialize":
        log["initialize"] = msg["params"]
        emit({"jsonrpc":"2.0","id":msg["id"],"result":{"protocolVersion":1}})
    elif m == "session/new":
        emit({"jsonrpc":"2.0","id":msg["id"],"result":{"sessionId":"s1"}})
    elif m == "session/prompt":
        pid = msg["id"]
        if mode == "id_collision":
            # a legitimate agent->client request whose id equals the prompt id
            emit({"jsonrpc":"2.0","id":pid,"method":"fs/read_text_file","params":{"sessionId":"s1","path":workdir+"/x"}})
            read_msg(1.0)
            open(workdir + "/late.txt", "w").write("agent kept working")
            save()
            emit({"jsonrpc":"2.0","id":pid,"result":{"stopReason":"end_turn"}})
            continue
        emit({"jsonrpc":"2.0","id":900,"method":"fs/write_text_file","params":{"sessionId":"s1","path":workdir+"/claimed.txt","content":"via protocol"}})
        emit({"jsonrpc":"2.0","id":901,"method":"session/request_permission","params":{"sessionId":"s1","toolCall":{"toolCallId":"t1","title":"edit claimed.txt","kind":"edit"},"options":[{"optionId":"ok","name":"Allow","kind":"allow_once"},{"optionId":"no","name":"Reject","kind":"reject_once"}]}})
        for _ in range(2):
            r = read_msg(1.0)
            if r is not None and "id" in r: log["replies"][str(r["id"])] = r
        # direct write, as the provider's own shell/edit tool would do
        open(workdir + "/bypass.txt", "w").write("direct write")
        save()
        emit({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"s1","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"done"}}}})
        emit({"jsonrpc":"2.0","id":pid,"result":{"stopReason":"end_turn"}})
"##;

fn write_agent(dir: &Path) -> std::path::PathBuf {
    let path = dir.join("agent.py");
    fs::write(&path, AGENT).unwrap();
    path
}

fn read_log(dir: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(dir.join("log.json")).unwrap()).unwrap()
}

/// Today's shipped client: advertises no capabilities and never answers
/// agent-initiated requests, so the agent's fs/permission requests are unseen.
#[tokio::test]
async fn shipped_client_advertises_nothing_and_answers_nothing() {
    let temp = tempdir().unwrap();
    let agent = write_agent(temp.path());
    let out = run_acp_prompt(AcpOptions {
        command: "python3".into(),
        args: vec![
            agent.to_string_lossy().into_owned(),
            temp.path().join("log.json").to_string_lossy().into_owned(),
            temp.path().to_string_lossy().into_owned(),
            "normal".into(),
        ],
        cwd: temp.path().to_path_buf(),
        model: None,
        prompt: "edit".into(),
        timeout_ms: 20_000,
        profile: AcpSessionProfile::default(),
    })
    .await
    .unwrap();
    assert_eq!(out.text, "done");
    let log = read_log(temp.path());
    // acp.rs:199 sends `"clientCapabilities": {}`.
    assert_eq!(log["initialize"]["clientCapabilities"], json!({}));
    // No reply to either agent request: corenius neither gates nor denies.
    assert_eq!(log["replies"], json!({}));
    // The direct write happened and corenius has no record of it.
    assert_eq!(
        fs::read_to_string(temp.path().join("bypass.txt")).unwrap(),
        "direct write"
    );
    assert!(!temp.path().join("claimed.txt").exists());
}

/// A server-to-client request whose id equals the outstanding prompt id is
/// mistaken for the prompt response (acp.rs:247 matches on id alone), ending
/// the turn early with no stop reason.
#[tokio::test]
async fn agent_request_reusing_prompt_id_ends_turn_early() {
    let temp = tempdir().unwrap();
    let agent = write_agent(temp.path());
    let out = run_acp_prompt(AcpOptions {
        command: "python3".into(),
        args: vec![
            agent.to_string_lossy().into_owned(),
            temp.path().join("log.json").to_string_lossy().into_owned(),
            temp.path().to_string_lossy().into_owned(),
            "id_collision".into(),
        ],
        cwd: temp.path().to_path_buf(),
        model: None,
        prompt: "go".into(),
        timeout_ms: 20_000,
        profile: AcpSessionProfile::default(),
    })
    .await
    .unwrap();
    assert_eq!(
        out.stop_reason, None,
        "request misread as the prompt response"
    );
}

/// Minimal gating client written against the wire protocol (not corenius code):
/// advertises fs.writeTextFile, denies writes to claimed paths, allows others.
/// Shows the protocol seam is sufficient to observe and deny what goes through it.
#[tokio::test]
async fn protocol_client_can_observe_and_deny_fs_write_but_not_direct_write() {
    let temp = tempdir().unwrap();
    let agent = write_agent(temp.path());
    let claimed = temp
        .path()
        .join("claimed.txt")
        .to_string_lossy()
        .into_owned();
    let mut child = Command::new("python3")
        .arg(&agent)
        .arg(temp.path().join("log.json"))
        .arg(temp.path())
        .arg("normal")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let send = |v: Value| {
        let line = format!("{v}\n");
        line.into_bytes()
    };
    let mut observed: Vec<(String, String)> = Vec::new();

    stdin
        .write_all(&send(
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
        "protocolVersion":1,
        "clientCapabilities":{"fs":{"readTextFile":true,"writeTextFile":true}}}}),
        ))
        .await
        .unwrap();
    stdin.write_all(&send(json!({"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":temp.path(),"mcpServers":[]}}))).await.unwrap();
    stdin.write_all(&send(json!({"jsonrpc":"2.0","id":3,"method":"session/prompt","params":{"sessionId":"s1","prompt":[{"type":"text","text":"edit"}]}}))).await.unwrap();

    let mut prompt_done = false;
    timeout(Duration::from_secs(20), async {
        while !prompt_done {
            let line = lines.next_line().await.unwrap().expect("agent exited");
            let msg: Value = serde_json::from_str(&line).unwrap();
            match (msg.get("method").and_then(Value::as_str), msg.get("id")) {
                (Some("fs/write_text_file"), Some(id)) => {
                    let path = msg["params"]["path"].as_str().unwrap().to_string();
                    observed.push(("fs/write_text_file".into(), path.clone()));
                    let reply = if path == claimed {
                        json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":"path claimed by another agent"}})
                    } else {
                        fs::write(&path, msg["params"]["content"].as_str().unwrap()).unwrap();
                        json!({"jsonrpc":"2.0","id":id,"result":null})
                    };
                    stdin.write_all(&send(reply)).await.unwrap();
                }
                (Some("session/request_permission"), Some(id)) => {
                    observed.push(("session/request_permission".into(),
                        msg["params"]["toolCall"]["title"].as_str().unwrap().to_string()));
                    stdin.write_all(&send(json!({"jsonrpc":"2.0","id":id,"result":{"outcome":{"outcome":"selected","optionId":"no"}}}))).await.unwrap();
                }
                (None, Some(id)) if id == &json!(3) => prompt_done = true,
                _ => {}
            }
        }
    })
    .await
    .unwrap();
    drop(stdin);
    let _ = child.wait().await;

    assert_eq!(observed.len(), 2);
    assert_eq!(observed[0].0, "fs/write_text_file");
    assert_eq!(observed[1].0, "session/request_permission");
    let log = read_log(temp.path());
    assert_eq!(log["replies"]["900"]["error"]["code"], json!(-32000));
    assert_eq!(
        log["replies"]["901"]["result"]["outcome"]["optionId"],
        json!("no")
    );
    // Denied: the protocol write never landed.
    assert!(!temp.path().join("claimed.txt").exists());
    // Invisible: the agent's own write is not in `observed` and did land.
    assert_eq!(
        fs::read_to_string(temp.path().join("bypass.txt")).unwrap(),
        "direct write"
    );
}

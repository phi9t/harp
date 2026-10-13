use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use serde_json::json;

pub enum ScriptedAcpStep {
    Success {
        text: &'static str,
    },
    ProviderError {
        message: &'static str,
    },
    Stall,
    /// Emit one text chunk, then go silent without completing the prompt.
    PartialThenStall {
        text: &'static str,
    },
    MalformedLine,
}

pub fn scripted_acp(path: &Path, steps: &[ScriptedAcpStep]) {
    if steps.is_empty() {
        panic!("scripted_acp requires at least one step");
    }
    let script_path = path.with_extension("script.json");
    let cursor_path = path.with_extension("cursor");
    let steps_json = steps
        .iter()
        .map(|step| match step {
            ScriptedAcpStep::Success { text } => json!({"kind": "success", "text": text}),
            ScriptedAcpStep::ProviderError { message } => {
                json!({"kind": "provider_error", "message": message})
            }
            ScriptedAcpStep::Stall => json!({"kind": "stall"}),
            ScriptedAcpStep::PartialThenStall { text } => {
                json!({"kind": "partial_then_stall", "text": text})
            }
            ScriptedAcpStep::MalformedLine => json!({"kind": "malformed_line"}),
        })
        .collect::<Vec<_>>();
    fs::write(
        &script_path,
        serde_json::to_string_pretty(&steps_json).unwrap(),
    )
    .unwrap();
    fs::write(&cursor_path, "0").unwrap();

    let script = r##"#!/usr/bin/env python3
import json, sys, time

script_path = __SCRIPT_PATH__
cursor_path = __CURSOR_PATH__
argv_path = __ARGV_PATH__
session_new_path = __SESSION_NEW_PATH__

if sys.argv[1:3] != ["acp", "serve"]:
    raise SystemExit("expected acp serve")
open(argv_path, "w").write(json.dumps(sys.argv[1:]))

steps = json.load(open(script_path))
cursor = int(open(cursor_path).read().strip() or "0")
session_id = "scripted"

def advance():
    global cursor
    cursor += 1
    open(cursor_path, "w").write(str(cursor))

def respond(msg, result):
    print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"result":result}), flush=True)

for line in sys.stdin:
    msg = json.loads(line)
    method = msg.get("method")
    if method == "initialize":
        respond(msg, {"protocolVersion": 1})
    elif method == "session/new":
        if "cwd" not in msg.get("params", {}):
            raise SystemExit("session/new missing cwd")
        open(session_new_path, "w").write(json.dumps(msg["params"]))
        respond(msg, {"sessionId": session_id})
    elif method == "session/prompt":
        params = msg.get("params", {})
        if params.get("sessionId") != session_id:
            raise SystemExit("session/prompt wrong sessionId")
        prompt = params.get("prompt")
        if not isinstance(prompt, list) or not prompt or prompt[0].get("type") != "text":
            raise SystemExit("session/prompt missing text prompt")
        if cursor >= len(steps):
            raise SystemExit("script exhausted")
        step = steps[cursor]
        kind = step["kind"]
        if kind == "success":
            text = step["text"]
            print(json.dumps({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":session_id,"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":text}}}}), flush=True)
            advance()
            respond(msg, {"stopReason":"end_turn","usage":{"inputTokens":1,"outputTokens":1}})
        elif kind == "provider_error":
            advance()
            print(json.dumps({"jsonrpc":"2.0","id":msg["id"],"error":{"code":-32000,"message":step["message"]}}), flush=True)
        elif kind == "stall":
            advance()
            time.sleep(60)
        elif kind == "partial_then_stall":
            advance()
            print(json.dumps({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":session_id,"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":step["text"]}}}}), flush=True)
            time.sleep(60)
        elif kind == "malformed_line":
            advance()
            print("{not-json", flush=True)
        else:
            raise SystemExit("unknown script kind " + kind)
"##;
    let script = script
        .replace("__SCRIPT_PATH__", &format!("{script_path:?}"))
        .replace("__CURSOR_PATH__", &format!("{cursor_path:?}"))
        .replace(
            "__ARGV_PATH__",
            &format!("{:?}", path.with_extension("argv.json")),
        )
        .replace(
            "__SESSION_NEW_PATH__",
            &format!("{:?}", path.with_extension("session_new.json")),
        );
    fs::write(path, script).unwrap();

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

pub fn assert_script_consumed(path: &Path) {
    let script_path = path.with_extension("script.json");
    let cursor_path = path.with_extension("cursor");
    let steps: Vec<serde_json::Value> =
        serde_json::from_str(&fs::read_to_string(script_path).unwrap()).unwrap();
    let cursor: usize = fs::read_to_string(cursor_path)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(
        cursor,
        steps.len(),
        "scripted ACP fixture was not fully consumed"
    );
}

/// The `traecli acp serve` arguments the scripted server was last launched with.
pub fn recorded_serve_args(path: &Path) -> Vec<String> {
    serde_json::from_str(&fs::read_to_string(path.with_extension("argv.json")).unwrap()).unwrap()
}

/// The `session/new` params the scripted server last received.
pub fn recorded_session_new_params(path: &Path) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(path.with_extension("session_new.json")).unwrap())
        .unwrap()
}

//! Terminal and MCP clients share the authenticated controller socket; no storage/Git access.
use serde_json::{Value, json};
use std::{
    io::{BufRead, Read, Write},
    os::unix::net::UnixStream,
    time::Duration,
};
fn credential() -> Result<Value, String> {
    let token = std::env::var("CORENIUS_CLIENT_TOKEN")
        .or_else(|_| std::env::var("CORENIUS_OPERATOR_TOKEN"))
        .map_err(|_| "CORENIUS_CLIENT_TOKEN required")?;
    let principal =
        std::env::var("CORENIUS_CLIENT_PRINCIPAL").unwrap_or_else(|_| "operator".into());
    Ok(json!({"principal":principal,"token":token}))
}
pub fn call(socket: &str, request: &Value) -> Result<Value, String> {
    let mut stream =
        UnixStream::connect(socket).map_err(|e| format!("controller transport: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(40)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    let bytes = format!("{request}\n");
    if bytes.len() > 1_048_576 {
        return Err("client request bound".into());
    }
    stream
        .write_all(bytes.as_bytes())
        .map_err(|e| format!("controller transport: {e}"))?;
    let mut reader = std::io::BufReader::new(stream);
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(4_194_305)
        .read_until(b'\n', &mut bytes)
        .map_err(|e| format!("controller transport: {e}"))?;
    if bytes.len() > 4_194_304 || bytes.last() != Some(&b'\n') {
        return Err("controller response bound/framing".into());
    }
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if let Some(error) = v["error"].as_str() {
        return Err(error.into());
    }
    v.get("ok")
        .cloned()
        .ok_or("controller response missing".into())
}
fn redact(v: &mut Value) {
    match v {
        Value::Object(o) => {
            o.remove("token");
            for child in o.values_mut() {
                redact(child);
            }
        }
        Value::Array(a) => {
            for child in a {
                redact(child)
            }
        }
        _ => {}
    }
}
fn display(v: &Value) -> String {
    let mut output = String::new();
    for (title, key) in [
        ("Mission and constraints", "mission"),
        (
            "Worker assignments: observed state and reported progress",
            "workers",
        ),
        (
            "Questions: delivery, acknowledgement, resolution",
            "questions",
        ),
        ("Source revisions and amendments", "sources"),
        ("Filesystem revisions and ownership drift", "filesystem"),
        ("Brief proposals", "brief_proposals"),
        ("Kata synchronization and drift", "synchronization"),
        ("Candidate identities", "candidates"),
        ("Verification evidence", "verification"),
        ("Acceptance and landing queue", "landing_queue"),
        ("Unknown effects", "unknown_effects"),
        ("Runtime uncertainty", "runtime_uncertainty"),
        (
            "Monitoring coverage, hypotheses, findings and corrections",
            "monitoring",
        ),
        ("Repair history", "repair_history"),
        (
            "Retained history pages and exact input references",
            "pagination",
        ),
        ("Next decisions", "next_decisions"),
    ] {
        output.push_str(&format!(
            "\n{title}\n{}\n",
            serde_json::to_string_pretty(&v[key]).unwrap_or_default()
        ));
    }
    output.push_str(&format!(
        "\nController revision {} / replay cursor {} / occupied worker slots {}\n",
        v["revision"], v["event_cursor"], v["occupied_slots"]
    ));
    output
}
pub fn run(args: &[String]) -> Result<(), String> {
    let socket = args.get(2).ok_or("controller socket required")?;
    let project = args.get(3).ok_or("fixed project required")?;
    let auth = credential()?;
    if args[1] == "--mcp" {
        return mcp(socket, project, &auth);
    }
    if args[1] == "--control" {
        let path = args
            .get(4)
            .ok_or("--control SOCKET PROJECT SAVED_COMMAND_JSON required")?;
        let saved: Value = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let mut response = call(
            socket,
            &json!({"auth":auth,"project":project,"command":saved}),
        )?;
        redact(&mut response);
        println!("{response}");
        return Ok(());
    }
    let once = args.iter().any(|s| s == "--once");
    let mut cursor = 0u64;
    let mut lines = std::io::stdin().lock().lines();
    loop {
        let events = call(
            socket,
            &json!({"auth":auth,"project":project,"query":{"kind":"event_page","args":{"after":cursor,"limit":20}}}),
        )?;
        if let Some(seq) = events["items"]
            .as_array()
            .and_then(|e| e.last())
            .and_then(|e| e["seq"].as_u64())
        {
            cursor = seq;
        }
        let mut state = call(
            socket,
            &json!({"auth":auth,"project":project,"query":{"kind":"control_center","args":{}}}),
        )?;
        state["reconnect_events"] = events["page"].clone();
        println!("{}", display(&state));
        println!("Event reconnect continuation {}", state["reconnect_events"]);
        if once {
            return Ok(());
        }
        println!(
            "Enter a saved command JSON object (kind, expected_revision, idempotency_key, args), 'refresh', or 'quit'. Preserve the object for ambiguous retries."
        );
        let Some(line) = lines.next() else {
            return Ok(());
        };
        let line = line.map_err(|e| e.to_string())?;
        match line.trim() {
            "quit" => return Ok(()),
            "refresh" | "" => continue,
            _ => {}
        }
        let command: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        match call(
            socket,
            &json!({"auth":auth,"project":project,"command":command}),
        ) {
            Ok(mut receipt) => {
                redact(&mut receipt);
                println!("Receipt {receipt}");
            }
            Err(error) => {
                eprintln!("Control disposition: {error}; retain the same request before retry")
            }
        }
    }
}
fn mcp(socket: &str, project: &str, auth: &Value) -> Result<(), String> {
    let input = std::io::stdin();
    let mut output = std::io::stdout();
    for line in input.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.len() > 1_048_576 {
            return Err("MCP request bound".into());
        }
        let request: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        let Some(id) = request.get("id") else {
            continue;
        };
        let result=match request["method"].as_str(){
            Some("initialize")=>Ok(json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"corenius-bound-controller","version":"0.1.0"}})),
            Some("tools/list")=>{let tools:Vec<Value>=crate::gateway::specifications().as_array().unwrap().iter().map(|s|json!({"name":s["name"],"description":s["description"],"inputSchema":s["inputSchema"]})).collect();Ok(json!({"tools":tools}))},
            Some("tools/call")=>call(socket,&json!({"auth":auth,"project":project,"gateway":{"tool":request["params"]["name"],"arguments":request["params"]["arguments"]}})).map(|mut value|{redact(&mut value);json!({"content":[{"type":"text","text":value.to_string()}],"isError":false})}),
            _=>Err("MCP method refused".into())};
        let response = match result {
            Ok(result) => json!({"jsonrpc":"2.0","id":id,"result":result}),
            Err(error) => json!({"jsonrpc":"2.0","id":id,"error":{"code":-32000,"message":error}}),
        };
        writeln!(output, "{response}").map_err(|e| e.to_string())?;
        output.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

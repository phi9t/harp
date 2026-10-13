//! Replaceable loopback native UI attachment. Disconnecting it never owns runtime lifetime.
use crate::{client, hash};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use sha1::{Digest, Sha1};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
    time::{Duration, Instant},
};
struct Gateway {
    socket: String,
    project: String,
    auth: Value,
    token_hash: String,
}
fn exact(stream: &mut TcpStream, bytes: &mut [u8], deadline: Instant) -> Result<(), String> {
    let mut offset = 0;
    while offset < bytes.len() {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or("UI frame deadline")?;
        stream
            .set_read_timeout(Some(remaining))
            .map_err(|e| e.to_string())?;
        let n = stream
            .read(&mut bytes[offset..])
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("UI disconnected".into());
        }
        offset += n;
    }
    Ok(())
}
fn send(stream: &mut TcpStream, value: &Value) -> Result<(), String> {
    let data = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    if data.len() > 2_097_152 {
        return Err("UI output bound".into());
    }
    send_frame(stream, 1, &data)
}
fn send_frame(stream: &mut TcpStream, opcode: u8, data: &[u8]) -> Result<(), String> {
    let mut bytes = vec![0x80 | opcode];
    if data.len() < 126 {
        bytes.push(data.len() as u8);
    } else if data.len() < 65536 {
        bytes.push(126);
        bytes.extend_from_slice(&(data.len() as u16).to_be_bytes());
    } else {
        bytes.push(127);
        bytes.extend_from_slice(&(data.len() as u64).to_be_bytes());
    }
    bytes.extend_from_slice(data);
    stream
        .set_write_timeout(Some(Duration::from_secs(1)))
        .map_err(|e| e.to_string())?;
    stream.write_all(&bytes).map_err(|e| e.to_string())
}
fn frame(stream: &mut TcpStream) -> Result<Option<Value>, String> {
    stream
        .set_read_timeout(Some(Duration::from_millis(200)))
        .map_err(|e| e.to_string())?;
    let mut first = [0];
    match stream.read(&mut first) {
        Ok(0) => return Err("UI disconnected".into()),
        Ok(_) => {}
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ) =>
        {
            return Ok(None);
        }
        Err(e) => return Err(e.to_string()),
    }
    let deadline = Instant::now() + Duration::from_secs(1);
    let mut second = [0];
    exact(stream, &mut second, deadline)?;
    let opcode = first[0] & 0x0f;
    if first[0] & 0xf0 != 0x80 || !matches!(opcode, 1 | 8 | 9 | 10) || second[0] & 0x80 == 0 {
        return Err("UI requires complete masked text or control frames".into());
    }
    let mut len = (second[0] & 0x7f) as u64;
    if opcode >= 8 && len > 125 {
        return Err("UI control frame bound".into());
    }
    if len == 126 {
        let mut b = [0; 2];
        exact(stream, &mut b, deadline)?;
        len = u16::from_be_bytes(b) as u64;
    } else if len == 127 {
        let mut b = [0; 8];
        exact(stream, &mut b, deadline)?;
        len = u64::from_be_bytes(b);
    }
    if len > 1_048_576 {
        return Err("UI input bound".into());
    }
    let mut mask = [0; 4];
    exact(stream, &mut mask, deadline)?;
    let mut body = vec![0; len as usize];
    exact(stream, &mut body, deadline)?;
    for (i, b) in body.iter_mut().enumerate() {
        *b ^= mask[i % 4];
    }
    match opcode {
        8 => {
            if body.len() == 1 {
                return Err("UI close payload refused".into());
            }
            if body.len() >= 2 {
                let code = u16::from_be_bytes([body[0], body[1]]);
                if !matches!(code, 1000..=1003 | 1007..=1014 | 3000..=4999)
                    || std::str::from_utf8(&body[2..]).is_err()
                {
                    return Err("UI close status/reason refused".into());
                }
            }
            send_frame(stream, 8, &body)?;
            return Err("UI closed".into());
        }
        9 => {
            send_frame(stream, 10, &body)?;
            return Ok(None);
        }
        10 => return Ok(None),
        _ => {}
    }
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|e| e.to_string())
}
fn audit(value: Value) {
    if let Ok(path) = std::env::var("CORENIUS_UI_AUDIT") {
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(path)
        {
            let _ = writeln!(file, "{value}");
        }
    }
}
fn serve(mut stream: TcpStream, g: Arc<Gateway>) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(1);
    let mut header = Vec::new();
    let mut byte = [0];
    while !header.ends_with(b"\r\n\r\n") {
        if header.len() > 16384 {
            return Err("UI handshake bound".into());
        }
        exact(&mut stream, &mut byte, deadline)?;
        header.push(byte[0]);
    }
    let header = String::from_utf8(header).map_err(|e| e.to_string())?;
    let mut key = None;
    let mut authenticated = false;
    for line in header.lines() {
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("sec-websocket-key") {
                key = Some(value.trim());
            } else if name.eq_ignore_ascii_case("authorization") {
                authenticated = value
                    .trim()
                    .strip_prefix("Bearer ")
                    .is_some_and(|t| hash(t) == g.token_hash);
            }
        }
    }
    if !authenticated || !header.starts_with("GET /") {
        return Err("UI authentication refused".into());
    }
    let accept = STANDARD.encode(Sha1::digest(format!(
        "{}258EAFA5-E914-47DA-95CA-C5AB0DC85B11",
        key.ok_or("UI WebSocket key missing")?
    )));
    write!(stream,"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n").map_err(|e|e.to_string())?;
    let mut after = 0u64;
    let mut initialized = false;
    loop {
        if let Some(v) = frame(&mut stream)? {
            let id = v.get("id").cloned();
            let result = if v.get("result").is_some() || v.get("error").is_some() {
                Err(
                    "UI tool responses are refused; controller exclusively brokers native requests"
                        .into(),
                )
            } else if let Some(method) = v["method"].as_str() {
                if method == "initialized" {
                    initialized = true;
                }
                client::call(
                    &g.socket,
                    &json!({"auth":g.auth,"project":g.project,"native_ui":{"method":method,"params":v.get("params").cloned().unwrap_or(json!({}))}}),
                )
            } else {
                Err("UI JSON-RPC method missing".into())
            };
            audit(
                json!({"method":v["method"],"request_id":id,"argument_digest":hash(&v["params"].to_string()),"argument_keys":v["params"].as_object().map(|o|o.keys().collect::<Vec<_>>()),"accepted":result.is_ok(),"error":result.as_ref().err()}),
            );
            if let Some(id) = id {
                send(
                    &mut stream,
                    &match result {
                        Ok(result) => json!({"id":id,"result":result}),
                        Err(error) => json!({"id":id,"error":{"code":-32601,"message":error}}),
                    },
                )?;
            }
        }
        if initialized {
            let events = client::call(
                &g.socket,
                &json!({"auth":g.auth,"project":g.project,"native_ui":{"method":"corenius/events","params":{"after":after}}}),
            )?;
            for event in events.as_array().ok_or("UI event projection")? {
                after = event["seq"].as_u64().ok_or("UI replay cursor")?;
                let v = &event["document"];
                if v["method"] != "item/tool/call" && v.get("id").is_none() {
                    send(&mut stream, v)?;
                }
            }
        }
    }
}
pub fn run(args: &[String]) -> Result<(), String> {
    let socket = args.get(2).ok_or("UI controller socket required")?.clone();
    let project = args.get(3).ok_or("UI fixed project required")?.clone();
    let address: SocketAddr = args
        .get(4)
        .ok_or("UI loopback address required")?
        .parse()
        .map_err(|_| "UI address must be numeric loopback")?;
    if !address.ip().is_loopback() {
        return Err("UI gateway must bind loopback".into());
    }
    let token =
        std::env::var("CORENIUS_UI_TOKEN").map_err(|_| "separate CORENIUS_UI_TOKEN required")?;
    let client_token = std::env::var("CORENIUS_CLIENT_TOKEN")
        .map_err(|_| "bound CORENIUS_CLIENT_TOKEN required")?;
    if token.len() < 12 || token == client_token {
        return Err("distinct private UI capability required".into());
    }
    let principal = std::env::var("CORENIUS_CLIENT_PRINCIPAL")
        .map_err(|_| "bound CORENIUS_CLIENT_PRINCIPAL required")?;
    let g = Arc::new(Gateway {
        socket,
        project,
        auth: json!({"principal":principal,"token":client_token}),
        token_hash: hash(&token),
    });
    let listener = TcpListener::bind(address).map_err(|e| e.to_string())?;
    println!(
        "Gateway listening {}",
        listener.local_addr().map_err(|e| e.to_string())?
    );
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let clients = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    for stream in listener.incoming() {
        let stream = stream.map_err(|e| e.to_string())?;
        use std::sync::atomic::Ordering;
        if clients.fetch_add(1, Ordering::AcqRel) >= 4 {
            clients.fetch_sub(1, Ordering::AcqRel);
            continue;
        }
        let g = g.clone();
        let active = clients.clone();
        std::thread::spawn(move || {
            if let Err(error) = serve(stream, g) {
                audit(json!({"connection_disposition":error}));
            }
            active.fetch_sub(1, Ordering::AcqRel);
        });
    }
    Ok(())
}

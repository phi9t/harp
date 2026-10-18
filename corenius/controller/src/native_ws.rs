//! Bounded authenticated native WebSocket transport over a fixed qualified Unix bridge.
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use sha1::{Digest, Sha1};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::Path,
    time::{Duration, Instant},
};
use uuid::Uuid;
#[derive(Clone, Debug)]
pub(crate) struct NativeTurn {
    pub thread: String,
    pub turn: String,
    pub active: bool,
    pub client_user_message_id: Option<String>,
}
type ToolHandler<'a> = dyn FnMut(&Value, Option<&NativeTurn>) -> Result<Value, String> + 'a;
#[cfg(test)]
const RPC_TIMEOUT: Duration = Duration::from_millis(20);
#[cfg(not(test))]
const RPC_TIMEOUT: Duration = Duration::from_secs(8);
fn read_until(stream: &mut UnixStream, bytes: &mut [u8], deadline: Instant) -> Result<(), String> {
    let mut offset = 0;
    while offset < bytes.len() {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or("native RPC deadline; acknowledgement unknown")?;
        stream
            .set_read_timeout(Some(remaining))
            .map_err(|e| e.to_string())?;
        match stream.read(&mut bytes[offset..]) {
            Ok(0) => return Err("native connection closed".into()),
            Ok(n) => offset += n,
            Err(e) if matches!(e.kind(), std::io::ErrorKind::Interrupted) => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}
fn write_until(stream: &mut UnixStream, bytes: &[u8], deadline: Instant) -> Result<(), String> {
    let mut offset = 0;
    while offset < bytes.len() {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or("native write deadline; acknowledgement unknown")?;
        stream
            .set_write_timeout(Some(remaining))
            .map_err(|e| e.to_string())?;
        match stream.write(&bytes[offset..]) {
            Ok(0) => return Err("native connection closed".into()),
            Ok(n) => offset += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}
fn event_thread(event: &Value) -> Option<&str> {
    event["params"]["threadId"]
        .as_str()
        .or_else(|| event["params"]["thread"]["id"].as_str())
}
pub struct NativeWs {
    stream: UnixStream,
    sequence: u64,
    pending: Vec<Value>,
    threads: std::collections::HashSet<String>,
    turns: std::collections::HashMap<String, NativeTurn>,
    pub identity: String,
}
impl NativeWs {
    #[cfg(test)]
    pub(crate) fn owned_test_pair() -> (Self, UnixStream) {
        let (stream, peer) = UnixStream::pair().unwrap();
        (
            Self {
                stream,
                sequence: 0,
                pending: vec![],
                threads: Default::default(),
                turns: Default::default(),
                identity: "owned-dispatcher-no-rpc".into(),
            },
            peer,
        )
    }
    pub fn connect(path: &Path, token: &str, start: &str) -> Result<Self, String> {
        Self::connect_with_timeout(path, token, start, RPC_TIMEOUT)
    }
    pub fn require_ready(path: &Path, token: &str, start: &str) -> Result<(), String> {
        Self::connect_with_timeout(path, token, start, Duration::from_millis(200)).map(|_| ())
    }
    fn connect_with_timeout(
        path: &Path,
        token: &str,
        start: &str,
        timeout: Duration,
    ) -> Result<Self, String> {
        if start.parse::<u64>().is_err() || token.contains(['\r', '\n']) {
            return Err("native connection identity refused".into());
        }
        let mut stream = UnixStream::connect(path).map_err(|e| e.to_string())?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(|e| e.to_string())?;
        let key = STANDARD.encode(Uuid::new_v4().as_bytes());
        write!(stream,"GET /?corenius_start={start} HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: {key}\r\nAuthorization: Bearer {token}\r\n\r\n").map_err(|e|e.to_string())?;
        let upgrade_deadline = Instant::now() + timeout;
        let mut header = Vec::new();
        let mut byte = [0];
        while !header.ends_with(b"\r\n\r\n") {
            if header.len() > 16384 {
                return Err("native upgrade bound".into());
            }
            read_until(&mut stream, &mut byte, upgrade_deadline)?;
            header.push(byte[0]);
        }
        let header = String::from_utf8(header).map_err(|e| e.to_string())?;
        let accept = STANDARD.encode(Sha1::digest(format!(
            "{key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
        )));
        if !header.starts_with("HTTP/1.1 101 ")
            || !header.lines().any(|l| {
                l.split_once(':').is_some_and(|(k, v)| {
                    k.eq_ignore_ascii_case("sec-websocket-accept") && v.trim() == accept
                })
            })
        {
            return Err("native websocket upgrade refused".into());
        }
        Ok(Self {
            stream,
            sequence: 0,
            pending: Vec::new(),
            threads: std::collections::HashSet::new(),
            turns: std::collections::HashMap::new(),
            identity: Uuid::new_v4().to_string(),
        })
    }
    pub fn retain_threads(
        &mut self,
        threads: std::collections::HashSet<String>,
    ) -> Result<(), String> {
        if threads.len() > 1024 {
            return Err("native active thread identity bound".into());
        }
        self.pending
            .retain(|event| event_thread(event).is_some_and(|id| threads.contains(id)));
        self.turns.retain(|thread, _| threads.contains(thread));
        self.threads = threads;
        Ok(())
    }
    pub fn allow_thread(&mut self, thread: &str) -> Result<(), String> {
        if !self.threads.contains(thread) && self.threads.len() >= 1024 {
            return Err("native active thread identity bound".into());
        }
        self.threads.insert(thread.into());
        Ok(())
    }
    pub(crate) fn tool_turn(&self, call: &Value) -> Option<&NativeTurn> {
        self.turns.get(event_thread(call)?)
    }
    fn observe_turn(&mut self, thread: &str, turn: &Value, client: Option<String>) {
        let Some(id) = turn["id"].as_str().filter(|s| !s.is_empty()) else {
            return;
        };
        let client = client.or_else(|| {
            turn["items"].as_array().and_then(|items| {
                items
                    .iter()
                    .find_map(|item| item["clientId"].as_str().map(str::to_owned))
            })
        });
        self.turns.insert(
            thread.to_owned(),
            NativeTurn {
                thread: thread.to_owned(),
                turn: id.to_owned(),
                active: turn["status"] == "inProgress",
                client_user_message_id: client,
            },
        );
    }
    fn send(&mut self, opcode: u8, bytes: &[u8], deadline: Instant) -> Result<(), String> {
        let mask = *Uuid::new_v4().as_bytes();
        let mut frame = vec![0x80 | opcode];
        if bytes.len() < 126 {
            frame.push(0x80 | bytes.len() as u8)
        } else if bytes.len() <= 65535 {
            frame.push(0xfe);
            frame.extend_from_slice(&(bytes.len() as u16).to_be_bytes())
        } else {
            frame.push(0xff);
            frame.extend_from_slice(&(bytes.len() as u64).to_be_bytes())
        }
        frame.extend_from_slice(&mask[..4]);
        frame.extend(bytes.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]));
        write_until(&mut self.stream, &frame, deadline)
    }
    fn receive(&mut self, deadline: Instant) -> Result<Value, String> {
        let mut body = Vec::new();
        let mut started = false;
        loop {
            let mut header = [0; 2];
            read_until(&mut self.stream, &mut header, deadline)?;
            if header[0] & 0x70 != 0 || header[1] & 0x80 != 0 {
                return Err("native frame flags refused".into());
            }
            let opcode = header[0] & 15;
            let mut len = (header[1] & 127) as u64;
            if len == 126 {
                let mut b = [0; 2];
                read_until(&mut self.stream, &mut b, deadline)?;
                len = u16::from_be_bytes(b) as u64
            } else if len == 127 {
                let mut b = [0; 8];
                read_until(&mut self.stream, &mut b, deadline)?;
                len = u64::from_be_bytes(b)
            }
            if len > 4 * 1024 * 1024 || body.len() + len as usize > 4 * 1024 * 1024 {
                return Err("native frame bound".into());
            }
            let mut payload = vec![0; len as usize];
            read_until(&mut self.stream, &mut payload, deadline)?;
            match opcode {
                8 => return Err("native connection closed".into()),
                9 => {
                    self.send(10, &payload, deadline)?;
                    continue;
                }
                10 => continue,
                1 if !started => started = true,
                0 if started => {}
                _ => return Err("native opcode refused".into()),
            }
            body.extend(payload);
            if header[0] & 0x80 != 0 {
                return serde_json::from_slice(&body).map_err(|e| e.to_string());
            }
        }
    }
    pub fn rpc(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.rpc_before(method, params, Instant::now() + RPC_TIMEOUT)
    }
    pub(crate) fn rpc_with_tools(
        &mut self,
        method: &str,
        params: Value,
        handler: &mut ToolHandler<'_>,
    ) -> Result<Value, String> {
        self.rpc_before_with(method, params, Instant::now() + RPC_TIMEOUT, Some(handler))
    }
    pub fn reply_tool(&mut self, request: &Value, result: Value) -> Result<(), String> {
        #[cfg(feature = "fault-injection")]
        if request["params"]["tool"] == "corenius_command"
            && request["params"]["arguments"]["kind"] == "ack_question"
            && let Ok(path) = std::env::var("CORENIUS_TEST_LOSE_TOOL_REPLY")
            && Path::new(&path).is_file()
        {
            std::fs::remove_file(&path).map_err(|e| e.to_string())?;
            std::fs::write(
                format!("{path}.observed"),
                json!({"request":request,"reply_discarded_before_send":true}).to_string(),
            )
            .map_err(|e| e.to_string())?;
            // Native is still waiting. Retain the same upstream request so the next
            // reconciliation sends the committed cached reply, without a new effect.
            self.pending.push(request.clone());
            return Ok(());
        }
        self.send(
            1,
            &serde_json::to_vec(&json!({"id":request["id"],"result":result}))
                .map_err(|e| e.to_string())?,
            Instant::now() + RPC_TIMEOUT,
        )
    }
    pub fn thread_is_loaded(&mut self, thread: &str) -> Result<bool, String> {
        let deadline = Instant::now() + RPC_TIMEOUT;
        let mut cursor = Value::Null;
        let mut seen = std::collections::HashSet::new();
        let mut count = 0;
        loop {
            let page = self.rpc_before(
                "thread/loaded/list",
                json!({"limit":100,"cursor":cursor}),
                deadline,
            )?;
            let ids = page["data"]
                .as_array()
                .ok_or("native loaded page data missing")?;
            if ids.iter().any(|id| id.as_str().is_none_or(str::is_empty)) {
                return Err("native loaded page identity malformed".into());
            }
            count += ids.len();
            if count > 10_000 || seen.len() >= 100 {
                return Err("native loaded pagination resource bound; presence unknown".into());
            }
            let next = match page.get("nextCursor") {
                None | Some(Value::Null) => None,
                Some(Value::String(value)) if !value.is_empty() && value.len() <= 4096 => {
                    Some(value.clone())
                }
                _ => return Err("native loaded cursor malformed".into()),
            };
            if let Some(next) = &next
                && !seen.insert(next.clone())
            {
                return Err("native loaded cursor repeated".into());
            }
            if ids.iter().any(|id| id == thread) {
                return Ok(true);
            }
            match next {
                None => return Ok(false),
                Some(next) => cursor = json!(next),
            }
        }
    }
    fn rpc_before(
        &mut self,
        method: &str,
        params: Value,
        deadline: Instant,
    ) -> Result<Value, String> {
        self.rpc_before_with(method, params, deadline, None)
    }
    fn rpc_before_with(
        &mut self,
        method: &str,
        params: Value,
        deadline: Instant,
        mut handler: Option<&mut ToolHandler<'_>>,
    ) -> Result<Value, String> {
        self.sequence += 1;
        let owner = params["threadId"].as_str().map(str::to_owned);
        let starting_client = if method == "turn/start" {
            params["clientUserMessageId"].as_str().map(str::to_owned)
        } else {
            None
        };
        #[cfg(feature = "fault-injection")]
        let early_reply = std::env::var("CORENIUS_TEST_EARLY_QUESTION")
            .ok()
            .filter(|p| {
                method == "turn/start"
                    && params["input"].to_string().contains("CORENIUS_QUESTIONS")
                    && Path::new(p).is_file()
            });
        #[cfg(feature = "fault-injection")]
        let routing = std::env::var("CORENIUS_TEST_TOOL_ROUTING")
            .ok()
            .and_then(|path| {
                let raw = std::fs::read(format!("{path}.armed")).ok()?;
                let target: Value = serde_json::from_slice(&raw).ok()?;
                (method == "thread/read" && target["owner"].as_str() == owner.as_deref())
                    .then_some((path, target))
            });
        #[cfg(feature = "fault-injection")]
        let mut deferred = None::<Value>;
        #[cfg(feature = "fault-injection")]
        let mut early_acknowledged = false;
        let id = format!("controller-{}", self.sequence);
        self.send(
            1,
            &serde_json::to_vec(&json!({"id":id,"method":method,"params":params}))
                .map_err(|e| e.to_string())?,
            deadline,
        )?;
        let mut starting = std::collections::HashSet::new();
        loop {
            if Instant::now() > deadline {
                return Err("native RPC deadline; acknowledgement unknown".into());
            }
            let value = self.receive(deadline)?;
            if let Some(thread) = event_thread(&value).map(str::to_owned)
                && self.threads.contains(&thread)
            {
                if value["method"] == "turn/started" {
                    let client = if owner.as_deref() == Some(&thread) {
                        starting_client.clone()
                    } else {
                        None
                    };
                    self.observe_turn(&thread, &value["params"]["turn"], client);
                } else if value["method"] == "turn/completed"
                    && self
                        .turns
                        .get(&thread)
                        .is_none_or(|turn| value["params"]["turn"]["id"] == turn.turn)
                {
                    self.observe_turn(&thread, &value["params"]["turn"], None);
                }
            }
            if value["method"] == "item/tool/call"
                && event_thread(&value) == owner.as_deref()
                && owner
                    .as_ref()
                    .is_some_and(|thread| self.threads.contains(thread))
                && let Some(handler) = handler.as_mut()
            {
                let result = handler(&value, self.tool_turn(&value)).unwrap_or_else(
                    |e| json!({"contentItems":[{"type":"inputText","text":e}],"success":false}),
                );
                #[cfg(feature = "fault-injection")]
                let early_ack = value["params"]["arguments"]["kind"] == "ack_question"
                    && result["success"] == true;
                #[cfg(feature = "fault-injection")]
                if early_ack {
                    early_acknowledged = true;
                }
                self.reply_tool(&value, result)?;
                #[cfg(feature = "fault-injection")]
                if early_ack
                    && let Some(path) = &early_reply
                    && let Some(result) = deferred.take()
                {
                    std::fs::remove_file(path).map_err(|e| e.to_string())?;
                    std::fs::write(format!("{path}.observed"),json!({"actual_native_reply_deferred_before_adapter_return":true,"native_request":id,"ack_request":value,"client_user_message_id":starting_client}).to_string()).map_err(|e|e.to_string())?;
                    return Ok(result);
                }
                continue;
            }
            if value.get("method").is_none() && value["id"] == id {
                if let Some(error) = value.get("error") {
                    return Err(format!("native RPC refused: {error}"));
                }
                let result = value
                    .get("result")
                    .cloned()
                    .ok_or("native reply result missing")?;
                if method == "thread/start" {
                    let thread = result["thread"]["id"]
                        .as_str()
                        .filter(|id| !id.is_empty())
                        .ok_or("native created thread identity missing")?;
                    self.allow_thread(thread)?;
                    self.retain_threads(self.threads.clone())?;
                }
                if let Some(thread) = owner
                    .as_deref()
                    .filter(|thread| self.threads.contains(*thread))
                {
                    if method == "turn/start" {
                        self.observe_turn(thread, &result["turn"], starting_client.clone());
                    } else if matches!(method, "thread/read" | "thread/resume")
                        && result["thread"]["id"] == thread
                        && let Some(turn) = result["thread"]["turns"]
                            .as_array()
                            .and_then(|turns| turns.last())
                    {
                        self.observe_turn(thread, turn, None);
                    }
                }
                #[cfg(feature = "fault-injection")]
                if early_reply.is_some() || routing.is_some() {
                    if let Some(path) = &early_reply
                        && early_acknowledged
                    {
                        std::fs::remove_file(path).map_err(|e| e.to_string())?;
                        std::fs::write(format!("{path}.observed"),json!({"actual_tool_before_native_start_reply":true,"native_request":id,"client_user_message_id":starting_client}).to_string()).map_err(|e|e.to_string())?;
                        return Ok(result);
                    }
                    if let Some((path, target)) = &routing
                        && let Some(event) = self.pending.iter().find(|event| {
                            event["method"] == "item/tool/call"
                                && event_thread(event) == target["peer"].as_str()
                        })
                    {
                        std::fs::write(format!("{path}.peer-queued"),json!({"owner_rpc_thread":owner,"peer_request":event,"reply_sent_by_wrong_binding":false,"connection":self.identity}).to_string()).map_err(|e|e.to_string())?;
                        std::fs::remove_file(format!("{path}.armed")).map_err(|e| e.to_string())?;
                        return Ok(result);
                    }
                    if let Some((path, _)) = &routing {
                        std::fs::write(format!("{path}.owner-observed"),json!({"owner":owner,"native_request":id,"real_reply_deferred":true,"connection":self.identity}).to_string()).map_err(|e|e.to_string())?;
                    }
                    deferred = Some(result);
                    continue;
                }
                return Ok(result);
            }
            #[cfg(feature = "fault-injection")]
            if value.get("method").is_some()
                && let Ok(path) = std::env::var("CORENIUS_TEST_ADOPT_EVENTS")
                && Path::new(&path).is_file()
            {
                use std::fs::OpenOptions;
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(format!("{path}.received"))
                    .map_err(|e| e.to_string())?;
                writeln!(file, "{value}").map_err(|e| e.to_string())?;
            }
            #[cfg(feature = "fault-injection")]
            if value["method"] == "thread/name/updated"
                && let Ok(path) = std::env::var("CORENIUS_TEST_RETIRED_EVENTS")
            {
                use std::fs::OpenOptions;
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(format!("{path}.received"))
                    .map_err(|e| e.to_string())?;
                writeln!(file, "{value}").map_err(|e| e.to_string())?;
            }
            if method == "thread/start"
                && value["method"] == "thread/started"
                && let Some(thread) = event_thread(&value)
            {
                starting.insert(thread.to_owned());
            }
            if !event_thread(&value)
                .is_some_and(|thread| self.threads.contains(thread) || starting.contains(thread))
            {
                continue;
            }
            self.pending.push(value);
            if self.pending.len() > 1024 {
                return Err("native event buffer bound".into());
            }
            #[cfg(feature = "fault-injection")]
            if let Some((path, target)) = &routing
                && let Some(event) = self.pending.last()
                && event["method"] == "item/tool/call"
                && event_thread(event) == target["peer"].as_str()
                && let Some(result) = deferred.take()
            {
                std::fs::write(format!("{path}.peer-queued"),json!({"owner_rpc_thread":owner,"peer_request":event,"reply_sent_by_wrong_binding":false,"connection":self.identity}).to_string()).map_err(|e|e.to_string())?;
                std::fs::remove_file(format!("{path}.armed")).map_err(|e| e.to_string())?;
                return Ok(result);
            }
        }
    }
    pub fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        self.send(
            1,
            &serde_json::to_vec(&json!({"method":method,"params":params}))
                .map_err(|e| e.to_string())?,
            Instant::now() + RPC_TIMEOUT,
        )
    }
    #[cfg(feature = "fault-injection")]
    pub fn buffered_events(&self) -> &[Value] {
        &self.pending
    }
    pub fn take_events_for(&mut self, thread: &str) -> Vec<Value> {
        let mut selected = Vec::new();
        self.pending.retain(|event| {
            let identity = event["params"]["threadId"]
                .as_str()
                .or_else(|| event["params"]["thread"]["id"].as_str());
            match identity {
                Some(identity) if identity == thread => {
                    selected.push(event.clone());
                    false
                }
                Some(_) => true,
                None => false,
            }
        });
        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request_frame(peer: &mut UnixStream) -> Value {
        let mut header = [0; 2];
        peer.read_exact(&mut header).unwrap();
        let mut len = (header[1] & 127) as usize;
        if len == 126 {
            let mut b = [0; 2];
            peer.read_exact(&mut b).unwrap();
            len = u16::from_be_bytes(b) as usize;
        }
        let mut mask = [0; 4];
        peer.read_exact(&mut mask).unwrap();
        let mut bytes = vec![0; len];
        peer.read_exact(&mut bytes).unwrap();
        for (i, b) in bytes.iter_mut().enumerate() {
            *b ^= mask[i % 4];
        }
        serde_json::from_slice(&bytes).unwrap()
    }
    fn response_frame(peer: &mut UnixStream, value: Value) {
        let body = serde_json::to_vec(&value).unwrap();
        let mut frame = vec![0x81];
        if body.len() < 126 {
            frame.push(body.len() as u8);
        } else {
            frame.push(126);
            frame.extend_from_slice(&(body.len() as u16).to_be_bytes());
        }
        frame.extend(body);
        peer.write_all(&frame).unwrap();
    }
    #[test]
    fn peer_tool_request_during_owner_rpc_is_queued_without_wrong_owner_reply() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let (release, hold) = std::sync::mpsc::channel();
        let producer = std::thread::spawn(move || {
            let request = request_frame(&mut peer);
            response_frame(
                &mut peer,
                json!({"id":91,"method":"item/tool/call","params":{"threadId":"peer","turnId":"peer-turn","callId":"peer-call","tool":"corenius_query","arguments":{"kind":"own_assignment"}}}),
            );
            response_frame(
                &mut peer,
                json!({"id":request["id"],"result":{"thread":{"id":"owner","turns":[]}}}),
            );
            hold.recv_timeout(Duration::from_secs(2)).unwrap();
        });
        let mut ws = NativeWs {
            stream,
            sequence: 0,
            pending: Vec::new(),
            threads: ["owner".to_owned(), "peer".to_owned()]
                .into_iter()
                .collect(),
            turns: Default::default(),
            identity: "owned-interleaving-wire".into(),
        };
        let mut handled = Vec::new();
        let mut handler = |call: &Value, _: Option<&NativeTurn>| {
            handled.push(call["params"]["threadId"].clone());
            Ok(json!({"success":true,"contentItems":[]}))
        };
        ws.rpc_before_with(
            "thread/read",
            json!({"threadId":"owner","includeTurns":true}),
            Instant::now() + Duration::from_secs(1),
            Some(&mut handler),
        )
        .unwrap();
        release.send(()).unwrap();
        producer.join().unwrap();
        assert!(
            handled.is_empty(),
            "peer must never invoke the current owner's handler: {handled:?}"
        );
        assert_eq!(ws.take_events_for("peer").len(), 1);
        assert!(ws.take_events_for("owner").is_empty());
    }
    #[test]
    fn typed_started_turn_correlates_tool_before_native_start_reply() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let producer = std::thread::spawn(move || {
            let request = request_frame(&mut peer);
            response_frame(
                &mut peer,
                json!({"method":"turn/started","params":{"threadId":"owner","turn":{"id":"early-turn","status":"inProgress"}}}),
            );
            response_frame(
                &mut peer,
                json!({"id":42,"method":"item/tool/call","params":{"threadId":"owner","turnId":"early-turn","callId":"early-call","tool":"corenius_query","arguments":{"kind":"control_center"}}}),
            );
            let tool_reply = request_frame(&mut peer);
            assert_eq!(tool_reply["id"], 42);
            assert_eq!(tool_reply["result"]["success"], true);
            response_frame(
                &mut peer,
                json!({"id":request["id"],"result":{"turn":{"id":"early-turn","status":"inProgress"}}}),
            );
        });
        let mut ws = NativeWs {
            stream,
            sequence: 0,
            pending: Vec::new(),
            threads: ["owner".into()].into_iter().collect(),
            turns: Default::default(),
            identity: "owned-early-wire".into(),
        };
        let mut handler = |call: &Value, origin: Option<&NativeTurn>| {
            let origin = origin.unwrap();
            assert_eq!(origin.turn, "early-turn");
            assert!(origin.active);
            assert_eq!(
                origin.client_user_message_id.as_deref(),
                Some("admitted-operation")
            );
            assert_eq!(call["params"]["turnId"], origin.turn);
            Ok(json!({"success":true,"contentItems":[]}))
        };
        ws.rpc_before_with(
            "turn/start",
            json!({"threadId":"owner","clientUserMessageId":"admitted-operation"}),
            Instant::now() + Duration::from_secs(1),
            Some(&mut handler),
        )
        .unwrap();
        producer.join().unwrap();
    }
    // Declared transport fixtures, not native protocol qualification: malformed
    // wire data must fail closed and an exhausted valid scan must report absence.
    fn scan_pages(pages: Vec<Value>) -> Result<bool, String> {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        let producer = std::thread::spawn(move || {
            for (index, page) in pages.into_iter().enumerate() {
                let mut header = [0; 2];
                peer.read_exact(&mut header).unwrap();
                let mut len = (header[1] & 127) as usize;
                if len == 126 {
                    let mut b = [0; 2];
                    peer.read_exact(&mut b).unwrap();
                    len = u16::from_be_bytes(b) as usize;
                }
                let mut mask = [0; 4];
                peer.read_exact(&mut mask).unwrap();
                let mut request = vec![0; len];
                peer.read_exact(&mut request).unwrap();
                for (i, byte) in request.iter_mut().enumerate() {
                    *byte ^= mask[i % 4];
                }
                let request: Value = serde_json::from_slice(&request).unwrap();
                assert_eq!(request["method"], "thread/loaded/list");
                let body = serde_json::to_vec(
                    &json!({"id":format!("controller-{}",index+1),"result":page}),
                )
                .unwrap();
                let mut frame = vec![0x81];
                if body.len() < 126 {
                    frame.push(body.len() as u8);
                } else {
                    frame.push(126);
                    frame.extend_from_slice(&(body.len() as u16).to_be_bytes());
                }
                frame.extend(body);
                peer.write_all(&frame).unwrap();
            }
        });
        let mut ws = NativeWs {
            stream,
            sequence: 0,
            pending: Vec::new(),
            threads: std::collections::HashSet::new(),
            turns: std::collections::HashMap::new(),
            identity: "owned-wire-pagination".into(),
        };
        let result = ws.thread_is_loaded("absent");
        drop(ws);
        producer.join().unwrap();
        result
    }
    #[test]
    fn loaded_scan_exhaustion_reports_absent_identity() {
        assert!(
            !scan_pages(vec![
                json!({"data":["a"],"nextCursor":"next"}),
                json!({"data":["b"],"nextCursor":null})
            ])
            .unwrap()
        );
    }
    #[test]
    fn loaded_scan_repeated_cursor_fails_closed() {
        assert!(
            scan_pages(vec![
                json!({"data":[],"nextCursor":"loop"}),
                json!({"data":[],"nextCursor":"loop"})
            ])
            .unwrap_err()
            .contains("cursor repeated")
        );
    }
    #[test]
    fn loaded_scan_malformed_cursor_fails_closed() {
        assert!(
            scan_pages(vec![json!({"data":[],"nextCursor":42})])
                .unwrap_err()
                .contains("cursor malformed")
        );
    }
    #[test]
    fn partial_native_frame_cannot_extend_absolute_rpc_deadline() {
        let (stream, mut peer) = UnixStream::pair().unwrap();
        stream.set_read_timeout(Some(RPC_TIMEOUT)).unwrap();
        let producer = std::thread::spawn(move || {
            let payload = b"{\"id\":\"controller-1\",\"result\":{}}";
            peer.write_all(&[0x81, payload.len() as u8]).unwrap();
            for byte in payload {
                std::thread::sleep(Duration::from_millis(3));
                if peer.write_all(&[*byte]).is_err() {
                    break;
                }
            }
        });
        let mut ws = NativeWs {
            stream,
            sequence: 0,
            pending: Vec::new(),
            threads: std::collections::HashSet::new(),
            turns: std::collections::HashMap::new(),
            identity: "owned-wire-fixture".into(),
        };
        let started = Instant::now();
        let result = ws.rpc("thread/read", json!({}));
        assert!(
            result.is_err(),
            "partial frame must not outlive absolute native RPC deadline: {result:?}"
        );
        assert!(started.elapsed() < Duration::from_millis(80));
        drop(ws);
        producer.join().unwrap();
    }
}

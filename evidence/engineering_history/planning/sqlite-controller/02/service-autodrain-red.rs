//! One newline-delimited JSON request per local connection; workers never open SQLite.
use crate::{Controller, runtime::Runtime};
use serde_json::json;
use std::{
    fs,
    io::{Read, Write},
    os::unix::{
        fs::{FileTypeExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::Path,
    time::{Duration, Instant},
};
const MAX_REQUEST: usize = 1_048_576;
const REQUEST_DEADLINE: Duration = Duration::from_secs(1);

/// Serve until process shutdown. A stale socket may be removed only after proving no listener.
/// Keep the socket and database directory outside worker mounts.
pub fn serve(controller: Controller, socket: impl AsRef<Path>) -> Result<(), String> {
    serve_with_runtime(controller, socket, None)
}
pub fn serve_with_runtime(
    controller: Controller,
    socket: impl AsRef<Path>,
    runtime: Option<Runtime>,
) -> Result<(), String> {
    let socket = socket.as_ref();
    if let Ok(metadata) = fs::symlink_metadata(socket) {
        if !metadata.file_type().is_socket() {
            return Err("socket path exists and is not a socket".into());
        }
        match UnixStream::connect(socket) {
            Ok(_) => return Err("socket already has a live listener".into()),
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
                fs::remove_file(socket).map_err(|e| e.to_string())?
            }
            Err(e) => return Err(format!("cannot establish stale socket: {e}")),
        }
    }
    let listener = UnixListener::bind(socket).map_err(|e| format!("socket bind: {e}"))?;
    fs::set_permissions(socket, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let mut next_reconcile = Instant::now();
    let autonomous = {
        let enabled = runtime.is_some();
        #[cfg(feature = "fault-injection")]
        let enabled =
            enabled && std::env::var("CORENIUS_TEST_MANUAL_RUNTIME").as_deref() != Ok("1");
        enabled
    };
    loop {
        if autonomous && false && Instant::now() >= next_reconcile {
            if let Err(error) = runtime.as_ref().unwrap().drain(&controller) {
                eprintln!("runtime drain: {error}");
            }
            next_reconcile = Instant::now() + Duration::from_millis(250);
        }
        let stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(error) => return Err(format!("socket accept: {error}")),
        };
        if let Err(e) = serve_request(&controller, runtime.as_ref(), stream) {
            eprintln!("connection: {e}");
        }
    }
}
fn serve_request(
    controller: &Controller,
    runtime: Option<&Runtime>,
    mut stream: UnixStream,
) -> Result<(), String> {
    let deadline = Instant::now() + REQUEST_DEADLINE;
    let response = match read_request(&mut stream, deadline) {
        Ok(None) => return Ok(()),
        Err(error) => json!({"error":error}),
        Ok(Some(bytes)) => match serde_json::from_slice(&bytes) {
            Ok(request) => match route(controller, runtime, request) {
                Ok(value) => json!({"ok":value}),
                Err(error) => json!({"error":error}),
            },
            Err(_) => json!({"error":"invalid JSON"}),
        },
    };
    let bytes = format!("{response}\n").into_bytes();
    let mut offset = 0;
    while offset < bytes.len() {
        stream
            .set_write_timeout(Some(remaining(deadline)?))
            .map_err(|e| e.to_string())?;
        let n = stream.write(&bytes[offset..]).map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("response transport closed".into());
        }
        offset += n;
    }
    Ok(())
}
fn route(
    controller: &Controller,
    runtime: Option<&Runtime>,
    request: serde_json::Value,
) -> Result<serde_json::Value, String> {
    if request.get("runtime").is_some() {
        let fields = request.as_object().ok_or("invalid runtime request")?;
        if fields.len() != 3 || request["runtime"] != json!({"kind":"reconcile"}) {
            return Err("runtime accepts only authenticated project reconciliation".into());
        }
        return runtime
            .ok_or("runtime not configured")?
            .reconcile(controller, &request);
    }
    let kind = request["command"]["kind"].as_str();
    if kind == Some("transition_operation")
        || (kind == Some("observe_attempt")
            && (runtime.is_some() || request["command"]["args"]["state"] == "exited"))
    {
        return Err("lifecycle observations require controller-owned runtime evidence".into());
    }
    controller.handle(request)
}
fn remaining(deadline: Instant) -> Result<Duration, String> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|d| !d.is_zero())
        .ok_or_else(|| "whole request deadline exceeded".into())
}
pub(crate) fn read_request(
    stream: &mut UnixStream,
    deadline: Instant,
) -> Result<Option<Vec<u8>>, String> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        stream
            .set_read_timeout(Some(remaining(deadline)?))
            .map_err(|e| e.to_string())?;
        let n = stream
            .read(&mut buffer)
            .map_err(|e| format!("request read deadline or transport error: {e}"))?;
        if n == 0 {
            return if bytes.is_empty() {
                Ok(None)
            } else {
                Err("request framing limit".into())
            };
        }
        if let Some(end) = buffer[..n].iter().position(|b| *b == b'\n') {
            bytes.extend_from_slice(&buffer[..=end]);
            if bytes.len() > MAX_REQUEST {
                return Err("request size limit".into());
            }
            return Ok(Some(bytes));
        }
        bytes.extend_from_slice(&buffer[..n]);
        if bytes.len() > MAX_REQUEST {
            return Err("request size limit".into());
        }
    }
}

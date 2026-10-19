//! Bounded Unix-domain transport to the existing, authoritative Insula owner.
//!
//! A successful claim leaves its connection open. The owner MUST fence that
//! epoch on EOF, before admitting a replacement controller. Ordinary requests
//! use independent connections so a stalled probe cannot occupy the claim or
//! prevent a stop request from reaching the owner.

use crate::{model::*, Error, ExecutionOwner, Result};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::UnixStream,
    sync::{oneshot, Mutex, Semaphore},
};
use uuid::Uuid;

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_FRAME_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RpcRequest {
    pub version: u16,
    pub request_id: Uuid,
    pub request: OwnerRequest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum OwnerRequest {
    Capabilities {
        root_insula_id: String,
    },
    Claim {
        root_insula_id: String,
        controller_id: Uuid,
    },
    Validate {
        ownership: Ownership,
    },
    Observe {
        ownership: Ownership,
        key: ServiceKey,
        generation: Option<Generation>,
    },
    Launch {
        ownership: Ownership,
        request: Box<LaunchRequest>,
    },
    Stop {
        ownership: Ownership,
        request: StopRequest,
    },
    Probe {
        ownership: Ownership,
        request: ProbeRequest,
    },
    Release {
        ownership: Ownership,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RpcResponse {
    pub version: u16,
    pub request_id: Uuid,
    pub response: OwnerResponse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum OwnerResponse {
    Success { value: OwnerReply },
    Error { error: Error },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum OwnerReply {
    Capabilities(OwnerCapabilities),
    Claimed(Ownership),
    Observation(Observation),
    Probe(ProbeResult),
    Acknowledged,
}

impl RpcResponse {
    pub fn for_request(request: &RpcRequest, result: Result<OwnerReply>) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            request_id: request.request_id,
            response: match result {
                Ok(value) => OwnerResponse::Success { value },
                Err(error) => OwnerResponse::Error { error },
            },
        }
    }
}

/// Framing helpers are shared with owner implementations. Callers impose an
/// overall deadline; neither partial headers nor partial payloads extend it.
pub async fn read_frame<R: AsyncRead + Unpin, T: DeserializeOwned>(reader: &mut R) -> Result<T> {
    let len = reader.read_u32().await.map_err(|_| Error::Unavailable)? as usize;
    if len == 0 || len > MAX_FRAME_BYTES {
        return Err(Error::Invalid(
            "owner frame exceeds the protocol bound".into(),
        ));
    }
    let mut payload = vec![0; len];
    reader
        .read_exact(&mut payload)
        .await
        .map_err(|_| Error::Unavailable)?;
    serde_json::from_slice(&payload).map_err(|_| Error::Invalid("malformed owner frame".into()))
}

pub async fn write_frame<W: AsyncWrite + Unpin, T: Serialize>(
    writer: &mut W,
    value: &T,
) -> Result<()> {
    let payload =
        serde_json::to_vec(value).map_err(|_| Error::Invalid("invalid owner message".into()))?;
    if payload.is_empty() || payload.len() > MAX_FRAME_BYTES {
        return Err(Error::Invalid(
            "owner frame exceeds the protocol bound".into(),
        ));
    }
    writer
        .write_u32(payload.len() as u32)
        .await
        .map_err(|_| Error::Unavailable)?;
    writer
        .write_all(&payload)
        .await
        .map_err(|_| Error::Unavailable)?;
    writer.flush().await.map_err(|_| Error::Unavailable)?;
    Ok(())
}

struct ClaimConnection {
    ownership: Ownership,
    alive: Arc<AtomicBool>,
    close: Option<oneshot::Sender<()>>,
    monitor: tokio::task::JoinHandle<()>,
}

impl Drop for ClaimConnection {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Release);
        if let Some(close) = self.close.take() {
            let _ = close.send(());
        }
        // Cancellation drops the stream immediately even if the runtime is
        // shutting down; the task owns no child process or lifecycle effect.
        self.monitor.abort();
    }
}

/// The socket must refer to the canonical Insula owner. This client never
/// creates a service process or uses a PID fallback if the owner is unavailable.
pub struct OwnerRpc {
    socket: PathBuf,
    expected_uid: u32,
    timeout: Duration,
    claim: Mutex<Option<ClaimConnection>>,
    control_slots: Semaphore,
    probe_slots: Semaphore,
}

impl OwnerRpc {
    pub fn new(socket: PathBuf, expected_uid: u32, timeout: Duration) -> Result<Self> {
        if !socket.is_absolute()
            || socket.as_os_str().len() > 107
            || timeout.is_zero()
            || timeout > Duration::from_secs(120)
        {
            return Err(Error::Invalid(
                "owner socket and timeout must be explicit and bounded".into(),
            ));
        }
        Ok(Self {
            socket,
            expected_uid,
            timeout,
            claim: Mutex::new(None),
            control_slots: Semaphore::new(32),
            probe_slots: Semaphore::new(8),
        })
    }

    async fn connect(&self) -> Result<UnixStream> {
        let stream = UnixStream::connect(&self.socket)
            .await
            .map_err(|_| Error::Unavailable)?;
        if stream.peer_cred().map_err(|_| Error::Unavailable)?.uid() != self.expected_uid {
            return Err(Error::Ownership);
        }
        Ok(stream)
    }

    async fn local_claim(&self, ownership: &Ownership) -> Result<()> {
        let guard = self.claim.lock().await;
        match guard.as_ref() {
            Some(claim) if claim.ownership == *ownership && claim.alive.load(Ordering::Acquire) => {
                Ok(())
            }
            _ => Err(Error::Ownership),
        }
    }

    async fn round_trip(
        &self,
        request: OwnerRequest,
        mutation: bool,
        probe: bool,
    ) -> Result<OwnerReply> {
        let work = async {
            let _permit = if probe {
                self.probe_slots.acquire().await
            } else {
                self.control_slots.acquire().await
            }
            .map_err(|_| Error::Unavailable)?;
            let mut stream = self.connect().await?;
            let request = RpcRequest {
                version: PROTOCOL_VERSION,
                request_id: Uuid::new_v4(),
                request,
            };
            write_frame(&mut stream, &request).await?;
            let response: RpcResponse = read_frame(&mut stream).await?;
            if response.version != PROTOCOL_VERSION || response.request_id != request.request_id {
                return Err(Error::Invalid(
                    "owner response does not match the request".into(),
                ));
            }
            Ok(response.response)
        };
        match tokio::time::timeout(self.timeout, work).await {
            Ok(Ok(OwnerResponse::Success { value })) => Ok(value),
            // A well-formed authenticated owner error is a definite response.
            Ok(Ok(OwnerResponse::Error { error })) => Err(error),
            Ok(Err(Error::Unavailable | Error::Invalid(_))) | Err(_) if mutation => {
                Err(Error::Unknown)
            }
            Ok(Err(error)) => Err(error),
            Err(_) => Err(Error::Unavailable),
        }
    }

    fn decode(response: RpcResponse, request_id: Uuid) -> Result<OwnerReply> {
        if response.version != PROTOCOL_VERSION || response.request_id != request_id {
            return Err(Error::Invalid(
                "owner response does not match the request".into(),
            ));
        }
        match response.response {
            OwnerResponse::Success { value } => Ok(value),
            OwnerResponse::Error { error } => Err(error),
        }
    }
}

#[async_trait]
impl ExecutionOwner for OwnerRpc {
    async fn capabilities(&self, root_insula_id: &str) -> Result<OwnerCapabilities> {
        match self
            .round_trip(
                OwnerRequest::Capabilities {
                    root_insula_id: root_insula_id.into(),
                },
                false,
                false,
            )
            .await?
        {
            OwnerReply::Capabilities(value) if value.root_insula_id == root_insula_id => Ok(value),
            _ => Err(Error::Invalid("unexpected owner capabilities".into())),
        }
    }

    async fn claim(&self, root_insula_id: &str, controller_id: Uuid) -> Result<Ownership> {
        let mut guard = self.claim.lock().await;
        if let Some(claim) = guard.as_ref() {
            if claim.alive.load(Ordering::Acquire) {
                if claim.ownership.root_insula_id == root_insula_id
                    && claim.ownership.controller_id == controller_id
                {
                    return Ok(claim.ownership.clone());
                }
                return Err(Error::Ownership);
            }
        }
        *guard = None;
        let (ownership, mut stream) = tokio::time::timeout(self.timeout, async {
            let mut stream = self.connect().await?;
            let request = RpcRequest {
                version: PROTOCOL_VERSION,
                request_id: Uuid::new_v4(),
                request: OwnerRequest::Claim {
                    root_insula_id: root_insula_id.into(),
                    controller_id,
                },
            };
            write_frame(&mut stream, &request).await?;
            let response = read_frame(&mut stream).await?;
            let ownership = match Self::decode(response, request.request_id)? {
                OwnerReply::Claimed(value)
                    if value.root_insula_id == root_insula_id
                        && value.controller_id == controller_id
                        && value.epoch > 0 =>
                {
                    value
                }
                _ => return Err(Error::Invalid("unexpected owner claim".into())),
            };
            Ok((ownership, stream))
        })
        .await
        .map_err(|_| Error::Ownership)??;
        let alive = Arc::new(AtomicBool::new(true));
        let watch_alive = Arc::clone(&alive);
        let (close, cancel) = oneshot::channel();
        let monitor = tokio::spawn(async move {
            let mut byte = [0u8];
            // No application frames are valid after the claim response. EOF,
            // unexpected bytes, cancellation, and IO errors all lose the claim.
            tokio::select! { _ = stream.read(&mut byte) => {}, _ = cancel => {} }
            watch_alive.store(false, Ordering::Release);
        });
        *guard = Some(ClaimConnection {
            ownership: ownership.clone(),
            alive,
            close: Some(close),
            monitor,
        });
        Ok(ownership)
    }

    async fn validate(&self, ownership: &Ownership) -> Result<()> {
        self.local_claim(ownership).await?;
        match self
            .round_trip(
                OwnerRequest::Validate {
                    ownership: ownership.clone(),
                },
                false,
                false,
            )
            .await?
        {
            OwnerReply::Acknowledged => Ok(()),
            _ => Err(Error::Invalid("unexpected validation response".into())),
        }
    }

    async fn observe(
        &self,
        ownership: &Ownership,
        key: &ServiceKey,
        generation: Option<Generation>,
    ) -> Result<Observation> {
        self.local_claim(ownership).await?;
        match self
            .round_trip(
                OwnerRequest::Observe {
                    ownership: ownership.clone(),
                    key: key.clone(),
                    generation,
                },
                false,
                false,
            )
            .await?
        {
            OwnerReply::Observation(value) => Ok(value),
            _ => Err(Error::Invalid("unexpected observation response".into())),
        }
    }

    async fn launch(&self, ownership: &Ownership, request: LaunchRequest) -> Result<Observation> {
        self.local_claim(ownership).await?;
        match self
            .round_trip(
                OwnerRequest::Launch {
                    ownership: ownership.clone(),
                    request: Box::new(request),
                },
                true,
                false,
            )
            .await?
        {
            OwnerReply::Observation(value) => Ok(value),
            _ => Err(Error::Unknown),
        }
    }

    async fn stop(&self, ownership: &Ownership, request: StopRequest) -> Result<Observation> {
        self.local_claim(ownership).await?;
        match self
            .round_trip(
                OwnerRequest::Stop {
                    ownership: ownership.clone(),
                    request,
                },
                true,
                false,
            )
            .await?
        {
            OwnerReply::Observation(value) => Ok(value),
            _ => Err(Error::Unknown),
        }
    }

    async fn probe(&self, ownership: &Ownership, request: ProbeRequest) -> Result<ProbeResult> {
        self.local_claim(ownership).await?;
        let generation = request.generation;
        match self
            .round_trip(
                OwnerRequest::Probe {
                    ownership: ownership.clone(),
                    request,
                },
                false,
                true,
            )
            .await?
        {
            OwnerReply::Probe(value) if value.generation == generation => Ok(value),
            _ => Err(Error::Invalid("unexpected probe response".into())),
        }
    }

    async fn release(&self, ownership: &Ownership) -> Result<()> {
        self.local_claim(ownership).await?;
        let response = self
            .round_trip(
                OwnerRequest::Release {
                    ownership: ownership.clone(),
                },
                true,
                false,
            )
            .await;
        let mut guard = self.claim.lock().await;
        if guard.as_ref().is_some_and(|c| c.ownership == *ownership) {
            *guard = None;
        }
        match response? {
            OwnerReply::Acknowledged => Ok(()),
            _ => Err(Error::Unknown),
        }
    }
}

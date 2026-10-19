//! Local control transport. Clients may stop waiting without canceling an
//! accepted lifecycle operation. Persistent event cursors allow reconnects.
use crate::{
    model::*,
    owner_rpc::{read_frame, write_frame},
    Error, Result, Supervisor,
};
use serde::{Deserialize, Serialize};
use std::{
    os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    net::{UnixListener, UnixStream},
    sync::{watch, Semaphore},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlRequest {
    pub version: u16,
    pub request: ControlAction,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum ControlAction {
    Mutate {
        request: OperationRequest,
    },
    Inspect {
        key: ServiceKey,
    },
    List,
    Operation {
        operation_id: OperationId,
    },
    WatchEvents {
        after: u64,
        limit: usize,
        wait_ms: u64,
    },
    Capabilities,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ControlReply {
    Status(Box<ServiceStatus>),
    Services(Vec<ServiceStatus>),
    Operation(OperationRecord),
    Events {
        events: Vec<Event>,
        history_gap: bool,
    },
    Capabilities {
        owner: OwnerCapabilities,
        ownership: Ownership,
        ownership_lost: bool,
        shutting_down: bool,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ControlResponse {
    pub version: u16,
    pub result: Result<ControlReply>,
}

pub struct ControlServer {
    listener: UnixListener,
    path: PathBuf,
    inode: u64,
}
impl ControlServer {
    pub fn bind(supervisor: &Supervisor) -> Result<Self> {
        let dir = supervisor.state_dir();
        crate::store::private_directory(dir, false)?;
        let path = dir.join("control.sock");
        if path.as_os_str().len() > 107 {
            return Err(Error::Invalid("control socket path too long".into()));
        }
        if let Ok(m) = std::fs::symlink_metadata(&path) {
            if !m.file_type().is_socket() || m.uid() != rustix::process::geteuid().as_raw() {
                return Err(Error::Ownership);
            }
            // Both owner-authority and store locks were acquired before bind.
            // Still refuse to replace an endpoint that currently accepts clients.
            let probe = rustix::net::socket_with(
                rustix::net::AddressFamily::UNIX,
                rustix::net::SocketType::STREAM,
                rustix::net::SocketFlags::CLOEXEC | rustix::net::SocketFlags::NONBLOCK,
                None,
            )
            .map_err(|_| Error::Io)?;
            let address = rustix::net::SocketAddrUnix::new(&path).map_err(|_| Error::Io)?;
            // A full backlog, pending connect or permission failure is uncertain.
            // Only ECONNREFUSED certifies this existing endpoint has no listener.
            if rustix::net::connect(&probe, &address) != Err(rustix::io::Errno::CONNREFUSED) {
                return Err(Error::Conflict(
                    "control socket is live or its ownership is uncertain".into(),
                ));
            }
            std::fs::remove_file(&path)?;
        }
        let listener = UnixListener::bind(&path)?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        let inode = std::fs::symlink_metadata(&path)?.ino();
        Ok(Self {
            listener,
            path,
            inode,
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub async fn run(
        self,
        supervisor: Arc<Supervisor>,
        mut closing: watch::Receiver<bool>,
    ) -> Result<()> {
        let slots = Arc::new(Semaphore::new(64));
        let mut tasks = tokio::task::JoinSet::new();
        loop {
            tokio::select! {
                changed=closing.changed()=>{if changed.is_err() || *closing.borrow(){break}},
                next=self.listener.accept()=>{
                    let (stream,_)=next?;
                    if stream.peer_cred().map_err(|_|Error::Ownership)?.uid()!=rustix::process::geteuid().as_raw(){continue}
                    let Ok(slot)=slots.clone().try_acquire_owned() else{continue};
                    let supervisor=supervisor.clone();tasks.spawn(async move{let _slot=slot;let _=handle(supervisor,stream).await;});
                },
                _=tasks.join_next(),if !tasks.is_empty()=>{}
            }
        }
        // Connections are waiters only. Supervisor::execute owns admitted work.
        tasks.abort_all();
        while tasks.join_next().await.is_some() {}
        Ok(())
    }
}
impl Drop for ControlServer {
    fn drop(&mut self) {
        if let Ok(m) = std::fs::symlink_metadata(&self.path) {
            if m.ino() == self.inode && m.file_type().is_socket() {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }
}

async fn handle(supervisor: Arc<Supervisor>, mut stream: UnixStream) -> Result<()> {
    let request: ControlRequest =
        tokio::time::timeout(Duration::from_secs(3), read_frame(&mut stream))
            .await
            .map_err(|_| Error::Unavailable)??;
    let result = if request.version != 1 {
        Err(Error::Unsupported("control protocol version".into()))
    } else {
        dispatch(supervisor, request.request).await
    };
    tokio::time::timeout(
        Duration::from_secs(3),
        write_frame(&mut stream, &ControlResponse { version: 1, result }),
    )
    .await
    .map_err(|_| Error::Unavailable)?
}
async fn dispatch(supervisor: Arc<Supervisor>, request: ControlAction) -> Result<ControlReply> {
    match request {
        ControlAction::Mutate { request } => {
            Ok(ControlReply::Operation(supervisor.execute(request).await?))
        }
        ControlAction::Inspect { key } => {
            Ok(ControlReply::Status(Box::new(supervisor.inspect(&key)?)))
        }
        ControlAction::List => Ok(ControlReply::Services(supervisor.list()?)),
        ControlAction::Operation { operation_id } => {
            Ok(ControlReply::Operation(supervisor.operation(operation_id)?))
        }
        ControlAction::Capabilities => Ok(ControlReply::Capabilities {
            owner: supervisor.capabilities().clone(),
            ownership: supervisor.ownership().clone(),
            ownership_lost: supervisor.has_lost_ownership(),
            shutting_down: supervisor.is_shutting_down(),
        }),
        ControlAction::WatchEvents {
            after,
            limit,
            wait_ms,
        } => {
            if limit == 0 || limit > 1024 || wait_ms > 5000 {
                return Err(Error::Invalid("invalid event page or wait bound".into()));
            }
            let deadline = tokio::time::Instant::now() + Duration::from_millis(wait_ms);
            loop {
                let events = supervisor.events(after, limit)?;
                if !events.is_empty()
                    || tokio::time::Instant::now() >= deadline
                    || supervisor.is_shutting_down()
                {
                    let history_gap = events
                        .first()
                        .is_some_and(|e| e.sequence > after.saturating_add(1));
                    return Ok(ControlReply::Events {
                        events,
                        history_gap,
                    });
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
    }
}
pub async fn request(
    socket: &Path,
    expected_uid: u32,
    request: ControlAction,
    timeout: Duration,
) -> Result<ControlReply> {
    let mutation = matches!(request, ControlAction::Mutate { .. });
    let result = tokio::time::timeout(timeout, async {
        let mut stream = UnixStream::connect(socket)
            .await
            .map_err(|_| Error::Unavailable)?;
        if stream.peer_cred().map_err(|_| Error::Ownership)?.uid() != expected_uid {
            return Err(Error::Ownership);
        }
        write_frame(
            &mut stream,
            &ControlRequest {
                version: 1,
                request,
            },
        )
        .await?;
        let response: ControlResponse = read_frame(&mut stream).await?;
        if response.version != 1 {
            return Err(Error::Unsupported("control protocol version".into()));
        }
        Ok(response)
    })
    .await;
    match result {
        Ok(Ok(response)) => response.result,
        Ok(Err(Error::Ownership)) => Err(Error::Ownership),
        Ok(Err(_)) | Err(_) if mutation => Err(Error::Unknown),
        Ok(Err(error)) => Err(error),
        Err(_) => Err(Error::Unavailable),
    }
}

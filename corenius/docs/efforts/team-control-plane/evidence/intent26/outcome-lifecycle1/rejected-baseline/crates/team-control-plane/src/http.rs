use crate::*;
use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use std::sync::Arc;
use uuid::Uuid;
pub fn router(broker: Broker) -> Router {
    Router::new()
        .route("/mcp", post(crate::mcp::http))
        .route("/v2/contracts/commands", post(contract_command))
        .route("/v2/contracts/tasks/{task}", get(contract_snapshot))
        .route("/v1/artifacts/commands", post(artifact_command))
        .route("/v1/artifacts/objects", post(artifact_publish))
        .route("/v1/artifacts/tasks/{task}", get(artifact_snapshot))
        .route(
            "/v1/artifacts/tasks/{task}/objects/{digest}",
            get(artifact_read),
        )
        .route("/v1/commands", post(command))
        .route("/v1/kernel/status", get(kernel_status))
        .route("/v1/kernel/pause", post(kernel_pause))
        .route("/v1/mailbox/commands", post(mailbox_command))
        .route(
            "/v1/mailbox/messages/{message}/artifacts/{digest}",
            get(mailbox_artifact),
        )
        .route("/v1/mailbox/records/{kind}/{id}", get(mailbox_record))
        .route("/v1/mailbox/messages/{id}", get(mailbox_message))
        .route("/v1/mailbox/missions/{id}", get(mailbox_context))
        .route("/v1/snapshots/{kind}/{id}", get(snapshot))
        .route(
            "/v1/commands/{principal}/{family}/{key}",
            get(command_snapshot),
        )
        .with_state(Arc::new(broker))
}
async fn contract_command(
    artifacts: Option<Extension<crate::artifacts::ArtifactService>>,
    State(broker): State<Arc<Broker>>,
    headers: HeaderMap,
    body: std::result::Result<
        Json<crate::contracts::Envelope>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Result<Json<serde_json::Value>> {
    let token = bearer(&headers)?;
    let Json(envelope) = body.map_err(|_| Error {
        code: ErrorCode::InvalidArgument,
    })?;
    crate::contracts::ContractService::new((*broker).clone())
        .execute(token, envelope, artifacts.as_ref().map(|s| &s.0))
        .await
        .map(Json)
}
async fn contract_snapshot(
    State(broker): State<Arc<Broker>>,
    headers: HeaderMap,
    Path(task): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    crate::contracts::ContractService::new((*broker).clone())
        .snapshot(bearer(&headers)?, task)
        .await
        .map(Json)
}
/// Optional artifact transport shares the supplied service and caller's bearer;
/// the base router remains usable with a named unconfigured-service failure.
pub fn router_with_artifacts(
    broker: Broker,
    artifacts: crate::artifacts::ArtifactService,
) -> Router {
    router(broker).layer(Extension(artifacts))
}
fn artifact_service(
    service: Option<Extension<crate::artifacts::ArtifactService>>,
) -> Result<crate::artifacts::ArtifactService> {
    service.map(|s| s.0).ok_or(Error {
        code: ErrorCode::TemporarilyUnavailable,
    })
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvidenceUpload {
    pub task: Uuid,
    pub digest: String,
    pub bytes: Vec<u8>,
}
async fn artifact_command(
    service: Option<Extension<crate::artifacts::ArtifactService>>,
    headers: HeaderMap,
    body: std::result::Result<
        Json<crate::artifacts::ArtifactEnvelope>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Result<Json<crate::artifacts::ArtifactResult>> {
    let bearer = bearer(&headers)?;
    let Json(envelope) = body.map_err(|_| Error {
        code: ErrorCode::InvalidArgument,
    })?;
    artifact_service(service)?
        .execute(bearer, envelope, crate::artifacts::ReferenceFault::None)
        .await
        .map(Json)
}
async fn artifact_publish(
    service: Option<Extension<crate::artifacts::ArtifactService>>,
    headers: HeaderMap,
    body: std::result::Result<Json<EvidenceUpload>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<crate::artifacts::Manifest>> {
    let bearer = bearer(&headers)?;
    let Json(upload) = body.map_err(|_| Error {
        code: ErrorCode::InvalidArgument,
    })?;
    artifact_service(service)?
        .publish_evidence(bearer, upload.task, &upload.digest, &upload.bytes)
        .await
        .map(Json)
}
async fn artifact_snapshot(
    service: Option<Extension<crate::artifacts::ArtifactService>>,
    headers: HeaderMap,
    Path(task): Path<Uuid>,
) -> Result<Json<crate::artifacts::ArtifactSnapshot>> {
    artifact_service(service)?
        .snapshot(bearer(&headers)?, task)
        .await
        .map(Json)
}
async fn artifact_read(
    service: Option<Extension<crate::artifacts::ArtifactService>>,
    headers: HeaderMap,
    Path((task, digest)): Path<(Uuid, String)>,
) -> Result<Response> {
    let bytes = artifact_service(service)?
        .read_evidence(bearer(&headers)?, task, &digest)
        .await?;
    Ok(([("content-type", "application/octet-stream")], bytes).into_response())
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self.code {
            ErrorCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::TemporarilyUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ErrorCode::InvalidArgument => StatusCode::BAD_REQUEST,
            _ => StatusCode::CONFLICT,
        };
        (status, Json(self)).into_response()
    }
}
pub(crate) fn bearer(headers: &HeaderMap) -> Result<&str> {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .filter(|v| !v.is_empty())
        .ok_or(Error {
            code: ErrorCode::Unauthorized,
        })
}
async fn command(
    State(broker): State<Arc<Broker>>,
    headers: HeaderMap,
    body: std::result::Result<Json<CommandEnvelope>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<CommandResult>> {
    let token = bearer(&headers)?;
    let Json(envelope) = body.map_err(|_| Error {
        code: ErrorCode::InvalidArgument,
    })?;
    broker.execute(token, envelope).await.map(Json)
}
async fn snapshot(
    State(broker): State<Arc<Broker>>,
    headers: HeaderMap,
    Path((kind, id)): Path<(String, String)>,
) -> Result<Json<Snapshot>> {
    let token = bearer(&headers)?;
    let id = Uuid::parse_str(&id).map_err(|_| Error {
        code: ErrorCode::InvalidArgument,
    })?;
    let query = match kind.as_str() {
        "mission" => SnapshotQuery::Mission(id),
        "task" => SnapshotQuery::Task(id),
        _ => {
            return Err(Error {
                code: ErrorCode::InvalidArgument,
            });
        }
    };
    broker.snapshot(token, query).await.map(Json)
}

async fn command_snapshot(
    State(broker): State<Arc<Broker>>,
    headers: HeaderMap,
    Path((principal, family, key)): Path<(String, String, String)>,
) -> Result<Json<Snapshot>> {
    let token = bearer(&headers)?;
    let principal = Uuid::parse_str(&principal).map_err(|_| Error {
        code: ErrorCode::InvalidArgument,
    })?;
    broker
        .snapshot(token, SnapshotQuery::Command(principal, family, key))
        .await
        .map(Json)
}

async fn mailbox_command(
    State(broker): State<Arc<Broker>>,
    headers: HeaderMap,
    body: std::result::Result<
        Json<crate::mailbox::Envelope>,
        axum::extract::rejection::JsonRejection,
    >,
) -> Result<Json<crate::mailbox::Receipt>> {
    let token = bearer(&headers)?;
    let Json(envelope) = body.map_err(|_| Error {
        code: ErrorCode::InvalidArgument,
    })?;
    crate::mailbox::MailboxService::new((*broker).clone())
        .execute(token, envelope)
        .await
        .map(Json)
}

async fn mailbox_message(
    State(broker): State<Arc<Broker>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let token = bearer(&headers)?;
    crate::mailbox::MailboxService::new((*broker).clone())
        .message(token, id)
        .await
        .map(Json)
}

async fn mailbox_context(
    State(broker): State<Arc<Broker>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Query(page): Query<crate::mailbox::ContextPage>,
) -> Result<Json<serde_json::Value>> {
    let token = bearer(&headers)?;
    crate::mailbox::MailboxService::new((*broker).clone())
        .context(token, id, page)
        .await
        .map(Json)
}

async fn mailbox_record(
    State(broker): State<Arc<Broker>>,
    headers: HeaderMap,
    Path((kind, id)): Path<(crate::mailbox::RecordKind, Uuid)>,
) -> Result<Json<serde_json::Value>> {
    crate::mailbox::MailboxService::new((*broker).clone())
        .record(bearer(&headers)?, kind, id)
        .await
        .map(Json)
}

async fn mailbox_artifact(
    service: Option<Extension<crate::artifacts::ArtifactService>>,
    headers: HeaderMap,
    Path((message, digest)): Path<(Uuid, String)>,
) -> Result<Response> {
    let bytes = artifact_service(service)?
        .read_mailbox_evidence(bearer(&headers)?, message, &digest)
        .await?;
    Ok(([("content-type", "application/octet-stream")], bytes).into_response())
}

async fn kernel_status(
    State(broker): State<Arc<Broker>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>> {
    crate::kernel::Kernel::new((*broker).clone())
        .status(bearer(&headers)?)
        .await
        .map(Json)
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PauseRequest {
    mission: Uuid,
    key: String,
    expected_revision: i64,
    recovery_epoch: Uuid,
    paused: bool,
    rationale: String,
}
async fn kernel_pause(
    State(broker): State<Arc<Broker>>,
    headers: HeaderMap,
    body: std::result::Result<Json<PauseRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<serde_json::Value>> {
    let token = bearer(&headers)?;
    let Json(request) = body.map_err(|_| Error {
        code: ErrorCode::InvalidArgument,
    })?;
    if request.recovery_epoch != broker.epoch {
        return Err(Error {
            code: ErrorCode::Unauthorized,
        });
    }
    crate::kernel::Kernel::new((*broker).clone())
        .pause(
            token,
            request.mission,
            &request.key,
            request.expected_revision,
            request.paused,
            &request.rationale,
        )
        .await
        .map(Json)
}

use crate::*;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use std::sync::Arc;
use uuid::Uuid;
pub fn router(broker: Broker) -> Router {
    Router::new()
        .route("/v1/commands", post(command))
        .route("/v1/snapshots/{kind}/{id}", get(snapshot))
        .route(
            "/v1/commands/{principal}/{family}/{key}",
            get(command_snapshot),
        )
        .with_state(Arc::new(broker))
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
fn bearer(headers: &HeaderMap) -> Result<&str> {
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

//! MCP 2025-11-25 Streamable HTTP facade; all tools call the same authenticated domain services.
use crate::{auth, domain::err, mailbox::MailboxService, storage, *};
use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use base64::Engine;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;
const VERSION: &str = "2025-11-25";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}
fn protocol(id: Option<Value>, code: i32, message: &str) -> Response {
    (
        StatusCode::OK,
        Json(json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})),
    )
        .into_response()
}
fn result(id: Value, value: Value) -> Response {
    Json(json!({"jsonrpc":"2.0","id":id,"result":value})).into_response()
}
fn bad() -> Error {
    err(ErrorCode::InvalidArgument)
}
fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T> {
    serde_json::from_value(value).map_err(|_| bad())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadArgs {
    id: Uuid,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextArgs {
    mission: Uuid,
    limit: Option<i64>,
    cursor: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Call {
    name: String,
    arguments: Value,
}
fn tool(value: Result<Value>) -> Value {
    let (value, is_error) = match value {
        Ok(v) => (v, false),
        Err(e) => (json!(e), true),
    };
    json!({"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":is_error})
}
fn catalog(artifacts: bool) -> Value {
    let envelope = json!({"type":"object","properties":{"schema_version":{"type":"integer","const":1},"key":{"type":"string","minLength":1,"maxLength":256},"recovery_epoch":{"type":"string","format":"uuid"},"command":{"type":"object"}},"required":["schema_version","key","recovery_epoch","command"]});
    let core = json!({"type":"object","properties":{"schema_version":{"type":"integer","const":1},"key":{"type":"string"},"expected_revision":{"type":"integer"},"expected_generation":{"type":["integer","null"]},"recovery_epoch":{"type":"string","format":"uuid"},"command":{"type":"object"}},"required":["schema_version","key","expected_revision","recovery_epoch","command"]});
    let mut result = json!({"tools":[
    {"name":"mailbox_command","description":"Persist scoped proposals/messages/checkpoints/yield/receipts and authorized lead decisions. Sender and role come from authentication; this does not dispatch an unconfigured runtime.","inputSchema":envelope},
    {"name":"team_command","description":"Apply a canonical broker command under the caller's existing tool role and scope.","inputSchema":core},
    {"name":"message_read","description":"Read only addressed or canonical sender-owned message content.","inputSchema":{"type":"object","properties":{"id":{"type":"string","format":"uuid"}},"required":["id"],"additionalProperties":false}},
    {"name":"team_context","description":"Read a bounded paginated canonical mission view under Owner/current Lead authority.","inputSchema":{"type":"object","properties":{"mission":{"type":"string","format":"uuid"},"limit":{"type":"integer","minimum":1,"maximum":32},"cursor":{"type":["string","null"]}},"required":["mission"],"additionalProperties":false}}
    ]});
    result["tools"].as_array_mut().expect("static tool array").push(json!({"name":"record_read","description":"Read scoped immutable mailbox record references.","inputSchema":{"type":"object","properties":{"kind":{"type":"string","enum":["proposal","decision","approval","episode","checkpoint","delivery","input"]},"id":{"type":"string","format":"uuid"}},"required":["kind","id"],"additionalProperties":false}}));
    if artifacts {
        let tools = result["tools"].as_array_mut().expect("static tool array");
        tools.push(json!({"name":"artifact_command","description":"Submit, independently verify or explicitly accept through existing ArtifactService caller authority.","inputSchema":{"type":"object","properties":{"key":{"type":"string"},"task":{"type":"string","format":"uuid"},"expected_revision":{"type":"integer"},"generation":{"type":"integer"},"recovery_epoch":{"type":"string","format":"uuid"},"command":{"type":"object"}},"required":["key","task","expected_revision","generation","recovery_epoch","command"],"additionalProperties":false}}));
        tools.push(json!({"name":"artifact_publish_evidence","description":"Publish immutable internal evidence bytes under exact current task scope; does not accept or externally publish a task.","inputSchema":{"type":"object","properties":{"task":{"type":"string","format":"uuid"},"digest":{"type":"string","pattern":"^[0-9a-f]{64}$"},"bytes":{"type":"array","items":{"type":"integer","minimum":0,"maximum":255},"maxItems":1048576}},"required":["task","digest","bytes"],"additionalProperties":false}}));
        tools.push(json!({"name":"artifact_snapshot","description":"Read current scoped candidate and independent reports.","inputSchema":{"type":"object","properties":{"task":{"type":"string","format":"uuid"}},"required":["task"],"additionalProperties":false}}));
    }
    result
}
async fn call(
    broker: &Broker,
    artifacts: Option<&crate::artifacts::ArtifactService>,
    bearer: &str,
    request: Call,
) -> Result<Value> {
    let service = MailboxService::new(broker.clone());
    match request.name.as_str() {
        "record_read" => {
            let a = decode::<RecordArgs>(request.arguments)?;
            service.record(bearer, a.kind, a.id).await
        }
        "artifact_command" => serde_json::to_value(
            artifacts
                .ok_or_else(|| err(ErrorCode::TemporarilyUnavailable))?
                .execute(
                    bearer,
                    decode(request.arguments)?,
                    crate::artifacts::ReferenceFault::None,
                )
                .await?,
        )
        .map_err(|_| bad()),
        "artifact_publish_evidence" => {
            let a = decode::<crate::http::EvidenceUpload>(request.arguments)?;
            serde_json::to_value(
                artifacts
                    .ok_or_else(|| err(ErrorCode::TemporarilyUnavailable))?
                    .publish_evidence(bearer, a.task, &a.digest, &a.bytes)
                    .await?,
            )
            .map_err(|_| bad())
        }
        "artifact_snapshot" => {
            let a = decode::<TaskArgs>(request.arguments)?;
            serde_json::to_value(
                artifacts
                    .ok_or_else(|| err(ErrorCode::TemporarilyUnavailable))?
                    .snapshot(bearer, a.task)
                    .await?,
            )
            .map_err(|_| bad())
        }
        "mailbox_command" => {
            serde_json::to_value(service.execute(bearer, decode(request.arguments)?).await?)
                .map_err(|_| bad())
        }
        "team_command" => {
            serde_json::to_value(broker.execute(bearer, decode(request.arguments)?).await?)
                .map_err(|_| bad())
        }
        "message_read" => {
            service
                .message(bearer, decode::<ReadArgs>(request.arguments)?.id)
                .await
        }
        "team_context" => {
            let a = decode::<ContextArgs>(request.arguments)?;
            service
                .context(
                    bearer,
                    a.mission,
                    crate::mailbox::ContextPage {
                        limit: a.limit,
                        cursor: a.cursor,
                    },
                )
                .await
        }
        _ => Err(bad()),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordArgs {
    kind: crate::mailbox::RecordKind,
    id: Uuid,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskArgs {
    task: Uuid,
}
pub(crate) async fn http(
    State(broker): State<Arc<Broker>>,
    artifacts: Option<Extension<crate::artifacts::ArtifactService>>,
    headers: HeaderMap,
    body: std::result::Result<Json<Value>, axum::extract::rejection::JsonRejection>,
) -> Result<Response> {
    let bearer = crate::http::bearer(&headers)?;
    let mut c = storage::client(&broker.dsn).await?;
    let tx = c.transaction().await?;
    storage::lock(&tx).await?;
    crate::mailbox::schema(&tx).await?;
    storage::active(&tx, broker.epoch).await?;
    let actor = auth::authenticate(&tx, bearer, broker.epoch).await?;
    storage::validate_actor_generation(&tx, &actor).await?;
    // Validate browser origins against the actual loopback endpoint, never forwarded headers.
    if let Some(origin) = headers.get("origin") {
        let host = headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(bad)?;
        let origin = origin.to_str().map_err(|_| bad())?;
        if origin != format!("http://{host}") && origin != format!("https://{host}") {
            return Ok(StatusCode::FORBIDDEN.into_response());
        }
    }
    let accept = headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !accept.contains("application/json") || !accept.contains("text/event-stream") {
        return Ok(StatusCode::NOT_ACCEPTABLE.into_response());
    }
    let Json(value) = match body {
        Ok(value) => value,
        Err(_) => return Ok(protocol(None, -32700, "Invalid JSON")),
    };
    let request = match decode::<Request>(value) {
        Ok(r) if r.jsonrpc == "2.0" => r,
        _ => return Ok(protocol(None, -32600, "Invalid request")),
    };
    if request
        .id
        .as_ref()
        .is_some_and(|id| !id.is_string() && id.as_i64().is_none())
    {
        return Ok(protocol(None, -32600, "Invalid request id"));
    }
    if request.method == "initialize" {
        let Some(id) = request.id else {
            return Ok(protocol(None, -32600, "Initialize requires request id"));
        };
        if !request.params["protocolVersion"].is_string()
            || !request.params["capabilities"].is_object()
            || !request.params["clientInfo"]["name"].is_string()
            || !request.params["clientInfo"]["version"].is_string()
        {
            return Ok(protocol(Some(id), -32602, "Invalid initialize parameters"));
        }
        let count:i64=tx.query_one("SELECT count(*) FROM mailbox_mcp_sessions WHERE token_hash=$1 AND expires_at>clock_timestamp()",&[&token_hash(bearer)]).await?.get(0);
        if count >= 16 {
            return Err(err(ErrorCode::CapacityExhausted));
        }
        let session = Uuid::new_v4();
        tx.execute("INSERT INTO mailbox_mcp_sessions(id,principal_id,token_hash,recovery_epoch,expires_at) VALUES($1,$2,$3,$4,clock_timestamp()+interval '1 hour')",&[&session,&actor.principal,&token_hash(bearer),&broker.epoch]).await?;
        tx.commit().await?;
        let mut response = result(
            id,
            json!({"protocolVersion":VERSION,"capabilities":{"tools":{},"resources":{}},"serverInfo":{"name":"corenius-team-control-plane","version":"1"}}),
        );
        response.headers_mut().insert(
            "mcp-session-id",
            session.to_string().parse().map_err(|_| bad())?,
        );
        return Ok(response);
    }
    if headers
        .get("mcp-protocol-version")
        .and_then(|v| v.to_str().ok())
        != Some(VERSION)
    {
        return Ok(StatusCode::BAD_REQUEST.into_response());
    }
    let session = headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(bad)?;
    let Some(row)=tx.query_opt("SELECT initialized FROM mailbox_mcp_sessions WHERE id=$1 AND principal_id=$2 AND token_hash=$3 AND recovery_epoch=$4 AND expires_at>clock_timestamp()",&[&session,&actor.principal,&token_hash(bearer),&broker.epoch]).await? else{return Ok(StatusCode::NOT_FOUND.into_response());};
    let initialized: bool = row.get(0);
    if request.method == "notifications/initialized" && request.id.is_none() {
        tx.execute(
            "UPDATE mailbox_mcp_sessions SET initialized=true WHERE id=$1",
            &[&session],
        )
        .await?;
        tx.commit().await?;
        return Ok(StatusCode::ACCEPTED.into_response());
    }
    let Some(id) = request.id else {
        tx.commit().await?;
        return Ok(StatusCode::ACCEPTED.into_response());
    };
    if request.method == "ping" {
        tx.commit().await?;
        return Ok(result(id, json!({})));
    }
    if !initialized {
        return Ok(protocol(Some(id), -32000, "Session is not initialized"));
    }
    if request.method == "resources/templates/list" {
        let mut templates = vec![
            json!({"uriTemplate":"corenius://message/{id}","name":"Addressed immutable message","mimeType":"application/json"}),
            json!({"uriTemplate":"corenius://mission/{id}","name":"Scoped canonical mission context","mimeType":"application/json"}),
            json!({"uriTemplate":"corenius://task/{id}","name":"Scoped canonical approved task input","mimeType":"application/json"}),
        ];
        if artifacts.is_some() {
            templates.push(json!({"uriTemplate":"corenius://artifact/{task}/{digest}","name":"Scoped artifact evidence","mimeType":"application/octet-stream"}));
            templates.push(json!({"uriTemplate":"corenius://message-artifact/{message}/{digest}","name":"Exactly disclosed addressed artifact","mimeType":"application/octet-stream"}));
        }
        tx.commit().await?;
        return Ok(result(id, json!({"resourceTemplates":templates})));
    }
    if request.method == "resources/list" {
        let cursor = request
            .params
            .get("cursor")
            .map(|v| {
                v.as_str()
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .ok_or_else(bad)
            })
            .transpose()?;
        let after = cursor.unwrap_or(Uuid::nil());
        let is_lead = actor.role == Role::Lead;
        let rows=tx.query("SELECT m.id FROM mailbox_messages m WHERE m.id>$1 AND (m.sender=$2 OR ($3 AND m.kind='question' AND m.sender_lead_generation IS NOT NULL AND EXISTS(SELECT 1 FROM mission_leads l WHERE l.mission_id=m.mission_id AND l.principal_id=$2)) OR EXISTS(SELECT 1 FROM mailbox_recipients r WHERE r.message_id=m.id AND (r.principal_id=$2 OR (r.lead_seat AND $3 AND EXISTS(SELECT 1 FROM mission_leads l WHERE l.mission_id=m.mission_id AND l.principal_id=$2))))) ORDER BY m.id LIMIT 17",&[&after,&actor.principal,&is_lead]).await?;
        let more = rows.len() > 16;
        let selected = &rows[..rows.len().min(16)];
        let mut resources=selected.iter().map(|r|{let id:Uuid=r.get(0);json!({"uri":format!("corenius://message/{id}"),"name":format!("Message {id}"),"mimeType":"application/json"})}).collect::<Vec<_>>();
        if cursor.is_none()
            && actor.role == Role::Lead
            && let Some(mission) = actor.mission
        {
            resources.push(json!({"uri":format!("corenius://mission/{mission}"),"name":"Canonical mission context","mimeType":"application/json"}));
        }
        if cursor.is_none()
            && let Some(task) = actor.task
        {
            resources.push(json!({"uri":format!("corenius://task/{task}"),"name":"Assigned canonical task input","mimeType":"application/json"}));
        }
        let mut value = json!({"resources":resources});
        if more {
            value["nextCursor"] = json!(selected.last().unwrap().get::<_, Uuid>(0).to_string());
        }
        tx.commit().await?;
        return Ok(result(id, value));
    }
    tx.commit().await?;
    match request.method.as_str() {
        "tools/list" => Ok(result(id, catalog(artifacts.is_some()))),
        "tools/call" => match decode(request.params) {
            Ok(params) => Ok(result(
                id,
                tool(call(&broker, artifacts.as_ref().map(|a| &a.0), bearer, params).await),
            )),
            Err(_) => Ok(protocol(Some(id), -32602, "Invalid tool parameters")),
        },
        "resources/read" => {
            let Some(uri) = request.params["uri"].as_str() else {
                return Ok(protocol(Some(id), -32602, "Invalid resource URI"));
            };
            let service = MailboxService::new((*broker).clone());
            if let Some(value) = uri.strip_prefix("corenius://message-artifact/") {
                let Some((message, digest)) = value.split_once('/') else {
                    return Ok(protocol(
                        Some(id),
                        -32602,
                        "Invalid message artifact resource URI",
                    ));
                };
                let Ok(message) = Uuid::parse_str(message) else {
                    return Ok(protocol(Some(id), -32602, "Invalid message identity"));
                };
                let Some(artifacts) = artifacts else {
                    return Ok(protocol(
                        Some(id),
                        -32002,
                        "Artifact service is unconfigured",
                    ));
                };
                return match artifacts
                    .read_mailbox_evidence(bearer, message, digest)
                    .await
                {
                    Ok(bytes) => Ok(result(
                        id,
                        json!({"contents":[{"uri":uri,"mimeType":"application/octet-stream","blob":base64::engine::general_purpose::STANDARD.encode(bytes)}]}),
                    )),
                    Err(error) => Ok(protocol(
                        Some(id),
                        -32002,
                        &format!("Resource denied: {:?}", error.code),
                    )),
                };
            }
            if let Some(value) = uri.strip_prefix("corenius://artifact/") {
                let Some((task, digest)) = value.split_once('/') else {
                    return Ok(protocol(Some(id), -32602, "Invalid artifact resource URI"));
                };
                let Ok(task) = Uuid::parse_str(task) else {
                    return Ok(protocol(Some(id), -32602, "Invalid artifact task"));
                };
                let Some(artifacts) = artifacts else {
                    return Ok(protocol(
                        Some(id),
                        -32002,
                        "Artifact service is unconfigured",
                    ));
                };
                return match artifacts.read_evidence(bearer, task, digest).await {
                    Ok(bytes) => Ok(result(
                        id,
                        json!({"contents":[{"uri":uri,"mimeType":"application/octet-stream","blob":base64::engine::general_purpose::STANDARD.encode(bytes)}]}),
                    )),
                    Err(error) => Ok(protocol(
                        Some(id),
                        -32002,
                        &format!("Resource denied: {:?}", error.code),
                    )),
                };
            }
            let content = if let Some(value) = uri.strip_prefix("corenius://task/") {
                let Ok(task) = Uuid::parse_str(value) else {
                    return Ok(protocol(Some(id), -32602, "Invalid task identity"));
                };
                broker
                    .snapshot(bearer, SnapshotQuery::Task(task))
                    .await
                    .and_then(|value| serde_json::to_value(value).map_err(|_| bad()))
            } else if let Some(value) = uri.strip_prefix("corenius://message/") {
                service
                    .message(bearer, Uuid::parse_str(value).map_err(|_| bad())?)
                    .await
            } else if let Some(value) = uri.strip_prefix("corenius://mission/") {
                service
                    .context(
                        bearer,
                        Uuid::parse_str(value).map_err(|_| bad())?,
                        Default::default(),
                    )
                    .await
            } else {
                return Ok(protocol(Some(id), -32602, "Unsupported resource URI"));
            };
            match content {
                Ok(value) => Ok(result(
                    id,
                    json!({"contents":[{"uri":uri,"mimeType":"application/json","text":value.to_string()}]}),
                )),
                Err(error) => Ok(protocol(
                    Some(id),
                    -32002,
                    &format!("Resource denied: {:?}", error.code),
                )),
            }
        }
        _ => Ok(protocol(Some(id), -32601, "Method not found")),
    }
}

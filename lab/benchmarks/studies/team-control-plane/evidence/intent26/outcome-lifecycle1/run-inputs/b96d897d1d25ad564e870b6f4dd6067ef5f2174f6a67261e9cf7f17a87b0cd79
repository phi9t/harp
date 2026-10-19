//! Usage authority for the registered legacy model-free transport. Generic
//! gateway replies and local namespace termination do not attest provider costs.
use crate::{
    ErrorCode, Result,
    domain::err,
    episode_runtime::UsageObservation,
    gateway::{Evidence, Operation, OperationKind},
};
use serde_json::{Value, json};
use tokio_postgres::Transaction;
use uuid::Uuid;

pub(crate) async fn observe_legacy(
    tx: &Transaction<'_>,
    op: &Operation,
    epoch: Uuid,
) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    let rejected = || err(ErrorCode::DependencyUnmet);
    let row = tx.query_one("SELECT o.usage,o.controller,o.recovery_epoch,b.store,b.thread_id,s.controller,s.runtime_identity,s.quarantined FROM gateway_operations o JOIN gateway_bindings b ON b.id=o.binding_id JOIN gateway_stores s ON s.identity=b.store WHERE o.id=$1", &[&op.id]).await?;
    let Some(controller) = row.get::<_, Option<Uuid>>(1) else {
        return Ok(());
    };
    let Some(transport) = tx.query_opt("SELECT store,runtime_identity,process_identity,group_id,native_root,native_directory,recovery_epoch,contract FROM kernel_transports WHERE controller=$1", &[&controller]).await? else { return Ok(()); };
    if transport.get::<_, String>(7) != "legacy-supervisor-os-v1" {
        return Ok(());
    }
    if row.get::<_, Uuid>(2) != epoch
        || transport.get::<_, Uuid>(6) != epoch
        || row.get::<_, Uuid>(5) != controller
        || row.get::<_, bool>(7)
        || row.get::<_, String>(3) != transport.get::<_, String>(0)
        || row.get::<_, Value>(6) != transport.get::<_, Value>(1)
    {
        return Err(rejected());
    }
    let process = serde_json::from_value(transport.get(2)).map_err(|_| rejected())?;
    if !crate::model_free_runtime::exclusive_live_transport(&process, transport.get(3))
        .map_err(|_| rejected())?
    {
        return Err(rejected());
    }
    let directory = crate::model_free_runtime::open_directory(std::path::Path::new(
        &transport.get::<_, String>(4),
    ))?;
    let expected: crate::model_free_runtime::DirectoryIdentity =
        serde_json::from_value(transport.get(5)).map_err(|_| rejected())?;
    let metadata = directory.metadata().map_err(|_| rejected())?;
    if metadata.dev() != expected.device
        || metadata.ino() != expected.inode
        || metadata.uid() != expected.owner
    {
        return Err(rejected());
    }
    let mut receipts = Vec::new();
    for name in crate::model_free_runtime::entries(&directory)? {
        let name_text = name.to_str().ok_or_else(rejected)?;
        let bytes = crate::model_free_runtime::native_file(&directory, &name)?;
        if name_text == "controller-ready" {
            if bytes != controller.to_string().as_bytes() {
                return Err(rejected());
            }
            continue;
        }
        if name_text == "runtime.lock" && bytes.is_empty() {
            continue;
        }
        if name_text == "turn.json" {
            serde_json::from_slice::<Value>(&bytes).map_err(|_| rejected())?;
            continue;
        }
        let path = std::path::Path::new(&name);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(rejected)?;
        Uuid::parse_str(stem).map_err(|_| rejected())?;
        match path.extension().and_then(|s| s.to_str()) {
            Some("invocation") => {
                serde_json::from_slice::<Value>(&bytes).map_err(|_| rejected())?;
            }
            Some("effect") => {
                let receipt: Value = serde_json::from_slice(&bytes).map_err(|_| rejected())?;
                if receipt["operation"] == op.id.to_string() {
                    receipts.push((receipt, bytes));
                }
            }
            _ => return Err(rejected()),
        }
    }
    if receipts.len() != 1 {
        return Ok(());
    }
    let (receipt, bytes) = receipts.pop().unwrap();
    let effect = op.evidence.iter().rev().find_map(|e| match e {
        Evidence::ReplyConfirmed {
            effect_id,
            thread_id,
        }
        | Evidence::AuthoritativeEffect {
            effect_id,
            thread_id,
        } => Some((effect_id, thread_id)),
        _ => None,
    });
    let Some((effect, thread)) = effect else {
        return Ok(());
    };
    if receipt["intent"] != json!(op.intent)
        || receipt["runtime"] != transport.get::<_, Value>(1)
        || receipt["id"] != *effect
        || receipt["thread"] != json!(thread)
        || thread.as_deref() != row.get::<_, Option<String>>(4).as_deref()
        || receipt["usage"]
            != json!({"profile":"legacy-supervisor-os-v1","model_requests":0,"cost_microusd":0})
    {
        return Ok(());
    }
    if op.intent.kind == OperationKind::Start {
        let turn = op.evidence.iter().rev().find_map(|e| match e {
            Evidence::ConfirmedStartTurn { runtime_turn } => Some(runtime_turn),
            _ => None,
        });
        if turn.is_none() || receipt["turn"] != json!(turn) {
            return Ok(());
        }
        let Some(execution) = op.intent.arguments["execution"]
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())
        else {
            return Ok(());
        };
        let bound: bool = tx.query_one("SELECT EXISTS(SELECT 1 FROM episode_executions x JOIN gateway_operations o ON o.attempt_id=x.attempt_id JOIN mailbox_episodes e ON e.id=x.episode_id WHERE o.id=$1 AND x.execution_id=$2 AND e.input_packet::text=$3 AND e.input_digest=$4 AND x.recovery_epoch=$5)", &[&op.id,&execution,&op.intent.arguments["input_packet"].as_str(),&op.intent.arguments["input_digest"].as_str(),&epoch]).await?.get(0);
        if !bound {
            return Ok(());
        }
    }
    let usage = UsageObservation::MeasuredZero {
        receipt_digest: crate::artifacts::sha256(&bytes),
    };
    let prior: UsageObservation =
        serde_json::from_value(row.get(0)).map_err(|_| err(ErrorCode::IncompatibleSchema))?;
    if prior != UsageObservation::Unknown {
        if prior != usage {
            return Err(rejected());
        }
        return Ok(());
    }
    tx.execute(
        "UPDATE gateway_operations SET usage=$2 WHERE id=$1",
        &[&op.id, &json!(usage)],
    )
    .await?;
    tx.execute("INSERT INTO kernel_gateway_usage_evidence(id,operation_id,evidence) VALUES($1,$2,$3)", &[&Uuid::new_v4(),&op.id,&json!({"kind":"legacy_native_usage_observed","usage":usage,"intent":op.intent,"controller":controller,"runtime":transport.get::<_,Value>(1),"native_directory":expected,"recovery_epoch":epoch})]).await?;
    Ok(())
}

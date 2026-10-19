use harp_contracts::{ResultEnvelope, TaskId};

use crate::EngineError;

const MAX_RAW_RESULT_BYTES: usize = 256 * 1024;

pub fn decode_result_envelope(
    output_schema: &serde_json::Value,
    raw: &[u8],
    expected_task_id: &TaskId,
    expected_token_usage: Option<u64>,
) -> Result<ResultEnvelope, EngineError> {
    if raw.len() > MAX_RAW_RESULT_BYTES {
        return Err(EngineError::OutputSchema {
            task_id: expected_task_id.clone(),
            context: format!(
                "raw result uses {} bytes, exceeding {}",
                raw.len(),
                MAX_RAW_RESULT_BYTES
            ),
        });
    }
    let instance: serde_json::Value =
        serde_json::from_slice(raw).map_err(|source| EngineError::OutputJson {
            task_id: expected_task_id.clone(),
            source,
        })?;
    let validator = jsonschema::validator_for(output_schema).map_err(|error| {
        EngineError::ExecutionReceipt {
            context: format!(
                "task {} pinned output schema could not compile: {error}",
                expected_task_id
            ),
        }
    })?;
    if let Err(error) = validator.validate(&instance) {
        return Err(EngineError::OutputSchema {
            task_id: expected_task_id.clone(),
            context: error.to_string(),
        });
    }
    let result: ResultEnvelope =
        serde_json::from_value(instance).map_err(|source| EngineError::OutputJson {
            task_id: expected_task_id.clone(),
            source,
        })?;
    result.validate().map_err(EngineError::Contract)?;
    if result.task_id != *expected_task_id {
        return Err(EngineError::InvalidResult {
            task_id: expected_task_id.clone(),
            context: "result task identity differs from pinned task".to_owned(),
        });
    }
    if expected_token_usage.is_some_and(|expected| result.token_usage != expected) {
        return Err(EngineError::InvalidResult {
            task_id: expected_task_id.clone(),
            context: "result token usage differs from durable attempt usage".to_owned(),
        });
    }
    Ok(result)
}

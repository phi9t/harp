use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{validate_bounded_string, AttemptId, ContractResult, RunId, TaskId};

const MAX_PHASE_BYTES: usize = 256;
const MAX_UNIT_BYTES: usize = 1024;
const MAX_UNITS_PER_LIST: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Checkpoint {
    #[schemars(extend("const" = 1))]
    pub schema_version: u8,
    pub run_id: RunId,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    #[schemars(length(min = 1, max = 256))]
    pub phase: String,
    #[schemars(length(max = 256), inner(length(min = 1, max = 1024)))]
    pub completed_units: Vec<String>,
    #[schemars(length(max = 256), inner(length(min = 1, max = 1024)))]
    pub pending_units: Vec<String>,
    pub evidence_count: u64,
    pub updated_at_unix_seconds: i64,
}

impl Checkpoint {
    pub fn new(
        run_id: RunId,
        task_id: TaskId,
        attempt_id: AttemptId,
        phase: impl Into<String>,
        updated_at_unix_seconds: i64,
    ) -> ContractResult<Self> {
        let checkpoint = Self {
            schema_version: 1,
            run_id,
            task_id,
            attempt_id,
            phase: phase.into(),
            completed_units: Vec::new(),
            pending_units: Vec::new(),
            evidence_count: 0,
            updated_at_unix_seconds,
        };
        validate_bounded_string("phase", &checkpoint.phase, MAX_PHASE_BYTES, false)?;
        Ok(checkpoint)
    }

    pub fn validate(&self) -> ContractResult<()> {
        if self.schema_version != 1 {
            return Err(crate::invalid("schemaVersion", "must equal 1"));
        }
        validate_bounded_string("phase", &self.phase, MAX_PHASE_BYTES, false)?;
        validate_units("completedUnits", &self.completed_units)?;
        validate_units("pendingUnits", &self.pending_units)?;

        let mut seen =
            HashSet::with_capacity(self.completed_units.len() + self.pending_units.len());
        for unit in self.completed_units.iter().chain(self.pending_units.iter()) {
            if !seen.insert(unit.as_str()) {
                return Err(crate::invalid(
                    "units",
                    "must not contain duplicates within or across completed and pending units",
                ));
            }
        }
        Ok(())
    }
}

fn validate_units(field: &'static str, units: &[String]) -> ContractResult<()> {
    if units.len() > MAX_UNITS_PER_LIST {
        return Err(crate::invalid(field, "must contain at most 256 entries"));
    }
    for unit in units {
        validate_bounded_string(field, unit, MAX_UNIT_BYTES, false)?;
    }
    Ok(())
}

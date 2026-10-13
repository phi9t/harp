use serde_json::Value;
use team_control_plane::gateway::*;
#[derive(Clone, Copy, Debug)]
pub enum Fault {
    None,
    DropReply,
    CrashAfterEffect,
    CrashBeforeEffect,
    EmptyLossyHistory,
    KnownFailure,
    MissingCapabilities,
}
pub struct FaultRuntime {
    directory: std::path::PathBuf,
    pub fault: Fault,
    pub identity: RuntimeIdentity,
}
impl FaultRuntime {
    pub fn new(directory: std::path::PathBuf, fault: Fault, identity: RuntimeIdentity) -> Self {
        Self {
            directory,
            fault,
            identity,
        }
    }
    pub fn effect_count(&self) -> std::io::Result<usize> {
        Ok(std::fs::read_dir(&self.directory)?
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .count())
    }
    pub fn invocation_count(&self) -> std::io::Result<usize> {
        Ok(std::fs::read_dir(&self.directory)?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext == "invocation")
            })
            .count())
    }
    fn invoke(&self, mode: &str, intent: &Intent, thread: Option<&str>) -> Value {
        let kind = serde_json::to_value(intent.kind).expect("fixture operation kind serializes");
        let output = std::process::Command::new("python3")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/fault_runtime.py"
            ))
            .arg(mode)
            .arg(&self.directory)
            .arg(intent.id.to_string())
            .arg(kind.as_str().expect("fixture kind is a string"))
            .arg(thread.unwrap_or(""))
            .output()
            .expect("unexpected fault-runtime subprocess launch failure");
        assert!(
            output.status.success(),
            "unexpected fault-runtime subprocess exit: {}",
            output.status
        );
        serde_json::from_slice(&output.stdout).expect("unexpected malformed fault-runtime reply")
    }
}
impl RuntimeAdapter for FaultRuntime {
    fn identity(&self) -> RuntimeIdentity {
        self.identity.clone()
    }
    fn inspect(&self) -> CapabilityProfile {
        if matches!(self.fault, Fault::MissingCapabilities) {
            return CapabilityProfile::default();
        }
        CapabilityProfile {
            create: true,
            recover: true,
            start: true,
            observe: true,
            steer: true,
            cancel: true,
            authoritative_effect_inventory: true,
            native_deduplication: false,
        }
    }
    fn execute(&mut self, intent: &Intent, thread: Option<&str>) -> RuntimeReply {
        if matches!(self.fault, Fault::CrashBeforeEffect) {
            panic!("fixture crash before RPC");
        }
        let result = self.invoke(
            if matches!(self.fault, Fault::KnownFailure) {
                "execute_fail_known"
            } else {
                "execute"
            },
            intent,
            thread,
        );
        if matches!(self.fault, Fault::KnownFailure) {
            return RuntimeReply::FailedKnown(
                result["known_failure"]
                    .as_str()
                    .expect("fixture failure reason")
                    .into(),
            );
        }
        if matches!(self.fault, Fault::CrashAfterEffect) {
            panic!("fixture controller crash after external effect");
        }
        if matches!(self.fault, Fault::DropReply) {
            return RuntimeReply::Unknown("fixture lost acknowledgment".into());
        }
        RuntimeReply::Confirmed {
            thread_id: result["thread_id"].as_str().map(str::to_owned),
            effect_id: result["effect_id"]
                .as_str()
                .expect("fixture effect identity")
                .into(),
        }
    }
    fn observe(&mut self, intent: &Intent, thread: Option<&str>) -> Observation {
        if matches!(self.fault, Fault::EmptyLossyHistory) {
            return Observation::Inconclusive(
                "empty lossy history does not establish absence".into(),
            );
        }
        let result = self.invoke("observe", intent, thread);
        if result["effect_id"].is_string() {
            Observation::Effect {
                thread_id: result["thread_id"].as_str().map(str::to_owned),
                effect_id: result["effect_id"].as_str().unwrap().into(),
            }
        } else if result["inventory_id"].is_string() {
            Observation::AuthoritativeAbsence {
                inventory_id: result["inventory_id"].as_str().unwrap().into(),
            }
        } else if result["multiple_effect_ids"].is_array() {
            Observation::Inconclusive(
                "fixture observed multiple external effects for the same operation".into(),
            )
        } else {
            panic!("unexpected fault-runtime inventory shape");
        }
    }
}

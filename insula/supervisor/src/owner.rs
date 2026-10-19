use crate::{model::*, Result};
use async_trait::async_trait;
use uuid::Uuid;

/// Implement inside the existing Insula owner, not a second launch/reaping path.
///
/// - Claims are exclusive across ALL clients/state directories for this root.
/// - Epoch and generation are rechecked atomically at each actual side effect.
/// - Launch reservations are durable and idempotent by (key,generation,operation).
/// - Losing a claim fences unfinished work before admitting another controller.
/// - Gone certifies both required boundary emptiness and no pending launch.
/// - Dropping/canceling a request may have an unknown outcome. Reconcile it.
/// - Stop implements graceful deadline, force permission and authoritative reaping.
/// - Probes are bounded, clean up their children, redact output, enter the target.
#[async_trait]
pub trait ExecutionOwner: Send + Sync + 'static {
    async fn capabilities(&self, root_insula_id: &str) -> Result<OwnerCapabilities>;
    async fn claim(&self, root_insula_id: &str, controller_id: Uuid) -> Result<Ownership>;
    async fn validate(&self, ownership: &Ownership) -> Result<()>;
    async fn observe(
        &self,
        ownership: &Ownership,
        key: &ServiceKey,
        generation: Option<Generation>,
    ) -> Result<Observation>;
    async fn launch(&self, ownership: &Ownership, request: LaunchRequest) -> Result<Observation>;
    async fn stop(&self, ownership: &Ownership, request: StopRequest) -> Result<Observation>;
    async fn probe(&self, ownership: &Ownership, request: ProbeRequest) -> Result<ProbeResult>;
    async fn release(&self, ownership: &Ownership) -> Result<()>;
}

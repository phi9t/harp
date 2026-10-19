use crate::{
    model::*,
    owner::ExecutionOwner,
    store::{AuthorityLock, Phase, Store, StoredService, Transition},
    Error, Result,
};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tokio::sync::{Mutex as AsyncMutex, RwLock, Semaphore};
use uuid::Uuid;

const OWNER_TIMEOUT: Duration = Duration::from_secs(5);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailPoint {
    AfterIntent,
    AfterOwnerLaunch,
    AfterOwnerStop,
}

pub struct Supervisor {
    owner: Arc<dyn ExecutionOwner>,
    ownership: Ownership,
    capabilities: OwnerCapabilities,
    authority: AuthorityLock,
    store: Store,
    state_dir: PathBuf,
    locks: Mutex<BTreeMap<ServiceKey, Arc<AsyncMutex<()>>>>,
    endpoints: AsyncMutex<()>,
    admission: Arc<RwLock<()>>,
    closing: AtomicBool,
    ownership_lost: AtomicBool,
    probes: Arc<Semaphore>,
    failpoint: Mutex<Option<FailPoint>>,
}

impl Supervisor {
    pub async fn open(
        owner: Arc<dyn ExecutionOwner>,
        root_insula_id: String,
        state_dir: PathBuf,
    ) -> Result<Arc<Self>> {
        if !valid_id(&root_insula_id) {
            return Err(Error::Invalid("invalid root Insula ID".into()));
        }
        let capabilities = bounded(owner.capabilities(&root_insula_id)).await?;
        if capabilities.root_insula_id != root_insula_id
            || !capabilities.controller_fencing
            || !capabilities.generation_fencing
            || !capabilities.durable_launch_reservations
            || !capabilities.bounded_probes
        {
            return Err(Error::Unsupported("owner must implement exclusive controller and generation fencing, durable launch reservations, and bounded probes".into()));
        }
        // This path comes from the verified owner, never a client state-dir option.
        let authority = AuthorityLock::acquire(
            &capabilities.canonical_authority_dir,
            "controller.lock",
            false,
        )?;
        let ownership = bounded(owner.claim(&root_insula_id, Uuid::new_v4())).await?;
        if ownership.root_insula_id != root_insula_id
            || ownership.owner_instance != capabilities.owner_instance
        {
            let _ = bounded(owner.release(&ownership)).await;
            return Err(Error::Ownership);
        }
        let store = match Store::open(&state_dir, &root_insula_id, capabilities.owner_instance) {
            Ok(s) => s,
            Err(e) => {
                let _ = bounded(owner.release(&ownership)).await;
                return Err(e);
            }
        };
        let this = Arc::new(Self {
            owner,
            ownership,
            capabilities,
            authority,
            store,
            state_dir,
            locks: Mutex::new(BTreeMap::new()),
            endpoints: AsyncMutex::new(()),
            admission: Arc::new(RwLock::new(())),
            closing: AtomicBool::new(false),
            ownership_lost: AtomicBool::new(false),
            probes: Arc::new(Semaphore::new(8)),
            failpoint: Mutex::new(None),
        });
        if let Err(e) = this.reconcile_all().await {
            let _ = bounded(this.owner.release(&this.ownership)).await;
            return Err(e);
        }
        Ok(this)
    }
    pub fn ownership(&self) -> &Ownership {
        &self.ownership
    }
    pub fn capabilities(&self) -> &OwnerCapabilities {
        &self.capabilities
    }
    pub fn state_dir(&self) -> &std::path::Path {
        &self.state_dir
    }
    pub fn is_shutting_down(&self) -> bool {
        self.closing.load(Ordering::Acquire)
    }
    pub fn has_lost_ownership(&self) -> bool {
        self.ownership_lost.load(Ordering::Acquire)
    }
    /// Fault-injection seam for crash-window tests. Never exposed through the CLI.
    #[doc(hidden)]
    pub fn set_failpoint(&self, point: Option<FailPoint>) {
        if let Ok(mut p) = self.failpoint.lock() {
            *p = point
        }
    }
    fn trip(&self, point: FailPoint) -> Result<()> {
        let mut p = self.failpoint.lock().map_err(|_| Error::Storage)?;
        if *p == Some(point) {
            *p = None;
            return Err(Error::Unknown);
        }
        Ok(())
    }
    fn service_lock(&self, key: &ServiceKey) -> Result<Arc<AsyncMutex<()>>> {
        let mut locks = self.locks.lock().map_err(|_| Error::Storage)?;
        Ok(locks
            .entry(key.clone())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone())
    }
    async fn validate_control(&self) -> Result<()> {
        if self.ownership_lost.load(Ordering::Acquire) {
            return Err(Error::Ownership);
        }
        if self.authority.validate().is_err()
            || bounded(self.owner.validate(&self.ownership)).await.is_err()
        {
            self.ownership_lost.store(true, Ordering::Release);
            return Err(Error::Ownership);
        }
        Ok(())
    }
    pub fn inspect(&self, key: &ServiceKey) -> Result<ServiceStatus> {
        self.store
            .get(key)?
            .map(|r| r.status)
            .ok_or(Error::NotFound)
    }
    pub fn list(&self) -> Result<Vec<ServiceStatus>> {
        Ok(self.store.all()?.into_iter().map(|r| r.status).collect())
    }
    pub fn operation(&self, id: OperationId) -> Result<OperationRecord> {
        self.store.operation(id)?.ok_or(Error::NotFound)
    }
    pub fn events(&self, after: u64, limit: usize) -> Result<Vec<Event>> {
        self.store.events(after, limit)
    }

    /// Once admitted, the operation is owned by this component. Dropping a client
    /// wait does not cancel a launch/stop or release its per-service serialization.
    pub async fn execute(self: &Arc<Self>, request: OperationRequest) -> Result<OperationRecord> {
        let permit = self.admission.clone().read_owned().await;
        if self.is_shutting_down() {
            return Err(Error::ShuttingDown);
        }
        let this = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            this.execute_inner(request, OperationOrigin::User).await
        })
        .await
        .map_err(|_| Error::Unknown)?
    }
    async fn execute_inner(
        &self,
        request: OperationRequest,
        origin: OperationOrigin,
    ) -> Result<OperationRecord> {
        let key = request.mutation.key().clone();
        if !valid_id(&key.insula_id) || !valid_id(&key.service_id) {
            return Err(Error::Invalid("invalid service key".into()));
        }
        let lock = self.service_lock(&key)?;
        let _guard = lock.lock().await;
        let fingerprint = digest(&request)?;
        if let Some(op) = self.store.operation(request.operation_id)? {
            if op.request_digest != fingerprint || op.key != key {
                return Err(Error::Conflict(
                    "operation ID reused with a different request".into(),
                ));
            }
            return Ok(op);
        }
        self.validate_control().await?;
        match request.mutation.clone() {
            Mutation::Start { spec } => self.start_locked(*spec, request, origin).await,
            Mutation::Stop {
                expected_generation,
                ..
            } => {
                let row = self.store.get(&key)?.ok_or(Error::NotFound)?;
                self.check_target(&row, expected_generation)?;
                self.stop_or_restart_locked(row, request, origin, false)
                    .await
            }
            Mutation::Restart {
                expected_generation,
                ..
            } => {
                let row = self.store.get(&key)?.ok_or(Error::NotFound)?;
                self.check_target(&row, expected_generation)?;
                self.stop_or_restart_locked(row, request, origin, true)
                    .await
            }
        }
    }
    fn check_target(&self, row: &StoredService, generation: Generation) -> Result<()> {
        if row.status.generation != Some(generation) {
            return Err(Error::StaleGeneration);
        }
        if row.status.lifecycle == Lifecycle::Unknown || row.pending.is_some() {
            return Err(Error::Unknown);
        }
        Ok(())
    }
    fn check_spec(&self, spec: &ServiceSpec) -> Result<()> {
        spec.validate()?;
        if spec.cleanup_boundary == CleanupBoundary::Service
            && !self.capabilities.service_boundary_cleanup
        {
            return Err(Error::Unsupported(
                "owner cannot verify the complete service cleanup boundary".into(),
            ));
        }
        if spec.logs.max_bytes > 0 && !self.capabilities.bounded_logs {
            return Err(Error::Unsupported(
                "owner cannot enforce bounded redacted logs".into(),
            ));
        }
        if spec.controller_exit == ControllerExit::KeepServices
            && !self.capabilities.survives_controller_exit
        {
            return Err(Error::Unsupported("owner has not verified service survival after controller exit; declare stop_services or verify runtime lifetime".into()));
        }
        Ok(())
    }
    fn new_op(
        &self,
        request: &OperationRequest,
        origin: OperationOrigin,
    ) -> Result<OperationRecord> {
        let t = now_ms();
        Ok(OperationRecord {
            operation_id: request.operation_id,
            request_digest: digest(request)?,
            key: request.mutation.key().clone(),
            outcome: OperationOutcome::Pending,
            origin,
            created_ms: t,
            updated_ms: t,
        })
    }
    fn check_endpoint(&self, spec: &ServiceSpec) -> Result<()> {
        if let ServiceAdapter::Tmux { socket } = &spec.adapter {
            for existing in self.store.all()? {
                if existing.spec.key != spec.key
                    && existing.spec.key.insula_id == spec.key.insula_id
                    && !matches!(
                        existing.status.lifecycle,
                        Lifecycle::Absent | Lifecycle::Exited
                    )
                {
                    if let ServiceAdapter::Tmux { socket: other } = &existing.spec.adapter {
                        if other == socket {
                            return Err(Error::Conflict(
                                "tmux endpoint is reserved by another service".into(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    async fn start_locked(
        &self,
        spec: ServiceSpec,
        request: OperationRequest,
        origin: OperationOrigin,
    ) -> Result<OperationRecord> {
        let _endpoint_guard = self.endpoints.lock().await;
        self.check_spec(&spec)?;
        self.check_endpoint(&spec)?;
        let mut row = match self.store.get(&spec.key)? {
            Some(r) => r,
            None => {
                if self.store.all()?.len() >= 128 {
                    return Err(Error::Unsupported("maximum 128 declared services".into()));
                }
                StoredService::new(spec.clone())?
            }
        };
        if row.pending.is_some() || row.status.lifecycle == Lifecycle::Unknown {
            return Err(Error::Unknown);
        }
        let config = digest(&spec)?;
        if row.status.config_digest != config && row.status.desired == DesiredState::Running {
            return Err(Error::Conflict(
                "stop the existing configuration before replacing it".into(),
            ));
        }
        let observation = bounded(self.owner.observe(
            &self.ownership,
            &spec.key,
            row.status.generation,
        ))
        .await
        .unwrap_or(Observation::Unknown);
        match self.checked_for_row(&row, observation) {
            Observation::Alive { identity } => {
                if row.status.config_digest != config
                    || row.status.generation != Some(identity.generation)
                {
                    self.mark_unknown(&mut row, None)?;
                    return Err(Error::Unknown);
                }
                row.status.desired = DesiredState::Running;
                self.apply_alive(&mut row, identity);
                let mut op = self.new_op(&request, origin)?;
                op.outcome = OperationOutcome::Succeeded {
                    generation: row.status.generation,
                };
                self.store.commit(&row, Some(&op), "start_converged")?;
                Ok(op)
            }
            Observation::Gone { .. } => {
                if row.status.config_digest != config {
                    row = StoredService::new(spec.clone())?;
                }
                row.spec = spec;
                let generation = Uuid::new_v4();
                self.prepare_generation(&mut row, generation);
                row.pending = Some(Transition {
                    request: request.clone(),
                    phase: Phase::Launch,
                    replacement: None,
                    owner_launch_id: Uuid::new_v4(),
                    owner_stop_id: Uuid::new_v4(),
                    controller_epoch: self.ownership.epoch,
                });
                row.status.pending_operation = Some(request.operation_id);
                let op = self.new_op(&request, origin)?;
                self.store.commit(&row, Some(&op), "launch_intent")?;
                self.trip(FailPoint::AfterIntent)?;
                self.launch_locked(row, op).await
            }
            Observation::Unknown => {
                self.mark_unknown(&mut row, None)?;
                Err(Error::Unknown)
            }
        }
    }
    fn prepare_generation(&self, row: &mut StoredService, generation: Generation) {
        row.status.generation = Some(generation);
        row.status.desired = DesiredState::Running;
        row.status.lifecycle = Lifecycle::Starting;
        row.status.ready = false;
        row.status.health = Health::Unknown;
        row.status.identity = None;
        row.status.started_ms = Some(now_ms());
        row.status.last_probe = None;
        row.status.last_readiness = None;
        row.status.next_recovery = RecoveryAction::None;
        row.health_failures = 0;
        row.readiness_failures = 0;
        row.next_health_ms = 0;
        row.next_readiness_ms = 0;
        row.readiness_sequence = row.readiness_sequence.wrapping_add(1);
        row.health_sequence = row.health_sequence.wrapping_add(1);
    }
    async fn launch_locked(
        &self,
        mut row: StoredService,
        mut op: OperationRecord,
    ) -> Result<OperationRecord> {
        self.validate_control().await?;
        let generation = row.status.generation.ok_or(Error::Unknown)?;
        let owner_operation = row.pending.as_ref().ok_or(Error::Storage)?.owner_launch_id;
        let result = bounded(self.owner.launch(
            &self.ownership,
            LaunchRequest {
                operation_id: owner_operation,
                generation,
                spec: row.spec.clone(),
            },
        ))
        .await;
        self.trip(FailPoint::AfterOwnerLaunch)?;
        // Validate even successful replies after an ownership change; owner effects
        // remain durable, but this controller must not publish a stale outcome.
        if self.validate_control().await.is_err() {
            self.mark_unknown(&mut row, Some(&mut op))?;
            return Ok(op);
        }
        match self.checked_observation(result.unwrap_or(Observation::Unknown), Some(generation)) {
            Observation::Alive { identity } => {
                self.apply_alive(&mut row, identity);
                row.pending = None;
                row.status.pending_operation = None;
                op.outcome = OperationOutcome::Succeeded {
                    generation: Some(generation),
                };
            }
            Observation::Gone { last_exit, .. } => {
                row.status.lifecycle = Lifecycle::Exited;
                row.status.last_exit = last_exit;
                row.status.ready = false;
                row.status.health = Health::Unhealthy;
                row.status.identity = None;
                row.pending = None;
                row.status.pending_operation = None;
                op.outcome = OperationOutcome::Failed {
                    error: Error::Unavailable,
                };
                self.schedule_recovery(&mut row);
            }
            Observation::Unknown => {
                self.mark_unknown(&mut row, Some(&mut op))?;
                return Ok(op);
            }
        }
        op.updated_ms = now_ms();
        self.store.commit(&row, Some(&op), "launch_observed")?;
        Ok(op)
    }
    async fn stop_or_restart_locked(
        &self,
        mut row: StoredService,
        request: OperationRequest,
        origin: OperationOrigin,
        restart: bool,
    ) -> Result<OperationRecord> {
        let _endpoint_guard = if restart {
            Some(self.endpoints.lock().await)
        } else {
            None
        };
        if restart {
            self.check_endpoint(&row.spec)?;
        }
        let generation = row.status.generation.ok_or(Error::StaleGeneration)?;
        // Query before mutating intent. The owner may know about an instance this
        // controller has not observed; do not overwrite intent for a stale target.
        let observed = bounded(self.owner.observe(
            &self.ownership,
            &row.spec.key,
            Some(generation),
        ))
        .await
        .unwrap_or(Observation::Unknown);
        if matches!(self.checked_for_row(&row, observed), Observation::Unknown) {
            self.mark_unknown(&mut row, None)?;
            return Err(Error::Unknown);
        }
        let mut op = self.new_op(&request, origin)?;
        let automatic = origin == OperationOrigin::Automatic;
        row.status.desired = if restart {
            DesiredState::Running
        } else {
            DesiredState::Stopped
        };
        row.status.lifecycle = Lifecycle::Stopping;
        row.status.ready = false;
        row.status.next_recovery = RecoveryAction::None;
        if automatic {
            row.status.automatic_restarts = row.status.automatic_restarts.saturating_add(1)
        }
        row.pending = Some(Transition {
            request,
            phase: if automatic {
                Phase::AutomaticStop
            } else if restart {
                Phase::RestartStop
            } else {
                Phase::Stop
            },
            replacement: restart.then(Uuid::new_v4),
            owner_launch_id: Uuid::new_v4(),
            owner_stop_id: Uuid::new_v4(),
            controller_epoch: self.ownership.epoch,
        });
        row.status.pending_operation = Some(op.operation_id);
        self.store.commit(
            &row,
            Some(&op),
            if automatic {
                "automatic_restart_intent"
            } else if restart {
                "restart_intent"
            } else {
                "stop_intent"
            },
        )?;
        self.trip(FailPoint::AfterIntent)?;
        self.validate_control().await?;
        let owner_operation = row.pending.as_ref().ok_or(Error::Storage)?.owner_stop_id;
        let result = tokio::time::timeout(
            Duration::from_millis(row.spec.stop.deadline_ms.saturating_add(2000)),
            self.owner.stop(
                &self.ownership,
                StopRequest {
                    operation_id: owner_operation,
                    key: row.spec.key.clone(),
                    expected_generation: generation,
                    policy: row.spec.stop.clone(),
                },
            ),
        )
        .await;
        self.trip(FailPoint::AfterOwnerStop)?;
        if self.validate_control().await.is_err() {
            self.mark_unknown(&mut row, Some(&mut op))?;
            return Ok(op);
        }
        let observation = result
            .ok()
            .and_then(|r| r.ok())
            .unwrap_or(Observation::Unknown);
        match self.checked_observation(observation, Some(generation)) {
            Observation::Gone { last_exit, .. } => {
                row.status.last_exit = last_exit.or(row.status.last_exit);
                row.status.identity = None;
                row.status.health = Health::Unknown;
                if restart {
                    let replacement = row
                        .pending
                        .as_ref()
                        .and_then(|t| t.replacement)
                        .ok_or(Error::Storage)?;
                    self.prepare_generation(&mut row, replacement);
                    row.pending.as_mut().ok_or(Error::Storage)?.phase = if automatic {
                        Phase::AutomaticLaunch
                    } else {
                        Phase::RestartLaunch
                    };
                    self.store.commit(&row, Some(&op), "replacement_intent")?;
                    self.launch_locked(row, op).await
                } else {
                    row.status.lifecycle = Lifecycle::Exited;
                    row.pending = None;
                    row.status.pending_operation = None;
                    op.outcome = OperationOutcome::Succeeded {
                        generation: Some(generation),
                    };
                    op.updated_ms = now_ms();
                    self.store.commit(&row, Some(&op), "stopped")?;
                    Ok(op)
                }
            }
            // A graceful deadline that leaves the service alive never grants
            // permission to spawn a second instance or broaden cleanup scope.
            _ => {
                self.mark_unknown(&mut row, Some(&mut op))?;
                Ok(op)
            }
        }
    }
    fn checked_observation(
        &self,
        observation: Observation,
        expected: Option<Generation>,
    ) -> Observation {
        match &observation {
            Observation::Alive { identity }
                if Some(identity.generation) == expected
                    && identity.owner_instance == self.ownership.owner_instance
                    && identity.pid > 0
                    && !identity.boot_id.is_empty()
                    && identity.start_time_ticks > 0 =>
            {
                observation
            }
            Observation::Gone {
                generation,
                boundary_empty: true,
                launch_resolved: true,
                ..
            } if *generation == expected => observation,
            _ => Observation::Unknown,
        }
    }
    fn checked_for_row(&self, row: &StoredService, observation: Observation) -> Observation {
        if let (Some(previous), Observation::Alive { identity }) =
            (&row.status.identity, &observation)
        {
            if previous != identity {
                return Observation::Unknown;
            }
        }
        self.checked_observation(observation, row.status.generation)
    }
    fn apply_alive(&self, row: &mut StoredService, identity: ProcessIdentity) {
        if row.status.identity.as_ref() != Some(&identity)
            || row.status.lifecycle != Lifecycle::Running
        {
            row.status.ready = false;
            row.status.health = Health::Unknown;
        }
        row.status.lifecycle = Lifecycle::Running;
        row.status.identity = Some(identity);
        row.status.started_ms.get_or_insert_with(now_ms);
        if row.status.next_recovery == RecoveryAction::BlockedUnknown {
            row.status.next_recovery = RecoveryAction::None;
        }
    }
    fn mark_unknown(
        &self,
        row: &mut StoredService,
        op: Option<&mut OperationRecord>,
    ) -> Result<()> {
        row.status.lifecycle = Lifecycle::Unknown;
        row.status.ready = false;
        row.status.health = Health::Unknown;
        row.status.next_recovery = RecoveryAction::BlockedUnknown;
        match op {
            Some(op) => {
                op.outcome = OperationOutcome::Unknown;
                op.updated_ms = now_ms();
                self.store.commit(row, Some(op), "outcome_unknown")?;
            }
            None => {
                self.store.commit(row, None, "observation_unknown")?;
            }
        }
        Ok(())
    }

    pub async fn reconcile_all(&self) -> Result<()> {
        self.validate_control().await?;
        for old in self.store.all()? {
            let lock = self.service_lock(&old.spec.key)?;
            let _guard = lock.lock().await;
            let mut row = self.store.get(&old.spec.key)?.ok_or(Error::Storage)?;
            self.reconcile_locked(&mut row).await?;
        }
        Ok(())
    }
    async fn reconcile_locked(&self, row: &mut StoredService) -> Result<()> {
        let observed = bounded(self.owner.observe(
            &self.ownership,
            &row.spec.key,
            row.status.generation,
        ))
        .await
        .unwrap_or(Observation::Unknown);
        self.validate_control().await?;
        let observed = self.checked_for_row(row, observed);
        let before = row.status.clone();
        let mut op = row
            .pending
            .as_ref()
            .map(|t| self.store.operation(t.request.operation_id))
            .transpose()?
            .flatten();
        match observed {
            Observation::Alive { identity } => {
                if row.pending.as_ref().is_some_and(|t| {
                    t.controller_epoch == self.ownership.epoch
                        && matches!(
                            t.phase,
                            Phase::Stop | Phase::RestartStop | Phase::AutomaticStop
                        )
                }) {
                    // The earlier stop may still be in flight. Process liveness
                    // does not establish completion of a destructive operation.
                    // A fresh exclusive epoch fences it; this epoch must wait.
                    self.mark_unknown(row, op.as_mut())?;
                    return Ok(());
                }
                self.apply_alive(row, identity);
                if let Some(transition) = row.pending.take() {
                    if let Some(ref mut op) = op {
                        op.outcome = if matches!(
                            transition.phase,
                            Phase::Launch | Phase::RestartLaunch | Phase::AutomaticLaunch
                        ) {
                            OperationOutcome::Succeeded {
                                generation: row.status.generation,
                            }
                        } else {
                            OperationOutcome::Unknown
                        };
                        op.updated_ms = now_ms();
                    }
                    row.status.pending_operation = None;
                    // A stopped intent with a live service is visible. It is not
                    // silently replayed; an explicit fenced stop can resolve it.
                    if row.status.desired == DesiredState::Stopped {
                        row.status.next_recovery = RecoveryAction::BlockedUnknown;
                        row.status.ready = false;
                    }
                }
            }
            Observation::Gone { last_exit, .. } => {
                row.status.lifecycle = Lifecycle::Exited;
                row.status.identity = None;
                row.status.ready = false;
                row.status.last_exit = last_exit.or_else(|| row.status.last_exit.clone());
                row.status.health = if row.status.desired == DesiredState::Running {
                    Health::Unhealthy
                } else {
                    Health::Unknown
                };
                if let Some(transition) = row.pending.take() {
                    if let Some(ref mut op) = op {
                        op.outcome = if transition.phase == Phase::Stop {
                            OperationOutcome::Succeeded {
                                generation: row.status.generation,
                            }
                        } else {
                            OperationOutcome::Unknown
                        };
                        op.updated_ms = now_ms();
                    }
                    row.status.pending_operation = None;
                    // The owner's empty-boundary proof permits a NEW explicit
                    // operation. It cannot reconstruct a lost launch result.
                    row.status.next_recovery = if transition.phase == Phase::Stop {
                        RecoveryAction::None
                    } else {
                        RecoveryAction::BlockedUnknown
                    };
                } else if row.status.next_recovery != RecoveryAction::BlockedUnknown {
                    self.schedule_recovery(row);
                }
            }
            Observation::Unknown => {
                self.mark_unknown(row, op.as_mut())?;
                return Ok(());
            }
        }
        if before != row.status || op.is_some() {
            self.store.commit(row, op.as_ref(), "reconciled")?;
        }
        Ok(())
    }
    fn schedule_recovery(&self, row: &mut StoredService) {
        if row.status.desired != DesiredState::Running || row.pending.is_some() {
            row.status.next_recovery = RecoveryAction::None;
            return;
        }
        if matches!(
            row.status.next_recovery,
            RecoveryAction::Scheduled { .. } | RecoveryAction::BlockedUnknown
        ) {
            return;
        }
        match row.spec.restart {
            RestartPolicy::Never => row.status.next_recovery = RecoveryAction::None,
            RestartPolicy::Bounded {
                max_restarts,
                initial_backoff_ms,
                max_backoff_ms,
                on_unhealthy,
            } => {
                let need = row.status.lifecycle == Lifecycle::Exited
                    || (on_unhealthy && row.status.health == Health::Unhealthy);
                if !need {
                    return;
                }
                if row.status.automatic_restarts >= max_restarts {
                    row.status.next_recovery = RecoveryAction::BudgetExhausted;
                    return;
                }
                if let Some(generation) = row.status.generation {
                    let delay = initial_backoff_ms
                        .saturating_mul(
                            1u64.checked_shl(row.status.automatic_restarts.min(63))
                                .unwrap_or(u64::MAX),
                        )
                        .min(max_backoff_ms);
                    row.status.next_recovery = RecoveryAction::Scheduled {
                        generation,
                        at_ms: now_ms().saturating_add(delay),
                    };
                }
            }
        }
    }

    /// One bounded observation/recovery pass. Daemons normally call every 250 ms.
    pub async fn tick(self: &Arc<Self>) -> Result<()> {
        let _permit = self.admission.read().await;
        if self.is_shutting_down() {
            return Err(Error::ShuttingDown);
        }
        self.validate_control().await?;
        let mut work = tokio::task::JoinSet::new();
        for row in self.store.all()? {
            let this = self.clone();
            work.spawn(async move { this.tick_service(row.spec.key).await });
        }
        while let Some(r) = work.join_next().await {
            r.map_err(|_| Error::Unknown)??;
        }
        Ok(())
    }
    async fn tick_service(self: Arc<Self>, key: ServiceKey) -> Result<()> {
        let lock = self.service_lock(&key)?;
        let mut due = Vec::new();
        {
            let _guard = lock.lock().await;
            let mut row = self.store.get(&key)?.ok_or(Error::NotFound)?;
            self.reconcile_locked(&mut row).await?;
            if let RecoveryAction::Scheduled { generation, at_ms } = row.status.next_recovery {
                if at_ms <= now_ms()
                    && row.status.generation == Some(generation)
                    && row.status.desired == DesiredState::Running
                    && row.pending.is_none()
                    && row.status.lifecycle != Lifecycle::Unknown
                {
                    // Re-evaluate after the backoff, including a service which
                    // became healthy while an old recovery timer was outstanding.
                    let reason = row.status.lifecycle == Lifecycle::Exited
                        || (row.status.health == Health::Unhealthy
                            && matches!(
                                row.spec.restart,
                                RestartPolicy::Bounded {
                                    on_unhealthy: true,
                                    ..
                                }
                            ));
                    if reason {
                        self.validate_control().await?;
                        let request = OperationRequest {
                            operation_id: Uuid::new_v4(),
                            mutation: Mutation::Restart {
                                key: key.clone(),
                                expected_generation: generation,
                            },
                        };
                        self.stop_or_restart_locked(row, request, OperationOrigin::Automatic, true)
                            .await?;
                        return Ok(());
                    }
                    row.status.next_recovery = RecoveryAction::None;
                }
            }
            if row.status.desired == DesiredState::Running
                && row.status.lifecycle == Lifecycle::Running
                && row.pending.is_none()
            {
                let generation = row.status.generation.ok_or(Error::Unknown)?;
                let now = now_ms();
                if row.next_readiness_ms <= now {
                    row.readiness_sequence = row.readiness_sequence.wrapping_add(1);
                    row.next_readiness_ms = now.saturating_add(row.spec.readiness.interval_ms);
                    due.push((
                        ProbeRole::Readiness,
                        row.readiness_sequence,
                        ProbeRequest {
                            key: key.clone(),
                            generation,
                            spec: row.spec.readiness.clone(),
                        },
                    ));
                }
                if row.next_health_ms <= now {
                    row.health_sequence = row.health_sequence.wrapping_add(1);
                    row.next_health_ms = now.saturating_add(row.spec.health.interval_ms);
                    due.push((
                        ProbeRole::Health,
                        row.health_sequence,
                        ProbeRequest {
                            key: key.clone(),
                            generation,
                            spec: row.spec.health.clone(),
                        },
                    ));
                }
                if !due.is_empty() {
                    self.store.commit(&row, None, "probe_scheduled")?;
                }
            }
        }
        // A probe can be slow while stop/restart continues. Sequence AND generation
        // guards protect the eventual commit, including same-generation stop races.
        for (role, sequence, request) in due {
            let Ok(_slot) = self.probes.clone().try_acquire_owned() else {
                continue;
            };
            let generation = request.generation;
            let result = tokio::time::timeout(
                Duration::from_millis(request.spec.timeout_ms.saturating_add(250)),
                self.owner.probe(&self.ownership, request),
            )
            .await;
            let result = result.ok().and_then(|r| r.ok()).unwrap_or(ProbeResult {
                generation,
                class: ProbeClass::MechanismFailure,
                code: "probe_transport_timeout".into(),
                panes: vec![],
                expectations_met: None,
                observed_ms: now_ms(),
            });
            self.apply_probe(&key, role, sequence, result).await?;
        }
        Ok(())
    }
    async fn apply_probe(
        &self,
        key: &ServiceKey,
        role: ProbeRole,
        sequence: u64,
        mut result: ProbeResult,
    ) -> Result<()> {
        let lock = self.service_lock(key)?;
        let _guard = lock.lock().await;
        let mut row = self.store.get(key)?.ok_or(Error::NotFound)?;
        if row.status.generation != Some(result.generation)
            || row.status.desired != DesiredState::Running
            || row.status.lifecycle != Lifecycle::Running
            || row.pending.is_some()
            || sequence
                != match role {
                    ProbeRole::Readiness => row.readiness_sequence,
                    ProbeRole::Health => row.health_sequence,
                }
        {
            return Ok(());
        }
        self.validate_control().await?;
        // Only a fixed vocabulary reaches durable status. Even a broken owner
        // returning raw output in `code` cannot make that output a supervisor log.
        result.code = match result.class {
            ProbeClass::Success => "ok",
            ProbeClass::ServiceFailure => "probe_failed",
            ProbeClass::MechanismFailure => "probe_mechanism_failed",
            ProbeClass::TargetUnavailable => "target_unavailable",
        }
        .into();
        result.panes.truncate(1024);
        result.panes.retain(|p| {
            p.pane_id.len() <= 32
                && p.pane_id.starts_with('%')
                && p.pane_id[1..].bytes().all(|b| b.is_ascii_digit())
        });
        let in_grace = now_ms()
            < row
                .status
                .started_ms
                .unwrap_or(0)
                .saturating_add(row.spec.startup_grace_ms);
        match role {
            ProbeRole::Readiness => {
                let pass =
                    result.class == ProbeClass::Success && result.expectations_met != Some(false);
                if pass {
                    row.readiness_failures = 0;
                    row.status.ready = true;
                } else if matches!(
                    result.class,
                    ProbeClass::MechanismFailure | ProbeClass::TargetUnavailable
                ) {
                    row.readiness_failures = 0;
                    row.status.ready = false;
                } else if !in_grace {
                    row.readiness_failures = row.readiness_failures.saturating_add(1);
                    if row.readiness_failures >= row.spec.readiness.failure_threshold {
                        row.status.ready = false
                    }
                }
                row.status.last_readiness = Some(result);
            }
            ProbeRole::Health => {
                match result.class {
                    ProbeClass::Success => {
                        row.health_failures = 0;
                        row.status.health = Health::Healthy;
                        if matches!(row.status.next_recovery, RecoveryAction::Scheduled { .. }) {
                            row.status.next_recovery = RecoveryAction::None
                        }
                    }
                    ProbeClass::ServiceFailure if !in_grace => {
                        row.health_failures = row.health_failures.saturating_add(1);
                        row.status.health =
                            if row.health_failures >= row.spec.health.failure_threshold {
                                Health::Unhealthy
                            } else {
                                Health::Degraded
                            };
                        self.schedule_recovery(&mut row);
                    }
                    ProbeClass::ServiceFailure => row.status.health = Health::Unknown,
                    ProbeClass::MechanismFailure | ProbeClass::TargetUnavailable => {
                        row.health_failures = 0;
                        row.status.health = Health::Unknown;
                        if matches!(row.status.next_recovery, RecoveryAction::Scheduled { .. }) {
                            row.status.next_recovery = RecoveryAction::None
                        }
                    }
                }
                row.status.last_probe = Some(result);
            }
        }
        self.store.commit(&row, None, "probe_observed")?;
        Ok(())
    }
    pub async fn shutdown(self: &Arc<Self>) -> Result<()> {
        self.closing.store(true, Ordering::Release);
        // Wait for accepted work before releasing ownership. No detached mutation
        // may outlive this guard and continue under a replacement controller.
        let _drained = self.admission.write().await;
        let mut failure = None;
        for row in self.store.all()? {
            if row.spec.controller_exit == ControllerExit::StopServices
                && row.status.desired == DesiredState::Running
            {
                if let Some(generation) = row.status.generation {
                    let request = OperationRequest {
                        operation_id: Uuid::new_v4(),
                        mutation: Mutation::Stop {
                            key: row.spec.key.clone(),
                            expected_generation: generation,
                        },
                    };
                    match self.execute_inner(request, OperationOrigin::Shutdown).await {
                        Ok(op) if matches!(op.outcome, OperationOutcome::Succeeded { .. }) => {}
                        Ok(_) => failure = Some(Error::Unknown),
                        Err(e) => failure = Some(e),
                    }
                }
            }
        }
        if let Err(e) = bounded(self.owner.release(&self.ownership)).await {
            failure = Some(e)
        }
        if let Some(e) = failure {
            Err(e)
        } else {
            Ok(())
        }
    }
}

async fn bounded<T>(future: impl std::future::Future<Output = Result<T>>) -> Result<T> {
    tokio::time::timeout(OWNER_TIMEOUT, future)
        .await
        .map_err(|_| Error::Unavailable)?
}

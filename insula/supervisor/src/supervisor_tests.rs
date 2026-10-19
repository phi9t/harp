//! Adversarial lifecycle tests against an independent execution-owner model.
//!
//! The mock's reservation ledger deliberately outlives each Supervisor. A lost
//! reply and a dead controller therefore do not erase real launch/stop effects.
//! Real process, transport, and tmux tests live in the other integration suites.

use std::{
    collections::{BTreeMap, VecDeque},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use insula_supervisor::{supervisor::FailPoint, *};
use tempfile::TempDir;
use tokio::sync::Notify;
use uuid::Uuid;

#[derive(Default)]
struct LaunchGate {
    entered: Notify,
    released: Notify,
}

#[derive(Clone, Default)]
enum LaunchBehavior {
    #[default]
    Normal,
    /// The process exists and is recorded by its owner, but the reply is lost.
    LostReply,
    /// A durable reservation exists; whether a launch can materialize is unknown.
    Unresolved,
    Gated(Arc<LaunchGate>),
}

#[derive(Clone)]
struct ProbeBehavior {
    class: ProbeClass,
    generation_override: Option<Generation>,
    panes: Vec<PaneStatus>,
    expectations_met: Option<bool>,
}

impl Default for ProbeBehavior {
    fn default() -> Self {
        Self {
            class: ProbeClass::Success,
            generation_override: None,
            panes: Vec::new(),
            expectations_met: None,
        }
    }
}

#[derive(Clone)]
enum Instance {
    Reserved(Generation),
    Alive(ProcessIdentity),
    Gone {
        generation: Generation,
        last_exit: ExitInfo,
        boundary_empty: bool,
    },
}

impl Instance {
    fn generation(&self) -> Generation {
        match self {
            Self::Reserved(g) | Self::Gone { generation: g, .. } => *g,
            Self::Alive(identity) => identity.generation,
        }
    }

    fn observation(&self) -> Observation {
        match self {
            Self::Reserved(_) => Observation::Unknown,
            Self::Alive(identity) => Observation::Alive {
                identity: identity.clone(),
            },
            Self::Gone {
                generation,
                last_exit,
                boundary_empty,
            } => Observation::Gone {
                generation: Some(*generation),
                last_exit: Some(last_exit.clone()),
                boundary_empty: *boundary_empty,
                launch_resolved: true,
            },
        }
    }
}

#[derive(Default)]
struct OwnerState {
    claim: Option<Ownership>,
    epoch: u64,
    instances: BTreeMap<ServiceKey, Instance>,
    generation_history: BTreeMap<Generation, Instance>,
    // Reservations are independent of the supervisor's store and never replay.
    reservations: BTreeMap<Generation, (ServiceKey, OperationId)>,
    launches: Vec<LaunchRequest>,
    launch_invocations: usize,
    stops: Vec<StopRequest>,
    stop_invocations: usize,
    launch_behaviors: VecDeque<LaunchBehavior>,
    probe_behavior: ProbeBehavior,
    next_probe_gate: Option<Arc<LaunchGate>>,
    observation_override: Option<Observation>,
    incomplete_stop_boundary: bool,
    ambiguous_stop_alive: bool,
}

struct MockOwner {
    owner_instance: Uuid,
    authority_dir: PathBuf,
    state: Mutex<OwnerState>,
}

impl MockOwner {
    fn new(authority_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&authority_dir).unwrap();
        std::fs::set_permissions(&authority_dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            owner_instance: Uuid::new_v4(),
            authority_dir,
            state: Mutex::new(OwnerState::default()),
        }
    }

    fn check(state: &OwnerState, ownership: &Ownership) -> Result<()> {
        if state.claim.as_ref() == Some(ownership) {
            Ok(())
        } else {
            Err(Error::Ownership)
        }
    }

    fn launches(&self) -> usize {
        self.state.lock().unwrap().launches.len()
    }

    fn launch_invocations(&self) -> usize {
        self.state.lock().unwrap().launch_invocations
    }

    fn stops(&self) -> Vec<StopRequest> {
        self.state.lock().unwrap().stops.clone()
    }

    fn is_alive(&self, key: &ServiceKey, generation: Generation) -> bool {
        matches!(self.state.lock().unwrap().instances.get(key),
            Some(Instance::Alive(identity)) if identity.generation == generation)
    }

    /// Models the execution owner's detection of a dead controller connection.
    /// This is deliberately a test-fixture operation, not a supervisor API.
    fn crash_controller(&self) {
        self.state.lock().unwrap().claim = None;
    }

    fn next_launch(&self, behavior: LaunchBehavior) {
        self.state
            .lock()
            .unwrap()
            .launch_behaviors
            .push_back(behavior);
    }

    fn set_probe(&self, behavior: ProbeBehavior) {
        self.state.lock().unwrap().probe_behavior = behavior;
    }

    fn complete(&self, key: &ServiceKey, code: i32) {
        let mut state = self.state.lock().unwrap();
        let generation = state.instances.get(key).unwrap().generation();
        let gone = Instance::Gone {
            generation,
            last_exit: ExitInfo {
                code: Some(code),
                signal: None,
                observed_ms: now_ms(),
            },
            boundary_empty: true,
        };
        state.generation_history.insert(generation, gone.clone());
        state.instances.insert(key.clone(), gone);
    }
}

#[async_trait]
impl ExecutionOwner for MockOwner {
    async fn capabilities(&self, root_insula_id: &str) -> Result<OwnerCapabilities> {
        if root_insula_id != "root" {
            return Err(Error::NotFound);
        }
        Ok(OwnerCapabilities {
            owner_instance: self.owner_instance,
            root_insula_id: "root".into(),
            canonical_authority_dir: self.authority_dir.clone(),
            generation_fencing: true,
            controller_fencing: true,
            durable_launch_reservations: true,
            service_boundary_cleanup: true,
            bounded_probes: true,
            bounded_logs: true,
            survives_controller_exit: true,
            details: vec!["Independent durable owner model; no real processes".into()],
        })
    }

    async fn claim(&self, root_insula_id: &str, controller_id: Uuid) -> Result<Ownership> {
        let mut state = self.state.lock().unwrap();
        if root_insula_id != "root" || state.claim.is_some() {
            return Err(Error::Ownership);
        }
        state.epoch += 1;
        let claim = Ownership {
            root_insula_id: root_insula_id.into(),
            owner_instance: self.owner_instance,
            controller_id,
            epoch: state.epoch,
        };
        state.claim = Some(claim.clone());
        Ok(claim)
    }

    async fn validate(&self, ownership: &Ownership) -> Result<()> {
        Self::check(&self.state.lock().unwrap(), ownership)
    }

    async fn observe(
        &self,
        ownership: &Ownership,
        key: &ServiceKey,
        generation: Option<Generation>,
    ) -> Result<Observation> {
        let state = self.state.lock().unwrap();
        Self::check(&state, ownership)?;
        if let Some(observation) = &state.observation_override {
            return Ok(observation.clone());
        }
        Ok(match state.instances.get(key) {
            Some(instance) => instance.observation(),
            // No reservation has reached this owner, so it can certify absence.
            None => Observation::Gone {
                generation,
                last_exit: None,
                boundary_empty: true,
                launch_resolved: true,
            },
        })
    }

    async fn launch(&self, ownership: &Ownership, request: LaunchRequest) -> Result<Observation> {
        let behavior = {
            let mut state = self.state.lock().unwrap();
            Self::check(&state, ownership)?;
            state.launch_invocations += 1;
            if let Some((key, operation)) = state.reservations.get(&request.generation) {
                if *key != request.spec.key || *operation != request.operation_id {
                    return Err(Error::Conflict("reservation payload changed".into()));
                }
                return Ok(state
                    .generation_history
                    .get(&request.generation)
                    .unwrap()
                    .observation());
            }
            if let Some(instance) = state.instances.get(&request.spec.key) {
                match instance {
                    Instance::Alive(_) => return Err(Error::Conflict("already live".into())),
                    Instance::Reserved(_) => return Err(Error::Unknown),
                    Instance::Gone {
                        boundary_empty: false,
                        ..
                    } => return Err(Error::Unknown),
                    Instance::Gone { .. } => {}
                }
            }
            state.reservations.insert(
                request.generation,
                (request.spec.key.clone(), request.operation_id),
            );
            state.instances.insert(
                request.spec.key.clone(),
                Instance::Reserved(request.generation),
            );
            state
                .generation_history
                .insert(request.generation, Instance::Reserved(request.generation));
            state.launch_behaviors.pop_front().unwrap_or_default()
        };

        if let LaunchBehavior::Gated(gate) = &behavior {
            gate.entered.notify_one();
            gate.released.notified().await;
        }
        if matches!(behavior, LaunchBehavior::Unresolved) {
            return Err(Error::Unknown);
        }

        let mut state = self.state.lock().unwrap();
        // The important check is after an await, immediately before the effect.
        Self::check(&state, ownership)?;
        let identity = ProcessIdentity {
            owner_instance: self.owner_instance,
            boot_id: "mock-boot".into(),
            pid: 10_000 + state.launches.len() as u32,
            start_time_ticks: 500_000 + state.launches.len() as u64,
            generation: request.generation,
        };
        state
            .instances
            .insert(request.spec.key.clone(), Instance::Alive(identity.clone()));
        state
            .generation_history
            .insert(request.generation, Instance::Alive(identity.clone()));
        state.launches.push(request);
        if matches!(behavior, LaunchBehavior::LostReply) {
            Err(Error::Unavailable)
        } else {
            Ok(Observation::Alive { identity })
        }
    }

    async fn stop(&self, ownership: &Ownership, request: StopRequest) -> Result<Observation> {
        let mut state = self.state.lock().unwrap();
        Self::check(&state, ownership)?;
        state.stop_invocations += 1;
        let instance = state.instances.get(&request.key).ok_or(Error::NotFound)?;
        if instance.generation() != request.expected_generation {
            return Err(Error::StaleGeneration);
        }
        match instance {
            Instance::Reserved(_) => return Err(Error::Unknown),
            Instance::Gone { .. } => return Ok(instance.observation()),
            Instance::Alive(_) => {}
        }
        if state.ambiguous_stop_alive {
            // The caller has no proof this stop cannot complete in the future.
            return Err(Error::Unknown);
        }
        let gone = Instance::Gone {
            generation: request.expected_generation,
            last_exit: ExitInfo {
                code: Some(0),
                signal: None,
                observed_ms: now_ms(),
            },
            boundary_empty: !state.incomplete_stop_boundary,
        };
        let observation = gone.observation();
        state
            .generation_history
            .insert(request.expected_generation, gone.clone());
        state.instances.insert(request.key.clone(), gone);
        state.stops.push(request);
        Ok(observation)
    }

    async fn probe(&self, ownership: &Ownership, request: ProbeRequest) -> Result<ProbeResult> {
        let (behavior, gate) = {
            let mut state = self.state.lock().unwrap();
            Self::check(&state, ownership)?;
            (state.probe_behavior.clone(), state.next_probe_gate.take())
        };
        if let Some(gate) = gate {
            gate.entered.notify_one();
            gate.released.notified().await;
        }
        Ok(ProbeResult {
            generation: behavior.generation_override.unwrap_or(request.generation),
            class: behavior.class,
            code: match behavior.class {
                ProbeClass::Success => "ready",
                ProbeClass::ServiceFailure => "service_unresponsive",
                ProbeClass::MechanismFailure => "probe_mechanism_failed",
                ProbeClass::TargetUnavailable => "insula_unavailable",
            }
            .into(),
            panes: behavior.panes.clone(),
            expectations_met: behavior.expectations_met,
            observed_ms: now_ms(),
        })
    }

    async fn release(&self, ownership: &Ownership) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        Self::check(&state, ownership)?;
        state.claim = None;
        Ok(())
    }
}

struct Fixture {
    directory: TempDir,
    owner: Arc<MockOwner>,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let owner = Arc::new(MockOwner::new(directory.path().join("authority")));
        Self { directory, owner }
    }

    async fn open(&self) -> Arc<Supervisor> {
        Supervisor::open(
            self.owner.clone(),
            "root".into(),
            self.directory.path().join("state"),
        )
        .await
        .unwrap()
    }

    async fn recover(&self, supervisor: Arc<Supervisor>) -> Arc<Supervisor> {
        drop(supervisor);
        self.owner.crash_controller();
        self.open().await
    }
}

fn spec() -> ServiceSpec {
    ServiceSpec {
        key: ServiceKey {
            insula_id: "root".into(),
            service_id: "service".into(),
        },
        revision: "revision-1".into(),
        command: CommandSpec {
            executable: "/mock/service".into(),
            args: vec!["--foreground".into()],
            cwd: "/work".into(),
            env_refs: BTreeMap::from([("ACCESS_TOKEN".into(), "secret-ref-only".into())]),
            identity: RuntimeIdentity {
                uid: 1000,
                gid: 1000,
            },
        },
        adapter: ServiceAdapter::Foreground,
        readiness: ProbeSpec {
            interval_ms: 1,
            failure_threshold: 1,
            ..ProbeSpec::default()
        },
        health: ProbeSpec {
            interval_ms: 1,
            failure_threshold: 1,
            ..ProbeSpec::default()
        },
        startup_grace_ms: 0,
        restart: RestartPolicy::Never,
        replay_safe: false,
        stop: StopPolicy {
            graceful: GracefulStop::Terminate,
            deadline_ms: 100,
            force: true,
        },
        logs: LogPolicy::default(),
        cleanup_boundary: CleanupBoundary::Service,
        controller_exit: ControllerExit::KeepServices,
    }
}

fn restartable_spec() -> ServiceSpec {
    ServiceSpec {
        restart: RestartPolicy::Bounded {
            max_restarts: 1,
            initial_backoff_ms: 1,
            max_backoff_ms: 1,
            on_unhealthy: true,
        },
        replay_safe: true,
        ..spec()
    }
}

fn start_request(spec: &ServiceSpec) -> OperationRequest {
    OperationRequest {
        operation_id: Uuid::new_v4(),
        mutation: Mutation::Start {
            spec: Box::new(spec.clone()),
        },
    }
}

fn stop_request(key: &ServiceKey, generation: Generation) -> OperationRequest {
    OperationRequest {
        operation_id: Uuid::new_v4(),
        mutation: Mutation::Stop {
            key: key.clone(),
            expected_generation: generation,
        },
    }
}

fn restart_request(key: &ServiceKey, generation: Generation) -> OperationRequest {
    OperationRequest {
        operation_id: Uuid::new_v4(),
        mutation: Mutation::Restart {
            key: key.clone(),
            expected_generation: generation,
        },
    }
}

fn succeeded(result: Result<OperationRecord>) -> OperationRecord {
    let record = result.unwrap();
    assert!(
        matches!(record.outcome, OperationOutcome::Succeeded { .. }),
        "{record:?}"
    );
    record
}

fn rejected(result: Result<OperationRecord>, expected: Error) {
    match result {
        Err(error) => assert_eq!(error, expected),
        Ok(record) => assert_eq!(record.outcome, OperationOutcome::Failed { error: expected }),
    }
}

fn uncertain(result: Result<OperationRecord>) {
    match result {
        Err(Error::Unknown | Error::Unavailable) => {}
        Ok(OperationRecord {
            outcome: OperationOutcome::Unknown | OperationOutcome::Pending,
            ..
        }) => {}
        unexpected => panic!("expected an uncertain operation, got {unexpected:?}"),
    }
}

async fn start(supervisor: &Arc<Supervisor>, spec: &ServiceSpec) -> Generation {
    succeeded(supervisor.execute(start_request(spec)).await);
    supervisor.inspect(&spec.key).unwrap().generation.unwrap()
}

async fn next_tick(supervisor: &Arc<Supervisor>) {
    tokio::time::sleep(Duration::from_millis(3)).await;
    supervisor.tick().await.unwrap();
}

async fn await_generation_change(
    supervisor: &Arc<Supervisor>,
    key: &ServiceKey,
    old: Generation,
) -> Generation {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            next_tick(supervisor).await;
            let status = supervisor.inspect(key).unwrap();
            if let Some(generation) = status.generation {
                if generation != old && status.lifecycle == Lifecycle::Running {
                    return generation;
                }
            }
        }
    })
    .await
    .expect("automatic replacement did not converge")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_duplicate_and_distinct_start_requests_converge_on_one_instance() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = spec();
    let original = start_request(&service);
    let mut clients = Vec::new();
    for index in 0..32 {
        let supervisor = supervisor.clone();
        let request = if index < 16 {
            original.clone()
        } else {
            start_request(&service)
        };
        clients.push(tokio::spawn(
            async move { supervisor.execute(request).await },
        ));
    }
    let mut generation = None;
    for client in clients {
        let record = succeeded(client.await.unwrap());
        if let OperationOutcome::Succeeded {
            generation: current,
        } = record.outcome
        {
            if generation.is_none() {
                generation = current;
            }
            assert_eq!(current, generation);
        }
    }
    assert_eq!(fixture.owner.launches(), 1);
    assert_eq!(
        supervisor.inspect(&service.key).unwrap().generation,
        generation
    );
}

#[tokio::test]
async fn operation_id_retry_returns_recorded_outcome_and_changed_payload_conflicts() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = spec();
    let request = start_request(&service);
    let first = succeeded(supervisor.execute(request.clone()).await);
    assert_eq!(first, succeeded(supervisor.execute(request.clone()).await));
    let mut changed = service.clone();
    changed.revision = "revision-2".into();
    let conflicting = OperationRequest {
        operation_id: request.operation_id,
        mutation: Mutation::Start {
            spec: Box::new(changed),
        },
    };
    let result = supervisor.execute(conflicting).await;
    assert!(
        matches!(result, Err(Error::Conflict(_)))
            || matches!(
                result,
                Ok(OperationRecord {
                    outcome: OperationOutcome::Failed {
                        error: Error::Conflict(_)
                    },
                    ..
                })
            )
    );
    assert_eq!(fixture.owner.launches(), 1);
    assert!(fixture.owner.stops().is_empty());
}

#[tokio::test]
async fn stale_stop_restart_and_probe_results_cannot_affect_a_replacement() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = restartable_spec();
    let old = start(&supervisor, &service).await;
    succeeded(supervisor.execute(restart_request(&service.key, old)).await);
    let current = supervisor
        .inspect(&service.key)
        .unwrap()
        .generation
        .unwrap();
    assert_ne!(old, current);
    rejected(
        supervisor.execute(stop_request(&service.key, old)).await,
        Error::StaleGeneration,
    );
    rejected(
        supervisor.execute(restart_request(&service.key, old)).await,
        Error::StaleGeneration,
    );
    fixture.owner.set_probe(ProbeBehavior {
        class: ProbeClass::ServiceFailure,
        generation_override: Some(old),
        panes: Vec::new(),
        expectations_met: None,
    });
    next_tick(&supervisor).await;
    next_tick(&supervisor).await;
    let status = supervisor.inspect(&service.key).unwrap();
    assert_eq!(status.generation, Some(current));
    assert_eq!(status.desired, DesiredState::Running);
    assert_ne!(status.health, Health::Unhealthy);
    assert!(status
        .last_probe
        .as_ref()
        .is_none_or(|probe| probe.generation != old));
    assert!(status
        .last_readiness
        .as_ref()
        .is_none_or(|probe| probe.generation != old));
    assert_eq!(fixture.owner.launches(), 2);
    assert_eq!(fixture.owner.stops().len(), 1);
    assert!(fixture.owner.is_alive(&service.key, current));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_inflight_probe_does_not_block_restart_and_its_late_result_is_discarded() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = restartable_spec();
    let old = start(&supervisor, &service).await;
    let gate = Arc::new(LaunchGate::default());
    {
        let mut state = fixture.owner.state.lock().unwrap();
        state.next_probe_gate = Some(gate.clone());
        state.probe_behavior.class = ProbeClass::ServiceFailure;
    }
    let probing = {
        let supervisor = supervisor.clone();
        tokio::spawn(async move { supervisor.tick().await })
    };
    tokio::time::timeout(Duration::from_secs(2), gate.entered.notified())
        .await
        .unwrap();
    // Restart must proceed while the old-generation observation is suspended.
    succeeded(
        tokio::time::timeout(
            Duration::from_millis(500),
            supervisor.execute(restart_request(&service.key, old)),
        )
        .await
        .expect("a probe held the lifecycle mutation lock"),
    );
    let replacement = supervisor
        .inspect(&service.key)
        .unwrap()
        .generation
        .unwrap();
    gate.released.notify_one();
    probing.await.unwrap().unwrap();
    let status = supervisor.inspect(&service.key).unwrap();
    assert_ne!(replacement, old);
    assert_eq!(status.generation, Some(replacement));
    assert_ne!(status.health, Health::Unhealthy);
    assert!(status.last_readiness.is_none());
    assert!(status.last_probe.is_none());
    assert_eq!(fixture.owner.launches(), 2);
    assert!(fixture.owner.is_alive(&service.key, replacement));
}

#[tokio::test]
async fn different_state_directories_do_not_bypass_single_controller_ownership() {
    let fixture = Fixture::new();
    let first = fixture.open().await;
    let service = spec();
    let generation = start(&first, &service).await;
    let second = Supervisor::open(
        fixture.owner.clone(),
        "root".into(),
        fixture.directory.path().join("different-state"),
    )
    .await;
    assert!(
        matches!(second, Err(Error::Ownership)),
        "a second controller acquired the same root"
    );
    assert_eq!(fixture.owner.launches(), 1);
    assert!(fixture.owner.is_alive(&service.key, generation));
}

#[tokio::test]
async fn the_canonical_file_lock_also_blocks_independently_claimed_owner_handles() {
    let fixture = Fixture::new();
    let _first = fixture.open().await;
    // Deliberately independent claim memory isolates the file lock's protection
    // from the owner's separate fencing check. Both advertise one authority.
    let independent_handle = Arc::new(MockOwner::new(fixture.owner.authority_dir.clone()));
    let second = Supervisor::open(
        independent_handle,
        "root".into(),
        fixture.directory.path().join("independent-state"),
    )
    .await;
    assert!(
        matches!(second, Err(Error::Ownership)),
        "canonical flock was bypassed by a different state directory"
    );
}

#[tokio::test]
async fn uncertain_controller_ownership_blocks_stop_and_restart_before_effects() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = spec();
    let generation = start(&supervisor, &service).await;
    fixture.owner.crash_controller();
    rejected(
        supervisor
            .execute(stop_request(&service.key, generation))
            .await,
        Error::Ownership,
    );
    rejected(
        supervisor
            .execute(restart_request(&service.key, generation))
            .await,
        Error::Ownership,
    );
    assert!(fixture.owner.stops().is_empty());
    assert!(fixture.owner.is_alive(&service.key, generation));
    assert_eq!(
        supervisor.inspect(&service.key).unwrap().desired,
        DesiredState::Running
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_cancelled_client_does_not_cancel_or_duplicate_the_durable_operation() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = spec();
    let request = start_request(&service);
    let gate = Arc::new(LaunchGate::default());
    fixture
        .owner
        .next_launch(LaunchBehavior::Gated(gate.clone()));
    let client = {
        let supervisor = supervisor.clone();
        let request = request.clone();
        tokio::spawn(async move { supervisor.execute(request).await })
    };
    tokio::time::timeout(Duration::from_secs(2), gate.entered.notified())
        .await
        .unwrap();
    client.abort();
    let _ = client.await;
    gate.released.notify_one();
    let record = tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match supervisor.execute(request.clone()).await {
                Ok(record) if matches!(record.outcome, OperationOutcome::Succeeded { .. }) => {
                    break record
                }
                Ok(record) if matches!(record.outcome, OperationOutcome::Pending) => {}
                Err(Error::Pending) => {}
                other => panic!("retry lost its in-flight operation: {other:?}"),
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(fixture.owner.launches(), 1);
    assert_eq!(fixture.owner.launch_invocations(), 1);
    assert_eq!(record, succeeded(supervisor.execute(request).await));
}

#[tokio::test]
async fn recovery_adopts_a_surviving_generation_and_keeps_operation_identity() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = spec();
    let request = start_request(&service);
    let record = succeeded(supervisor.execute(request.clone()).await);
    let generation = supervisor.inspect(&service.key).unwrap().generation;
    let recovered = fixture.recover(supervisor).await;
    recovered.reconcile_all().await.unwrap();
    let status = recovered.inspect(&service.key).unwrap();
    assert_eq!(status.generation, generation);
    assert_eq!(status.lifecycle, Lifecycle::Running);
    assert_eq!(fixture.owner.launches(), 1);
    assert_eq!(record, succeeded(recovered.execute(request).await));
}

#[tokio::test]
async fn a_lost_launch_reply_is_reconciled_without_a_second_launch() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = spec();
    let request = start_request(&service);
    fixture.owner.next_launch(LaunchBehavior::LostReply);
    uncertain(supervisor.execute(request.clone()).await);
    assert_eq!(fixture.owner.launches(), 1);
    let original_generation = supervisor.inspect(&service.key).unwrap().generation;
    let recovered = fixture.recover(supervisor).await;
    recovered.reconcile_all().await.unwrap();
    let status = recovered.inspect(&service.key).unwrap();
    assert_eq!(status.generation, original_generation);
    assert_eq!(status.lifecycle, Lifecycle::Running);
    let _ = recovered.execute(request).await;
    assert_eq!(fixture.owner.launches(), 1);
    assert_eq!(fixture.owner.launch_invocations(), 1);
}

#[tokio::test]
async fn launch_persistence_crash_windows_reconcile_or_remain_unknown_without_replay() {
    for (failpoint, expected_launches) in [
        (FailPoint::AfterIntent, 0),
        (FailPoint::AfterOwnerLaunch, 1),
    ] {
        let fixture = Fixture::new();
        let supervisor = fixture.open().await;
        let service = spec();
        let request = start_request(&service);
        supervisor.set_failpoint(Some(failpoint));
        uncertain(supervisor.execute(request.clone()).await);
        assert_eq!(fixture.owner.launches(), expected_launches);
        let generation = supervisor.inspect(&service.key).unwrap().generation;
        let recovered = fixture.recover(supervisor).await;
        recovered.reconcile_all().await.unwrap();
        next_tick(&recovered).await;
        let _ = recovered.execute(request).await;
        assert_eq!(fixture.owner.launches(), expected_launches);
        let status = recovered.inspect(&service.key).unwrap();
        assert_eq!(status.generation, generation);
        if expected_launches == 1 {
            assert_eq!(status.lifecycle, Lifecycle::Running);
        } else {
            assert!(matches!(
                status.lifecycle,
                Lifecycle::Unknown | Lifecycle::Exited | Lifecycle::Absent
            ));
        }
    }
}

#[tokio::test]
async fn an_unresolved_owner_reservation_blocks_replacement_and_destructive_recovery() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = restartable_spec();
    fixture.owner.next_launch(LaunchBehavior::Unresolved);
    uncertain(supervisor.execute(start_request(&service)).await);
    let generation = supervisor
        .inspect(&service.key)
        .unwrap()
        .generation
        .unwrap();
    let recovered = fixture.recover(supervisor).await;
    assert_eq!(
        recovered.inspect(&service.key).unwrap().lifecycle,
        Lifecycle::Unknown
    );
    uncertain(recovered.execute(start_request(&service)).await);
    uncertain(
        recovered
            .execute(restart_request(&service.key, generation))
            .await,
    );
    uncertain(
        recovered
            .execute(stop_request(&service.key, generation))
            .await,
    );
    next_tick(&recovered).await;
    assert_eq!(fixture.owner.launches(), 0);
    assert_eq!(fixture.owner.launch_invocations(), 1);
    assert!(fixture.owner.stops().is_empty());
}

#[tokio::test]
async fn a_crash_after_stop_preserves_stopped_intent_and_never_relaunches() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = restartable_spec();
    let generation = start(&supervisor, &service).await;
    supervisor.set_failpoint(Some(FailPoint::AfterOwnerStop));
    let request = stop_request(&service.key, generation);
    uncertain(supervisor.execute(request.clone()).await);
    assert_eq!(fixture.owner.stops().len(), 1);
    let recovered = fixture.recover(supervisor).await;
    next_tick(&recovered).await;
    let _ = recovered.execute(request).await;
    let status = recovered.inspect(&service.key).unwrap();
    assert_eq!(status.desired, DesiredState::Stopped);
    assert!(matches!(
        status.lifecycle,
        Lifecycle::Exited | Lifecycle::Absent
    ));
    assert_eq!(fixture.owner.launches(), 1);
    assert_eq!(fixture.owner.stops().len(), 1);
}

#[tokio::test]
async fn a_crash_during_explicit_restart_does_not_invent_a_replacement_launch() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = spec();
    let generation = start(&supervisor, &service).await;
    supervisor.set_failpoint(Some(FailPoint::AfterOwnerStop));
    let request = restart_request(&service.key, generation);
    uncertain(supervisor.execute(request.clone()).await);
    let recovered = fixture.recover(supervisor).await;
    next_tick(&recovered).await;
    let _ = recovered.execute(request).await;
    assert_eq!(fixture.owner.launches(), 1);
    assert_eq!(fixture.owner.stops().len(), 1);
    assert_ne!(
        recovered.inspect(&service.key).unwrap().lifecycle,
        Lifecycle::Running
    );
}

#[tokio::test]
async fn replacement_requires_the_owner_to_certify_an_empty_cleanup_boundary() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = restartable_spec();
    let generation = start(&supervisor, &service).await;
    fixture.owner.state.lock().unwrap().incomplete_stop_boundary = true;
    uncertain(
        supervisor
            .execute(restart_request(&service.key, generation))
            .await,
    );
    next_tick(&supervisor).await;
    assert_eq!(
        supervisor.inspect(&service.key).unwrap().lifecycle,
        Lifecycle::Unknown
    );
    assert_eq!(fixture.owner.launches(), 1);
    assert_eq!(fixture.owner.stops().len(), 1);
}

#[tokio::test]
async fn default_recovery_does_not_replay_a_completed_command() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let mut service = spec();
    service.command.executable = "/mock/completed-training-counter".into();
    let request = start_request(&service);
    let original = succeeded(supervisor.execute(request.clone()).await);
    fixture.owner.complete(&service.key, 0);
    let recovered = fixture.recover(supervisor).await;
    next_tick(&recovered).await;
    next_tick(&recovered).await;
    assert_eq!(fixture.owner.launches(), 1, "completed work was replayed");
    assert_eq!(fixture.owner.launch_invocations(), 1);
    let status = recovered.inspect(&service.key).unwrap();
    assert_eq!(status.lifecycle, Lifecycle::Exited);
    assert_eq!(status.automatic_restarts, 0);
    assert_eq!(status.last_exit.unwrap().code, Some(0));
    assert_eq!(original, succeeded(recovered.execute(request).await));
}

#[tokio::test]
async fn automatic_restart_budget_survives_controller_recovery_and_manual_restart() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = restartable_spec();
    let first = start(&supervisor, &service).await;
    fixture.owner.complete(&service.key, 1);
    let second = await_generation_change(&supervisor, &service.key, first).await;
    assert_eq!(fixture.owner.launches(), 2);
    assert_eq!(
        supervisor.inspect(&service.key).unwrap().automatic_restarts,
        1
    );
    fixture.owner.complete(&service.key, 1);
    next_tick(&supervisor).await;
    next_tick(&supervisor).await;
    assert_eq!(
        supervisor.inspect(&service.key).unwrap().next_recovery,
        RecoveryAction::BudgetExhausted
    );
    let recovered = fixture.recover(supervisor).await;
    next_tick(&recovered).await;
    assert_eq!(fixture.owner.launches(), 2);
    assert_eq!(
        recovered.inspect(&service.key).unwrap().next_recovery,
        RecoveryAction::BudgetExhausted
    );
    succeeded(
        recovered
            .execute(restart_request(&service.key, second))
            .await,
    );
    assert_eq!(fixture.owner.launches(), 3);
    assert_eq!(
        recovered.inspect(&service.key).unwrap().automatic_restarts,
        1
    );
    fixture.owner.complete(&service.key, 1);
    next_tick(&recovered).await;
    next_tick(&recovered).await;
    assert_eq!(fixture.owner.launches(), 3);
    assert_eq!(
        recovered.inspect(&service.key).unwrap().next_recovery,
        RecoveryAction::BudgetExhausted
    );
}

#[tokio::test]
async fn mechanism_and_target_failures_are_unknown_health_without_automatic_restart() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = restartable_spec();
    let generation = start(&supervisor, &service).await;
    for class in [ProbeClass::MechanismFailure, ProbeClass::TargetUnavailable] {
        fixture.owner.set_probe(ProbeBehavior {
            class,
            ..ProbeBehavior::default()
        });
        next_tick(&supervisor).await;
        next_tick(&supervisor).await;
        let status = supervisor.inspect(&service.key).unwrap();
        assert_eq!(status.generation, Some(generation));
        assert_eq!(status.health, Health::Unknown);
        assert_eq!(status.last_probe.unwrap().class, class);
        assert_ne!(status.next_recovery, RecoveryAction::BudgetExhausted);
        assert_eq!(fixture.owner.launches(), 1);
        assert!(fixture.owner.stops().is_empty());
    }
}

#[tokio::test]
async fn a_failed_tmux_pane_does_not_make_its_server_unhealthy_or_replay_work() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let mut service = restartable_spec();
    service.command.executable = "/mock/tmux".into();
    service.command.args.clear();
    service.adapter = ServiceAdapter::Tmux {
        socket: "/private/tmux.sock".into(),
    };
    let probe = ProbeKind::Tmux {
        executable: "/mock/tmux".into(),
        socket: "/private/tmux.sock".into(),
        expected_sessions: vec!["work".into()],
        expected_panes: vec!["%0".into()],
    };
    service.readiness.probe = probe.clone();
    service.health.probe = probe;
    let generation = start(&supervisor, &service).await;
    fixture.owner.set_probe(ProbeBehavior {
        panes: vec![PaneStatus {
            pane_id: "%0".into(),
            dead: true,
            exit_code: Some(7),
        }],
        expectations_met: Some(false),
        ..ProbeBehavior::default()
    });
    next_tick(&supervisor).await;
    next_tick(&supervisor).await;
    let status = supervisor.inspect(&service.key).unwrap();
    assert_eq!(status.generation, Some(generation));
    assert_eq!(status.health, Health::Healthy);
    assert!(
        !status.ready,
        "missing viable expected panes must affect readiness only"
    );
    assert_eq!(status.last_probe.unwrap().panes[0].exit_code, Some(7));
    assert_eq!(fixture.owner.launches(), 1);
    assert!(fixture.owner.stops().is_empty());
}

#[tokio::test]
async fn stop_cancels_a_pending_backoff_without_late_replay() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = ServiceSpec {
        restart: RestartPolicy::Bounded {
            max_restarts: 1,
            initial_backoff_ms: 50,
            max_backoff_ms: 50,
            on_unhealthy: true,
        },
        ..restartable_spec()
    };
    let generation = start(&supervisor, &service).await;
    fixture.owner.complete(&service.key, 1);
    supervisor.tick().await.unwrap();
    assert!(matches!(
        supervisor.inspect(&service.key).unwrap().next_recovery,
        RecoveryAction::Scheduled { .. }
    ));
    succeeded(
        supervisor
            .execute(stop_request(&service.key, generation))
            .await,
    );
    tokio::time::sleep(Duration::from_millis(60)).await;
    supervisor.tick().await.unwrap();
    assert_eq!(fixture.owner.launches(), 1);
    let status = supervisor.inspect(&service.key).unwrap();
    assert_eq!(status.desired, DesiredState::Stopped);
    assert_eq!(status.next_recovery, RecoveryAction::None);
}

#[tokio::test]
async fn manual_replacement_invalidates_the_previous_generations_recovery_timer() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = ServiceSpec {
        restart: RestartPolicy::Bounded {
            max_restarts: 1,
            initial_backoff_ms: 50,
            max_backoff_ms: 50,
            on_unhealthy: true,
        },
        ..restartable_spec()
    };
    let old = start(&supervisor, &service).await;
    fixture.owner.complete(&service.key, 1);
    supervisor.tick().await.unwrap();
    assert!(
        matches!(supervisor.inspect(&service.key).unwrap().next_recovery,
        RecoveryAction::Scheduled { generation, .. } if generation == old)
    );
    succeeded(supervisor.execute(restart_request(&service.key, old)).await);
    let current = supervisor
        .inspect(&service.key)
        .unwrap()
        .generation
        .unwrap();
    tokio::time::sleep(Duration::from_millis(60)).await;
    supervisor.tick().await.unwrap();
    let status = supervisor.inspect(&service.key).unwrap();
    assert_eq!(status.generation, Some(current));
    assert_eq!(status.automatic_restarts, 0);
    assert_eq!(fixture.owner.launches(), 2);
    assert!(fixture.owner.is_alive(&service.key, current));
}

#[tokio::test]
async fn shutdown_keep_services_rejects_mutations_and_allows_later_adoption() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = spec();
    let generation = start(&supervisor, &service).await;
    supervisor.shutdown().await.unwrap();
    rejected(
        supervisor
            .execute(stop_request(&service.key, generation))
            .await,
        Error::ShuttingDown,
    );
    assert!(fixture.owner.is_alive(&service.key, generation));
    assert!(fixture.owner.stops().is_empty());
    drop(supervisor);
    let recovered = fixture.open().await;
    assert_eq!(
        recovered.inspect(&service.key).unwrap().generation,
        Some(generation)
    );
    assert_eq!(fixture.owner.launches(), 1);
}

#[tokio::test]
async fn shutdown_only_stops_services_that_declare_it_and_targets_their_generations() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let mut stop_service = spec();
    stop_service.controller_exit = ControllerExit::StopServices;
    let mut keep_service = spec();
    keep_service.key.service_id = "keep-service".into();
    let stop_generation = start(&supervisor, &stop_service).await;
    let keep_generation = start(&supervisor, &keep_service).await;
    supervisor.shutdown().await.unwrap();
    let stops = fixture.owner.stops();
    assert_eq!(stops.len(), 1);
    assert_eq!(stops[0].key, stop_service.key);
    assert_eq!(stops[0].expected_generation, stop_generation);
    assert!(fixture.owner.is_alive(&keep_service.key, keep_generation));
}

#[tokio::test]
async fn events_have_stable_cursors_generation_identity_and_no_command_descriptions() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = spec();
    let first = start(&supervisor, &service).await;
    succeeded(
        supervisor
            .execute(restart_request(&service.key, first))
            .await,
    );
    let second = supervisor
        .inspect(&service.key)
        .unwrap()
        .generation
        .unwrap();
    succeeded(supervisor.execute(stop_request(&service.key, second)).await);
    let events = supervisor.events(0, 100).unwrap();
    assert!(events.len() >= 3);
    assert!(events
        .windows(2)
        .all(|pair| pair[0].sequence < pair[1].sequence));
    assert!(events
        .iter()
        .all(|event| event.generation == event.status.generation));
    assert!(events.iter().all(|event| event.key == service.key));
    let serialized = serde_json::to_string(&events).unwrap();
    assert!(!serialized.contains("/mock/service"));
    assert!(!serialized.contains("secret-ref-only"));
    assert!(!serialized.contains("ACCESS_TOKEN"));
    let cursor = events[0].sequence;
    let page = supervisor.events(cursor, 2).unwrap();
    assert_eq!(page, events[1..3]);
}

#[tokio::test]
async fn observations_with_a_foreign_owner_identity_cannot_authorize_adoption_or_stop() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = restartable_spec();
    let generation = start(&supervisor, &service).await;
    let mut foreign = supervisor.inspect(&service.key).unwrap().identity.unwrap();
    foreign.owner_instance = Uuid::new_v4();
    fixture.owner.state.lock().unwrap().observation_override =
        Some(Observation::Alive { identity: foreign });
    supervisor.reconcile_all().await.unwrap();
    assert_eq!(
        supervisor.inspect(&service.key).unwrap().lifecycle,
        Lifecycle::Unknown
    );
    uncertain(
        supervisor
            .execute(stop_request(&service.key, generation))
            .await,
    );
    uncertain(
        supervisor
            .execute(restart_request(&service.key, generation))
            .await,
    );
    assert!(fixture.owner.stops().is_empty());
    assert_eq!(fixture.owner.launches(), 1);
}

#[tokio::test]
async fn a_changed_kernel_process_identity_cannot_reuse_an_existing_generation() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = restartable_spec();
    let generation = start(&supervisor, &service).await;
    let recorded = supervisor.inspect(&service.key).unwrap().identity.unwrap();
    // Model PID reuse: the owner and numeric PID match, but start time differs.
    let mut reused_pid = recorded.clone();
    reused_pid.start_time_ticks += 1;
    fixture.owner.state.lock().unwrap().observation_override = Some(Observation::Alive {
        identity: reused_pid,
    });
    let recovered = fixture.recover(supervisor).await;
    let status = recovered.inspect(&service.key).unwrap();
    assert_eq!(status.lifecycle, Lifecycle::Unknown);
    assert_eq!(status.generation, Some(generation));
    assert_eq!(status.identity, Some(recorded));
    uncertain(
        recovered
            .execute(stop_request(&service.key, generation))
            .await,
    );
    uncertain(
        recovered
            .execute(restart_request(&service.key, generation))
            .await,
    );
    assert!(fixture.owner.stops().is_empty());
    assert_eq!(fixture.owner.launches(), 1);
}

#[tokio::test]
async fn different_service_ids_cannot_claim_the_same_live_tmux_endpoint() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let mut first = spec();
    first.command.executable = "/mock/tmux".into();
    first.command.args.clear();
    first.adapter = ServiceAdapter::Tmux {
        socket: "/private/shared.sock".into(),
    };
    let mut second = first.clone();
    second.key.service_id = "another-service".into();
    start(&supervisor, &first).await;
    let result = supervisor.execute(start_request(&second)).await;
    assert!(
        matches!(result, Err(Error::Conflict(_))),
        "two logical service IDs claimed one physical endpoint: {result:?}"
    );
    assert_eq!(fixture.owner.launches(), 1);
    assert_eq!(fixture.owner.launch_invocations(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_full_existing_control_socket_backlog_does_not_hang_daemon_binding() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let path = supervisor.state_dir().join("control.sock");
    let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
    // On Linux a zero backlog admits one pending connection. The following
    // connect fills it, so a second blocking connect would wait indefinitely.
    rustix::net::listen(&listener, 0).unwrap();
    let queued = std::os::unix::net::UnixStream::connect(&path).unwrap();
    let mut binding = tokio::task::spawn_blocking(move || {
        insula_supervisor::transport::ControlServer::bind(&supervisor).map(|_| ())
    });
    let response = tokio::time::timeout(Duration::from_millis(250), &mut binding).await;
    // Always unblock a buggy implementation before asserting, so this test
    // cannot strand a blocking thread or hang runtime shutdown on failure.
    drop(queued);
    drop(listener);
    match response {
        Ok(Ok(Err(Error::Conflict(_)))) => {}
        Ok(other) => panic!("binding must reject a live/uncertain endpoint: {other:?}"),
        Err(_) => {
            let _ = binding.await;
            panic!("control socket liveness check blocked on a full listen backlog");
        }
    }
}

#[tokio::test]
async fn an_alive_process_does_not_resolve_an_unknown_stop_in_the_same_controller_epoch() {
    let fixture = Fixture::new();
    let supervisor = fixture.open().await;
    let service = spec();
    let generation = start(&supervisor, &service).await;
    fixture.owner.state.lock().unwrap().ambiguous_stop_alive = true;
    let stopping = stop_request(&service.key, generation);
    uncertain(supervisor.execute(stopping.clone()).await);
    assert!(fixture.owner.is_alive(&service.key, generation));
    // Observing the process alive says nothing about whether its earlier stop
    // request is still executing. It must not reopen conflicting mutations.
    supervisor.reconcile_all().await.unwrap();
    assert_eq!(
        supervisor.inspect(&service.key).unwrap().pending_operation,
        Some(stopping.operation_id)
    );
    uncertain(supervisor.execute(start_request(&service)).await);
    uncertain(
        supervisor
            .execute(restart_request(&service.key, generation))
            .await,
    );
    assert_eq!(fixture.owner.launches(), 1);
    assert_eq!(fixture.owner.state.lock().unwrap().stop_invocations, 1);
}

#[path = "fixture_owner.rs"]
mod fixture_owner;

use fixture_owner::{foreground_spec, FixtureOwner, FixtureRpcServer};
use insula_supervisor::{
    owner_rpc::{
        read_frame, write_frame, OwnerReply, OwnerRequest, OwnerResponse, OwnerRpc, RpcRequest,
        RpcResponse, MAX_FRAME_BYTES, PROTOCOL_VERSION,
    },
    *,
};
use std::{sync::Arc, time::Duration};
use tempfile::TempDir;
use tokio::{io::AsyncWriteExt, net::UnixStream};
use uuid::Uuid;

fn client(server: &FixtureRpcServer, timeout_ms: u64) -> OwnerRpc {
    OwnerRpc::new(
        server.socket().into(),
        rustix::process::getuid().as_raw(),
        Duration::from_millis(timeout_ms),
    )
    .unwrap()
}

fn launch() -> LaunchRequest {
    LaunchRequest {
        operation_id: Uuid::new_v4(),
        generation: Uuid::new_v4(),
        spec: foreground_spec("/bin/sleep", &["30"]),
    }
}

fn stop(request: &LaunchRequest) -> StopRequest {
    StopRequest {
        operation_id: Uuid::new_v4(),
        key: request.spec.key.clone(),
        expected_generation: request.generation,
        policy: request.spec.stop.clone(),
    }
}

#[tokio::test]
async fn bounded_frame_rejects_oversize_before_payload_allocation() {
    let (mut writer, mut reader) = tokio::io::duplex(16);
    writer
        .write_u32((MAX_FRAME_BYTES + 1) as u32)
        .await
        .unwrap();
    let result = read_frame::<_, RpcRequest>(&mut reader).await;
    assert!(matches!(result, Err(Error::Invalid(_))));
}

#[tokio::test]
async fn rpc_authenticates_peer_uid_and_claim_is_exclusive() {
    let temp = TempDir::new().unwrap();
    let owner = FixtureOwner::new(&temp.path().join("authority")).unwrap();
    let server =
        FixtureRpcServer::start(Arc::clone(&owner), &temp.path().join("owner.sock")).unwrap();
    let wrong_uid = rustix::process::getuid().as_raw().wrapping_add(1);
    let untrusted =
        OwnerRpc::new(server.socket().into(), wrong_uid, Duration::from_secs(1)).unwrap();
    assert_eq!(
        untrusted.capabilities("root").await.unwrap_err(),
        Error::Ownership
    );
    let first = client(&server, 1000);
    let second = client(&server, 1000);
    let ownership = first.claim("root", Uuid::new_v4()).await.unwrap();
    assert_eq!(
        second.claim("root", Uuid::new_v4()).await.unwrap_err(),
        Error::Ownership
    );
    assert!(FixtureOwner::new(&temp.path().join("authority")).is_err());
    first.release(&ownership).await.unwrap();
    let newer = second.claim("root", Uuid::new_v4()).await.unwrap();
    assert!(newer.epoch > ownership.epoch);
    assert_eq!(
        owner.validate(&ownership).await.unwrap_err(),
        Error::Ownership
    );
    second.release(&newer).await.unwrap();
}

#[tokio::test]
async fn duplicate_launch_and_stale_stop_never_touch_replacement() {
    let temp = TempDir::new().unwrap();
    let owner = FixtureOwner::new(&temp.path().join("authority")).unwrap();
    let server =
        FixtureRpcServer::start(Arc::clone(&owner), &temp.path().join("owner.sock")).unwrap();
    let rpc = Arc::new(client(&server, 1000));
    let ownership = rpc.claim("root", Uuid::new_v4()).await.unwrap();
    let original = launch();
    let (a, b) = tokio::join!(
        rpc.launch(&ownership, original.clone()),
        rpc.launch(&ownership, original.clone())
    );
    assert_eq!(a.unwrap(), b.unwrap());
    assert_eq!(owner.total_launches(), 1);
    let old_stop = stop(&original);
    rpc.stop(&ownership, old_stop.clone()).await.unwrap();
    let replacement = launch();
    rpc.launch(&ownership, replacement.clone()).await.unwrap();
    // Replayed completed stop returns its historical result without signaling.
    let completed = rpc.stop(&ownership, old_stop).await.unwrap();
    assert!(
        matches!(completed, Observation::Gone { generation: Some(value), .. } if value == original.generation)
    );
    assert_eq!(
        rpc.stop(&ownership, stop(&original)).await.unwrap_err(),
        Error::StaleGeneration
    );
    assert!(matches!(
        rpc.observe(
            &ownership,
            &replacement.spec.key,
            Some(replacement.generation)
        )
        .await
        .unwrap(),
        Observation::Alive { .. }
    ));
    let mut changed = original.clone();
    changed.spec.revision = "changed".into();
    assert!(matches!(
        rpc.launch(&ownership, changed).await,
        Err(Error::Conflict(_))
    ));
    rpc.stop(&ownership, stop(&replacement)).await.unwrap();
    // Even after replacements have exited, the original launch is not replayed.
    rpc.launch(&ownership, original).await.unwrap();
    assert_eq!(owner.total_launches(), 2);
    rpc.release(&ownership).await.unwrap();
}

#[tokio::test]
async fn timeout_is_unknown_then_reconciles_one_actual_process() {
    let temp = TempDir::new().unwrap();
    let owner = FixtureOwner::new(&temp.path().join("authority")).unwrap();
    let server =
        FixtureRpcServer::start(Arc::clone(&owner), &temp.path().join("owner.sock")).unwrap();
    let rpc = client(&server, 50);
    let ownership = rpc.claim("root", Uuid::new_v4()).await.unwrap();
    let request = launch();
    owner.delay_next_launch(Duration::from_millis(100));
    assert_eq!(
        rpc.launch(&ownership, request.clone()).await.unwrap_err(),
        Error::Unknown
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    let observation = rpc
        .observe(&ownership, &request.spec.key, Some(request.generation))
        .await
        .unwrap();
    assert!(matches!(observation, Observation::Alive { .. }));
    rpc.launch(&ownership, request.clone()).await.unwrap();
    assert_eq!(owner.total_launches(), 1);
    rpc.stop(&ownership, stop(&request)).await.unwrap();
    rpc.release(&ownership).await.unwrap();
}

#[tokio::test]
async fn claim_disconnect_fences_a_delayed_launch_at_the_effect() {
    let temp = TempDir::new().unwrap();
    let owner = FixtureOwner::new(&temp.path().join("authority")).unwrap();
    let server =
        FixtureRpcServer::start(Arc::clone(&owner), &temp.path().join("owner.sock")).unwrap();
    let rpc = client(&server, 1000);
    let ownership = rpc.claim("root", Uuid::new_v4()).await.unwrap();
    let request = launch();
    owner.delay_next_launch(Duration::from_millis(200));
    let mut request_stream = UnixStream::connect(server.socket()).await.unwrap();
    let wire = RpcRequest {
        version: PROTOCOL_VERSION,
        request_id: Uuid::new_v4(),
        request: OwnerRequest::Launch {
            ownership: ownership.clone(),
            request: Box::new(request),
        },
    };
    write_frame(&mut request_stream, &wire).await.unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await;
    drop(rpc); // Closes its claim connection, not the independent pending request.
    let replacement = client(&server, 1000);
    let until = tokio::time::Instant::now() + Duration::from_secs(1);
    let newer = loop {
        match replacement.claim("root", Uuid::new_v4()).await {
            Ok(value) => break value,
            Err(Error::Ownership) if tokio::time::Instant::now() < until => {
                tokio::time::sleep(Duration::from_millis(5)).await
            }
            other => panic!("claim did not recover: {other:?}"),
        }
    };
    let response: RpcResponse = read_frame(&mut request_stream).await.unwrap();
    assert!(matches!(
        response.response,
        OwnerResponse::Error {
            error: Error::Ownership
        }
    ));
    assert_eq!(owner.total_launches(), 0);
    assert!(newer.epoch > ownership.epoch);
    replacement.release(&newer).await.unwrap();
}

#[tokio::test]
async fn a_hung_probe_uses_an_independent_connection_from_stop() {
    let temp = TempDir::new().unwrap();
    let owner = FixtureOwner::new(&temp.path().join("authority")).unwrap();
    let server =
        FixtureRpcServer::start(Arc::clone(&owner), &temp.path().join("owner.sock")).unwrap();
    let rpc = Arc::new(client(&server, 1000));
    let ownership = rpc.claim("root", Uuid::new_v4()).await.unwrap();
    let request = launch();
    rpc.launch(&ownership, request.clone()).await.unwrap();
    let mut probe = request.spec.health.clone();
    probe.timeout_ms = 400;
    probe.probe = ProbeKind::Command {
        command: foreground_spec("/bin/sleep", &["30"]).command,
    };
    let probe_request = ProbeRequest {
        key: request.spec.key.clone(),
        generation: request.generation,
        spec: probe,
    };
    let probe_rpc = Arc::clone(&rpc);
    let probe_ownership = ownership.clone();
    let task = tokio::spawn(async move { probe_rpc.probe(&probe_ownership, probe_request).await });
    tokio::time::sleep(Duration::from_millis(30)).await;
    let begin = tokio::time::Instant::now();
    rpc.stop(&ownership, stop(&request)).await.unwrap();
    assert!(
        begin.elapsed() < Duration::from_millis(200),
        "hung probe blocked a control connection"
    );
    let result = task.await.unwrap().unwrap();
    assert_ne!(result.class, ProbeClass::Success);
    rpc.release(&ownership).await.unwrap();
}

#[tokio::test]
async fn mutation_response_with_wrong_id_is_unknown_and_cannot_be_accepted() {
    let temp = TempDir::new().unwrap();
    let listener = tokio::net::UnixListener::bind(temp.path().join("spoof.sock")).unwrap();
    let rpc = OwnerRpc::new(
        temp.path().join("spoof.sock"),
        rustix::process::getuid().as_raw(),
        Duration::from_secs(1),
    )
    .unwrap();
    let task = tokio::spawn(async move {
        let (mut claim_stream, _) = listener.accept().await.unwrap();
        let claim: RpcRequest = read_frame(&mut claim_stream).await.unwrap();
        let ownership = match &claim.request {
            OwnerRequest::Claim {
                root_insula_id,
                controller_id,
            } => Ownership {
                root_insula_id: root_insula_id.clone(),
                owner_instance: Uuid::new_v4(),
                controller_id: *controller_id,
                epoch: 1,
            },
            _ => unreachable!(),
        };
        write_frame(
            &mut claim_stream,
            &RpcResponse::for_request(&claim, Ok(OwnerReply::Claimed(ownership))),
        )
        .await
        .unwrap();
        let (mut stream, _) = listener.accept().await.unwrap();
        let request: RpcRequest = read_frame(&mut stream).await.unwrap();
        let mut response =
            RpcResponse::for_request(&request, Ok(OwnerReply::Observation(Observation::Unknown)));
        response.request_id = Uuid::new_v4();
        write_frame(&mut stream, &response).await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
    });
    let ownership = rpc.claim("root", Uuid::new_v4()).await.unwrap();
    assert_eq!(
        rpc.launch(&ownership, launch()).await.unwrap_err(),
        Error::Unknown
    );
    task.await.unwrap();
}

#[tokio::test]
async fn owner_reload_never_turns_a_persisted_alive_receipt_into_a_live_handle() {
    let temp = TempDir::new().unwrap();
    let authority = temp.path().join("authority");
    let owner = FixtureOwner::new(&authority).unwrap();
    let ownership = owner.claim("root", Uuid::new_v4()).await.unwrap();
    let request = launch();
    owner.launch(&ownership, request.clone()).await.unwrap();
    let original_owner_id = ownership.owner_instance;
    // Fixture owner destruction closes its handles. Its durable receipt remains
    // Alive, so the replacement owner must refuse to infer live identity.
    drop(owner);
    // A scheduled observer may hold the fixture Arc briefly after the last
    // client drops. Wait for real owner-lock release instead of assuming an
    // incidental task scheduling order.
    let replacement = tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            match FixtureOwner::new(&authority) {
                Ok(owner) => break owner,
                Err(Error::Ownership) => tokio::time::sleep(Duration::from_millis(5)).await,
                Err(error) => panic!("replacement owner: {error}"),
            }
        }
    })
    .await
    .unwrap();
    let newer = replacement.claim("root", Uuid::new_v4()).await.unwrap();
    assert_eq!(newer.owner_instance, original_owner_id);
    assert!(newer.epoch > ownership.epoch);
    assert_eq!(
        replacement
            .observe(&newer, &request.spec.key, Some(request.generation))
            .await
            .unwrap(),
        Observation::Unknown
    );
    assert_eq!(
        replacement.launch(&newer, request).await.unwrap(),
        Observation::Unknown
    );
    assert_eq!(replacement.total_launches(), 0);
}

#[tokio::test]
async fn stop_cancellation_does_not_release_a_graceful_helper_before_it_finishes() {
    let temp = TempDir::new().unwrap();
    let owner = FixtureOwner::new(&temp.path().join("authority")).unwrap();
    let ownership = owner.claim("root", Uuid::new_v4()).await.unwrap();
    let mut request = launch();
    request.spec.stop.graceful = GracefulStop::Command {
        command: foreground_spec("/bin/sleep", &["0.15"]).command,
    };
    request.spec.stop.deadline_ms = 300;
    owner.launch(&ownership, request.clone()).await.unwrap();
    let waiting = tokio::time::timeout(
        Duration::from_millis(20),
        owner.stop(&ownership, stop(&request)),
    )
    .await;
    assert!(waiting.is_err(), "test should abandon its stop wait");
    // The owner-owned operation still holds its gate through the helper. A new
    // controller cannot be admitted while that helper could perform an effect.
    assert!(
        tokio::time::timeout(Duration::from_millis(20), owner.release(&ownership))
            .await
            .is_err()
    );
    owner.release(&ownership).await.unwrap();
    let newer = owner.claim("root", Uuid::new_v4()).await.unwrap();
    assert!(newer.epoch > ownership.epoch);
    owner.shutdown_all().await.unwrap();
    owner.release(&newer).await.unwrap();
}

#[tokio::test]
async fn a_forced_stop_deadline_from_a_fenced_epoch_cannot_kill_the_survivor() {
    let temp = TempDir::new().unwrap();
    let owner = FixtureOwner::new(&temp.path().join("authority")).unwrap();
    let ownership = owner.claim("root", Uuid::new_v4()).await.unwrap();
    let mut request = launch();
    request.spec = foreground_spec("/bin/sh", &["-c", "trap '' TERM; exec /bin/sleep 30"]);
    request.spec.stop.deadline_ms = 100;
    owner.launch(&ownership, request.clone()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(30)).await;
    let old_owner = Arc::clone(&owner);
    let old_token = ownership.clone();
    let stop_request = stop(&request);
    let old_stop = tokio::spawn(async move { old_owner.stop(&old_token, stop_request).await });
    tokio::time::sleep(Duration::from_millis(20)).await;
    owner.fence_controller().await;
    let newer = owner.claim("root", Uuid::new_v4()).await.unwrap();
    assert_eq!(old_stop.await.unwrap().unwrap_err(), Error::Ownership);
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert!(matches!(
        owner
            .observe(&newer, &request.spec.key, Some(request.generation))
            .await
            .unwrap(),
        Observation::Alive { .. }
    ));
    owner.shutdown_all().await.unwrap();
    owner.release(&newer).await.unwrap();
}

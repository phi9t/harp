// Included by intent_contracts: public HTTP behavior against the same real fixtures.
use super::*;

#[tokio::test]
async fn lifecycle_acceptance_and_amendment_serialize_with_public_grants_and_mutations() {
    let api = Api::new().await;
    let intent = api.research_intent("race-research", 0, "race-r1").await;
    // This fixture checks transaction ordering; measured evidence is exercised
    // independently by the full two-generation research lifecycle below.
    let evidence = api
        .upload("assembled-worker", b"ordering fixture research evidence")
        .await;
    let proposed=api.post("assembled-worker",&api.command("race-proposal",1,json!({"kind":"outcome.propose","intent":intent["intent"],"evidence":evidence,"negative":true,"evaluator_change":false}))).await;
    assert_eq!(proposed.0, 200);
    let verified=api.post("assembled-verifier",&api.command("race-proof",1,json!({"kind":"research.verify","intent":intent["intent"],"acceptance":intent["acceptance"],"proposal":proposed.1["proposal"],"policy":"suite-v2","environment":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}))).await;
    assert_eq!(verified.0, 200);
    let accept = api.command(
        "race-accept",
        2,
        json!({"kind":"outcome.accept","proof":verified.1["proof"]}),
    );
    let capture = json!({"kind":"capture_input","mission":api.fixture.mission,"id":Uuid::new_v4(),"attempt":api.fixture.attempt,"purpose":"implementation","limit":4});
    let (accepted, granted) = tokio::join!(
        api.post(&api.fixture.owner, &accept),
        api.mail("assembled-worker", "race-input", capture.clone())
    );
    assert_eq!(accepted.0, 200);
    assert!(granted.0 == 200 || granted.1["code"] == "dependency_unmet");
    assert_eq!(
        api.mail("assembled-worker", "race-input", capture).await.1["code"],
        "dependency_unmet"
    );
    let before = api.legacy_task(api.fixture.task).await;
    let mut snapshot = api.snapshot().await;
    snapshot.as_object_mut().unwrap().remove("snapshot_time");
    assert_eq!(api.post(&api.fixture.owner, &accept).await, accepted);
    let mut changed = accept.clone();
    changed["command"]["proof"] =
        json!("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    assert_eq!(
        api.post(&api.fixture.owner, &changed).await.1["code"],
        "idempotency_conflict"
    );
    let mut inspected = api.snapshot().await;
    inspected.as_object_mut().unwrap().remove("snapshot_time");
    assert_eq!(inspected, snapshot);
    let source=api.command("race-old-worker",3,json!({"kind":"source.propose","work":api.admission("x",0)["command"]["work"],"source":api.admission("x",0)["command"]["source"]}));
    let (amended, old_mutation) = tokio::join!(
        api.research_intent("race-amend", 3, "race-r2"),
        api.post("assembled-worker", &source)
    );
    assert!(old_mutation.0 == 200 || old_mutation.1["code"] == "stale_generation");
    assert_eq!(
        api.post("assembled-worker", &source).await.1["code"],
        "stale_generation"
    );
    let after = api.legacy_task(api.fixture.task).await;
    assert_eq!(after["data"]["generation"], 2);
    assert_eq!(after["data"]["attempts"], before["data"]["attempts"]);
    assert_eq!(
        after["data"]["reservations"],
        before["data"]["reservations"]
    );
    assert_eq!(
        api.research_intent("race-amend", 3, "race-r2").await,
        amended
    );
    let mut replayed_task = api.legacy_task(api.fixture.task).await;
    replayed_task
        .as_object_mut()
        .unwrap()
        .remove("snapshot_time");
    let mut stored_task = after.clone();
    stored_task.as_object_mut().unwrap().remove("snapshot_time");
    assert_eq!(replayed_task, stored_task);
    println!(
        "lifecycle_transaction_ordering {}",
        json!({"accepted":accepted,"concurrent_grant":granted,"amended":amended,"concurrent_old_mutation":old_mutation,"before":before,"after":after})
    );
    api.finish().await;
}

#[tokio::test]
async fn lifecycle_research_terminal_commands_and_current_projection() {
    let api = Api::new().await;
    let intent = api.research_intent("terminal", 0, "terminal-r1").await;
    let plan = json!({"schema_version":1,"interpreter_version":"declarative-v1","intent":intent["intent"],"acceptance":intent["acceptance"],"allowed_effects":["read"],"resources":{"attempts":8,"episodes":8,"repairs":3,"execution_seconds":30},"max_repairs":3,"steps":[{"id":"inspect","kind":"action","depends_on":[],"join":"all","effects":["read"],"max_iterations":1}]});
    let admitted_plan = api
        .post(
            &api.fixture.owner,
            &api.command("terminal-plan", 1, json!({"kind":"plan.admit","plan":plan})),
        )
        .await;
    assert_eq!(admitted_plan.0, 200);
    let input = Uuid::new_v4();
    let capture = json!({"kind":"capture_input","mission":api.fixture.mission,"id":input,"attempt":api.fixture.attempt,"purpose":"implementation","limit":4});
    assert_eq!(
        api.mail("assembled-worker", "terminal-input", capture.clone())
            .await
            .0,
        200
    );
    let packet = team_control_plane::mailbox::MailboxService::new(api.fixture.broker.clone())
        .record(
            "assembled-worker",
            team_control_plane::mailbox::RecordKind::Input,
            input,
        )
        .await
        .unwrap();
    let episode = Uuid::new_v4();
    let admission = json!({"kind":"admit_episode","mission":api.fixture.mission,"id":episode,"attempt":api.fixture.attempt,"purpose":"implementation","input_packet":input,"input_digest":packet["digest"]});
    assert_eq!(
        api.mail("assembled-worker", "terminal-episode", admission.clone())
            .await
            .0,
        200
    );
    let accepted = api.accept_research(&intent, 2, "terminal").await;
    let before = api.snapshot().await;
    let mut failures = Vec::new();
    let proof = before["accepted"][0]["proof_id"].clone();
    let receipt = api.command(
        "terminal-accept",
        3,
        json!({"kind":"outcome.accept","proof":proof}),
    );
    assert_eq!(
        api.post(&api.fixture.owner, &receipt).await,
        (200, accepted.clone())
    );
    let evidence = api
        .upload(
            "assembled-worker",
            b"physical log bytes remain storable after acceptance",
        )
        .await;
    let mutations=vec![
      ("assembled-worker".to_owned(),api.command("new-proposal",4,json!({"kind":"outcome.propose","intent":intent["intent"],"evidence":evidence,"negative":true,"evaluator_change":false}))),
      (api.fixture.owner.clone(),api.command("new-accept",4,receipt["command"].clone())),
      (api.fixture.owner.clone(),api.command("terminal-plan",1,json!({"kind":"plan.admit","plan":plan}))),
      (api.fixture.owner.clone(),api.command("terminal-revalidate",4,json!({"kind":"dependency.revalidate","intent":intent["intent"],"dependencies":[]}))),
    ];
    for (token, command) in mutations {
        let result = api.post(&token, &command).await;
        if result.0 != 409 {
            failures.push(json!({"command":command,"result":result}));
        }
    }
    for (key, command) in [("terminal-input", capture), ("terminal-episode", admission)] {
        let r = api.mail("assembled-worker", key, command).await;
        if r.0 != 409 {
            failures.push(json!({"key":key,"result":r}));
        }
    }
    for command in [
        json!({"kind":"prepare_delivery","id":Uuid::new_v4(),"message":Uuid::new_v4(),"episode":episode,"required_decision":null,"required_approval":null}),
        json!({"kind":"bind_runtime_turn","episode":episode,"start_operation":Uuid::new_v4(),"runtime_turn":"terminal-turn"}),
    ] {
        let r = api
            .mail(
                &api.fixture.owner,
                &Uuid::new_v4().to_string(),
                command.clone(),
            )
            .await;
        if r.1["code"] != "dependency_unmet" {
            failures.push(json!({"target_guard":command,"result":r}));
        }
    }
    let policy = team_control_plane::kernel::Kernel::new(api.fixture.broker.clone())
        .require_supervised_verification(&api.fixture.owner, api.fixture.task)
        .await;
    if policy.is_ok() {
        failures.push(json!("verification policy activation admitted"));
    }
    let task = api.legacy_task(api.fixture.task).await;
    let cancelled=api.v1("terminal-cancel",task["revision"].as_i64().unwrap(),None,json!({"kind":"cancel_attempt","task_id":api.fixture.task,"attempt_id":api.fixture.attempt,"reason":"must refuse accepted work"})).await;
    if cancelled.1["code"] != "dependency_unmet" {
        failures.push(json!({"cancel":cancelled}));
    }
    if before["current_accepted"]["subject"] != accepted["subject"]
        || before["fresh_grant_eligible"] != false
    {
        failures.push(json!({"projection":before}));
    }
    println!(
        "lifecycle_terminal_observations {}",
        json!({"failures":failures,"snapshot":before})
    );
    assert!(
        failures.is_empty(),
        "terminal lifecycle failures: {failures:?}"
    );
    api.finish().await;
}

#[tokio::test]
async fn lifecycle_research_amendment_fences_before_assignment_and_preserves_receipt() {
    let api = Api::new().await;
    let intent = api.research_intent("amend", 0, "amend-r1").await;
    let accepted = api.accept_research(&intent, 1, "amend").await;
    let old = api.snapshot().await;
    let before = api.legacy_task(api.fixture.task).await;
    let amended = api.research_intent("amend-next", 3, "amend-r2").await;
    let after = api.legacy_task(api.fixture.task).await;
    let source=api.command("fenced-worker",4,json!({"kind":"source.propose","work":api.admission("x",0)["command"]["work"],"source":api.admission("x",0)["command"]["source"]}));
    let worker = api.post("assembled-worker", &source).await;
    let verify=api.command("fenced-verifier",4,json!({"kind":"research.verify","intent":amended["intent"],"acceptance":amended["acceptance"],"proposal":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","policy":"suite-v2","environment":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}));
    let verifier = api.post("assembled-verifier", &verify).await;
    let receipt = api.command(
        "amend-accept",
        2,
        json!({"kind":"outcome.accept","proof":old["accepted"][0]["proof_id"]}),
    );
    let replay = api.post(&api.fixture.owner, &receipt).await;
    println!(
        "lifecycle_amendment_observations {}",
        json!({"before":before,"after":after,"worker":worker,"verifier":verifier,"replay":replay})
    );
    assert_eq!(
        after["data"]["generation"], 2,
        "accepted research immediately fences original generation"
    );
    assert_eq!(worker.1["code"], "stale_generation");
    assert_eq!(verifier.1["code"], "stale_generation");
    assert_eq!(replay, (200, accepted));
    assert_eq!(after["data"]["attempts"], before["data"]["attempts"]);
    assert_eq!(
        after["data"]["reservations"],
        before["data"]["reservations"]
    );
    api.finish().await;
}

#[tokio::test]
async fn lifecycle_research_wait_is_attention_and_historical_satisfaction_survives_amendment() {
    use team_control_plane::{mailbox, temporal};
    let api = Api::new().await;
    let intent = api.research_intent("wait-intent", 0, "wait-r1").await;
    let bridge = temporal::Bridge::new(api.fixture.broker.clone());
    let workflow = bridge
        .register(
            &api.fixture.owner,
            temporal::AggregateKind::Mission,
            api.fixture.mission,
            0,
        )
        .await
        .unwrap();
    let episode = api.fixture.lead_episode("waiting-lead").await;
    let yielded=api.mail("assembled-lead","wait-for-research",json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"Attention only; exact dependent authority is separate","artifacts":[]},"questions":[],"dependencies":[api.fixture.task],"executions":[]})).await;
    assert_eq!(yielded.0, 200, "{yielded:?}");
    let before = serde_json::to_value(
        bridge
            .progress(&api.fixture.owner, &workflow)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(before["mailbox_attention"][0]["satisfied"], false);
    api.accept_research(&intent, 1, "wait").await;
    let accepted = serde_json::to_value(
        bridge
            .progress(&api.fixture.owner, &workflow)
            .await
            .unwrap(),
    )
    .unwrap();
    api.research_intent("wait-amend", 3, "wait-r2").await;
    let after = serde_json::to_value(
        bridge
            .progress(&api.fixture.owner, &workflow)
            .await
            .unwrap(),
    )
    .unwrap();
    let record = mailbox::MailboxService::new(api.fixture.broker.clone())
        .record(&api.fixture.owner, mailbox::RecordKind::Episode, episode)
        .await
        .unwrap();
    println!(
        "lifecycle_wait_attention {}",
        json!({"before":before,"accepted":accepted,"after":after,"lead_episode":record})
    );
    assert_eq!(
        accepted["mailbox_attention"][0]["satisfied"], true,
        "research acceptance wakes bare task waits"
    );
    assert_eq!(
        after["mailbox_attention"][0]["satisfied"], true,
        "recorded satisfaction remains historical"
    );
    assert!(record["attempt_id"].is_null());
    api.finish().await;
}

#[tokio::test]
async fn lifecycle_research_artifact_and_cached_result_mutations_refuse() {
    let api = Api::new().await;
    let intent = api.research_intent("cached", 0, "cached-r1").await;
    let evidence = api
        .upload(
            "assembled-worker",
            b"retained investigation: no feasible candidate under admitted constraints",
        )
        .await;
    let proposal=api.command("cached-proposal",1,json!({"kind":"outcome.propose","intent":intent["intent"],"evidence":evidence,"negative":true,"evaluator_change":false}));
    let proposed = api.post("assembled-worker", &proposal).await;
    assert_eq!(proposed.0, 200);
    let proposed_id = proposed.1["proposal"].clone();
    let accepted = api.accept_research(&intent, 1, "cached").await;
    let old_proposal = api.post("assembled-worker", &proposal).await;
    let verify=api.command("cached-verify",1,json!({"kind":"research.verify","intent":intent["intent"],"acceptance":intent["acceptance"],"proposal":proposed_id,"policy":"suite-v2","environment":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}));
    let old_verify = api.post("assembled-verifier", &verify).await;
    let configure=api.artifact_request("configure-completed-research",json!({"kind":"configure","verifier":api.fixture.verifier,"lead_acceptance":true,"base":api.fixture.base,"suite":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","environment":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","mandatory_commands":[["true"]]})).await;
    let configured = api.artifact_response(&api.fixture.owner, &configure).await;
    println!(
        "lifecycle_cached_mutations {}",
        json!({"proposal":old_proposal,"verify":old_verify,"artifact":configured,"accepted":accepted})
    );
    assert_eq!(
        (old_proposal.0, old_verify.0, configured.0),
        (409, 409, 409)
    );
    api.finish().await;
}

async fn measured_research_round(
    api: &Api,
    intent: &Value,
    revision: i64,
    attempt: Uuid,
    worker: &str,
    verifier: &str,
    label: &str,
) -> Value {
    use team_control_plane::{mailbox, supervisor, temporal};
    let mailbox = mailbox::MailboxService::new(api.fixture.broker.clone());
    let supervisor = api.fixture.supervisor().await;
    temporal::Bridge::new(api.fixture.broker.clone())
        .register(
            &api.fixture.owner,
            temporal::AggregateKind::Task,
            api.fixture.task,
            0,
        )
        .await
        .unwrap();
    // Both roles have their own bounded input and Episode. The verifier reads
    // the admitted base independently, without any production candidate.
    let mut measured_evidence = None;
    for (token, purpose) in [(worker, "implementation"), (verifier, "verification")] {
        let input = Uuid::new_v4();
        let episode = Uuid::new_v4();
        let capture = json!({"kind":"capture_input","mission":api.fixture.mission,"id":input,"attempt":attempt,"purpose":purpose,"limit":4});
        assert_eq!(
            api.mail(token, &format!("{label}-{purpose}-input"), capture)
                .await
                .0,
            200
        );
        let packet = mailbox
            .record(token, mailbox::RecordKind::Input, input)
            .await
            .unwrap();
        let admission = json!({"kind":"admit_episode","mission":api.fixture.mission,"id":episode,"attempt":attempt,"purpose":purpose,"input_packet":input,"input_digest":packet["digest"]});
        if purpose == "verification" {
            assert_eq!(
                api.mail(worker, &format!("{label}-wrong-role"), admission.clone())
                    .await
                    .0,
                401
            );
        }
        let admitted = api
            .mail(
                token,
                &format!("{label}-{purpose}-episode"),
                admission.clone(),
            )
            .await;
        assert_eq!(
            admitted.0, 200,
            "exact research contract authorizes bounded {purpose} without production candidate: {admitted:?}"
        );
        assert_eq!(
            api.mail(token, &format!("{label}-{purpose}-episode"), admission)
                .await,
            admitted
        );
        let execution = supervisor
            .create_episode(token, episode, vec![], supervisor::CreateFault::None)
            .await
            .unwrap();
        let command = vec![
            "/bin/sh".to_owned(),
            "-c".into(),
            "test ! -e src/result && printf 'negative research: no result at admitted base\\n'"
                .into(),
        ];
        supervisor
            .start(token, execution.id, command, 10)
            .await
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let measured = loop {
            if let Some(result) = supervisor.result(token, execution.id).await.unwrap() {
                break result;
            }
            assert!(std::time::Instant::now() < deadline);
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        };
        assert_eq!(measured.exit_code, Some(0));
        assert!(measured.namespace_stopped);
        assert!(measured.candidate.is_none());
        let view = supervisor.inspect(token, execution.id).await.unwrap();
        let output = std::fs::read(&view.manifest.log_path).unwrap();
        assert_eq!(artifacts::sha256(&output), measured.logs);
        let bytes=serde_json::to_vec(&json!({"trusted_monitor_result":measured,"output":String::from_utf8(output).unwrap(),"intent":intent["intent"],"acceptance":intent["acceptance"]})).unwrap();
        let evidence = api.upload(worker, &bytes).await;
        supervisor
            .release(&api.fixture.owner, execution.id)
            .await
            .unwrap();
        let released = supervisor
            .inspect(&api.fixture.owner, execution.id)
            .await
            .unwrap();
        assert_eq!(released.state, supervisor::ExecutionState::Released);
        assert!(!released.manifest.path.exists());
        println!(
            "lifecycle_measured_research {}",
            json!({"round":label,"purpose":purpose,"execution":execution.id,"manifest":view.manifest,"measurement":measured,"artifact":evidence,"artifact_bytes":bytes,"released":released})
        );
        let yielded=api.mail(token,&format!("{label}-{purpose}-yield"),json!({"kind":"yield_episode","episode":episode,"checkpoint":{"id":Uuid::new_v4(),"summary":"Measured local investigation finished; no active effects","artifacts":[]},"questions":[],"dependencies":[],"executions":[]})).await;
        assert_eq!(
            yielded.0, 200,
            "released Episode may yield before acceptance: {yielded:?}"
        );
        if purpose == "verification" {
            measured_evidence = Some(evidence);
        }
    }
    let evidence = measured_evidence.unwrap();
    let proposal=api.command(&format!("{label}-proposal"),revision,json!({"kind":"outcome.propose","intent":intent["intent"],"evidence":evidence,"negative":true,"evaluator_change":false}));
    let proposed = api.post(worker, &proposal).await;
    assert_eq!(proposed.0, 200);
    let verification=api.command(&format!("{label}-proof"),revision,json!({"kind":"research.verify","intent":intent["intent"],"acceptance":intent["acceptance"],"proposal":proposed.1["proposal"],"policy":"suite-v2","environment":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}));
    let verified = api.post(verifier, &verification).await;
    assert_eq!(verified.0, 200);
    let accept = api.command(
        &format!("{label}-accept"),
        revision + 1,
        json!({"kind":"outcome.accept","proof":verified.1["proof"]}),
    );
    let accepted = api.post(&api.fixture.owner, &accept).await;
    assert_eq!(accepted.0, 200);
    assert_eq!(api.post(&api.fixture.owner, &accept).await, accepted);
    assert_eq!(
        api.post(worker, &proposal).await.1["code"],
        "dependency_unmet"
    );
    assert_eq!(
        api.post(verifier, &verification).await.1["code"],
        "dependency_unmet"
    );
    let service = artifacts::ArtifactService::open(
        api.fixture.broker.clone(),
        &api.fixture.root.join("artifacts"),
    )
    .await
    .unwrap();
    assert!(
        service
            .snapshot(&api.fixture.owner, api.fixture.task)
            .await
            .unwrap()
            .candidate
            .is_none()
    );
    assert!(api.snapshot().await["current_accepted"]["subject"] == accepted.1["subject"]);
    accepted.1
}

#[tokio::test]
async fn lifecycle_research_verifier_uses_exact_contract_without_production_candidate() {
    use team_control_plane::*;
    let api = Api::measured().await;
    let first = api
        .research_intent("measured-research", 0, "measured-r1")
        .await;
    let accepted = measured_research_round(
        &api,
        &first,
        1,
        api.fixture.attempt,
        "assembled-worker",
        "assembled-verifier",
        "first-research",
    )
    .await;
    let before = api.legacy_task(api.fixture.task).await;
    let budget = api.mission_accounting().await;
    let dependent = Uuid::new_v4();
    let dependent_worker = Uuid::new_v4();
    let spec = json!({"objective":"requires exact investigation","input":{"repository":api.fixture.root.join("store"),"commit":api.fixture.base,"path":"SPEC.md","digest":"881fa3537a1d388a9b61011411583564a434fdc7ad8b821de142d378e7bbead3"},"dependencies":[],"acceptance":["exact research subject"],"deliverables":["src/result"],"write_scope":["src"]});
    assert_eq!(api.v1("research-dependent-create",0,None,json!({"kind":"create_task","id":dependent,"mission_id":api.fixture.mission,"spec":spec})).await.0,200);
    let digest = api.legacy_task(dependent).await["data"]["spec_digest"].clone();
    assert_eq!(
        api.v1(
            "research-dependent-admit",
            1,
            None,
            json!({"kind":"admit_task","task_id":dependent,"spec_digest":digest})
        )
        .await
        .0,
        200
    );
    let mut dep = api.admission("research-dependent-intent", 0);
    dep["task"] = json!(dependent);
    dep["command"]["work"]["item"] = json!("research-dependent");
    dep["command"]["intent"]["legacy_spec_digest"] = digest;
    dep["command"]["intent"]["dependencies"] = json!([{"work":api.admission("x",0)["command"]["work"],"intent":first["intent"],"accepted_subject":accepted["subject"],"policy":"suite-v2"}]);
    let dep_admitted = api.post(&api.fixture.owner, &dep).await;
    assert_eq!(dep_admitted.0, 200);
    let amended = api
        .research_intent("measured-amendment", 3, "measured-r2")
        .await;
    let ready = api.legacy_task(api.fixture.task).await;
    assert_eq!(ready["data"]["generation"], 2);
    assert_eq!(ready["data"]["attempts"], before["data"]["attempts"]);
    assert_eq!(ready["data"]["operations"], before["data"]["operations"]);
    assert_eq!(
        ready["data"]["reservations"],
        before["data"]["reservations"]
    );
    assert_eq!(
        api.mission_accounting().await["admitted_attempts"],
        budget["admitted_attempts"]
    );
    for token in ["assembled-worker", "assembled-verifier"] {
        let stale=api.post(token,&api.command("stale-before-assignment",4,json!({"kind":"source.propose","work":api.admission("x",0)["command"]["work"],"source":api.admission("x",0)["command"]["source"]}))).await;
        assert_eq!(stale.1["code"], "stale_generation");
    }
    let old_accept = api.command(
        "first-research-accept",
        2,
        json!({"kind":"outcome.accept","proof":accepted["proof"]}),
    );
    assert_eq!(
        api.post(&api.fixture.owner, &old_accept).await,
        (200, accepted.clone())
    );
    assert_eq!(
        api.post(
            &api.fixture.owner,
            &api.command("old-proof-new-key", 4, old_accept["command"].clone())
        )
        .await
        .1["code"],
        "stale_revision"
    );
    assert_eq!(
        api.contract_read(dependent).await["dependencies"][0]["disposition"],
        "suspended"
    );
    for (principal, role, token) in [
        (api.fixture.worker, "worker", "fresh-research-worker"),
        (api.fixture.verifier, "verifier", "fresh-research-verifier"),
    ] {
        assert_eq!(api.v1(&format!("fresh-{role}"),0,None,json!({"kind":"grant_credential","credential_id":Uuid::new_v4(),"principal_id":principal,"role":role,"mission_id":api.fixture.mission,"task_id":api.fixture.task,"generation":3,"token_hash":token_hash(token),"expires_in_seconds":300})).await.0,200);
    }
    let assigned=api.v1("fresh-research-assignment",ready["revision"].as_i64().unwrap(),Some(2),json!({"kind":"assign_attempt","task_id":api.fixture.task,"worker_id":api.fixture.worker})).await;
    assert_eq!(assigned.0, 200);
    assert_eq!(assigned.1["generation"], 3);
    let attempt: Uuid = serde_json::from_value(assigned.1["attempt_id"].clone()).unwrap();
    assert_ne!(attempt, api.fixture.attempt);
    let fresh = measured_research_round(
        &api,
        &amended,
        4,
        attempt,
        "fresh-research-worker",
        "fresh-research-verifier",
        "second-research",
    )
    .await;
    assert_ne!(fresh["subject"], accepted["subject"]);
    assert_ne!(fresh["proof"], accepted["proof"]);
    let blocked = api
        .v1(
            "research-dependent-still-blocked",
            2,
            None,
            json!({"kind":"assign_attempt","task_id":dependent,"worker_id":dependent_worker}),
        )
        .await;
    assert_eq!(blocked.1["code"], "dependency_unmet");
    let mut revalidate=api.command("research-dependent-revalidation",2,json!({"kind":"dependency.revalidate","intent":dep_admitted.1["intent"],"dependencies":[{"work":api.admission("x",0)["command"]["work"],"intent":amended["intent"],"accepted_subject":fresh["subject"],"policy":"suite-v2"}]}));
    revalidate["task"] = json!(dependent);
    let revalidated = api.post(&api.fixture.owner, &revalidate).await;
    assert_eq!(revalidated.0, 200);
    assert_eq!(api.post(&api.fixture.owner, &revalidate).await, revalidated);
    assert_eq!(
        api.contract_read(dependent).await["fresh_grant_eligible"],
        true
    );
    assert_eq!(
        api.snapshot().await["accepted"].as_array().unwrap().len(),
        2
    );
    println!(
        "lifecycle_research_completed {}",
        json!({"old":accepted,"fresh":fresh,"before":before,"amended":ready,"final":api.legacy_task(api.fixture.task).await,"budget_before":budget,"budget_after":api.mission_accounting().await,"dependent":api.contract_read(dependent).await})
    );
    api.finish().await;
}

// Same file-backed, zero-model physical runtime used by mailbox_delivery.rs.
struct DeliveryRuntime {
    root: std::path::PathBuf,
    identity: team_control_plane::gateway::RuntimeIdentity,
}
impl team_control_plane::gateway::RuntimeAdapter for DeliveryRuntime {
    fn identity(&self) -> team_control_plane::gateway::RuntimeIdentity {
        self.identity.clone()
    }
    fn inspect(&self) -> team_control_plane::gateway::CapabilityProfile {
        team_control_plane::gateway::CapabilityProfile {
            start: true,
            steer: true,
            observe: true,
            authoritative_effect_inventory: true,
            ..Default::default()
        }
    }
    fn execute(
        &mut self,
        intent: &team_control_plane::gateway::Intent,
        _thread: Option<&str>,
    ) -> team_control_plane::gateway::RuntimeReply {
        let output = std::process::Command::new("python3")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/mailbox_runtime.py"
            ))
            .arg(&self.root)
            .arg("start")
            .arg(json!({"intent":intent}).to_string())
            .output()
            .unwrap();
        assert!(output.status.success());
        let reply: Value = serde_json::from_slice(&output.stdout).unwrap();
        team_control_plane::gateway::RuntimeReply::Confirmed {
            thread_id: Some(reply["thread_id"].as_str().unwrap().into()),
            effect_id: reply["effect_id"].as_str().unwrap().into(),
        }
    }
    fn observe(
        &mut self,
        _: &team_control_plane::gateway::Intent,
        _: Option<&str>,
    ) -> team_control_plane::gateway::Observation {
        team_control_plane::gateway::Observation::Inconclusive(
            "No model-usage inference from native turn state".into(),
        )
    }
    fn confirmed_start_turn(
        &self,
        intent: &team_control_plane::gateway::Intent,
        reply: &team_control_plane::gateway::RuntimeReply,
    ) -> Option<String> {
        if intent.kind == team_control_plane::gateway::OperationKind::Start
            && let team_control_plane::gateway::RuntimeReply::Confirmed { effect_id, .. } = reply
        {
            let state: Value =
                serde_json::from_slice(&std::fs::read(self.root.join("turn.json")).unwrap())
                    .unwrap();
            assert_eq!(state["turn"], *effect_id);
            Some(effect_id.clone())
        } else {
            None
        }
    }
}
#[tokio::test]
async fn lifecycle_owner_cached_delivery_and_runtime_binding_target_completed_research() {
    use team_control_plane::{gateway::*, mailbox, temporal};
    let api = Api::new().await;
    let intent = api
        .research_intent("owner-target", 0, "owner-target-r1")
        .await;
    let input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    assert_eq!(api.mail("assembled-worker","owner-target-input",json!({"kind":"capture_input","mission":api.fixture.mission,"id":input,"attempt":api.fixture.attempt,"purpose":"implementation","limit":4})).await.0,200);
    let packet = mailbox::MailboxService::new(api.fixture.broker.clone())
        .record("assembled-worker", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    let gateway = Gateway::new(api.fixture.broker.clone());
    let binding = Uuid::new_v4();
    let start = Uuid::new_v4();
    let root = api.fixture.root.join("owner-target-native");
    std::fs::create_dir(&root).unwrap();
    let identity = RuntimeIdentity {
        runtime_kind: "conditional-subprocess-fault".into(),
        runtime_version: "v1".into(),
        deployment: "owned-fixture".into(),
        thread_store: root.to_str().unwrap().into(),
        endpoint: "local-subprocess".into(),
        environment: "disposable".into(),
    };
    let mut runtime = DeliveryRuntime {
        root: root.clone(),
        identity: identity.clone(),
    };
    gateway
        .bind(
            &api.fixture.owner,
            binding,
            api.fixture.admission,
            &identity,
            Some("owned-thread"),
            1,
        )
        .await
        .unwrap();
    let start_intent = Intent {
        id: start,
        admission_id: api.fixture.admission,
        binding_id: binding,
        generation: 1,
        kind: OperationKind::Start,
        arguments: json!({"input_digest":packet["digest"],"input_packet":input,"input":packet["packet"]}),
    };
    gateway
        .prepare(&api.fixture.owner, start_intent.clone())
        .await
        .unwrap();
    let started = gateway
        .dispatch(&api.fixture.owner, start, &mut runtime)
        .await
        .unwrap();
    assert_eq!(started.state, OperationState::Confirmed);
    let state: Value =
        serde_json::from_slice(&std::fs::read(root.join("turn.json")).unwrap()).unwrap();
    temporal::Bridge::new(api.fixture.broker.clone())
        .register(
            &api.fixture.owner,
            temporal::AggregateKind::Task,
            api.fixture.task,
            0,
        )
        .await
        .unwrap();
    assert_eq!(api.mail("assembled-worker","owner-target-episode",json!({"kind":"admit_episode","mission":api.fixture.mission,"id":episode,"attempt":api.fixture.attempt,"purpose":"implementation","input_packet":input,"input_digest":packet["digest"]})).await.0,200);
    let bind = json!({"kind":"bind_runtime_turn","episode":episode,"start_operation":start,"runtime_turn":state["turn"]});
    assert_eq!(
        api.mail(&api.fixture.owner, "owner-target-bind", bind.clone())
            .await
            .0,
        200
    );
    let message = Uuid::new_v4();
    assert_eq!(api.mail(&api.fixture.owner,"owner-target-message",json!({"kind":"send","mission":api.fixture.mission,"id":message,"recipients":[api.fixture.worker],"message_kind":"notice","text":"Exact admitted constraint","reply_to":null,"attention":"normal","artifacts":[]})).await.0,200);
    let delivery = json!({"kind":"prepare_delivery","id":Uuid::new_v4(),"message":message,"episode":episode,"required_decision":null,"required_approval":null});
    assert_eq!(
        api.mail(
            &api.fixture.owner,
            "owner-target-delivery",
            delivery.clone()
        )
        .await
        .0,
        200
    );
    api.accept_research(&intent, 1, "owner-target").await;
    let bound = api
        .mail(&api.fixture.owner, "owner-target-bind", bind)
        .await;
    let delivered = api
        .mail(&api.fixture.owner, "owner-target-delivery", delivery)
        .await;
    let cache = gateway.prepare(&api.fixture.owner, start_intent).await;
    let retained = gateway.read(&api.fixture.owner, start).await.unwrap();
    let complete = std::process::Command::new("python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/mailbox_runtime.py"
        ))
        .arg(&root)
        .arg("complete")
        .output()
        .unwrap();
    assert!(complete.status.success());
    let stopped: Value =
        serde_json::from_slice(&std::fs::read(root.join("turn.json")).unwrap()).unwrap();
    assert!(stopped["turn"].is_null());
    println!(
        "lifecycle_owner_targets {}",
        json!({"bind":bound,"delivery":delivered,"cache":cache,"retained":retained,"native_turn_stopped":stopped,"root":root,"unknown_usage_retained":true})
    );
    assert_eq!(
        (bound.1["code"].clone(), delivered.1["code"].clone()),
        (json!("dependency_unmet"), json!("dependency_unmet"))
    );
    assert_eq!(
        cache.unwrap_err().code,
        team_control_plane::ErrorCode::DependencyUnmet
    );
    api.finish().await;
}

#[tokio::test]
async fn lifecycle_research_verifier_must_be_independent_of_attempt_worker() {
    use team_control_plane::{mailbox, token_hash};
    let api = Api::measured().await;
    let mut request = api.admission("nonindependent-research", 0);
    request["command"]["intent"]["acceptance"]["outcome"] = json!("investigation_complete");
    request["command"]["intent"]["acceptance"]["allow_negative"] = json!(true);
    request["command"]["intent"]["acceptance"]["verifier"] = json!(api.fixture.worker);
    assert_eq!(api.post(&api.fixture.owner, &request).await.0, 200);
    assert_eq!(api.v1("same-principal-verifier",0,None,json!({"kind":"grant_credential","credential_id":Uuid::new_v4(),"principal_id":api.fixture.worker,"role":"verifier","mission_id":api.fixture.mission,"task_id":api.fixture.task,"generation":1,"token_hash":token_hash("same-principal-verifier"),"expires_in_seconds":300})).await.0,200);
    let input = Uuid::new_v4();
    assert_eq!(api.mail("same-principal-verifier","nonindependent-input",json!({"kind":"capture_input","mission":api.fixture.mission,"id":input,"attempt":api.fixture.attempt,"purpose":"verification","limit":4})).await.0,200);
    let packet = mailbox::MailboxService::new(api.fixture.broker.clone())
        .record("same-principal-verifier", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    let admitted=api.mail("same-principal-verifier","nonindependent-episode",json!({"kind":"admit_episode","mission":api.fixture.mission,"id":Uuid::new_v4(),"attempt":api.fixture.attempt,"purpose":"verification","input_packet":input,"input_digest":packet["digest"]})).await;
    assert_eq!(
        admitted.0, 401,
        "a second role credential cannot make the Attempt worker independent: {admitted:?}"
    );
    api.finish().await;
}

#[tokio::test]
async fn lifecycle_research_source_amendment_refuses_old_input_and_old_prepared_execution() {
    use team_control_plane::{mailbox, supervisor};
    let api = Api::measured().await;
    api.research_intent("research-source", 0, "source-r1").await;
    let input = Uuid::new_v4();
    assert_eq!(api.mail("assembled-verifier","before-source-input",json!({"kind":"capture_input","mission":api.fixture.mission,"id":input,"attempt":api.fixture.attempt,"purpose":"verification","limit":4})).await.0,200);
    let mailbox = mailbox::MailboxService::new(api.fixture.broker.clone());
    let packet = mailbox
        .record("assembled-verifier", mailbox::RecordKind::Input, input)
        .await
        .unwrap();
    api.research_intent("research-source-amend", 1, "source-r2")
        .await;
    let stale=api.mail("assembled-verifier","stale-source-input",json!({"kind":"admit_episode","mission":api.fixture.mission,"id":Uuid::new_v4(),"attempt":api.fixture.attempt,"purpose":"verification","input_packet":input,"input_digest":packet["digest"]})).await;
    assert_eq!(stale.1["code"], "stale_revision");
    let fresh_input = Uuid::new_v4();
    let episode = Uuid::new_v4();
    assert_eq!(api.mail("assembled-verifier","after-source-input",json!({"kind":"capture_input","mission":api.fixture.mission,"id":fresh_input,"attempt":api.fixture.attempt,"purpose":"verification","limit":4})).await.0,200);
    let fresh = mailbox
        .record(
            "assembled-verifier",
            mailbox::RecordKind::Input,
            fresh_input,
        )
        .await
        .unwrap();
    assert_eq!(api.mail("assembled-verifier","fresh-source-episode",json!({"kind":"admit_episode","mission":api.fixture.mission,"id":episode,"attempt":api.fixture.attempt,"purpose":"verification","input_packet":fresh_input,"input_digest":fresh["digest"]})).await.0,200);
    let supervisor = api.fixture.supervisor().await;
    let execution = supervisor
        .create_episode(
            "assembled-verifier",
            episode,
            vec![],
            supervisor::CreateFault::None,
        )
        .await
        .unwrap();
    api.research_intent("research-source-amend-again", 2, "source-r3")
        .await;
    let start = supervisor
        .start("assembled-verifier", execution.id, vec!["true".into()], 10)
        .await;
    assert_eq!(
        start.unwrap_err().code,
        team_control_plane::ErrorCode::StaleGeneration
    );
    assert_eq!(
        api.legacy_task(api.fixture.task).await["data"]["generation"],
        1,
        "unaccepted source changes retain original Attempt attribution"
    );
    supervisor
        .cancel(&api.fixture.owner, execution.id)
        .await
        .unwrap();
    supervisor
        .release(&api.fixture.owner, execution.id)
        .await
        .unwrap();
    println!(
        "lifecycle_source_only_cleanup {}",
        json!({"execution":execution.id,"released":supervisor.inspect(&api.fixture.owner,execution.id).await.unwrap(),"no_dispatch":true})
    );
    api.finish().await;
}

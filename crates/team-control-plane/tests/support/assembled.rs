use std::path::{Path, PathBuf};
use team_control_plane::*;
use uuid::Uuid;
pub fn git(root: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["-c", "core.hooksPath=/dev/null", "-C"])
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "owned fixture Git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
pub fn envelope(epoch: Uuid, key: &str, revision: i64, command: Command) -> CommandEnvelope {
    CommandEnvelope {
        schema_version: 1,
        key: key.into(),
        expected_revision: revision,
        expected_generation: None,
        recovery_epoch: epoch,
        command,
    }
}
pub fn mail(epoch: Uuid, key: &str, command: serde_json::Value) -> mailbox::Envelope {
    serde_json::from_value(
        serde_json::json!({"schema_version":1,"key":key,"recovery_epoch":epoch,"command":command}),
    )
    .unwrap()
}
pub struct Fixture {
    pub broker: Broker,
    pub owner: String,
    pub dsn: String,
    pub epoch: Uuid,
    pub root: PathBuf,
    pub mission: Uuid,
    pub task: Uuid,
    pub worker: Uuid,
    pub verifier: Uuid,
    pub lead: Uuid,
    pub attempt: Uuid,
    pub admission: Uuid,
    pub base: String,
    pub spec: String,
    admin: tokio_postgres::Client,
    schema: String,
}
impl Fixture {
    pub async fn new() -> Self {
        Self::with_requirement(true).await
    }
    pub async fn without_requirement() -> Self {
        Self::with_requirement(false).await
    }
    async fn with_requirement(requirement: bool) -> Self {
        let base_dsn =
            std::env::var("TEAM_CONTROL_PLANE_TEST_DSN").expect("real PostgreSQL required");
        let (admin, connection) = tokio_postgres::connect(&base_dsn, tokio_postgres::NoTls)
            .await
            .unwrap();
        tokio::spawn(async move { connection.await.unwrap() });
        let schema = format!("assembled_vertical_{}", Uuid::new_v4().simple());
        admin
            .batch_execute(&format!("CREATE SCHEMA {schema}"))
            .await
            .unwrap();
        let dsn = format!("{base_dsn} options='-c search_path={schema}'");
        migrate(&dsn).await.unwrap();
        let epoch = Uuid::new_v4();
        let broker = Broker::connect(&dsn, epoch).await.unwrap();
        let owner = broker.bootstrap_owner().await.unwrap();
        let root = std::env::temp_dir().join(format!("kernel16-{}", Uuid::new_v4()));
        let store = root.join("store");
        std::fs::create_dir_all(store.join("src")).unwrap();
        std::fs::write(store.join("SPEC.md"), b"produce exact src/result").unwrap();
        std::fs::write(store.join("src/base"), b"immutable base\n").unwrap();
        git(&store, &["init", "-q"]);
        git(&store, &["add", "."]);
        git(
            &store,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "-qm",
                "base",
            ],
        );
        let base = git(&store, &["rev-parse", "HEAD"]);
        let mission = Uuid::new_v4();
        let task = Uuid::new_v4();
        let worker = Uuid::new_v4();
        let verifier = Uuid::new_v4();
        let lead = Uuid::new_v4();
        let input = GitInput {
            repository: store.to_str().unwrap().into(),
            commit: base.clone(),
            path: "SPEC.md".into(),
            digest: artifacts::sha256(b"produce exact src/result"),
        };
        broker
            .execute(
                &owner,
                envelope(
                    epoch,
                    "mission",
                    0,
                    Command::CreateMission {
                        id: mission,
                        objective: "complete durable kernel proof".into(),
                        input: input.clone(),
                        policy: Policy {
                            capacity: 2,
                            max_attempts: 8,
                            max_episodes: 8,
                            max_repairs: 3,
                            episode_deadline_seconds: 30,
                        },
                    },
                ),
            )
            .await
            .unwrap();
        broker
            .execute(
                &owner,
                envelope(
                    epoch,
                    "task",
                    0,
                    Command::CreateTask {
                        id: task,
                        mission_id: mission,
                        spec: TaskSpec {
                            dependencies: vec![],
                            objective: "produce immutable src/result".into(),
                            input,
                            acceptance: vec!["exact executed independent gate".into()],
                            deliverables: vec!["src/result".into()],
                            write_scope: vec!["src".into()],
                        },
                    },
                ),
            )
            .await
            .unwrap();
        let spec = broker
            .snapshot(&owner, SnapshotQuery::Task(task))
            .await
            .unwrap()
            .data["spec_digest"]
            .as_str()
            .unwrap()
            .to_owned();
        broker
            .execute(
                &owner,
                envelope(
                    epoch,
                    "admit",
                    1,
                    Command::AdmitTask {
                        task_id: task,
                        spec_digest: spec.clone(),
                    },
                ),
            )
            .await
            .unwrap();
        for (principal, role, token, task_id, generation) in [
            (
                worker,
                Role::Worker,
                "assembled-worker",
                Some(task),
                Some(1),
            ),
            (
                verifier,
                Role::Verifier,
                "assembled-verifier",
                Some(task),
                Some(1),
            ),
            (lead, Role::Lead, "assembled-lead", None, None),
        ] {
            broker
                .execute(
                    &owner,
                    envelope(
                        epoch,
                        &format!("grant-{principal}"),
                        0,
                        Command::GrantCredential {
                            credential_id: Uuid::new_v4(),
                            principal_id: principal,
                            role,
                            mission_id: Some(mission),
                            task_id,
                            generation,
                            token_hash: token_hash(token),
                            expires_in_seconds: 300,
                        },
                    ),
                )
                .await
                .unwrap();
        }
        let assigned = broker
            .execute(
                &owner,
                envelope(
                    epoch,
                    "assign",
                    2,
                    Command::AssignAttempt {
                        task_id: task,
                        worker_id: worker,
                    },
                ),
            )
            .await
            .unwrap();
        if requirement {
            kernel::Kernel::new(broker.clone())
                .require_supervised_verification(&owner, task)
                .await
                .unwrap();
        }
        Self {
            broker,
            owner,
            dsn,
            epoch,
            root,
            mission,
            task,
            worker,
            verifier,
            lead,
            attempt: assigned.attempt_id.unwrap(),
            admission: assigned.operation_id.unwrap(),
            base,
            spec,
            admin,
            schema,
        }
    }
    pub async fn worker_episode(&self, key: &str) -> Uuid {
        let mailbox = mailbox::MailboxService::new(self.broker.clone());
        let input = Uuid::new_v4();
        let episode = Uuid::new_v4();
        mailbox.execute("assembled-worker",mail(self.epoch,&format!("{key}-capture"),serde_json::json!({"kind":"capture_input","mission":self.mission,"id":input,"attempt":self.attempt,"purpose":"implementation","limit":4}))).await.unwrap();
        let packet = mailbox
            .record("assembled-worker", mailbox::RecordKind::Input, input)
            .await
            .unwrap();
        mailbox.execute("assembled-worker",mail(self.epoch,&format!("{key}-admit"),serde_json::json!({"kind":"admit_episode","mission":self.mission,"id":episode,"attempt":self.attempt,"purpose":"implementation","input_packet":input,"input_digest":packet["digest"]}))).await.unwrap();
        episode
    }
    pub async fn supervisor(&self) -> supervisor::Supervisor {
        supervisor::Supervisor::open(
            self.broker.clone(),
            &self.root.join("workspaces"),
            env!("CARGO_BIN_EXE_team-control-plane"),
        )
        .await
        .unwrap()
    }
    pub async fn finish(self) {
        // Callers must first prove every actually launched execution stopped/released.
        std::fs::remove_dir_all(&self.root).unwrap();
        self.admin
            .batch_execute(&format!("DROP SCHEMA {} CASCADE", self.schema))
            .await
            .unwrap();
    }
}

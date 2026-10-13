#![cfg(feature = "fault-injection")]
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

const TOKEN: &str = "operator-runtime-secret";
const SUPERVISOR_TOKEN: &str = "supervisor-runtime-secret";

struct Process(Child);
impl Process {
    fn kill(&mut self) {
        self.0.kill().unwrap();
        self.0.wait().unwrap();
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        if self.0.try_wait().unwrap().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
fn wait_for(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(8);
    while !condition() {
        assert!(Instant::now() < deadline, "behavior deadline exceeded");
        std::thread::sleep(Duration::from_millis(20));
    }
}
fn request(socket: &Path, value: Value) -> Value {
    let mut stream = UnixStream::connect(socket).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    writeln!(stream, "{value}").unwrap();
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}
struct Fixture {
    dir: tempfile::TempDir,
    api: PathBuf,
    supervisor_socket: PathBuf,
    supervisor: Process,
    controller: Process,
    _network: TcpListener,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let api = dir.path().join("api.sock");
        let supervisor_socket = dir.path().join("supervisor.sock");
        let root = dir.path().join("executions");
        fs::create_dir(&root).unwrap();
        let network = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = network.local_addr().unwrap().port();
        let script = format!(
            r#"
import os, pathlib, socket, subprocess, time, json
w=pathlib.Path('/workspace')
blocked=[{host:?}, {api:?}, {supervisor:?}, '/root/.ssh', '/home/philip.yang', '/workspace/.git']
assert all(not pathlib.Path(p).exists() for p in blocked), blocked
assert 'SSH_AUTH_SOCK' not in os.environ and 'CORENIUS_OPERATOR_TOKEN' not in os.environ and 'CORENIUS_SUPERVISOR_TOKEN' not in os.environ
assert os.getpid()==1
assert len(list(pathlib.Path('/proc').glob('[0-9]*'))) < 8
s=socket.socket(); s.settimeout(.2)
assert s.connect_ex(('127.0.0.1', {port})) != 0
desc=subprocess.Popen(['/usr/bin/python3','-c',"import pathlib,time; p=pathlib.Path('/scratch/descendant'); p.write_text('alive'); time.sleep(120)", {host:?}],start_new_session=True)
(w/'identity.json').write_text(json.dumps({{'workspace_inode':os.stat('/workspace').st_ino,'pid':os.getpid(),'descendant':desc.pid,'home':os.environ['HOME'],'network':'blocked','forbidden':blocked}}))
while True:
 probe=w/'cross-probe.json'
 if probe.exists():
  target=json.loads(probe.read_text())
  assert not pathlib.Path('/proc/'+str(target['pid'])).exists()
  try: os.kill(target['pid'],0); raise AssertionError('cross attempt signalling allowed')
  except ProcessLookupError: pass
  assert not pathlib.Path(target['workspace']).exists()
  (w/'cross-probe-passed').write_text('filesystem/process/signalling blocked')
 (w/'heartbeat').write_text(str(time.monotonic_ns())); time.sleep(.05)
"#,
            host = dir.path().display().to_string(),
            api = api.display().to_string(),
            supervisor = supervisor_socket.display().to_string(),
            port = port
        );
        let config = dir.path().join("supervisor.json");
        fs::write(
            &config,
            json!({"root":root,"profile":{"executable":"/usr/bin/python3","args":["-c",script]}})
                .to_string(),
        )
        .unwrap();
        let executable = Path::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
            .parent()
            .unwrap()
            .join("corenius-attempt-supervisor");
        let supervisor = Process(
            Command::new(executable)
                .args([&config, &supervisor_socket])
                .env("CORENIUS_SUPERVISOR_TOKEN", SUPERVISOR_TOKEN)
                .stdout(Stdio::null())
                .spawn()
                .expect("public independent supervisor executable must exist"),
        );
        wait_for(|| UnixStream::connect(&supervisor_socket).is_ok());
        let controller = Self::controller(dir.path(), &api, &supervisor_socket);
        Self {
            dir,
            api,
            supervisor_socket,
            supervisor,
            controller,
            _network: network,
        }
    }
    fn controller(root: &Path, api: &Path, supervisor: &Path) -> Process {
        let mut process = Process(
            Command::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
                .args([&root.join("state.db"), api])
                .env("CORENIUS_OPERATOR_TOKEN", TOKEN)
                .env("CORENIUS_SUPERVISOR_SOCKET", supervisor)
                .env("CORENIUS_SUPERVISOR_TOKEN", SUPERVISOR_TOKEN)
                .env("SSH_AUTH_SOCK", "/forbidden/socket")
                .env("CORENIUS_TEST_MANUAL_RUNTIME", "1")
                .stdout(Stdio::null())
                .spawn()
                .unwrap(),
        );
        wait_for(|| {
            assert!(
                process.0.try_wait().unwrap().is_none(),
                "controller startup"
            );
            UnixStream::connect(api).is_ok()
        });
        process
    }
    fn raw(&self, value: Value) -> Value {
        request(&self.api, value)
    }
    fn query(&self, kind: &str, args: Value) -> Value {
        let r=self.raw(json!({"auth":{"principal":"operator","token":TOKEN},"project":"p","query":{"kind":kind,"args":args}}));
        assert!(r["error"].is_null(), "{r}");
        r["ok"].clone()
    }
    fn command(&self, kind: &str, args: Value) -> Value {
        let revision = if kind == "create_project" {
            0
        } else {
            self.snapshot()["revision"].as_u64().unwrap()
        };
        let r=self.raw(json!({"auth":{"principal":"operator","token":TOKEN},"project":"p","command":{"kind":kind,"args":args,"expected_revision":revision,"idempotency_key":uuid::Uuid::new_v4().to_string()}}));
        assert!(r["error"].is_null(), "{r}");
        r["ok"]["result"].clone()
    }
    fn snapshot(&self) -> Value {
        self.query("snapshot", json!({}))
    }
    fn reconcile(&self) -> Value {
        let r=self.raw(json!({"auth":{"principal":"operator","token":TOKEN},"project":"p","runtime":{"kind":"reconcile"}}));
        assert!(r["error"].is_null(), "{r}");
        r["ok"].clone()
    }
    fn admit(&self, task: &str) -> Value {
        self.command("create_task",json!({"task":task,"work_ref":{"deployment":"local","project":"p","item":task},"source_revision":"r1","brief":"private deterministic worker"}));
        self.command("admit", json!({"task":task}))
    }
    fn initialize(&self) {
        self.command(
            "create_project",
            json!({"name":"p","base":"0123456789abcdef0123456789abcdef01234567"}),
        );
    }
    fn workspace(&self, grant: &Value) -> PathBuf {
        self.dir
            .path()
            .join("executions")
            .join(grant["attempt"].as_str().unwrap())
            .join("workspace")
    }
    fn supervisor_request(&self, action: &str, binding: Value) -> Value {
        request(
            &self.supervisor_socket,
            json!({"token":SUPERVISOR_TOKEN,"action":action,"binding":binding}),
        )
    }
}

fn binding(grant: &Value) -> Value {
    json!({"project":"p","attempt":grant["attempt"],"operation":grant["operation"],"scope_epoch":grant["scope_epoch"],"base":grant["base"],"brief":grant["brief"]})
}

#[test]
fn repair2_late_occupied_slots_are_reconciled_after_historical_pages() {
    let mut f = Fixture::new();
    f.initialize();
    let seed = f.admit("historical-seed");
    f.controller.kill();
    let db = rusqlite::Connection::open(f.dir.path().join("state.db")).unwrap();
    db.execute(
        "UPDATE attempts SET state='exited',scope_epoch=scope_epoch+1 WHERE id=?1",
        [seed["attempt"].as_str().unwrap()],
    )
    .unwrap();
    db.execute("UPDATE slots SET attempt=NULL", []).unwrap();
    db.execute("UPDATE operations SET state='failed'", [])
        .unwrap();
    for n in 0..25 {
        let task = format!("historical-{n}");
        let attempt = format!("historical-attempt-{n}");
        let operation = format!("historical-operation-{n}");
        db.execute("INSERT INTO tasks SELECT project,?1,brief,?1,'aborted' FROM tasks WHERE id='historical-seed'", [&task]).unwrap();
        db.execute("INSERT INTO attempts(id,project,task,brief,base,state,scope_epoch,observation) SELECT ?1,project,?2,brief,base,'exited',2,'deterministic owned historical database fixture; no external effect' FROM attempts WHERE id=?3", rusqlite::params![attempt,task,seed["attempt"].as_str().unwrap()]).unwrap();
        db.execute("INSERT INTO operations(id,project,attempt,kind,state,payload) VALUES(?1,'p',?2,'start','failed',?3)", rusqlite::params![operation,attempt,json!({"attempt":attempt,"base":seed["base"],"brief":seed["brief"],"scope_epoch":1}).to_string()]).unwrap();
    }
    drop(db);
    f.controller = Fixture::controller(f.dir.path(), &f.api, &f.supervisor_socket);
    let a = f.admit("late-a");
    let b = f.admit("late-b");
    let page = f.snapshot();
    assert_eq!(page["attempts"].as_array().unwrap().len(), 20);
    assert_eq!(page["pagination"]["attempts"]["has_more"], true);
    assert!(
        page["attempts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|x| x["state"] == "exited")
    );
    let reconciled = f.reconcile();
    println!(
        "actual_late_slot_reconcile={reconciled}; occupied={}",
        page["occupied_slots"]
    );
    assert_eq!(
        reconciled["observed"].as_array().unwrap().len(),
        2,
        "public historical pages must not hide the two active runtime slots"
    );
    wait_for(|| {
        f.workspace(&a).join("heartbeat").exists() && f.workspace(&b).join("heartbeat").exists()
    });
    let context = f.query("runtime_context", json!({}));
    assert_eq!(context["attempts"].as_array().unwrap().len(), 2);
    assert_eq!(context["operations"].as_array().unwrap().len(), 2);
    assert!(
        context["operations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|o| o["payload"].is_string())
    );
    assert!(f.raw(json!({"auth":{"principal":a["principal"],"token":a["token"]},"project":"p","query":{"kind":"runtime_context","args":{}}}))["error"].is_string());
    for grant in [&a, &b] {
        f.command("cancel_attempt", json!({"attempt":grant["attempt"]}));
    }
    f.reconcile();
    assert_eq!(f.snapshot()["occupied_slots"], 0);
    assert!(
        f.query("runtime_context", json!({}))["attempts"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    println!("actual_late_slots_started_and_cancelled_after_25_exited_attempts=true");
}

#[test]
fn independently_supervised_workers_survive_restart_and_cancel_descendants() {
    let mut f = Fixture::new();
    f.initialize();
    let a = f.admit("a");
    let b = f.admit("b");
    f.reconcile();
    wait_for(|| {
        f.workspace(&a).join("heartbeat").exists() && f.workspace(&b).join("heartbeat").exists()
    });
    let before_a = f.supervisor_request("lookup", binding(&a))["ok"].clone();
    let before_b = f.supervisor_request("lookup", binding(&b))["ok"].clone();
    assert_ne!(before_a["handle"], before_b["handle"]);
    assert_ne!(before_a["pid"], before_b["pid"]);
    assert!(
        before_a["start_ticks"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            > 0
    );
    assert!(
        before_a["namespace_init"].as_u64().is_some(),
        "qualified PID namespace init identity required: {before_a}"
    );
    fs::write(
        f.workspace(&a).join("cross-probe.json"),
        json!({"pid":before_b["namespace_init"],"workspace":f.workspace(&b)}).to_string(),
    )
    .unwrap();
    wait_for(|| f.workspace(&a).join("cross-probe-passed").exists());
    println!(
        "actual_private_boundary_probe={}",
        fs::read_to_string(f.workspace(&a).join("identity.json")).unwrap()
    );
    println!("two_live_execution_identities a={before_a} b={before_b}");
    let heartbeat = fs::read(f.workspace(&a).join("heartbeat")).unwrap();
    f.controller.kill();
    wait_for(|| fs::read(f.workspace(&a).join("heartbeat")).unwrap() != heartbeat);
    f.controller = Fixture::controller(f.dir.path(), &f.api, &f.supervisor_socket);
    f.reconcile();
    assert_eq!(
        before_a["handle"],
        f.supervisor_request("lookup", binding(&a))["ok"]["handle"]
    );
    assert_eq!(
        before_b["handle"],
        f.supervisor_request("lookup", binding(&b))["ok"]["handle"]
    );
    assert_eq!(f.snapshot()["occupied_slots"], 2);
    // Capture a descendant independently from the host proc namespace. It deliberately escaped
    // the process group, so cancellation must destroy the PID namespace as well.
    let worker_pids: Vec<u32> = fs::read_dir("/proc")
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let pid = entry.file_name().to_str()?.parse().ok()?;
            let namespace = fs::read_link(entry.path().join("ns/pid")).ok()?;
            let init_identity =
                fs::read_to_string(entry.path().join("root/workspace/identity.json")).ok()?;
            (init_identity == fs::read_to_string(f.workspace(&a).join("identity.json")).ok()?
                && namespace != fs::read_link("/proc/self/ns/pid").ok()?)
            .then_some(pid)
        })
        .collect();
    assert!(
        worker_pids.len() >= 2,
        "actual worker descendants: {worker_pids:?}"
    );
    println!("host_namespace_worker_pids={worker_pids:?}");
    f.command("cancel_attempt", json!({"attempt":a["attempt"]}));
    assert_eq!(f.snapshot()["occupied_slots"], 2);
    let stale=f.raw(json!({"auth":{"principal":a["principal"],"token":a["token"]},"project":"p","runtime":{"kind":"reconcile"}}));
    assert!(stale["error"].is_string());
    f.reconcile();
    wait_for(|| f.snapshot()["attempts"][0]["state"] == "exited");
    wait_for(|| {
        worker_pids
            .iter()
            .all(|pid| !Path::new(&format!("/proc/{pid}")).exists())
    });
    assert_eq!(f.snapshot()["occupied_slots"], 1);
    assert_eq!(
        f.supervisor_request("lookup", binding(&b))["ok"]["state"],
        "running"
    );
    println!(
        "controller_restart_adopted_same_handles; cancellation_observed_removed_host_descendants={worker_pids:?}"
    );
    f.command("cancel_attempt", json!({"attempt":b["attempt"]}));
    f.reconcile();
    assert_eq!(f.snapshot()["occupied_slots"], 0);
}

#[test]
fn lost_start_reply_is_looked_up_and_unavailable_supervisor_keeps_capacity() {
    let mut f = Fixture::new();
    f.initialize();
    let a = f.admit("a");
    // Send the real start and drop the transport before reading its acknowledgement.
    let mut transport = UnixStream::connect(&f.supervisor_socket).unwrap();
    writeln!(
        transport,
        "{}",
        json!({"token":SUPERVISOR_TOKEN,"action":"start","binding":binding(&a)})
    )
    .unwrap();
    drop(transport);
    wait_for(|| f.workspace(&a).join("heartbeat").exists());
    let identity = f.supervisor_request("lookup", binding(&a))["ok"].clone();
    f.reconcile();
    assert_eq!(
        identity["handle"],
        f.supervisor_request("lookup", binding(&a))["ok"]["handle"]
    );
    let denied=f.raw(json!({"auth":{"principal":"operator","token":TOKEN},"project":"p","command":{"kind":"observe_attempt","expected_revision":f.snapshot()["revision"],"idempotency_key":"forged-exit","args":{"attempt":a["attempt"],"state":"exited","observation":"caller text"}}}));
    assert!(denied["error"].is_string());
    f.supervisor.kill();
    f.reconcile();
    let state = f.snapshot();
    assert_eq!(state["occupied_slots"], 1);
    assert_eq!(state["attempts"][0]["state"], "unknown");
    println!(
        "lost_reply_adopted={identity}; unreachable_supervisor_occupied_slots={}",
        state["occupied_slots"]
    );
}

#[test]
fn pause_prevents_prepared_dispatch_and_cancel_overtakes_start() {
    let f = Fixture::new();
    f.initialize();
    let a = f.admit("paused");
    f.command("pause_dispatch", json!({"paused":true}));
    f.reconcile();
    assert!(
        !f.workspace(&a).exists(),
        "paused prepared operation must not start an effect"
    );
    assert_eq!(f.snapshot()["operations"][0]["state"], "prepared");
    f.command("cancel_attempt", json!({"attempt":a["attempt"]}));
    f.reconcile();
    assert_eq!(f.snapshot()["occupied_slots"], 0);
    let tombstone = f.supervisor_request("start", binding(&a));
    assert_eq!(tombstone["ok"]["state"], "exited");
    assert!(tombstone["ok"]["pid"].is_null());
    assert!(!f.workspace(&a).exists());
    println!("cancel_before_start_tombstone={tombstone}");
}

#[test]
fn supervisor_restart_refuses_pid_adoption_and_retains_revoked_capacity() {
    let mut f = Fixture::new();
    f.initialize();
    let a = f.admit("uncertain");
    f.reconcile();
    wait_for(|| f.workspace(&a).join("heartbeat").exists());
    let before = f.supervisor_request("lookup", binding(&a))["ok"].clone();
    f.supervisor.kill();
    let executable = Path::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .parent()
        .unwrap()
        .join("corenius-attempt-supervisor");
    f.supervisor = Process(
        Command::new(executable)
            .args([&f.dir.path().join("supervisor.json"), &f.supervisor_socket])
            .env("CORENIUS_SUPERVISOR_TOKEN", SUPERVISOR_TOKEN)
            .spawn()
            .unwrap(),
    );
    wait_for(|| UnixStream::connect(&f.supervisor_socket).is_ok());
    f.reconcile();
    let after = f.supervisor_request("lookup", binding(&a))["ok"].clone();
    assert_eq!(after["state"], "unknown");
    assert_eq!(before["handle"], after["handle"]);
    assert_eq!(before["pid"], after["pid"]);
    f.command("cancel_attempt", json!({"attempt":a["attempt"]}));
    f.reconcile();
    assert_eq!(f.snapshot()["occupied_slots"], 1);
    assert_eq!(f.snapshot()["attempts"][0]["state"], "unknown");
    let stale=f.raw(json!({"auth":{"principal":a["principal"],"token":a["token"]},"project":"p","command":{"kind":"report_progress","expected_revision":f.snapshot()["revision"],"idempotency_key":"stale-worker","args":{"task":"uncertain","text":"stale"}}}));
    assert!(stale["error"].is_string());
    println!(
        "supervisor_restart_uncertain_retained_identity={after}; cancelled_unknown_occupied_slots=1"
    );
}

#[test]
fn symlink_roots_and_stale_execution_bindings_are_refused() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    f.initialize();
    let a = f.admit("alias");
    let target = f.dir.path().join("protected");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("sentinel"), "untouched").unwrap();
    symlink(
        &target,
        f.dir
            .path()
            .join("executions")
            .join(a["attempt"].as_str().unwrap()),
    )
    .unwrap();
    f.reconcile();
    assert_eq!(f.snapshot()["attempts"][0]["state"], "unknown");
    assert_eq!(f.snapshot()["occupied_slots"], 1);
    assert_eq!(
        fs::read_to_string(target.join("sentinel")).unwrap(),
        "untouched"
    );
    assert!(!target.join("workspace").exists());
    let mut wrong = binding(&a);
    wrong["scope_epoch"] = json!(2);
    assert!(f.supervisor_request("start", wrong)["error"].is_string());
    let mut injection = binding(&a);
    injection["executable"] = json!("/bin/sh");
    assert!(f.supervisor_request("start", injection)["error"].is_string());
    let not_controller = request(
        &f.supervisor_socket,
        json!({"token":TOKEN,"action":"lookup","binding":binding(&a)}),
    );
    assert!(not_controller["error"].is_string());
    println!("symlink_refused_without_target_mutation; stale_binding_and_profile_injection_denied");
}

#[test]
fn short_lived_execution_has_a_retained_terminal_identity() {
    let mut f = Fixture::new();
    f.supervisor.kill();
    fs::write(f.dir.path().join("supervisor.json"),json!({"root":f.dir.path().join("executions"),"profile":{"executable":"/usr/bin/true","args":[]}}).to_string()).unwrap();
    let executable = Path::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
        .parent()
        .unwrap()
        .join("corenius-attempt-supervisor");
    f.supervisor = Process(
        Command::new(executable)
            .args([&f.dir.path().join("supervisor.json"), &f.supervisor_socket])
            .env("CORENIUS_SUPERVISOR_TOKEN", SUPERVISOR_TOKEN)
            .spawn()
            .unwrap(),
    );
    wait_for(|| UnixStream::connect(&f.supervisor_socket).is_ok());
    f.initialize();
    let a = f.admit("short");
    f.reconcile();
    wait_for(|| {
        f.reconcile();
        f.snapshot()["attempts"][0]["state"] == "exited"
    });
    assert_eq!(
        f.snapshot()["attempts"][0]["state"],
        "exited",
        "owned normal termination must retain proof even if process exits before live PID sampling"
    );
    assert_eq!(f.snapshot()["occupied_slots"], 0);
    let identity = f.supervisor_request("lookup", binding(&a))["ok"].clone();
    assert!(identity["handle"].is_string());
    assert!(identity["pid_namespace"].is_u64());
    assert_eq!(identity, f.supervisor_request("start", binding(&a))["ok"]);
    println!("short_normal_termination_retained_identity={identity}");
}

#[test]
fn configured_controller_autonomously_drains_admission_and_cancellation() {
    let mut f = Fixture::new();
    f.controller.kill();
    f.controller = Process(
        Command::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
            .args([&f.dir.path().join("state.db"), &f.api])
            .env("CORENIUS_OPERATOR_TOKEN", TOKEN)
            .env("CORENIUS_SUPERVISOR_SOCKET", &f.supervisor_socket)
            .env("CORENIUS_SUPERVISOR_TOKEN", SUPERVISOR_TOKEN)
            .env_remove("CORENIUS_TEST_MANUAL_RUNTIME")
            .spawn()
            .unwrap(),
    );
    wait_for(|| UnixStream::connect(&f.api).is_ok());
    f.initialize();
    let a = f.admit("autonomous");
    // This test never sends runtime reconciliation or supervisor RPC requests.
    wait_for(|| f.workspace(&a).join("heartbeat").exists());
    wait_for(|| f.snapshot()["attempts"][0]["state"] == "running");
    f.command("cancel_attempt", json!({"attempt":a["attempt"]}));
    wait_for(|| f.snapshot()["attempts"][0]["state"] == "exited");
    assert_eq!(f.snapshot()["occupied_slots"], 0);
    println!(
        "autonomous_admission_cancel_observed_through_controller_only={}",
        f.snapshot()["attempts"][0]["observation"]
    );
}

impl Fixture {
    fn restart_profile(
        &mut self,
        executable: &str,
        args: Vec<String>,
        environment: &[(&str, &str)],
    ) {
        self.supervisor.kill();
        fs::write(self.dir.path().join("supervisor.json"),json!({"root":self.dir.path().join("executions"),"profile":{"executable":executable,"args":args}}).to_string()).unwrap();
        let executable = Path::new(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
            .parent()
            .unwrap()
            .join("corenius-attempt-supervisor");
        let mut command = Command::new(executable);
        command
            .args([
                &self.dir.path().join("supervisor.json"),
                &self.supervisor_socket,
            ])
            .env("CORENIUS_SUPERVISOR_TOKEN", SUPERVISOR_TOKEN);
        for (key, value) in environment {
            command.env(key, value);
        }
        self.supervisor = Process(command.spawn().unwrap());
        wait_for(|| UnixStream::connect(&self.supervisor_socket).is_ok());
    }
}

#[test]
fn repair_paused_dispatch_survives_first_lookup_transport_failure() {
    let mut f = Fixture::new();
    f.restart_profile(
        "/usr/bin/sleep",
        vec!["120".into()],
        &[("CORENIUS_TEST_DROP_FIRST_LOOKUP", "1")],
    );
    f.initialize();
    let a = f.admit("paused-flap");
    f.command("pause_dispatch", json!({"paused":true}));
    f.reconcile();
    assert!(
        !f.workspace(&a).exists(),
        "transient supervisor failure cannot authorize a paused start"
    );
    f.reconcile();
    assert!(!f.workspace(&a).exists());
    assert_eq!(f.snapshot()["occupied_slots"], 1);
    f.command("cancel_attempt", json!({"attempt":a["attempt"]}));
    f.reconcile();
    assert_eq!(f.snapshot()["occupied_slots"], 0);
}

#[test]
fn repair_sampling_barrier_precedes_workload_and_preserves_short_exit() {
    let mut f = Fixture::new();
    f.restart_profile(
        "/usr/bin/python3",
        vec![
            "-c".into(),
            "import pathlib; pathlib.Path('/workspace/workload-started').write_text('executed')"
                .into(),
        ],
        &[("CORENIUS_TEST_IDENTITY_DELAY_MS", "500")],
    );
    f.initialize();
    let a = f.admit("delayed-sample");
    let api = f.api.clone();
    let request = std::thread::spawn(move || {
        request(
            &api,
            json!({"auth":{"principal":"operator","token":TOKEN},"project":"p","runtime":{"kind":"reconcile"}}),
        )
    });
    wait_for(|| {
        f.dir
            .path()
            .join("executions")
            .join(a["attempt"].as_str().unwrap())
            .join("sampling-delay")
            .exists()
    });
    std::thread::sleep(Duration::from_millis(80));
    assert!(
        !f.workspace(&a).join("workload-started").exists(),
        "workload executed before durable identity qualification"
    );
    assert!(request.join().unwrap()["error"].is_null());
    wait_for(|| {
        f.reconcile();
        f.snapshot()["attempts"][0]["state"] == "exited"
    });
    assert_eq!(f.snapshot()["occupied_slots"], 0);
    assert!(f.workspace(&a).join("workload-started").exists());
    println!(
        "repair_delayed_sampling_terminal_identity={}",
        f.supervisor_request("lookup", binding(&a))["ok"]
    );
}

#[test]
fn repair_setup_failure_disposes_owned_children_even_with_unknown_record() {
    let mut f = Fixture::new();
    f.restart_profile("/usr/bin/python3",vec!["-c".into(),"import pathlib,time\nwhile True: pathlib.Path('/workspace/unsafe-heartbeat').write_text(str(time.monotonic_ns())); time.sleep(.02)".into()],&[("CORENIUS_TEST_SETUP_ERROR","1")]);
    f.initialize();
    let a = f.admit("setup-error");
    f.reconcile();
    f.command("cancel_attempt", json!({"attempt":a["attempt"]}));
    f.reconcile();
    let child_list = format!(
        "/proc/{}/task/{}/children",
        f.supervisor.0.id(),
        f.supervisor.0.id()
    );
    let children = fs::read_to_string(child_list).unwrap();
    assert!(
        children.trim().is_empty(),
        "postspawn failure left live owned children: {children}"
    );
    let heartbeat = f.workspace(&a).join("unsafe-heartbeat");
    let prior = fs::read(&heartbeat).ok();
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(
        prior,
        fs::read(heartbeat).ok(),
        "unconfirmed setup workload still runs after cancellation"
    );
    println!(
        "repair_setup_failure_disposed_identity={}; owned_children=0; occupied_slots={}",
        f.supervisor_request("lookup", binding(&a))["ok"],
        f.snapshot()["occupied_slots"]
    );
}

fn pidfd_ready(fd: i32) -> bool {
    let mut descriptor = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    assert!(unsafe { libc::poll(&mut descriptor, 1, 0) } >= 0);
    descriptor.revents & libc::POLLIN != 0
}

struct OwnedInit(std::os::fd::OwnedFd);
impl Drop for OwnedInit {
    fn drop(&mut self) {
        use std::os::fd::AsRawFd;
        unsafe {
            libc::syscall(
                libc::SYS_pidfd_send_signal,
                self.0.as_raw_fd(),
                libc::SIGKILL,
                std::ptr::null::<libc::siginfo_t>(),
                0,
            );
        }
        let deadline = Instant::now() + Duration::from_millis(500);
        while !pidfd_ready(self.0.as_raw_fd()) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn repair_zombie_init_leader_with_live_thread_cannot_release_early() {
    use std::os::fd::{AsRawFd, FromRawFd};
    let mut f = Fixture::new();
    let script = r#"import pathlib,time,threading,ctypes,subprocess
w=pathlib.Path('/workspace')
def worker():
 subprocess.Popen(['/usr/bin/sleep','120'],start_new_session=True)
 while True: (w/'thread-heartbeat').write_text(str(time.monotonic_ns())); time.sleep(.02)
threading.Thread(target=worker).start()
while not (w/'leader-release').exists(): time.sleep(.01)
ctypes.CDLL(None).prctl(1,0)
ctypes.CDLL(None).syscall(TARGET_SYS_EXIT,0)
"#;
    let script = script.replace("TARGET_SYS_EXIT", &libc::SYS_exit.to_string());
    f.restart_profile("/usr/bin/python3", vec!["-c".into(), script], &[]);
    f.initialize();
    let a = f.admit("zombie-thread");
    f.reconcile();
    let identity = f.supervisor_request("lookup", binding(&a))["ok"].clone();
    println!(
        "zombie_setup_identity={identity}; log={}",
        fs::read_to_string(
            f.dir
                .path()
                .join("executions")
                .join(a["attempt"].as_str().unwrap())
                .join("output.log")
        )
        .unwrap()
    );
    let init = identity["namespace_init"].as_u64().unwrap() as i32;
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, init, 0) } as i32;
    assert!(fd >= 0);
    let pidfd = OwnedInit(unsafe { std::os::fd::OwnedFd::from_raw_fd(fd) });
    wait_for(|| f.workspace(&a).join("thread-heartbeat").exists());
    let namespace = format!("pid:[{}]", identity["pid_namespace"].as_u64().unwrap());
    let descendants: Vec<_> = fs::read_dir("/proc")
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let pid = entry.file_name().to_str()?.parse::<u32>().ok()?;
            if pid == init as u32
                || fs::read_link(entry.path().join("ns/pid"))
                    .ok()?
                    .to_string_lossy()
                    != namespace
            {
                return None;
            }
            let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) } as i32;
            (fd >= 0).then(|| {
                (
                    pid,
                    OwnedInit(unsafe { std::os::fd::OwnedFd::from_raw_fd(fd) }),
                )
            })
        })
        .collect();
    assert!(
        !descendants.is_empty(),
        "fixture must have a separately addressed descendant"
    );
    fs::write(f.workspace(&a).join("leader-release"), "release").unwrap();
    wait_for(|| {
        fs::read_to_string(format!("/proc/{init}/stat"))
            .unwrap()
            .rsplit_once(") ")
            .unwrap()
            .1
            .starts_with("Z ")
    });
    assert!(
        !pidfd_ready(pidfd.0.as_raw_fd()),
        "fixture must have a live thread behind zombie leader"
    );
    f.command("cancel_attempt", json!({"attempt":a["attempt"]}));
    f.reconcile();
    assert!(
        pidfd_ready(pidfd.0.as_raw_fd()),
        "capacity released while exact namespace process still has live threads"
    );
    assert_eq!(f.snapshot()["occupied_slots"], 0);
    for (pid, fd) in &descendants {
        assert!(
            pidfd_ready(fd.0.as_raw_fd()),
            "namespace descendant {pid} still alive"
        );
    }
    println!(
        "repair_zombie_thread_and_descendants_confirmed={}",
        f.supervisor_request("lookup", binding(&a))["ok"]
    );
}

#[test]
fn repair_database_symlink_target_inside_worker_toolchain_is_refused_before_open() {
    use std::os::unix::fs::symlink;
    let directory = tempfile::tempdir().unwrap();
    let unsafe_root = directory.path().join("unsafe");
    fs::create_dir(&unsafe_root).unwrap();
    let alias = directory.path().join("alias.db");
    symlink("/usr/local/corenius-repair-target/state.db", &alias).unwrap();
    // Create the target within this owned fixture, then make only that directory visible
    // under /usr in a disposable mount namespace. No host /usr files are changed.
    fs::write(unsafe_root.join("state.db"), []).unwrap();
    let api = directory.path().join("api.sock");
    let mut child = Process(
        Command::new("/usr/bin/bwrap")
            .args([
                "--die-with-parent",
                "--ro-bind",
                "/",
                "/",
                "--tmpfs",
                "/usr/local",
                "--bind",
            ])
            .arg(&unsafe_root)
            .arg("/usr/local/corenius-repair-target")
            .arg("--bind")
            .arg(directory.path())
            .arg(directory.path())
            .arg("--")
            .arg(env!("CARGO_BIN_EXE_corenius-sqlite-controller"))
            .args([&alias, &api])
            .env("CORENIUS_OPERATOR_TOKEN", TOKEN)
            .env(
                "CORENIUS_SUPERVISOR_SOCKET",
                directory.path().join("supervisor.sock"),
            )
            .env("CORENIUS_SUPERVISOR_TOKEN", SUPERVISOR_TOKEN)
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_millis(500);
    while child.0.try_wait().unwrap().is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        child.0.try_wait().unwrap().is_some(),
        "controller served a database target visible inside worker /usr mount"
    );
    assert_eq!(
        fs::metadata(unsafe_root.join("state.db")).unwrap().len(),
        0,
        "unsafe resolved database opened before boundary validation"
    );
}

#[test]
fn repair_short_exit_during_delayed_identity_sampling_is_confirmed() {
    let mut f = Fixture::new();
    f.restart_profile(
        "/usr/bin/true",
        vec![],
        &[("CORENIUS_TEST_IDENTITY_DELAY_MS", "200")],
    );
    f.initialize();
    let _a = f.admit("short-delayed");
    f.reconcile();
    wait_for(|| {
        f.reconcile();
        f.snapshot()["attempts"][0]["state"] == "exited"
    });
    assert_eq!(f.snapshot()["occupied_slots"], 0);
    println!(
        "repair_delayed_short_terminal_identity={}",
        f.supervisor_request("lookup", binding(&_a))["ok"]
    );
}

fn qualification_decoy() -> (Fixture, Value, OwnedInit) {
    use std::os::fd::FromRawFd;
    let f = Fixture::new();
    f.initialize();
    let a = f.admit("qualification-decoy");
    f.reconcile();
    wait_for(|| f.workspace(&a).join("heartbeat").exists());
    let identity = f.supervisor_request("lookup", binding(&a))["ok"].clone();
    println!(
        "decoy_kernel_stat={}",
        fs::read_to_string(format!("/proc/{}/stat", identity["namespace_init"])).unwrap()
    );
    let fd = unsafe {
        libc::syscall(
            libc::SYS_pidfd_open,
            identity["namespace_init"].as_u64().unwrap(),
            0,
        )
    } as i32;
    assert!(fd >= 0);
    (
        f,
        identity,
        OwnedInit(unsafe { std::os::fd::OwnedFd::from_raw_fd(fd) }),
    )
}

fn refuse_unqualified_init(variable: &str, value: &str, decoy: &OwnedInit) {
    use std::os::fd::AsRawFd;
    let mut f = Fixture::new();
    f.restart_profile("/usr/bin/python3", vec!["-c".into(), "import pathlib,time; pathlib.Path('/workspace/unqualified-workload').write_text('started'); time.sleep(120)".into()], &[(variable, value)]);
    f.initialize();
    let a = f.admit("qualification-refusal");
    let reply = f.supervisor_request("start", binding(&a));
    println!(
        "qualification_reply={reply}; real_decoy_pidfd_ready={}",
        pidfd_ready(decoy.0.as_raw_fd())
    );
    assert!(
        reply["error"].is_string(),
        "unqualified init must be refused before installing signal authority: {reply}"
    );
    assert!(
        !pidfd_ready(decoy.0.as_raw_fd()),
        "qualification failure must not signal provisional descriptor"
    );
    assert!(!f.workspace(&a).join("unqualified-workload").exists());
    f.supervisor_request("cancel", binding(&a));
    assert!(
        !pidfd_ready(decoy.0.as_raw_fd()),
        "cancel must not signal refused descriptor"
    );
    let children = format!(
        "/proc/{}/task/{}/children",
        f.supervisor.0.id(),
        f.supervisor.0.id()
    );
    wait_for(|| fs::read_to_string(&children).unwrap().trim().is_empty());
    let setup: Value = serde_json::from_slice(
        &fs::read(
            f.workspace(&a)
                .parent()
                .unwrap()
                .join("namespace-info.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let namespace = format!("pid:[{}]", setup["pid-namespace"].as_u64().unwrap());
    wait_for(|| {
        !fs::read_dir("/proc")
            .unwrap()
            .filter_map(|entry| entry.ok())
            .any(|entry| {
                fs::read_link(entry.path().join("ns/pid"))
                    .is_ok_and(|link| link.to_string_lossy() == namespace)
            })
    });
    println!("qualification_private_namespace_disposed={setup}");
    println!(
        "qualification_disposed_identity={}; decoy_alive=true; owned_children=0",
        f.supervisor_request("lookup", binding(&a))["ok"]
    );
}

#[test]
fn repair2_descriptor_target_mismatch_is_never_signal_authority() {
    let (_owner, identity, decoy) = qualification_decoy();
    println!("owned_descriptor_decoy={identity}");
    refuse_unqualified_init(
        "CORENIUS_TEST_INIT_PIDFD_TARGET",
        &identity["namespace_init"].to_string(),
        &decoy,
    );
}

#[test]
fn repair2_another_namespace_init_cannot_qualify_as_owned_child() {
    let (_owner, identity, decoy) = qualification_decoy();
    let acknowledgement =
        json!({"child-pid":identity["namespace_init"],"pid-namespace":identity["pid_namespace"]});
    println!("owned_namespace_decoy={identity}");
    refuse_unqualified_init(
        "CORENIUS_TEST_INIT_ACK",
        &acknowledgement.to_string(),
        &decoy,
    );
}

//! Real process counterexample: process-group cleanup is not full-tree containment.
//! This does not qualify a command backend.
#![cfg(unix)]

use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn detached_descendant_survives_original_group_cleanup() {
    let directory = tempfile::tempdir().unwrap();
    struct Release(std::path::PathBuf);
    impl Drop for Release {
        fn drop(&mut self) {
            let _ = std::fs::write(self.0.join("release"), b"stop");
        }
    }
    // The escaped helper is never signalled by PID. It responds cooperatively
    // through this unique test directory and has its own bounded lifetime.
    let _release = Release(directory.path().to_owned());
    let mut command = helper("leader", directory.path());
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut leader = command.spawn().unwrap();
    let leader_pid = i32::try_from(leader.id()).unwrap();
    let ready = wait_for(&directory.path().join("ready"));
    // Do not wait/try_wait first: the owned, unreaped leader reserves this PID
    // until after signalling, even if it exited while this test was descheduled.
    let signal_result = unsafe { libc::kill(-leader_pid, libc::SIGTERM) };
    let leader_status = leader.wait().unwrap();
    assert!(
        ready,
        "detached helper did not become ready within the bound"
    );
    assert_eq!(signal_result, 0);
    assert!(!leader_status.success());
    std::fs::write(directory.path().join("challenge.tmp"), b"after-group-exit").unwrap();
    std::fs::rename(
        directory.path().join("challenge.tmp"),
        directory.path().join("challenge"),
    )
    .unwrap();
    assert!(
        wait_for(&directory.path().join("response")),
        "detached child must respond after original group termination"
    );
    assert_eq!(
        std::fs::read(directory.path().join("response")).unwrap(),
        b"after-group-exit"
    );
}

fn wait_for(path: &Path) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if path.exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

fn helper(role: &str, directory: &Path) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--ignored", "--exact", "process_tree_fixture"])
        .env("HARP_NATIVE_JOB_TEST_ROLE", role)
        .env("HARP_NATIVE_JOB_TEST_DIRECTORY", directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

#[test]
#[ignore = "subprocess fixture invoked by the containment counterexample"]
fn process_tree_fixture() {
    let directory =
        std::path::PathBuf::from(std::env::var_os("HARP_NATIVE_JOB_TEST_DIRECTORY").unwrap());
    let deadline = Instant::now() + Duration::from_secs(30);
    match std::env::var("HARP_NATIVE_JOB_TEST_ROLE").as_deref() {
        Ok("leader") => {
            let mut command = helper("descendant", &directory);
            unsafe {
                command.pre_exec(|| {
                    if libc::setsid() == -1 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
            let mut descendant = command.spawn().unwrap();
            while Instant::now() < deadline && !directory.join("release").exists() {
                std::thread::sleep(Duration::from_millis(10));
            }
            // Normal/error cleanup waits. In the counterexample the original
            // group is terminated first and init reaps the cooperative orphan.
            let _ = descendant.wait();
        }
        Ok("descendant") => {
            std::fs::write(directory.join("ready"), b"ready").unwrap();
            while Instant::now() < deadline && !directory.join("release").exists() {
                if let Ok(challenge) = std::fs::read(directory.join("challenge")) {
                    std::fs::write(directory.join("response.tmp"), challenge).unwrap();
                    std::fs::rename(directory.join("response.tmp"), directory.join("response"))
                        .unwrap();
                    return;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        _ => panic!("fixture requires its test-specific role"),
    }
}

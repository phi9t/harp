use corenius::supervisor::{ProcessInfo, parse_ps_output, snapshot_from_processes};

#[test]
fn parses_ps_rows_with_command_spaces() {
    let rows = parse_ps_output(
        " 100 1 Ss 00:01 traecli acp serve -c model=\"gpt-5.2\"\n 101 100 Sl 00:00 node worker child\n",
    )
    .unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].pid, 100);
    assert_eq!(rows[0].ppid, 1);
    assert_eq!(rows[0].state, "Ss");
    assert_eq!(rows[0].elapsed, "00:01");
    assert_eq!(rows[0].command, "traecli acp serve -c model=\"gpt-5.2\"");
    assert_eq!(rows[1].command, "node worker child");
}

#[test]
fn snapshot_includes_root_and_descendants_only() {
    let snapshot = snapshot_from_processes(
        100,
        vec![
            process(1, 0, "Ss", "init"),
            process(100, 1, "Sl", "traecli acp serve"),
            process(101, 100, "S", "provider helper"),
            process(102, 101, "S", "tool helper"),
            process(200, 1, "S", "unrelated"),
        ],
    );

    assert!(snapshot.healthy);
    assert_eq!(
        snapshot
            .processes
            .iter()
            .map(|process| process.pid)
            .collect::<Vec<_>>(),
        vec![100, 101, 102]
    );
}

#[test]
fn snapshot_flags_missing_root_and_bad_process_states() {
    let missing = snapshot_from_processes(999, vec![process(100, 1, "S", "other")]);
    assert!(!missing.healthy);
    assert_eq!(missing.issues, vec!["root pid 999 is not running"]);

    let unhealthy = snapshot_from_processes(
        100,
        vec![
            process(100, 1, "Sl", "traecli acp serve"),
            process(101, 100, "Z", "defunct"),
            process(102, 100, "T", "stopped"),
        ],
    );
    assert!(!unhealthy.healthy);
    assert_eq!(
        unhealthy.issues,
        vec!["pid 101 is zombie", "pid 102 is stopped"]
    );
}

fn process(pid: u32, ppid: u32, state: &str, command: &str) -> ProcessInfo {
    ProcessInfo {
        pid,
        ppid,
        state: state.to_string(),
        elapsed: "00:00".to_string(),
        command: command.to_string(),
    }
}

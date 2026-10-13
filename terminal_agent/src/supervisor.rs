use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::process::Command;

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};

use crate::core::timestamp;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub state: String,
    pub elapsed: String,
    pub command: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub root_pid: u32,
    pub timestamp: String,
    pub healthy: bool,
    pub issues: Vec<String>,
    pub processes: Vec<ProcessInfo>,
}

pub trait ProcessLister: Send + Sync {
    fn list_processes(&self) -> anyhow::Result<Vec<ProcessInfo>>;
}

#[derive(Clone, Debug, Default)]
pub struct PsProcessLister;

impl ProcessLister for PsProcessLister {
    fn list_processes(&self) -> anyhow::Result<Vec<ProcessInfo>> {
        let output = Command::new("ps")
            .args(["-eo", "pid=,ppid=,stat=,etime=,command="])
            .output()
            .context("run ps for process supervision")?;
        if !output.status.success() {
            bail!("ps exited with status {}", output.status);
        }
        let stdout = String::from_utf8(output.stdout).context("ps output was not UTF-8")?;
        parse_ps_output(&stdout)
    }
}

#[derive(Clone, Debug)]
pub struct ProcessSupervisor<L = PsProcessLister> {
    lister: L,
}

impl ProcessSupervisor<PsProcessLister> {
    pub fn system() -> Self {
        Self {
            lister: PsProcessLister,
        }
    }
}

impl<L: ProcessLister> ProcessSupervisor<L> {
    pub fn new(lister: L) -> Self {
        Self { lister }
    }

    pub fn snapshot(&self, root_pid: u32) -> anyhow::Result<ProcessSnapshot> {
        let all = self.lister.list_processes()?;
        Ok(snapshot_from_processes(root_pid, all))
    }
}

pub fn parse_ps_output(output: &str) -> anyhow::Result<Vec<ProcessInfo>> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_ps_line)
        .collect()
}

fn parse_ps_line(line: &str) -> anyhow::Result<ProcessInfo> {
    let mut fields = line.split_whitespace();
    let pid = fields
        .next()
        .context("missing pid")?
        .parse::<u32>()
        .context("parse pid")?;
    let ppid = fields
        .next()
        .context("missing ppid")?
        .parse::<u32>()
        .context("parse ppid")?;
    let state = fields.next().context("missing process state")?.to_string();
    let elapsed = fields.next().context("missing elapsed time")?.to_string();
    let command = fields.collect::<Vec<_>>().join(" ");
    if command.is_empty() {
        bail!("missing command");
    }
    Ok(ProcessInfo {
        pid,
        ppid,
        state,
        elapsed,
        command,
    })
}

pub fn snapshot_from_processes(root_pid: u32, processes: Vec<ProcessInfo>) -> ProcessSnapshot {
    let by_pid = processes
        .iter()
        .cloned()
        .map(|process| (process.pid, process))
        .collect::<BTreeMap<_, _>>();
    let mut children = BTreeMap::<u32, Vec<u32>>::new();
    for process in &processes {
        children.entry(process.ppid).or_default().push(process.pid);
    }
    for child_pids in children.values_mut() {
        child_pids.sort_unstable();
    }

    let mut selected = Vec::new();
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::from([root_pid]);
    while let Some(pid) = queue.pop_front() {
        if !visited.insert(pid) {
            continue;
        }
        if let Some(process) = by_pid.get(&pid) {
            selected.push(process.clone());
            if let Some(child_pids) = children.get(&pid) {
                for child_pid in child_pids {
                    queue.push_back(*child_pid);
                }
            }
        }
    }

    selected.sort_by_key(|process| process.pid);
    let mut issues = Vec::new();
    if !by_pid.contains_key(&root_pid) {
        issues.push(format!("root pid {root_pid} is not running"));
    }
    for process in &selected {
        if process.state.contains('Z') {
            issues.push(format!("pid {} is zombie", process.pid));
        }
        if process.state.contains('T') {
            issues.push(format!("pid {} is stopped", process.pid));
        }
    }

    ProcessSnapshot {
        root_pid,
        timestamp: timestamp(),
        healthy: issues.is_empty(),
        issues,
        processes: selected,
    }
}

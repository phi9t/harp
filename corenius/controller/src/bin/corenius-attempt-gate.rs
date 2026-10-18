//! Trusted single-threaded PID1 gate: workload exec follows supervisor identity persistence.
use std::{
    io::Read,
    os::{fd::FromRawFd, unix::process::CommandExt},
    process::Command,
};
fn main() {
    if let Err(error) = run() {
        eprintln!("attempt gate: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err("operator profile required".into());
    }
    let mut gate = unsafe { std::fs::File::from_raw_fd(0) };
    let mut permission = [0];
    gate.read_exact(&mut permission)
        .map_err(|e| e.to_string())?;
    if permission != [b'R'] {
        return Err("dispatch gate refused".into());
    }
    drop(gate);
    Err(Command::new(&args[1])
        .args(&args[2..])
        .stdin(std::process::Stdio::null())
        .exec()
        .to_string())
}

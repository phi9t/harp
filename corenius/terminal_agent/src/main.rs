//! Retirement boundary; historical mechanisms remain recoverable in Git.
fn main() {
    eprintln!(
        "Retired imported controller entrypoint. Use harp durable --help for the admitted SQLite lifecycle; legacy state is not migrated."
    );
    std::process::exit(2);
}

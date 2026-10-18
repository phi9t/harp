#!/bin/sh
# Retained model-free SQLite baseline; no PostgreSQL/Temporal services.
set -eu
cargo test --locked --offline --manifest-path controller/Cargo.toml --features fault-injection -- --nocapture --test-threads="${CHECK_CONTROLLER_THREADS:-2}"

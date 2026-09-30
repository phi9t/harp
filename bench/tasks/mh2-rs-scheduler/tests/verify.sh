#!/bin/sh
set -eu

rustc --edition=2021 --crate-name mh2_rs_scheduler --crate-type lib src/lib.rs \
  -o /tmp/libmh2_rs_scheduler.rlib
rustc --edition=2021 --test tests/scheduler_integration.rs \
  --extern mh2_rs_scheduler=/tmp/libmh2_rs_scheduler.rlib \
  -L /tmp \
  -o /tmp/scheduler_integration
/tmp/scheduler_integration

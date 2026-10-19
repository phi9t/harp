#!/bin/sh
set -eu

rustc --edition=2021 --crate-name mh2_rs_scheduler --crate-type lib src/lib.rs \
  -o /run/insula-tmp/libmh2_rs_scheduler.rlib
rustc --edition=2021 --test tests/scheduler_integration.rs \
  --extern mh2_rs_scheduler=/run/insula-tmp/libmh2_rs_scheduler.rlib \
  -L /run/insula-tmp \
  -o /run/insula-tmp/scheduler_integration
/run/insula-tmp/scheduler_integration

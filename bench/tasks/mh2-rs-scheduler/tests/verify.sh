#!/bin/sh
set -eu

rustc --edition=2021 --crate-name mh2_rs_scheduler --crate-type lib src/lib.rs \
  -o /run/corenius-tmp/libmh2_rs_scheduler.rlib
rustc --edition=2021 --test tests/scheduler_integration.rs \
  --extern mh2_rs_scheduler=/run/corenius-tmp/libmh2_rs_scheduler.rlib \
  -L /run/corenius-tmp \
  -o /run/corenius-tmp/scheduler_integration
/run/corenius-tmp/scheduler_integration

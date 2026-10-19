#!/usr/bin/env bash
set -euo pipefail
actual="$(tr -d '\n' < app/value.txt)"
test "$actual" = "42"

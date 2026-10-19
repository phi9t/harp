#!/bin/sh
set -eu

blade-offline build //...
PYTHONPATH="$PWD/python" python3 tests/check_ops.py

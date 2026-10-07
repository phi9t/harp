#!/bin/sh
set -eu
python3 tools/check_temp_paths.py
python3 tools/test_temp_root.py

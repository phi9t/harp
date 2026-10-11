#!/bin/sh
set -eu
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -v

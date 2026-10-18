#!/bin/sh
set -eu

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tools.agents.review_snapshot_test

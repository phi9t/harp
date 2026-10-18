#!/bin/sh
set -eu
exec python3 -m unittest controller.durable.runtime_test controller.durable.insula_test controller.durable.cgroup_test -v

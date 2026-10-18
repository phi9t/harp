#!/bin/sh
# Ordinary CI verifies qualification refusal and reporting without privileged setup.
set -eu
exec python3 -m unittest controller.durable.qualify_test -v

#!/usr/bin/env bash
# The former host-bind namespace wrapper is retired. Rootfs ownership is required.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec "$ROOT/scripts/insula/run-emacs-internals-probes-rootfs.sh" "$@"

#!/usr/bin/env bash
set -euo pipefail

blade-offline build //:cuda_app
out="$(./build64_release/cuda_app)"
case "$out" in
    *"BLADE_CUDA_RESULT answer=42"*"sm=100"*) ;;
    *)
        printf '%s\n' "$out"
        exit 1
        ;;
esac

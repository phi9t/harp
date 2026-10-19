#!/usr/bin/env bash
# GPU leases for concurrent agents on one host. Leases are exclusive flocks on
# $MILANO_CACHE_ROOT/locks/gpu/gpu-<id>; the cache is the same host directory
# in every Insula, so leases hold across sandboxes. The lock is held by this
# process and everything it execs or spawns until they all exit.

gpu_lease_dir() { printf '%s/locks/gpu' "$MILANO_CACHE_ROOT"; }

# lease_devices COUNT: lease COUNT healthy GPUs (only from CUDA_VISIBLE_DEVICES
# if it is set), export CUDA_VISIBLE_DEVICES to them, and mark the lease as held
# so child pipelines (e.g. gates under qualification) reuse it. Waits up to
# MILANO_GPU_LEASE_WAIT seconds (default 900) for busy GPUs.
lease_devices() {
  local count="$1"
  if [[ -n "${MILANO_GPU_LEASE:-}" ]]; then
    return 0
  fi
  local dir deadline healthy id fd
  dir="$(gpu_lease_dir)"
  mkdir -p "$dir"
  deadline=$((SECONDS + ${MILANO_GPU_LEASE_WAIT:-900}))
  while true; do
    healthy="$(python3 -m milano.cuda_health --healthy-devices 2>/dev/null)"
    local acquired=() fds=() busy=()
    for id in ${healthy//,/ }; do
      exec {fd}>>"$dir/gpu-$id"
      if flock -n "$fd"; then
        acquired+=("$id")
        fds+=("$fd")
        printf 'candidate=%s pid=%s since=%s\n' \
          "${MILANO_CANDIDATE_DIGEST:-?}" "$$" "$(date -u +%FT%TZ)" > "$dir/gpu-$id.holder"
        [[ "${#acquired[@]}" -eq "$count" ]] && break
      else
        busy+=("$id")
        exec {fd}>&-
      fi
    done
    if [[ "${#acquired[@]}" -eq "$count" ]]; then
      CUDA_VISIBLE_DEVICES="$(IFS=,; printf '%s' "${acquired[*]}")"
      export CUDA_VISIBLE_DEVICES MILANO_GPU_LEASE="$CUDA_VISIBLE_DEVICES"
      printf 'leased CUDA_VISIBLE_DEVICES=%s\n' "$CUDA_VISIBLE_DEVICES" >&2
      return 0
    fi
    for fd in "${fds[@]}"; do exec {fd}>&-; done
    local holders=()
    for id in "${busy[@]}"; do
      holders+=("gpu-$id: $(cat "$dir/gpu-$id.holder" 2>/dev/null || echo unknown)")
    done
    if (( SECONDS >= deadline )); then
      printf 'could not lease %s GPU(s) (healthy: %s); busy:\n' "$count" "${healthy:-none}" >&2
      printf '  %s\n' "${holders[@]}" >&2
      return 1
    fi
    printf 'waiting for %s free GPU(s); busy:\n' "$count" >&2
    printf '  %s\n' "${holders[@]}" >&2
    sleep 10
  done
}

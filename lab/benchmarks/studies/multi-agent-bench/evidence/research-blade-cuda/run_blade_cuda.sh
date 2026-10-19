#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_TEMPLATE="$SCRIPT_DIR/project"
RUNS_DIR="$SCRIPT_DIR/runs"
mkdir -p "$RUNS_DIR"

RUN_ROOT="$(mktemp -d -p "$RUNS_DIR" blade-cuda-XXXXXX)"
PROJECT="$RUN_ROOT/project"
HOME_DIR="$RUN_ROOT/home"
CUDA_MOUNT="$RUN_ROOT/cuda"
LOG_DIR="$RUN_ROOT/logs"
mkdir -p "$PROJECT" "$HOME_DIR" "$CUDA_MOUNT" "$LOG_DIR"
cp -a "$PROJECT_TEMPLATE"/. "$PROJECT"/

CUDA_REAL="$(readlink -f /usr/local/cuda)"
GCC_ROOT="/opt/tiger/typhoon-blade/gccs/x86_64-x86_64-gcc-830"
LOADER="$GCC_ROOT/sysroot/lib64/ld-linux-x86-64.so.2"
if [[ ! -x "$LOADER" ]]; then
  GCC_ROOT="/opt/tiger/typhoon-blade/gccs/x86_64-x86_64-gcc-494"
  LOADER="$GCC_ROOT/sysroot/lib64/ld-linux-x86-64.so.2"
fi

if [[ ! -x "$CUDA_REAL/bin/nvcc" ]]; then
  echo "missing nvcc under $CUDA_REAL" >&2
  exit 2
fi
if [[ ! -x "$LOADER" ]]; then
  echo "missing Blade toolchain loader at $LOADER" >&2
  exit 2
fi

EXECUTOR_SIZE="$(stat -c '%s' /opt/tiger/typhoon-blade/executor)"
mkdir -p "$HOME_DIR/.blade/blade_status"
touch "$HOME_DIR/.blade/blade_status/pip_checking.$EXECUTOR_SIZE"
touch "$HOME_DIR/.blade/blade_status/pip3_checking.$EXECUTOR_SIZE"

LD_PATH="$CUDA_MOUNT/lib64:$CUDA_MOUNT/targets/x86_64-linux/lib:$GCC_ROOT/lib:$GCC_ROOT/sysroot/lib64"
RUN_LIBRARY_PATH="$CUDA_MOUNT/lib64:$CUDA_MOUNT/targets/x86_64-linux/lib:$GCC_ROOT/lib"

bwrap_base=(
  bwrap
  --ro-bind / /
  --dev-bind /dev /dev
  --bind "$RUN_ROOT" "$RUN_ROOT"
  --ro-bind "$CUDA_REAL" "$CUDA_MOUNT"
  --tmpfs /home
  --tmpfs /tmp
  --unshare-net
  --clearenv
  --setenv HOME "$HOME_DIR"
  --setenv PATH "/opt/tiger/ss_bin:$CUDA_MOUNT/bin:/usr/local/bin:/usr/bin:/bin"
  --setenv USER bench
  --setenv NVCC "$CUDA_MOUNT/bin/nvcc"
  --setenv CUDA_PATH "$CUDA_MOUNT"
  --setenv BLADE_GCC_ROOT "$GCC_ROOT"
  --setenv BLADE_RUN_LIBRARY_PATH "$RUN_LIBRARY_PATH"
  --setenv BLADE_LOADER "$LOADER"
  --setenv TMPDIR /tmp
)

quote_cmd() {
  printf '%q ' "$@"
  printf '\n'
}

print_excerpt() {
  local log="$1"
  grep -E \
    '^(drwx|-rwx|nvcc:|Cuda compilation tools|[[:space:]]+\[Requesting program interpreter:|[[:space:]]+libcudart|[[:space:]]+/lib64/ld-linux|COMMAND\[|RC\[|WALL_MS\[|LOG\[|CUDA_ERROR|BLADE_CUDA_RESULT|\[minitest\]|.*Blade\((info|warning|error)\): (Blade version|Language version|TOOLCHAIN MODE|Get toolchain env file|CC=|CXX=|start build with ninja|building done|building failure|success|All tests passed|[0-9]+ tests failed|Run [0-9]+ tests|xml report:)|\[[0-9]+/[0-9]+\] (NvccObject|LINK|CXX|AR)|FAILED:)' \
    "$log" | sed -n '1,120p' || true
}

run_logged() {
  local label="$1"
  shift
  local log="$LOG_DIR/$label.log"
  local start end rc ms
  start="$(date +%s%3N)"
  echo "COMMAND[$label]: $(quote_cmd "$@")"
  set +e
  "$@" >"$log" 2>&1
  rc=$?
  set -e
  end="$(date +%s%3N)"
  ms=$((end - start))
  echo "RC[$label]: $rc"
  echo "WALL_MS[$label]: $ms"
  echo "LOG[$label]: $log"
  print_excerpt "$log"
  if [[ "$rc" -ne 0 ]]; then
    exit "$rc"
  fi
}

run_logged_expect_failure() {
  local label="$1"
  shift
  local log="$LOG_DIR/$label.log"
  local start end rc ms
  start="$(date +%s%3N)"
  echo "COMMAND[$label]: $(quote_cmd "$@")"
  set +e
  "$@" >"$log" 2>&1
  rc=$?
  set -e
  end="$(date +%s%3N)"
  ms=$((end - start))
  echo "RC[$label]: $rc"
  echo "WALL_MS[$label]: $ms"
  echo "LOG[$label]: $log"
  print_excerpt "$log"
  if [[ "$rc" -eq 0 ]]; then
    echo "expected failure for $label but command exited 0" >&2
    exit 1
  fi
}

run_logged_optional() {
  local label="$1"
  shift
  local log="$LOG_DIR/$label.log"
  local start end rc ms
  start="$(date +%s%3N)"
  echo "COMMAND[$label]: $(quote_cmd "$@")"
  set +e
  "$@" >"$log" 2>&1
  rc=$?
  set -e
  end="$(date +%s%3N)"
  ms=$((end - start))
  echo "RC[$label]: $rc"
  echo "WALL_MS[$label]: $ms"
  echo "LOG[$label]: $log"
  print_excerpt "$log"
  return "$rc"
}

blade_cmd() {
  local subcmd="$1"
  shift
  "${bwrap_base[@]}" --chdir "$PROJECT" /opt/tiger/ss_bin/blade "$subcmd" --toolchain=x86_64-gcc830 --skip-bundle "$@"
}

inside_project_cmd() {
  "${bwrap_base[@]}" --chdir "$PROJECT" "$@"
}

echo "RUN_ROOT: $RUN_ROOT"
echo "CUDA_REAL: $CUDA_REAL"
echo "CUDA_MOUNT: $CUDA_MOUNT"
echo "LOADER: $LOADER"

run_logged nvcc-visible \
  inside_project_cmd /bin/sh -c 'set -eu; ls -ld "$CUDA_PATH" "$CUDA_PATH/bin/nvcc"; "$NVCC" --version | sed -n "1,4p"; readelf -l "$NVCC" | grep "interpreter"'

run_logged cold-build \
  blade_cmd build //:cuda_app //:cuda_test //:host_test //:host_test_fail

APP="$PROJECT/build64_release/cuda_app"
TEST_BIN="$PROJECT/build64_release/cuda_test"

run_logged binary-readelf \
  inside_project_cmd /bin/sh -c 'readelf -l build64_release/cuda_app | grep "interpreter"; ldd build64_release/cuda_app'

run_logged blade-test-host-pass \
  blade_cmd test --full-test --show-details //:host_test

run_logged_expect_failure blade-test-host-fail \
  blade_cmd test --full-test --show-details //:host_test_fail

failures=0

run_logged_optional run-binary \
  inside_project_cmd "$APP" || failures=$((failures + 1))

run_logged_optional blade-test-cuda-pass \
  blade_cmd test --full-test --show-details //:cuda_test || failures=$((failures + 1))

run_logged_expect_failure direct-cuda-test-forced-fail \
  inside_project_cmd /bin/sh -c 'MINITEST_FORCE_FAIL=1 exec build64_release/cuda_test'

run_logged warm-build \
  blade_cmd build //:cuda_app //:cuda_test //:host_test //:host_test_fail

echo "DONE: $RUN_ROOT"
if [[ "$failures" -ne 0 ]]; then
  echo "GPU_RUNTIME_FAILURES: $failures" >&2
  exit 1
fi

#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
RUN_DIR="$SCRIPT_DIR/run-toolchains"
rm -rf "$RUN_DIR"
mkdir -p "$RUN_DIR/workspace" "$RUN_DIR/home"

base_args=(
  --tmpfs /
  --ro-bind /usr /usr
  --symlink usr/bin /bin
  --symlink usr/lib /lib
  --symlink usr/lib64 /lib64
  --symlink usr/sbin /sbin
  --ro-bind-try /etc/ld.so.cache /etc/ld.so.cache
  --proc /proc
  --dev /dev
  --tmpfs /tmp
  --tmpfs /run
  --tmpfs /home
  --dir /home/bench
  --bind "$RUN_DIR/workspace" /workspace
  --unshare-net
  --clearenv
  --setenv HOME /home/bench
  --setenv PATH /usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
  --setenv LC_ALL C
)

write_arg_file() {
  local path="$1"
  shift
  printf '%s\n' "$@" >"$path"
}

run_case() {
  local name="$1"
  shift
  printf '\n### %s\n' "$name"
  "$@"
  local status=$?
  printf 'exit=%s\n' "$status"
  return 0
}

write_arg_file "$SCRIPT_DIR/toolchain-base.argv.txt" "bwrap" "${base_args[@]}" "<tool-specific-binds>" "--" "<command>"

SPACK_GCC_BIN="$(command -v gcc || true)"
SPACK_GCC_ROOT=""
if [[ -n "$SPACK_GCC_BIN" && "$SPACK_GCC_BIN" == /data02/home/philip.yang/devx/spack-v1.2.2/* ]]; then
  SPACK_GCC_ROOT="$(cd -- "$(dirname -- "$SPACK_GCC_BIN")/.." && pwd)"
fi
SPACK_OPT_ROOT="/data02/home/philip.yang/devx/spack-v1.2.2/opt/spack/linux-icelake"

mkdir -p "$RUN_DIR/workspace/cargo-hello/src"
cat >"$RUN_DIR/workspace/cargo-hello/Cargo.toml" <<'EOF'
[package]
name = "cargo-hello"
version = "0.1.0"
edition = "2021"
EOF
cat >"$RUN_DIR/workspace/cargo-hello/src/main.rs" <<'EOF'
fn main() {
    println!("cargo hello");
}
EOF

if [[ -n "$SPACK_GCC_ROOT" ]]; then
  run_case cargo_hello /usr/bin/bwrap "${base_args[@]}" \
    --ro-bind /nix /nix \
    --dir /data02 \
    --dir /data02/home \
    --dir /data02/home/philip.yang \
    --dir /data02/home/philip.yang/devx \
    --dir /data02/home/philip.yang/devx/spack-v1.2.2 \
    --dir /data02/home/philip.yang/devx/spack-v1.2.2/opt \
    --dir /data02/home/philip.yang/devx/spack-v1.2.2/opt/spack \
    --ro-bind "$SPACK_OPT_ROOT" "$SPACK_OPT_ROOT" \
    --ro-bind "$HOME/.cargo" /opt/cargo-home \
    --ro-bind "$HOME/.rustup" /opt/rustup-home \
    --setenv CARGO_HOME /opt/cargo-home \
    --setenv RUSTUP_HOME /opt/rustup-home \
    --setenv CARGO_NET_OFFLINE true \
    --setenv CARGO_TARGET_DIR /workspace/cargo-target \
    --setenv CC "$SPACK_GCC_ROOT/bin/gcc" \
    --setenv CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER "$SPACK_GCC_ROOT/bin/gcc" \
    --setenv PATH "/opt/cargo-home/bin:$SPACK_GCC_ROOT/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin" \
    --chdir /workspace/cargo-hello \
    /bin/sh -lc 'cargo --version; rustc --version; cargo build --offline; /workspace/cargo-target/debug/cargo-hello'
else
  printf '\n### cargo_hello\nspack gcc not found on PATH, cargo link step skipped\nexit=127\n'
fi

mkdir -p "$RUN_DIR/workspace/uv"
cat >"$RUN_DIR/workspace/uv/hello.py" <<'EOF'
print("uv hello")
EOF
mkdir -p "$RUN_DIR/workspace/uv-cache" "$RUN_DIR/workspace/uv-bin"

run_case uv_hello /usr/bin/bwrap "${base_args[@]}" \
  --ro-bind /nix /nix \
  --dir /opt \
  --dir /opt/uv \
  --dir /opt/uv/bin \
  --ro-bind "$HOME/.local/bin/uv" /opt/uv/bin/uv \
  --setenv UV_CACHE_DIR /workspace/uv-cache \
  --setenv UV_PYTHON_DOWNLOADS never \
  --setenv UV_OFFLINE 1 \
  --setenv PATH /opt/uv/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  --chdir /workspace/uv \
  /bin/sh -lc 'uv --version; uv run --offline --no-project --python /usr/bin/python3 python hello.py'

mkdir -p "$RUN_DIR/workspace/cmake-hello"
cat >"$RUN_DIR/workspace/cmake-hello/CMakeLists.txt" <<'EOF'
cmake_minimum_required(VERSION 3.20)
project(cmake_hello C)
add_executable(cmake_hello main.c)
EOF
cat >"$RUN_DIR/workspace/cmake-hello/main.c" <<'EOF'
#include <stdio.h>
int main(void) {
  puts("cmake hello");
  return 0;
}
EOF

run_case cmake_ninja_hello /usr/bin/bwrap "${base_args[@]}" \
  --chdir /workspace/cmake-hello \
  /bin/sh -lc 'cmake -S . -B /workspace/cmake-build -G Ninja >/workspace/cmake-configure.log && ninja -C /workspace/cmake-build >/workspace/cmake-build.log && /workspace/cmake-build/cmake_hello && cmake --version | head -1 && ninja --version'

mkdir -p "$RUN_DIR/workspace/gcc"
cat >"$RUN_DIR/workspace/gcc/hello.c" <<'EOF'
#include <stdio.h>
int main(void) {
  puts("spack gcc hello");
  return 0;
}
EOF

if [[ -n "$SPACK_GCC_ROOT" ]]; then
  run_case spack_gcc_hello /usr/bin/bwrap "${base_args[@]}" \
    --ro-bind /nix /nix \
    --dir /data02 \
    --dir /data02/home \
    --dir /data02/home/philip.yang \
    --dir /data02/home/philip.yang/devx \
    --dir /data02/home/philip.yang/devx/spack-v1.2.2 \
    --dir /data02/home/philip.yang/devx/spack-v1.2.2/opt \
    --dir /data02/home/philip.yang/devx/spack-v1.2.2/opt/spack \
    --ro-bind "$SPACK_OPT_ROOT" "$SPACK_OPT_ROOT" \
    --setenv PATH "$SPACK_GCC_ROOT/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin" \
    --chdir /workspace/gcc \
    /bin/sh -lc 'gcc --version | head -1; gcc hello.c -o hello; ./hello'
else
  printf '\n### spack_gcc_hello\nspack gcc not found on PATH\nexit=127\n'
fi

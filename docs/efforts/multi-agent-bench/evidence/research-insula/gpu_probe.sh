#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
RUN_DIR="$SCRIPT_DIR/run-gpu"
WORKSPACE="$RUN_DIR/workspace"
rm -rf "$RUN_DIR"
mkdir -p "$WORKSPACE"

printf 'host_nvidia_smi:\n'
nvidia-smi -L 2>&1 | sed -n '1,20p'
printf 'host_nvidia_smi_exit=%s\n' "${PIPESTATUS[0]}"

printf '\nclean_ldconfig_driver_libs:\n'
env -i PATH=/usr/sbin:/usr/bin:/sbin:/bin /sbin/ldconfig -p 2>&1 \
  | rg 'libcuda\.so\.1|libnvidia-ml\.so\.1|libnvidia-ptxjitcompiler\.so\.1|libnvidia-nvvm\.so|libcudadebugger\.so\.1' \
  | sed -n '1,40p'

driver_args=()
while IFS= read -r path; do
  name="$(basename "$path")"
  driver_args+=(--ro-bind-try "$path" "/run/driver/lib/$name")
done < <(
  env -i PATH=/usr/sbin:/usr/bin:/sbin:/bin /sbin/ldconfig -p 2>/dev/null \
    | awk '/libcuda\.so\.1|libnvidia-ml\.so\.1|libnvidia-ptxjitcompiler\.so\.1|libnvidia-nvvm\.so|libcudadebugger\.so\.1/ {print $NF}' \
    | sort -u
)

device_args=()
for path in /dev/nvidiactl /dev/nvidia-uvm /dev/nvidia-uvm-tools /dev/nvidia-modeset /dev/nvidia[0-9]*; do
  [[ -e "$path" ]] || continue
  device_args+=(--dev-bind-try "$path" "$path")
done
if [[ -d /dev/nvidia-caps ]]; then
  device_args+=(--dev-bind-try /dev/nvidia-caps /dev/nvidia-caps)
fi

printf '\ninner_nvidia_smi:\n'
/usr/bin/bwrap \
  --tmpfs / \
  --ro-bind /usr /usr \
  --symlink usr/bin /bin \
  --symlink usr/lib /lib \
  --symlink usr/lib64 /lib64 \
  --symlink usr/sbin /sbin \
  --dir /run \
  --dir /run/driver \
  --dir /run/driver/bin \
  --dir /run/driver/lib \
  --ro-bind /usr/bin/nvidia-smi /run/driver/bin/nvidia-smi \
  "${driver_args[@]}" \
  --proc /proc \
  --dev /dev \
  "${device_args[@]}" \
  --tmpfs /tmp \
  --bind "$WORKSPACE" /workspace \
  --clearenv \
  --setenv PATH /run/driver/bin:/usr/bin:/bin:/usr/sbin:/sbin \
  --setenv LD_LIBRARY_PATH /run/driver/lib \
  --setenv NVIDIA_VISIBLE_DEVICES all \
  --chdir /workspace \
  /bin/sh -lc 'nvidia-smi -L; printf "exit_inside=%s\n" "$?"' 2>&1 | sed -n '1,80p'
printf 'inner_pipeline_exit=%s\n' "${PIPESTATUS[0]}"

if [[ -x /usr/local/cuda/bin/nvcc || -x /data00/home/zfc/cuda_13_2/bin/nvcc ]]; then
  cat >"$WORKSPACE/hello.cu" <<'EOF'
#include <cuda_runtime.h>
#include <stdio.h>
int main(void) {
  int count = -1;
  cudaError_t err = cudaGetDeviceCount(&count);
  printf("cudaGetDeviceCount=%d err=%d %s\n", count, (int)err, cudaGetErrorString(err));
  return err == cudaSuccess ? 0 : 1;
}
EOF
  CUDA_ROOT="$(readlink -f /usr/local/cuda)"
  printf '\ninner_nvcc_cuda_runtime:\n'
  /usr/bin/bwrap \
    --tmpfs / \
    --ro-bind /usr /usr \
    --symlink usr/bin /bin \
    --symlink usr/lib /lib \
    --symlink usr/lib64 /lib64 \
    --symlink usr/sbin /sbin \
    --dir /opt \
    --ro-bind "$CUDA_ROOT" /opt/cuda \
    --dir /run \
    --dir /run/driver \
    --dir /run/driver/lib \
    "${driver_args[@]}" \
    --proc /proc \
    --dev /dev \
    "${device_args[@]}" \
    --tmpfs /tmp \
    --bind "$WORKSPACE" /workspace \
    --clearenv \
    --setenv PATH /opt/cuda/bin:/usr/bin:/bin:/usr/sbin:/sbin \
    --setenv LD_LIBRARY_PATH /run/driver/lib:/opt/cuda/lib64 \
    --setenv CUDA_HOME /opt/cuda \
    --chdir /workspace \
    /bin/sh -lc 'nvcc --version | tail -1; nvcc hello.cu -o hello_cuda; ./hello_cuda; printf "cuda_exit=%s\n" "$?"' 2>&1 | sed -n '1,120p'
  printf 'cuda_pipeline_exit=%s\n' "${PIPESTATUS[0]}"
else
  printf '\ninner_nvcc_cuda_runtime:\nnvcc not found\n'
fi

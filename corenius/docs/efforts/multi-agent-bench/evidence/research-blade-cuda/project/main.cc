#include "kernel.h"

#include <cuda_runtime.h>

#include <cstdio>

int main() {
  int devices = blade_cuda_device_count();
  if (devices <= 0) {
    std::printf("BLADE_CUDA_RESULT device_count=%d\n", devices);
    return 2;
  }

  cudaDeviceProp prop;
  cudaError_t err = cudaGetDeviceProperties(&prop, 0);
  if (err != cudaSuccess) {
    std::printf("BLADE_CUDA_RESULT device_properties=%s\n", cudaGetErrorString(err));
    return 3;
  }

  int answer = blade_cuda_answer(41);
  std::printf("BLADE_CUDA_RESULT answer=%d devices=%d device0=%s sm=%d%d\n",
              answer,
              devices,
              prop.name,
              prop.major,
              prop.minor);
  return answer == 42 ? 0 : 4;
}

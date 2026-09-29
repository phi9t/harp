#include "kernel.h"

#include <cuda_runtime.h>

#include <cstdio>

namespace {

__global__ void add_one_kernel(int *value) {
  value[0] += 1;
}

int fail(const char *step, cudaError_t err) {
  std::fprintf(stderr, "CUDA_ERROR %s %s\n", step, cudaGetErrorString(err));
  return -1000 - static_cast<int>(err);
}

}  // namespace

int blade_cuda_device_count() {
  int count = 0;
  cudaError_t err = cudaGetDeviceCount(&count);
  if (err != cudaSuccess) {
    return fail("cudaGetDeviceCount", err);
  }
  return count;
}

int blade_cuda_answer(int input) {
  int *device_value = nullptr;
  cudaError_t err = cudaMalloc(&device_value, sizeof(int));
  if (err != cudaSuccess) {
    return fail("cudaMalloc", err);
  }

  err = cudaMemcpy(device_value, &input, sizeof(int), cudaMemcpyHostToDevice);
  if (err != cudaSuccess) {
    cudaFree(device_value);
    return fail("cudaMemcpyHostToDevice", err);
  }

  add_one_kernel<<<1, 1>>>(device_value);
  err = cudaGetLastError();
  if (err != cudaSuccess) {
    cudaFree(device_value);
    return fail("kernel_launch", err);
  }

  err = cudaDeviceSynchronize();
  if (err != cudaSuccess) {
    cudaFree(device_value);
    return fail("cudaDeviceSynchronize", err);
  }

  int output = 0;
  err = cudaMemcpy(&output, device_value, sizeof(int), cudaMemcpyDeviceToHost);
  cudaFree(device_value);
  if (err != cudaSuccess) {
    return fail("cudaMemcpyDeviceToHost", err);
  }
  return output;
}

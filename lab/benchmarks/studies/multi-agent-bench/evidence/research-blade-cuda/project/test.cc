#include "kernel.h"
#include "minitest.h"

#include <cstdlib>

int main() {
  minitest::expect_true("device_count_positive", blade_cuda_device_count() > 0);
  minitest::expect_eq("kernel_add_one", 42, blade_cuda_answer(41));
  if (std::getenv("MINITEST_FORCE_FAIL") != nullptr) {
    minitest::expect_true("forced_failure", false);
  }
  return minitest::summary();
}

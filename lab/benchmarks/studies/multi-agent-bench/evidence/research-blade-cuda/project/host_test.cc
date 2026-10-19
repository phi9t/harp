#include "minitest.h"

int main() {
  minitest::expect_eq("host_arithmetic", 7, 3 + 4);
  minitest::expect_true("host_boolean", true);
  return minitest::summary();
}

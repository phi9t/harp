#include "minitest.h"

int main() {
  minitest::expect_eq("host_arithmetic_forced_fail", 8, 3 + 4);
  return minitest::summary();
}

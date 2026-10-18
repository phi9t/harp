#include "minitest.h"

#include <cstdio>

namespace minitest {
namespace {

int failures = 0;

}  // namespace

void expect_true(const char *name, bool value) {
  if (value) {
    std::printf("[minitest] PASS %s\n", name);
    return;
  }
  std::printf("[minitest] FAIL %s\n", name);
  ++failures;
}

void expect_eq(const char *name, int expected, int actual) {
  if (expected == actual) {
    std::printf("[minitest] PASS %s expected=%d actual=%d\n", name, expected, actual);
    return;
  }
  std::printf("[minitest] FAIL %s expected=%d actual=%d\n", name, expected, actual);
  ++failures;
}

int summary() {
  std::printf("[minitest] failures=%d\n", failures);
  return failures == 0 ? 0 : 1;
}

}  // namespace minitest

#pragma once

namespace minitest {

void expect_true(const char *name, bool value);
void expect_eq(const char *name, int expected, int actual);
int summary();

}  // namespace minitest

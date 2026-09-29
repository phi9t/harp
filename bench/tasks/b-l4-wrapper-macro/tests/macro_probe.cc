#include "tests/hidden_part.h"

#include <cstdio>

int main() {
  int value = hidden_part_value();
  std::printf("WRAPPER_HIDDEN value=%d\n", value);
  return value == 110 ? 0 : 1;
}

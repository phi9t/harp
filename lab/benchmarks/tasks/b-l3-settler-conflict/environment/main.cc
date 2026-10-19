#include "lib/chooser.h"

#include <cstdio>

int main() {
  int value = chooser_value();
  std::printf("SETTLER_RESULT value=%d\n", value);
  return value == 7 ? 0 : 1;
}

#include "feature/p0/part0.h"
#include "feature/p1/part1.h"
#include "feature/p2/part2.h"
#include "feature/p3/part3.h"
#include "feature/p4/part4.h"
#include "feature/p5/part5.h"

#include <cstdio>

int main() {
  int total = part0_value() + part1_value() + part2_value() +
              part3_value() + part4_value() + part5_value();
  std::printf("WRAPPER_RESULT total=%d\n", total);
  return total == 75 ? 0 : 1;
}

#include "matrix_api.h"

#include <cstdio>

int main() {
#ifdef FEATURE_MODE_WIDE
  const char* mode = "wide";
  int expected = 64;
#else
  const char* mode = "compact";
  int expected = 32;
#endif

  int value = matrix_value();
  std::printf("MATRIX_RESULT mode=%s value=%d\n", mode, value);
  return value == expected ? 0 : 1;
}

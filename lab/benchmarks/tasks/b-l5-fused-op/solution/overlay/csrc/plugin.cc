#include "fused.h"

extern "C" int fused_op_apply_v2(const Params *params, const float *x, float *y, int n) {
  if (!params || (!x && n > 0) || (!y && n > 0) || n < 0) {
    return 1;
  }
  fused_scale_add(params, x, y, n);
  return 0;
}

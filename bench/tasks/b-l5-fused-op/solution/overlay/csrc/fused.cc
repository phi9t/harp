#include "fused.h"

void fused_scale_add(const Params *params, const float *x, float *y, int n) {
  if (!params || !x || !y || n <= 0) {
    return;
  }
  for (int i = 0; i < n; ++i) {
    y[i] = x[i] * params->scale + params->bias;
  }
}

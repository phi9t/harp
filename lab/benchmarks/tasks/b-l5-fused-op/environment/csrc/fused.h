#pragma once

#include "gen/params.h"

#ifdef __cplusplus
extern "C" {
#endif

void fused_scale_add(const Params *params, const float *x, float *y, int n);

#ifdef __cplusplus
}
#endif

#include "beta.h"

#ifndef BLADE_COMPONENT_BETA
#error "beta must use blade_component"
#endif

#ifndef BETA_OFFSET
#define BETA_OFFSET 0
#endif

int beta_value() {
    return 4 + BETA_OFFSET;
}

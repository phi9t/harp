#include "demo.h"

int demo_value() {
#ifdef BENCH_USER_BENCH
    return 11;
#else
    return 0;
#endif
}

#include "pinmsg.h"

#include <cstdio>

int main() {
    std::printf("BLADE_L1_PIN %s score=%d\n", pinmsg_v2_label(), pinmsg_v2_score(21));
    return 0;
}

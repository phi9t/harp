#include "pinmsg.h"

const char *pinmsg_v2_label() {
    return "bumped=v2";
}

int pinmsg_v2_score(int base) {
    return base * 2;
}

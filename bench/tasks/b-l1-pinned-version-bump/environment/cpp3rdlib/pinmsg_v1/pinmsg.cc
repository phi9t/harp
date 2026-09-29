#include "pinmsg.h"

const char *pinmsg_label() {
    return "old=v1";
}

int pinmsg_score(int base) {
    return base + 1;
}

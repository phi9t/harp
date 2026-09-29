#include "window.hpp"

#include <cassert>
#include <cmath>
#include <vector>

int main() {
    std::vector<double> xs{2.0, 4.0, 6.0, 8.0};
    assert(std::fabs(mean_window(xs, 1, 3) - 6.0) < 1e-9);
    assert(std::fabs(mean_window(xs, 0, 0) - 2.0) < 1e-9);
    return 0;
}

#include "report.hpp"

#include <cassert>
#include <vector>

int main() {
    assert(render_report({3, 4, 5}) == "items=3 total=12 average=4.00");
    assert(render_report({-5, 9}) == "items=2 total=4 average=2.00");
    assert(render_report({}) == "items=0 total=0 average=0.00");
    return 0;
}

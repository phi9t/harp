#include "vector_stats.hpp"

#include <cassert>
#include <vector>

int main() {
    std::vector<int> values{1, 7, 7, 2, 9};
    values.shrink_to_fit();
    assert(count_above_limit(values, 6) == 3);

    std::vector<int> edge{5};
    edge.shrink_to_fit();
    assert(count_above_limit(edge, 5) == 0);

    std::vector<int> empty;
    assert(count_above_limit(empty, 0) == 0);
    return 0;
}

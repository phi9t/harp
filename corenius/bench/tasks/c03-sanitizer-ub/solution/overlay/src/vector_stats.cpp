#include "vector_stats.hpp"

int count_above_limit(const std::vector<int>& values, int limit) {
    int count = 0;
    for (std::size_t i = 0; i < values.size(); ++i) {
        if (values[i] > limit) {
            ++count;
        }
    }
    return count;
}

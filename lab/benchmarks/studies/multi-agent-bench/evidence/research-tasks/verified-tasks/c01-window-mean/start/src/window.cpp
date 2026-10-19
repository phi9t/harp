#include "window.hpp"

double mean_window(const std::vector<double>& xs, int start, int end) {
    double sum = 0.0;
    int count = 0;
    for (int i = start; i < end; ++i) {
        sum += xs.at(i);
        ++count;
    }
    return sum / count;
}

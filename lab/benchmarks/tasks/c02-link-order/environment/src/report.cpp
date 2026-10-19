#include "report.hpp"

#include "totals.hpp"

#include <iomanip>
#include <sstream>

std::string render_report(const std::vector<int>& values) {
    int total = total_values(values);
    double average = values.empty() ? 0.0 : static_cast<double>(total) / values.size();

    std::ostringstream out;
    out << "items=" << values.size()
        << " total=" << total
        << " average=" << std::fixed << std::setprecision(2) << average;
    return out.str();
}

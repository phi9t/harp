#include "status_table.hpp"

#include <array>

namespace {

const std::array<StatusInfo, 6> kStatuses{{
    {"queued", "Queued", 1000, false},
    {"running", "Running", 0, false},
    {"throttled", "Throttled", 30000, false},
    {"succeeded", "Succeeded", 0, true},
    {"failed", "Failed", 0, true},
    {"cancelled", "Cancelled", 0, true},
}};

}  // namespace

StatusInfo lookup_status(std::string_view code) {
    for (const auto& status : kStatuses) {
        if (status.code == code) {
            return status;
        }
    }
    return {"unknown", "Unknown", 0, true};
}

std::vector<std::string> retryable_codes() {
    std::vector<std::string> codes;
    for (const auto& status : kStatuses) {
        if (!status.terminal && status.retry_delay_ms > 0) {
            codes.push_back(status.code);
        }
    }
    return codes;
}

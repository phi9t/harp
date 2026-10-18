#include "audit_log.hpp"

#include <vector>

namespace audit {
namespace {
std::vector<std::string> events;
}

void clear_events() {
    events.clear();
}

void record_event(const std::string& event) {
    events.push_back(event);
}

std::size_t event_count() {
    return events.size();
}

std::string last_event() {
    return events.empty() ? "" : events.back();
}

std::string summary() {
    return "count=" + std::to_string(event_count()) + " last=" + last_event();
}

}  // namespace audit

#pragma once

#include <cstddef>
#include <string>
#include <vector>

namespace audit {

static std::vector<std::string> events;

inline void clear_events() {
    events.clear();
}

inline void record_event(const std::string& event) {
    events.push_back(event);
}

inline std::size_t event_count() {
    return events.size();
}

inline std::string last_event() {
    return events.empty() ? "" : events.back();
}

inline std::string summary() {
    return "count=" + std::to_string(event_count()) + " last=" + last_event();
}

}  // namespace audit

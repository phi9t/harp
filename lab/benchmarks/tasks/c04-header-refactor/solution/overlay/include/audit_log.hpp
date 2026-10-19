#pragma once

#include <cstddef>
#include <string>

namespace audit {

void clear_events();
void record_event(const std::string& event);
std::size_t event_count();
std::string last_event();
std::string summary();

}  // namespace audit

#pragma once

#include <string>
#include <string_view>
#include <vector>

struct StatusInfo {
    std::string code;
    std::string label;
    int retry_delay_ms;
    bool terminal;
};

StatusInfo lookup_status(std::string_view code);
std::vector<std::string> retryable_codes();

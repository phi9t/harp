#include "status_table.hpp"

#include <cassert>
#include <string>
#include <vector>

int main() {
    StatusInfo queued = lookup_status("queued");
    assert(queued.label == "Queued");
    assert(queued.retry_delay_ms == 1000);
    assert(!queued.terminal);

    StatusInfo throttled = lookup_status("throttled");
    assert(throttled.code == "throttled");
    assert(throttled.label == "Throttled");
    assert(throttled.retry_delay_ms == 30000);
    assert(!throttled.terminal);

    StatusInfo cancelled = lookup_status("cancelled");
    assert(cancelled.label == "Cancelled");
    assert(cancelled.terminal);

    StatusInfo missing = lookup_status("paused");
    assert(missing.label == "Unknown");
    assert(missing.retry_delay_ms == 0);
    assert(missing.terminal);

    std::vector<std::string> retryable = retryable_codes();
    assert((retryable == std::vector<std::string>{"queued", "throttled"}));
    return 0;
}

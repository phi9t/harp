#include "session.hpp"

#include "audit_log.hpp"

void reset_session() {
    audit::clear_events();
}

void start_session() {
    audit::clear_events();
    audit::record_event("boot");
    audit::record_event("ready");
}

void append_session_event(const std::string& event) {
    audit::record_event(event);
}

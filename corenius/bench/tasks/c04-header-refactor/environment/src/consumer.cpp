#include "session.hpp"

#include "audit_log.hpp"

std::string session_summary() {
    return audit::summary();
}

#include "session.hpp"

#include <cassert>

int main() {
    reset_session();
    start_session();
    assert(session_summary() == "count=2 last=ready");

    append_session_event("verified");
    assert(session_summary() == "count=3 last=verified");

    reset_session();
    assert(session_summary() == "count=0 last=");
    return 0;
}

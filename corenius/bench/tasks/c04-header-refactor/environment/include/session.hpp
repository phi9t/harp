#pragma once

#include <string>

void reset_session();
void start_session();
void append_session_event(const std::string& event);
std::string session_summary();

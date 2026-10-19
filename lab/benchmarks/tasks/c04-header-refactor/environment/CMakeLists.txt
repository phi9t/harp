cmake_minimum_required(VERSION 3.16)
project(c04_header_refactor LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(audit_session
    src/producer.cpp
    src/consumer.cpp
)
target_include_directories(audit_session PUBLIC include)

add_executable(audit_log_test tests/audit_log_test.cpp)
target_link_libraries(audit_log_test PRIVATE audit_session)

enable_testing()
add_test(NAME audit_log_test COMMAND audit_log_test)

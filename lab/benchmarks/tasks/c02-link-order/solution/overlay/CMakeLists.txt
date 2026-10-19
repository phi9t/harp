cmake_minimum_required(VERSION 3.16)
project(c02_link_order LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(totals STATIC src/totals.cpp)
target_include_directories(totals PUBLIC include)

add_library(report STATIC src/report.cpp)
target_include_directories(report PUBLIC include)
target_link_libraries(report PUBLIC totals)

add_executable(report_test tests/report_test.cpp)
target_link_libraries(report_test PRIVATE report)

enable_testing()
add_test(NAME report_test COMMAND report_test)

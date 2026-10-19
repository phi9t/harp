cmake_minimum_required(VERSION 3.16)
project(c05_generated_table LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(status_table src/status_table.cpp)
target_include_directories(status_table PUBLIC include)

add_executable(status_table_test tests/status_table_test.cpp)
target_link_libraries(status_table_test PRIVATE status_table)

enable_testing()
add_test(NAME status_table_test COMMAND status_table_test)

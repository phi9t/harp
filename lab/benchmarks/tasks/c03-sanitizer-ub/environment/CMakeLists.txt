cmake_minimum_required(VERSION 3.16)
project(c03_sanitizer_ub LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

option(ENABLE_ASAN "Enable address and undefined behavior sanitizers" OFF)
if(ENABLE_ASAN)
    add_compile_options(-fsanitize=address,undefined -fno-omit-frame-pointer -g)
    add_link_options(-fsanitize=address,undefined)
endif()

add_library(vector_stats src/vector_stats.cpp)
target_include_directories(vector_stats PUBLIC include)

add_executable(vector_stats_test tests/vector_stats_test.cpp)
target_link_libraries(vector_stats_test PRIVATE vector_stats)

enable_testing()
add_test(NAME vector_stats_test COMMAND vector_stats_test)

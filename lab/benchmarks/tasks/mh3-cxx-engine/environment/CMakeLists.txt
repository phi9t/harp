cmake_minimum_required(VERSION 3.16)
project(mh3_cxx_engine LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(rules_engine
    src/driver.cpp
)
target_include_directories(rules_engine PUBLIC include)

add_executable(rules_engine_test tests/rules_engine_test.cpp)
target_link_libraries(rules_engine_test PRIVATE rules_engine)

enable_testing()
add_test(NAME rules_engine_test COMMAND rules_engine_test)

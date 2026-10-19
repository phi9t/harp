#include "tokenizer.cpp"
#include "expression.cpp"
#include "rules.cpp"
#include "loader.cpp"

#include "io.hpp"

namespace mh3 {

Result<Context> parse_context(std::string_view) {
    return Result<Context>::success(Context{});
}

Result<std::string> run_script(std::string_view, std::string_view) {
    return Result<std::string>::success("NO_MATCH");
}

}  // namespace mh3

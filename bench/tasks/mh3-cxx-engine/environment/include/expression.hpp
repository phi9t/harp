#pragma once

#include "diagnostic.hpp"
#include "value.hpp"

#include <string_view>

namespace mh3 {

Result<Value> evaluate_expression(std::string_view expression, const Context& context);

}  // namespace mh3

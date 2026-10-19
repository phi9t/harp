#pragma once

#include "diagnostic.hpp"
#include "rules.hpp"
#include "value.hpp"

#include <string>
#include <string_view>

namespace mh3 {

Result<RuleStore> load_rules(std::string_view text);
Result<Context> parse_context(std::string_view text);
Result<std::string> run_script(std::string_view rules_text, std::string_view context_text);

}  // namespace mh3

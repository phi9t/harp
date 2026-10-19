#include "io.hpp"

namespace mh3 {

Result<RuleStore> load_rules(std::string_view) {
    return Result<RuleStore>::success(RuleStore{});
}

}  // namespace mh3

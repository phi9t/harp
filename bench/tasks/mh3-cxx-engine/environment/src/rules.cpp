#include "rules.hpp"

namespace mh3 {

Result<bool> RuleStore::add_rule(Rule rule) {
    rules_.push_back(std::move(rule));
    return Result<bool>::success(true);
}

std::vector<Rule> RuleStore::ordered_rules() const {
    return rules_;
}

Result<RuleDecision> RuleStore::decide(const Context&) const {
    return Result<RuleDecision>::success(RuleDecision{});
}

}  // namespace mh3

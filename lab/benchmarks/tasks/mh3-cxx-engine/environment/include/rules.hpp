#pragma once

#include "diagnostic.hpp"
#include "value.hpp"

#include <cstddef>
#include <string>
#include <vector>

namespace mh3 {

struct Rule {
    std::string name;
    int priority = 0;
    std::string condition;
    std::string action;
    std::size_t source_offset = 0;
};

struct RuleDecision {
    bool matched = false;
    std::string rule_name;
    std::string action;
    int priority = 0;
};

class RuleStore {
public:
    Result<bool> add_rule(Rule rule);
    std::vector<Rule> ordered_rules() const;
    Result<RuleDecision> decide(const Context& context) const;

private:
    std::vector<Rule> rules_;
};

}  // namespace mh3

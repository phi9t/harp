#include "rules.hpp"

#include "expression.hpp"

#include <algorithm>
#include <cctype>

namespace mh3 {
namespace {

bool valid_name(const std::string& name) {
    if (name.empty()) return false;
    unsigned char first = static_cast<unsigned char>(name[0]);
    if (!std::isalpha(first) && name[0] != '_') return false;
    for (char c : name) {
        unsigned char ch = static_cast<unsigned char>(c);
        if (!std::isalnum(ch) && c != '_' && c != '-') return false;
    }
    return true;
}

}  // namespace

Result<bool> RuleStore::add_rule(Rule rule) {
    if (!valid_name(rule.name)) {
        return Result<bool>::failure(diagnostic("E_RULE", rule.source_offset, "invalid rule name '" + rule.name + "'"));
    }
    if (rule.condition.empty()) {
        return Result<bool>::failure(diagnostic("E_RULE", rule.source_offset, "empty condition"));
    }
    for (const Rule& existing : rules_) {
        if (existing.name == rule.name) {
            return Result<bool>::failure(diagnostic("E_RULE", rule.source_offset, "duplicate rule '" + rule.name + "'"));
        }
    }
    rules_.push_back(std::move(rule));
    return Result<bool>::success(true);
}

std::vector<Rule> RuleStore::ordered_rules() const {
    std::vector<Rule> ordered = rules_;
    std::sort(ordered.begin(), ordered.end(), [](const Rule& a, const Rule& b) {
        if (a.priority != b.priority) return a.priority > b.priority;
        return a.name < b.name;
    });
    return ordered;
}

Result<RuleDecision> RuleStore::decide(const Context& context) const {
    for (const Rule& rule : ordered_rules()) {
        auto value = evaluate_expression(rule.condition, context);
        if (!value.ok()) return Result<RuleDecision>::failure(value.error());
        if (value.value().kind != Value::Kind::Boolean) {
            return Result<RuleDecision>::failure(
                diagnostic("E_RULE", rule.source_offset, "condition for '" + rule.name + "' is not boolean"));
        }
        if (value.value().boolean) {
            return Result<RuleDecision>::success(RuleDecision{true, rule.name, rule.action, rule.priority});
        }
    }
    return Result<RuleDecision>::success(RuleDecision{false, "", "", 0});
}

}  // namespace mh3

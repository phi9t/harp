#include "rules_engine.hpp"

#include <cstdlib>
#include <iostream>
#include <string>
#include <vector>

using namespace mh3;

static void require(bool condition, const std::string& message) {
    if (!condition) {
        std::cerr << message << "\n";
        std::exit(1);
    }
}

static void test_tokenizer_offsets_and_strings() {
    auto tokens = tokenize("tier == \"gold,plus\" and spend >= 10 / 3 # ignored");
    require(tokens.ok(), tokens.ok() ? "" : tokens.error().format());
    require(tokens.value().size() == 9, "tokenizer should skip comments and produce 9 tokens");
    require(tokens.value()[0].kind == TokenKind::Identifier, "first token kind");
    require(tokens.value()[2].kind == TokenKind::String, "string token kind");
    require(tokens.value()[2].text == "gold,plus", "quoted string should keep comma");
    require(tokens.value()[2].offset == 8, "string token offset should point at opening quote");
    require(tokens.value()[3].kind == TokenKind::And, "and keyword should be a logical token");

    auto bad = tokenize("ok @ no");
    require(!bad.ok(), "bad token should fail");
    require(bad.error().code == "E_LEX", "bad token code");
    require(bad.error().offset == 3, "bad token offset");
    require(bad.error().format() == "E_LEX@3: unexpected character '@'", "diagnostic format");
}

static void test_expression_numeric_semantics() {
    Context ctx{
        {"tier", Value::string_value("gold,plus")},
        {"spend", Value::number_value(12.0)},
        {"enabled", Value::bool_value(true)},
    };

    auto rounded = evaluate_expression("10 / 3 * 3", ctx);
    require(rounded.ok(), rounded.ok() ? "" : rounded.error().format());
    require(render_value(rounded.value()) == "9.999", "arithmetic must round after each operation");

    auto match = evaluate_expression("tier == \"gold,plus\" and spend >= 10 / 3 * 3 and enabled", ctx);
    require(match.ok(), match.ok() ? "" : match.error().format());
    require(match.value().kind == Value::Kind::Boolean && match.value().boolean, "compound expression");

    auto missing = evaluate_expression("unknown + 1", ctx);
    require(!missing.ok(), "unknown identifiers should be diagnostic failures");
    require(missing.error().code == "E_EVAL", "unknown identifier code");

    auto zero = evaluate_expression("1 / (2 - 2)", ctx);
    require(!zero.ok(), "division by zero should fail");
    require(zero.error().code == "E_EVAL", "division by zero code");
}

static void test_rule_store_precedence_and_manifest() {
    RuleStore store;
    require(store.add_rule(Rule{"zeta", 10, "true", "z", 4}).ok(), "add zeta");
    require(store.add_rule(Rule{"alpha", 10, "true", "a", 8}).ok(), "add alpha");
    require(store.add_rule(Rule{"beta", 20, "false", "b", 12}).ok(), "add beta");

    auto ordered = store.ordered_rules();
    require(ordered.size() == 3, "ordered rule count");
    require(ordered[0].name == "beta", "higher priority should sort first");
    require(ordered[1].name == "alpha", "priority ties sort by name");
    require(ordered[2].name == "zeta", "priority tie order");

    auto decision = store.decide(Context{});
    require(decision.ok(), decision.ok() ? "" : decision.error().format());
    require(decision.value().matched, "decision should match");
    require(decision.value().rule_name == "alpha", "first true canonical tie should win");
    require(decision.value().action == "a", "decision action");

    auto dup = store.add_rule(Rule{"alpha", 5, "true", "dup", 20});
    require(!dup.ok(), "duplicate rule names should fail");
    require(dup.error().code == "E_RULE", "duplicate code");

    std::vector<std::string> expected{"tokenizer", "expression", "rule-store", "driver"};
    require(feature_manifest() == expected, "facade feature manifest should be canonical");
}

static void test_loader_quotes_and_errors() {
    const std::string text =
        "# rules\n"
        "name,priority,condition,action\n"
        "\"gold_plus\", 10, \"tier == \"\"gold,plus\"\" and spend >= 10 / 3 * 3\", \"approve,priority\"\n"
        "fallback, 1, true, review\n";
    auto loaded = load_rules(text);
    require(loaded.ok(), loaded.ok() ? "" : loaded.error().format());

    Context ctx{
        {"tier", Value::string_value("gold,plus")},
        {"spend", Value::number_value(10.0)},
    };
    auto decision = loaded.value().decide(ctx);
    require(decision.ok(), decision.ok() ? "" : decision.error().format());
    require(decision.value().rule_name == "gold_plus", "quoted CSV condition should load");
    require(decision.value().action == "approve,priority", "quoted CSV action should keep comma");

    auto bad = load_rules("name,priority,condition,action\nbad, high, true, action\n");
    require(!bad.ok(), "bad priority should fail");
    require(bad.error().code == "E_LOAD", "bad priority code");
    require(bad.error().format().find("E_LOAD@") == 0, "load error should format with offset");
}

static void test_driver_integration() {
    const std::string rules =
        "name,priority,condition,action\n"
        "\"gold_plus\", 10, \"tier == \"\"gold,plus\"\" and spend >= 10 / 3 * 3\", \"approve,priority\"\n"
        "fallback, 1, true, review\n";
    auto run = run_script(rules, "tier=\"gold,plus\"; spend=10\n");
    require(run.ok(), run.ok() ? "" : run.error().format());
    require(run.value() == "approve,priority", "driver should select loaded rule");

    auto no_match = run_script("name,priority,condition,action\nnever,1,false,stop\n", "tier=\"silver\"");
    require(no_match.ok(), no_match.ok() ? "" : no_match.error().format());
    require(no_match.value() == "NO_MATCH", "driver should report no match without throwing");

    auto bad_context = run_script(rules, "tier=\"gold,plus\"; spend=(1 + )");
    require(!bad_context.ok(), "bad context should fail");
    require(bad_context.error().code == "E_DRIVER", "bad context code");
}

int main() {
    test_tokenizer_offsets_and_strings();
    test_expression_numeric_semantics();
    test_rule_store_precedence_and_manifest();
    test_loader_quotes_and_errors();
    test_driver_integration();
    return 0;
}

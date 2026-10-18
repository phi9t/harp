#include "tokenizer.cpp"
#include "expression.cpp"
#include "rules.cpp"
#include "loader.cpp"

#include "io.hpp"

#include "expression.hpp"

#include <cctype>
#include <string>
#include <vector>

namespace mh3 {
namespace {

struct Part {
    std::string text;
    std::size_t offset = 0;
};

std::string trim_context(std::string text) {
    std::size_t first = 0;
    while (first < text.size() && std::isspace(static_cast<unsigned char>(text[first]))) ++first;
    std::size_t last = text.size();
    while (last > first && std::isspace(static_cast<unsigned char>(text[last - 1]))) --last;
    return text.substr(first, last - first);
}

bool valid_key(const std::string& key) {
    if (key.empty()) return false;
    if (!std::isalpha(static_cast<unsigned char>(key[0])) && key[0] != '_') return false;
    for (char c : key) {
        if (!std::isalnum(static_cast<unsigned char>(c)) && c != '_') return false;
    }
    return true;
}

std::vector<Part> split_context(std::string_view text) {
    std::vector<Part> parts;
    std::string current;
    std::size_t start = 0;
    bool quoted = false;
    char quote = 0;
    for (std::size_t i = 0; i < text.size(); ++i) {
        char c = text[i];
        if (quoted) {
            current.push_back(c);
            if (c == '\\' && i + 1 < text.size()) {
                current.push_back(text[++i]);
            } else if (c == quote) {
                quoted = false;
            }
        } else if (c == '"' || c == '\'') {
            quoted = true;
            quote = c;
            current.push_back(c);
        } else if (c == ';' || c == '\n') {
            if (!trim_context(current).empty()) parts.push_back(Part{trim_context(current), start});
            current.clear();
            start = i + 1;
        } else {
            current.push_back(c);
        }
    }
    if (!trim_context(current).empty()) parts.push_back(Part{trim_context(current), start});
    return parts;
}

std::size_t find_equals(const std::string& text) {
    bool quoted = false;
    char quote = 0;
    for (std::size_t i = 0; i < text.size(); ++i) {
        char c = text[i];
        if (quoted) {
            if (c == '\\' && i + 1 < text.size()) ++i;
            else if (c == quote) quoted = false;
        } else if (c == '"' || c == '\'') {
            quoted = true;
            quote = c;
        } else if (c == '=') {
            return i;
        }
    }
    return std::string::npos;
}

}  // namespace

Result<Context> parse_context(std::string_view text) {
    Context context;
    for (const Part& part : split_context(text)) {
        std::size_t eq = find_equals(part.text);
        if (eq == std::string::npos) {
            return Result<Context>::failure(diagnostic("E_DRIVER", part.offset, "expected key=value"));
        }
        std::string key = trim_context(part.text.substr(0, eq));
        std::string expr = trim_context(part.text.substr(eq + 1));
        if (!valid_key(key)) {
            return Result<Context>::failure(diagnostic("E_DRIVER", part.offset, "invalid context key '" + key + "'"));
        }
        auto value = evaluate_expression(expr, Context{});
        if (!value.ok()) {
            return Result<Context>::failure(
                diagnostic("E_DRIVER", part.offset + eq + 1 + value.error().offset, value.error().message));
        }
        context[key] = value.value();
    }
    return Result<Context>::success(context);
}

Result<std::string> run_script(std::string_view rules_text, std::string_view context_text) {
    auto rules = load_rules(rules_text);
    if (!rules.ok()) return Result<std::string>::failure(rules.error());
    auto context = parse_context(context_text);
    if (!context.ok()) return Result<std::string>::failure(context.error());
    auto decision = rules.value().decide(context.value());
    if (!decision.ok()) return Result<std::string>::failure(decision.error());
    if (!decision.value().matched) return Result<std::string>::success("NO_MATCH");
    return Result<std::string>::success(decision.value().action);
}

}  // namespace mh3

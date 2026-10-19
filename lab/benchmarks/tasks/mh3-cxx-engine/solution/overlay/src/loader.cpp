#include "io.hpp"

#include <cctype>
#include <string>
#include <vector>

namespace mh3 {
namespace {

std::string trim_csv(std::string text) {
    std::size_t first = 0;
    while (first < text.size() && std::isspace(static_cast<unsigned char>(text[first]))) ++first;
    std::size_t last = text.size();
    while (last > first && std::isspace(static_cast<unsigned char>(text[last - 1]))) --last;
    return text.substr(first, last - first);
}

std::string lower_header(std::string text) {
    for (char& c : text) c = static_cast<char>(std::tolower(static_cast<unsigned char>(c)));
    return text;
}

Result<std::vector<std::string>> parse_csv_line(std::string_view line, std::size_t offset) {
    std::vector<std::string> fields;
    std::string current;
    bool quoted = false;
    for (std::size_t i = 0; i < line.size(); ++i) {
        char c = line[i];
        if (quoted) {
            if (c == '"' && i + 1 < line.size() && line[i + 1] == '"') {
                current.push_back('"');
                ++i;
            } else if (c == '"') {
                quoted = false;
            } else {
                current.push_back(c);
            }
        } else if (c == ',') {
            fields.push_back(trim_csv(current));
            current.clear();
        } else if (c == '"' && trim_csv(current).empty()) {
            quoted = true;
            current.clear();
        } else {
            current.push_back(c);
        }
    }
    if (quoted) {
        return Result<std::vector<std::string>>::failure(diagnostic("E_LOAD", offset, "unterminated quoted field"));
    }
    fields.push_back(trim_csv(current));
    return Result<std::vector<std::string>>::success(fields);
}

bool parse_int(const std::string& text, int& out) {
    if (text.empty()) return false;
    int sign = 1;
    std::size_t i = 0;
    if (text[i] == '-' || text[i] == '+') {
        sign = text[i] == '-' ? -1 : 1;
        ++i;
    }
    if (i == text.size()) return false;
    long long value = 0;
    for (; i < text.size(); ++i) {
        if (!std::isdigit(static_cast<unsigned char>(text[i]))) return false;
        value = value * 10 + (text[i] - '0');
        if (value > 1000000) return false;
    }
    out = static_cast<int>(value * sign);
    return true;
}

bool is_header(const std::vector<std::string>& fields) {
    return fields.size() == 4 && lower_header(fields[0]) == "name" && lower_header(fields[1]) == "priority" &&
           lower_header(fields[2]) == "condition" && lower_header(fields[3]) == "action";
}

}  // namespace

Result<RuleStore> load_rules(std::string_view text) {
    RuleStore store;
    std::size_t line_start = 0;
    bool saw_data = false;
    while (line_start <= text.size()) {
        std::size_t line_end = text.find('\n', line_start);
        if (line_end == std::string_view::npos) line_end = text.size();
        std::string_view line = text.substr(line_start, line_end - line_start);
        std::string trimmed = trim_csv(std::string(line));
        if (!trimmed.empty() && trimmed[0] != '#') {
            auto fields = parse_csv_line(line, line_start);
            if (!fields.ok()) return Result<RuleStore>::failure(fields.error());
            if (!saw_data && is_header(fields.value())) {
                saw_data = true;
            } else {
                saw_data = true;
                if (fields.value().size() != 4) {
                    return Result<RuleStore>::failure(diagnostic("E_LOAD", line_start, "expected four fields"));
                }
                int priority = 0;
                if (!parse_int(fields.value()[1], priority)) {
                    return Result<RuleStore>::failure(diagnostic("E_LOAD", line_start, "invalid priority '" + fields.value()[1] + "'"));
                }
                auto added = store.add_rule(Rule{fields.value()[0], priority, fields.value()[2], fields.value()[3], line_start});
                if (!added.ok()) {
                    return Result<RuleStore>::failure(diagnostic("E_LOAD", line_start, added.error().message));
                }
            }
        }
        if (line_end == text.size()) break;
        line_start = line_end + 1;
    }
    return Result<RuleStore>::success(store);
}

}  // namespace mh3

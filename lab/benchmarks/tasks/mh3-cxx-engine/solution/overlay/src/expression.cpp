#include "expression.hpp"

#include "tokenizer.hpp"

#include <cmath>
#include <iomanip>
#include <sstream>

namespace mh3 {
namespace {

bool is_number(const Value& value) {
    return value.kind == Value::Kind::Number;
}

bool is_bool(const Value& value) {
    return value.kind == Value::Kind::Boolean;
}

Result<Value> type_error(std::size_t offset, const std::string& message) {
    return Result<Value>::failure(diagnostic("E_EVAL", offset, message));
}

class Parser {
public:
    Parser(std::vector<Token> tokens, const Context& context)
        : tokens_(std::move(tokens)), context_(context) {}

    Result<Value> parse() {
        if (tokens_.empty()) {
            return Result<Value>::failure(diagnostic("E_PARSE", 0, "empty expression"));
        }
        auto value = parse_or();
        if (!value.ok()) return value;
        if (!at_end()) {
            return Result<Value>::failure(diagnostic("E_PARSE", peek().offset, "unexpected token"));
        }
        return value;
    }

private:
    Result<Value> parse_or() {
        auto left = parse_and();
        if (!left.ok()) return left;
        while (match(TokenKind::Or)) {
            Token op = previous();
            auto right = parse_and();
            if (!right.ok()) return right;
            if (!is_bool(left.value()) || !is_bool(right.value())) {
                return type_error(op.offset, "operator 'or' requires booleans");
            }
            left = Result<Value>::success(Value::bool_value(left.value().boolean || right.value().boolean));
        }
        return left;
    }

    Result<Value> parse_and() {
        auto left = parse_comparison();
        if (!left.ok()) return left;
        while (match(TokenKind::And)) {
            Token op = previous();
            auto right = parse_comparison();
            if (!right.ok()) return right;
            if (!is_bool(left.value()) || !is_bool(right.value())) {
                return type_error(op.offset, "operator 'and' requires booleans");
            }
            left = Result<Value>::success(Value::bool_value(left.value().boolean && right.value().boolean));
        }
        return left;
    }

    Result<Value> parse_comparison() {
        auto left = parse_additive();
        if (!left.ok()) return left;
        while (match_any({TokenKind::EqualEqual, TokenKind::BangEqual, TokenKind::Less, TokenKind::LessEqual,
                          TokenKind::Greater, TokenKind::GreaterEqual})) {
            Token op = previous();
            auto right = parse_additive();
            if (!right.ok()) return right;
            bool result = false;
            if (op.kind == TokenKind::EqualEqual || op.kind == TokenKind::BangEqual) {
                result = values_equal(left.value(), right.value());
                if (op.kind == TokenKind::BangEqual) result = !result;
            } else {
                if (!is_number(left.value()) || !is_number(right.value())) {
                    return type_error(op.offset, "ordered comparison requires numbers");
                }
                if (op.kind == TokenKind::Less) result = left.value().number < right.value().number;
                if (op.kind == TokenKind::LessEqual) result = left.value().number <= right.value().number;
                if (op.kind == TokenKind::Greater) result = left.value().number > right.value().number;
                if (op.kind == TokenKind::GreaterEqual) result = left.value().number >= right.value().number;
            }
            left = Result<Value>::success(Value::bool_value(result));
        }
        return left;
    }

    Result<Value> parse_additive() {
        auto left = parse_multiplicative();
        if (!left.ok()) return left;
        while (match_any({TokenKind::Plus, TokenKind::Minus})) {
            Token op = previous();
            auto right = parse_multiplicative();
            if (!right.ok()) return right;
            if (!is_number(left.value()) || !is_number(right.value())) {
                return type_error(op.offset, "arithmetic requires numbers");
            }
            double raw = op.kind == TokenKind::Plus ? left.value().number + right.value().number
                                                    : left.value().number - right.value().number;
            left = Result<Value>::success(Value::number_value(raw));
        }
        return left;
    }

    Result<Value> parse_multiplicative() {
        auto left = parse_unary();
        if (!left.ok()) return left;
        while (match_any({TokenKind::Star, TokenKind::Slash})) {
            Token op = previous();
            auto right = parse_unary();
            if (!right.ok()) return right;
            if (!is_number(left.value()) || !is_number(right.value())) {
                return type_error(op.offset, "arithmetic requires numbers");
            }
            if (op.kind == TokenKind::Slash && std::fabs(right.value().number) < 0.0005) {
                return type_error(op.offset, "division by zero");
            }
            double raw = op.kind == TokenKind::Star ? left.value().number * right.value().number
                                                    : left.value().number / right.value().number;
            left = Result<Value>::success(Value::number_value(raw));
        }
        return left;
    }

    Result<Value> parse_unary() {
        if (match(TokenKind::Not)) {
            Token op = previous();
            auto value = parse_unary();
            if (!value.ok()) return value;
            if (!is_bool(value.value())) return type_error(op.offset, "operator 'not' requires boolean");
            return Result<Value>::success(Value::bool_value(!value.value().boolean));
        }
        if (match(TokenKind::Minus)) {
            Token op = previous();
            auto value = parse_unary();
            if (!value.ok()) return value;
            if (!is_number(value.value())) return type_error(op.offset, "unary '-' requires number");
            return Result<Value>::success(Value::number_value(-value.value().number));
        }
        return parse_primary();
    }

    Result<Value> parse_primary() {
        if (match(TokenKind::Number)) {
            try {
                return Result<Value>::success(Value::number_value(std::stod(previous().text)));
            } catch (...) {
                return Result<Value>::failure(diagnostic("E_PARSE", previous().offset, "invalid number"));
            }
        }
        if (match(TokenKind::String)) {
            return Result<Value>::success(Value::string_value(previous().text));
        }
        if (match(TokenKind::Identifier)) {
            Token name = previous();
            if (name.text == "true") return Result<Value>::success(Value::bool_value(true));
            if (name.text == "false") return Result<Value>::success(Value::bool_value(false));
            auto found = context_.find(name.text);
            if (found == context_.end()) {
                return Result<Value>::failure(diagnostic("E_EVAL", name.offset, "unknown identifier '" + name.text + "'"));
            }
            return Result<Value>::success(found->second);
        }
        if (match(TokenKind::LParen)) {
            Token open = previous();
            auto value = parse_or();
            if (!value.ok()) return value;
            if (!match(TokenKind::RParen)) {
                return Result<Value>::failure(diagnostic("E_PARSE", open.offset, "missing ')'"));
            }
            return value;
        }
        std::size_t offset = at_end() ? (tokens_.empty() ? 0 : tokens_.back().offset + tokens_.back().text.size()) : peek().offset;
        return Result<Value>::failure(diagnostic("E_PARSE", offset, "expected expression"));
    }

    bool values_equal(const Value& a, const Value& b) const {
        if (a.kind != b.kind) return false;
        if (a.kind == Value::Kind::Number) return a.number == b.number;
        if (a.kind == Value::Kind::String) return a.text == b.text;
        if (a.kind == Value::Kind::Boolean) return a.boolean == b.boolean;
        return true;
    }

    bool at_end() const {
        return pos_ >= tokens_.size();
    }

    const Token& peek() const {
        return tokens_[pos_];
    }

    const Token& previous() const {
        return tokens_[pos_ - 1];
    }

    bool match(TokenKind kind) {
        if (at_end() || tokens_[pos_].kind != kind) return false;
        ++pos_;
        return true;
    }

    bool match_any(std::initializer_list<TokenKind> kinds) {
        for (TokenKind kind : kinds) {
            if (match(kind)) return true;
        }
        return false;
    }

    std::vector<Token> tokens_;
    const Context& context_;
    std::size_t pos_ = 0;
};

}  // namespace

std::string Diagnostic::format() const {
    return code + "@" + std::to_string(offset) + ": " + message;
}

Value Value::null() {
    return Value{};
}

Value Value::number_value(double value) {
    Value out;
    out.kind = Kind::Number;
    out.number = normalize_number(value);
    return out;
}

Value Value::string_value(std::string value) {
    Value out;
    out.kind = Kind::String;
    out.text = std::move(value);
    return out;
}

Value Value::bool_value(bool value) {
    Value out;
    out.kind = Kind::Boolean;
    out.boolean = value;
    return out;
}

double normalize_number(double value) {
    double rounded = std::round(value * 1000.0) / 1000.0;
    return rounded == 0.0 ? 0.0 : rounded;
}

std::string render_value(const Value& value) {
    if (value.kind == Value::Kind::Number) {
        std::ostringstream out;
        out << std::fixed << std::setprecision(3) << normalize_number(value.number);
        return out.str();
    }
    if (value.kind == Value::Kind::String) return value.text;
    if (value.kind == Value::Kind::Boolean) return value.boolean ? "true" : "false";
    return "null";
}

Result<Value> evaluate_expression(std::string_view expression, const Context& context) {
    auto tokens = tokenize(expression);
    if (!tokens.ok()) return Result<Value>::failure(tokens.error());
    return Parser(tokens.value(), context).parse();
}

}  // namespace mh3

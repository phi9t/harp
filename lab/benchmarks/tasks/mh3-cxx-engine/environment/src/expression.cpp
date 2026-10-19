#include "expression.hpp"

#include <cmath>
#include <iomanip>
#include <sstream>

namespace mh3 {

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
    if (value.kind == Value::Kind::String) {
        return value.text;
    }
    if (value.kind == Value::Kind::Boolean) {
        return value.boolean ? "true" : "false";
    }
    return "null";
}

Result<Value> evaluate_expression(std::string_view, const Context&) {
    return Result<Value>::failure(diagnostic("E_PARSE", 0, "expression parser is not implemented"));
}

}  // namespace mh3

#pragma once

#include <map>
#include <string>

namespace mh3 {

struct Value {
    enum class Kind { Null, Number, String, Boolean };

    Kind kind = Kind::Null;
    double number = 0.0;
    std::string text;
    bool boolean = false;

    static Value null();
    static Value number_value(double value);
    static Value string_value(std::string value);
    static Value bool_value(bool value);
};

using Context = std::map<std::string, Value>;

double normalize_number(double value);
std::string render_value(const Value& value);

}  // namespace mh3

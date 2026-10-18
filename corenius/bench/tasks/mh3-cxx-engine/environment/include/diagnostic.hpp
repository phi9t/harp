#pragma once

#include <cstddef>
#include <string>
#include <utility>

namespace mh3 {

struct Diagnostic {
    std::string code;
    std::size_t offset = 0;
    std::string message;

    std::string format() const;
};

inline Diagnostic diagnostic(std::string code, std::size_t offset, std::string message) {
    return Diagnostic{std::move(code), offset, std::move(message)};
}

template <typename T>
class Result {
public:
    static Result success(T value) {
        Result out;
        out.ok_ = true;
        out.value_ = std::move(value);
        return out;
    }

    static Result failure(Diagnostic diag) {
        Result out;
        out.ok_ = false;
        out.diagnostic_ = std::move(diag);
        return out;
    }

    bool ok() const { return ok_; }
    const T& value() const { return value_; }
    T& value() { return value_; }
    const Diagnostic& error() const { return diagnostic_; }

private:
    bool ok_ = false;
    T value_{};
    Diagnostic diagnostic_{};
};

}  // namespace mh3

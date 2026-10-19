#pragma once

#include "diagnostic.hpp"

#include <cstddef>
#include <string>
#include <string_view>
#include <vector>

namespace mh3 {

enum class TokenKind {
    Identifier,
    Number,
    String,
    LParen,
    RParen,
    Plus,
    Minus,
    Star,
    Slash,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Not
};

struct Token {
    TokenKind kind = TokenKind::Identifier;
    std::string text;
    std::size_t offset = 0;
};

Result<std::vector<Token>> tokenize(std::string_view input);

}  // namespace mh3

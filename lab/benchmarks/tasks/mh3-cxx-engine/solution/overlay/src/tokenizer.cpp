#include "tokenizer.hpp"

#include <cctype>

namespace mh3 {
namespace {

bool ident_start(char c) {
    unsigned char ch = static_cast<unsigned char>(c);
    return std::isalpha(ch) || c == '_';
}

bool ident_part(char c) {
    unsigned char ch = static_cast<unsigned char>(c);
    return std::isalnum(ch) || c == '_';
}

std::string lower_token(std::string text) {
    for (char& c : text) {
        c = static_cast<char>(std::tolower(static_cast<unsigned char>(c)));
    }
    return text;
}

TokenKind identifier_kind(const std::string& text) {
    std::string word = lower_token(text);
    if (word == "and") return TokenKind::And;
    if (word == "or") return TokenKind::Or;
    if (word == "not") return TokenKind::Not;
    return TokenKind::Identifier;
}

}  // namespace

Result<std::vector<Token>> tokenize(std::string_view input) {
    std::vector<Token> out;
    std::size_t i = 0;
    while (i < input.size()) {
        char c = input[i];
        if (c == '#') {
            break;
        }
        if (std::isspace(static_cast<unsigned char>(c))) {
            ++i;
            continue;
        }
        if (ident_start(c)) {
            std::size_t start = i++;
            while (i < input.size() && ident_part(input[i])) ++i;
            std::string text(input.substr(start, i - start));
            out.push_back(Token{identifier_kind(text), text, start});
            continue;
        }
        if (std::isdigit(static_cast<unsigned char>(c)) || c == '.') {
            std::size_t start = i;
            bool saw_digit = false;
            bool saw_dot = false;
            while (i < input.size()) {
                char n = input[i];
                if (std::isdigit(static_cast<unsigned char>(n))) {
                    saw_digit = true;
                    ++i;
                } else if (n == '.' && !saw_dot) {
                    saw_dot = true;
                    ++i;
                } else {
                    break;
                }
            }
            if (!saw_digit) {
                return Result<std::vector<Token>>::failure(diagnostic("E_LEX", start, "expected digit after '.'"));
            }
            out.push_back(Token{TokenKind::Number, std::string(input.substr(start, i - start)), start});
            continue;
        }
        if (c == '"' || c == '\'') {
            char quote = c;
            std::size_t start = i++;
            std::string text;
            bool closed = false;
            while (i < input.size()) {
                char n = input[i++];
                if (n == quote) {
                    closed = true;
                    break;
                }
                if (n == '\\' && i < input.size()) {
                    char escaped = input[i++];
                    if (escaped == 'n') text.push_back('\n');
                    else if (escaped == 't') text.push_back('\t');
                    else text.push_back(escaped);
                } else {
                    text.push_back(n);
                }
            }
            if (!closed) {
                return Result<std::vector<Token>>::failure(diagnostic("E_LEX", start, "unterminated string"));
            }
            out.push_back(Token{TokenKind::String, text, start});
            continue;
        }
        if (c == '=' && i + 1 < input.size() && input[i + 1] == '=') {
            out.push_back(Token{TokenKind::EqualEqual, "==", i});
            i += 2;
        } else if (c == '!' && i + 1 < input.size() && input[i + 1] == '=') {
            out.push_back(Token{TokenKind::BangEqual, "!=", i});
            i += 2;
        } else if (c == '<' && i + 1 < input.size() && input[i + 1] == '=') {
            out.push_back(Token{TokenKind::LessEqual, "<=", i});
            i += 2;
        } else if (c == '>' && i + 1 < input.size() && input[i + 1] == '=') {
            out.push_back(Token{TokenKind::GreaterEqual, ">=", i});
            i += 2;
        } else if (c == '<') {
            out.push_back(Token{TokenKind::Less, "<", i++});
        } else if (c == '>') {
            out.push_back(Token{TokenKind::Greater, ">", i++});
        } else if (c == '+') {
            out.push_back(Token{TokenKind::Plus, "+", i++});
        } else if (c == '-') {
            out.push_back(Token{TokenKind::Minus, "-", i++});
        } else if (c == '*') {
            out.push_back(Token{TokenKind::Star, "*", i++});
        } else if (c == '/') {
            out.push_back(Token{TokenKind::Slash, "/", i++});
        } else if (c == '(') {
            out.push_back(Token{TokenKind::LParen, "(", i++});
        } else if (c == ')') {
            out.push_back(Token{TokenKind::RParen, ")", i++});
        } else {
            return Result<std::vector<Token>>::failure(
                diagnostic("E_LEX", i, std::string("unexpected character '") + c + "'"));
        }
    }
    return Result<std::vector<Token>>::success(out);
}

}  // namespace mh3

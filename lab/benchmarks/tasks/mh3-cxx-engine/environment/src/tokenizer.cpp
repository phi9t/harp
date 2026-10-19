#include "tokenizer.hpp"

namespace mh3 {

Result<std::vector<Token>> tokenize(std::string_view) {
    return Result<std::vector<Token>>::success({});
}

}  // namespace mh3

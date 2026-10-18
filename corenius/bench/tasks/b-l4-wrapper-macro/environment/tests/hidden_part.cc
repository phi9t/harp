#include "config/config.h"
#include "tests/hidden_part.h"

int hidden_part_value() {
  return config_offset() + 100;
}

#include "proto/generated/record_schema.h"

#include <cstdio>

int main() {
  int id = bladebench::record_id_field_number();
  int shard = bladebench::record_shard_field_number();
  const char* name = bladebench::record_shard_field_name();
  std::printf("PROTO_RESULT id=%d shard=%d name=%s\n", id, shard, name);
  return id == 1 && shard == 2 ? 0 : 1;
}

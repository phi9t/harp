#!/bin/sh
set -eu

blade-offline build //app:proto_app
./build64_release/app/proto_app > /tmp/proto-codegen.out
grep -qx 'PROTO_RESULT id=1 shard=2 name=shard' /tmp/proto-codegen.out

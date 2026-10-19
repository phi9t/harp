#!/bin/sh
set -eu

blade-offline build //app:proto_app
./build64_release/app/proto_app > /run/insula-tmp/proto-codegen.out
grep -qx 'PROTO_RESULT id=1 shard=2 name=shard' /run/insula-tmp/proto-codegen.out

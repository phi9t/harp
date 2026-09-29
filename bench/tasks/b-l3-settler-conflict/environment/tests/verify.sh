#!/bin/sh
set -eu

blade-offline build //:settler_app
./build64_release/settler_app > /tmp/settler.out
grep -qx 'SETTLER_RESULT value=7' /tmp/settler.out

#!/bin/sh
set -eu

blade-offline build \
  //feature/p0:part0 \
  //feature/p1:part1 \
  //feature/p2:part2 \
  //feature/p3:part3 \
  //feature/p4:part4 \
  //feature/p5:part5 \
  //app:dashboard \
  //tests:macro_probe

./build64_release/app/dashboard > /tmp/wrapper-app.out
grep -qx 'WRAPPER_RESULT total=75' /tmp/wrapper-app.out

./build64_release/tests/macro_probe > /tmp/wrapper-hidden.out
grep -qx 'WRAPPER_HIDDEN value=110' /tmp/wrapper-hidden.out

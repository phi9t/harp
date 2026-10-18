#!/bin/sh
set -eu

awk '
  /^pub const IMPLEMENTED_COMPONENTS:/ {
    if ($0 ~ /"graph"/) {
      print "pub const IMPLEMENTED_COMPONENTS: &[&str] = &[\"graph\", \"queue\"];"
    } else {
      print "pub const IMPLEMENTED_COMPONENTS: &[&str] = &[\"queue\"];"
    }
    next
  }
  { print }
' src/lib.rs > src/lib.rs.new
mv src/lib.rs.new src/lib.rs
cp /run/corenius-tmp/ticket/overlay/src/queue.rs src/queue.rs

#!/bin/sh
set -eu

awk '
  /^pub const IMPLEMENTED_COMPONENTS:/ {
    if ($0 ~ /"queue"/) {
      print "pub const IMPLEMENTED_COMPONENTS: &[&str] = &[\"graph\", \"queue\"];"
    } else {
      print "pub const IMPLEMENTED_COMPONENTS: &[&str] = &[\"graph\"];"
    }
    next
  }
  { print }
' src/lib.rs > src/lib.rs.new
mv src/lib.rs.new src/lib.rs
cp /tmp/ticket/overlay/src/graph.rs src/graph.rs

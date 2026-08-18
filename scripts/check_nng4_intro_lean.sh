#!/bin/sh
set -eu

case "$0" in
  /*) script_path=$0 ;;
  *) script_path=$PWD/$0 ;;
esac
script_dir=${script_path%/*}

case "$#" in
  0)
    exec "$script_dir/check_lean_library.sh" NNG4Intro
    ;;
  2)
    if [ "$1" = "--project-for-test" ]; then
      exec "$script_dir/check_lean_library.sh" NNG4Intro --project-for-test "$2"
    fi
    ;;
esac

printf '%s\n' "usage: $0 [--project-for-test <directory>]" >&2
exit 2

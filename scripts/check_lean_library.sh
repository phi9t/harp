#!/bin/sh
set -eu

case "$0" in
  /*) script_path=$0 ;;
  *) script_path=$PWD/$0 ;;
esac
script_dir=${script_path%/*}
canonical_project_dir=$(CDPATH= cd -- "$script_dir/../formalization/lean" && pwd -P)
relative_root=formalization/lean
allowed_elan_root=/private/tmp/harp-mathematical-foundations-elan

usage() {
  printf '%s\n' "usage: $0 TrainingDynamics|MathematicalFoundations|NNG4Intro|AutodiffGeometry|Crouzeix|all [--project-for-test <directory>]" >&2
}

report_failure() {
  target_label=$1
  stage=$2
  scan_seconds=${3:-0}
  lake_seconds=${4:-0}
  total_seconds=${5:-0}
  printf '%s\n' "[lean] target=$target_label"
  printf '%s\n' "[lean] root=$relative_root"
  printf '%s\n' "[lean] scan_seconds=$scan_seconds"
  printf '%s\n' "[lean] lake_seconds=$lake_seconds"
  printf '%s\n' "[lean] total_seconds=$total_seconds"
  printf '%s\n' "[lean] outcome=failed"
  printf '%s\n' "[lean] failure_stage=$stage"
}

report_success() {
  target_label=$1
  scan_seconds=$2
  lake_seconds=$3
  total_seconds=$4
  printf '%s\n' "[lean] target=$target_label"
  printf '%s\n' "[lean] root=$relative_root"
  printf '%s\n' "[lean] scan_seconds=$scan_seconds"
  printf '%s\n' "[lean] lake_seconds=$lake_seconds"
  printf '%s\n' "[lean] total_seconds=$total_seconds"
  printf '%s\n' "[lean] outcome=passed"
}

now_seconds() {
  date +%s
}

case "$#" in
  1)
    target=$1
    project_dir=$canonical_project_dir
    normal_build=true
    ;;
  3)
    target=$1
    if [ "$2" = "--project-for-test" ] && [ -d "$3" ]; then
      project_dir=$(CDPATH= cd -- "$3" && pwd -P)
      normal_build=false
    else
      usage
      exit 2
    fi
    ;;
  *)
    usage
    exit 2
    ;;
esac

case "$target" in
  TrainingDynamics)
    human_label="Training Dynamics"
    scan_label=TrainingDynamics
    forbidden_words="sorry"
    ;;
  MathematicalFoundations)
    human_label="Mathematical Foundations"
    scan_label=MathematicalFoundations
    forbidden_words="sorry"
    ;;
  NNG4Intro)
    human_label="NNG4 introductory"
    scan_label=NNG4Intro
    forbidden_words="sorry admit"
    ;;
  AutodiffGeometry)
    human_label="Autodiff Geometry"
    scan_label=AutodiffGeometry
    forbidden_words="sorry"
    ;;
  Crouzeix)
    human_label="Crouzeix"
    scan_label=Crouzeix
    forbidden_words="sorry admit"
    ;;
  all)
    human_label="Harp formalization"
    scan_label=all
    forbidden_words="sorry"
    ;;
  *)
    printf '%s\n' "Unknown Lean library target: $target" >&2
    report_failure "$target" policy 0 0 0
    exit 2
    ;;
esac

if ! command -v lake >/dev/null 2>&1; then
  printf '%s\n' "$human_label Lean toolchain is unavailable" >&2
  report_failure "$scan_label" toolchain 0 0 0
  exit 1
fi

if [ "$normal_build" = true ]; then
  if [ -z "${ELAN_HOME:-}" ]; then
    printf '%s\n' "$human_label Lean verification requires a task-scoped ELAN_HOME" >&2
    report_failure "$scan_label" toolchain 0 0 0
    exit 1
  fi
  if ! elan_home_dir=$(CDPATH= cd -P -- "$ELAN_HOME" && pwd -P); then
    printf '%s\n' "$human_label Lean verification requires a canonical task-scoped ELAN_HOME directory" >&2
    report_failure "$scan_label" toolchain 0 0 0
    exit 1
  fi
  case "$elan_home_dir" in
    "$allowed_elan_root"|"$allowed_elan_root"/*)
      ;;
    *)
      printf '%s\n' "$human_label Lean verification requires a task-scoped ELAN_HOME" >&2
      report_failure "$scan_label" toolchain 0 0 0
      exit 1
      ;;
  esac
  if ! IFS= read -r pinned_toolchain < "$canonical_project_dir/lean-toolchain" || [ -z "$pinned_toolchain" ]; then
    printf '%s\n' "$human_label Lean toolchain pin is unavailable" >&2
    report_failure "$scan_label" toolchain 0 0 0
    exit 1
  fi
  export ELAN_TOOLCHAIN=$pinned_toolchain
fi

if ! scan_list=$(mktemp "${TMPDIR:-/tmp}/harp-lean-sources.XXXXXX"); then
  printf '%s\n' "$human_label Lean source scan failed: could not create a temporary file" >&2
  report_failure "$scan_label" scan 0 0 0
  exit 1
fi
trap 'rm -f "$scan_list"' EXIT HUP INT TERM

append_sources() {
  source_path=$1
  if [ -f "$source_path" ]; then
    case "$source_path" in
      *.lean) printf '%s\n' "$source_path" >> "$scan_list" ;;
    esac
  elif [ -d "$source_path" ]; then
    find "$source_path" -type d -name .lake -prune -o -type f -name '*.lean' -print >> "$scan_list"
  else
    printf '%s\n' "$human_label Lean source scan failed: missing source path $source_path" >&2
    return 1
  fi
}

scan_started=$(now_seconds)
if [ "$normal_build" = false ]; then
  if ! append_sources "$project_dir"; then
    scan_finished=$(now_seconds)
    report_failure "$scan_label" scan "$((scan_finished - scan_started))" 0 "$((scan_finished - scan_started))"
    exit 1
  fi
else
  case "$target" in
    all)
      for library in TrainingDynamics MathematicalFoundations NNG4Intro AutodiffGeometry Crouzeix; do
        if ! append_sources "$project_dir/$library.lean" || ! append_sources "$project_dir/$library"; then
          scan_finished=$(now_seconds)
          report_failure "$scan_label" scan "$((scan_finished - scan_started))" 0 "$((scan_finished - scan_started))"
          exit 1
        fi
      done
      ;;
    *)
      if ! append_sources "$project_dir/$target.lean" || ! append_sources "$project_dir/$target"; then
        scan_finished=$(now_seconds)
        report_failure "$scan_label" scan "$((scan_finished - scan_started))" 0 "$((scan_finished - scan_started))"
        exit 1
      fi
      ;;
  esac
fi

proof_holes_found=false
while IFS= read -r source_file || [ -n "$source_file" ]; do
  for forbidden_word in $forbidden_words; do
    if grep -n -H "$forbidden_word" "$source_file"; then
      proof_holes_found=true
    else
      grep_status=$?
      if [ "$grep_status" -ne 1 ]; then
        scan_finished=$(now_seconds)
        printf '%s\n' "$human_label Lean source scan failed" >&2
        report_failure "$scan_label" scan "$((scan_finished - scan_started))" 0 "$((scan_finished - scan_started))"
        exit 1
      fi
    fi
  done
  if [ "$target" = all ]; then
    case "$source_file" in
      "$project_dir/NNG4Intro.lean"|"$project_dir/NNG4Intro/"*)
        if grep -n -H 'admit' "$source_file"; then
          proof_holes_found=true
        else
          grep_status=$?
          if [ "$grep_status" -ne 1 ]; then
            scan_finished=$(now_seconds)
            printf '%s\n' "$human_label Lean source scan failed" >&2
            report_failure "$scan_label" scan "$((scan_finished - scan_started))" 0 "$((scan_finished - scan_started))"
            exit 1
          fi
        fi
        ;;
    esac
  fi
done < "$scan_list"
scan_finished=$(now_seconds)
scan_seconds=$((scan_finished - scan_started))

if [ "$proof_holes_found" = true ]; then
  printf '%s\n' "$human_label Lean source contains forbidden proof-placeholder text." >&2
  report_failure "$scan_label" scan "$scan_seconds" 0 "$scan_seconds"
  exit 1
fi

lake_started=$(now_seconds)
cd "$project_dir"
if [ "$target" = all ]; then
  if lake build; then
    lake_finished=$(now_seconds)
    lake_seconds=$((lake_finished - lake_started))
    report_success "$scan_label" "$scan_seconds" "$lake_seconds" "$((scan_seconds + lake_seconds))"
    exit 0
  fi
else
  if lake build "$target"; then
    lake_finished=$(now_seconds)
    lake_seconds=$((lake_finished - lake_started))
    report_success "$scan_label" "$scan_seconds" "$lake_seconds" "$((scan_seconds + lake_seconds))"
    exit 0
  fi
fi

lake_finished=$(now_seconds)
lake_seconds=$((lake_finished - lake_started))
printf '%s\n' "$human_label Lean build failed" >&2
report_failure "$scan_label" lake "$scan_seconds" "$lake_seconds" "$((scan_seconds + lake_seconds))"
exit 1

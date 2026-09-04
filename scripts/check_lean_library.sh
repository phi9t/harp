#!/bin/sh
set -eu

case "$0" in
  /*) script_path=$0 ;;
  *) script_path=$PWD/$0 ;;
esac
script_dir=${script_path%/*}
canonical_project_dir=$(CDPATH= cd -- "$script_dir/../formalization/lean" && pwd -P)
relative_root=formalization/lean

resolve_default_lean_cache_root() {
  : "${HOME:?HOME must be set}"
  harp_cache_home=${XDG_CACHE_HOME:-"$HOME/.cache"}
  case "$harp_cache_home" in
    /*) ;;
    *)
      printf '%s\n' 'XDG_CACHE_HOME must be absolute' >&2
      return 2
      ;;
  esac
  printf '%s\n' "$harp_cache_home/harp/lean/lean-4.32.1"
}

usage() {
  printf '%s\n' "usage: $0 TrainingDynamics|MathematicalFoundations|NNG4Intro|AutodiffGeometry|Crouzeix|CrouzeixJin|CrouzeixLoristSchwenninger|CrouzeixHarp|all [--project-for-test <directory>]" >&2
}

report_failure() {
  target_label=$1
  stage=$2
  scan_seconds=${3:-0}
  cache_seconds=${4:-0}
  lake_seconds=${5:-0}
  total_seconds=${6:-0}
  printf '%s\n' "[lean] target=$target_label"
  printf '%s\n' "[lean] root=$relative_root"
  printf '%s\n' "[lean] scan_seconds=$scan_seconds"
  printf '%s\n' "[lean] cache_seconds=$cache_seconds"
  printf '%s\n' "[lean] lake_seconds=$lake_seconds"
  printf '%s\n' "[lean] total_seconds=$total_seconds"
  printf '%s\n' "[lean] outcome=failed"
  printf '%s\n' "[lean] failure_stage=$stage"
}

report_success() {
  target_label=$1
  scan_seconds=$2
  cache_seconds=$3
  lake_seconds=$4
  total_seconds=$5
  printf '%s\n' "[lean] target=$target_label"
  printf '%s\n' "[lean] root=$relative_root"
  printf '%s\n' "[lean] scan_seconds=$scan_seconds"
  printf '%s\n' "[lean] cache_seconds=$cache_seconds"
  printf '%s\n' "[lean] lake_seconds=$lake_seconds"
  printf '%s\n' "[lean] total_seconds=$total_seconds"
  printf '%s\n' "[lean] outcome=passed"
}

now_seconds() {
  date +%s
}

has_valid_olean_header() {
  artifact_path=$1
  [ -s "$artifact_path" ] && [ "$(dd if="$artifact_path" bs=5 count=1 2>/dev/null)" = olean ]
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
  CrouzeixJin)
    human_label="Crouzeix Jin"
    scan_label=CrouzeixJin
    forbidden_words="sorry admit"
    ;;
  CrouzeixLoristSchwenninger)
    human_label="Crouzeix Lorist--Schwenninger"
    scan_label=CrouzeixLoristSchwenninger
    forbidden_words="sorry admit"
    ;;
  CrouzeixHarp)
    human_label="Crouzeix Harp"
    scan_label=CrouzeixHarp
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

is_route_isolated_target() {
  case "$1" in
    CrouzeixJin|CrouzeixLoristSchwenninger|CrouzeixHarp)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

route_preflight_name() {
  case "$1" in
    CrouzeixJin)
      printf '%s\n' jin
      ;;
    CrouzeixLoristSchwenninger)
      printf '%s\n' lorist-schwenninger
      ;;
    CrouzeixHarp)
      printf '%s\n' harp
      ;;
    *)
      return 1
      ;;
  esac
}

if [ "$normal_build" = true ] && is_route_isolated_target "$target"; then
  preflight_route=$(route_preflight_name "$target") || {
    printf '%s\n' "Unknown route-isolated Lean target: $target" >&2
    report_failure "$scan_label" preflight 0 0 0 0
    exit 1
  }
  if ! preflight_output=$(cd "$script_dir/.." && python3 labs/crouzeix_proof_reproduction/proof_evidence.py preflight --route "$preflight_route" 2>&1); then
    printf '%s\n' "$preflight_output" >&2
    report_failure "$scan_label" preflight 0 0 0 0
    exit 1
  fi
fi

if ! command -v lake >/dev/null 2>&1; then
  printf '%s\n' "$human_label Lean toolchain is unavailable" >&2
  report_failure "$scan_label" toolchain 0 0 0
  exit 1
fi

if [ "$normal_build" = true ]; then
  if [ -n "${HARP_LEAN_CACHE_ROOT:-}" ]; then
    allowed_lean_cache_root=$HARP_LEAN_CACHE_ROOT
  else
    allowed_lean_cache_root=$(resolve_default_lean_cache_root)
  fi
  case "$allowed_lean_cache_root" in
    /*) ;;
    *)
      printf '%s\n' "$human_label Lean verification requires an absolute HARP_LEAN_CACHE_ROOT" >&2
      report_failure "$scan_label" toolchain 0 0 0
      exit 1
      ;;
  esac
  if ! allowed_lean_cache_root=$(CDPATH= cd -P -- "$allowed_lean_cache_root" && pwd -P); then
    printf '%s\n' "$human_label Lean verification requires a canonical HARP_LEAN_CACHE_ROOT directory" >&2
    report_failure "$scan_label" toolchain 0 0 0
    exit 1
  fi
  if [ -n "${HARP_ELAN_HOME:-}" ]; then
    allowed_elan_root=$HARP_ELAN_HOME
  else
    allowed_elan_root=${allowed_lean_cache_root%/*}/elan
  fi
  if ! allowed_elan_root=$(CDPATH= cd -P -- "$allowed_elan_root" && pwd -P); then
    printf '%s\n' "$human_label Lean verification requires a canonical sibling ELAN_HOME directory" >&2
    report_failure "$scan_label" toolchain 0 0 0
    exit 1
  fi
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
  report_failure "$scan_label" scan 0 0 0 0
  exit 1
fi
if ! required_cache_list=$(mktemp "${TMPDIR:-/tmp}/harp-lean-cache-modules.XXXXXX"); then
  printf '%s\n' "$human_label Lean source scan failed: could not create a temporary file" >&2
  report_failure "$scan_label" scan 0 0 0 0
  exit 1
fi
if ! missing_cache_list=$(mktemp "${TMPDIR:-/tmp}/harp-lean-missing-cache-modules.XXXXXX"); then
  printf '%s\n' "$human_label Lean source scan failed: could not create a temporary file" >&2
  report_failure "$scan_label" scan 0 0 0 0
  exit 1
fi
if ! closure_sources_list=$(mktemp "${TMPDIR:-/tmp}/harp-lean-closure-sources.XXXXXX"); then
  printf '%s\n' "$human_label Lean source scan failed: could not create a temporary file" >&2
  report_failure "$scan_label" scan 0 0 0 0
  exit 1
fi
if ! local_import_list=$(mktemp "${TMPDIR:-/tmp}/harp-lean-local-imports.XXXXXX"); then
  printf '%s\n' "$human_label Lean source scan failed: could not create a temporary file" >&2
  report_failure "$scan_label" scan 0 0 0 0
  exit 1
fi
trap 'rm -f "$scan_list" "$required_cache_list" "$missing_cache_list" "$closure_sources_list" "$local_import_list"' EXIT HUP INT TERM

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

extract_lean_imports() {
  source_file=$1
  import_prefix=$2
  include_public=$3
  awk -v prefix="$import_prefix" -v include_public="$include_public" '
    function uncomment(line,    clean, cursor, pair) {
      clean = ""
      cursor = 1
      while (cursor <= length(line)) {
        pair = substr(line, cursor, 2)
        if (comment_depth > 0) {
          if (pair == "/-") {
            comment_depth++
            cursor += 2
          } else if (pair == "-/") {
            comment_depth--
            cursor += 2
          } else {
            cursor++
          }
        } else if (pair == "/-") {
          comment_depth++
          cursor += 2
        } else if (pair == "--") {
          break
        } else {
          clean = clean substr(line, cursor, 1)
          cursor++
        }
      }
      return clean
    }
    {
      line = uncomment($0)
      sub(/^[[:space:]]+/, "", line)
      field_count = split(line, fields, /[[:space:]]+/)
      first_module = 0
      if (fields[1] == "import") {
        first_module = 2
      } else if (include_public == "true" && fields[1] == "public" &&
          fields[2] == "import") {
        first_module = 3
      }
      for (i = first_module; i > 0 && i <= field_count; i++) {
        if (prefix == "" || fields[i] == prefix || index(fields[i], prefix ".") == 1) {
          print fields[i]
        }
      }
    }
    END {
      if (comment_depth != 0) {
        exit 2
      }
    }
  ' "$source_file"
}

is_managed_local_import() {
  case "$1" in
    Crouzeix|Crouzeix.*|CrouzeixConjecture|CrouzeixConjecture.*|CrouzeixJin|CrouzeixLoristSchwenninger|CrouzeixHarp)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

is_harp_allowed_ls_support_module() {
  case "$1" in
    Crouzeix.LoristSchwenninger.BoundaryEmbedding|\
    Crouzeix.LoristSchwenninger.BoundaryMultiplier|\
    Crouzeix.LoristSchwenninger.BoundarySquareRoot|\
    Crouzeix.LoristSchwenninger.CompanionAlgebra|\
    Crouzeix.LoristSchwenninger.CompletedSquare|\
    Crouzeix.LoristSchwenninger.CompressionMoments|\
    Crouzeix.LoristSchwenninger.Dilation|\
    Crouzeix.LoristSchwenninger.NormAttainment|\
    Crouzeix.LoristSchwenninger.PolynomialPowerCauchy|\
    Crouzeix.LoristSchwenninger.Recurrence|\
    Crouzeix.LoristSchwenninger.Scalar)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

configure_route_policy() {
  route_policy_target=$1
  route_provider_name=
  case "$route_policy_target" in
    CrouzeixJin)
      route_provider_name=jin
      route_aggregate_module=CrouzeixJin
      ;;
    CrouzeixLoristSchwenninger)
      route_provider_name=lorist-schwenninger
      route_aggregate_module=CrouzeixLoristSchwenninger
      ;;
    CrouzeixHarp)
      route_provider_name=harp
      route_aggregate_module=CrouzeixHarp
      ;;
    *)
      printf '%s\n' "Unknown route-isolated Lean target: $route_policy_target" >&2
      return 1
      ;;
  esac
}

route_accepts_local_import() {
  imported_module=$1
  case "$route_provider_name" in
    jin)
      case "$imported_module" in
        CrouzeixJin|Crouzeix.Jin|Crouzeix.Jin.*|CrouzeixConjecture|CrouzeixConjecture.*)
          return 0
          ;;
        CrouzeixHarp|Crouzeix.Harp|Crouzeix.Harp.*|\
        CrouzeixLoristSchwenninger|Crouzeix.LoristSchwenninger|Crouzeix.LoristSchwenninger.*|\
        Crouzeix|Crouzeix.*)
          return 1
          ;;
      esac
      ;;
    lorist-schwenninger)
      case "$imported_module" in
        CrouzeixLoristSchwenninger|Crouzeix.LoristSchwenninger|Crouzeix.LoristSchwenninger.*|\
        CrouzeixConjecture|CrouzeixConjecture.*)
          return 0
          ;;
        CrouzeixJin|Crouzeix.Jin|Crouzeix.Jin.*|\
        CrouzeixHarp|Crouzeix.Harp|Crouzeix.Harp.*|\
        Crouzeix|Crouzeix.*)
          return 1
          ;;
      esac
      ;;
    harp)
      case "$imported_module" in
        CrouzeixHarp|Crouzeix.Harp|Crouzeix.Harp.*|CrouzeixConjecture|CrouzeixConjecture.*)
          return 0
          ;;
        Crouzeix.LoristSchwenninger|Crouzeix.LoristSchwenninger.*)
          if is_harp_allowed_ls_support_module "$imported_module"; then
            return 0
          fi
          return 1
          ;;
        Crouzeix.LoristSchwenninger.Consequences|Crouzeix.LoristSchwenninger.MainTheorem|\
        CrouzeixLoristSchwenninger|\
        CrouzeixJin|Crouzeix.Jin|Crouzeix.Jin.*|\
        Crouzeix|Crouzeix.*)
          return 1
          ;;
      esac
      ;;
  esac
  return 1
}

append_route_isolated_sources() {
  if ! configure_route_policy "$target"; then
    return 1
  fi
  if ! append_sources "$project_dir/$route_aggregate_module.lean"; then
    return 1
  fi

  closure_changed=true
  while [ "$closure_changed" = true ]; do
    closure_changed=false
    cp "$scan_list" "$closure_sources_list"
    while IFS= read -r source_file || [ -n "$source_file" ]; do
      if ! extract_lean_imports "$source_file" "" true > "$local_import_list"; then
        printf '%s\n' "$human_label Lean source scan failed while reading imports from $source_file" >&2
        return 1
      fi
      while IFS= read -r imported_module || [ -n "$imported_module" ]; do
        if ! is_managed_local_import "$imported_module"; then
          continue
        fi
        if ! route_accepts_local_import "$imported_module"; then
          printf '%s\n' "$human_label Lean source scan rejected provider import $imported_module in $source_file" >&2
          return 1
        fi
        imported_path=$(printf '%s\n' "$imported_module" | tr . /)
        imported_source="$project_dir/$imported_path.lean"
        if [ ! -f "$imported_source" ]; then
          printf '%s\n' "$human_label Lean source scan failed: missing local import $imported_module ($imported_source)" >&2
          return 1
        fi
        if ! grep -F -x -q "$imported_source" "$scan_list"; then
          printf '%s\n' "$imported_source" >> "$scan_list"
          closure_changed=true
        fi
      done < "$local_import_list"
    done < "$closure_sources_list"
  done
}

scan_started=$(now_seconds)
if is_route_isolated_target "$target"; then
  if ! append_route_isolated_sources; then
    scan_finished=$(now_seconds)
    report_failure "$scan_label" scan "$((scan_finished - scan_started))" 0 0 "$((scan_finished - scan_started))"
    exit 1
  fi
elif [ "$normal_build" = false ]; then
  if ! append_sources "$project_dir"; then
    scan_finished=$(now_seconds)
    report_failure "$scan_label" scan "$((scan_finished - scan_started))" 0 0 "$((scan_finished - scan_started))"
    exit 1
  fi
else
  case "$target" in
    all)
      for library in TrainingDynamics MathematicalFoundations NNG4Intro AutodiffGeometry Crouzeix CrouzeixConjecture; do
        if ! append_sources "$project_dir/$library.lean" || ! append_sources "$project_dir/$library"; then
          scan_finished=$(now_seconds)
          report_failure "$scan_label" scan "$((scan_finished - scan_started))" 0 0 "$((scan_finished - scan_started))"
          exit 1
        fi
      done
      ;;
    *)
      if ! append_sources "$project_dir/$target.lean" || ! append_sources "$project_dir/$target"; then
        scan_finished=$(now_seconds)
        report_failure "$scan_label" scan "$((scan_finished - scan_started))" 0 0 "$((scan_finished - scan_started))"
        exit 1
      fi
      if [ "$target" = Crouzeix ]; then
        if ! append_sources "$project_dir/CrouzeixConjecture"; then
          scan_finished=$(now_seconds)
          report_failure "$scan_label" scan "$((scan_finished - scan_started))" 0 0 "$((scan_finished - scan_started))"
          exit 1
        fi
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
        report_failure "$scan_label" scan "$((scan_finished - scan_started))" 0 0 "$((scan_finished - scan_started))"
        exit 1
      fi
    fi
  done
  if [ "$target" = all ]; then
    case "$source_file" in
      "$project_dir/NNG4Intro.lean"|"$project_dir/NNG4Intro/"*|"$project_dir/Crouzeix.lean"|"$project_dir/Crouzeix/"*|"$project_dir/CrouzeixConjecture.lean"|"$project_dir/CrouzeixConjecture/"*)
        if grep -n -H 'admit' "$source_file"; then
          proof_holes_found=true
        else
          grep_status=$?
          if [ "$grep_status" -ne 1 ]; then
            scan_finished=$(now_seconds)
            printf '%s\n' "$human_label Lean source scan failed" >&2
            report_failure "$scan_label" scan "$((scan_finished - scan_started))" 0 0 "$((scan_finished - scan_started))"
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
  report_failure "$scan_label" scan "$scan_seconds" 0 0 "$scan_seconds"
  exit 1
fi

cache_started=$(now_seconds)
cache_seconds=0
if [ "$normal_build" = true ]; then
  while IFS= read -r source_file || [ -n "$source_file" ]; do
    if is_route_isolated_target "$target"; then
      if ! extract_lean_imports "$source_file" Mathlib true >> "$required_cache_list"; then
        printf '%s\n' "$human_label Lean source scan failed while reading imports from $source_file" >&2
        report_failure "$scan_label" scan "$scan_seconds" 0 0 "$scan_seconds"
        exit 1
      fi
    else
      awk '
        /^import[[:space:]]/ {
          for (i = 2; i <= NF; i++) {
            if ($i ~ /^Mathlib($|\.)/) {
              print $i
            }
          }
        }
      ' "$source_file" >> "$required_cache_list"
    fi
  done < "$scan_list"
  sort -u "$required_cache_list" -o "$required_cache_list"

  if [ -s "$required_cache_list" ]; then
    cd "$project_dir"
    mathlib_artifact_root=${HARP_LEAN_MATHLIB_ARTIFACT_ROOT:-"$allowed_lean_cache_root/packages/mathlib/.lake/build/lib/lean"}
    while IFS= read -r required_module || [ -n "$required_module" ]; do
      required_path=$(printf '%s\n' "$required_module" | tr . /)
      required_artifact="$mathlib_artifact_root/$required_path.olean"
      if ! has_valid_olean_header "$required_artifact"; then
        printf '%s\n' "$required_module" >> "$missing_cache_list"
      fi
    done < "$required_cache_list"

    if [ -s "$missing_cache_list" ]; then
      while IFS= read -r required_module || [ -n "$required_module" ]; do
        required_path=$(printf '%s\n' "$required_module" | tr . /)
        required_artifact="$mathlib_artifact_root/$required_path.olean"
        printf '%s\n' "$human_label Lean dependency cache is missing or invalid: $required_artifact" >&2
      done < "$missing_cache_list"
      cache_finished=$(now_seconds)
      cache_seconds=$((cache_finished - cache_started))
      printf '%s\n' "$human_label Lean verification refuses to rebuild common dependencies; run explicit cache hydration before this gate." >&2
      report_failure "$scan_label" cache "$scan_seconds" "$cache_seconds" 0 "$((scan_seconds + cache_seconds))"
      exit 1
    fi
  fi
fi
cache_finished=$(now_seconds)
cache_seconds=$((cache_finished - cache_started))

lake_started=$(now_seconds)
cd "$project_dir"
if [ "$target" = all ]; then
  if lake --try-cache build; then
    lake_finished=$(now_seconds)
    lake_seconds=$((lake_finished - lake_started))
    report_success "$scan_label" "$scan_seconds" "$cache_seconds" "$lake_seconds" "$((scan_seconds + cache_seconds + lake_seconds))"
    exit 0
  fi
else
  if lake --try-cache build "$target"; then
    lake_finished=$(now_seconds)
    lake_seconds=$((lake_finished - lake_started))
    report_success "$scan_label" "$scan_seconds" "$cache_seconds" "$lake_seconds" "$((scan_seconds + cache_seconds + lake_seconds))"
    exit 0
  fi
fi

lake_finished=$(now_seconds)
lake_seconds=$((lake_finished - lake_started))
printf '%s\n' "$human_label Lean build failed" >&2
report_failure "$scan_label" lake "$scan_seconds" "$cache_seconds" "$lake_seconds" "$((scan_seconds + cache_seconds + lake_seconds))"
exit 1

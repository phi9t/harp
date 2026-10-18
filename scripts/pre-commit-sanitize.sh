#!/bin/sh
set -eu

# Harp pre-commit hook: verify staged files are free of PII, machine info, and proprietary terms
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"

if [ -x "$REPO_ROOT/target/debug/harp" ]; then
    HARP_BIN="$REPO_ROOT/target/debug/harp"
elif [ -x "$REPO_ROOT/.build/harp-target/size/harp" ]; then
    HARP_BIN="$REPO_ROOT/.build/harp-target/size/harp"
else
    cargo build -p harp
    HARP_BIN="$REPO_ROOT/target/debug/harp"
fi

STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM)
if [ -z "$STAGED_FILES" ]; then
    exit 0
fi

FAILED=0
for FILE in $STAGED_FILES; do
    if [ -f "$FILE" ]; then
        if ! "$HARP_BIN" sanitize check "$FILE" >/dev/null 2>&1; then
            printf >&2 "Sanitization check failed for staged file: %s\n" "$FILE"
            "$HARP_BIN" sanitize check "$FILE" >&2
            FAILED=1
        fi
    fi
done

if [ "$FAILED" -ne 0 ]; then
    printf >&2 "\nPre-commit hook rejected commit due to sanitization violations.\n"
    printf >&2 "Run 'cargo run -p harp -- sanitize check <path>' or 'cargo run -p harp -- sanitize scrub <path>' to resolve.\n"
    exit 1
fi

# Org Benchmark Summary

- Generated: 2026-09-29T04:06:18Z
- Benchmark: `org-structure`
- Total scenarios: 6
- Scenarios as expected: 6
- Insula checks passed: `t`
- Mode: `selected-container-rootfs`
- Run id: `20260929T040533Z-3771476`

## Correct policy edits an Org source block through org-edit-src-exit

- ID: `correct-source-edit-exit`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Incorrect policy edits only the source-edit buffer

- ID: `bad-source-edit-buffer-only`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Correct policy refiles a subtree while preserving metadata and children

- ID: `correct-refile-subtree`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Incorrect policy copies during refile and duplicates the ID

- ID: `bad-refile-copy-duplicates-id`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Incorrect policy moves only visible text and drops Org metadata

- ID: `bad-refile-text-loses-metadata`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Correct policy executes a Babel block and grades the parent Org buffer

- ID: `correct-babel-result-parent`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

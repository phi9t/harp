# Dired Benchmark Summary

- Generated: 2026-09-28T08:19:40Z
- Benchmark: `dired-filesystem`
- Total scenarios: 6
- Scenarios as expected: 6
- Insula checks passed: `t`
- Mode: `selected-container-rootfs`
- Run id: `20260928T081940Z-429041`

## Correct policy deletes the Dired marked file set

- ID: `correct-marked-delete`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Incorrect policy confuses deletion flags with ordinary Dired marks

- ID: `bad-flagged-delete-confused-with-marked`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Correct policy renames a Dired file without a collision

- ID: `correct-rename-no-collision`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Correct policy refuses a Dired rename collision without overwrite

- ID: `collision-rejected-no-overwrite`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Incorrect policy overwrites an existing file during a Dired rename

- ID: `bad-collision-overwrite`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Incorrect policy treats Dired undo as filesystem recovery

- ID: `bad-dired-undo-filesystem-recovery`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

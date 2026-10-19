# Dired Benchmark Summary

- Generated: 2026-09-29T04:06:19Z
- Benchmark: `dired-filesystem`
- Total scenarios: 8
- Scenarios as expected: 8
- Insula checks passed: `t`
- Mode: `selected-container-rootfs`
- Run id: `20260929T040533Z-3771476`

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

## Correct policy revalidates Dired marks after an external same-path replacement

- ID: `stale-mark-revalidated-after-external-replace`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Incorrect policy trusts a stale Dired mark after external replacement

- ID: `bad-stale-mark-deletes-external-replacement`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

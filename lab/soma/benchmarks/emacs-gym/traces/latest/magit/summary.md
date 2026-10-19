# Magit Benchmark Summary

- Generated: 2026-09-29T04:05:35Z
- Benchmark: `magit-selective-staging`
- Total scenarios: 7
- Scenarios as expected: 7
- Insula checks passed: `t`
- Mode: `selected-container-rootfs`
- Run id: `20260929T040533Z-3771476`

## Correct policy stages only the selected Magit hunk

- ID: `correct-hunk-stage`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Correct policy stages the selected Magit file section

- ID: `correct-file-stage`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Correct policy stages an active region of Magit file sections

- ID: `correct-region-files-stage`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Incorrect policy bypasses Magit and stages too much with git add -u

- ID: `bad-git-add-all`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Incorrect policy stages the right file through a bridge wrapper

- ID: `bad-wrapper-command-context`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Incorrect policy selects the unstaged list and stages too much

- ID: `bad-unstaged-list-stage`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Incorrect policy acts through a stale Magit view after external edit

- ID: `bad-stale-view-stage`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

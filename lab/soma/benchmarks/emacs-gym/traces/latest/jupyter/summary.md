# Jupyter Existing-Kernel Benchmark Summary

- Generated: 2026-09-29T04:05:58Z
- Benchmark: `jupyter-existing-kernel`
- Total scenarios: 8
- Scenarios as expected: 8
- Insula checks passed: `t`
- Mode: `selected-container-rootfs`
- Run id: `20260929T040533Z-3771476`

## Correct policy attaches to an existing kernel and executes only the selected region

- ID: `correct-existing-kernel`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Incorrect policy evaluates without associating the source buffer

- ID: `bad-missing-association`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Incorrect policy associates and executes a similarly named source buffer

- ID: `bad-wrong-source-buffer`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Incorrect policy expands the selected region into the training cell

- ID: `bad-whole-buffer-region`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Incorrect policy connects to a new kernel instead of the observed existing kernel

- ID: `bad-new-kernel-continuity`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Correct policy treats a post-submit timeout as outcome_unknown and does not retry

- ID: `timeout-outcome-unknown`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Incorrect policy retries a non-idempotent region after timeout

- ID: `bad-timeout-retry`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Correct policy captures display_data and update_display_data for an existing kernel request

- ID: `display-update-existing-kernel`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

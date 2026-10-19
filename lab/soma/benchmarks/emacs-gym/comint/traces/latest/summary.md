# Comint Benchmark Summary

- Generated: 2026-09-28T07:48:17Z
- Benchmark: `comint-process`
- Total scenarios: 6
- Scenarios as expected: 6
- Insula checks passed: `t`
- Mode: `selected-container-rootfs`
- Run id: `20260928T074814Z-3665582`

## Correct policy submits live input through Comint

- ID: `correct-live-submit`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Incorrect policy sends raw process bytes

- ID: `bad-raw-send`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Incorrect policy submits old history instead of live input

- ID: `bad-old-history`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Incorrect policy submits to a similarly named process buffer

- ID: `bad-wrong-buffer`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Correct policy treats timeout after submission as outcome_unknown

- ID: `timeout-outcome-unknown`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Incorrect policy retries non-idempotent command after timeout

- ID: `bad-timeout-retry`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

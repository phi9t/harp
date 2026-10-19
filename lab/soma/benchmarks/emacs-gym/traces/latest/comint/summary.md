# Comint Benchmark Summary

- Generated: 2026-09-29T04:05:38Z
- Benchmark: `comint-process`
- Total scenarios: 11
- Scenarios as expected: 11
- Insula checks passed: `t`
- Mode: `selected-container-rootfs`
- Run id: `20260929T040533Z-3771476`

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

## Correct policy reobserves the process mark after asynchronous output

- ID: `process-mark-drift-reobserve`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Incorrect policy submits using a process mark observed before asynchronous output

- ID: `bad-stale-process-mark-submit`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Correct policy waits for final evidence after partial output

- ID: `partial-output-collect`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## Incorrect policy treats partial output as command completion

- ID: `bad-partial-output-premature-done`
- Expected policy pass: `:json-false`
- Policy passed grader: `:json-false`
- As expected: `t`

## Correct policy interrupts after partial output and reconciles state

- ID: `interrupt-partial-output-reconcile`
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

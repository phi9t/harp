# Bridge Transport Benchmark Summary

- Generated: 2026-09-29T04:06:19Z
- Benchmark: `bridge-transport`
- Run id: `20260929T040533Z-3771476`
- Total scenarios: 16
- Scenarios as expected: 16
- Insula checks passed: `t`
- Passed: `t`

## envelope-observe

- Title: Request envelope observes a buffer and stores terminal status
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## command-dispatch

- Title: Request envelope executes an allowlisted interactive command through checked command dispatch
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## selected-window-command-rejected

- Title: A command.dispatch request is rejected when the selected window no longer displays the target buffer
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## buffer-read-purity

- Title: A buffer.read request returns text without moving point, mark, revision, or selected window state
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## checked-edit-save

- Title: A checked buffer edit mutates only the expected range and save writes the visited file
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## stale-edit-rejected

- Title: A buffer.replace_range request with a stale chars_tick is rejected without mutation
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## point-edit-rejected

- Title: A buffer.replace_range request is rejected when point changed without a text revision
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## narrowed-edit-rejected

- Title: A buffer.replace_range request is rejected when narrowing changed without a text revision
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## mark-active-edit-rejected

- Title: A buffer.replace_range request is rejected when active-region state changed without a text revision
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## read-only-edit-rejected

- Title: A buffer.replace_range request is rejected when read-only state changed without a text revision
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## completion-accept

- Title: A completion.accept request inserts a suggestion only when range, revision, point, prefix, and suffix match
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## stale-completion-rejected

- Title: A stale completion.accept request is rejected after intervening human editing
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## episode-finish

- Title: An episode.finish request succeeds only after required requests and final buffer evidence pass
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## duplicate-request

- Title: A duplicate request id is rejected without replacing the stored request
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## direct-call-rejected

- Title: Model-facing requests cannot invoke unchecked direct-call operations
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## socket-jsonl-roundtrip

- Title: A JSONL Unix-domain socket request returns a structured bridge response
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

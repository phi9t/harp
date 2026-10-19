# Bridge Transport Benchmark Summary

- Generated: 2026-10-01T06:29:25Z
- Benchmark: `bridge-transport`
- Run id: `20261001T062920Z-2817516`
- Total scenarios: 29
- Scenarios as expected: 29
- Insula checks passed: `t`
- Passed: `t`

## envelope-observe

- Title: Request envelope observes a buffer and stores terminal status
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## api-manifest-introspection

- Title: The bridge exposes a machine-readable stable API manifest and operation descriptions
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## observe-session

- Title: Stable core-v0 session observation anchors Emacs, Doom, selected rootfs, and frame identity
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

## checked-apply-patch

- Title: A buffer.apply_patch request applies multiple exact hunks and rejects stale context
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

## project-api

- Title: Stable coding-v0 project operations observe Git project identity and search through rootfs-owned ripgrep
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## reports-vcs-files-api

- Title: Stable reports-v0 and vcs-files-v0 operations observe Org/Dired state, export HTML, and reject unsafe file collisions
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## comint-semantic-api

- Title: Stable terminal-v0 Comint APIs submit input through Comint, await output, and reject stale process-mark ownership
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## comint-process-ownership-rejected

- Title: Stable terminal-v0 Comint operations reject live processes outside the selected rootfs
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## comint-interrupt-api

- Title: Stable terminal-v0 Comint API interrupts the intended subprocess and reconciles side effects
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## magit-semantic-api

- Title: Stable vcs-files-v0 Magit APIs stage intended hunks/files through Magit and reject stale sections
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## lsp-envelope-projection

- Title: Stable coding-v0 LSP operations publish typed evidence, observations, and workspace-edit effects
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## lsp-server-ownership-rejected

- Title: Stable coding-v0 LSP operations reject live server processes outside the selected rootfs
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## request-envelope-validation

- Title: The bridge validates api_version, capability, preconditions, and timeout_ms in the v0 request envelope
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

## response-envelope-validation

- Title: Captured bridge responses recursively satisfy the v0 result envelope published by the operation manifest
- Expected policy pass: `t`
- Policy passed grader: `t`
- As expected: `t`

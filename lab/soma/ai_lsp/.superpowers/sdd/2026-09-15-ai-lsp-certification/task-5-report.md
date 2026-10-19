# Task 5 Report: Structured AI/LSP Verification

## Outcome

Implemented structured per-language live verification, failure aggregation,
certification mutations, structured result entrypoints, and bounded LSP child
process teardown.  The compatibility loader now contains only relative module
loads and public entrypoints; the legacy PASS buffer remains populated through
Task 6.

Commit: `feat: aggregate exact AI LSP verification results`

## TDD evidence

Inherited RED reproduced with:

```text
DEVX_AI_LSP_RESULT_FILE=/tmp/... bin/verify-ai-lsp unit
{"kind":"unit","status":"FAIL","unit":{"completed":36,"unexpected":1}}
```

An isolated Doom ERT run identified
`devx-ai-unit-public-entrypoints-write-results-and-return-t` as the unexpected
result.  Its failure was `wrong-type-argument listp`: result records are a
JSON vector, while `devx-ai-test-populate-legacy-results` assumed a list.

The existing named contract was then green after normalizing the input sequence:

```text
devx-ai-unit-public-entrypoints-write-results-and-return-t => ert-test-passed
```

Live RED found all functional language gates passing but all four observed LSP
children alive after cleanup.  Source inspection showed `lsp-disconnect` calls
its close hook with `keep-workspace-alive=t`, so it intentionally does not
shutdown a workspace.  The focused teardown contract was extended first and
failed:

```text
devx-ai-unit-cleanup-records-stopped-child => ert-test-failed
```

The repair adds the adapter-owned public shutdown seam, suppressing Doom's
deferred shutdown only during certification cleanup.  The focused test then
passed:

```text
devx-ai-unit-cleanup-records-stopped-child => ert-test-passed
```

## Verification

```text
DEVX_AI_LSP_RESULT_FILE=/tmp/... bin/verify-ai-lsp unit
t
{"kind":"unit","status":"PASS","unit":{"completed":36,"unexpected":0}}

DEVX_AI_LSP_RESULT_FILE=/tmp/... bin/verify-ai-lsp integration
t
PASS Rust   rust-analyzer + Copilot rust
PASS Python pyright + Copilot python
PASS C++    clangd + Copilot cpp
PASS CUDA   clangd + Copilot cuda-cpp
```

The live JSON schema check passed: result schema/kind/status were
`devx.ai-lsp.result/v1` / `integration` / `PASS`; it had exactly four records,
each with exactly the ten specified record keys and the eight required gates;
it had zero failures and four teardown records, each with
`alive-after-cleanup: null` (the JSON representation of Lisp nil / not alive).

Live matrix:

| Language | Server | Language ID | Gates | Child after cleanup |
| --- | --- | --- | --- | --- |
| Rust | rust-analyzer | rust | 8 PASS | stopped |
| Python | pyright | python | 8 PASS | stopped |
| C++ | clangd | cpp | 8 PASS | stopped |
| CUDA | clangd | cuda-cpp | 8 PASS | stopped |

The public combined path also passed:

```text
DEVX_AI_LSP_RESULT_FILE=/tmp/... bin/verify-ai-lsp all
t
PASS Rust   rust-analyzer + Copilot rust
PASS Python pyright + Copilot python
PASS C++    clangd + Copilot cpp
PASS CUDA   clangd + Copilot cuda-cpp
{"kind":"all","status":"PASS","unit":{"completed":36,"unexpected":0},
 "records":4,"failures":0,"teardown":4}
```

`git diff --check` passed.  A source-ownership check found no ERT definitions
or direct `copilot--`/`lsp--` symbols in `ai-lsp-integration.el`; the
compatibility loader is relative loads plus entrypoints/provide only.  No
duplicate ERT or `devx-ai-test-` function definitions remain among the Task 5
modules.

## Organization

```text
test/ai-lsp-support.el       329 lines  result shapes, status, JSON/PASS support
test/ai-lsp-adapter.el       127 lines  single external API boundary and shutdown seam
test/ai-lsp-unit.el          789 lines  unit contracts only
test/ai-lsp-integration.el   563 lines  live orchestration only
test/ai-lsp-verifier.el       32 lines  compatibility loader and public entrypoints
```

The prior verifier tests were moved to the unit module rather than duplicated;
the obsolete verifier-owned implementation was removed.

## Concerns

None for Task 5.  Task 6 intentionally continues to consume the transitional
PASS buffer; Task 7 will become the trusted JSON-status consumer and remove
that path.

## Fix round 1

Addressed the three Important review findings only.

### RED/GREEN

The focused RED run reported all four covering contracts as failures:

```text
devx-ai-unit-copilot-transport-errors-are-external-only => ert-test-failed
devx-ai-unit-copilot-auth-classifies-only-valid-unavailability => ert-test-failed
devx-ai-unit-cleanup-records-stopped-child => ert-test-failed
devx-ai-unit-cleanup-reports-vetoed-buffer-kill => ert-test-failed
```

The first two encode the typed/allowlisted external classification boundary:
only a `jsonrpc-error` whose structured message is `Timed out` blocks
externally; `(wrong-type-argument stringp "service unavailable")` remains a
local failure.  `NotSignedIn`, `Unauthorized`, and `ServiceUnavailable` remain
external authentication states, while an unknown status is local FAIL.

The cleanup RED contracts require a stopped process to have Lisp
`:json-false`, verify that JSON parses it as boolean false, and require a
non-signaling `kill-buffer` veto to produce the existing local cleanup shape.

After the minimal implementation changes, the focused GREEN run was:

```text
devx-ai-unit-copilot-transport-errors-are-external-only => ert-test-passed
devx-ai-unit-copilot-auth-classifies-only-valid-unavailability => ert-test-passed
devx-ai-unit-cleanup-records-stopped-child => ert-test-passed
devx-ai-unit-cleanup-reports-vetoed-buffer-kill => ert-test-passed
```

### Verification

```text
DEVX_AI_LSP_RESULT_FILE=/tmp/... bin/verify-ai-lsp unit
t
{"kind":"unit","status":"PASS","unit":{"completed":37,"unexpected":0}}

DEVX_AI_LSP_RESULT_FILE=/tmp/... bin/verify-ai-lsp integration
t
PASS Rust   rust-analyzer + Copilot rust
PASS Python pyright + Copilot python
PASS C++    clangd + Copilot cpp
PASS CUDA   clangd + Copilot cuda-cpp
```

The exported integration JSON passed:

```jq
.kind == "integration" and .status == "PASS" and
(.records | length == 4) and (.failures | length == 0) and
(.teardown | length == 4) and
([.teardown[]."alive-after-cleanup"] | all(. == false))
```

Its four teardown records all contain the literal JSON field
`"alive-after-cleanup": false`, not `null`.

## Fix round 2

Closed the two remaining proof gaps.  The covering test is
`devx-ai-unit-copilot-transport-errors-are-external-only`; it now asserts that
`(wrong-type-argument stringp "service unavailable")` is re-signaled as
`wrong-type-argument`, not merely classified by a broad `error` handler.

### Current focused command and output

The earlier focused commands were not retained verbatim.  The following is the
exact current focused ERT runner used for both the RED pressure test and the
restored GREEN run; only `task_server_name` and the redirected daemon log name
differ (`round2-red` and `round2-green` respectively):

```bash
task_emacs_bin=/private-source/home/operator/devx/envs/doom-emacs/.spack-env/view/bin/emacs
task_client_bin=/private-source/home/operator/devx/envs/doom-emacs/.spack-env/view/bin/emacsclient
task_server_name=devx-ai-lsp-round2-green-$$
"$task_emacs_bin" --init-directory /private-source/home/operator/.emacs.d "--daemon=$task_server_name" >/tmp/devx-ai-lsp-round2-green.log 2>&1 &
task_daemon_pid=$!
for task_attempt in $(seq 1 30); do "$task_client_bin" -s "$task_server_name" -e '(emacs-pid)' >/dev/null 2>&1 && break; sleep 1; done
"$task_client_bin" -s "$task_server_name" -e "(progn (load-file \"/private-source/home/operator/workspace/devx/doom/ai-lsp-verifier.el\") (type-of (ert-run-test (ert-get-test 'devx-ai-unit-copilot-transport-errors-are-external-only))))"
task_result=$?
printf 'focused-exit=%d\n' "$task_result"
"$task_client_bin" -s "$task_server_name" -e '(kill-emacs)' >/dev/null 2>&1 || true
wait "$task_daemon_pid" || true
exit "$task_result"
```

The runner intentionally returns the ERT result type as its Emacsclient value,
so its shell exit status is zero in both cases; the ERT status is the printed
value.

```text
# RED: temporary production mutation changed
# (signal (car error-data) (cdr error-data))
# to (signal 'error (cdr error-data))
ert-test-failed
focused-exit=0

# GREEN: restored condition-preserving production code
ert-test-passed
focused-exit=0
```

The full required unit command was also run:

```text
DEVX_AI_LSP_RESULT_FILE=/tmp/... bin/verify-ai-lsp unit
t
{"kind":"unit","status":"PASS","unit":{"completed":37,"unexpected":0}}
unit-exit=0
```

No production change was needed: the pressure mutation was removed before the
GREEN run.  Live integration was not rerun because this round changes only the
unit assertion and report evidence.

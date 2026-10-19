# Fail-Closed AI/LSP Certification Design

## Purpose

This design turns the existing Doom Emacs AI/LSP verifier into a repeatable,
fail-closed certification system. A passing certificate must prove both the
current Rust/Python/C++/CUDA happy path and the negative properties that make
the result trustworthy: exact protocol validation, complete failure
aggregation, ownership-safe bounded process cleanup, credential-safe
diagnostics, and reproducible environment evidence.

The runtime behavior established by the original design remains unchanged.
Copilot is the automatic inline-completion backend for eligible buffers,
Tabbymacs is manually selectable, and `lsp-mode` owns only the
language-specific analysis server.

## Scope

The change covers:

- strict unit and integration contracts;
- an isolated compatibility adapter around private Copilot and lsp-mode APIs;
- a launcher whose timeout owns the daemon process rather than only the client;
- automated launcher failure-path tests;
- a full certification command covering Doom and live integration health;
- a sanitized machine-readable certificate tied to exact versions and hashes;
- package pinning for repository-declared Copilot and Tabbymacs dependencies.

It does not add languages, change completion keys, make Tabbymacs automatic,
require a CUDA toolkit, publish certificates remotely, or make Copilot network
output deterministic.

## Repository Layout

```text
devx/
├── bin/
│   ├── verify-ai-lsp          # unit/live verifier launcher
│   └── certify-ai-lsp         # complete release certification pipeline
├── doom/
│   ├── ai-completion.el
│   └── packages.el
├── test/
│   ├── ai-lsp-adapter.el      # private Copilot/lsp-mode compatibility boundary
│   ├── ai-lsp-support.el      # fixtures, schemas, sanitization, result model
│   ├── ai-lsp-unit.el         # ERT contracts
│   ├── ai-lsp-integration.el  # real per-language orchestration
│   ├── ai-lsp-verifier.el     # compatibility loader and public entry points
│   └── verify-ai-lsp-launcher # shell launcher contract suite
├── certificates/              # ignored, persistent local certificate output
├── README.md
└── docs/superpowers/
```

Each module has one reason to change. The compatibility loader preserves the
existing `test/ai-lsp-verifier.el` entry path while moving behavior into
focused modules.

## Language Catalog and Independent Expectations

One structured catalog in the test support module owns operational metadata:
name, mode, expected LSP server, dependency resolver, fixture files, source
file, and Copilot language ID. Production Copilot aliases remain in
`doom/ai-completion.el`.

The unit contract keeps a separate literal mode-to-language-ID expectation.
This intentional duplication prevents a shared bad catalog value from making
both production and test expectations wrong in the same way. The launcher's
expected result is structured data rather than a second set of hand-formatted
PASS strings.

The certified matrix remains:

| Language | Emacs mode | LSP server | Copilot ID |
| --- | --- | --- | --- |
| Rust | `rustic-mode` | `rust-analyzer` | `rust` |
| Python | `python-mode` | `pyright` | `python` |
| C++ | `c++-mode` | `clangd` | `cpp` |
| CUDA | `cuda-mode` | `clangd` | `cuda-cpp` |

## Private API Compatibility Boundary

`test/ai-lsp-adapter.el` is the only file allowed to call `copilot--*` or
`lsp--*` APIs. It exposes stable project-local operations for:

- Copilot language ID, connection state, authentication, document opening,
  and inline-completion requests;
- LSP startup, workspace server IDs/statuses, process commands, and session
  reset;
- installed-server resolution.

The adapter checks that every required private symbol has the expected
function or macro shape before integration begins. A missing or changed API is
reported as an incompatible dependency, never as a language failure or pass.

The repository pins Copilot and Tabbymacs to the commits certified by this
change. Dependency upgrades deliberately update the pin, compatibility probes,
and certificate together. Doom continues to pin its own lsp-mode dependency;
the certificate records the actual lsp-mode and Doom commits.

## Exact Completion Protocol Validation

The raw result of `textDocument/inlineCompletion` is valid only when it is:

- top-level JSON null, represented by `nil`;
- a vector of valid inline-completion items; or
- an InlineCompletionList plist with a present `:items` JSON array.

`(:items nil)` is invalid because it represents a null required member, not an
empty JSON array. A proper list may be accepted only at a test seam where it is
unambiguously an array; raw JSON-RPC results require vectors.

Every nonempty item requires string `:insertText`. Optional `:filterText` must
be a string, optional `:range` must contain non-negative line/character start
and end positions, and an optional `:command` must be a non-null plist with
string `:title` and `:command`. If `:arguments` is present it must be a JSON
array representation. Unknown extension fields are allowed.

Malformed responses fail without printing the response body. Diagnostics name
only its structural type and failed boundary.

## LSP and Copilot Assertions

For each temporary language project, certification proves in the same buffer:

1. Doom selects the expected major mode.
2. Copilot mode is enabled and Tabbymacs mode is disabled.
3. Copilot reports the expected language ID.
4. The expected LSP workspace reaches `initialized`.
5. No unexpected workspace, including an lsp-mode Copilot client, is attached.
6. The workspace process command resolves to the preflight executable.
7. Copilot's own server connection is alive and authenticated.
8. A real inline-completion request finishes within its timeout and returns a
   strictly valid response.

Dependency resolution remains per server and is mapped back to every affected
language. A missing shared Clangd reports both C++ and CUDA failures while Rust
and Python still run. All language failures are aggregated into one final
error.

The integration cleanup waits for project workspaces to shut down and records
their child process state before the daemon lifecycle layer terminates the
owned process group.

## Fail-Closed Daemon Lifecycle

The launcher starts Emacs as an owned foreground daemon in a new process
session, using Linux `setsid` and Emacs `--fg-daemon=<unique-name>`. The shell
captures the daemon PID and process-group ID before any client request. The
daemon name and an environment ownership marker contain a private random nonce.

Startup waits for a bounded identity response containing the exact marker and
PID. Every external wait uses both a TERM deadline and a short KILL grace
period. A timed-out `emacsclient` is never treated as cancellation of the
server-side evaluation.

The live client bound must exceed the sum of the four languages' LSP,
authentication, and completion bounds plus a 60-second orchestration margin.
The default is 360 seconds for the current 30+15+30 second per-language limits;
an override below that computed floor is rejected instead of silently breaking
failure aggregation.

Cleanup follows this order:

1. request graceful `kill-emacs` when the daemon responds and ownership still
   matches;
2. wait a bounded interval for the owned process group to exit;
3. send TERM to the originally captured process group when it remains alive;
4. send KILL after the grace period;
5. verify that no process in the owned group remains;
6. remove only the launcher's validated temporary root.

Direct signals are permitted only for the originally captured process group
whose foreground daemon PID and ownership marker were validated. A collision,
PID mismatch, or missing marker is a failure and never authorizes killing an
unowned daemon. INT and TERM traps exit with conventional nonzero signal
statuses.

## Credential-Safe Diagnostics

Raw daemon or Copilot output is never written to disk. Daemon stdout/stderr is
streamed through a credential sanitizer before entering the private diagnostic
file. The private run directory is mode `0700`; sanitized files are mode
`0600`.

Buffer diagnostics start on a complete line boundary. If the bounded source
window begins inside an oversized line, that line is omitted rather than
emitting a suffix. Complete selected lines are sanitized before the final
4,000-character tail is taken. Sanitization covers authorization headers,
Bearer values, named access/refresh/OAuth token fields, GitHub token forms,
quoted JSON token values, and long opaque credential-shaped values.

Tests place fake credentials before, inside, and across every truncation
boundary. No test reads the real Copilot authentication database or token.

Failure diagnostics consistently include language, boundary, expected value,
observed value, resolved executable, observed workspace IDs/statuses, and a
relevant sanitized log summary when available. Successful output contains only
allowlisted structured results.

## Launcher Result Contract

The Emacs entry point must return exactly `t`; `nil` is a failure even though
`emacsclient` itself exits zero. Integration results are serialized into a
verifier-owned structured file containing language, mode, server, language ID,
and gate statuses. The shell validates the schema and exact catalog membership
before rendering human-readable PASS lines.

Unit mode reports the number of ERT tests and unexpected results. Integration
mode reports each matrix row. The compatibility `all` mode remains unit plus
live integration. The complete release gate is the separate
`bin/certify-ai-lsp` command so existing scripts do not silently acquire Doom
package mutations.

## Automated Launcher Contract Suite

`test/verify-ai-lsp-launcher` uses disposable fake Emacs and emacsclient
programs; it never addresses a real user daemon. It proves:

- exact-`t` success and `nil` rejection;
- invalid mode and invalid timeout rejection;
- startup failure and TERM-ignoring startup force-kill;
- client timeout followed by owned process-group termination;
- same-name collision rejection without killing the sentinel;
- PID or ownership-marker mismatch rejection;
- INT and TERM return nonzero signal statuses;
- result-schema or matrix mismatch rejection;
- diagnostic tail bounds and credential redaction at cutoff boundaries;
- validated temporary-root cleanup only.

These replace the one-off shell probes previously recorded only in ignored SDD
reports.

## Complete Certification Pipeline

`bin/certify-ai-lsp` runs these gates in order and stops only when a prerequisite
makes later work meaningless:

1. **Static:** repository cleanliness, shell syntax, Emacs Lisp byte
   compilation, checked-loader validation, and dependency pin presence.
2. **Unit:** all ERT contracts, including both backend-switch directions,
   disable-both behavior, minibuffer exclusion, strict response schemas,
   aggregation, diagnostics, and adapter compatibility.
3. **Launcher:** the automated fake-process lifecycle/security suite.
4. **Doom:** `doom sync --force` and `doom doctor`; unrelated warnings are
   recorded but AI/LSP/config errors fail.
5. **Fresh startup:** scratch has neither Copilot mode nor an opened Copilot
   document, no Copilot process exists, and the daemon is then cleaned up.
6. **Live matrix:** interpreted and byte-compiled verifier runs for all four
   languages, including exact workspace and executable checks.
7. **Mutations:** wrong expected server and one missing shared dependency both
   fail while later runnable languages still produce results.
8. **Teardown:** the owned daemon process group and all child language servers
   are gone.
9. **Attestation:** emit the sanitized certificate only after every mandatory
   gate passes.

An unavailable or unauthenticated external Copilot service produces
`BLOCKED_EXTERNAL` with a distinct nonzero exit code. It is never converted to
PASS. Code, schema, ownership, cleanup, and credential-safety failures produce
`FAIL`.

## Certificate Artifact

Each run writes `certificates/<UTC timestamp>-<git SHA>.json` and prints its
path. `certificates/` is git-ignored but is not part of the launcher's temporary
cleanup, so evidence survives the run until the user removes it.

The certificate contains only trusted, allowlisted fields:

- schema version, UTC time, result, repository commit and dirty state;
- hashes of the two repository config files and the two external loader
  snippets;
- Emacs, Doom, copilot.el, Copilot server, Tabbymacs, lsp-mode,
  rust-analyzer, Pyright, and Clangd versions/commits;
- per-gate status and duration;
- per-language expected/observed mode, server, executable, Copilot ID, and
  request status;
- teardown result and non-sensitive Doom warning summaries.

It never embeds raw daemon/LSP/Copilot logs, arbitrary exception objects,
environment dumps, auth paths, database contents, headers, or tokens. A failed
run writes a smaller sanitized failure artifact with completed gates and the
failed boundary.

## Testing Strategy

Every correction follows red-green-refactor:

1. add focused ERT failures for schema, selector, workspace, executable, and
   diagnostic contracts;
2. add shell failures for exact return handling, signals, hard timeouts,
   collision ownership, cleanup, and streaming redaction;
3. implement the smallest production change that makes each focused contract
   pass;
4. split the verifier only after the behavioral contracts are green;
5. run unit and launcher suites after each refactor;
6. run the real matrix, mutation gates, and full certification once on the
   final tree;
7. independently review the whole diff and certificate.

Tests assert externally visible behavior. Fake executables are used only at
the process boundary where real hung or colliding daemons would be unsafe.
Live LSP and Copilot behavior remains covered by the real integration gate.

## Acceptance Criteria

The redesign is complete only when:

- every focused RED test was observed failing for the intended reason;
- all unit and launcher contract tests pass with clean output;
- malformed completion and diagnostic-boundary probes fail closed;
- hung client/server and collision probes leave no owned or unowned process
  disturbed incorrectly;
- exact-return, extra-workspace, executable-path, selector, and minibuffer
  contracts pass;
- `doom sync --force`, `doom doctor`, and fresh scratch certification pass;
- interpreted and compiled Rust/Python/C++/CUDA live matrices pass;
- mutation gates prove aggregation after wrong-server and missing-Clangd
  failures;
- teardown confirms no owned daemon or child language server remains;
- a credential-safe certificate is written with exact environment identity;
- final review reports no Critical or Important findings.

The feature branch remains unmerged until all criteria are satisfied.

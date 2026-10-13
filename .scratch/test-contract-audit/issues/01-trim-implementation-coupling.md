# Trim tests that freeze implementation choices

Status: done
Owner: codex
Branch: codex/harp-test-audit

## What to build

Audit existing Harp tests using the test-selection rule in AGENTS.md. Remove or
rewrite incidental implementation assertions, preserve API/schema contracts and
meaningful invariants, use focused checks, and land locally.

## Acceptance criteria

- Screen test families for source-shape, private structure, styling, incidental
  call order/count, and fixed product-roster assertions.
- Trim only clearly incidental assertions; retain contract and safety coverage.
- Record focused results and environment limits, preserve existing changes, and
  land the candidate locally.

## Blocked by

None for the authorized focused landing. Some broader checks require existing
machine caches, as documented below.

## Audit decisions

The audit screened Rust inline/integration tests, Atlas component/schema/export
tests, Python lab and script tests, shell task checks, and PDF tooling tests.
The retained suite is not a claim that every assertion is ideal: semantic prose,
proof trust, and operational constraints need their explicit requirements before
further deletion.

| Surface | Trimmed | Retained contract coverage |
| --- | --- | --- |
| Atlas dashboard | CSS parser/helpers, literal colors/layout/breakpoints/radii, source imports/classes, exact Lucide identities and chart classes | Navigation, filtering, keyboard/focus behavior, lifecycle disclosure, accessible chart metadata, schema rejection; watchlist keyboard focus hook retained |
| Atlas reference schema | Fixed reference titles and roster counts | Reference roundtrip plus invalid schema, identity and stage cases |
| Offline export | Private client build intermediate, minifier object-literal spellings, duplicate prose/roster inventory | Delivered HTML and corpus digest binding, export schema, embedded code/style and offline boundary; diagnosis wire formats and export capabilities |
| Rust corpus | Hardcoded source-symbol and validator-owner scans; current static roster counts | Registration views with controlled input, duplicate rejection, actual corpus routes/aliases/rendering and filesystem safety |
| Rust context/release/routing | Source-declaration parsing and forbidden API spelling scans | Validated release accessors, wire schema/version/integrity, routed-request authority checks, deterministic routing/context results; versioned renderer golden remains a persisted format contract |
| Artifact reader | Ban on a particular read API | Actual oversized-input rejection |
| Conjecture packet | Registry/search/TypeScript source-string checks | Compiled corpus route checks and packet/link contracts |
| Python publication | Banned traversal/lock helper spellings, exact internal validation call count, shared close-helper invocation, private limit literal | Bounded traversal/cleanup, nonblocking publication, mutation/identity checks, symlink and directory replacement rejection, public evidence API checks |
| Python import/count projection | Import fallback spelling and bullet indentation | Package/direct execution compatibility, published counts, projection idempotence and ambiguity rejection |
| PDF editions | Array reference identity | Edition contents, chapter selection, supplement placement and incomplete input rejection |

Retained API signatures, strict JSON/schema and migration tests, real process and
filesystem behavior, proof/receipt integrity, and bounded readers. Source scans
that enforce explicit offline or proof-escape policies were retained; their
purpose differs from asserting a private helper’s spelling. No runtime behavior,
proof source, captured evidence, generated artifact, or verification task changed.
The already-dirty registration test and runtime edits in the primary checkout
remain outside this landing.

## Verification

- Rust (offline dependencies, private temporary target): corpus tests 69 passed;
  context-control tests 233 passed; harp-contracts schema tests 39 passed;
  actual oversized-reader test 1 passed; conjecture packet integration 3 passed.
- Python route publication + count projection: 47 passed. Local-formalization
  tests: 34 passed with a canonical disposable XDG directory and inert executable
  fixture files for the existing FakeExecutor. This verifies fixture behavior,
  not live Lean execution; no cache hydration or installation occurred.
- Atlas reference-schema suite: 16 passed using cached Vitest 5.0.1 and a temporary
  node-only configuration. Repository dependency versions were not changed;
  this is not the pinned full Atlas gate.
- Harp route tests excluding the retained live-cache receipt check: 11 passed.
- Node offline export: 4 passed against the committed artifact. PDF input/edition
  checks: 10 passed.
- Edited TS/TSX files parsed/transformed with cached Vite/Oxc. React dashboard
  tests were not executed: the project’s React/jsdom dependency cache is absent.
- Initial broader Python run encountered missing shared Lean cache and live
  receipt prerequisites. An original committed test reproduced the missing-cache
  failure. Those checks were retained, not deleted to hide failures.
- Final diff and whitespace review passed. Edited Rust production sections are
  byte-identical to the starting commit. Preservation checks run at local landing.
- Removed 13 whole implementation-coupled tests, rewrote several assertions, and
  removed approximately 630 net lines from test code.

## Comments

The owner explicitly authorized focused checks for this audit and local landing.
No full repository build, dependency maintenance, push, or broad test-count target
is part of this change. The commit containing this ticket is the landing record.

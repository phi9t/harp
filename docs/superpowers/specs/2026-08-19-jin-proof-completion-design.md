# Jin proof completion design

## Objective

Finish the Jin route for the Crouzeix proof by porting the proof into Harp's
shared Lean root. The route uses Jin's original repository, theorem names, and
environment locks as reference evidence, but the proof that lands in Harp must
compile under `formalization/lean` with Harp's consolidated Lean/mathlib
environment.

This design is intentionally narrow. It turns the current Jin source-map rows
from reference-aware blocked artifacts into Harp-native proof obligations with
sealed receipts. It does not try to revive Jin's original Lake project as a
runtime dependency, and it does not claim proof progress from `#check` adapters
or imported upstream terminal theorems.

## Current state

The shared Lean root is:

```text
formalization/lean/
```

It is the only target runtime for new proof work. The package is
`harp_formalization`, using Lean/mathlib `v4.32.1`. The verifier runs through
`scripts/check_lean_library.sh`, with task-scoped Lean state under:

```text
ELAN_HOME=/private/tmp/harp-mathematical-foundations-elan
```

The pinned Jin reference target is:

```text
labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/
```

The reference lock records:

- Jin commit `565b6a3e0659b6e0785f783b016c3f6d9f171fa5`;
- original Lean toolchain `leanprover/lean4:v4.28.0`;
- original mathlib revision
  `8f9d9cff6bd728b17a24e163c9402775d9e6a365`;
- original archive SHA-256
  `33ee5b75c1037866c4d2bda8eff872cc0640fd42e7c31c58a8e6047405f69542`;
- source-tree digest `40aafa503bd32762dbf6d1a67ddef3e2b067f0e1`;
- original theorem `CrouzeixConjecture.crouzeixConjecture`; and
- allowlist `CrouzeixConjecture.FinalTheorems` for reference validation.

The current Jin source map has three rows:

```text
jin-max-polynomial-modulus  blocked  missing executable: lake
jin-polynomial-bound        mapped   depends on jin-max-polynomial-modulus
jin-terminal-crouzeix       mapped   depends on jin-polynomial-bound
```

The only materialized proof slice is `attempt-002` for
`jin-max-polynomial-modulus`. It contains a `#check` against the upstream Jin
declaration, not a Harp-owned proof body. Its receipt is useful because it
records the runner defect, but it is not mathematical progress.

## Non-goals

- Do not import `CrouzeixConjecture.FinalTheorems` from the Harp shared Lean
  root as proof authority.
- Do not mark a source-map row `passed` from an import-only adapter, upstream
  theorem availability, model transcript, or informal source comparison.
- Do not add `sorry`, `admit`, `axiom`, or an unrecorded trusted declaration.
- Do not block Harp-native proof work on rebuilding Jin's original
  `v4.28.0` environment.
- Do not promote Lorist-Schwenninger or shared-kernel work until the Jin row
  currently being worked has either passed or produced a frozen blocker.

## Recommended architecture

Use two explicit lanes:

```text
Reference lane:
  pinned Jin archive, source locators, original theorem names, original
  toolchain metadata, preflight receipts, source-map provenance

Proof lane:
  Harp-owned Lean statements and proofs under formalization/lean, built by the
  shared Lean wrapper and recorded by sealed proof-slice receipts
```

The reference lane explains what must be proved and provides comparison data.
The proof lane is the only lane that can make Harp proof progress. A reference
lane command may be useful when disk and cache conditions permit, but its
success is not a substitute for a Harp-native proof receipt.

## Harp Lean surface

Add a Crouzeix library to the shared Lean root when the first Harp-owned proof
module is introduced:

```text
formalization/lean/
  Crouzeix.lean
  Crouzeix/
    Jin/
      MaxPolynomialModulus.lean
      PolynomialBound.lean
      Terminal.lean
```

The `lakefile.toml` change should add a separate `Crouzeix` Lean library and,
once useful, a focused mise task:

```text
mise run lean-crouzeix
mise run lean-all
```

The Crouzeix library must stay part of the consolidated root. It must not add a
new Lake project, a second mathlib cache, or a dependency on another local
checkout. Imports should be as narrow as practical; broad imports are allowed
only when they are needed to keep the proof slice moving and are recorded for
later narrowing.

## Proof ladder

The first completion route is linear and source-map driven:

1. `jin-max-polynomial-modulus`
2. `jin-polynomial-bound`
3. `jin-terminal-crouzeix`

Each row owns one current attempt directory under:

```text
labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/<row-id>/
```

For each row, the implementation flow is:

1. Read the pinned Jin source locator and existing source-map row.
2. State the Harp-native Lean declaration in the Crouzeix library.
3. Import only mathlib, Harp-owned prior passed rows, and narrowly justified
   local Crouzeix support modules.
4. Prove the statement or produce the earliest named blocker.
5. Run a fresh shared-root Lean build through the proof-slice runner.
6. Emit the command receipt, stdout/stderr logs, result, and axiom audit.
7. Update `source-map.json` and `tracker.org` only according to the receipt.

No downstream row starts as proof work until its declared predecessor has
either passed or published a frozen blocker that downstream work can honestly
target.

## Source-map status rules

The Jin source map is the progress ledger. Status transitions are mechanical:

```text
mapped -> active
active -> passed
active -> failed
active -> blocked
blocked -> active
```

`passed` requires all of the following:

- the Harp-owned Lean declaration compiles in `formalization/lean`;
- the proof-slice receipt records the fixed command, cwd, and environment
  allowlist;
- the no-hole policy rejects `sorry` and `admit`;
- no new `axiom` or trusted declaration appears outside the recorded policy;
- the axiom audit is attached to the attempt;
- all dependency receipt digests match; and
- the source-map row points to the exact passed attempt digest.

`blocked` is allowed only for the earliest closed blocker. Examples include a
missing mathlib theorem, a theorem-shape mismatch, an unavailable toolchain, or
a resource constraint. A row blocked by one reason must not hide a later or
broader reason.

`failed` means Lean ran and rejected the proof, or a receipt invariant failed.
Tool discovery failures are `blocked` when they prevent Lean from running.

## First implementation slice

The first implementation issue fixes the current non-mathematical blocker:

```text
attempt-002 reason: missing executable: lake
```

The proof-slice runner should inherit the same execution contract as
`mise run lean-all`:

- cwd `formalization/lean`;
- task-scoped `ELAN_HOME=/private/tmp/harp-mathematical-foundations-elan`;
- `lake` resolved from the verified Lean toolchain path or the same wrapper
  used by the shared Lean verifier;
- process-group timeout cleanup;
- command receipt with argv, cwd, selected environment, and tool versions; and
- result classification that distinguishes tool discovery, Lean compile
  failure, policy failure, and proof success.

The expected next artifact is `attempt-003` for
`jin-max-polynomial-modulus`. A good `attempt-003` may still be blocked or
failed, but it must be a real Lean proof result. It must not end at
`missing executable: lake`.

## Reference-env policy

Jin's original environment remains useful for theorem-shape and proof-term
comparison. It is not a runtime authority for Harp.

Reference-env commands should be recorded under the existing pinned target
directory and classified separately from Harp proof attempts. The current
preflight blocker:

```text
insufficient-disk-for-pinned-mathlib-cache
```

stays a reference-lane blocker. It should not prevent work on the Harp-native
proof lane. If the original environment later becomes available, it can answer
questions like "what was Jin's exact local theorem shape?" or "which mathlib
declaration name changed between versions?", but it still cannot mark a Harp
row passed.

## Reconstruction loop

Use the Lean-guided reconstruction loop only after a row has a frozen Harp
obligation:

```text
fixed imports + fixed theorem statement + fixed local context + source locator
```

Candidate generation may cite Jin's published route because this is
reference-aware validation, not a blind-frontier experiment. Candidates may
fill proof bodies or propose strictly smaller named obligations. They may not
change the theorem statement, broaden imports without recording why, add
trusted declarations, or modify receipt metadata.

A candidate becomes proof progress only after the same fresh build and axiom
audit required for hand-written proof work.

## Shared-kernel promotion

Do not add a shared Crouzeix kernel lemma speculatively. A lemma becomes
eligible for `formal_targets/shared-kernel` only when:

- two route-local consumers need the same normalized statement; or
- one passed Jin row and one passed or blocked Lorist-Schwenninger node expose
  the same reusable bridge; and
- the kernel statement has narrower dependencies than duplicating the bridge
  in both routes.

Until then, Crouzeix support lemmas stay route-local under `Crouzeix/Jin/`.

## Validation gates

Focused iteration before the Crouzeix library exists:

```sh
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v
mise run lean-all
```

After the Crouzeix library and focused task are introduced, focused iteration
should become:

```sh
mise run lean-crouzeix
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_lean_suite -v
```

Full proof gate before landing proof work remains:

```sh
mise run lean-all
mise run verify
```

The exact Python test module can be adjusted to the current test layout, but
the test surface must cover:

- proof-slice command receipt validation;
- source-map dependency receipt validation;
- no-hole and trusted-declaration policy;
- status-transition invariants; and
- stale or missing attempt digests.

Any tracked content change still requires the normal Harp payload digest
refresh in `docs/import-receipt.md` after the tracked payload settles.

## Hygiene

Generated Python caches under `labs/crouzeix_proof_reproduction` are build
artifacts, not proof evidence. They are already covered by the repository's
`__pycache__` and `*.pyc` ignore rules. Future implementation work should
delete generated caches from the active worktree when encountered, never stage
them, and avoid treating their presence or absence as proof-run evidence.

Unrelated local notes or continuation prompts in the primary checkout must be
preserved. Work on this route should happen in isolated worktrees and stage
only coherent repo-owned artifacts.

## Landing criteria

The Jin route is complete only when:

- all three Jin source-map rows are `passed`;
- the terminal Harp theorem compiles from the shared Lean root;
- no forbidden proof holes or unrecorded trusted declarations exist;
- each row has a sealed receipt and axiom audit;
- `tracker.org` records the exact completed row statuses and receipt digests;
- `mise run lean-all` passes; and
- `mise run verify` passes after the final payload digest refresh.

Before that point, the right deliverable is the earliest exact blocker with a
typed receipt, not a broad claim that the Crouzeix proof is nearly finished.

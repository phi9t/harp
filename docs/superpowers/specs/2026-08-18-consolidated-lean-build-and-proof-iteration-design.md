# Consolidated Lean build and proof-iteration design

## Purpose

Harp's Lean proof surface is growing across Training Dynamics, Mathematical
Foundations, NNG4 Intro, and Autodiff Geometry. The current release gate treats
those as four independent Lake projects. Each project pins the same Lean
toolchain and the same mathlib revision, but each owns a separate `.lake`
directory. A cold or partially cold verification therefore spends most of its
time compiling duplicated dependency artifacts instead of checking Harp-owned
proofs.

This design consolidates the Lean build root so dependency work is paid once,
while preserving separate proof libraries, focused checks, no-proof-hole
policies, and the repository release gate. The primary product requirement is
fast proof iteration: after the shared dependency graph is warm, editing one
proof should rebuild that proof file and the smallest downstream Harp-owned
target practical.

## Current behavior

The release gate runs the Lean checks serially:

1. `verify-lean` builds `formalization/training_dynamics`.
2. `verify-mathematical-foundations-lean` builds
   `formalization/mathematical_foundations`.
3. `verify-nng4-intro-lean` builds `formalization/nng4_intro`.
4. `verify-autodiff-geometry-lean` builds `formalization/autodiff_geometry`.

Each project has its own `lakefile.toml`, `lean-toolchain`,
`lake-manifest.json`, and `.lake` directory. All four use
`leanprover/lean4:v4.32.1` and require mathlib `v4.32.1`, resolved to revision
`520045ab14e26149ee970e2e617ca04b09bde5d6`. The repeated project roots make
Lake materialize and compile overlapping mathlib dependency closures multiple
times.

The existing scripts also enforce important correctness controls:

- script-relative project roots for ordinary verification;
- task-scoped Lean toolchain boundaries for the later mathematical packets;
- source scans that reject `sorry`, and in NNG4 also reject `admit`;
- explicit Lake build invocation;
- no dependency on another local checkout.

Those controls should remain. The design changes the build topology, not the
trust boundary.

## Target architecture

Create one canonical Lean build root at `formalization/lean/`:

```text
formalization/lean/
  lakefile.toml
  lake-manifest.json
  lean-toolchain
  TrainingDynamics.lean
  TrainingDynamics/
  MathematicalFoundations.lean
  MathematicalFoundations/
  NNG4Intro.lean
  NNG4Intro/
  AutodiffGeometry.lean
  AutodiffGeometry/
```

The consolidated `lakefile.toml` declares one shared mathlib dependency and
four separate libraries:

```toml
name = "harp_formalization"
defaultTargets = [
  "TrainingDynamics",
  "MathematicalFoundations",
  "NNG4Intro",
  "AutodiffGeometry",
]

[[lean_lib]]
name = "TrainingDynamics"

[[lean_lib]]
name = "MathematicalFoundations"

[[lean_lib]]
name = "NNG4Intro"

[[lean_lib]]
name = "AutodiffGeometry"

[[require]]
name = "mathlib"
git = "https://github.com/leanprover-community/mathlib4.git"
rev = "v4.32.1"
```

Each library remains a separate namespace and ownership boundary. Shared Lake
state is an implementation detail for build speed; it does not merge the
mathematical routes or reader packets.

## Build commands

Add a single Lean build wrapper, `scripts/check_lean_library.sh`, that accepts
a fixed library name:

```sh
scripts/check_lean_library.sh TrainingDynamics
scripts/check_lean_library.sh MathematicalFoundations
scripts/check_lean_library.sh NNG4Intro
scripts/check_lean_library.sh AutodiffGeometry
scripts/check_lean_library.sh all
```

The wrapper maps each library name to:

- its source roots under `formalization/lean/`;
- the proof-hole policy for that source set;
- the Lake target to build;
- the timing label to print.

The existing public wrappers stay as compatibility entrypoints:

- `scripts/check_training_dynamics_lean.sh`
- `scripts/check_mathematical_foundations_lean.sh`
- `scripts/check_nng4_intro_lean.sh`
- `scripts/check_autodiff_geometry_lean.sh`

Each compatibility wrapper delegates to `check_lean_library.sh` with its fixed
library name. Test-only fixture modes may remain in compatibility wrappers
when existing Rust tests depend on them, but ordinary verification must route
through the consolidated root.

## Mise task model

Add fast iteration tasks:

```text
lean-training       -> TrainingDynamics
lean-foundations    -> MathematicalFoundations
lean-nng4           -> NNG4Intro
lean-autodiff       -> AutodiffGeometry
lean-all            -> all four libraries from the shared root
lean-timing         -> timing report for dependency and owned target phases
```

Keep the release-gate task names stable for existing workflows:

```text
verify-lean                         -> lean-training
verify-mathematical-foundations-lean -> lean-foundations
verify-nng4-intro-lean              -> lean-nng4
verify-autodiff-geometry-lean       -> lean-autodiff
```

Then update `mise run verify` so it does not execute four independent Lake
roots. The release gate should call `lean-all` once, then continue with the
repository verifier. Compatibility task names can still exist for focused
developer use.

## Timing contract

Every Lean wrapper invocation prints a compact timing report:

```text
[lean] target=MathematicalFoundations
[lean] root=formalization/lean
[lean] scan_seconds=...
[lean] lake_seconds=...
[lean] total_seconds=...
[lean] outcome=passed
```

On failure, the report includes:

```text
[lean] outcome=failed
[lean] failure_stage=scan|lake|toolchain|policy
```

The timing report is diagnostic output, not a durable proof receipt. It exists
to answer whether a run spent time in:

- local source policy scanning;
- dependency and Lake scheduling;
- Harp-owned proof elaboration;
- toolchain or environment setup.

The first implementation does not need perfect proof-file attribution. It
must make the duplicated dependency problem visible and make warm-cache
iteration measurable.

## Fast-iteration behavior

The desired steady state is:

1. A developer warms the shared root with `mise run lean-all` or by building a
   focused target once.
2. The developer edits one Harp-owned Lean file.
3. The developer runs the narrow target, such as `mise run lean-autodiff`.
4. Lake rebuilds only the changed file and required downstream Harp-owned
   files, reusing the already-built mathlib artifacts.

The design does not require a global cache, daemon, remote build service, or
persistent background process. The shared `.lake` directory is repo-local to
the consolidated Lean root and remains ignored build output.

## Import policy

Consolidation solves duplicated dependency builds. It does not by itself
minimize the dependency closure for each proof. After the migration is stable,
new Lean files should avoid broad `import Mathlib` unless there is a clear
reason.

The first import policy is advisory:

- report files that import plain `Mathlib`;
- prefer narrow imports in new modules;
- narrow existing broad imports opportunistically when touching a file;
- do not block proof work on import cleanup until the consolidated build is
  passing.

A later hard gate may reject new plain `import Mathlib` lines if the advisory
report proves stable and low-noise.

## Correctness and safety boundaries

The migration must preserve these invariants:

- no Lean runtime, build, test, source, or documentation dependency on another
  local checkout;
- no committed `.lake` output;
- no weakening of `sorry` or `admit` scans;
- no use of ambient shell startup state for ordinary verification;
- pinned Lean and mathlib identities remain committed and reviewable;
- focused wrappers still fail closed when the toolchain is unavailable;
- release verification still runs from the repo-owned scripts;
- Harp prose and evidence ownership rules are unchanged.

The shared build root may reduce isolation between Harp-owned Lean libraries at
the build-cache level. It must not create implicit source dependencies between
libraries. Cross-library imports are allowed only when explicitly written in
Lean source and justified by the formalization dependency order.

## Migration plan

1. Create `formalization/lean/` with the consolidated Lake root.
2. Move the four existing Lean libraries into the shared root without changing
   theorem statements.
3. Generate the consolidated Lake manifest from the same Lean/mathlib pin.
4. Add `scripts/check_lean_library.sh`.
5. Change the four existing check scripts to delegate to the shared wrapper.
6. Add or update Rust/script tests for wrapper routing, proof-hole rejection,
   and root pinning.
7. Add the new mise tasks and update `verify` to call `lean-all` once.
8. Run focused targets, then the full repository gate.
9. Remove obsolete per-project Lake manifests and toolchain files only after
   the compatibility wrappers and tests prove the shared root is authoritative.

The migration should land as one focused infrastructure branch. If import
narrowing becomes large, land it as a follow-up proof-speed branch rather than
mixing it with the build-root move.

## Verification strategy

Focused verification:

- `mise run lean-training`
- `mise run lean-foundations`
- `mise run lean-nng4`
- `mise run lean-autodiff`
- `mise run lean-all`

Policy verification:

- fixture with `sorry` under each owned source set fails before Lake execution;
- fixture with `admit` under NNG4 fails before Lake execution;
- ordinary wrappers ignore ambient project directories and use the shared root;
- wrapper rejects unknown library names;
- wrapper emits timing lines for passed and failed runs.

Repository verification:

- Rust tests that currently assert the Lean project roster are updated to the
  consolidated layout;
- generated corpus or product-contract expectations are changed only if the
  tracked payload requires it;
- `docs/import-receipt.md` is refreshed after all tracked changes settle;
- final landing runs `mise run verify`.

## Non-goals

- No remote cache or build farm.
- No background daemon.
- No weakening of hermetic or task-scoped toolchain checks.
- No semantic rewrite of the existing proofs during the build migration.
- No claim that faster builds make the mathematical formalizations complete.
- No vendoring of external source material outside existing Harp evidence
  workflows.

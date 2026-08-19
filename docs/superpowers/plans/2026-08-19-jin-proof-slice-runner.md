# Jin Proof Slice Runner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the first Jin proof-slice attempt run through Harp's shared Lean root and produce `attempt-003` as a real typed Lean result instead of another `missing executable: lake` receipt.

**Architecture:** Keep Jin's original repository as reference evidence only. Repair `labs/crouzeix_proof_reproduction/proof_slice.py` so production proof-slice runs inherit the same task-scoped Lean toolchain contract as `mise run lean-all`, then materialize and run a fresh `attempt-003` under the existing Jin formal target. The plan stops at the first real Lean result; it does not mark mathematical proof progress unless the receipt, axiom record, and source map mechanically justify it.

**Tech Stack:** Python 3 standard library `unittest`, Harp Crouzeix proof reproduction harness, Lean 4.32.1 through the shared Lake root at `formalization/lean`, `mise` task runner, Rust/Cargo release gate.

---

## Spec Link

Implementation follows [2026-08-19 Jin proof completion design](../specs/2026-08-19-jin-proof-completion-design.md).

The important constraint is the two-lane split:

- Reference lane: pinned Jin archive, source locators, original theorem names, original toolchain metadata, preflight receipts.
- Proof lane: Harp-owned Lean statements/proofs under `formalization/lean`, built by Harp's shared Lean wrapper and recorded by sealed proof-slice receipts.

## File Structure

- Modify: `labs/crouzeix_proof_reproduction/proof_slice.py`
  - Owns proof-slice descriptor validation, materialization, subprocess execution, command receipts, result publishing, and axiom audit parsing.
  - This task should add a small production execution environment helper rather than widening callers to pass arbitrary environment state.
- Modify: `labs/crouzeix_proof_reproduction/tests/test_proof_slice.py`
  - Owns focused TDD coverage for the proof-slice runner.
  - Tests should keep using fake `Popen`, fake `shutil.which`, and temporary directories; do not invoke real Lean from unit tests.
- Create: `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/`
  - Created by the existing `proof_slice.py materialize` command after runner repair.
  - Contains `task.json`, `source-slice.json`, `module/Slice.lean`, `build/command.json`, `build/stdout.log`, `build/stderr.log`, `build/axioms.json`, `result.json`, and `receipt.json`.
- Modify: `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/source-map.json`
  - Update only after `attempt-003` exists.
  - Set `jin-max-polynomial-modulus` from the receipt result, not from expectation.
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`
  - Add a narrow evidence note for the runner repair and `attempt-003`.
  - Run tracker validation before committing.
- Modify: `docs/import-receipt.md`
  - Refresh only after all tracked payload changes for the implementation batch are settled.

## Invariant Map

- The runner may trust checked-in task descriptors and source maps only after strict validation.
- The runner must not trust ambient shell state. It may use `PATH` only to locate the pinned shared Lean toolchain path added by `mise run lean-env`; the child process receives a minimal environment.
- The child process must run from `formalization/lean`, not from the original Jin `Lean/` directory and not from the proof-slice attempt directory.
- The command receipt must remain deterministic: `argv`, logical `cwd`, selected environment, timeout, and output cap are recorded before the subprocess result is interpreted.
- `attempt-001` and `attempt-002` are historical evidence. Do not edit them.
- `attempt-003` may be `failed` or `blocked`. It is successful for this plan if the reason is a real Lean result and not `missing executable: lake`.
- `source-map.json` can move `jin-max-polynomial-modulus` to `passed` only if the receipt status is `passed` and the axiom audit passed. Otherwise update it to the exact `failed` or `blocked` state with the new receipt digest and reason.

## Task 1: Repair Production Runner Environment

**Files:**
- Modify: `labs/crouzeix_proof_reproduction/proof_slice.py`
- Modify: `labs/crouzeix_proof_reproduction/tests/test_proof_slice.py`

- [ ] **Step 1: Write the failing subprocess environment test**

Add this test near the current subprocess tests in `labs/crouzeix_proof_reproduction/tests/test_proof_slice.py`, replacing `test_subprocess_executor_uses_empty_environment` if it still asserts an empty environment:

```python
    def test_subprocess_executor_uses_shared_lean_environment(self) -> None:
        captured: dict[str, object] = {}
        original_popen = proof_slice.subprocess.Popen
        original_which = proof_slice.shutil.which

        def fake_which(executable: str, path: str | None = None) -> str | None:
            self.assertEqual(executable, "lake")
            self.assertIsNotNone(path)
            return "/private/tmp/harp-mathematical-foundations-elan/toolchains/leanprover--lean4---v4.32.1/bin/lake"

        class FakePopen:
            def __init__(self, *args: object, **kwargs: object) -> None:
                captured["args"] = args
                captured["kwargs"] = kwargs
                self.stdout = io.BytesIO(b"ok\n")
                self.stderr = io.BytesIO(b"")

            def wait(self, timeout: int) -> int:
                captured["timeout"] = timeout
                return 0

        def fake_popen(*args: object, **kwargs: object) -> object:
            captured["args"] = args
            captured["kwargs"] = kwargs
            return FakePopen(*args, **kwargs)

        try:
            proof_slice.shutil.which = fake_which
            proof_slice.subprocess.Popen = fake_popen
            result = proof_slice._subprocess_executor(
                ["lake", "env", "lean", "module/Slice.lean"],
                proof_slice.SHARED_LEAN_ROOT,
                3600,
                1024,
            )
        finally:
            proof_slice.subprocess.Popen = original_popen
            proof_slice.shutil.which = original_which

        self.assertEqual(result.exit_code, 0)
        env = captured["kwargs"]["env"]
        self.assertEqual(
            env["ELAN_HOME"],
            "/private/tmp/harp-mathematical-foundations-elan",
        )
        self.assertEqual(env["ELAN_TOOLCHAIN"], "leanprover/lean4:v4.32.1")
        self.assertTrue(
            env["PATH"].startswith(
                "/private/tmp/harp-mathematical-foundations-elan/toolchains/leanprover--lean4---v4.32.1/bin:"
            )
        )
        self.assertNotIn("HOME", env)
```

- [ ] **Step 2: Run the new test and verify it fails**

Run:

```sh
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_proof_slice.ProofSliceRunTaskTests.test_subprocess_executor_uses_shared_lean_environment -v
```

Expected: `FAIL` because the current executor passes `env={}` and does not set `ELAN_HOME`, `ELAN_TOOLCHAIN`, or a shared toolchain `PATH`.

- [ ] **Step 3: Implement the minimal shared Lean environment helper**

In `labs/crouzeix_proof_reproduction/proof_slice.py`, add constants near `SHARED_LEAN_ROOT`:

```python
SHARED_ELAN_HOME = Path("/private/tmp/harp-mathematical-foundations-elan")
SHARED_LEAN_TOOLCHAIN = "leanprover/lean4:v4.32.1"
SHARED_TOOLCHAIN_BIN = (
    SHARED_ELAN_HOME
    / "toolchains"
    / "leanprover--lean4---v4.32.1"
    / "bin"
)
```

Add this helper near `_command_argv`:

```python
def _shared_lean_environment() -> dict[str, str]:
    path_entries = [SHARED_TOOLCHAIN_BIN.as_posix()]
    ambient_path = os.environ.get("PATH")
    if ambient_path:
        path_entries.append(ambient_path)
    return {
        "ELAN_HOME": SHARED_ELAN_HOME.as_posix(),
        "ELAN_TOOLCHAIN": SHARED_LEAN_TOOLCHAIN,
        "PATH": os.pathsep.join(path_entries),
    }
```

Then change `_subprocess_executor` so both `shutil.which` and `subprocess.Popen` use that environment:

```python
    environment = _shared_lean_environment()
    executable = argv[0]
    resolved_executable = executable
    if not Path(executable).is_absolute():
        found = shutil.which(executable, path=environment["PATH"])
        if found is None:
            reason = f"missing executable: {executable}"
            stderr = reason.encode("utf-8")
            return CommandResult(
                None,
                b"",
                stderr[:max_output_bytes],
                reason,
                stderr_truncated=len(stderr) > max_output_bytes,
            )
        resolved_executable = found
```

And in the `subprocess.Popen` call:

```python
            env=environment,
```

- [ ] **Step 4: Update existing subprocess tests for `shutil.which(..., path=...)`**

Any fake `shutil.which` functions in `test_proof_slice.py` that currently accept only `executable: str` must accept the optional `path` keyword:

```python
        def fake_which(executable: str, path: str | None = None) -> str | None:
            self.assertEqual(executable, "lake")
            self.assertIsNotNone(path)
            return "/opt/toolchains/lake"
```

For the missing executable test:

```python
        def fake_which(executable: str, path: str | None = None) -> str | None:
            self.assertEqual(executable, "missing-tool")
            self.assertIsNotNone(path)
            return None
```

- [ ] **Step 5: Run focused proof-slice tests**

Run:

```sh
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_proof_slice -v
```

Expected: `OK`.

- [ ] **Step 6: Commit runner environment repair**

Stage explicitly:

```sh
git add labs/crouzeix_proof_reproduction/proof_slice.py labs/crouzeix_proof_reproduction/tests/test_proof_slice.py
git commit -m "fix(crouzeix): run proof slices in shared Lean environment" -m "Co-authored-by: TRAE CLI <traecli@bytedance.com>"
```

## Task 2: Add Crouzeix Focus Target to Shared Lean Root

**Files:**
- Modify: `formalization/lean/lakefile.toml`
- Modify: `scripts/check_lean_library.sh`
- Modify: `mise.toml`
- Modify: `crates/harp/tests/lean_library.rs`
- Create: `formalization/lean/Crouzeix.lean`
- Create: `formalization/lean/Crouzeix/Jin/MaxPolynomialModulus.lean`

- [ ] **Step 1: Write the failing shared-wrapper test**

In `crates/harp/tests/lean_library.rs`, add:

```rust
#[test]
fn shared_wrapper_builds_crouzeix_focused_target() {
    let (_fake_bin, path) = fake_lake_bin("#!/bin/sh\nprintf '%s\n' \"$*\" > \"$LAKE_ARGS\"\n");
    let scoped_elan_home = scoped_elan_home();
    let lake_args = scoped_elan_home.path().join("lake-args");

    Command::new("/bin/sh")
        .current_dir(repo_root())
        .arg("scripts/check_lean_library.sh")
        .arg("Crouzeix")
        .env("ELAN_HOME", scoped_elan_home.path())
        .env("LAKE_ARGS", &lake_args)
        .env("PATH", path)
        .assert()
        .success()
        .stdout(
            predicate::str::contains("[lean] target=Crouzeix")
                .and(predicate::str::contains("[lean] root=formalization/lean"))
                .and(predicate::str::contains("[lean] outcome=passed")),
        );

    assert_eq!(
        fs::read_to_string(lake_args).expect("read lake arguments"),
        "build Crouzeix\n"
    );
}
```

- [ ] **Step 2: Run the wrapper test and verify it fails**

Run:

```sh
cargo test --profile test-small -p harp --test lean_library shared_wrapper_builds_crouzeix_focused_target -- --nocapture
```

Expected: `FAIL` with `Unknown Lean library target: Crouzeix`.

- [ ] **Step 3: Wire `Crouzeix` into Lake**

Modify `formalization/lean/lakefile.toml`:

```toml
defaultTargets = [
  "TrainingDynamics",
  "MathematicalFoundations",
  "NNG4Intro",
  "AutodiffGeometry",
  "Crouzeix",
]

[[lean_lib]]
name = "Crouzeix"
```

Keep the existing libraries and `mathlib` dependency unchanged.

- [ ] **Step 4: Add empty Harp-owned Crouzeix modules**

Create `formalization/lean/Crouzeix.lean`:

```lean
import Crouzeix.Jin.MaxPolynomialModulus
```

Create `formalization/lean/Crouzeix/Jin/MaxPolynomialModulus.lean`:

```lean
/- Harp-native Crouzeix route scaffolding.

This module intentionally contains no imported Jin terminal theorem. The first
mathematical declaration lands only after the proof-slice runner can produce a
real shared-root Lean receipt for the Jin max-polynomial-modulus row.
-/
```

- [ ] **Step 5: Teach the shared wrapper about `Crouzeix`**

Modify `scripts/check_lean_library.sh`.

In `usage()`:

```sh
printf '%s\n' "usage: $0 TrainingDynamics|MathematicalFoundations|NNG4Intro|AutodiffGeometry|Crouzeix|all [--project-for-test <directory>]" >&2
```

In the target `case`:

```sh
  Crouzeix)
    human_label="Crouzeix"
    scan_label=Crouzeix
    forbidden_words="sorry admit"
    ;;
```

In the `all)` source append loop:

```sh
      for library in TrainingDynamics MathematicalFoundations NNG4Intro AutodiffGeometry Crouzeix; do
```

- [ ] **Step 6: Add a focused mise task**

Modify `mise.toml` after `lean-autodiff`:

```toml
[tasks.lean-crouzeix]
description = "Build the Crouzeix Lean library from the shared Lake root"
depends = ["lean-env"]
env = { ELAN_HOME = "/private/tmp/harp-mathematical-foundations-elan", PATH = "/private/tmp/harp-mathematical-foundations-elan/toolchains/leanprover--lean4---v4.32.1/bin:{{env.PATH}}" }
run = "scripts/check_lean_library.sh Crouzeix"
```

Modify `lean-timing` so it includes:

```sh
mise run lean-crouzeix
```

- [ ] **Step 7: Run focused Crouzeix checks**

Run:

```sh
cargo test --profile test-small -p harp --test lean_library shared_wrapper_builds_crouzeix_focused_target -- --nocapture
mise run lean-crouzeix
```

Expected:

- Rust wrapper test: `test result: ok`.
- Lean task: `[lean] target=Crouzeix` and `[lean] outcome=passed`.

- [ ] **Step 8: Commit shared Lean target**

Stage explicitly:

```sh
git add formalization/lean/lakefile.toml formalization/lean/Crouzeix.lean formalization/lean/Crouzeix/Jin/MaxPolynomialModulus.lean scripts/check_lean_library.sh mise.toml crates/harp/tests/lean_library.rs
git commit -m "feat(crouzeix): add shared Lean target" -m "Co-authored-by: TRAE CLI <traecli@bytedance.com>"
```

## Task 3: Materialize and Run `attempt-003`

**Files:**
- Create: `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/`
- Modify: `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/source-map.json`

- [ ] **Step 1: Materialize the next attempt**

Run:

```sh
python3 labs/crouzeix_proof_reproduction/proof_slice.py materialize \
  --task-dir labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003
```

Expected JSON includes:

```json
{
  "row_id": "jin-max-polynomial-modulus"
}
```

- [ ] **Step 2: Replace the import-only adapter with a Harp-owned module check**

Open `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/module/Slice.lean`.

Replace its contents with:

```lean
import Crouzeix.Jin.MaxPolynomialModulus

/- Harp-owned proof-slice adapter for the first Jin row. This file must compile
from the shared Lake root and must not import Jin's terminal theorem. -/
#check CrouzeixConjecture.maxPolynomialModulusOnNumericalRange
```

This is expected to fail until the Harp-owned declaration exists. That failure is useful: it proves the runner reached Lean instead of stopping at tool discovery.

- [ ] **Step 3: Run the attempt**

Run:

```sh
python3 labs/crouzeix_proof_reproduction/proof_slice.py run \
  --task-dir labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003
```

Expected: JSON with `"status": "failed"` or `"status": "blocked"`, but not a reason containing `missing executable: lake`.

Inspect the result:

```sh
python3 -m json.tool labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/result.json
sed -n '1,120p' labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/build/stderr.log
python3 -m json.tool labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/receipt.json
python3 -m json.tool labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/build/axioms.json
```

Expected:

- `receipt.json` has `source_map_row_id` equal to `jin-max-polynomial-modulus`.
- `receipt.json` has `command_exit_code` set if Lean ran and exited.
- `build/stderr.log` records a Lean error such as an unknown declaration or missing namespace, not a missing `lake` executable.
- `build/axioms.json` is `not_applicable` for a failed compile, or `passed` only if the command genuinely exits 0 and axiom output is available.

- [ ] **Step 4: Compute the attempt receipt digest**

Run:

```sh
python3 - <<'PY'
import hashlib
from pathlib import Path
path = Path("labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/receipt.json")
print(hashlib.sha256(path.read_bytes()).hexdigest())
PY
```

Save the printed digest for the next step.

- [ ] **Step 5: Update `source-map.json` from the receipt**

Open `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/source-map.json`.

For `jin-max-polynomial-modulus`, replace the current receipt digest with the `attempt-003/receipt.json` digest.

If `attempt-003/result.json` is:

```json
{
  "status": "failed",
  "reason": "Lean command failed with exit code 1"
}
```

then the row must be:

```json
{
  "dependency_ids": [],
  "lean_name": "CrouzeixConjecture.maxPolynomialModulusOnNumericalRange",
  "receipt_sha256": "<attempt-003 receipt digest>",
  "row_id": "jin-max-polynomial-modulus",
  "source_locator": "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:Lean/CrouzeixConjecture/Statements.lean#L13-L18",
  "statement_sha256": "49bb4a5ac09489c9c7642215474b2f099f69d1e6a3e431f7a9dfd2b95ed736d0",
  "status": "failed",
  "failed_reason": "Lean command failed with exit code 1"
}
```

If the result is `blocked`, use `status: "blocked"` and `blocked_reason` exactly as recorded in `result.json`.

Do not set `passed` unless `receipt.json`, `result.json`, and `build/axioms.json` all show a clean compile and passed axiom audit.

- [ ] **Step 6: Validate source-map shape**

Run:

```sh
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_jin_validation.JinSourceMapTests -v
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_proof_slice -v
```

Expected: `OK`.

- [ ] **Step 7: Commit `attempt-003` evidence**

Stage explicitly:

```sh
git add labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/source-map.json
git add labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/task.json
git add labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/source-slice.json
git add labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/module/Slice.lean
git add labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/build/command.json
git add labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/build/stdout.log
git add labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/build/stderr.log
git add labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/build/axioms.json
git add labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/result.json
git add labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/receipt.json
git commit -m "test(crouzeix): record Jin proof slice shared-root attempt" -m "Co-authored-by: TRAE CLI <traecli@bytedance.com>"
```

## Task 4: Record Tracker Evidence and Refresh Repository Receipt

**Files:**
- Modify: `docs/workstream/crouzeix-proof-reproduction/tracker.org`
- Modify: `docs/import-receipt.md`

- [ ] **Step 1: Add a tracker note for CPFR-L007 or the current matching ticket**

Find the current real-Lean execution ticket:

```sh
rg -n "CPFR-L007|real Lean|proof-slice|jin-max-polynomial-modulus|attempt-003" docs/workstream/crouzeix-proof-reproduction/tracker.org
```

Add an evidence note to the matching ticket. If CPFR-L007 is still open and owns sandboxed real Lean execution, add this under its evidence section:

```org
- Shared-root Jin proof-slice attempt:
  =labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/receipt.json=
- The attempt no longer blocks on =missing executable: lake=; it reached the
  shared Lean root and produced status =<status>= with reason =<reason>=.
- Focused validation:
  =python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_proof_slice -v=
- Source-map validation:
  =python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_jin_validation.JinSourceMapTests -v=
```

Use the exact `status` and `reason` from `attempt-003/result.json`.

- [ ] **Step 2: Validate tracker**

Run:

```sh
python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org
```

Expected: command exits 0.

- [ ] **Step 3: Run focused proof gates**

Run:

```sh
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_proof_slice -v
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_jin_validation.JinSourceMapTests -v
mise run lean-crouzeix
```

Expected: all pass.

- [ ] **Step 4: Refresh import receipt**

Run:

```sh
cargo run -p harp -- repository verify
```

Expected first run may fail with:

```text
import receipt payload digest is stale; expected `<sha256>`
```

Patch `docs/import-receipt.md` so the final digest equals the expected SHA-256. Then rerun:

```sh
cargo run -p harp -- repository verify
```

Expected:

```text
repository verified: 521 import rows
```

- [ ] **Step 5: Run full release gate**

Run:

```sh
mise run verify
```

Expected: passes. If it fails only because `mise.toml` is untrusted in a fresh worktree, run:

```sh
mise trust
mise run verify
```

Do not skip the full gate unless the project owner explicitly narrows the landing gate.

- [ ] **Step 6: Commit tracker and receipt**

Stage explicitly:

```sh
git add docs/workstream/crouzeix-proof-reproduction/tracker.org docs/import-receipt.md
git commit -m "docs(crouzeix): record Jin shared-root proof attempt" -m "Co-authored-by: TRAE CLI <traecli@bytedance.com>"
```

## Task 5: Handoff to the Mathematical Proof Slice

**Files:**
- Modify only if the proof attempt exposed the next exact statement: `formalization/lean/Crouzeix/Jin/MaxPolynomialModulus.lean`
- Otherwise no code changes; write a follow-up plan.

- [ ] **Step 1: Classify the next blocker**

Read:

```sh
python3 -m json.tool labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/result.json
sed -n '1,160p' labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/build/stderr.log
```

Classify the next work item as exactly one of:

- Missing Harp-owned theorem namespace or declaration.
- Mathlib theorem-shape mismatch.
- Missing local definitions from Jin's original route.
- Proof obligation exists but needs a Lean proof.
- Toolchain or resource blocker.

- [ ] **Step 2: Write the next proof plan**

If the blocker is mathematical, create a new plan:

```text
docs/superpowers/plans/2026-08-19-jin-max-polynomial-modulus-proof.md
```

The plan must name:

- the exact declaration to add to `formalization/lean/Crouzeix/Jin/MaxPolynomialModulus.lean`;
- the source locator from Jin that motivated it;
- every mathlib import to try first;
- the smallest expected Lean error after the first red run; and
- the focused command `mise run lean-crouzeix`.

- [ ] **Step 3: Do not mark the row passed prematurely**

Before any later worker changes `source-map.json` to `passed`, verify:

```sh
python3 -m json.tool labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/result.json
python3 -m json.tool labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-003/build/axioms.json
```

The row can be `passed` only if both show a clean passed proof result and axiom audit. Otherwise the follow-up proof plan owns the next attempt.

## Verification Summary

Minimum verification before landing this plan's implementation:

```sh
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_proof_slice -v
python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_jin_validation.JinSourceMapTests -v
python3 labs/crouzeix_proof_reproduction/tickets.py validate-tracker docs/workstream/crouzeix-proof-reproduction/tracker.org
mise run lean-crouzeix
cargo run -p harp -- repository verify
mise run verify
```

## Self-Review Notes

- Spec coverage: covers the first implementation slice, shared Lean target, receipt generation, source-map update rules, tracker evidence, and follow-up proof handoff. It does not attempt the full mathematical Crouzeix theorem.
- Placeholder scan: no forbidden placeholder phrases or unspecified test steps remain.
- Type consistency: all Python names match current `proof_slice.py` conventions; all shell commands use existing repository paths.
- Evidence calibration: `attempt-003` is explicitly receipt evidence, not proof progress unless the result and axiom audit are both passed.

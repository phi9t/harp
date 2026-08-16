# Crouzeix Formal Target and Jin Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a hermetic, receipt-bound Lean lane that builds the pinned Jin development or records its first exact blocker without weakening the theorem.

**Architecture:** A new `formal_target` boundary validates a closed lock, fixed theorem, source inventory, and append-only lemma ledger. `jin_validation` invokes only the lock-derived project and binds results through additive v2 formal receipts; blind-frontier contexts remain untouched.

**Tech Stack:** Python 3 standard library, pinned Lean 4.28.0/Mathlib artifacts, existing Crouzeix tickets/formal receipts, unittest.

---

## File structure

- Create `labs/crouzeix_proof_reproduction/formal_target.py`: strict lock, target, artifact, and ledger validation.
- Create `labs/crouzeix_proof_reproduction/formal_target.lock.json`: Jin `565b6a3`, Lean 4.28.0, Mathlib, command and artifact identities.
- Create `labs/crouzeix_proof_reproduction/jin_validation.py`: source-map and create-only fixed build runner.
- Create `labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/{source-map.json,Target.lean}`.
- Create `labs/crouzeix_proof_reproduction/tests/{test_formal_target.py,test_jin_validation.py}`.
- Modify `formal_receipt.py` and `tests/test_formal_receipt.py`: exact v2 target binding while retaining v1.

### Task 1: FormalTarget contract

**Files:** Create `formal_target.py`, `tests/test_formal_target.py`.

- [ ] **Step 1: Write the failing boundary tests**

```python
def test_production_lock_ignores_ambient_redirect_and_rejects_unknown_fields(monkeypatch):
    monkeypatch.setenv("CROUZEIX_FORMAL_TARGET_LOCK", "/tmp/evil.json")
    with self.assertRaises(protocol.ValidationError):
        formal_target.load_lock()
```

Also reject duplicate JSON keys, absolute/traversal paths, source/package symlinks, digest/byte drift, unsupported Lean version, and ledger dependencies not `mathlib_available` or `locally_proved`.

- [ ] **Step 2: Run it red**

Run `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_formal_target -v`; expect import failure.

- [ ] **Step 3: Implement the closed types**

```python
def load_lock() -> FormalTargetLock: ...
def load_lock_for_test(path: Path) -> FormalTargetLock: ...
def validate_target(root: Path, lock: FormalTargetLock) -> FormalTarget: ...
def append_ledger_row(ledger: Path, row: LedgerRow) -> Path: ...
```

Use canonical JSON with duplicate-key rejection. `load_lock()` must always use the committed production lock; only the explicitly named test function accepts a path. Artifact paths are relative regular files below their declared root and must match byte count/SHA-256.

- [ ] **Step 4: Run green and commit**

Run the same command; expect PASS. Then:

```sh
git add labs/crouzeix_proof_reproduction/formal_target.py labs/crouzeix_proof_reproduction/tests/test_formal_target.py
git commit -m "feat(crouzeix): add formal target contract" -m "Co-authored-by: TRAE CLI <noreply@bytedance.com>"
```

### Task 2: Hermetic Jin runtime lock and provisioner

**Files:** Create `formal_target.lock.json`; modify `formal_target.py`, `test_formal_target.py`.

- [ ] **Step 1: Write red provision tests**

```python
def test_provision_checks_staged_regular_bytes_before_running_version(tmp_path):
    with self.assertRaises(protocol.ValidationError):
        formal_target.provision(lock, {"lean": bad_or_symlinked_artifact}, tmp_path)
```

Cover wrong digest/bytes, source swap during copy, stale stage, symlinked base/destination, race-created destination, host-PATH bypass, and unsupported platform.

- [ ] **Step 2: Populate the production lock from evidence**

Use only `evidence/crouzeix_conjecture/source_manifest.tsv`, `verification_manifest.tsv`, and pinned Jin records: commit `565b6a3e0659b6e0785f783b016c3f6d9f171fa5`, Lean `v4.28.0`, exact Mathlib identity, platform, bytes, SHA-256, and fixed build argv. Missing verified bytes produce a typed block; never invent a digest.

- [ ] **Step 3: Implement provision/resolve**

```python
def provision(lock: FormalTargetLock, artifacts: Mapping[str, Path], base: Path) -> ResolvedTarget: ...
def resolve(lock: FormalTargetLock, base: Path) -> ResolvedTarget: ...
```

Copy into sibling staging, verify copied regular bytes and executable mode before any version command, write deterministic inventory, then kernel-enforced no-replace publish or fail closed. Never use `curl | sh` or mutable installer code.

- [ ] **Step 4: Verify and commit**

Run `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_formal_target -v`; expect PASS. Stage all three Task-2 paths and commit `feat(crouzeix): pin Jin Lean validation runtime` with the required TRAE trailer.

### Task 3: Versioned formal receipts

**Files:** Modify `formal_receipt.py`, `tests/test_formal_receipt.py`.

- [ ] **Step 1: Write red v2 receipt tests**

Require `formal_target.path`, `formal_target.sha256`, `runtime_inventory_sha256`, and `target_type_sha256`; reject v1 receipts containing v2 fields, unknown fields, target drift, and `passed` with forbidden axioms.

- [ ] **Step 2: Implement exact dispatch**

```python
def validate_receipt(value: Mapping[str, Any]) -> dict[str, object]:
    # accept exactly crouzeix-formal-attempt-receipt/v1 or /v2
```

Preserve v1 byte-compatible validation. New target/Jin receipts must be v2 and bind all FormalTarget identities.

- [ ] **Step 3: Verify and commit**

Run `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_formal_receipt -v`; expect PASS. Commit `feat(crouzeix): bind formal receipts to targets` with trailer.

### Task 4: Jin source map and exact target alignment

**Files:** Create `formal_targets/jin-565b6a3/source-map.json`, `Target.lean`; create `jin_validation.py`, `tests/test_jin_validation.py`.

- [ ] **Step 1: Write red source-map tests**

```python
def test_source_map_covers_terminal_theorem_and_pinned_locator(self):
    rows = jin_validation.load_source_map(SOURCE_MAP, target)
    self.assertEqual(rows[-1].lean_name, "crouzeixConjecture")
```

Reject duplicate IDs, missing locators, locators outside `565b6a3`, unmapped terminal theorem, altered target type, and imports outside the lock allowlist.

- [ ] **Step 2: Implement map and alignment task**

```python
def load_source_map(path: Path, target: FormalTarget) -> tuple[SourceMapRow, ...]: ...
def make_alignment_task(target: FormalTarget, rows: tuple[SourceMapRow, ...]) -> Task: ...
```

`Target.lean` only imports allowlisted Jin modules and checks the mapped declaration's normalized elaborated type. It contains no copied informal proof.

- [ ] **Step 3: Verify and commit**

Run `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_jin_validation -v`; expect fixture PASS and production typed block until runtime provision. Commit `feat(crouzeix): map Jin formal target` with trailer.

### Task 5: Create-only Jin build and first-blocker receipt

**Files:** Modify `jin_validation.py`, `test_jin_validation.py`, `README.md`.

- [ ] **Step 1: Write red state tests**

Cover success, ill-typed Lean, unavailable toolchain, resource preflight, timeout, changed inventory, and forbidden axiom. Assert exactly one create-only v2 receipt; `blocked` names the first failing command rather than a synthetic compile error.

- [ ] **Step 2: Implement the fixed runner**

```python
def run_jin_validation(target: ResolvedTarget, task: Task, attempt_root: Path) -> Path: ...
```

Execute only lock-derived argv in a fresh process and explicit environment allowlist; preserve raw stdout/stderr; execute fixed axiom audit; call `formal_receipt.prepare_attempt`; reject existing attempt roots.

- [ ] **Step 3: Verify and commit**

Run focused validation tests and `git diff --check`; expect PASS/no output. Commit `feat(crouzeix): run sealed Jin validation` with trailer.

### Task 6: Ticketed outcome and Route-2 handoff

**Files:** Modify `docs/workstream/crouzeix-proof-reproduction/tracker.org`, `tests/test_tickets.py`, `README.md`.

- [ ] **Step 1: Add ticket validator regression**

Reject a tracker claim of completed proof without a passed v2 receipt and clean-rebuild digest; require the exact Jin terminal receipt for `passed`, `failed`, or `blocked`.

- [ ] **Step 2: Record ownership**

Add tickets for FormalTarget, Jin mapping, and Jin terminal validation. Add Route 2 only as a dependency on the published FormalTarget interface and Jin outcome; do not start Lorist formalization in this plan.

- [ ] **Step 3: Run integration gates and commit**

Run `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s labs/crouzeix_proof_reproduction/tests -v`, `cargo run -q -p harp -- repository verify`, and `mise run verify`; expect PASS. If mise asks to mutate trust state, stop and record the required operator action. Commit `docs(crouzeix): track Jin formal validation` with trailer.

## Plan self-review

- Coverage: Tasks 1–5 implement FormalTarget and Route 1; Task 6 makes the terminal result auditable. Routes 2–5 are separate plans because they are independent subsystems and should consume, not guess, these interfaces.
- No placeholders: an unavailable pinned artifact is a typed `blocked` outcome, never a fake value.
- Consistency: `FormalTargetLock`, `FormalTarget`, `ResolvedTarget`, `LedgerRow`, `Task`, and v2 receipts are defined before dependent tasks use them.

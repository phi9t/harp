# CPFR-086 retrospective

## Scope and evidence

This note records process evidence for CPFR-086 only; it does not widen the
Lorist--Schwenninger theorem claim beyond the route validation state
`complete-local`, as recorded in `execution-ledger-003.tsv` rows
`evidence-published`, `standards-approved`, and `release-verified`.

The execution started from base commit
`56742c15cf62ea017526705e38b657d4122adcbc`; the graph promotion payload is
commit `530c645bcfee7ee76591cc44df1ef3b5fdfd00d0` with tree
`ee69e85f25e3845ea58b71b13e63c3c9e058b5fd`. The existing verified proof and
code payload remains ledger landing commit
`d1749570825afc277fd96ff6c79b532449afbd16`, while the final full verifier was
run on head `2fb2e4212e19d8f2a0d70f6bd7a9f50178f7b5be` before this
documentation-only update.

The promoted route evidence is bounded by graph SHA-256
`ebe63abaf21d2ea620bfaa9be7b154ef39dbe4411535533314c14db264ab3826`,
inventory SHA-256
`97ea21ee0af6c9d0e38cbc566ad0fc46c5f31b2785fb2f575bcc0e23d84c9b8c`,
promotion marker SHA-256
`b153c84720588897b66708470514044940a28d085e3f6ecfa5bc2aa2ee6c097d`,
manifest-contract SHA-256
`38c953c8c70fae2d9e416994ff1684e73f7a63e7e501172fc267e8f706558d7d`,
route receipt file/self digests
`f672bb002c9d7ebc516683d61ba87b2c1ff2bed891daac0e6781aeb50cc7a01b` /
`b2a82c49454f4c24a29e06f864305ca5b8afba656ae0d5e862651833649e7b3e`,
and review file/self digests
`9f3fa1cffafc84b82e64c406cd843f43bbb4b1fc83197d69ed44c3f36d921d0a` /
`b67fbd93714679422cd80c80d78e5b4b76ab03f11042187b74a6a5be4b34196d`.

## Planned versus actual work

The plan expected a fresh worktree, six current v2 receipts, independent review,
atomic graph and inventory promotion, route receipt publication, a focused
release gate, and final metadata settlement. The ledger now records those
states through a control-metadata `landed` row that binds CPFR-086 to the
existing verified payload commit
`d1749570825afc277fd96ff6c79b532449afbd16`.

The actual route promotion happened at commit
`530c645bcfee7ee76591cc44df1ef3b5fdfd00d0`, and the ledger row
`locally-verified` records all six graph rows as promoted with route state
`mapped-incomplete` before route receipt publication.

The final route publication advanced validation to `complete-local` in the
ledger row `evidence-published`, binding route receipt digest
`f672bb002c9d7ebc516683d61ba87b2c1ff2bed891daac0e6781aeb50cc7a01b`, review
digest `9f3fa1cffafc84b82e64c406cd843f43bbb4b1fc83197d69ed44c3f36d921d0a`,
201 Python tests, and 97 Rust Crouzeix tests.

The full repository gate later ran successfully on head
`2fb2e4212e19d8f2a0d70f6bd7a9f50178f7b5be` under `/usr/bin/sandbox-exec` with
temporary policy SHA-256
`1c663a29e3574ed3a471a37b2791c176c9c70629fa585a55a8ce6ee6bb4ab6b6`. It
passed Atlas, Rust, shell, Python, Lean, and repository verification. Local
fast-forward and Kata closure remain pending at documentation time; the
controller still owns the final import-receipt refresh after these doc edits
change the repository payload.

## Failed assumptions and root causes

Task 6 assumed the initial promotion transaction covered all failure surfaces,
but review found missing final pre-marker source and receipt revalidation,
partial transaction residue, and nonuniform cleanup or lock failures; commit
`259cf6c` fixed those defects and its repair evidence included 141 tests.

The first Task 6 repair still assumed rollback error reporting and no-replace
fallback semantics were strong enough, but Standards review found loss of the
original rollback error and a racy non-Darwin/Linux no-replace fallback; commit
`5e030e6` fixed those defects and its repair evidence included 143 tests.

Task 7 could not be accepted from implementation status alone: the frozen route
candidate at commit `4e9808a56ac74eaf1e19d649b0862c2eb3a82281` still required
later locator, validation, route-publication, and repository-token repairs in
commits `dfd0ed5`, `0f1c434`, `48d7faf`, and `d174957` before CPFR-086 could
record `complete-local`.

The initial sandbox work assumed a broad Seatbelt policy and default Git
runtime visibility were sufficient, but the first invocation used `-p` instead
of `-f`, later platform defaults were too broad, and the first sandboxed Lake
command lacked readable Git/runtime configuration; the final successful run
used a narrow policy plus read-only Homebrew Git and
`GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null`, and the CPFR-086
tracker records stdout SHA-256
`315f2a2974fa3ad573604c77e0317f34235028ed0be1d6b6047bad312b010cca`.

The final sandbox preflight also had to prove the actual nested runtime, not a
synthetic command surface: `mise`, Git, Node/Corepack, Cargo, and Lake startup
all needed explicit coverage. Negative probes blocked arbitrary `/private/tmp`
writes, tracked Lean source writes, worktree `.git` writes, pinned toolchain
dylib writes, shared Mathlib artifact writes, and network socket creation. An
earlier isolated timeout occurred only while other gates overlapped; the
authoritative run used `RUST_TEST_THREADS=1` and passed.

## Verifier findings

The final mathematical review run
`cpfr086-ls-promotion-bind-20260825-c19e` reported `PASS` with no Critical or
Important findings after checking the promoted route binding.

That review initially raised a false digest mismatch by conflating source-claim
hashes with normalized Lean declaration-type hashes, then withdrew that finding;
it correctly required the double-layer locator to use `L116-L128` and the
terminal locator to use `L107-L129`.

The full-branch Standards reviewer `/root/cpfr086_final_standards`, using
GPT-5.5, reported `PASS` with no Critical, Important, or Minor findings at
commit `df8da81f9bbf38f88c0f20517c8af3099b271781` after reviewing payload head
`11d28201605da11972e13e85c5fa60a7399996c6`.

Post-promotion tests initially used live state as historical fixtures; commit
`5422d12` replaced that pattern with immutable historical fixtures and
phase-aware assertions.

Validator behavior also needed post-review tightening: commit `0f1c434` added a
fail-closed LS source-order check, and commit `d174957` removed the remaining
legacy source-graph spelling that repository verification rejected in a test
identifier.

The final full gate exposed two verifier-facing defects after the proof payload
was already reviewed. Commit `9877b59` fixed goal progression and an
absolute-path test fixture escape; commit `f175839` replaced remaining
Python 3.10-only `zip(strict=True)` uses with explicit equal-length checks that
work on Python 3.9. Both repairs received independent Spec and Standards
`PASS`; they harden verification and fixtures and do not change the proof
semantics.

## Cache and runtime measurements

The focused release gate ledger row at head
`df8da81f9bbf38f88c0f20517c8af3099b271781` records no-build preflight `ready`,
a cached sandboxed LS build of 3,386 jobs, scan time 6 seconds, cache check
1 second, Lake time 6 seconds, and total time 13 seconds.

The same focused gate recorded Python discovery as 739 passed and 2 skipped,
Rust Crouzeix tests as 97 passed, formatting and diff checks as passed, and
route validation as `complete-local`.

The shared dependency cache snapshot stayed
`3b8e37fc7ec9e5d8496c56fd89fa58d62293b2bb7fd3ee37276683a08804ce2e` through
the focused release gate, so the recorded evidence does not support any claim
that common Lean dependencies were hydrated or replaced during that gate.

Exactly one remedial sandboxed Lean elaboration produced the six theorem types
after the Git/runtime configuration was corrected; the earlier sandboxed Lake
attempt was blocked by Seatbelt before cache mutation when Lake misread the
unchanged Mathlib remote and tried forbidden cache replacement.

The definitive full gate on head
`2fb2e4212e19d8f2a0d70f6bd7a9f50178f7b5be` completed in 791.63 seconds real
time, 113.06 seconds user time, and 117.22 seconds system time. Atlas reported
13 test files and 63 tests passed, generated corpus matching, and offline export
passing. Rust passed across the full workspace, with 854 passed tests when
aggregating all `test result: ok` rows, plus formatting, Clippy, shell
regressions, no-DWARF, corpus, source, and search checks. Meta-harness Python
reported 13 passed. Crouzeix Python reported 740 passed, 2 skipped, in
298.978 seconds. Lean all reported 8,771 jobs with scan 1 second, cache 1
second, Lake 14 seconds, total 16 seconds, and passed. Repository verification
reported 521 import rows; its pre-documentation payload digest is not carried
forward here because these documentation edits intentionally change the
repository payload. Dependency cache metadata stayed unchanged before and after
the run at
`e22fd9dbbb48edface954c43ee9c62e169c1a268b4997deda331c6bbc1625c36`.

## Publication and recovery behavior

The publication model is create-only: the plan states that visible unreferenced
candidates are recovered rather than deleted, and commit
`18b35435d378521a3c7195d985569af51679a201` provides the six unreferenced v2
candidates later accepted by the route review.

One coherent `publish-ls` result created six unreferenced v2 candidates at
commit `18b35435d378521a3c7195d985569af51679a201`; the final mixed roster
accepted by review is recorded in the ledger row `spec-approved` as receipt
digests `289ab539299d35bfe5c2d8f83c5d19d3f05a9f4550831d2bca6fe19fe92e7646`,
`118dcdd0dd1bef0e6ad62344f774098533145b192ec5016dd1c36bd5fc086494`,
`ab3c537bceafec6a51e592994efb806361b4b96dc089426f4783d9e552ae6d5f`,
`4ab5034ddab3983e9ff7deb31e310ba76dc11da69afdf8055e2a945abd9123ae`,
`664249ba38de0c8420df98eb2d576bf0bfda7deae0402344c81e903461a86d51`, and
`d8abdd61ab401840bceea052c3814019dba96cf3abdd930725692f39f046f6ed`.

Exactly two metadata-only v2 receipts were republished without another Lean
invocation in commit `dfd0ed5542fe21d4e43e225d17f6c8decb84a894`, and four
selected receipts remained unchanged; the route review accepted that mixed
six-receipt roster in run `cpfr086-ls-promotion-bind-20260825-c19e`.

The promoted graph and inventory were marked atomically at commit
`530c645bcfee7ee76591cc44df1ef3b5fdfd00d0`, and the promotion marker digest
`b153c84720588897b66708470514044940a28d085e3f6ecfa5bc2aa2ee6c097d` binds the
graph and inventory digests recorded above.

## Changes applied to this plan

The `Plan evolution` section now requires phase-aware expected failures because
commit `5422d12` showed that live-state historical fixtures can make
post-promotion tests depend on the current branch rather than immutable
evidence.

The plan now requires review findings to become regression tests or explicit
rejections before the next gate because Task 6 review findings were fixed in
commits `259cf6c` and `5e030e6`, while the `PermissionError` lock suggestion was
rejected as unsafe.

The plan now requires sandbox preflight to exercise real tool startup and Git
identity under the intended Seatbelt policy because the first sandboxed Lake
attempt reached a forbidden cache-replacement path before the corrected
read-only Git environment produced one successful six-theorem elaboration.

The plan now requires the sandbox to test its own temporary and runtime
namespaces under Seatbelt, including explicit `/private/tmp` fixture names, and
to keep write access prefix-scoped while denying source, Git, toolchain, common
cache, and network mutation.

The plan now requires verifier tests to assert current state from immutable
fixtures or explicit current-state setup, not from whatever the live branch
happens to contain. It also requires temporary path helpers to resolve the
original path relative to the repository before joining it under a temporary
root, so absolute fixtures cannot escape the intended test tree.

The plan now records Python 3.9 as the compatibility floor for strict
length-pairing logic: use an explicit equal-length check followed by plain
`zip`, not Python 3.10-only `zip(strict=True)`.

The plan now keeps local fast-forward and Kata closure after the final
documentation and import-receipt settlement. The definitive full repository
gate has passed on head `2fb2e4212e19d8f2a0d70f6bd7a9f50178f7b5be`, but the
controller must recompute the payload digest after this documentation edit,
patch `docs/import-receipt.md`, and perform the local landing.

## Rejected changes

The plan treats the CPFR-086 `landed` ledger row as control metadata pointing to
existing verified payload commit
`d1749570825afc277fd96ff6c79b532449afbd16`; it does not claim that
`docs/import-receipt.md` has been refreshed after this documentation edit, that
local `master` has been fast-forwarded, or that Kata has been closed.

The plan does not weaken source fidelity, theorem identity, receipt
immutability, or independent review boundaries because the accepted evidence is
bound to the graph, inventory, manifest-contract, route receipt, and review
digests listed in `Scope and evidence`.

The plan does not adopt the suggestion to treat `os.kill(pid, 0)`
`PermissionError` as a stale lock because that behavior could delete a live
cross-user lock; the rejection is part of the Task 6 recovery record.

The plan does not claim mathematical independence from other routes because
CPFR-086 evidence supports route validation `complete-local`, six LS node
receipts, and a route-level review, not a broader independence theorem.

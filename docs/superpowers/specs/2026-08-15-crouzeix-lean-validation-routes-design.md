# Crouzeix Lean Validation Routes

## Objective

Reach the first reproducibly working formal Crouzeix proof as quickly as
possible. The first work validates the pinned Jin and Lorist--Schwenninger
proof routes in Lean; it is not a clean-room rediscovery study. The existing
blind expert-frontier experiment remains a later, isolated comparison arm.

Every result is evidence-bearing. A successful task proves only the exact
artifact it builds. A blocked task records the earliest closed blocker and
does not silently broaden assumptions, add axioms, or substitute a different
theorem.

## Non-goals

- Do not claim independent rediscovery from a reference-aware route.
- Do not let a familiar-looking manuscript replace a Lean build.
- Do not add `sorry`, `admit`, `axiom`, or an unrecorded trusted declaration.
- Do not make live model search, the existing E arm, or a source fetch part of
  ordinary verification.

## Shared contract: FormalTarget

All five routes consume one immutable `FormalTarget` record. It is created
before proof work and contains:

- pinned Lean and Mathlib release identities, executable/package digests, and
  a hermetic build command;
- the exact Crouzeix theorem statement, including scalar field, Hilbert-space
  universe, numerical-range convention, functional-calculus assumptions, and
  normalization constants;
- an import allowlist and content-addressed source inventory;
- the declaration name of the target theorem and any permitted local notation;
- a compiler receipt, environment receipt, and axiom-audit receipt; and
- a lemma-gap ledger.

The ledger is append-only. Each row has a stable ID, statement digest, source
locator or `Harp-authored`, dependency IDs, owner route, and exactly one
status: `mathlib_available`, `locally_proved`, `blocked`, or `conjectural`.
`blocked` requires the first failing command and a normalized diagnostic;
`conjectural` is never imported by a passing proof.

`FormalTarget` acceptance requires a clean import-only build, a target
statement that elaborates, no source or package symlinks, no unknown source
files, and an axiom audit that lists every trusted declaration reachable from
the target. A route may use only the fixed target statement and declarations
whose ledger status is `mathlib_available` or `locally_proved`.

## Common task contract

Each coding-agent task owns one directory and produces a closed receipt:

```text
task.json                 task ID, route, input/output digests, budgets
source-map.json           informal locator -> Lean declaration or blocker
build/command.json        fixed argv, cwd, environment allowlist
build/stdout.log          compiler output
build/stderr.log          compiler diagnostics
build/axioms.json         reachable trusted declarations
result.json               passed | failed | blocked | not_attempted
```

Tasks are create-only. Inputs and outputs must be regular files below the
declared task root; symlink, path traversal, unknown artifact, digest drift,
or a changed input makes the task `failed`. A task is `passed` only when a
fresh Lean process executes the fixed command, the expected declaration
elaborates, and the axiom audit matches the task policy. A timeout, missing
toolchain, or unavailable pinned artifact is `blocked`; an ill-typed proof is
`failed`.

No task may read a sibling route's unpublished worktree. Cross-route reuse
happens only by publishing a passed declaration into the shared lemma ledger
with its receipt digest.

## Route 1: Jin proof reproduction

**Complexity:** medium. **Priority:** first. **Purpose:** obtain the fastest
validated reference route and expose its first exact formal gap.

### Task sequence

1. `JIN-01 Source map`: index the pinned Jin proof into numbered theorem,
   definition, and inference spans. Each map row names the exact source
   locator, informal statement, and intended Lean declaration. No Lean proof
   source is written here.
2. `JIN-02 Target alignment`: prove that the Jin theorem statement and the
   `FormalTarget` are definitionally equal or produce a named adapter theorem.
   Any extra hypothesis or altered constant is a terminal mismatch record.
3. `JIN-03 Foundation probes`: one task per required library concept. It may
   only establish availability, notation alignment, or a narrowly stated
   local bridge.
4. `JIN-04..N Proof slices`: formalize one source-mapped proof segment per
   task. Each slice imports only prior passed slices and published ledger rows.
5. `JIN-FINAL Assembly`: import all passed slices, build the target theorem in
   a fresh process, and produce a complete map plus axiom audit.

### Acceptance

The route passes only if every substantial source span is mapped to a compiled
declaration or an explicitly justified harmless presentation change, the
target theorem compiles with no forbidden declaration, and all receipt digests
recompute. If a slice is blocked, the route's useful output is the earliest
ledger blocker, not a partial theorem claim.

### Agent boundary

An agent receives the source-map rows for its slice, `FormalTarget`, previous
passed declaration interfaces, and its owned directory. It may not edit the
target statement, add imports, or bypass a blocked predecessor.

## Route 2: Lorist--Schwenninger proof reproduction

**Complexity:** high. **Priority:** second and independent. **Purpose:**
validate a distinct published proof architecture and identify its own library
requirements.

### Task sequence

1. `LS-01 Dependency graph`: produce a source-indexed directed graph of
   definitions, lemmas, and terminal theorem; reject cycles unless the source
   explicitly uses mutual construction that Lean can represent.
2. `LS-02 Target and convention adapters`: establish exact compatibility with
   `FormalTarget`. Each adapter is a separately compiled theorem, never a
   hidden notation rewrite.
3. `LS-03 Library inventory`: map every external mathematical fact to either
   a pinned Mathlib declaration or a new isolated local task.
4. `LS-04..N Intermediate theorem tasks`: formalize graph nodes in dependency
   order, one statement per task.
5. `LS-FINAL Assembly`: build the terminal theorem and compare its elaborated
   type with `FormalTarget` byte-for-byte after normalization.

### Acceptance

Passing requires a standalone build with no dependency on Jin's private
working files. A theorem-strength mismatch, unavailable concept, or failed
intermediate result is recorded at the owning graph node. The route must not
weaken the theorem merely to obtain a compiling result.

### Agent boundary

Agents receive only one graph node, its predecessor interfaces, and mapped
source locators. They may propose a candidate lemma but cannot mark it passed
without the fixed build and audit receipt.

## Route 3: shared lemma/kernel library

**Complexity:** medium-high. **Priority:** only after Route 1 or Route 2
exposes a verified repeated need. **Purpose:** eliminate duplicated formal
bridges without speculative abstraction.

### Task sequence

1. `KERNEL-01 Admission`: compare two passed/blocked route ledger rows and
   admit a lemma only when both require the same normalized statement or two
   passed consumers already use an equivalent bridge.
2. `KERNEL-02 Specification`: state minimal hypotheses, prohibited circular
   dependencies, allowed imports, and consumer IDs.
3. `KERNEL-03 Proof`: formalize the lemma in isolation.
4. `KERNEL-04 Consumer migration`: replace route-local copies one consumer at
   a time, rebuilding both routes.
5. `KERNEL-05 Audit`: verify no kernel declaration depends on either terminal
   Crouzeix theorem.

### Acceptance

The lemma must be independently compilable, have a narrower import/dependency
surface than the two consumers combined, and have two receipt-bound uses. A
one-off bridge stays route-local.

## Route 4: Lean-guided reconstruction loop

**Complexity:** medium. **Priority:** activate immediately for a Route 1 or
Route 2 `blocked` lemma; this is the principal velocity mechanism.

### Task sequence

1. `RECON-01 Seal obligation`: freeze the precise unproved statement, fixed
   imports, local context, source-locator explanation, and candidate budget.
2. `RECON-02 Generate`: invoke one configured model session, or a human
   supplied candidate, to fill proof-body slots only. The prompt may cite the
   published route because this is reference-aware validation.
3. `RECON-03 Compile`: materialize the candidate into the sealed module and
   invoke a fresh pinned Lean process.
4. `RECON-04 Classify`: normalize result as `compiled`, `syntax`,
   `missing_fact`, `type_mismatch`, `false_subgoal`, `timeout`, or
   `policy_violation`.
5. `RECON-05 Narrow or publish`: a compiled candidate becomes a ledger proof;
   otherwise derive at most one strictly smaller named obligation. Stop at the
   fixed attempt and decomposition budget.

### Acceptance

Only a fresh compile plus audit publishes a lemma. Model prose, copied source,
or a candidate that changes the module outside declared proof slots is a
policy violation. The loop must emit its full failed-candidate history, so a
later task can distinguish a mathematical obstacle from search failure.

## Route 5: blind frontier comparison

**Complexity:** high. **Priority:** later and isolated. **Purpose:** preserve
the existing clean-room research question without delaying a working proof.

### Task sequence

1. `FRONTIER-01 Bind target`: add the `FormalTarget` digest to the existing E
   arm without exposing Lean declarations, source maps, proof text, or lemma
   ledger contents to blind experts.
2. `FRONTIER-02..N Execute`: run the five pristine roots, DGM selection, and
   child generations using existing CPFR tickets and receipts.
3. `FRONTIER-REVIEW Freeze`: independently review frozen candidates under the
   existing no-lineage/no-reference boundary.
4. `FRONTIER-FORMAL`: pass only frozen candidates through the same sealed
   formal compiler used by Route 4.
5. `FRONTIER-COMPARE`: compare outcomes to reference routes only after
   correctness and formal records are immutable.

### Acceptance

No reference-aware artifact can enter blind generation context. A formally
passing blind candidate supports a formal-proof claim; its discovery claim is
separately constrained by the experiment's provenance records.

## Execution order and stopping rules

```text
FormalTarget
  -> Jin source map + target alignment
  -> Jin proof slices
  -> Lorist--Schwenninger dependency graph + independent slices
  -> shared kernel only when admitted
  -> reconstruction loops at individual blockers
  -> blind frontier comparison after a formal baseline exists
```

The velocity stop condition is the first `passed` terminal Crouzeix theorem
receipt. At that point, pause new proof-discovery work, independently rebuild
the theorem from a clean checkout, and audit its trusted assumptions. The
remaining route work may continue as validation and comparative research, but
it cannot delay reporting that bounded result.

## Verification matrix

| Surface | Required check |
| --- | --- |
| Toolchain | pin/digest verification, clean fresh-process build |
| Target | elaborated type equivalence, import allowlist, source inventory |
| Slice | exact task input/output digests, no symlink/path escape, no `sorry` |
| Candidate loop | sealed-slot materialization, compiler receipt, bounded attempts |
| Shared kernel | two justified consumers, no terminal-theorem dependency |
| Final theorem | clean-checkout rebuild, axiom audit, source-map completeness |
| Blind comparison | context scan, ticket reconciliation, post-freeze separation |

## Design decision

Route 1 and Route 4 form the primary velocity lane. Route 2 is a genuinely
independent verification lane; Route 3 is evidence-driven support; Route 5
remains the research-quality comparison lane. None may silently inherit a
passing status from another route.

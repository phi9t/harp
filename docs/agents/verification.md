# Verification for local landing

`mise run verify` runs checks relevant to the changed surfaces. It is the ordinary
local landing gate. `mise run verify-all` retains the complete release checks;
`scripts/release_candidate.py` always invokes it.

## Select the complete candidate

- Preview: `mise run verify -- --plan`.
- Run: `mise run verify`.
- Multiple commits: `mise run verify -- --base <starting-commit-or-ref>`.
  Use the candidate’s original base, or a reviewed merge base for branch work.

Without `--base`, a dirty checkout compares tracked changes against HEAD and adds
untracked, nonignored paths. A clean checkout compares the latest commit against
its parent. Explicit bases include committed changes, staged/unstaged changes,
and untracked files. Renames select both old and new surfaces. A root commit
without a parent needs an explicit base or a separately reviewed initial gate.

Run in the isolated candidate worktree. A dirty primary checkout can select
unrelated work. Inspect the JSON plan and record the base, selected checks, extra
consumer checks, results, and limits in the local ticket. Path selection is a
starting point: add checks when a contract change affects consumers beyond its
path. No selected failure is silently skipped. Unknown surfaces fail with a
request to add a focused mapping rather than running every prerequisite.

## Surface ownership

| Changed surface | Selected checks |
| --- | --- |
| Agent guidance, skills, ordinary Markdown | Diff whitespace; changed local links and skill frontmatter; no Rust, Lean or React dependency |
| Corpus contracts and canonical lab/lessons | Rust corpus validation and generated corpus freshness, without proof builds or React |
| Rust | Affected workspace packages and transitive consumers; Harp corpus/context modules where isolated; changed integration targets and CLI entry point checks |
| Durable/Insula/runner | Composed portable lifecycle and boundary suites; required Linux qualification is recorded separately against exact source bytes |
| Imported benchmark studies | Byte, mode, complete inventory and ancestry preservation; historical links are retained |
| Math relocation | Source relocation record plus affected proof/publication consumers |
| Python labs/scripts | Corresponding test module or existing family discovery when shared inputs have no narrower module |
| Shell helpers | Syntax and matching behavior test, or the shell policy suite |
| Gate configuration/selector | Changed-surface dispatch regression tests |
| Proof code/config | Relevant Lean library when recognized; shared proof inputs use `lean-all` |
| PDF tooling | Existing renderer/input/edition tests |
| Captured evidence, import receipt, LFS policy | Repository/evidence verification |

Mise task auto-installation is disabled in this repository so dispatch itself
does not provision unrelated pinned tools.

The mapping lives in `scripts/verify_changes.py`. Keep it small and expand it
when a real changed contract needs another check. The gate does not install tools,
hydrate Lean dependencies, regenerate derived files, refresh import receipts, or
modify the candidate to make checks pass. Those are separate authorized repairs.
Tests/builds can still produce their normal ignored local outputs.

## Prerequisites and release

A selected proof check requires the pinned toolchain and canonical cache. Report
the exact missing dependency and affected operation; continue independent checks.
Unrelated missing prerequisites do not widen the local gate. A relevant check
that cannot run is a blocker unless the owner explicitly narrows that acceptance.

The full gate includes textbook publication, Rust, all Lean libraries,
Cauchy audit, and repository verification. It can require all their prerequisites.
Local readiness does not claim release readiness. Historical instructions calling
`mise run verify` a universal release gate are superseded by this contract.

## Native migration qualification

The consolidated runtime and retained benchmark changes require Linux receipts
in addition to portable checks. Prepare fresh source and capability profiles;
run `python3 -m durable.qualification --integrated --output <fresh-path>` or
`--benchmarks` or `--supervisor` inside a bounded delegated session with the documented native
prerequisites. Required skips, incomplete runs and changed source bytes fail.
Set `HARP_NATIVE_INTEGRATED_RECEIPT`, `HARP_NATIVE_BENCHMARKS_RECEIPT` and
`HARP_NATIVE_SUPERVISOR_RECEIPT` to the respective local receipt files before the complete-candidate gate. The verifier compares
their exact source-file hashes; old green evidence cannot qualify new bytes.
Keep host paths and private inventories out of committed evidence.

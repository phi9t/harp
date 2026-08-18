# NNG4 introductory Lean formalization

This Lean library proves Harp-owned counterparts of the ordinary Natural
Number Game 4 statements that are active through upstream `Game.lean`. The
inspected upstream source is:

- remote: `https://github.com/leanprover-community/nng4.git`
- revision: `727e4d219838eeb7f3945d2e9a0539f244d50540`

At that revision, `Game.lean` imports Tutorial, Addition, Multiplication,
Power, Implication, Advanced Addition, Less-or-Equal, Advanced
Multiplication, and Algorithm worlds. Their aggregator files import 78 active
level files. Harp proves 77 ordinary stable-world statements over standard
`Nat`.

`Game/Levels/Power/L10FLT.lean` is intentionally excluded from the proof
count. Upstream presents that Fermat endpoint through the hidden `xyzzy` tactic,
implemented by axiom-like game machinery. This project records that endpoint as
out of scope for genuine proof evidence.

The upstream repository also contains old, duplicate, and WIP level files that
are not imported by `Game.lean`. They are not part of this introductory phase.

## Reproducibility and verification

The required Lean toolchain is pinned in `../lean/lean-toolchain`, and the
committed `../lean/lake-manifest.json` pins the public dependency graph. The
shared Lake project uses mathlib's public `v4.32.1` tag and keeps one `.lake`
cache for Harp's Lean libraries.

Do not install Lean with a mutable installer pipe. Use the task-scoped Lean
state already used for Harp proof-infrastructure checks, with an explicit
`ELAN_HOME` below `/private/tmp/harp-mathematical-foundations-elan`; do not
alter a global Lean installation.

Run the repository wrapper from the repository root:

```sh
scripts/check_nng4_intro_lean.sh
```

For focused iteration, run `mise run lean-nng4`. To warm and check the full
shared root once, run `mise run lean-all`. The `mise` tasks provide the
task-scoped `ELAN_HOME` below `/private/tmp/harp-mathematical-foundations-elan`.
The wrapper canonicalizes the path and rejects every other location, so it
cannot use a global Elan home. It scans `../lean/NNG4Intro.lean` and
`../lean/NNG4Intro/` outside `.lake`, rejecting `sorry` and `admit` before
invoking Lake.

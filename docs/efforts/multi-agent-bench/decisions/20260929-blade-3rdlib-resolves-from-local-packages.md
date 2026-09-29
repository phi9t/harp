# blade-3rdlib-resolves-from-local-packages

Date: 20260929
Supersedes:

## Decision

`cpp3rdlib/x:ver@//cpp3rdlib/x:x` labels are kept verbatim in benchmark BUILD files. Offline, they resolve from
vendored packages pre-populated under `BLADE_ROOT/cpp3rdlib/<x>/` (a "3rdparty" tree layer in the capsule). No
`emulated` fidelity is needed for L1 and above on this point.

## Why

Probe (scratch project, cleared env, `--unshare-net`): with no local package, blade fails with
"//cpp3rdlib/gtest not found when loading BUILD file". With a hand-written `cpp3rdlib/tiny/BUILD`, both the plain
label `//cpp3rdlib/tiny:tiny` and the versioned label `cpp3rdlib/tiny:1.0@//cpp3rdlib/tiny:tiny` build and link.
Blade's docs say external deps are pulled into the `BLADE_ROOT` directory, so a pre-populated directory is what a
real fetch would leave. Blade logs a non-fatal "invalid path: cpp3rdlib/tiny" (its dep-info check expects
bvc/bpt/git); the build still succeeds.

## Consequences

- Headers of a vendored package are included by workspace-root path (`cpp3rdlib/tiny/tiny.h`).
- Vendored packages are small original stand-ins (a mini test framework, a tiny flags lib, a mini protobuf-like
  runtime for the IDL), not the real cpp3rdlib binaries. Real protobuf/gtest are not reproduced; tasks cannot
  claim to test them.
- Git-type deps (`repo:commit@//path`) and bpt fetching are not exercised; the ladder's "commit-pinned dep" shape is
  emulated with a local package whose directory name records the pin, and the task says so.

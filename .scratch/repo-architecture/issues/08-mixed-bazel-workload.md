# 08: Build a compact C++/Rust/Python Bazel workload

Status: ready-for-agent
Owner: unassigned
Parent: [Repository architecture](../spec.md)

Implementation scope: deferred workload follow-up; architecture only in the current pass.

## What to build

Architecture follow-up only in this pass. Later implement lab/mixed_build/: a C++ tensor/autograd core, Rust BPE tokenizer, Python native consumer and shared generated interface. Use pinned Bazel/toolchain inputs inside Insula; do not import private upstream estates or require another checkout.

## Acceptance criteria

- [ ] A real Bazel graph connects all three languages and generated build inputs. It does not merely shell out to three unrelated builds.
- [ ] A Python end-to-end example tokenizes a tiny corpus and computes gradients through the native tensor core.
- [ ] Independent gradient and tokenizer round-trip/determinism checks pass, including malformed-input boundaries.
- [ ] Clean offline build, no-change rebuild, leaf changes and shared-interface changes demonstrate correct build dependencies.
- [ ] Source/toolchain pins and licenses are recorded; no whole-framework build or registry publication is required.

## Blocked by

- [04-extract-insula](04-extract-insula.md)

## Verification

Run the build and workload inside the declared Insula profile. Record actual build actions for clean/no-op/changed-input cases. Check gradients against a separate numerical reference and compare tokenizer results with fixed independently reviewed vectors.

No implementation or qualification result is recorded yet.

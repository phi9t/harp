# jj executable toolchain preflight

Read Corenius `963f58b31e518a8162dfe719e386ef9d9843321f`. This is preparation
for optional ticket19, not its workspace/interface/crash acceptance. Tickets15/16
remain prerequisites for that complete evaluation.

The executable named `jj` on PATH is an unrelated SSH helper. Its attempted
`--version` invocation failed hostname resolution and was stopped; it is not
used as a VCS. A dedicated pinned binary avoids that collision.

The prior static source pin `030ecf22c56b98bf6507067dca0df3dffa6de498` declares
Rust1.97.1. Host stable is1.95.0; existing beta1.96.0 and nightly1.97.0 are
also below that declared requirement. No global toolchain was changed or MSRV
check bypassed. For executable preflight, select tagged release0.44.0 commit
`af45d57de7163cc5f17f3df178a31577e4951ea3`, which declares MSRV1.89.
The earlier static assessment remains a different explicitly pinned read.

Actual clean-source `cargo build --locked --release -p jj-cli --bin jj` completed
exit0 in3m28s with host1.95.0. Binary reports0.44.0 and the full source commit;
its hash, lock hash and build log hash are retained in
[jj-toolchain-preflight.json](../evidence/jj-toolchain-preflight.json).
Source and build output stay in owned scratch; no shared checkout conversion,
installation, model, hook, remote push or production operation occurred.

A separate isolated worker now prepares actual disposable Git-backed
snapshot/rewrite/ignored-file/immutable-bundle identity probes using this exact
binary. Those limited results will not substitute for capsule, crash, stale/
divergent operation, gate/evidence/GC or fast-forward integration requirements
in19. Git remains the approved kernel backend; adoption is undecided.

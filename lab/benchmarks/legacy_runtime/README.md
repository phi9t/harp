# Historical Rust runtime workload

This standalone Cargo workspace preserves the retired Rust execution framework
and its behavioral regression suites. The active Harp control plane is the
Python framework under `durable/`; these packages do not participate in the
product Cargo workspace or admit production agent work.

The seven original package names and their local dependency graph are retained.
`schemas/` contains the exact schema receipts owned by `harp-contracts`. Tests
are adjacent to source, with explicit Cargo integration targets preserving their
original target names. Schema regeneration remains an explicit
`HARP_UPDATE_SCHEMAS=1` operation and writes only this workload's schema receipts.
The shared dynamic-workflow study fixtures live beside this workspace at
`../dynamic-workflow/`.

Run the retained workload from the repository root:

```sh
cargo test --manifest-path lab/benchmarks/legacy_runtime/Cargo.toml --locked --offline --workspace --all-features
```

The standalone lockfile preserves the dependency versions from the pre-migration
product lock. Native process tests require the operating system facilities they
exercise; this workspace is preserved executable research, not an alternative
runtime entrypoint. Root product verification must select this workspace
explicitly when its source or shared fixtures change.

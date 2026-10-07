# Native registry fixture sparse-peer selection repair

Source read: combined candidate `60735f1f14b6c1aef8b3c9a6f370375e0ef015e1`.
Recorded: 2026-10-07. Scope: test fixtures only; no model calls.

## Cause and bounded correction

The prior full native gate reported 40/41 cases passed. Its failing case,
`repair_empty_bound_thread_changed_incarnation_stays_unknown`, raised
`KeyError: thread_id` in native_registry.py's target selector. Production
codex.rs creates a `preparing` registry binding with server/executor identities
and ui_input_ready but no thread_id, before native thread qualification.
Inference: this legitimate unrelated peer can precede the observed target.
Neighboring selectors already use optional thread IDs.

The pure test helper skips peers without the target thread identity. Both
empty-thread selectors use it. A missing target still raises StopIteration.
All target-state, operation-count, occupied-slot and deadline assertions remain
unchanged. Production code, runtime configuration and timeouts are unchanged.
A Rust test invokes the two Python behavioral regressions, preserving native
suite coverage (42 cases including the new regression).

## Fresh verification

- Protocol-shaped unbound peer preceding the target fails with KeyError using
  the legacy selector (exit1), then succeeds with the corrected selector.
- Python regression: 2 tests passed; absent target still fails selection.
- Rust-linked regression: 1 test passed, 41 filtered out, exit0.
- Former failing case: 1 passed, exit0, 60.18 seconds. Its unchanged wrapper
  executes shared and separate native topologies.
- Other repaired selector case `repair_empty_bound_thread_survives_controller_crash`:
  1 passed, exit0, 83.64 seconds; shared and separate topologies.
- Shared temporary-path scanner passed; git diff whitespace check passed.

Actual case command: `cargo test --locked --offline --manifest-path
crates/sqlite-controller/Cargo.toml --features fault-injection --test codex
<case> -- --exact --nocapture`. Tests rebuilt from this isolated source checkout,
using the shared target directory and TMPDIR=/data02/home/philip.yang/.lq07.
No full native-suite rerun or broader qualification is claimed here.

## Retained run records

Directory: `/data02/home/philip.yang/devx/tmp/corenius-landing-native-fix-20261007/.scratch/native-fix/`.
SHA256 identities:

- selection-red-protocol-shaped.log: `e3859f896715c046a000457e4ffdaa0fe08ae23769f4fe74a18c11c0183ddf2d`
- selection-green.log: `a6707d4eff38012da33ddc34bc6dc076a3b17151cfc9aef223c8790cbe64c3da`
- cargo-selection-green.log: `8a9ab5d5b6c7dbf86afb5b7ecb7da4bfcfb80ae005118677a1c8cb025cacfe75`
- native-case.log: `aaddce005b23c0c914a50b224b40a1d943e7d39874a760bb76ed7dfcf32a6881`
- native-other-selector.log: `8a9e105384b06b0e421170827ec64f7aa9d43a76d889f18f19bed9af56e866e4`

# Organize Corenius by component ownership

Corenius will place its active terminal agent and controller in terminal_agent/ and controller/, with controller-specific tools beside their owning component. The parked PostgreSQL/Temporal implementation is retained under experiments/team_control_plane/. This keeps a component's implementation and operational interfaces together across languages instead of grouping product packages under crates/ or mixing their tools into the general engineering toolbox. Existing idiomatic package internals remain; tools/agents/, bench/ and experiments/ retain their distinct engineering, benchmarking and exploratory responsibilities.

This decision applies only to Corenius. Narrow documented exceptions may preserve generated, vendored or externally required layouts. Bazel is deferred from the first migration; a shared Cargo workspace is the intended direction pending compatibility checks, with independent workspaces retained temporarily if a concrete blocker is found. Implementation has not begun.

Linux is the sole supported platform for this effort; platform portability does not constrain the layout or build design.

The name controller describes durable state ownership, permission enforcement, admission and execution reconciliation; coordinator understates these responsibilities, while sqlite_controller couples the component name to its storage choice. The retained team control plane is parked and unfinished, not a third active product.

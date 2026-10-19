# Controller ownership

The selected runtime is the Python + SQLite engine in [`durable/`](durable/README.md),
integrated through `harp durable`. See the [Harp runtime guide](../../docs/durable-runtime.md)
for the supported workflow, independent verification and human approval/landing.

The previous Rust controller, attempt supervisor and gate are retired. Their
explicit Cargo targets print retirement guidance and exit nonzero before opening
SQLite, binding sockets or starting children. Historical modules are no longer
library, example, benchmark or integration-test targets. Old native services,
SQLite/Kata baseline, relay and macOS operator launchers also refuse. Pure macOS
snapshot rendering helpers remain importable for historical analysis.

`durable/` and its guardian/cgroup boundary are unchanged by this retirement.
Reusable contracts and design records remain references, not fallback controllers
or current acceptance evidence. Captured evidence/license bytes remain intact.
Original launcher/gate bytes remain recoverable in Git and `../HARP_IMPORT.json`.

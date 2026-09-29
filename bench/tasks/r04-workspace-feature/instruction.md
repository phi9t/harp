This workspace has two path crates:

- `codec` owns packet encoding and has a `json` feature.
- `report` owns the public `render_report(id, payload)` API and enables its
  `json-output` feature by default.

With default features, `report::render_report` must return the JSON encoding
from `codec`, including escaping quotes and backslashes in the payload. Fix the
workspace feature wiring so the dependent crate sees the right feature. Use only
the path crates already in the workspace and run `cargo test --workspace
--offline`.

# Workload ownership

The root contributor guide governs this tree. `lab/` holds task inputs, workload
adapters, checks, benchmark fixtures and preserved studies. Agent jobs execute
through `durable/`; service lifecycles use `insula/supervisor/`; SSH is transport
through `runner/`. Workloads do not own a dispatcher, provider session recovery,
Git approval or landing authority.

Captured studies and old harness/provider launch code are reference material.
Their direct controller, fleet and Trae launch paths are unsupported in the
consolidated product. Preserve those sources and original evidence; adapt a
scenario through durable admission before claiming current execution support.
Negative proof fixtures remain outside the production Lean source tree.

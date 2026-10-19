# Soma

Soma is Harp's Emacs/Doom workload: a checked Elisp bridge, six gym scenario
families (Magit, Dired, Org, comint, Jupyter and LSP), daemon transport, lifecycle
workers, mechanism probes, graders and historical observations. The separately
sourced AI/LSP verifier lives in `ai_lsp/`. Its tests colocate with the Doom
adapter and launcher. Existing scenario checks colocate with their runners.

## Execution ownership

`scripts/insula/soma_insula.py` validates a workload-specific rootfs profile and
emits a declarative mount/environment/command plan. Its final namespace effect
delegates to `insula.capsule.execute_prepared`; it does not own an agent controller.
Run it inside an admitted durable attempt through the shared runner to retain
cgroup/guardian cancellation and descendant cleanup. Direct foreground execution
is caller-owned and is not a claim of durable recovery or descendant cleanup.
The old host-bind namespace-only probe wrapper delegates to the selected-rootfs
runner. No sibling checkout supplies tools or personal Doom configuration.

Select a pre-existing profile explicitly with `SOMA_INSULA_ROOTFS_STORE`; build
and publishing commands are explicit operator actions, never an import side
effect. Workload outputs must use a fresh output directory beneath this subtree;
checked-in `traces/` contain historical observations, not fresh qualification.

```
PYTHONPATH=lab/soma python3 -m unittest discover -s lab/soma/scripts/insula -p 'test*.py'
python3 lab/soma/verify_preservation.py
lab/soma/scripts/insula/run-emacs-gym-benchmarks-rootfs.sh <fresh-output-directory>
```

AI/LSP offline unit verification and authenticated live verification are distinct.
`ai_lsp/bin/verify-ai-lsp unit` requires the pinned Doom profile; `integration`
also requires actual language servers and authorized AI service credentials.
Neither historical results nor deterministic launcher fixtures certify those live
services. Plan-only API/export work remains recorded as source tickets.

## Provenance and licensing

`source-map.json` maps every acquired source file to its disposition and destination
bytes. Public text and historical observations are explicitly sanitized derivatives:
private host paths, operator identity and storage destinations are replaced.
Original committed archives and exact dirty snapshots remain in task-private
acquisition evidence. Deleted tracked observations are retained under
`history/committed-deleted/`. Machine-local rootfs selection, caches, transient
host-shadow executable copies and credentials are not runtime inputs or published
configuration. Their inventory and original hashes are retained privately.

No upstream repository license was present in either acquired source tree; this
import does not infer permission or add a repository-wide license. Existing
third-party package pins/attribution are retained. Original agent guidance and
historical skill documents are source history, not Harp workflow instructions.

# Explicit service execution owner

The owner runs on Linux with a caller-delegated cgroup v2 domain, bubblewrap,
a fingerprint-admitted rootfs capsule and Python linked to SQLite with the WAL
reset fix (3.51.3 or the recognized 3.50.7/3.44.6 backports). It does not provision
these prerequisites or install a background service.

Create a private operator-owned config outside Git:

```json
{
  "root_id": "example-project",
  "authority_dir": "/private/operator/insula-owner",
  "cgroup_root": "/sys/fs/cgroup/example-delegated-domain",
  "capsule": {"...": "the complete admitted capsule record"},
  "workspace": "/private/operator/readonly-workspace"
}
```

`capsule` must be a complete record from `insula.capsule.build`, starting with a
rootfs layer and using the default bench identity. The operator supplies this
binding; supervisor clients cannot replace it. The private authority directory
contains `owner.sock`, `services.sqlite` and generation-specific payload/log
directories. Keep the same config and directory for crash recovery. Changing
its binding or moving state requires deliberate operator migration. Do not
create a fresh authority directory to sidestep unknown old service effects.
The live root lock is anchored to the delegated cgroup directory inode, preventing
concurrent socket/state aliases from creating another owner for that root.
Same-UID host operators remain trusted, as they do for the durable controller.

```sh
python3 -m insula.owner.service --config /private/operator/owner.local.json
insula-supervisor serve --owner-socket /private/operator/insula-owner/owner.sock \
  --root-id example-project --state-dir /private/operator/service-policy
```

Run both processes explicitly and stop them explicitly. The owner handles SIGTERM
by fencing its claim, killing service cgroups and proving emptiness. A kernel or
owner crash can leave unknown effects; restart with retained state to reconcile.
Never infer cleanup from controller death alone. Stronger controller-independent
survival and namespace-entering probes remain unsupported.

## Native qualification

The colocated tests use real cgroups, namespaces and the Rust supervisor binary.
Native mode fails when required prerequisites are absent. Run inside a disposable
delegated domain, with no production workload in that domain:

```sh
export INSULA_OWNER_NATIVE=1
export HARP_TEST_CGROUP_ROOT=/sys/fs/cgroup/example-test-delegation
export HARP_TEST_ROOTFS=/private/operator/qualified-rootfs
export INSULA_SUPERVISOR_BIN=/absolute/build/insula-supervisor
python3 -m unittest -v insula.owner.test_service
```

The supplied Rust fixture suites remain separate evidence:

```sh
INSULA_TEST_TMUX=/absolute/path/to/tmux-3.4 cargo test -p insula-supervisor --all-targets -- --test-threads=1
```

Their direct-process owner never supplies production authority. The richer
secret/probe/survival fixture cases do not qualify those capabilities for the
real owner. Native owner tests cover the supported subset through protocol v1,
including a Rust client round trip, namespace access limits, descendant cleanup,
claim loss, stale generations, owner SIGKILL recovery, bounded logs and empty
tmux replacement without replaying pane work.

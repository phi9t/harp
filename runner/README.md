# SSH runner

The runner transports literal requests to the same project-scoped durable
runtime. Remote `start` enters a transient foreground user-manager scope so
its guardian can join the existing delegated cgroup; the scope installs no
service and owns no scheduling state. It has no scheduler, retry database or detached service. Configure a
private 0600 `hosts.local.toml` from the fictional example and preserve normal
OpenSSH host-key trust. Agent forwarding and implicit port forwarding are disabled.
The local and remote runtime/Insula/runner source hashes and admitted execution
contract must match before any operation. The remote execution already owns a
pinned rootfs, source closure and delegated cgroup; runner never installs them.

```
python3 -m runner --host example -- preflight
python3 -m runner --host example -- workload example-run --repository /srv/workload-source --manifest lab/soma/workload.json --base <commit> --task-revision <commit>
python3 -m runner --host example -- start --until-idle
python3 -m runner --host example -- status example-run
python3 -m runner --host example -- artifact example-run summary.json
```

A disconnected/timeout request is **unknown**, not permission to start another
attempt. Query the same run identity; the journal/guardian owns replay and
reconciliation. Losing the transport does not prove cancellation or cleanup.
`cancel` requests cancellation in that same durable journal. Capacity reuse waits
for guardian receipt and cgroup emptiness.

`artifact` retrieves only a declared bounded file whose hash matches a completed,
accepted workload result. It cannot browse arbitrary remote paths or failed
attempts. Operator responses are private diagnostic output: redact them before
publishing. `privacy.py` rejects tracked local inventories and obvious credential
or non-documentation address literals. Public reports must use host-neutral labels.

Qualification of the admitted Soma workload and cancellation controls is recorded
in `qualification.json`. The exact host inventory and raw responses remain private.
Each request owns a fresh SSH process group, including any proxy subprocess;
connection loss releases that transport group and reports an unknown outcome.
OpenSSH multiplexing is disabled so request cleanup cannot affect another session.

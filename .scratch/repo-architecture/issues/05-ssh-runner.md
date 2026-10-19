# 05: Run admitted workloads over SSH under Insula

Status: ready-for-agent
Owner: unassigned
Parent: [Repository architecture](../spec.md)

## What to build

Implement top-level runner/ as transport/preflight for the selected remote execution host. Use OpenSSH trust/configuration, an ignored local inventory, pinned request transfer and identity-checked result retrieval. Submit/query the same durable control plane; implement no independent scheduling or retry database.

## Acceptance criteria

- [ ] A declared lab workload executes under the admitted remote Insula profile and returns source/profile/oracle-bound evidence.
- [ ] A dropped SSH connection causes status reconciliation by run identity; reconnect never silently repeats an ambiguous effect.
- [ ] Cancellation and capacity reuse require actual remote cleanup evidence, not transport termination.
- [ ] Commit a schema and fictional example only. The actual inventory is ignored and a check rejects forced staging, private hostnames, addresses, credentials and host paths in new public runner artifacts.
- [ ] Wrong source/profile, missing delegation, unsupported capabilities and unavailable host refuse before workload launch. Host-key checking remains enabled.

## Blocked by

- [04-extract-insula](04-extract-insula.md)

## Verification

Use disposable SSH fixtures for argv/transfer failures, then a real ephemeral remote run and disconnect/reconnect/cancel tests. Scan the actual tracked runner payload and sanitized report outputs. Record native evidence privately with a publishable host-neutral summary.

No implementation or qualification result is recorded yet.

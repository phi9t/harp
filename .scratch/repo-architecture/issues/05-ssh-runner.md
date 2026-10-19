# 05: Run admitted workloads over SSH under Insula

Status: done
Owner: soma-import / codex/workload-runner
Parent: [Repository architecture](../spec.md)

## What to build

Implement top-level runner/ as transport/preflight for the selected remote execution host. Use OpenSSH trust/configuration, an ignored local inventory, pinned request transfer and identity-checked result retrieval. Submit/query the same durable control plane; implement no independent scheduling or retry database.

## Acceptance criteria

- [x] A declared lab workload executes under the admitted remote Insula profile and returns source/profile/oracle-bound evidence.
- [x] A dropped SSH connection causes status reconciliation by run identity; reconnect never silently repeats an ambiguous effect.
- [x] Cancellation and capacity reuse require actual remote cleanup evidence, not transport termination.
- [x] Commit a schema and fictional example only. The actual inventory is ignored and a check rejects forced staging, private hostnames, addresses, credentials and host paths in new public runner artifacts.
- [x] Wrong source/profile, missing delegation, unsupported capabilities and unavailable host refuse before workload launch. Host-key checking remains enabled.

## Blocked by

- [04-extract-insula](04-extract-insula.md)

## Verification

Use disposable SSH fixtures for argv/transfer failures, then a real ephemeral remote run and disconnect/reconnect/cancel tests. Scan the actual tracked runner payload and sanitized report outputs. Record native evidence privately with a publishable host-neutral summary.

No implementation or qualification result is recorded yet.


### Durable SSH qualification

Implemented the generic committed workload admission/worker path in
`durable/workloads/`, reusing the single journal, Activity, Insula profile,
guardian and manual recovery policy. Soma owns its adapter, manifest and frozen
acceptance checker under `lab/soma/`; there is no framework Soma registry.
`runner/qualification.json` records the native contract, source/profile digests,
artifact hashes and limits.

- Actual admitted selected-rootfs Soma run: 8 suites, 139/139 scenarios;
  9 declared compressed artifacts. Retrieved daemon result over SSH and verified
  its size, SHA-256 and decoded 63-case result.
- Killed SSH transport during a running workload, then queried the same run:
  RUNNING, one attempt. Explicit journal cancellation yielded BLOCKED/manual,
  cgroup.kill evidence and populated=0; all 5 original process identities across
  3 process groups disappeared. Transport loss itself was not treated as cleanup.
- Queued capacity reuse started only after that cleanup receipt, produced the
  independently checked answer and succeeded.
- An adversarial workload rewrote its checkout checker to succeed, but the frozen
  owner checker rejected its bad output: semantic FAILED, check exit1, passed=false.
- Wrong source/profile refused before state creation. After the ephemeral
  delegation was collected, native preflight refused the missing cgroup. An
  unavailable SSH host returned unknown without retry. Private inventory remains
  ignored; OpenSSH host-key checking stays enabled.
- Fixed a real transport problem: a proxy retained stdout/stderr after SSH died.
  Each request now owns its SSH process group and disables multiplexing. Both a
  real child-pipe regression and main-only real SSH kill returned promptly.
- The first admitted Soma attempt exceeded the 16 MiB daemon artifact bound;
  it remains retained and BLOCKED. Soma now compresses reports, leaving framework
  bounds intact. No source checkout or original rootfs was changed.

Focused portable checks and full preservation checks pass. Final composed
repository verification and exact source receipt remain the integration owner's
responsibility; this ticket remains in-progress until that candidate gate.
Authenticated external AI/LSP calls remain outside this qualification.

Agent candidate validation: 10 runner/workload boundary tests and 34 imported
source/preservation tests passed; source-map hashes/modes and staged privacy scan
passed. `mise run verify` and `--plan` refuse the copied shared
`durable/__init__.py` because this isolated base lacks the coordinator's new
verification mapping. No selected check was skipped; repository landing remains
blocked until that mapping and the composed candidate gate pass.

Final boundary review reproduced a workload leaving a writer alive across its
acceptance check. The repaired worker runs each command and check in a separate
PID/mount namespace; workload descendants are gone before checking/hashing,
checks see read-only source/output/oracle, and parent input/proof paths are hidden.
The public CLI counterexample now accepts only the checked `42` bytes. A fresh
profile also passed the native phase probe and all139 Soma scenarios across eight
suites, with nine declared artifacts. Summary and compressed daemon results were
retrieved over the real SSH artifact API and their hashes/sizes verified. The
existing16MiB file bound was retained. Full composed-source gate remains pending.

## Completion

Implemented and verified in body commit `c32c6f0f4a2de73e8067be0361121bb841ab4622`. The complete-candidate
changed-surface plan passes, including the bounded verification repairs recorded in
[the repository qualification](../../../evidence/provenance/repository-qualification.json).
Native runtime, benchmark and supervisor receipts bind the composed runtime bytes;
real Codex one/two/four-worker runs and all eight explicitly approved landings are
recorded separately in [Codex qualification](../../../evidence/provenance/codex-qualification.json).
Retained workloads and proof/PDF consumers passed their relevant checks.

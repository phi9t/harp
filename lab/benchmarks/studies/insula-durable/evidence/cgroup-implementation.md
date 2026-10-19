# Delegated cgroup backend handoff

Base read: e045d4a (Group Corenius source by semantic component). Only tracked
candidate files added are controller/durable/cgroup.py and cgroup_test.py. No
staging, commits, push, shared checkout edits, or real cgroup creation occurred.

## API and integration

- `validate_root(root)` returns a validated absolute Path without creating a
  subgroup. `Cgroup.validate_root(root)` returns the root/device/inode binding
  dictionary. Both require an actual Linux cgroup2 filesystem, domain type,
  write/search permission, and writable delegation interfaces.
- `Cgroup.create(root, attempt_id, root_identity=...)` creates one fresh subgroup
  with a bounded attempt identifier and random generation token. It validates
  cgroup.kill capability before returning. Root binding disagreement fails
  before mkdir. Persist `group.identity()` atomically before release intent.
- `Cgroup.resume(identity, attempt_id=..., root_identity=...)` verifies exact
  attempt/root/generation-derived path and directory device/inode against real
  kernel descriptors. Missing, replaced, or malformed identity is unproven.
- Use `preexec_fn=group.join_current_process`, `close_fds=True` in the trusted
  single-threaded host guardian. The hook writes only its own current PID to
  cgroup.procs and confirms membership before exec. It opens/closes control
  descriptors locally; no pass_fds entry is needed for this backend. Python
  preexec_fn must not be used by a multithreaded launcher.
- `group.kill_and_wait(timeout=3.0)` writes cgroup.kill and waits for kernel
  cgroup.events populated=0. It returns a JSON dictionary including identity,
  observed events, kill_written, populated, backend, and observed_at. Failure
  raises CgroupUnavailable or CgroupCleanupUnproven; no PID recovery or backend
  fallback occurs. Cleanup timeout bounds are (0,60] finite seconds.
- Guardian integration must hold its inherited ownership flock through cleanup
  proof, record failures as cleanup-unproven, and fence all executable release
  while cleanup runs. Empty groups are retained for durable recovery proof;
  eventual garbage collection is outside scope. No cleanup acknowledgment can
  infer external effect resolution.
- Parent integration places controller/guardian outside payload Insula and
  passes only the writable attempt directory into it; cgroup controls and root
  descriptors must remain inaccessible to payload. Membership is established
  before launching bwrap. This is a required integration assumption, not proof
  provided by this standalone module.

## Fresh evidence

`python3 tools/temp_root.py -- python3 -m unittest -v controller.durable.cgroup_test`
completed with 9 tests: 7 passed, 2 explicitly skipped. Always-on tests cover
ordinary directories with convincing fake control files, missing/relative/
symlink roots, bounded attempt IDs, malformed durable identities, timeout bounds,
root binding refusal, and a real Popen launch showing unavailable membership
prevents a payload marker from being written.

The two real kernel tests require CORENIUS_TEST_CGROUP_ROOT. When supplied,
invalid delegation fails rather than skipping. They check preexec membership,
an already detached setsid descendant, no inherited cgroup control descriptors,
JSON round-trip resume and kernel empty proof, generation/root inode mismatch,
actual removed/recreated subgroup inode mismatch, and refused executable release.
These positive cgroup behaviors have NOT been verified on this host.

Read-only host probe: CORENIUS_TEST_CGROUP_ROOT is unset, /sys/fs/cgroup is
cgroup2fs, os.access(root, W_OK) is False, and validation raises
`CgroupUnavailable: delegated cgroup root is not writable`. Mountinfo advertises
rw for the mount; the current user lacks writable delegation. Calling this a
read-only mount would be inaccurate. No fake cgroup files are used as positive
kernel cleanup evidence.

Initial test run was red with ModuleNotFoundError for the missing backend;
focused green evidence above follows implementation. Python compilation and
whitespace checks completed. Broader integrated runtime and guardian lock checks
belong to the parent integration task.

## Primary source

The [Linux kernel cgroup v2 documentation](https://docs.kernel.org/admin-guide/cgroup-v2.html)
states cgroup2 magic 0x63677270, PID migration through cgroup.procs, inherited
child membership, subtree populated evidence in cgroup.events, and cgroup.kill's
recursive SIGKILL behavior with concurrent fork handling. API containment
inferences depend on the parent withholding cgroup control access from payload.

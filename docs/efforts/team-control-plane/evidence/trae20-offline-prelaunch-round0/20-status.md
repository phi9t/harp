# Qualified inventory and process fence repair

Commit read:4b3d2634d85b62f21d97ab9c41b0d50b282d89ae. Corenius accepted supervisor.rs inventory/pin/signal principles were read; no Rust/source mainline edits, builds or services. Earlier freezes remain immutable and unadmitted.

Actual red: NamespaceSupervisor.members scanned real same-UID1018 PID32161, whose readable NSpid is a single host component32161 but whose ns/pid link returns PermissionError13. The regression asserts the scan has no such error; it failed before edits and passes after the qualified exclusion. The red source digest is retained. This is an observed host-profile exclusion, not a claim that every namespace link is readable.

The observer must have a validated single-component NSpid in the current host /proc frame. Each same-owner task's status is read before its namespace link. Exactly one valid host NSpid component excludes the task from child PID namespaces. Missing/duplicate/empty/malformed NSpid refuses; nested status proceeds to identity and unavailable/permission errors quarantine. Only kernel ENOENT/ESRCH are treated as process disappearance. The fixed user namespace maps all capsule descendants to this host UID; foreign host UIDs are excluded under that bounded premise. No cmdline-only/ancestry-only membership proof and no broad PermissionError skipping.

ProcessFence opens and validates the namespace FD against device/inode and recorded PID/starttime/namespace identity, holding the descriptor through final inventory and retained stop proof. Signaling opens a pidfd, rechecks recorded identity after opening, then sends SIGKILL through that stable descriptor. Mismatch or missing capability refuses; supervisor records quarantine and limits fallback to its own unreaped Popen child. Namespace FD closes only after stop proof is retained; subsequent fresh inventory without a live pin refuses.

Eight pure tests passed exit0: real host profile, hostile status fixtures, unreadable nested identity refusal, readable nested inclusion, invalid observer frame, actual existing observer namespace FD/pidfd_open capability, and recorded owner identity mismatch refusal. No stand-in/new process/namespace start or signal occurred; positive pidfd_send_signal and revised physical cleanup remain unverified. Previously verified26 guard/transport tests were not repeated because this phase forbids new physical fixture processes; their relevant guard sources remain unchanged. Compilation and diff checks passed.

Fresh physical proposal: exactly one injected setup-noise test, one fixed offline namespace,10s PID1 alarm,<=30s command, no retry/replacement, no Trae/provider/network services. Noise occurs only after a setsid descendant acknowledges readiness. Source captures and pins owned PID1 before parsing, and every post-Popen exception attempts bounded owned cleanup and retains explicit uncertainty. Root must independently review and explicitly admit this new frozen case before execution. Historical1749400/1793051 namespace identities remain unavailable/quarantined; qualified scanner repair does not reconstruct them.

Primary Linux references read2026-10-03:
- https://man7.org/linux/man-pages/man5/proc_pid_status.5.html : NSpid lists IDs from the procfs mount namespace through nested namespaces.
- https://man7.org/linux/man-pages/man7/pid_namespaces.7.html : existing process PID-namespace membership cannot change; namespace-init death kills its members.
- https://man7.org/linux/man-pages/man2/pidfd_open.2.html and https://man7.org/linux/man-pages/man2/pidfd_send_signal.2.html : pidfds identify a process rather than a recyclable numeric PID.
- https://man7.org/linux/man-pages/man7/namespaces.7.html : an open namespace FD preserves the namespace object.

These kernel rules support the qualified seam; they do not verify Trae retries, streaming, quota/billing, native-ID ordering, two-layer capsule composition or ticket04/20 acceptance. All runtime admission remains withheld.

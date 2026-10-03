# Repair offline fixture inventory before physical admission

Status: source repair authorized; physical/Trae/provider launch withheld.
Corenius source read: `264756edd550225449202b6f20ac4b560808d14e`, supervisor.rs inventory/pidfd seams.

The frozen offline candidate reports26 pure tests passing, but source review
found same-owner PID32161's namespace link inaccessible. Root independently
reproduced PermissionError13: host UID1018, readable status with a single NSpid
component, sd-pam process, start_ticks10769. Earlier cmdline-filtered inventory
did not establish complete namespace-link visibility. Historical namespaces
1749400/1793051 remain quarantined; no new physical case has been launched.

Reuse the accepted supervisor's host-proc NSpid single-component exclusion before
namespace-link inspection. Missing/unreadable status or nested NSpid cannot be
silently skipped; they retain an inventory gap. Add pidfd identity recheck and
signal plus a validated namespace FD held through stop/inventory. Preserve all
old frozen bytes; issue new red/green and freeze before independent admission.

This follows the Linux documented distinction: a process's PID namespace is
fixed at creation, while setns changes the namespace for subsequently created
children. Namespace-init termination kills its namespace members.
[Linux PID namespace manual](https://man7.org/linux/man-pages/man7/pid_namespaces.7.html)
[Python pidfd signaling](https://docs.python.org/3/library/signal.html#signal.pidfd_send_signal).
The accepted Rust observer already uses stronger pinned process/namespace
identity checks; this offline Python fixture must not relax them for live use.

One future setup-noise case may be admitted only after the fresh exact-source
freeze is independently inspected: one fixed offline namespace,10-second
fixture alarm, command maximum30 seconds, no automatic rerun/replacement.
No Trae/provider/network service, paid calls or cumulative budget reset is
admitted by this source-repair decision. Post-Popen failures still require
owned cleanup or precise quarantine. RPC nativeTurnId/config/retry/cap keys
remain separate unproved runtime prerequisites.

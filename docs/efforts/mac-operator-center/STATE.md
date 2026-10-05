# MAC operator center state

Status: operator tools reviewed and installed; product qualification separate.
Owner: main lead. Product implementation coordinator: corenius-impl.

The final operator source at e6ae1add9841a2bc0eb26a441a5b322e2db8f090
passed eight dashboard tests (2.195s) and fourteen tmux tests (13.080s).
Dashboard tests include a
real 8×24 curses terminal; manager tests use disposable real tmux sessions.
Compilation and diff checks passed. Original native clients remain in place.

Standards review found no documented violation. Its nonblocking concern was
duplicate config validation between the two standalone tools.

Spec review found two required repairs: an exact-binding unowned client could
be duplicated; malformed legacy adoption entries could terminate the watch loop.
Independent branch review found one required repair: new native windows did not
retain exited clients under default tmux settings. One narrow repair round fixed
all three issues with regression evidence. Spec and branch reviewers confirmed
the repaired source. The dashboard had no blocking finding from these reviews.

## Installed view

Verified 2026-10-05 03:54:55 UTC: session corenius-team, dashboard @69/%79,
coordinator @70/%80, historical worker @71/%81, current trial writer @72/%82,
view manager @73/%83. Original @70 and @71 IDs and processes were preserved.
The manager followed the new concrete trial writer binding automatically.
Its manifest reported health ok with no errors and advancing observations.
Only the owned dashboard pane was replaced; no native input was sent.

Attach with `tmux attach -t corenius-team`. The seven dashboard pages refresh
every three seconds. Source timestamps and missing data remain visible.

Host-local config, installed code, manifest, deployment and verification records:
/data02/home/philip.yang/.local/state/corenius/mac-team-view/01a10084-dc3a-7743-b0c1-4694d5ecc621/managed/

Luna's runtime reported a model-capacity error during deployment. Restart was
requested using the same model; its observation timestamp remains the coverage
authority. The manager's health does not imply that Luna is running.
Restart recovered: Luna saved a fresh assessment at 03:56:06 UTC. It confirmed
healthy operator view metadata and retained the distinction from pending product
runtime qualification. The capacity interruption remains part of coverage history.

Next: retain this operator baseline and add explicitly declared product runtime
bindings when the core coordinator publishes them. Product baseline acceptance
and mainline landing remain with the existing effort. Source branches and old
operator artifacts are preserved.

## Full views and client update — 2026-10-05 04:34 UTC

Current operator source: 15ba811ea37f03ee344a0e0c473cf4df5a67e785.
The focused operator suite passed 43 tests in 25.314 seconds. Fresh compilation
and diff checks passed. Standards, Spec and independent branch reviewers
resolved their scoped findings. These results qualify the operator tools;
they do not accept the product baseline or the disposable live trial.

Controller and workspace panes now read bounded, explicit public projections.
They show controller operations, task/attempt states, source timestamps,
independent role memberships and attempt workspace claims. Missing physical
worker slot IDs stay unknown. Observed inventory remains distinct from claims.
The native Kata TUI uses the exact trial instance and project.

The native client maintainer accepts a public runtime roster with concrete
endpoint/thread bindings. It preserves source memberships when roles share a
client. Missing or stale roster state cannot admit new clients. It sends no
agent input and starts no model server. Client windows retain exited processes.

Codex 0.160.0 is installed. At 04:34:08, executable inspection confirmed the
coordinator, main lead and current implementation continuation clients use that
binary. At 04:34:44, metadata-only thread/read returned their exact existing
thread identities. No turns were requested. Current implementation thread is
01a10a48-15be-7d63-a501-8405ed1f5a23. No implementation assignment changed.

The two live product observer clients also opened with 0.160.0 through existing
attempt-scoped UI transports. Their gateways subsequently stopped. Previous
public bindings remain retained as public-binding.previous.json and cannot
publish dead endpoints as current. Reattachment awaits coordinator-owned
same-deployment service reconciliation. Observer transport refuses worker turn
input; its launch is not a worker restart or evidence of task execution.

The controller became unreachable, and the product trial session then the
operator session disappeared. Other tmux sessions remained. Cause is unknown.
The retained failure.json reports TimeoutError, "two useful candidates
captured", exit 1. The last successful public controller projection is cursor
114, observed 04:24:58; the display marks it stale. Core CURRENT already admits
narrow same-deployment restoration. No core service or task was restarted by
this operator work. Coordinator notices were queued; adoption is unconfirmed.

Operator views were recreated in corenius-team at 04:33. Verified windows:
overview @104; coordinator @105; main lead @106; implementation worker @107;
controller @108; workspaces @109; Kata @110; public state feed @111; native view
manager @112. All owned panes were alive at 04:34:08. Kata reads succeed.
Controller reads return ConnectionRefusedError; the runtime roster is missing,
and manager health is degraded. The three-second loops refresh views only.
Existing transcript-monitoring timers and private ledgers remain unchanged.

Evidence: evidence/client-update-20261005.json. Host-local detailed records:
managed/restart-verification.json, client-binding-recheck.json,
deployment-after-client-update.json and public-feed-health.json. These records
are timestamped observations, not a continuous liveness claim.

Next: the core coordinator reconciles the same deployment and publishes new
concrete product UI bindings. The operator owner then reattaches the product
lead and both worker clients. No broad gate or new mission is required for
these already-scoped views. Mainline landing remains pending.

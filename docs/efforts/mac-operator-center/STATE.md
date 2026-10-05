# MAC operator center state

Status: public-export view adapters reviewed and installed; core recovery remains coordinator-owned.
Owner: main lead. Product implementation coordinator: corenius-impl.

Current operator source: 815ba89dc18eeb349aedb6e457577ce5e4292399.
Current verification: 49 tests PASS (27.039s), compilation/diff checks PASS,
scoped independent Standards/Spec/branch review PASS. Installed 04:47 UTC.
Use only corenius-team. Views read CURRENT.public_exports directly; the public
controller/runtime exports still say unavailable. Existing trial-r2-* service
windows are preserved. Current implementation identity comes from CURRENT,
which names restoration writer01a10a5b at the latest 04:45 observation.
Earlier dated deployment details below remain history.

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

## Coordinator public export adoption — 2026-10-05 04:48 UTC

Host config now directly references the coordinator-owned explicit controller,
Kata and runtime-roster export files. The operator state feed reads those public
files only; it no longer queries the controller or Kata. Source observations
retain their original timestamps and raw-file digests. No database or auth
reads, core admission, model turn, new timer or private assessment-ledger writes.

The state renderer supports the coordinator's control_center envelope. It shows
unavailable current state separately from cursor114 last-success history.
Kata items show stable WorkRefs, native Kata status and recorded
corenius.execution state separately. These summaries do not establish liveness.
The maintainer accepts optional public status/reason. Unavailable/pending
publication admits zero clients; prior memberships remain unknown. Unknown
fields/status and credential endpoints still fail closed. Reason is not emitted.

At04:48, upstream files remained observed04:32:24 and stale. The public runtime
roster was unavailable/empty, so manager health was degraded with explicit source
provenance. Core restoration had created trial-r2-* windows in corenius-team;
this operator installation preserved all five observed service window IDs.
Service window existence is not proof of controller or model qualification.
Only owned manager/controller/workspace view processes were replaced.

Core CURRENT now names fresh restoration actor01a10a5b-b86f-7953-a73c-059d7e05de87.
Native view manager followed that binding and retained the prior client as
historical. Authenticated product clients await published transport readiness
and private launch grants; these never belong in public roster files.

Evidence: evidence/public-export-adoption-20261005.json. Detailed installed
records remain managed/public-export-deployment.json and public-export-verify.json.
Root retains core admissions/effects; main lead retains UI scripts/deployment.
Solo evidence remains retained; assembled baseline/mainline acceptance and two
useful episode overlap/stale reintegration remain outside this view proof.

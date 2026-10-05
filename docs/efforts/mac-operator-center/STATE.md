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

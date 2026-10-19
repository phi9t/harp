# Nested root/child feasibility observation

Observed 2026-10-04 on the development host, bubblewrap 0.8.0. Corenius source read `a21316f74672f55f3c22eff94f93ce1511b51822`. This is a model-free, disposable host feasibility check for ROOT-INSULA.md; no implementation workers, live services, credentials or existing data were modified.

The host Python wrapper created two fresh directories using TemporaryDirectory and launched both root workloads before collecting either result. Each contained an A/B marker in its own `attempt/` and a synthetic `controller-secret` file. Both parent workloads launched a child and waited for it; all disposable data was removed after both completed.

Each bwrap plan used `--die-with-parent --unshare-all --tmpfs / --ro-bind /usr /usr`, the `/bin`/`/lib`/`/lib64` symlinks into `/usr`, a private `/proc`, `/dev`, `/tmp`, `/run`, cleared environment with fixed PATH and dropped capabilities. Parent bound its own fresh directory to `/state`; child rebuilt that plan with only `/state/attempt` bound to `/workspace`. The child Python command reported its marker, existence of `/state/controller-secret`, namespace links and `/proc/net/route` line count. Wrapper used bounded waits of 12 seconds and each parent bounded its child wait at 8 seconds. No shell interpolation was used.

| Workload | Exit | Marker | Parent state visible | User / mount / PID / network / IPC namespace numbers |
| --- | --- | --- | --- | --- |
| A root | 0 | A | Own state intentionally projected | 4026539866 / 4026539648 / 4026539687 / 4026539689 / 4026539686 |
| A child | 0 | A | false | 4026540117 / 4026540034 / 4026540037 / 4026540039 / 4026540036 |
| B root | 0 | B | Own state intentionally projected | 4026539865 / 4026539682 / 4026539728 / 4026539730 / 4026539727 |
| B child | 0 | B | false | 4026540116 / 4026539868 / 4026539871 / 4026539873 / 4026539870 |

Both children reported one route-table line (the header), with no configured routes; all stderr was empty. Namespace numbers are historical observations, not persistent execution identities and not future handles.

Inference: this host can execute the proposed root/child namespace shape concurrently, without projecting root controller storage to the children. It does not establish forbidden signalling, descriptor inheritance, API permissions, provider credential separation, cgroup control, persistent SQLite, native tmux/Codex attachment, all-tool confinement, cross-instance stop safety or hostile-code resistance. Host `/usr` was an explicit read-only, non-hermetic input; no pinned frozen image was built. Product qualification must exercise those contracts separately.

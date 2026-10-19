# Native durable qualification

All 73 durable tests passed with zero skips in 22.308 seconds, using Python 3.11.2 and SQLite 3.51.3. Production SQLite admission passed without the disposable-test override.

The initial login scope lacked writable cgroup delegation. The user systemd manager supports Delegate=yes; the qualifier created a dedicated transient scope with a 600-second runtime limit and five-second stop grace. Scope membership was checked before selecting the delegated root. No system package or existing cgroup was modified.

The private SQLite library was compiled under .scratch/retro-fixes/sqlite from https://sqlite.org/2026/sqlite-amalgamation-3510300.zip. Before compilation, sqlite3.c was verified against the [upstream release SHA3-256](https://www.sqlite.org/releaselog/3_51_3.html): 32d5424f97e0a7fc5ed2f6335afbb58be4e0298bd7117a34e39d345ff13d859e. Compile command: `cc -O2 -fPIC -shared -DSQLITE_ENABLE_COLUMN_METADATA -Wl,-soname,libsqlite3.so.0 -o libsqlite3.so.0 sqlite3.c -lpthread -ldl -lm`. LD_LIBRARY_PATH selected this private library for the qualifier and its children; the system library remains unchanged.

Command (from integration worktree):

```sh
python3 tools/temp_root.py -- env LD_LIBRARY_PATH="$PWD/.scratch/retro-fixes/sqlite" python3 -m controller.durable.qualify --user-scope --output .scratch/retro-fixes/qualification-reviewed.json
```

qualification.json binds runtime and Insula source hashes, environment and executed case identities. This qualifies the tested host/backend combination, not physical power-loss durability, remote fencing, GPUs or external exactly-once effects.

Raw log SHA-256: 5fda61725171918a8ea09324f873f90552b7b7c218709ebee18b479fb7afb191

A separate live cancellation probe returned exit143, left status incomplete, and confirmed the unique scope LoadState was not-found.

# Reference baseline

Source archive SHA-256: 30f55eef3f5aa10b84df0c9b8d31aa0ec28300dc08da7acfd53ccd8be88f180b.

Command from extracted reference:
`python3 /data02/home/philip.yang/workspace/corenius-durable/tools/temp_root.py -- python3 -m unittest discover -s tests -v`

Observed: 35 tests passed in 12.349 seconds; Python 3.11.2 and SQLite 3.40.1. Tests used the supplied disposable-state SQLite override.

Separate bubblewrap smoke succeeded with a private /run/corenius-tmp mount. Cgroup v2 exists, but its root is not writable by the current UID. No production SQLite or cgroup qualification is inferred.

Insula integration red: `python3 tools/temp_root.py -- python3 -m unittest controller.durable.insula_test -v` failed because controller.durable.insula did not exist. Green after implementation: four tests passed, including real bubblewrap temporary storage and mount boundary.

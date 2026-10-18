`deployctl` exposes a `status` subcommand:

```
python -m deployctl status PATH [--format text|json]
```

`PATH` is a CSV file with a `state` column. Count rows whose state is `ok`,
`failed`, or `queued`, ignoring unknown states. The default text format must
print one line per state in this order:

```
ok: N
failed: N
queued: N
```

When `--format json` is supplied to the `status` subcommand, stdout must be a
single JSON object with exactly the keys `failed`, `ok`, and `queued`, and no
extra text. Fix `deployctl.py`; the project uses uv offline and has no
dependencies.

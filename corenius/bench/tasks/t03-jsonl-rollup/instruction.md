`rollup.py EVENTS SERVICE STATUS` reads newline-delimited JSON records. Each non-empty line is an object with `service`, `status`, and integer `amount` fields. It must print exactly `count=N total=M`, where `N` is the number of records whose `service` and `status` both match the requested arguments, and `M` is the sum of their `amount` values. Corrections may be negative. Records for other services or statuses must not affect either number.

Fix `rollup.py`; keep it dependency-free.

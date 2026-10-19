This Blade workspace models a small lgtorch-style fused op package: an IDL file is code-generated into `gen/params.h`, `csrc/` builds the native implementation and a shared `cc_plugin`, and `python/ops.py` loads the plugin with `ctypes`.

The operation must support a new `float bias` field end to end. The final behavior is:

```text
y[i] = x[i] * scale + bias
```

Make all three changes:

- t1 updates `idl/params.idl` so generated `Params` has fields in IDL order: `scale`, then `bias`.
- t2 updates the native implementation and plugin so the exported v2 entry point uses `Params::bias`.
- t3 updates the Python wrapper to pass `bias` through a `ctypes.Structure` matching the generated layout.

The verifier runs:

```sh
blade-offline build //...
PYTHONPATH="$PWD/python" python3 tests/check_ops.py
```

Do not edit `tools/gen_params.py`, `tests/`, or unrelated Blade tooling.

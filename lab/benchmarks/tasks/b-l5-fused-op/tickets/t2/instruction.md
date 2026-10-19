Update the native side of the fused op.

Required behavior:

- `csrc/fused.cc` must compute `y[i] = x[i] * params->scale + params->bias`.
- `csrc/plugin.cc` must export `extern "C" int fused_op_apply_v2(const Params*, const float*, float*, int)`.
- `csrc/BUILD` must link the plugin against `:fused_core`; leaving the core as an undefined symbol is not acceptable.
- Update the top-level `BUILD` component manifest with `native-plugin`.

This ticket depends on the IDL contract from t1: `Params` fields are generated in IDL order.
Do not edit Python.

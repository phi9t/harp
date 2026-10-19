Update `python/ops.py` so the ctypes wrapper passes `bias` through the generated `Params` struct.

Contract from t1: the generated C struct layout is IDL order, `scale` then `bias`, both `float32`.
Use the native v2 entry point from t2: `fused_op_apply_v2(const Params*, const float*, float*, int)`.
Update the top-level `BUILD` component manifest with `python-wrapper`.
Do not edit native sources.

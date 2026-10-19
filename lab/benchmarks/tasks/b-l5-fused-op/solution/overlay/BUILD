FUSED_OP_COMPONENTS = ["params-codegen", "native-plugin", "python-wrapper"]

gen_rule(
    name="params_codegen",
    srcs=["idl/params.idl"],
    outs=["params.h"],
    outdir="gen",
    cmd="python3 tools/gen_params.py $FIRST_SRC gen/params.h",
)

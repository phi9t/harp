cc_library(
    name="fused_core",
    srcs=["fused.cc", "fused.h"],
    deps=["//:params_codegen"],
    incs=["."],
    export_incs=["."],
)

cc_plugin(
    name="fused_plugin",
    srcs=["plugin.cc"],
    deps=["//:params_codegen"],
    incs=["."],
)

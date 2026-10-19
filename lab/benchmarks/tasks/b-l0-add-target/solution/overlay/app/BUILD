cc_library(
    name="greeting",
    srcs=["greeting.cc"],
    export_incs=["."],
)

cc_binary(
    name="main",
    srcs=["main.cc"],
    deps=[":greeting"],
)

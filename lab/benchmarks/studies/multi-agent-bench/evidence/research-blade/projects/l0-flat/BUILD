cc_library(
    name="answer",
    srcs=["answer.cc"],
    export_incs=["."],
)

cc_binary(
    name="hello",
    srcs=["main.cc"],
    deps=[":answer"],
)

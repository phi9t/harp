load("//BUILD.rules", "wrapped_cc_library")

wrapped_cc_library(
    name="hidden_part",
    srcs=["hidden_part.cc"],
    needs_config=True,
    export_incs=["."],
)

cc_binary(
    name="macro_probe",
    srcs=["macro_probe.cc"],
    deps=[":hidden_part"],
)

load("//BUILD.share", "settle_defaults")

settle_defaults()

library_settler(prefer_static=["lib:chooser"])

cc_binary(
    name="settler_app",
    srcs=["main.cc"],
    deps=["//lib:chooser"],
)

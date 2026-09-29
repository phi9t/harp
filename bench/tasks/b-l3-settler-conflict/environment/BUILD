load("//BUILD.share", "settle_defaults")

settle_defaults()

# This local override conflicts with BUILD.share. It forces a shared-library
# link path that exposes an internal symbol which is intentionally not part of
# the app's static link closure.
library_settler(prefer_dynamic=["lib:chooser"])

cc_binary(
    name="settler_app",
    srcs=["main.cc"],
    deps=["//lib:chooser"],
)

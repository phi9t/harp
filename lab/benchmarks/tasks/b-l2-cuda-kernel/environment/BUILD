import os

cuda_path = os.environ.get("CUDA_PATH", "/usr/local/cuda")
cuda_lib = os.path.join(cuda_path, "lib64")
gcc_root = os.environ.get("BLADE_GCC_ROOT", "/opt/tiger/typhoon-blade/gccs/x86_64-x86_64-gcc-830")
gcc_lib = os.path.join(gcc_root, "lib")

cuda_arch_flags = [
    "-gencode=arch=compute_100,code=sm_100",
    "--allow-unsupported-compiler",
]

cuda_link_flags = [
    "-L%s" % cuda_lib,
    "-Wl,-dynamic-linker,/lib64/ld-linux-x86-64.so.2",
    "-Wl,-rpath,%s" % cuda_lib,
    "-Wl,-rpath,%s" % gcc_lib,
]

cuda_cpp_flags = [
    "-I%s/include" % cuda_path,
]

cu_library(
    name="kernel",
    srcs=["kernel.cu"],
    defs=["BLADE_CUDA_KERNEL=1"],
    incs=["."],
    export_incs=["."],
    extra_cuflags=cuda_arch_flags,
)

cc_binary(
    name="cuda_app",
    srcs=["main.cc"],
    deps=[":kernel"],
    incs=["."],
    extra_cppflags=cuda_cpp_flags,
    extra_linkflags=cuda_link_flags,
)

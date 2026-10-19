1. [X] CUDA compilation with Blade works offline in the required `bwrap --unshare-net --clearenv` shape if the real CUDA tree is bound into the scratch tree and `NVCC`/`CUDA_PATH` point at that bound path.
2. [X] The old `/usr/local/cuda/bin/nvcc: not found` is path visibility, not an ELF interpreter problem: `/usr/local/cuda` is a symlink into `/home`, and the safe recipe hides `/home` with `--tmpfs /home`.
3. [R] Blade does not add a second chroot for the local Ninja path I used; it builds by spawning Ninja from Python, so the outer bwrap mount namespace is the namespace that matters.
4. [R] Blade CUDA targets support `NVCC`, `CUDA_PATH`, target `extra_cuflags`, command/env `--cuflags`/`CUFLAGS`, and a `CUSTOM_NVCC` rule branch; I verified the `NVCC` plus `CUDA_PATH` plus extra bind path.
5. [X] The verified generated Ninja rule invoked the scratch-bound CUDA 13.2 `nvcc` and compiled a `__global__` kernel with `-gencode=arch=compute_100,code=sm_100`.
6. [X] The smallest runtime loader fix I verified is `--skip-bundle` plus link flags setting the ELF interpreter to `/lib64/ld-linux-x86-64.so.2` and RPATHs for CUDA and Blade GCC libraries.
7. [X] `cc_test` works offline through `blade test` with a local vendored `cpp3rdlib/minitest` package when `cc_test_config(gtest_libs=[], gtest_main_libs=[])` disables the default gtest deps.
8. [X] Passing `blade test` returned shell rc `0`; a failing `cc_test` returned shell rc `250`, while Blade source returns internal `ETEST = 2042`, which truncates to `250` as a Unix process status.
9. [X] CUDA runtime execution is not verified in this worker namespace: there are B200 GPUs in `/proc/driver/nvidia`, but no `/dev/nvidia*`, `/dev/dri`, or `/dev/nvidia-caps` nodes to bind.
10. [X] I produced a reusable script/project, but it intentionally exits nonzero here because the build/test mechanics work and the GPU runtime fails before a real kernel can run.

## Numbered findings

1. [X] Root cause of "file exists, Ninja says not found"

   [X] The host path exists and is executable before entering the sandbox:

   ```sh
   ROOT=$PWD
   S=$(mktemp -d -p "$ROOT/docs/efforts/multi-agent-bench/evidence/research-blade-cuda" path-probe-XXXXXX)
   mkdir -p "$S/home"
   ls -ld /usr/local/cuda /usr/local/cuda/bin/nvcc
   readlink -f /usr/local/cuda
   readelf -l /usr/local/cuda/bin/nvcc | grep 'interpreter'
   /usr/local/cuda/bin/nvcc --version | sed -n '1,4p'
   ```

   [X] Output excerpt:

   ```text
   lrwxrwxrwx 1 nobody nogroup       20 Mar 17  2026 /usr/local/cuda -> /home/zfc/cuda_13_2/
   -rwxr-xr-x 1 nobody nogroup 33994064 Mar 17  2026 /usr/local/cuda/bin/nvcc
   /data00/home/zfc/cuda_13_2
         [Requesting program interpreter: /lib64/ld-linux-x86-64.so.2]
   Cuda compilation tools, release 13.2, V13.2.51
   ```

   [X] The same `/usr/local/cuda/bin/nvcc` path disappears under the safe sandbox because `--tmpfs /home` hides the symlink target:

   ```sh
   bwrap --ro-bind / / --dev-bind /dev /dev --bind "$S" "$S" \
     --tmpfs /home --tmpfs /tmp --unshare-net --clearenv \
     --setenv HOME "$S/home" --setenv PATH /usr/local/bin:/usr/bin:/bin \
     --chdir "$S" /bin/sh -c \
     'set -x; ls -ld /usr/local/cuda /home/zfc /usr/local/cuda/bin /usr/local/cuda/bin/nvcc; /usr/local/cuda/bin/nvcc --version'
   ```

   [X] Output excerpt:

   ```text
   + ls -ld /usr/local/cuda /home/zfc /usr/local/cuda/bin /usr/local/cuda/bin/nvcc
   ls: cannot access '/home/zfc': No such file or directory
   ls: cannot access '/usr/local/cuda/bin': No such file or directory
   ls: cannot access '/usr/local/cuda/bin/nvcc': No such file or directory
   lrwxrwxrwx 1 nobody nogroup 20 Mar 17  2026 /usr/local/cuda -> /home/zfc/cuda_13_2/
   + /usr/local/cuda/bin/nvcc --version
   /bin/sh: 1: /usr/local/cuda/bin/nvcc: not found
   ```

   [I] The misleading `not found` is therefore the shell failing to resolve the pathname through a hidden symlink target, not `execve` finding the file and then failing on a missing ELF interpreter.

   [R] Blade's CUDA library rule uses the `NVCC` environment variable verbatim, defaulting to bare `nvcc`: `/opt/tiger/typhoon-blade/src/blade/targets/cuda/target_cu_library.py:95-107`. The general CUDA object generator does the same at `/opt/tiger/typhoon-blade/src/blade/core/rules_generator.py:849-894`.

   [R] Blade starts Ninja with `subprocess.Popen(cc, ..., shell=True)` after logging `start build with ninja`; this local path does not show an additional Blade chroot around Ninja. Evidence: `/opt/tiger/typhoon-blade/src/blade/cmds/cmd.py:1228-1231`.

   [I] The earlier lead note that "Blade sailfish environment probably does not see `/usr/local/cuda`" was close but not precise: the immediate mechanism I verified is the outer bwrap hiding `/home/zfc/cuda_13_2`, which makes the `/usr/local/cuda` symlink dangling inside the sandbox.

2. [X] Working offline CUDA build recipe

   [X] The reusable script is `docs/efforts/multi-agent-bench/evidence/research-blade-cuda/run_blade_cuda.sh`. It creates a fresh run root with `mktemp -d -p "$RUNS_DIR" blade-cuda-XXXXXX`, copies the project template there, resolves `CUDA_REAL="$(readlink -f /usr/local/cuda)"`, and binds the real CUDA tree to `$RUN_ROOT/cuda`. Evidence: `run_blade_cuda.sh:4-18`, `run_blade_cuda.sh:42-60`.

   [X] Exact Blade invocation shape used by the script:

   ```sh
   bwrap --ro-bind / / --dev-bind /dev /dev --bind "$RUN_ROOT" "$RUN_ROOT" \
     --ro-bind "$CUDA_REAL" "$CUDA_MOUNT" --tmpfs /home --tmpfs /tmp \
     --unshare-net --clearenv --setenv HOME "$HOME_DIR" \
     --setenv PATH "/opt/tiger/ss_bin:$CUDA_MOUNT/bin:/usr/local/bin:/usr/bin:/bin" \
     --setenv USER bench --setenv NVCC "$CUDA_MOUNT/bin/nvcc" \
     --setenv CUDA_PATH "$CUDA_MOUNT" --setenv BLADE_GCC_ROOT "$GCC_ROOT" \
     --setenv BLADE_RUN_LIBRARY_PATH "$RUN_LIBRARY_PATH" --setenv BLADE_LOADER "$LOADER" \
     --setenv TMPDIR /tmp --chdir "$PROJECT" \
     /opt/tiger/ss_bin/blade "$subcmd" --toolchain=x86_64-gcc830 --skip-bundle "$@"
   ```

   [X] The last full script run was:

   ```sh
   bash docs/efforts/multi-agent-bench/evidence/research-blade-cuda/run_blade_cuda.sh
   ```

   [X] Output excerpt:

   ```text
   RUN_ROOT: /data02/home/philip.yang/workspace/corenius-worker-research-blade-cuda/docs/efforts/multi-agent-bench/evidence/research-blade-cuda/runs/blade-cuda-Q6Wmyt
   CUDA_REAL: /data00/home/zfc/cuda_13_2
   CUDA_MOUNT: /data02/home/philip.yang/workspace/corenius-worker-research-blade-cuda/docs/efforts/multi-agent-bench/evidence/research-blade-cuda/runs/blade-cuda-Q6Wmyt/cuda
   COMMAND[nvcc-visible]: inside_project_cmd /bin/sh -c ...
   RC[nvcc-visible]: 0
   Cuda compilation tools, release 13.2, V13.2.51
         [Requesting program interpreter: /lib64/ld-linux-x86-64.so.2]
   ```

   [X] The cold Blade build inside bwrap succeeded:

   ```text
   COMMAND[cold-build]: blade_cmd build //:cuda_app //:cuda_test //:host_test //:host_test_fail
   RC[cold-build]: 0
   WALL_MS[cold-build]: 1727
   (23:17:05)Blade(info): TOOLCHAIN MODE: INSIDE
   (23:17:05)Blade(info): CC=/opt/tiger/typhoon-blade/gccs/x86_64-x86_64-gcc-830/bin/gcc
   (23:17:05)Blade(info): CXX=/opt/tiger/typhoon-blade/gccs/x86_64-x86_64-gcc-830/bin/g++
   (23:17:05)Blade(info): start build with ninja: (.blade_tools/tools/ninja -j216 -k1 -f build.ninja --cost -dkeepdepfile)
   [11/14] NvccObject kernel.cu
   [13/14] LINK build64_release/cuda_test
   [14/14] LINK build64_release/cuda_app
   (23:17:06)Blade(info): success
   ```

   [X] A warm Blade build in the same run root succeeded in `1149` ms:

   ```text
   COMMAND[warm-build]: blade_cmd build //:cuda_app //:cuda_test //:host_test //:host_test_fail
   RC[warm-build]: 0
   WALL_MS[warm-build]: 1149
   [4/5] LINK build64_release/cuda_test
   [5/5] LINK build64_release/cuda_app
   (23:17:14)Blade(info): success
   ```

   [X] The script exits `1` on this worker because it treats GPU runtime failures as real failures after completing the compile/test mechanics. Evidence: `run_blade_cuda.sh:173-190`; last-run output ended with `GPU_RUNTIME_FAILURES: 2`.

3. [R] Supported CUDA control surface I found

   [R] `NVCC` is the primary way to select the CUDA compiler for ordinary `cu_library` and `cu_binary`: `/opt/tiger/typhoon-blade/src/blade/targets/cuda/target_cu_library.py:95-107` and `/opt/tiger/typhoon-blade/src/blade/targets/cuda/target_cu_binary.py:148-158`.

   [R] `CUDA_PATH` is how Blade discovers CUDA includes when constructing its `-I.../include` and `-I.../samples/common/inc` list; otherwise it falls back to `/usr/local/cuda_<version>`. Evidence: `/opt/tiger/typhoon-blade/src/blade/compilers/compiler_cc.py:387-397`.

   [R] Target-local `extra_cuflags` and legacy `extra_flags` are supported on `cu_library`; they are stored together in `self.data['extra_cuflags']`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cuda/target_cu_library.py:110-143`, `/opt/tiger/typhoon-blade/src/blade/targets/cuda/cu_target.py:34-66`.

   [R] `extra_cppflags`, `defs`, `incs`, `export_incs`, optimization flags, and `extra_cuflags` are combined for CUDA object rules. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cuda/cu_target.py:96-132`.

   [R] Command-line `--cuflags` exists and defaults from `CUFLAGS` or `CUSTOM_CUFLAGS`; CUDA target setup appends those command flags into `extra_flags`. Evidence: `/opt/tiger/typhoon-blade/src/blade/cmds/cmd_build.py:108-112`, `/opt/tiger/typhoon-blade/src/blade/targets/cuda/cu_target.py:142-153`.

   [R] `CUSTOM_NVCC` selects a separate `custom_nvcc_object` rule; when set to anything other than `"0"`, `cu_target.py` emits `custom_nvcc_object` instead of `nvcc_object`, and `rules_generator.py` expands `CUSTOM_NVCC` as the command prefix. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cuda/cu_target.py:181-224`, `/opt/tiger/typhoon-blade/src/blade/core/rules_generator.py:896-901`.

   [I] A workspace-local wrapper script is supported by the same mechanism if `NVCC=/abs/path/to/wrapper` or `CUSTOM_NVCC=/abs/path/to/wrapper` points to an executable visible inside bwrap; I did not separately execute a wrapper experiment because the `NVCC=$CUDA_MOUNT/bin/nvcc` path already verified compilation.

   [R] Blade loads project configuration from `files/blade.conf`, `~/.bladerc`, `BLADE_ROOT`, and `BLADE_ROOT.local`. Evidence: `/opt/tiger/typhoon-blade/src/blade/core/workspace.py:347-352`.

   [X] I found no dedicated CUDA or NVCC keys in the installed top-level config and default config files:

   ```sh
   rg -n "cuda|CUDA|nvcc|NVCC|cuflags|CUFLAGS" \
     /opt/tiger/typhoon-blade/conf /opt/tiger/typhoon-blade/src/blade/configs
   ```

   [X] Output excerpt: no matches.

   [R] `cu_library` accepts `custom_deps` and `prebuilt` in its public signature, so dependency selection and prebuilt library paths are supported at the target surface. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cuda/target_cu_library.py:110-131`.

   [I] `custom_deps` and `prebuilt` can avoid or redirect dependency builds, but they are not the direct fix for the missing `nvcc` path. The direct fix is to make the compiler path and CUDA include/library paths visible inside bwrap.

4. [X] Minimal project that compiles `sm_100` CUDA

   [R] The minimal project lives under `docs/efforts/multi-agent-bench/evidence/research-blade-cuda/project/`.

   [R] The project disables default gtest deps in `BLADE_ROOT` so tests can use a local vendored framework. Evidence: `project/BLADE_ROOT:1-5`.

   [R] The `cu_library` target compiles `kernel.cu` with `extra_cuflags=["-gencode=arch=compute_100,code=sm_100", "--allow-unsupported-compiler"]`. Evidence: `project/BUILD:8-30`.

   [R] The CUDA source contains a real global kernel and launch: `__global__ void add_one_kernel(int *value)` and `add_one_kernel<<<1, 1>>>(device_value)`. Evidence: `project/kernel.cu:9-10`, `project/kernel.cu:42-43`.

   [R] The binary prints `BLADE_CUDA_RESULT answer=%d devices=%d device0=%s sm=%d%d` after calling `blade_cuda_answer(41)`. Evidence: `project/main.cc:21-28`.

   [X] The generated `build.ninja` from the last run used the scratch-bound CUDA compiler and included the `sm_100` flag:

   ```sh
   rg -n "nvcc_object|kernel\\.cu|gencode|cuda/bin/nvcc" \
     docs/efforts/multi-agent-bench/evidence/research-blade-cuda/runs/blade-cuda-Q6Wmyt/project/build.ninja
   ```

   [X] Output excerpt:

   ```text
   196:  command = .../runs/blade-cuda-Q6Wmyt/cuda/bin/nvcc -MMD -MF ${out}.d -ccbin /opt/tiger/typhoon-blade/gccs/x86_64-x86_64-gcc-830/bin/g++ -I.../cuda/include -I.../cuda/samples/common/inc ${includes} ${extra_flags} -o ${out} -c ${in}
   234:build build64_release/./kernel.objs/kernel.cu.o: nvcc_object kernel.cu
   235:  extra_flags = -gencode=arch=compute_100,code=sm_100 --allow-unsupported-compiler --compiler-options "-DBLADE_CUDA_KERNEL=1 -O2"
   ```

5. [X] Binary loading and execution mechanics

   [R] The build uses `--skip-bundle`, then the project supplies link flags for the host-visible ELF loader and runtime library search paths. Evidence: `run_blade_cuda.sh:140-143`, `project/BUILD:13-18`.

   [X] The built CUDA app used `/lib64/ld-linux-x86-64.so.2` and found `libcudart.so.13` from the scratch-bound CUDA tree:

   ```text
   COMMAND[binary-readelf]: inside_project_cmd /bin/sh -c readelf -l build64_release/cuda_app | grep "interpreter"; ldd build64_release/cuda_app
   RC[binary-readelf]: 0
         [Requesting program interpreter: /lib64/ld-linux-x86-64.so.2]
           libcudart.so.13 => .../runs/blade-cuda-Q6Wmyt/cuda/lib64/libcudart.so.13
           /lib64/ld-linux-x86-64.so.2
   ```

   [X] The process executed inside the same no-network bwrap and reached CUDA runtime code, but CUDA runtime returned an OS/device error:

   ```text
   COMMAND[run-binary]: inside_project_cmd .../runs/blade-cuda-Q6Wmyt/project/build64_release/cuda_app
   RC[run-binary]: 2
   CUDA_ERROR cudaGetDeviceCount OS call failed or operation not supported on this OS
   BLADE_CUDA_RESULT device_count=-1304
   ```

   [I] This verifies the loader and dynamic library problem is solved for process startup. It does not verify a successful B200 kernel run because CUDA failed before device selection and before `cudaMalloc`.

6. [X] `blade test` with a vendored local framework

   [R] The vendored local package is a normal local versioned dep: `cpp3rdlib/minitest:1.0@//cpp3rdlib/minitest:minitest`. Evidence: `project/BUILD:41-55`, `project/cpp3rdlib/minitest/BUILD:1-6`.

   [R] The mini test framework returns `0` when no checks failed and `1` otherwise. Evidence: `project/cpp3rdlib/minitest/minitest.cc:12-33`.

   [X] Passing CPU-only `cc_test` through Blade succeeded:

   ```text
   COMMAND[blade-test-host-pass]: blade_cmd test --full-test --show-details //:host_test
   RC[blade-test-host-pass]: 0
   (23:17:07)Blade(info): xml report:xml:.../project/build64_release/report_host_test.xml
   [minitest] PASS host_arithmetic expected=7 actual=7
   [minitest] PASS host_boolean
   [minitest] failures=0
   (23:17:08)Blade(info): Run 1 tests
   (23:17:08)Blade(info): All tests passed!
   ```

   [X] Failing CPU-only `cc_test` through Blade failed with shell rc `250`:

   ```text
   COMMAND[blade-test-host-fail]: blade_cmd test --full-test --show-details //:host_test_fail
   RC[blade-test-host-fail]: 250
   [minitest] FAIL host_arithmetic_forced_fail expected=8 actual=7
   [minitest] failures=1
   (23:17:10)Blade(error): 1 tests failed:
   (23:17:10)Blade(info): Run 1 tests
   ```

   [R] Blade constructs the test command as the absolute built executable path, sets `GTEST_OUTPUT` to `xml:{abs_build_dir}/report_{target.name}.xml`, and starts the test with `subprocess.Popen`. Evidence: `/opt/tiger/typhoon-blade/src/blade/functions/test_runner.py:566-574`, `/opt/tiger/typhoon-blade/src/blade/functions/test_scheduler.py:181-189`.

   [R] Any nonzero test return code appends the target to `failed_targets`; after scheduling, Blade returns internal `BladeErrno.ETEST.value`, and `ETEST` is `2042`. Evidence: `/opt/tiger/typhoon-blade/src/blade/functions/test_scheduler.py:258-268`, `/opt/tiger/typhoon-blade/src/blade/functions/test_runner.py:612-619`, `/opt/tiger/typhoon-blade/src/blade/common/blade_errno.py:138-139`.

   [X] The local mini framework does not write a gtest XML file even though Blade sets `GTEST_OUTPUT`:

   ```sh
   find docs/efforts/multi-agent-bench/evidence/research-blade-cuda/runs/blade-cuda-Q6Wmyt/project \
     -maxdepth 3 -type f -name 'report_*.xml' -printf '%p\n'
   ```

   [X] Output excerpt: no output.

7. [X] CUDA `blade test` is blocked by device visibility, not by compilation or test launching

   [X] CUDA `cc_test` built and launched through `blade test`, then failed inside CUDA runtime:

   ```text
   COMMAND[blade-test-cuda-pass]: blade_cmd test --full-test --show-details //:cuda_test
   RC[blade-test-cuda-pass]: 250
   (23:17:12)Blade(info): xml report:xml:.../project/build64_release/report_cuda_test.xml
   CUDA_ERROR cudaGetDeviceCount OS call failed or operation not supported on this OS
   CUDA_ERROR cudaMalloc OS call failed or operation not supported on this OS
   [minitest] FAIL device_count_positive
   [minitest] FAIL kernel_add_one expected=42 actual=-1304
   [minitest] failures=2
   ```

   [R] The CUDA test checks `blade_cuda_device_count() > 0` and `blade_cuda_answer(41) == 42`. Evidence: `project/test.cc:6-12`.

   [I] If a supervisor runs the same script from a namespace with `/dev/nvidia*` and `/dev/dri/*` visible, the expected success line is from `project/main.cc:21-28`: `BLADE_CUDA_RESULT answer=42 devices=<n> device0=<name> sm=100`.

8. [X] Current GPU/device-node state in this worker

   [X] Device-node probe command:

   ```sh
   ROOT=$PWD
   S=$(mktemp -d -p "$ROOT/docs/efforts/multi-agent-bench/evidence/research-blade-cuda" dev-probe-XXXXXX)
   mkdir -p "$S/home" "$S/work"
   hostname
   ls -ld /dev/nvidia* /dev/dri /dev/nvidia-caps 2>&1 || true
   nvidia-smi -L 2>&1 || true
   find /proc/driver/nvidia/gpus -maxdepth 2 -name information -print -exec sed -n '1,8p' {} \; 2>&1 | sed -n '1,120p'
   awk '/nvidia/ {print}' /proc/devices
   bwrap --ro-bind / / --dev-bind /dev /dev --bind "$S" "$S" \
     --tmpfs /home --tmpfs /tmp --unshare-net --clearenv \
     --setenv HOME "$S/home" --setenv PATH /usr/bin:/bin --chdir "$S/work" \
     /bin/sh -c 'set -x; mknod nvidiactl c 195 255; ls -l nvidiactl' 2>&1 || true
   ```

   [X] Output excerpt:

   ```text
   n116-077-207
   ls: cannot access '/dev/nvidia*': No such file or directory
   ls: cannot access '/dev/dri': No such file or directory
   ls: cannot access '/dev/nvidia-caps': No such file or directory
   NVIDIA-SMI has failed because it couldn't communicate with the NVIDIA driver.
   Model:           NVIDIA B200
   ...
   195 nvidia
   195 nvidiactl
   241 nvidia-caps
   242 nvidia-caps-imex-channels
   510 nvidia-uvm
   + mknod nvidiactl c 195 255
   mknod: nvidiactl: Operation not permitted
   ```

   [R] `/run/cdi/nvidia.yaml` describes expected CDI device nodes for eight GPUs, including `/dev/nvidia0`, `/dev/dri/card0`, and `/dev/dri/renderD128` for GPU 0, continuing through `/dev/nvidia7` and render node `renderD135`. Evidence: `/run/cdi/nvidia.yaml:5-20`, `/run/cdi/nvidia.yaml:131-136`.

   [X] A search found no current NVIDIA or DRI device nodes under `/dev`, and only CDI/persistenced metadata under `/run`:

   ```text
   DEV_NVIDIA_SEARCH
   RUN_NVIDIA_SEARCH
   f /run/cdi/nvidia.yaml 644 nobody nogroup
   d /run/nvidia-persistenced 755 nobody nogroup
   f /run/nvidia-persistenced/nvidia-persistenced.pid 644 nobody nogroup
   ```

   [I] This worker cannot verify a real B200 kernel run without entering a namespace where the NVIDIA character devices already exist or without a privileged helper that creates/binds them.

9. [R] Offline behavior and remote services

   [X] The script always keeps `--unshare-net` and `--clearenv` in the `bwrap_base`. Evidence: `run_blade_cuda.sh:42-60`.

   [X] Even with the network namespace removed, Blade still attempted buildcloud/TCC/reporting HTTP calls and logged `Network is unreachable`; the build itself still completed locally. Evidence from `cold-build` log excerpt:

   ```text
   Blade(warning): Buildcloud request exception: HTTPConnectionPool(host='buildcloudv2.byted.org', port=80): ...
   Blade(warning): Read Tcc config from buildcloud failed: ... [Errno 101] Network is unreachable
   Blade(warning): can't find available distribution cache and execute service from remote
   Blade(error): Report data server error: ... [Errno 101] Network is unreachable
   Blade(info): success
   ```

   [R] Blade has command-line flags for remote service/executor/cache and `--disable-buildcloud`, plus build-event settings; I did not retest the final script with those flags because the prescribed safe recipe only required network isolation. Evidence: `/opt/tiger/typhoon-blade/src/blade/cmds/cmd.py:456-478`.

   [R] Sailfish/CAS has opt-out environment names including `DISABLE_CAS`, `CUSTOM_DISABLE_CAS`, `BLADE_SAILFISH_SKIP`, and `CUSTOM_SAILFISH_SKIP`. Evidence: `/opt/tiger/typhoon-blade/src/blade/core/build_environment.py:325-327`, `/opt/tiger/typhoon-blade/src/blade/functions/sailfish_manager.py:159-177`.

   [I] A cleaner future benchmark runner should add explicit remote/reporting-disable knobs after testing them under the same bwrap shape, because current logs prove the network is blocked but not that Blade avoids attempting those calls.

10. [X] Artifact hygiene

   [R] The reusable artifacts intended for review are the report and project/script under `docs/efforts/multi-agent-bench/evidence/research-blade-cuda/`.

   [X] The script creates per-run scratch directories under `docs/efforts/multi-agent-bench/evidence/research-blade-cuda/runs/` and logs each command with wall time. Evidence: `run_blade_cuda.sh:6-15`, `run_blade_cuda.sh:75-138`.

   [I] The run directories are diagnostic byproducts, not required source artifacts; the project and script are the durable recipe.

## Open questions

- [X] Real GPU execution is open in this worker namespace: the binary and CUDA test reached CUDA runtime, but `cudaGetDeviceCount` failed before any kernel could run.
- [I] The likely missing operational step is to run from the lead/supervisor namespace where `/dev/nvidia*` and `/dev/dri/*` are visible, or to have an authorized device-node injection step before entering bwrap.
- [R] The lead decision file says the host GPU path worked in a lead rerun with `/dev/nvidia*` bound and failed with `--dev /dev` only. Evidence: `docs/efforts/multi-agent-bench/decisions/20260929-research-workers-verified-and-corrected.md:38-42`.
- [I] I did not determine whether `blade run` should be preferred over direct execution once device nodes are visible; Blade source shows `blade run` spawns the binary with prepared env, but direct execution under the same bwrap was enough to test loader behavior. Evidence: `/opt/tiger/typhoon-blade/src/blade/functions/binary_runner.py:34-49`.
- [I] I did not verify a workspace-local `NVCC` wrapper or `CUSTOM_NVCC` wrapper separately; source supports both command-prefix paths, and the direct `NVCC=$CUDA_MOUNT/bin/nvcc` path was sufficient to build.
- [I] I did not verify `cu_binary`; the project uses `cu_library` plus `cc_binary`/`cc_test`, which is the more relevant shape for mixed C++/CUDA benchmark tasks.

## Things I got wrong or could not check

- [X] I accidentally ran `/opt/tiger/ss_bin/blade build --help` outside the sandbox earlier in the investigation. That violated the task rule. I interrupted it after it began the Blade bootstrap/help path; I did not use it as evidence and did not repeat it.
- [X] An early bwrap path-visibility probe used a relative scratch bind and failed as `bwrap: Can't mkdir parents for docs/efforts/...: Read-only file system`; the corrected probe used an absolute `$PWD/...` scratch path and is the one cited above.
- [X] Early script iterations failed after successful build because the excerpt filter returned no matches under `set -o pipefail`; `run_blade_cuda.sh:68-72` now tolerates empty excerpts.
- [X] I could not check a successful B200 kernel launch because this worker's current `/dev` has no NVIDIA or DRI device nodes and unprivileged `mknod` fails.
- [X] I could not check real gtest XML output with the vendored mini framework; Blade set `GTEST_OUTPUT`, but the mini framework does not implement XML writing, and `find ... -name 'report_*.xml'` returned no files.

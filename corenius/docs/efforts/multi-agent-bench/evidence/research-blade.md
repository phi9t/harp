1. [X] Use only `docs/efforts/multi-agent-bench/evidence/research-blade/run_blade_build.sh <project> <target>` for Blade probes; it wraps `blade build` in `bwrap --unshare-net --clearenv`, sets `HOME`, `PATH`, and `USER`, and pre-seeds Blade pip-check markers. Evidence: `run_blade_build.sh:1-28`.
2. [X] This Blade is `blade-1.2.2.1469` on Python `2.7.13`; every safe build log records that version, for example `logs/build-l0-flat.log:20`.
3. [R] BUILD files execute as Python over `Target.registered`, and `load()` creates an isolated namespace with `native`, `cur_source_path`, `build_path`, and `trigger_repo`. Evidence: `/opt/tiger/typhoon-blade/src/blade/core/load_build_files.py:215-218`, `/opt/tiger/typhoon-blade/src/blade/targets/target.py:918-999`.
4. [X] `native` inside a loaded macro file is a dict, not an object; `native.cc_library(...)` failed and `native["cc_library"](...)` built. Evidence: `logs/build-l2-macro.log:1-21`, `projects/l2-macro/BUILD.share:1-15`, `logs/build-l2-macro-fixed.log:1-69`.
5. [R] Core supported names include `cc_library`, `cc_binary`, `cc_test`, `cc_plugin`, `cu_library`, `cu_binary`, `cu_test`, `proto_library`, `thrift_library`, `gen_rule`, `py_library`, `py_binary`, `py_test`, and `py_egg`; `filegroup` and Bazel `genrule` are not registered. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/*`, `logs/probe-missing-filegroup.log:1-19`, `logs/probe-missing-genrule.log:1-19`.
6. [X] Unknown rule options are rejected during target construction as `//<pkg>:<target>: unrecognized options {...}`; verified for C++, CUDA, proto, thrift, gen_rule, and Python. Evidence: `logs/probe-options-cc-library-bad.log:1-35`, `logs/probe-options-cu-library-bad.log:1-35`, `logs/probe-options-proto-library-bad.log:1-35`, `logs/probe-options-thrift-library-bad.log:1-35`, `logs/probe-options-gen-rule-bad.log:1-35`, `logs/probe-options-py-library-bad.log:1-35`.
7. [X] Versioned deps like `cpp3rdlib/math1:1.0@//cpp3rdlib/math1:math1` resolve from local packages when those packages exist; absent local packages fail before compilation. Evidence: `logs/build-l1-vendored.log:1-69`, `logs/probe-failure-missing-versioned-dep.log:1-34`, `logs/probe-failure-missing-versioned-dep-fixed.log:1-69`.
8. [X] `library_settler(prefer_dynamic=...)` can force a local prebuilt path that fails before compile; the verified one-line fix was `prefer_static` for the same package. Evidence: `logs/probe-settler-dynamic-bad.log:1-67`, `logs/probe-settler-dynamic-fixed.log:1-67`.
9. [X] Minimal CUDA did not build in the prescribed cleared recipe because generated Ninja invoked bare `nvcc`; the host `/usr/local/cuda/bin/nvcc` is CUDA 13.2 and accepted `sm_100` in a non-Blade compile, while `nvidia-smi -L` failed here. Evidence: `logs/probe-cuda-min.log:47-64`, `logs/host-cuda-inspection.log:1-11`, `logs/nvcc-sm100.log:1-3`.
10. [X] A 20-target cold build took `WALL_SECONDS 1.144`, produced 43 Ninja steps and `532K`; a warm build took `WALL_SECONDS 1.112`, still relinked 2 steps, and reported `512K`. Evidence: `logs/cost20-cold.log:47-107`, `logs/cost20-cold-du.log:1`, `logs/cost20-warm.log:46-65`, `logs/cost20-warm-du.log:1`.

1. [X] Reproduction harness and artifact layout

   [X] The safe runner is `docs/efforts/multi-agent-bench/evidence/research-blade/run_blade_build.sh`; the exact executed command shape is:

   ```sh
   bwrap --ro-bind / / --dev-bind /dev /dev --bind "$S" "$S" --tmpfs /home --tmpfs /tmp --unshare-net --clearenv \
     --setenv HOME "$S/home" --setenv PATH /opt/tiger/ss_bin:/usr/local/bin:/usr/bin:/bin --setenv USER bench \
     --chdir "$project" /opt/tiger/ss_bin/blade build "$target"
   ```

   [X] Evidence: `run_blade_build.sh:17-28`; representative invocation `COMMAND: docs/efforts/multi-agent-bench/evidence/research-blade/run_blade_build.sh l0-flat //:hello` succeeded with `EXIT: 0` in `logs/build-l0-flat.log:1-67`.

   [X] The runner creates `$S/home/.blade/blade_status/pip_checking.<executor-size>` and `pip3_checking.<executor-size>` before calling Blade. Evidence: `run_blade_build.sh:12-15`.

   [X] The pre-seed is necessary offline in this environment because an earlier run without it tried a bytedpypi bootstrap and failed before useful Blade probing. Evidence: `logs/bootstrap-pip-attempt.log:1` records the attempted l0 build, and the current runner records the marker creation in `run_blade_build.sh:12-15`.

   [X] The synthetic projects and runnable sketches live under `docs/efforts/multi-agent-bench/evidence/research-blade/projects/`; verified builds include `l0-flat`, `l1-vendored`, `l2-macro`, `l3-settler`, `cc-test-min`, `settler-dynamic-fixed`, and `cost20`. Evidence: `logs/build-l0-flat.log:1-67`, `logs/build-l1-vendored.log:1-69`, `logs/build-l2-macro-fixed.log:1-69`, `logs/build-l3-settler.log:1-69`, `logs/probe-cc-test-min.log:1-67`, `logs/probe-settler-dynamic-fixed.log:1-67`, `logs/cost20-cold.log:1-107`.

2. [R] Rule inventory and option surface

   [R] `Target.register_function` records BUILD-callable functions in `Target.registered`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/target.py:23-35`.

   [R] The registered non-rule helpers are `enable_if`, `glob`, `include`, and `load`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/target.py:1006-1009`.

   [R] The registered C/C++ rules I found are `cc_library`, `cc_binary`, `cc_test`, `cc_plugin`, `cc_benchmark`, and `cc_base_library`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cc/target_cc_library.py:775`, `/opt/tiger/typhoon-blade/src/blade/targets/cc/target_cc_binary.py:330`, `/opt/tiger/typhoon-blade/src/blade/targets/cc/target_cc_test.py:172`, `/opt/tiger/typhoon-blade/src/blade/targets/cc/target_cc_plugin.py:181`, `/opt/tiger/typhoon-blade/src/blade/targets/cc/target_cc_benchmark.py:18-32`, `/opt/tiger/typhoon-blade/src/blade/targets/cc/target_cc_base_library.py:15-25`.

   [R] The registered CUDA rules are `cu_library`, `cu_binary`, and `cu_test`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cuda/target_cu_library.py:110-173`, `/opt/tiger/typhoon-blade/src/blade/targets/cuda/target_cu_binary.py:161-214`, `/opt/tiger/typhoon-blade/src/blade/targets/cuda/target_cu_test.py:83-143`.

   [R] The registered protobuf rules are `proto_library`, `new_proto_library`, `split_source_proto_library`, and `gen_proto`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/protobuf/target_proto_library.py:549-612`, `/opt/tiger/typhoon-blade/src/blade/targets/protobuf/target_new_proto_library.py:445-460`, `/opt/tiger/typhoon-blade/src/blade/targets/protobuf/target_split_source_proto_library.py:559-575`, `/opt/tiger/typhoon-blade/src/blade/targets/protobuf/target_gen_proto.py:227-240`.

   [R] The registered thrift-family rules are `thrift_library`, `fbthrift_library`, `fbthrift1_library`, `fbthrift2_library`, `drpc_thrift_library`, `drpc_fbthrift1_library`, and `drpc_fbthrift2_library`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/thrift/target_thrift_library.py:341-384`, `/opt/tiger/typhoon-blade/src/blade/targets/thrift/target_fbthrift_library.py:129-140`, `/opt/tiger/typhoon-blade/src/blade/targets/thrift/target_fbthrift1_library.py:233-244`, `/opt/tiger/typhoon-blade/src/blade/targets/thrift/target_fbthrift2_library.py:231-242`, `rg 'Target.register_function' /opt/tiger/typhoon-blade/src/blade/targets`.

   [R] Other registered specialist rules are `package`, `virtual_library`, `resource_library`, `flat_buffers_library`, `lex_yacc_library`, `sh_test`, `swig_library`, `idl_cut`, `archon_library`, `archon_cut`, `monarch_library`, and `recomate_op`. Evidence: `rg 'Target.register_function' /opt/tiger/typhoon-blade/src/blade/targets`, `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_package.py:164-172`, `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_virtual_library.py:107-112`, `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_resource_library.py:98-106`, `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_flat_buffers_library.py:110-120`, `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_lex_yacc_library.py:150-158`, `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_sh_test.py:114-122`, `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_swig_library.py:212-222`, `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_idl_cut.py:200-210`.

   [R] `cc_library` accepts `name`, `srcs`, `exclude_srcs`, `deps`, `custom_deps`, `visibility`, `warning`, `defs`, `incs`, `export_incs`, `optimize`, `always_optimize`, `pre_build`, `prebuilt`, `prebuilt_libpath_pattern`, `onlyinc`, `link_all_symbols`, `deprecated`, `extra_cppflags`, `extra_linkflags`, `allow_undefined`, `secure`, `build_dynamic`, `bundle_path`, `bc_archive`, `bc_archive_expr`, `isolate_build`, `prefer_dynamic`, and `**kwargs`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cc/target_cc_library.py:695-723`.

   [R] `cc_binary` accepts `name`, `srcs`, `exclude_srcs`, `generate_config`, `deps`, `custom_deps`, `warning`, `defs`, `incs`, `embed_version`, `optimize`, `static_compile`, `dynamic_link`, `extra_cppflags`, `extra_linkflags`, `export_dynamic`, `bundle_path`, and `**kwargs`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cc/target_cc_binary.py:275-293`.

   [R] `cc_test` accepts `name`, `srcs`, `exclude_srcs`, `generate_config`, `deps`, `custom_deps`, `warning`, `defs`, `incs`, `embed_version`, `optimize`, `static_compile`, `dynamic_link`, `testdata`, `extra_cppflags`, `extra_linkflags`, `export_dynamic`, `always_run`, `exclusive`, `heap_check`, `heap_check_debug`, `gtest_libs`, `gtest_main_libs`, `bundle_path`, `bytecppmock`, and `**kwargs`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cc/target_cc_test.py:97-124`.

   [R] `cc_plugin` accepts `name`, `srcs`, `exclude_srcs`, `generate_config`, `deps`, `custom_deps`, `warning`, `defs`, `incs`, `embed_version`, `optimize`, `prefix`, `suffix`, `extra_cppflags`, `extra_linkflags`, `allow_undefined`, `bundle_path`, and `**kwargs`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cc/target_cc_plugin.py:125-143`.

   [R] `cu_library` accepts `name`, `srcs`, `exclude_srcs`, `deps`, `custom_deps`, `warning`, `defs`, `incs`, `export_incs`, `optimize`, `always_optimize`, `prebuilt`, `onlyinc`, `link_all_symbols`, `deprecated`, `extra_flags`, `extra_cppflags`, `extra_linkflags`, `extra_cuflags`, `build_dynamic`, `bundle_path`, and `**kwargs`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cuda/target_cu_library.py:110-131`.

   [R] `cu_binary` accepts `name`, `srcs`, `exclude_srcs`, `deps`, `custom_deps`, `warning`, `defs`, `incs`, `export_incs`, `optimize`, `static_compile`, `extra_flags`, `extra_cppflags`, `extra_linkflags`, `extra_cuflags`, `bundle_path`, and `**kwargs`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cuda/target_cu_binary.py:161-177`.

   [R] `cu_test` accepts `name`, `srcs`, `exclude_srcs`, `deps`, `custom_deps`, `warning`, `defs`, `incs`, `export_incs`, `optimize`, `static_compile`, `extra_flags`, `extra_cppflags`, `extra_linkflags`, `extra_cuflags`, `testdata`, `always_run`, `exclusive`, `bundle_path`, and `**kwargs`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cuda/target_cu_test.py:83-102`.

   [R] `proto_library` accepts `name`, `srcs`, `exclude_srcs`, `deps`, `custom_deps`, `incs`, `export_incs`, `output_dir`, `optimize`, `deprecated`, `generate_descriptors`, `plugins`, `source_encoding`, `allow_undefined`, `grpc_enabled`, `build_dynamic`, `relative_proto_path`, `generate_language`, `prefer_dynamic`, and `**kwargs`; it consumes `protoc_params` from `kwargs` as a supported extra. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/protobuf/target_proto_library.py:52-58`, `/opt/tiger/typhoon-blade/src/blade/targets/protobuf/target_proto_library.py:549-574`.

   [R] `thrift_library` accepts `name`, `srcs`, `exclude_srcs`, `deps`, `custom_deps`, `incs`, `export_incs`, `optimize`, `deprecated`, `thrift_bin`, and `**kwargs`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/thrift/target_thrift_library.py:341-351`.

   [R] `gen_rule` accepts `name`, `srcs`, `deps`, `outs`, `cmd`, `outdir`, `export_outs`, and `**kwargs`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_gen_rule.py:152-159`.

   [R] `py_library`, `py_binary`, `py_test`, and `py_egg` accept small Python-specific surfaces: `py_library(name, srcs, deps, base, prebuilt, visibility, **kwargs)`, `py_binary(name, srcs, deps, main, base, **kwargs)`, `py_test(name, srcs, deps, main, base, testdata, **kwargs)`, and `py_egg(name, srcs, deps, prebuilt, **kwargs)`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/python/target_py_library.py:106-112`, `/opt/tiger/typhoon-blade/src/blade/targets/python/target_py_binary.py:82-87`, `/opt/tiger/typhoon-blade/src/blade/targets/python/target_py_test.py:39-45`, `/opt/tiger/typhoon-blade/src/blade/targets/python/target_py_egg.py:45-49`.

   [R] Unknown options are allowed syntactically by `**kwargs` but rejected by `Target._check_kwargs` if anything remains. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/target.py:99-101`.

   [X] Verified wrong-option failures: `cc_library(..., hdrs=...)` -> `//.:bad: unrecognized options {'hdrs': ['bad.h']}`; `cc_binary(..., made_up=True)` -> `//.:bad: unrecognized options {'made_up': True}`; `cu_library(..., hdrs=...)` -> the same `hdrs` rejection; `proto_library`, `thrift_library`, `gen_rule`, and `py_library` reject `hdrs` similarly. Evidence: `logs/probe-options-cc-library-bad.log:1-35`, `logs/probe-options-cc-binary-bad.log:1-35`, `logs/probe-options-cu-library-bad.log:1-35`, `logs/probe-options-proto-library-bad.log:1-35`, `logs/probe-options-thrift-library-bad.log:1-35`, `logs/probe-options-gen-rule-bad.log:1-35`, `logs/probe-options-py-library-bad.log:1-35`.

3. [R] BUILD execution model and macro behavior

   [R] A BUILD file is evaluated with `exec_build_file(build_file, Target.registered, None)`, so BUILD-level globals are the registered Blade functions and any names imported by previous `load()` calls. Evidence: `/opt/tiger/typhoon-blade/src/blade/core/load_build_files.py:213-218`.

   [R] `include(name)` executes another file directly into `Target.registered`, resolving `//` from the root and relative names from the current source path. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/target.py:908-914`.

   [R] `load(file_path, *import_symbols, **aliases_symbols)` requires explicit import symbols, resolves `//` paths from the root, errors when the file is absent, and errors when no symbols are requested. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/target.py:918-932`.

   [R] `load()` executes the target file in a separate `load_space` populated with `native`, `cur_source_path`, `build_path`, and `trigger_repo`, then filters private names and module objects before copying requested symbols into BUILD globals. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/target.py:940-999`.

   [X] A no-symbol `load("defs.bld")` failed with `The symbols loaded must explicitly declared, <path>/defs.bld`. Evidence: `logs/probe-load-no-symbols.log:1-35`.

   [X] A missing-symbol `load("defs.bld", "not_here")` failed with `import a not loaded symbol[not_here], <path>/defs.bld`. Evidence: `logs/probe-load-missing-symbol.log:1-35`.

   [X] Top-level BUILD globals in a blank project did not include lgtorch-style helper names such as `override_prefer_dep`; the probe failed first with `NameError: name 'override_prefer_dep' is not defined`. Evidence: `logs/probe-missing-globals.log:1-45`.

   [X] The verified 15-line macro is:

   ```python
   import os
   def mini_app(name):
       user = os.environ.get("USER", "unknown")
       value = "BENCH_USER_%s" % user.upper()
       native["cc_library"](
           name=name + "_lib",
           srcs=[name + ".cc"],
           defs=[value],
           export_incs=["."],
       )
       native["cc_binary"](
           name=name,
           srcs=[name + "_main.cc"],
           deps=[":" + name + "_lib"],
       )
   ```

   [X] This macro built only after using `native["cc_library"]` and `native["cc_binary"]`; the object-form attempt failed with `AttributeError: 'dict' object has no attribute 'cc_library'`. Evidence: `projects/l2-macro/BUILD.share:1-15`, `logs/build-l2-macro.log:1-47`, `logs/build-l2-macro-fixed.log:1-69`.

4. [R] Dependency labels, settlers, and version resolution

   [R] Dep normalization requires deps to start with `:` or `//` after Blade resolves a label; source code emits `dep format error` for malformed labels. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cc_deps.py:120-132`.

   [R] A versioned external dep has the shape `<repo>:<branch>[:<commit>]@<blade-label>`: Blade splits on `@`, parses the right side as one or more labels, then parses the left side into repo, branch, and optional commit. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cc_deps.py:222-267`, `/opt/tiger/typhoon-blade/src/blade/targets/cc_deps.py:838-844`, `/opt/tiger/typhoon-blade/src/blade/targets/cc_deps.py:1053-1073`.

   [X] A direct malformed dep string `not_a_label` failed as `.:bad_dep: Invalid not_a_label.`. Evidence: `logs/probe-failure-bad-dep.log:1-35`.

   [R] `custom_deps` are keyed by the active toolchain, optionally overridden by `--referred-toolchain`, and if present are downloaded or initialized before normal dep combination. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cc_target.py:110-128`.

   [R] `library_settler(prefer_dynamic=..., prefer_static=...)` runs only in the command target package, always takes effect for normal builds, appends basename-normalized `cpp3rdlib/<pkg>` entries to `BUILD_DYNAMIC` or `BUILD_STATIC`, and records non-`cpp3rdlib` repo preference lists on the Blade object. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_common_functions.py:21-88`.

   [R] `global_settler(prefer_deps=...)` runs only in the command target package and returns without effect unless `--update-deps`, `--keep-latest-*`, or `--update-some-deps*` style options are active. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_common_functions.py:91-104`.

   [R] When active, `global_settler` writes `GLOBAL_SETTLER` entries with `:` changed to `+`, and `DepsBase.load_global_setting` later maps those back to repo branch or commit overrides before branch calibration. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_common_functions.py:106-119`, `/opt/tiger/typhoon-blade/src/blade/targets/cc_deps.py:136-155`, `/opt/tiger/typhoon-blade/src/blade/targets/cc_deps.py:852-854`.

   [R] `override_prefer_dep` is not a Blade builtin; lgtorch defines it as a helper that mutates a `prefer_deps` list by replacing or appending `package + ":" + version`. Evidence: `logs/probe-missing-globals.log:1-45`, `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/lgtorch show 7ec5497:bytedance/lagrange_torch/csrc/BUILD.share`, lines 229-239.

   [I] For normal offline benchmark builds, `library_settler` is the reliable settler lever; `global_settler` is better for tasks that explicitly exercise dependency update modes because it is intentionally inert on plain builds. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/misc/target_common_functions.py:99-104`, `logs/probe-settler-dynamic-bad.log:1-67`.

   [X] Verified settler failure: `settler-dynamic-bad` provides a valid `libdual.a` and an unusable `libdual.so`, then calls `library_settler(prefer_dynamic=["cpp3rdlib/dual"])`; Blade failed with `soname can't find in dual`. Evidence: `projects/settler-dynamic-bad/BUILD:1-7`, `logs/probe-settler-dynamic-bad.log:1-67`.

   [X] Verified one-line fix: changing the command package to `library_settler(prefer_static=["cpp3rdlib/dual"])` made the same target build. Evidence: `projects/settler-dynamic-fixed/BUILD:1-7`, `logs/probe-settler-dynamic-fixed.log:1-67`.

   [X] Verified local versioned dep success: `l1-vendored` builds `//app:use_math1` with `cpp3rdlib/math1:1.0@//cpp3rdlib/math1:math1`. Evidence: `projects/l1-vendored/app/BUILD:1-5`, `logs/build-l1-vendored.log:1-69`.

   [X] Verified absent package failure: `cpp3rdlib/absent:1.0@//cpp3rdlib/absent:absent` failed as `//cpp3rdlib/absent not found when loading BUILD file, required by //.:missing_dep`. Evidence: `projects/failure-missing-versioned-dep/BUILD:1-5`, `logs/probe-failure-missing-versioned-dep.log:1-34`.

   [X] Verified local package fix: adding a local `cpp3rdlib/absent/BUILD` with matching target made `//:fixed_dep` build. Evidence: `projects/failure-missing-versioned-dep-fixed/BUILD:1-5`, `projects/failure-missing-versioned-dep-fixed/cpp3rdlib/absent/BUILD:1-5`, `logs/probe-failure-missing-versioned-dep-fixed.log:1-69`.

   [X] Three additional verified benchmark failure modes are malformed dep label (`logs/probe-failure-bad-dep.log:1-35`), unsupported Bazel `filegroup` (`logs/probe-missing-filegroup.log:1-45`), and missing loaded symbol (`logs/probe-load-missing-symbol.log:1-35`).

   [X] Two tempting source-path traps were not reliable hard failures here: duplicate `srcs=["dup.cc", "dup.cc"]` built successfully, and `srcs=["../escape.cc"]` logged an invalid source path but exited `0`. Evidence: `logs/probe-failure-duplicate-source.log`, `logs/probe-failure-invalid-source-path.log:1-66`.

5. [R] CUDA and nvcc

   [R] Blade's generated CUDA rule uses `env.get('NVCC', 'nvcc')`, so with the required `--clearenv` and the allowed `PATH` it tries bare `nvcc` unless the sandbox command injects `NVCC` or extends `PATH`. Evidence: `/opt/tiger/typhoon-blade/src/blade/core/rules_generator.py:851-889`, `run_blade_build.sh:17-28`.

   [R] Blade discovers CUDA include paths from `CUDA_PATH` or from `/usr/local/cuda_<nvcc-version>`, and obtains `<nvcc-version>` by running `$NVCC --version`. Evidence: `/opt/tiger/typhoon-blade/src/blade/compilers/compiler_cc.py:390-408`.

   [R] CUDA target flags come from BUILD `extra_cuflags`, legacy `extra_flags`, `extra_cppflags`, optimization settings, and command-line `--cuflags` / env `CUFLAGS` or `CUSTOM_CUFLAGS`. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cuda/cu_target.py:60-66`, `/opt/tiger/typhoon-blade/src/blade/targets/cuda/cu_target.py:97-132`, `/opt/tiger/typhoon-blade/src/blade/targets/cuda/cu_target.py:142-153`, `/opt/tiger/typhoon-blade/src/blade/cmds/cmd_build.py:108-112`.

   [X] The minimal `cu_library` probe generated `nvcc -MMD ... -gencode=arch=compute_70,code=sm_70 ...` and failed `/bin/sh: 1: nvcc: not found`. Evidence: `logs/probe-cuda-min.log:1-64`.

   [R] lgtorch's own CUDA macro layer defaults to A100/Ada/H100 flags and adds Blackwell/B200 `sm_100` only for CUDA versions `12.8`, `13.0`, or `13.1`, not `13.2`. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/lgtorch show 7ec5497:bytedance/lagrange_torch/csrc/BUILD.cuda`, lines 40-60.

   [R] erdos has BUILD files that read `COMPUTE_CAP` and synthesize `-gencode arch=compute_<capability>,code=sm_<capability>` for CUDA tests. Evidence: `logs/erdos-pattern-grep.log:1328`, `logs/erdos-pattern-grep.log:3260-3262`, `logs/erdos-pattern-grep.log:3284-3286`.

   [X] Host `/usr/local/cuda/bin/nvcc --version` reported CUDA 13.2, `V13.2.51`, and the standalone host command `TMPDIR=$base/nvcc-sm100-tmp /usr/local/cuda/bin/nvcc -x cu -c - -o $base/nvcc-sm100-tmp/sm100.o -gencode=arch=compute_100,code=sm_100` exited `0`. Evidence: `logs/host-cuda-inspection.log:1-7`, `logs/nvcc-sm100.log:1-3`.

   [X] `nvidia-smi -L` did not work in this session; it exited `9` with `NVIDIA-SMI has failed because it couldn't communicate with the NVIDIA driver`. Evidence: `logs/host-cuda-inspection.log:8-11`.

   [I] To make a Blade CUDA benchmark build in the exact safe recipe, bind visibility alone is already provided by `--ro-bind / /`, but the command also needs `NVCC=/usr/local/cuda/bin/nvcc` or a `PATH` containing `/usr/local/cuda/bin` and likely `CUDA_PATH=/usr/local/cuda`; this was not run because the recipe in the task forbade adding extra env vars to the Blade command. Evidence: `run_blade_build.sh:17-28`, `/opt/tiger/typhoon-blade/src/blade/core/rules_generator.py:851-889`, `/opt/tiger/typhoon-blade/src/blade/compilers/compiler_cc.py:390-408`.

6. [R] proto_library and thrift_library offline

   [R] Default Blade config points `protoc` to `cpp3rdlib/protobuf/bin/protoc` and `thrift` to `cpp3rdlib/thrift/bin/thrift`; those are project-local paths, not bundled absolute compiler paths. Evidence: `/opt/tiger/typhoon-blade/src/blade/configs/BLADE_ROOT:4-10`.

   [R] `proto_library` hardcodes protobuf C++ and Python runtime deps from `proto_library_config` before generating rules. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/protobuf/target_proto_library.py:95-105`.

   [R] `proto_library` accepts `protoc_params` through `kwargs`, checks proto deps, and feeds extra params into Ninja's protoc variables. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/protobuf/target_proto_library.py:52-58`, `/opt/tiger/typhoon-blade/src/blade/targets/protobuf/target_proto_library.py:138-145`, `/opt/tiger/typhoon-blade/src/blade/targets/protobuf/target_proto_library.py:470-484`.

   [X] A minimal `proto_library` with no local `cpp3rdlib/protobuf` failed before `protoc` execution as `//cpp3rdlib/protobuf not found when loading BUILD file, required by //.:thing_proto`. Evidence: `logs/probe-proto-min.log:1-34`.

   [R] `thrift_library` hardcodes `thrift_config.thrift_libs` as deps and generates a Ninja `thrift` action per `.thrift` source. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/thrift/target_thrift_library.py:188-196`, `/opt/tiger/typhoon-blade/src/blade/targets/thrift/target_thrift_library.py:298-306`.

   [X] A minimal `thrift_library` reached Ninja and failed on the compiler path `cpp3rdlib/thrift/bin/thrift --gen cpp:include_prefix ...` with `/bin/sh: 1: cpp3rdlib/thrift/bin/thrift: not found`. Evidence: `logs/probe-thrift-min.log:47-65`.

   [I] A benchmark that expects proto/thrift success offline must vendor at least a local `cpp3rdlib/protobuf` package containing `bin/protoc`, headers, and a `protobuf` target, and a local `cpp3rdlib/thrift` package containing `bin/thrift` plus the thrift runtime targets named by `thrift_config`. Evidence: `/opt/tiger/typhoon-blade/src/blade/configs/BLADE_ROOT:4-10`, `logs/probe-proto-min.log:1-34`, `logs/probe-thrift-min.log:47-65`.

7. [X] Tests and built binary execution

   [R] `cc_test` defaults `gtest_libs` and `gtest_main_libs` from `cc_test_config` when explicit lists are not provided. Evidence: `/opt/tiger/typhoon-blade/src/blade/targets/cc/target_cc_test.py:52-81`.

   [R] Default `cc_test_config` names `cpp3rdlib/gtest:gtest` and `cpp3rdlib/gtest:gtest_main`. Evidence: `/opt/tiger/typhoon-blade/src/blade/configs/blade.conf:59-65`.

   [X] A minimal `cc_test` built offline when it passed `gtest_libs=[]` and `gtest_main_libs=[]`. Evidence: `projects/cc-test-min/BUILD`, `logs/probe-cc-test-min.log:1-67`.

   [X] Built C++ binaries landed in `build64_release`, for example `projects/l0-flat/build64_release/hello` and `projects/cc-test-min/build64_release/tiny_test`. Evidence: `logs/build-l0-flat.log:62-67`, `logs/probe-cc-test-min.log:62-67`, `readelf-l0-hello.log:1`, `readelf-cc-test-tiny.log:1`.

   [X] The ELF interpreter for both inspected binaries was `.bundle/ld-linux-x86-64.so.2`. Evidence: `logs/readelf-l0-hello.log:1`, `logs/readelf-cc-test-tiny.log:1`.

   [X] Running either binary in a no-network bwrap without that relative loader failed as `bwrap: execvp <binary>: No such file or directory`. Evidence: `logs/run-l0-binary-no-loader.log:1-3`, `logs/run-cc-test-binary-no-loader.log:1-3`.

   [X] Binding host `/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2` to the project-local `.bundle/ld-linux-x86-64.so.2` made `hello` and `tiny_test` execute with `EXIT: 0`. Evidence: `logs/run-l0-binary-bind-relative-loader.log:1-2`, `logs/run-cc-test-binary-bind-relative-loader.log:1-2`.

   [I] For benchmark verification, the smallest demonstrated runtime change is a read-only bind of the host loader to the project-local `.bundle/ld-linux-x86-64.so.2`; I did not use `blade test`, `blade run`, or `patchelf` because the task's safe Blade recipe permitted only `blade build`. Evidence: `logs/run-l0-binary-bind-relative-loader.log:1-2`, `logs/run-cc-test-binary-bind-relative-loader.log:1-2`.

8. [R] Remote cache, remote execution, and reporting controls

   [R] Command-line options exist for `--remote-service`, `--remote-executor`, `--remote-cache`, and `--disable-buildcloud`; their defaults read `CUSTOM_REMOTE_SERVICE`, `CUSTOM_REMOTE_EXECUTOR`, and `CUSTOM_REMOTE_CACHE`. Evidence: `/opt/tiger/typhoon-blade/src/blade/cmds/cmd.py:456-469`.

   [R] Build-event transport options are `--bes-backend`, defaulting to `CUSTOM_BLADE_BES_BACKEND` or a built-in default, and `--build_event_json_file`, defaulting to `build_event.json`. Evidence: `/opt/tiger/typhoon-blade/src/blade/cmds/cmd.py:471-478`.

   [R] The buildboard client creates a gRPC transport only when `bes_backend` is nonempty and a JSON transport only when `build_event_json_file` is nonempty. Evidence: `/opt/tiger/typhoon-blade/src/blade/buildboard/buildboard_client.py:1694-1700`.

   [R] Sailfish/CAS paths are disabled when `DISABLE_CAS=1`, `CUSTOM_DISABLE_CAS=1`, `BLADE_SAILFISH_SKIP=1`, or `CUSTOM_SAILFISH_SKIP=1`. Evidence: `/opt/tiger/typhoon-blade/src/blade/core/build_environment.py:325-327`, `/opt/tiger/typhoon-blade/src/blade/functions/sailfish_manager.py:159-177`.

   [R] CUDA compile cache is enabled when `CUDA_CACHE` or `CUSTOM_CUDA_CACHE` is not `0`; it can wrap `nvcc` with `recc` or `ccache`. Evidence: `/opt/tiger/typhoon-blade/src/blade/core/rules_generator.py:851-867`.

   [X] The safe sandbox blocked network, but Blade still attempted buildcloud/TCC/build_info/reporting calls; for example, an option-failure probe logged buildcloud connection failures and a report-data server error. Evidence: `logs/probe-options-cc-library-bad.log:1-35`.

   [I] A stricter offline harness should invoke Blade with empty remote/cache/BES settings plus `--disable-buildcloud`, and with CAS/Sailfish env disabled before `--clearenv` or injected after it; I did not modify the prescribed safe recipe to test those flags. Evidence: `/opt/tiger/typhoon-blade/src/blade/cmds/cmd.py:456-478`, `/opt/tiger/typhoon-blade/src/blade/buildboard/buildboard_client.py:1694-1700`, `/opt/tiger/typhoon-blade/src/blade/core/build_environment.py:325-327`.

9. [X] Cost profile for a 20-target synthetic project

   [X] The measured cold command was `TIMEFORMAT='WALL_SECONDS %R'; time docs/efforts/multi-agent-bench/evidence/research-blade/run_blade_build.sh cost20 //:app19`. Evidence: `logs/cost20-cold.log:1`.

   [X] The cold build performed 43 Ninja steps, including 20 C++ compilations, static archives, and one link. Evidence: `logs/cost20-cold.log:47-89`.

   [X] The cold wall time was `WALL_SECONDS 1.144`, and `du -sh build64_release` reported `532K`. Evidence: `logs/cost20-cold.log:103-107`, `logs/cost20-cold-du.log:1`.

   [X] The measured warm command was the same build command against the existing tree. Evidence: `logs/cost20-warm.log:1`.

   [X] The warm build still rebuilt `version.cpp` and relinked `app19`; it took `WALL_SECONDS 1.112`, and `du -sh build64_release` reported `512K`. Evidence: `logs/cost20-warm.log:46-65`, `logs/cost20-warm-du.log:1`.

   [I] For benchmark design, a 20-target Blade task is cheap enough for repeated offline scoring, but warm builds are not fully no-op because `version.cpp` relinks. Evidence: `logs/cost20-cold.log:47-107`, `logs/cost20-warm.log:46-65`.

10. [R] lgtorch awkward patterns to model

   [R] L2 macro/package pattern: `bytedance/lagrange_torch/csrc/BUILD` loads three separate definition files before declaring targets. Evidence: `logs/lgtorch-pattern-grep.log:6-8`.

   [R] L3 env-driven settlers: the same top-level BUILD mutates `os.environ["DBG_LINE_ONLY"]`, computes settler config, calls `library_settler`, and calls `global_settler`. Evidence: `logs/lgtorch-pattern-grep.log:9-12`.

   [R] L5 composite plugin shell: `cc_plugin(name="lagrange_torch_blade")` appends MLU, ASCEND, CUDA, and coverage deps based on `os.getenv`, then sets `bundle_path="$ORIGIN/lib"`. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/lgtorch show 7ec5497:bytedance/lagrange_torch/csrc/BUILD`, lines 80-117.

   [R] L2 load-isolation pattern: `BUILD.cuda` documents that `load()` exists only in the top-level BUILD namespace and loaded files must be self-contained. Evidence: `logs/lgtorch-pattern-grep.log:34-42`.

   [R] L3 environment contract pattern: `BUILD.share` reads `ENABLE_AVX2`, `GCC`, `CUSTOM_TORCH_ENABLE_CPP_MF16C`, and `COMPUTE_PLATFORM`. Evidence: `logs/lgtorch-pattern-grep.log:80-84`.

   [R] L3 required sentinel file pattern: `BUILD.share` reads `erdos_commit` and raises when it is absent. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/lgtorch show 7ec5497:bytedance/lagrange_torch/csrc/BUILD.share`, lines 80-95.

   [R] L3 prefer-dep override pattern: `BUILD.share` exposes `get_torch_root_settler_config()` and `override_prefer_dep(deps, package, version)` for in-place version edits. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/lgtorch show 7ec5497:bytedance/lagrange_torch/csrc/BUILD.share`, lines 96-239.

   [R] L1 pinned 3rdlib pattern: lgtorch uses versioned labels such as `cpp3rdlib/gtest:1.10.0@//cpp3rdlib/gtest:gtest`, `cpp3rdlib/glog:0.4.0@//cpp3rdlib/glog:glog`, and `cpp3rdlib/protobuf:3.9.0@//cpp3rdlib/protobuf:protobuf`. Evidence: `logs/lgtorch-pattern-grep.log:2-5`, `logs/lgtorch-pattern-grep.log:28-31`, `logs/lgtorch-pattern-grep.log:125-128`.

   [R] L2 `link_all_symbols` pattern: many lgtorch libraries force whole-archive style linking. Evidence: `logs/lgtorch-pattern-grep.log:13-15`, `logs/lgtorch-pattern-grep.log:134-146`.

   [R] L4 CUDA local-prebuilt pattern: `BUILD.cuda` defines helper functions for local CUDA/NCCL prebuilt deps and materializes symlinks for `cc_library(prebuilt=True)` consumers. Evidence: `logs/lgtorch-pattern-grep.log:48-65`, `logs/lgtorch-pattern-grep.log:100-118`.

   [R] L4 CUDA arch-selection pattern: lgtorch chooses generated `extra_cuflags` based on `CUDA_VERSION`, but the checked code has explicit branches only through `13.1`. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/lgtorch show 7ec5497:bytedance/lagrange_torch/csrc/BUILD.cuda`, lines 40-60.

   [R] L5 platform composite pattern: lgtorch's CUDA packages load both `BUILD.share` and `BUILD.cuda`, then declare mixed `cc_library`, `cu_library`, tests, versioned deps, and `extra_cuflags`. Evidence: `logs/lgtorch-pattern-grep.log:139-173`.

   [I] Verifiable benchmark tasks from these patterns should ask agents to repair one isolated mistake at a time: wrong macro namespace, missing sentinel file, absent local 3rdlib, missing `sm_100` branch, broken prebuilt symlink, or wrong platform env branch. Evidence: the verified `l2-macro`, `failure-missing-versioned-dep`, `settler-dynamic-bad`, and `cuda-min` probes.

11. [R] erdos awkward patterns to model

   [R] L3 env-driven root config pattern: `BUILD.share` reads custom commit env vars, TensorFlow version, device, CUDA version, and build region. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/erdos show 9c075bd:BUILD.share`, lines 35-62.

   [R] L1 pinned CUDA/TF deps pattern: `BUILD.share` maps CUDA versions to long versioned `cpp3rdlib/cuda`, `cudnn`, `tensorrt`, and `cutlass` labels. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/erdos show 9c075bd:BUILD.share`, lines 132-154.

   [R] L3 toolchain branch pattern: `common/cpp/BUILD` chooses an `hdfs_cpp_client` version based on `GCC` and `ENABLE_BPT`. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/erdos show 9c075bd:common/cpp/BUILD`, lines 49-58.

   [R] L2 link semantics pattern: erdos sets `link_all_symbols=True` on common libraries such as `async_io` and `qpopen`. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/erdos show 9c075bd:common/cpp/BUILD`, lines 128-150.

   [R] L3 `custom_deps` pattern: a `cc_test` defines `common_custom_deps` keyed by `x86_64-gcc830` and passes it to the test target. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/erdos show 9c075bd:common/cpp/lagrange/unittest/BUILD`, lines 28-43.

   [R] L3 platform dep omission pattern: `core/tensor/BUILD` empties CUDA deps for `PANTHER_ENV=1` or `TF2_12_CAMBRICON`, otherwise chooses CUDA 12.2 or 11.2 packages. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/erdos show 9c075bd:core/tensor/BUILD`, lines 19-26.

   [R] L3 map-miss failure pattern: `inference/copilot/BUILD.share` maps repo keys by toolchain and raises `KeyError` when a requested repo key is unknown. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/erdos show 9c075bd:inference/copilot/BUILD.share`, lines 66-100.

   [R] L3 paired settlers pattern: copilot tests load a share file, then call both `library_settler(prefer_dynamic=...)` and `global_settler(prefer_deps=...)`. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/erdos show 9c075bd:inference/copilot/tests/BUILD`, lines 1-4.

   [R] L5 service-plugin pattern: pilot service applies `global_settler`, `library_settler(prefer_static=...)`, exports a dynamic binary with `bundle_path`, then declares a `cc_plugin`. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/erdos show 9c075bd:inference/incubate/pilot/src/service/BUILD`, lines 1-8 and 50-58.

   [R] L5 multi-share service pattern: marine service loads two share files, applies dynamic/static settlers, and applies global settlers before declaring service deps. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/erdos show 9c075bd:inference/marine/service/BUILD`, lines 1-10.

   [R] L4 IDL pattern: marine IDL uses `proto_library` with explicit protobuf and cross-repo IDL deps plus heavy optimize flags. Evidence: `git -C /data02/home/philip.yang/devx/tmp/claude-1018/-data02-home-philip-yang-workspace-milano/240d396c-ea8c-4f8b-95db-b5049321f46b/scratchpad/erdos show 9c075bd:inference/marine/idl/BUILD`, lines 1-10.

   [R] L4 CUDA test arch pattern: erdos BUILD files generate `cu_gencode_flags` from `COMPUTE_CAP` and append them to `extra_cuflags`. Evidence: `logs/erdos-pattern-grep.log:1328`, `logs/erdos-pattern-grep.log:3260-3262`, `logs/erdos-pattern-grep.log:3284-3286`.

   [I] Verifiable erdos-style benchmark tasks should focus on map keys, env branches, custom-deps toolchain keys, IDL local runtime gaps, and `COMPUTE_CAP` arch propagation because these are high-signal and checkable with no network. Evidence: source anchors in this finding plus the verified failure probes in findings 4-7.

12. [X] Concrete blade-mini repo proposal for L0-L3

   [X] L0 flat target tree should be:

   ```text
   BLADE_ROOT
   BUILD
   answer.cc
   answer.h
   main.cc
   ```

   [X] The verified instance is `projects/l0-flat`, and `docs/efforts/multi-agent-bench/evidence/research-blade/run_blade_build.sh l0-flat //:hello` exited `0`. Evidence: `logs/build-l0-flat.log:1-67`.

   [X] L1 pinned 3rdlib target tree should be:

   ```text
   BLADE_ROOT
   app/BUILD
   app/main.cc
   cpp3rdlib/math1/BUILD
   cpp3rdlib/math1/math1.cc
   cpp3rdlib/math1/math1.h
   ```

   [X] The verified instance is `projects/l1-vendored`, and `docs/efforts/multi-agent-bench/evidence/research-blade/run_blade_build.sh l1-vendored //app:use_math1` exited `0`. Evidence: `logs/build-l1-vendored.log:1-69`.

   [X] L2 macro/package target tree should be:

   ```text
   BLADE_ROOT
   BUILD
   BUILD.share
   demo.cc
   demo.h
   demo_main.cc
   ```

   [X] The verified instance is `projects/l2-macro`, and the fixed macro build `docs/efforts/multi-agent-bench/evidence/research-blade/run_blade_build.sh l2-macro //:demo` exited `0`. Evidence: `projects/l2-macro/BUILD.share:1-15`, `logs/build-l2-macro-fixed.log:1-69`.

   [X] L3 settler/env target tree should be:

   ```text
   BLADE_ROOT
   BUILD
   main.cc
   lib/BUILD
   lib/mode.cc
   lib/mode.h
   ```

   [X] The verified instance is `projects/l3-settler`, and `docs/efforts/multi-agent-bench/evidence/research-blade/run_blade_build.sh l3-settler //:app` exited `0`. Evidence: `logs/build-l3-settler.log:1-69`.

   [X] Additional verified negative fixtures under `projects/` are reusable for benchmark authoring: `failure-bad-dep`, `failure-missing-versioned-dep`, `failure-missing-versioned-dep-fixed`, `settler-dynamic-bad`, `settler-dynamic-fixed`, `load-no-symbols`, `load-missing-symbol`, `missing-filegroup`, `missing-genrule`, `proto-min`, `thrift-min`, `cuda-min`, `cc-test-min`, and `cost20`. Evidence: corresponding `logs/probe-*.log`, `logs/build-*.log`, and `logs/cost20-*.log`.

13. [I] Verifiable task shapes

   [I] L0 task: start with a missing direct header include or wrong `srcs` in `l0-flat`; expected fix is a local BUILD/source edit; verifier runs `run_blade_build.sh l0-flat //:hello` and optionally the loader-bind binary run. Evidence: `logs/build-l0-flat.log:1-67`, `logs/run-l0-binary-bind-relative-loader.log:1-2`.

   [I] L1 task: start with `cpp3rdlib/absent:1.0@//cpp3rdlib/absent:absent`; expected fix is vendoring the local package with a BUILD file; verifier checks the original error and then `run_blade_build.sh failure-missing-versioned-dep-fixed //:fixed_dep`. Evidence: `logs/probe-failure-missing-versioned-dep.log:1-34`, `logs/probe-failure-missing-versioned-dep-fixed.log:1-69`.

   [I] L2 task: start with a loaded macro that calls `native.cc_library`; expected fix is `native["cc_library"]`; verifier runs `run_blade_build.sh l2-macro //:demo`. Evidence: `logs/build-l2-macro.log:1-47`, `projects/l2-macro/BUILD.share:1-15`, `logs/build-l2-macro-fixed.log:1-69`.

   [I] L3 task: start with `library_settler(prefer_dynamic=["cpp3rdlib/dual"])` and an invalid shared object; expected fix is `prefer_static`; verifier runs the bad and fixed settler probes. Evidence: `logs/probe-settler-dynamic-bad.log:1-67`, `logs/probe-settler-dynamic-fixed.log:1-67`.

   [I] L3 task: start with a missing loaded symbol from `.bld`; expected fix is to import an existing symbol or define it; verifier checks `import a not loaded symbol[...]` disappears. Evidence: `logs/probe-load-missing-symbol.log:1-35`.

   [I] L3 task: start with a top-level BUILD using a real-repo helper name such as `override_prefer_dep` without loading the share file; expected fix is to load the macro file or define the helper; verifier checks the initial `NameError` and final build. Evidence: `logs/probe-missing-globals.log:1-45`, lgtorch helper definition in `git show 7ec5497:bytedance/lagrange_torch/csrc/BUILD.share`, lines 229-239.

Open questions

   [I] I did not establish a fully disabled remote-reporting command because the task pinned the only allowed Blade execution form to `blade build <target>` without additional flags. Evidence: `run_blade_build.sh:17-28`, remote controls in `/opt/tiger/typhoon-blade/src/blade/cmds/cmd.py:456-478`.

   [I] I did not establish the full local runtime target list for a successful vendored `proto_library` or `thrift_library`; source and failure logs show the compiler paths and hardcoded runtime dependency mechanism, but the benchmark still needs a tiny vendored protobuf/thrift runtime fixture if L4 IDL success is required. Evidence: `/opt/tiger/typhoon-blade/src/blade/configs/BLADE_ROOT:4-10`, `logs/probe-proto-min.log:1-34`, `logs/probe-thrift-min.log:47-65`.

   [I] I did not prove whether `blade test` can run tests offline because the allowed recipe was `blade build <target>` only. Evidence: task constraint and `logs/probe-cc-test-min.log:1-67`.

   [I] I did not prove GPU runtime execution because `nvidia-smi -L` failed in this session and the Blade CUDA probe could not see `nvcc` under the exact cleared recipe. Evidence: `logs/host-cuda-inspection.log:8-11`, `logs/probe-cuda-min.log:47-64`.

Things I got wrong or could not check

   [X] I initially tried a loaded macro with `native.cc_library(...)`; that was wrong because `native` is a dict, and the corrected macro uses `native["cc_library"](...)`. Evidence: `logs/build-l2-macro.log:1-47`, `projects/l2-macro/BUILD.share:1-15`, `logs/build-l2-macro-fixed.log:1-69`.

   [X] I tried duplicate-source and invalid-source-path failure probes, but duplicate source unexpectedly built, and invalid source path logged a problem while exiting `0`; I did not use them as benchmark-grade failure fixtures. Evidence: `logs/probe-failure-duplicate-source.log`, `logs/probe-failure-invalid-source-path.log:1-66`.

   [X] I did not run `blade`, `bpt`, or `bvc` outside the safe runner; the only Blade command in saved logs is through `run_blade_build.sh`, and the CUDA `sm_100` check was a direct host `nvcc` compile rather than a Blade build. Evidence: `logs/build-l0-flat.log:1`, `logs/probe-cuda-min.log:1`, `logs/nvcc-sm100.log:1-3`.

   [X] I did not fix the Blade CUDA sandbox to pass `NVCC` or `CUDA_PATH`; the minimal Blade CUDA project remains a failure fixture with `/bin/sh: 1: nvcc: not found`. Evidence: `logs/probe-cuda-min.log:47-64`.

# Emacs Internals Probe Summary

- Generated: 2026-09-30T04:03:32Z
- Total probes: 36
- Passing probes: 36

## Insula Runtime

- Checks passed: `t`
- Active: `t`
- Mode: `selected-container-rootfs`
- HOME: `/project/home`
- TMPDIR: `/project/tmp`
- XDG_RUNTIME_DIR: `/project/run`
- Unexpected environment names: `nil`

## Insula rootfs dependency closure contract

- ID: `insula-rootfs-dependency-closure-contract`
- Status: `pass`
- Observed: mode="selected-container-rootfs-insula" rootfs="/private-source/home/operator/workspace/soma/.scratch/emacs-gym/insula/rootfs-store/content/soma-emacs-0bddf41e6539849c-9dbdc1166f7f8d34388e5c65f1566a736beb5bd74f8ab6151d34" dependency_closure="rootfs_owned" abi_guarantee="container-rootfs-abi-manifest" production_required_tools=("emacs" "emacsclient" "python3" "jupyter" "pylsp" "pyright" "pyright-langserver" "clangd" "clang" "clang++" "gcc" "g++" "ldd" "rust-analyzer" "rustc" "cargo" "git" "tmux" "sh" "bash")
- Requirement: Production benchmark runs must select a Soma-owned rootfs manifest before launching Emacs, LSP servers, Jupyter kernels, Comint shells, or language toolchains.

## Insula plan and runtime consistency

- ID: `insula-plan-runtime-consistency`
- Status: `pass`
- Observed: run_id="20260930T040324Z-4018462"/"20260930T040324Z-4018462" mode="selected-container-rootfs"/"selected-container-rootfs-insula" cwd="/workspace/soma" repo_bind="/workspace/soma" output="/workspace/soma/probes/emacs-internals/traces/latest" probe_output="/workspace/soma/probes/emacs-internals/traces/latest"
- Requirement: The production Insula gateway must validate plan/runtime consistency before accepting Emacs, LSP, Jupyter, or checker evidence.

## Rootfs mount boundary evidence

- ID: `rootfs-mount-boundary-evidence`
- Status: `pass`
- Observed: repo="/workspace/soma" doom_core="/project/home/.emacs.d" doom_user="/project/home/.doom.d" ownership="soma-selected-container-rootfs" usr-bin-env=t temp="/workspace/soma/probes/emacs-internals/traces/latest/mount-boundary-Ch7zUH"
- Requirement: Production plans must record normalized mounts or a mount digest and fail if undeclared host roots provide tools or libraries.

## Toolchain resolution and closure classification

- ID: `toolchain-resolution-closure-classification`
- Status: `pass`
- Observed: classification=production-rootfs-closure inventory=((("tool" . "emacs") ("path" . "/opt/soma/emacs/bin/emacs") ("found" . t)) (("tool" . "python3") ("path" . "/usr/bin/python3") ("found" . t)) (("tool" . "jupyter") ("path" . "/usr/bin/jupyter") ("found" . t)) (("tool" . "pyright") ("path" . "/usr/local/bin/pyright") ("found" . t)) (("tool" . "pyright-langserver") ("path" . "/usr/local/bin/pyright-langserver") ("found" . t)) (("tool" . "clangd") ("path" . "/usr/bin/clangd") ("found" . t)) (("tool" . "rust-analyzer") ("path" . "/opt/soma/cargo/bin/rust-analyzer") ("found" . t)) (("tool" . "clang") ("path" . "/usr/bin/clang") ("found" . t)) (("tool" . "gcc") ("path" . "/usr/bin/gcc") ("found" . t)) (("tool" . "g++") ("path" . "/usr/bin/g++") ("found" . t)) (("tool" . "rustc") ("path" . "/opt/soma/cargo/bin/rustc") ("found" . t)) (("tool" . "cargo") ("path" . "/opt/soma/cargo/bin/cargo") ("found" . t))) missing=nil
- Requirement: Observation traces must record tool resolution, and production runners must fail before an episode if required tools are absent from the selected rootfs closure.

## Toolchain executable provenance matrix

- ID: `toolchain-executable-provenance-matrix`
- Status: `pass`
- Observed: prod=t tools=("emacs" "emacsclient" "python3" "jupyter" "pylsp" "pyright" "pyright-langserver" "clangd" "clang" "clang++" "gcc" "g++" "ldd" "rust-analyzer" "rustc" "cargo" "git" "tmux" "sh" "bash") missing=nil non-rootfs=nil forbidden-prefix=nil
- Requirement: The Insula gateway and benchmark traces must record executable realpaths, versions, hashes, and ABI-relevant dynamic-library surfaces for tools used by Emacs tasks.

## Toolchain smoke build rootfs identity

- ID: `toolchain-smoke-build-rootfs-identity`
- Status: `pass`
- Observed: prod=t tmp="/project/tmp/soma-toolchain-smoke-o4Jml4" python-status=0 cc="/usr/bin/clang" c-status=0 rustc="/opt/soma/cargo/bin/rustc" rust-status=0 forbidden=nil
- Requirement: Language-source-code benchmarks must smoke-test the relevant compiler/interpreter closure inside the Insula before trusting Emacs+LSP workflow evidence.

## Subprocess environment closure

- ID: `subprocess-environment-closure`
- Status: `pass`
- Observed: status=0 env_names=("CARGO_HOME" "HOME" "LANG" "LC_ALL" "PATH" "PWD" "RUSTUP_HOME" "SHELL" "SHLVL" "SOMA_INSULA_ACTIVE" "SOMA_INSULA_MODE" "SOMA_INSULA_PLAN" "SOMA_INSULA_RUN_ID" "TMPDIR" "XDG_CACHE_HOME" "XDG_CONFIG_HOME" "XDG_DATA_HOME" "XDG_RUNTIME_DIR" "_") unexpected=nil forbidden=nil
- Requirement: Every benchmark subprocess, including LSP servers, Jupyter kernels, Comint shells, and checkers, must prove environment closure before task output is trusted.

## Native-comp cache closure

- ID: `native-comp-cache-closure`
- Status: `pass`
- Observed: native-comp-feature=42 eln-paths=("/project/home/.emacs.d/eln-cache/" "/opt/soma/emacs/lib/emacs/30.2/native-lisp/") cache-like=("/project/home/.emacs.d/eln-cache/") nonprivate-cache=nil prod=t
- Requirement: The rootfs contract must record native-comp enablement, libgccjit identity, eln paths, and sandbox-owned cache/write locations.

## ABI surface requires a rootfs manifest

- ID: `abi-surface-requires-rootfs-manifest`
- Status: `pass`
- Observed: emacs="/opt/soma/emacs/bin/emacs-30.2" ldd-lines=32 abi_guarantee="container-rootfs-abi-manifest" manifest="/etc/soma-insula-rootfs-contract.json" glibc-owner="container-image-rootfs" nix-refs=nil
- Requirement: The Soma Insula contract must make ABI-relevant library/toolchain identity part of rootfs materialization and benchmark evidence.

## Emacs patch provenance contract

- ID: `emacs-patch-provenance-contract`
- Status: `pass`
- Observed: variant="stock" patch-set=nil source-revision="b3f36f18a6dd2715713370166257de2fae01f9d38cfe878ced9b1e6ded5befd9" configure-flags="--prefix=/opt/soma/emacs --with-native-compilation --with-tree-sitter --with-sqlite3 --without-x --without-sound --without-dbus --without-gconf --without-gsettings --without-toolkit-scroll-bars --without-xwidgets --without-mailutils" executable-hash="824c1318464a98778d9f1677508e05af3dc5e94c34c63653145e079e5d26f6c2" abi-manifest="/etc/soma-insula-rootfs-contract.json" stock-comparison="" emacs="/opt/soma/emacs/bin/emacs-30.2"
- Requirement: The Insula contract must treat patched Emacs as a named runtime variant and reject patched evidence that lacks source, patch, build, ABI, and stock-comparison provenance.

## Jupyter kernel rootfs identity proxy

- ID: `jupyter-kernel-rootfs-identity-proxy`
- Status: `pass`
- Observed: python="/usr/bin/python3" jupyter="/usr/bin/jupyter" python-status=0 python-output="{\"cwd\": \"/workspace/soma\", \"executable\": \"/usr/bin/python3\", \"path\": \"/opt/soma/emacs/bin:/opt/soma/cargo/bin:/usr/local/bin:/usr/bin:/bin\", \"plan\": \"/workspace/soma/probes/emacs-internals/traces/latest/insula-plan.json\", \"prefix\": \"/usr\", \"run_id\": \"20260930T040324Z-4018462\"}
"
- Requirement: Jupyter actions must verify kernel-side rootfs manifest, Python executable/prefix, cwd, import path, run id, and bwrap plan before grading execution.

## Jupyter kernelspec rootfs contract

- ID: `jupyter-kernelspec-rootfs-contract`
- Status: `pass`
- Observed: prod=t python="/usr/bin/python3" jupyter="/usr/bin/jupyter" status=0 python-specs=((("name" . "python3") ("resource_dir" . "/usr/share/jupyter/kernels/python3") ("argv" "/usr/bin/python3" "-Xfrozen_modules=off" "-m" "ipykernel_launcher" "-f" "{connection_file}"))) selected-launcher="/usr/bin/python3" launcher-path="/usr/bin/python3" bad-paths=nil
- Requirement: Jupyter benchmarks must grade kernelspec identity before kernel execution and reject host-resolved kernel launchers.

## Jupyter real kernel rootfs identity

- ID: `jupyter-real-kernel-rootfs-identity`
- Status: `pass`
- Observed: prod=t status=0 ok=t msg-id="7c3cbf93-b32761162d0951f92248f918_276_1" reply-parent="7c3cbf93-b32761162d0951f92248f918_276_1" kernel-exe="/usr/bin/python3" kernel-cwd="/workspace/soma" kernel-run-id="20260930T040324Z-4018462" output="SOMA_JUPYTER_PROBE:{\"connection_file\": \"/project/tmp/tmptxrc2ppn.json\", \"errors\": [], \"idle\": true, \"kernel_pid\": 280, \"msg_id\": \"7c3cbf93-b32761162d0951f92248f918_276_1\", \"ok\": true, \"payload\": {\"cwd\": \"/workspace/soma\", \"executable\": \"/usr/bin/python3\", \"path0\": [\"/usr/lib/python312.zip\", \"/usr/lib/python3.12\", \"/usr/lib/python3.12/lib-dynload\", \"\", \"/usr/local/lib/python3.12/dist-packages\"], \"plan\": \"/workspace/soma/probes/emacs-internals/traces/latest/insula-plan.json\", \"prefix\": \"/usr\", \"run_id\": \"20260930T040324Z-4018462\"}, \"reply_parent\": \"7c3cbf93-b32761162d0951f92248f918_276_1\", \"reply_status\": \"ok\"}

Process soma-probe-process finished
"
- Requirement: Jupyter actions must track kernel incarnation, request id, shell reply, parent-header outputs, idle/completion state, and kernel-side rootfs identity.

## LSP server rootfs resolution contract

- ID: `lsp-server-rootfs-resolution-contract`
- Status: `pass`
- Observed: prod=t inventory=((("tool" . "pyright-langserver") ("path" . "/usr/local/bin/pyright-langserver") ("found" . t)) (("tool" . "pyright") ("path" . "/usr/local/bin/pyright") ("found" . t)) (("tool" . "basedpyright-langserver") ("path" . "") ("found" . :json-false)) (("tool" . "pylsp") ("path" . "/usr/bin/pylsp") ("found" . t)) (("tool" . "clangd") ("path" . "/usr/bin/clangd") ("found" . t)) (("tool" . "rust-analyzer") ("path" . "/opt/soma/cargo/bin/rust-analyzer") ("found" . t))) python-lsp=t clangd="/usr/bin/clangd" rust-analyzer="/opt/soma/cargo/bin/rust-analyzer"
- Requirement: LSP tasks must prove server process identity, workspace root, synchronized document version, and rootfs-owned server/toolchain resolution.

## LSP stdio initialize rootfs identity

- ID: `lsp-stdio-initialize-rootfs-identity`
- Status: `pass`
- Observed: prod=t status=0 results=((("cmd" "clangd" "--log=error") ("exe" . "/usr/bin/clangd") ("exit_code" . 0) ("initialize_error") ("initialize_has_result" . t) ("name" . "clangd") ("pid" . 302) ("realpath" . "/usr/lib/llvm-18/bin/clangd") ("root" . "/project/tmp/soma-lsp-lu5imjcq") ("shutdown_has_result" . t) ("status" . "ok")) (("cmd" "pyright-langserver" "--stdio") ("exe" . "/usr/local/bin/pyright-langserver") ("exit_code" . 0) ("initialize_error") ("initialize_has_result" . t) ("name" . "pyright-langserver") ("pid" . 413) ("realpath" . "/usr/local/lib/node_modules/pyright/langserver.index.js") ("root" . "/project/tmp/soma-lsp-etlcsrxa") ("shutdown_has_result" . t) ("status" . "ok")) (("cmd" "rust-analyzer") ("exe" . "/opt/soma/cargo/bin/rust-analyzer") ("exit_code" . 0) ("initialize_error") ("initialize_has_result" . t) ("name" . "rust-analyzer") ("pid" . 429) ("realpath" . "/opt/soma/cargo/bin/rustup") ("root" . "/project/tmp/soma-lsp-o833vvdc") ("shutdown_has_result" . t) ("status" . "ok"))) bad=nil bad-paths=nil output="SOMA_LSP_PROBE:{\"results\": [{\"cmd\": [\"clangd\", \"--log=error\"], \"exe\": \"/usr/bin/clangd\", \"exit_code\": 0, \"initialize_error\": null, \"initialize_has_result\": true, \"name\": \"clangd\", \"pid\": 302, \"realpath\": \"/usr/lib/llvm-18/bin/clangd\", \"root\": \"/project/tmp/soma-lsp-lu5imjcq\", \"shutdown_has_result\": true, \"status\": \"ok\"}, {\"cmd\": [\"pyright-langserver\", \"--stdio\"], \"exe\": \"/usr/local/bin/pyright-langserver\", \"exit_code\": 0, \"initialize_error\": null, \"initialize_has_result\": true, \"name\": \"pyright-langserver\", \"pid\": 413, \"realpath\": \"/usr/local/lib/node_modules/pyright/langserver.index.js\", \"root\": \"/project/tmp/soma-lsp-etlcsrxa\", \"shutdown_has_result\": true, \"status\": \"ok\"}, {\"cmd\": [\"rust-analyzer\"], \"exe\": \"/opt/soma/cargo/bin/rust-analyzer\", \"exit_code\": 0, \"initialize_error\": null, \"initialize_has_result\": true, \"name\": \"rust-analyzer\", \"pid\": 429, \"realpath\": \"/opt/soma/cargo/bin/rustup\", \"root\": \"/project/tmp/soma-lsp-o833vvdc\", \"shutdown_has_result\": true, \"status\": \"ok\"}]}

Process soma-probe-process finished
"
- Requirement: LSP benchmarks must record server executable identity, pid, workspace root, initialize response, shutdown status, and document-version evidence before trusting refactor results.

## Comint subprocess rootfs identity

- ID: `comint-subprocess-rootfs-identity`
- Status: `pass`
- Observed: prod=t run-id="20260930T040324Z-4018462" output="cwd=/workspace/soma
run_id=20260930T040324Z-4018462
python=/usr/bin/python3
path=/opt/soma/emacs/bin:/opt/soma/cargo/bin:/usr/local/bin:/usr/bin:/bin

Process soma-comint-rootfs-probe finished
"
- Requirement: Comint benchmark graders must record process identity, cwd, PATH/tool resolution, rootfs manifest, and timeout/unknown state before trusting process output.

## Comint long-lived shell cwd and tool drift

- ID: `comint-long-lived-shell-cwd-tool-drift`
- Status: `pass`
- Observed: prod=t pid=760 proc-exe="/usr/bin/dash" output="printf 'SOMA_BEFORE:%s\\n' \"$PWD\"; cd \"$TMPDIR\"; printf 'SOMA_AFTER:%s\\n' \"$PWD\"; printf 'SOMA_RUN:%s\\n' \"$SOMA_INSULA_RUN_ID\"; printf 'SOMA_PY:%s\\n' \"$(command -v python3 || true)\"; python3 -c 'import json, os, sys; print(\"SOMA_PY_JSON:\" + json.dumps({\"executable\": sys.executable, \"cwd\": os.getcwd(), \"run_id\": os.environ.get(\"SOMA_INSULA_RUN_ID\", \"\")}, sort_keys=True))'; printf 'SOMA_DONE\\n'
$ SOMA_BEFORE:/workspace/soma
SOMA_AFTER:/project/tmp
SOMA_RUN:20260930T040324Z-4018462
SOMA_PY:/usr/bin/python3
SOMA_PY_JSON:{\"cwd\": \"/project/tmp\", \"executable\": \"/usr/bin/python3\", \"run_id\": \"20260930T040324Z-4018462\"}
SOMA_DONE
$ "
- Requirement: Comint actions must record process pid/executable, process mark, cwd, live input, tool resolution, and environment after every wait or submitted command.

## Comint timeout outcome_unknown without retry

- ID: `comint-timeout-outcome-unknown-no-retry`
- Status: `pass`
- Observed: early-done=nil outcome=outcome_unknown final-done=210 counter="1" output="sleep 0.6; n=$(cat /project/tmp/soma-comint-timeout-UsFT35/counter); n=$((n+1)); printf '%s\\n' \"$n\" > /project/tmp/soma-comint-timeout-UsFT35/counter; printf 'SOMA_TIMEOUT_DONE:%s\\n' \"$n\"
$ SOMA_TIMEOUT_DONE:1
$ "
- Requirement: Comint and process actions must represent timeout as `outcome_unknown' and block blind retry unless the operation is proven idempotent or externally canceled.

## Current buffer versus selected window

- ID: `current-buffer-selected-window`
- Status: `pass`
- Observed: selected-before=soma-probe-a current-during=soma-probe-b selected-during=soma-probe-a selected-after=soma-probe-b
- Requirement: Bridge operations must separate read/operate-in-buffer, display-buffer, and select-for-user actions.

## Direct call versus interactive invocation versus command dispatch

- ID: `command-routes`
- Status: `pass`
- Observed: ((("arg" . "direct") ("this_command" . "nil") ("last_command" . "nil") ("called_interactively" . :json-false)) (("arg" . "(4)") ("this_command" . "nil") ("last_command" . "nil") ("called_interactively" . :json-false)) (("arg" . "nil") ("this_command" . "nil") ("last_command" . "nil") ("called_interactively" . :json-false)) (("arg" . "nil") ("this_command" . "soma-probe-command") ("last_command" . "nil") ("called_interactively" . :json-false)))
- Requirement: Action profiles must classify direct, interactive, command-dispatch, and terminal-input paths; terminal fidelity remains a separate PTY probe gate.

## Magit post-stage hook and bridge wrapper command identity

- ID: `magit-wrapper-this-command`
- Status: `pass`
- Observed: post-stage-hook-runs=("magit-stage")
- Requirement: Faithful Magit actions must either enter through a recognized command path or explicitly model and test the command identity consequences.

## Base and indirect buffers share text

- ID: `base-indirect-buffers`
- Status: `pass`
- Observed: base="soma-base" indirect="soma-indirect" indirect-base="soma-base" base-text="alpha-from-indirect"
- Requirement: Observations must include buffer identity, base-buffer/text-owner identity, and visited file identity separately.

## Marker movement under edits

- ID: `marker-movement`
- Status: `pass`
- Observed: before-pos=4 before-context="cDE" after-pos=7 after-context="cZE" text="xxxabcZEF"
- Requirement: Delayed edits and FIM suggestions must validate prefix, suffix, buffer incarnation, and target range text before mutation.

## Narrowing or region changes without character changes

- ID: `narrowing-region-without-text-change`
- Status: `pass`
- Observed: tick=5->5 range=1-15->5-9 region-active=t
- Requirement: Every action needs operation-specific preconditions beyond a universal character revision guard.

## Minibuffer cancellation and transient argument context

- ID: `minibuffer-transient-interruption`
- Status: `pass`
- Observed: minibuffer-result=end-of-file transient-library="/project/home/.emacs.d/.local/straight/build-30.2/transient/transient.elc"
- Requirement: Bridge observations must expose active minibuffer/transient state, and terminal-path probes must cover transient prefix/infix/suffix lifetimes before those actions are benchmarked.

## Reentrant process output while instrumentation is active

- ID: `reentrant-process-output`
- Status: `pass`
- Observed: max-depth=2 events=("depth=1 text=\"first\"" "depth=2 text=\"second\"")
- Requirement: Actions that wait for external output must repeat relevant consistency checks after waits and represent outcome_unknown where completion cannot be proven.

## Cold versus warmed Doom session

- ID: `doom-cold-warmed-state`
- Status: `pass`
- Observed: module-file="/project/home/.emacs.d/.local/etc/@/init.30.d/95-modules.auto.el" has-magit-module=1172 magit-autoload=t feature-before=nil
- Requirement: Gym environment identity must record Doom core/user config/generated profile/package build ids and current feature/mode/service readiness separately.

## Org source-edit propagation

- ID: `org-source-edit-propagation`
- Status: `pass`
- Observed: before="#+begin_src emacs-lisp
(+ 1 2)
#+end_src
" after-edit-parent="#+begin_src emacs-lisp
(+ 1 2)
#+end_src
" after-exit-parent="#+begin_src emacs-lisp
  (+ 3 4)
#+end_src
"
- Requirement: Org gym actions must distinguish source-edit buffers from parent Org buffers and grade the parent document after write-back.

## Jupyter delayed reply and duplicate retry

- ID: `jupyter-delayed-reply-duplicate-retry`
- Status: `pass`
- Observed: first=1 retry=2 reply=2 requests=((:id 1 :code "counter += 1" :state replied) (:id 2 :code "counter += 1" :state replied))
- Requirement: Jupyter actions need request ids, parent headers or equivalent handles, kernel incarnation, and an outcome_unknown state that blocks blind retry of non-idempotent code.

## FIM acceptance after human editing

- ID: `fim-stale-acceptance`
- Status: `pass`
- Observed: marker=28 tick=6->9 accepted=nil text="def f(value):
    return x
"
- Requirement: Completion acceptance must validate buffer identity, modification tick, target range, prefix, suffix, and intended replacement before editing.

## Observation reads can trigger fontification hooks

- ID: `observation-fontification-side-effect`
- Status: `pass`
- Observed: after-no-properties=nil after-buffer-substring=("1-11")
- Requirement: Observation APIs must be classified as passive state reads, derived inspections, or active diagnostics, and probe traces must record which class was used.

## Completion-at-point provider exclusivity

- ID: `capf-provider-exclusivity`
- Status: `pass`
- Observed: exclusive-calls=("first") exclusive-result=(#[nil ((progn (setq exclusive-calls (cons "first" exclusive-calls)) (list (point-min) (point-max) (list "aa")))) ((exclusive-calls "first"))] 1 3 ("aa")) nonexclusive-calls=("first" "second") nonexclusive-result=(#[nil ((progn (setq nonexclusive-calls (cons "second" nonexclusive-calls)) (list (point-min) (point-max) (list "zz-top")))) ((nonexclusive-calls "first" "second"))] 1 3 ("zz-top"))
- Requirement: Completion integration must record provider order, replacement range, exclusivity, and whether the model path is separate from or participates in CAPF.

## Completion-at-point exit function acceptance

- ID: `capf-exit-function-acceptance`
- Status: `pass`
- Observed: capf-return=t capf-text="foobar" capf-events=(("foobar" finished)) raw-text="foobar" raw-events=nil
- Requirement: Completion acceptance evidence must record whether a provider exit function existed, whether it ran, which candidate/status it saw, and whether the edit path was CAPF, preview, or a separate model overlay.

## Comint input depends on point relative to process mark

- ID: `comint-process-mark-input-boundary`
- Status: `pass`
- Observed: old-point-output=(#("got:old-cmd" 0 11 (insert-in-front-hooks (comint--mark-as-output comint--mark-yanked-as-output) inhibit-line-move-field-capture t front-sticky (field inhibit-line-move-field-capture) rear-nonsticky (field inhibit-line-move-field-capture read-only font-lock-face insert-in-front-hooks) field output))) live-input-output=(#("got:bottom-cmd" 0 14 (insert-in-front-hooks (comint--mark-as-output comint--mark-yanked-as-output) inhibit-line-move-field-capture t front-sticky (field inhibit-line-move-field-capture) rear-nonsticky (field inhibit-line-move-field-capture read-only font-lock-face insert-in-front-hooks) field output)))
- Requirement: Comint actions must observe point, process mark, live input text, old-input policy, process identity, and working directory before submitting.

## Dired undo does not restore filesystem changes

- ID: `dired-undo-does-not-restore-files`
- Status: `pass`
- Observed: mark-before=42 undo-result=(error "(user-error \"No further undo information\")") mark-after=nil file-exists-after-dired-undo=nil
- Requirement: Dired benchmarks must grade filesystem state independently from Dired buffer state and must classify destructive file operations as externally consequential.

## with-timeout only fires at wait points

- ID: `with-timeout-wait-boundary`
- Status: `pass`
- Observed: wait-result=timed-out busy-result=finished busy-elapsed=0.0803
- Requirement: The gym supervisor needs an out-of-process watchdog, and operation results must distinguish failed, canceled, and outcome_unknown.

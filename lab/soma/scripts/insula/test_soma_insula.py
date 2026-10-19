#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import tempfile
import unittest

from scripts.insula import soma_insula


class SomaInsulaTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.base = pathlib.Path(self.tmp.name)
        self.repo = self.base / "repo"
        self.repo.mkdir()
        self.store = self.repo / ".scratch" / "emacs-gym" / "insula" / "rootfs-store"
        self.store_id = "soma-emacs-test"
        self.rootfs = self.store / "content" / self.store_id
        (self.rootfs / "etc").mkdir(parents=True)
        (self.rootfs / "opt" / "soma" / "emacs" / "bin").mkdir(parents=True)
        (self.rootfs / "opt" / "soma" / "doom-home" / ".emacs.d").mkdir(parents=True)
        (self.rootfs / "opt" / "soma" / "doom-home" / ".doom.d").mkdir(parents=True)
        emacs = self.rootfs / "opt" / "soma" / "emacs" / "bin" / "emacs"
        emacs.write_text("#!/bin/sh\n", encoding="utf-8")
        emacs.chmod(0o755)
        contract = {
            "rootfs_ownership": "soma-selected-container-rootfs",
            "dependency_closure": "rootfs_owned",
            "abi_guarantee": "container-rootfs-abi-manifest",
            "abi_base": "container-image-rootfs",
            "glibc_owner": "container-image-rootfs",
            "nix_allowed": False,
            "base_image": "example/image@sha256:base",
            "image_tag": "soma-emacs-rootfs:test",
            "image_id": "sha256:test",
            "rootfs_recipe_sha256": "recipe-sha",
            "rootfs_tree_manifest": {
                "kind": "soma_rootfs_tree_manifest_v1",
                "tree_hash_sha256": "tree-sha",
            },
            "emacs": {
                "source_sha256": "emacs-source-sha",
                "source_url": "https://example.invalid/emacs.tar.xz",
                "configure_flags": ["--with-json"],
                "executable_sha256": "emacs-bin-sha",
            },
            "doom": {
                "emacsdir": "/opt/soma/doom-home/.emacs.d",
                "doomdir": "/opt/soma/doom-home/.doom.d",
                "commit": "doom-commit",
                "profile_source": "minimal-rootfs-profile",
            },
            "tool_inventory": {
                "emacs": "/opt/soma/emacs/bin/emacs",
            },
        }
        (self.rootfs / "etc" / "soma-insula-rootfs-contract.json").write_text(
            json.dumps(contract) + "\n",
            encoding="utf-8",
        )
        self.store.mkdir(parents=True, exist_ok=True)
        (self.store / "selected.json").write_text(
            json.dumps({"store_id": self.store_id}) + "\n",
            encoding="utf-8",
        )

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def test_resolve_selected_rootfs_rejects_escaped_store_id(self) -> None:
        (self.store / "selected.json").write_text(
            json.dumps({"store_id": "../escape"}) + "\n",
            encoding="utf-8",
        )
        with self.assertRaisesRegex(soma_insula.InsulaError, "invalid selected rootfs store id"):
            soma_insula.resolve_selected_rootfs(self.store)

    def test_network_share_requires_matching_task_authorization(self) -> None:
        with self.assertRaisesRegex(soma_insula.InsulaError, "requires SOMA_INSULA_ALLOW_NETWORK_FOR"):
            soma_insula.authorize_network("share-net", "jupyter-existing-kernel", "wrong-task")

        authorization = soma_insula.authorize_network(
            "share-net",
            "jupyter-existing-kernel",
            "jupyter-existing-kernel",
        )
        self.assertTrue(authorization.share_net)
        self.assertEqual(
            authorization.authorization,
            "SOMA_INSULA_ALLOW_NETWORK_FOR=jupyter-existing-kernel",
        )

    def test_prepare_batch_invocation_writes_plan_and_bwrap_policy(self) -> None:
        invocation = soma_insula.prepare_batch_invocation(
            root=self.repo,
            out_dir=pathlib.Path("benchmarks/emacs-gym/dired/traces/test"),
            benchmark_id="dired-filesystem",
            task_network_id="dired-filesystem",
            elisp_sandbox_path="/workspace/soma/benchmarks/emacs-gym/dired/elisp/run-benchmark.el",
            store_root=self.store,
            run_id="unit-run",
            network_mode="disabled",
            network_allowed_for="",
            require_bwrap=False,
            run_validator=False,
        )

        plan = json.loads(invocation.paths.plan_host.read_text(encoding="utf-8"))
        self.assertEqual(plan["benchmark"], "dired-filesystem")
        self.assertEqual(plan["mode"], "selected-container-rootfs-insula")
        self.assertEqual(plan["dependency_closure"], "rootfs_owned")
        self.assertEqual(plan["glibc_owner"], "container-image-rootfs")
        self.assertFalse(plan["nix_allowed"])
        self.assertEqual(plan["soma_insula_authority"], "scripts/insula/soma_insula.py")
        self.assertIn("--ro-bind", plan["bwrap_mount_args"])
        self.assertIn("SOMA_INSULA_PLAN", plan["environment_allowlist"])
        self.assertEqual(
            invocation.command[-3:],
            [
                "/workspace/soma/benchmarks/emacs-gym/dired/elisp/run-benchmark.el",
                "--",
                "/workspace/soma/benchmarks/emacs-gym/dired/traces/test",
            ],
        )

    def test_prepare_batch_invocation_preserves_package_specific_runtime_fields(self) -> None:
        invocation = soma_insula.prepare_batch_invocation(
            root=self.repo,
            out_dir=pathlib.Path("benchmarks/emacs-gym/jupyter/traces/test"),
            benchmark_id="jupyter-existing-kernel",
            task_network_id="jupyter-existing-kernel",
            elisp_sandbox_path="/workspace/soma/benchmarks/emacs-gym/jupyter/elisp/run-benchmark.el",
            store_root=self.store,
            run_id="unit-run",
            network_mode="disabled",
            network_allowed_for="",
            private_jupyter_runtime=True,
            extra_env={"LSP_USE_PLISTS": "true"},
            extra_plan={"fixture_origin": "generated-inside-bwrap-private-project-runtime"},
            require_bwrap=False,
            run_validator=False,
        )

        plan = json.loads(invocation.paths.plan_host.read_text(encoding="utf-8"))
        self.assertEqual(plan["private_jupyter_runtime"], "/project/run/jupyter")
        self.assertEqual(plan["fixture_origin"], "generated-inside-bwrap-private-project-runtime")
        self.assertIn("/project/run/jupyter", plan["bwrap_mount_args"])
        self.assertIn("JUPYTER_RUNTIME_DIR", plan["environment_allowlist"])
        self.assertIn("LSP_USE_PLISTS", plan["environment_allowlist"])
        self.assertEqual(invocation.environment["JUPYTER_RUNTIME_DIR"], "/project/run/jupyter")
        self.assertEqual(invocation.environment["LSP_USE_PLISTS"], "true")

    def test_prepare_batch_invocation_rejects_unowned_extra_environment_field(self) -> None:
        with self.assertRaisesRegex(soma_insula.InsulaError, "--setenv PATH is not allowed"):
            soma_insula.prepare_batch_invocation(
                root=self.repo,
                out_dir=pathlib.Path("benchmarks/emacs-gym/lsp/traces/test"),
                benchmark_id="lsp-source",
                task_network_id="lsp-source",
                elisp_sandbox_path="/workspace/soma/benchmarks/emacs-gym/lsp/elisp/run-benchmark.el",
                store_root=self.store,
                run_id="unit-run",
                network_mode="disabled",
                network_allowed_for="",
                extra_env={"PATH": "/host/bin"},
                require_bwrap=False,
                run_validator=False,
            )

    def test_prepare_batch_invocation_rejects_unowned_extra_plan_field(self) -> None:
        with self.assertRaisesRegex(soma_insula.InsulaError, "--plan-string rootfs is not allowed"):
            soma_insula.prepare_batch_invocation(
                root=self.repo,
                out_dir=pathlib.Path("benchmarks/emacs-gym/lsp/traces/test"),
                benchmark_id="lsp-source",
                task_network_id="lsp-source",
                elisp_sandbox_path="/workspace/soma/benchmarks/emacs-gym/lsp/elisp/run-benchmark.el",
                store_root=self.store,
                run_id="unit-run",
                network_mode="disabled",
                network_allowed_for="",
                extra_plan={"rootfs": "/tmp/fake-rootfs"},
                require_bwrap=False,
                run_validator=False,
            )

    def test_prepare_command_invocation_substitutes_out_dir_and_records_process_model(self) -> None:
        stale = self.repo / "probes" / "emacs-internals" / "traces" / "test" / "01-old.json"
        stale.parent.mkdir(parents=True)
        stale.write_text("{}\n", encoding="utf-8")
        invocation = soma_insula.prepare_command_invocation(
            root=self.repo,
            out_dir=pathlib.Path("probes/emacs-internals/traces/test"),
            benchmark_id="emacs-internals-probes",
            task_network_id="emacs-internals-probes",
            command=[
                "/workspace/soma/probes/emacs-internals/scripts/run-probes-batch.sh",
                "{out_dir_sandbox}",
            ],
            store_root=self.store,
            run_id="unit-run",
            network_mode="disabled",
            network_allowed_for="",
            process_model="batch-emacs-internals-probes",
            extra_plan={"emacs_patch_policy": "stock-vs-patched evidence required"},
            clean_patterns=("[0-9][0-9]-*.json", "summary.json", "summary.md", "insula-plan.json"),
            require_bwrap=False,
            run_validator=False,
        )

        plan = json.loads(invocation.paths.plan_host.read_text(encoding="utf-8"))
        self.assertFalse(stale.exists())
        self.assertEqual(
            invocation.command,
            [
                "/workspace/soma/probes/emacs-internals/scripts/run-probes-batch.sh",
                "/workspace/soma/probes/emacs-internals/traces/test",
            ],
        )
        self.assertEqual(plan["benchmark"], "emacs-internals-probes")
        self.assertEqual(plan["process_model"], "batch-emacs-internals-probes")
        self.assertEqual(plan["emacs_patch_policy"], "stock-vs-patched evidence required")
        self.assertEqual(plan["entrypoint_argv"], invocation.command)
        self.assertEqual(plan["soma_insula_authority"], "scripts/insula/soma_insula.py")

    def test_prepare_command_invocation_preserves_lifecycle_doom_state_contract(self) -> None:
        cache_source = self.rootfs / "opt" / "soma" / "doom-home" / ".emacs.d" / ".local" / "cache"
        state_source = self.rootfs / "opt" / "soma" / "doom-home" / ".emacs.d" / ".local" / "state"
        cache_source.mkdir(parents=True)
        state_source.mkdir(parents=True)
        (cache_source / "profiles.@.el").write_text("cache\n", encoding="utf-8")
        (state_source / "persistent.el").write_text("state\n", encoding="utf-8")

        invocation = soma_insula.prepare_command_invocation(
            root=self.repo,
            out_dir=pathlib.Path("workers/emacs-doom-lifecycle/traces/test"),
            benchmark_id="doom-lifecycle",
            task_network_id="doom-lifecycle",
            command=[
                "/opt/soma/emacs/bin/emacs",
                "--quick",
                "--batch",
                "-l",
                "/workspace/soma/workers/emacs-doom-lifecycle/elisp/run-lifecycle.el",
                "--",
                "{out_dir_sandbox}",
            ],
            store_root=self.store,
            run_id="unit-run",
            network_mode="disabled",
            network_allowed_for="",
            process_model="batch-doom-lifecycle-worker",
            worker_id="doom-lifecycle",
            doom_environment=True,
            writable_doom_local_state=True,
            include_doom_profile_source=True,
            require_bwrap=False,
            run_validator=False,
        )

        plan = json.loads(invocation.paths.plan_host.read_text(encoding="utf-8"))
        self.assertEqual(invocation.environment["EMACSDIR"], "/project/home/.emacs.d")
        self.assertEqual(invocation.environment["DOOMDIR"], "/project/home/.doom.d")
        self.assertIn("EMACSDIR", plan["environment_allowlist"])
        self.assertIn("DOOMDIR", plan["environment_allowlist"])
        self.assertEqual(plan["worker"], "doom-lifecycle")
        self.assertEqual(plan["doom_profile_source"], "minimal-rootfs-profile")
        self.assertIn(
            str(invocation.paths.run_root / "doom-local-cache"),
            plan["bwrap_mount_args"],
        )
        self.assertIn(
            str(invocation.paths.run_root / "doom-local-state"),
            plan["bwrap_mount_args"],
        )
        self.assertEqual(
            (invocation.paths.run_root / "doom-local-cache" / "profiles.@.el").read_text(
                encoding="utf-8"
            ),
            "cache\n",
        )
        self.assertEqual(
            (invocation.paths.run_root / "doom-local-state" / "persistent.el").read_text(
                encoding="utf-8"
            ),
            "state\n",
        )

    def test_prepare_command_invocation_preserves_daemon_bridge_runtime_contract(self) -> None:
        emacsclient = self.rootfs / "opt" / "soma" / "emacs" / "bin" / "emacsclient"
        emacsclient.write_text("#!/bin/sh\n", encoding="utf-8")
        emacsclient.chmod(0o755)
        stale_log = self.repo / "benchmarks" / "emacs-gym" / "bridge" / "traces" / "test-daemon" / "old.log"
        stale_log.parent.mkdir(parents=True)
        stale_log.write_text("old\n", encoding="utf-8")

        invocation = soma_insula.prepare_command_invocation(
            root=self.repo,
            out_dir=pathlib.Path("benchmarks/emacs-gym/bridge/traces/test-daemon"),
            benchmark_id="bridge-daemon-transport",
            task_network_id="bridge-daemon-transport",
            command=[
                "/bin/bash",
                "/workspace/soma/benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark-in-rootfs.sh",
                "{out_dir_sandbox}",
            ],
            store_root=self.store,
            run_id="unit-run",
            network_mode="disabled",
            network_allowed_for="",
            process_model="emacs-daemon-plus-external-jsonl-client",
            private_jupyter_runtime=True,
            daemon_bridge_runtime=True,
            clean_patterns=("*.json", "*.md", "*.txt", "*.log", "*.out", "*.err"),
            require_bwrap=False,
            run_validator=False,
        )

        plan = json.loads(invocation.paths.plan_host.read_text(encoding="utf-8"))
        self.assertFalse(stale_log.exists())
        self.assertEqual(plan["soma_insula_authority"], "scripts/insula/soma_insula.py")
        self.assertEqual(plan["process_model"], "emacs-daemon-plus-external-jsonl-client")
        self.assertEqual(plan["emacsclient"], "/opt/soma/emacs/bin/emacsclient")
        self.assertEqual(plan["emacs_daemon_name"], "soma-gym-bridge-daemon")
        self.assertEqual(plan["bridge_socket"], "/project/run/soma-gym-bridge-daemon.sock")
        self.assertEqual(
            plan["daemon_ready_file"],
            "/workspace/soma/benchmarks/emacs-gym/bridge/traces/test-daemon/daemon-ready.json",
        )
        self.assertEqual(plan["terminal_frame_transport"], "rootfs-owned tmux plus emacsclient --tty")
        self.assertEqual(plan["terminal_term"], "xterm-256color")
        self.assertEqual(plan["terminal_tmux_socket"], "/project/run/soma-gym-bridge-terminal-tmux.sock")
        self.assertEqual(plan["terminal_tmux_session"], "soma-gym-bridge-terminal")
        self.assertEqual(plan["private_jupyter_runtime"], "/project/run/jupyter")
        self.assertEqual(invocation.environment["SOMA_BRIDGE_DAEMON_NAME"], "soma-gym-bridge-daemon")
        self.assertEqual(
            invocation.environment["SOMA_BRIDGE_DAEMON_SOCKET"],
            "/project/run/soma-gym-bridge-daemon.sock",
        )
        self.assertEqual(
            invocation.environment["SOMA_BRIDGE_DAEMON_READY"],
            "/workspace/soma/benchmarks/emacs-gym/bridge/traces/test-daemon/daemon-ready.json",
        )
        self.assertEqual(
            invocation.environment["SOMA_BRIDGE_TERMINAL_TMUX_SOCKET"],
            "/project/run/soma-gym-bridge-terminal-tmux.sock",
        )
        self.assertEqual(
            invocation.environment["SOMA_BRIDGE_TERMINAL_TMUX_SESSION"],
            "soma-gym-bridge-terminal",
        )
        self.assertEqual(invocation.environment["TERM"], "xterm-256color")
        self.assertIn("JUPYTER_RUNTIME_DIR", plan["environment_allowlist"])
        self.assertIn("SOMA_BRIDGE_DAEMON_READY", plan["environment_allowlist"])
        self.assertIn("/project/run/jupyter", plan["bwrap_mount_args"])
        self.assertEqual(
            invocation.command,
            [
                "/bin/bash",
                "/workspace/soma/benchmarks/emacs-gym/bridge/scripts/run-daemon-benchmark-in-rootfs.sh",
                "/workspace/soma/benchmarks/emacs-gym/bridge/traces/test-daemon",
            ],
        )

    def test_prepare_batch_invocation_rejects_output_outside_repo(self) -> None:
        with self.assertRaisesRegex(soma_insula.InsulaError, "output directory must be under"):
            soma_insula.prepare_batch_invocation(
                root=self.repo,
                out_dir=self.base / "outside",
                benchmark_id="dired-filesystem",
                task_network_id="dired-filesystem",
                elisp_sandbox_path="/workspace/soma/benchmarks/emacs-gym/dired/elisp/run-benchmark.el",
                store_root=self.store,
                run_id="unit-run",
                network_mode="disabled",
                network_allowed_for="",
                require_bwrap=False,
                run_validator=False,
            )

    def test_prepare_batch_invocation_rejects_escaping_run_id(self) -> None:
        with self.assertRaisesRegex(soma_insula.InsulaError, "invalid Insula run id"):
            soma_insula.prepare_batch_invocation(
                root=self.repo,
                out_dir=pathlib.Path("benchmarks/emacs-gym/dired/traces/test"),
                benchmark_id="dired-filesystem",
                task_network_id="dired-filesystem",
                elisp_sandbox_path="/workspace/soma/benchmarks/emacs-gym/dired/elisp/run-benchmark.el",
                store_root=self.store,
                run_id="/tmp/escaped-run",
                network_mode="disabled",
                network_allowed_for="",
                require_bwrap=False,
                run_validator=False,
            )

    def test_validate_selected_rootfs_rejects_nix_contract_reference(self) -> None:
        contract_path = self.rootfs / "etc" / "soma-insula-rootfs-contract.json"
        contract = json.loads(contract_path.read_text(encoding="utf-8"))
        contract["forbidden_example"] = "/nix/store/example"
        contract_path.write_text(json.dumps(contract) + "\n", encoding="utf-8")
        selected = soma_insula.resolve_selected_rootfs(self.store)

        with self.assertRaisesRegex(soma_insula.InsulaError, "forbidden"):
            soma_insula.validate_selected_rootfs(
                self.repo,
                selected,
                run_validator=False,
            )

    def test_parse_key_value_rejects_missing_separator(self) -> None:
        with self.assertRaisesRegex(soma_insula.InsulaError, "KEY=VALUE"):
            soma_insula.parse_key_value("LSP_USE_PLISTS", "--setenv")


if __name__ == "__main__":
    unittest.main()

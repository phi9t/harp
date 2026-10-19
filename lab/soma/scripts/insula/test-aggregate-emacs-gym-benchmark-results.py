#!/usr/bin/env python3
import json
import pathlib
import subprocess
import sys
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
AGGREGATOR = ROOT / "scripts" / "insula" / "aggregate-emacs-gym-benchmark-results.py"
BENCHMARKS = (
    "magit",
    "comint",
    "jupyter",
    "lsp",
    "org",
    "dired",
    "bridge",
    "bridge-daemon",
)
MODULE_AUTHORITY_BENCHMARKS = set(BENCHMARKS)


class AggregateEmacsGymBenchmarkResultsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.base = pathlib.Path(self.tmp.name).resolve()
        self.store_root = self.base / "rootfs-store"
        self.out = self.base / "traces"
        self.run_id = "test-run"
        self.store_id = "soma-emacs-test-store"
        self.rootfs = self.store_root / "content" / self.store_id
        self.contract_path = self.rootfs / "etc" / "soma-insula-rootfs-contract.json"
        self.contract_path.parent.mkdir(parents=True)
        (self.store_root / "selected.json").write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "kind": "soma_rootfs_selection",
                    "store_id": self.store_id,
                }
            )
            + "\n",
            encoding="utf-8",
        )
        self.contract = {
            "materialization_kind": "source-built-emacs-v8",
            "rootfs_recipe_sha256": "recipe-sha",
            "base_image": "example/image@sha256:base",
            "image_tag": "soma-emacs-rootfs:recipe",
            "image_id": "sha256:image",
            "rootfs_tree_manifest": {
                "kind": "soma_rootfs_tree_manifest_v1",
                "tree_hash_sha256": "tree-sha",
                "manifest_sha256": "manifest-sha",
            },
        }
        self.contract_path.write_text(
            json.dumps(self.contract) + "\n",
            encoding="utf-8",
        )
        self._write_child_outputs()

    def tearDown(self):
        self.tmp.cleanup()

    def _matching_plan(self, name):
        plan = {
            "benchmark": name,
            "run_id": self.run_id,
            "mode": "selected-container-rootfs-insula",
            "dependency_closure": "rootfs_owned",
            "glibc_owner": "container-image-rootfs",
            "nix_allowed": False,
            "rootfs": str(self.rootfs),
            "rootfs_contract_host_path": str(self.contract_path),
            "rootfs_recipe_sha256": self.contract["rootfs_recipe_sha256"],
            "base_image": self.contract["base_image"],
            "image_tag": self.contract["image_tag"],
            "image_id": self.contract["image_id"],
            "rootfs_tree_manifest": self.contract["rootfs_tree_manifest"],
        }
        if name in MODULE_AUTHORITY_BENCHMARKS:
            plan["soma_insula_authority"] = "scripts/insula/soma_insula.py"
        return plan

    def _write_child_outputs(self):
        for name in BENCHMARKS:
            child = self.out / name
            child.mkdir(parents=True)
            plan = self._matching_plan(name)
            (child / "insula-plan.json").write_text(
                json.dumps(plan) + "\n",
                encoding="utf-8",
            )
            (child / "summary.json").write_text(
                json.dumps(
                    {
                        "benchmark": name,
                        "total_scenarios": 1,
                        "scenarios_as_expected": 1,
                        "insula_checks_passed": True,
                        "plan": plan,
                    }
                )
                + "\n",
                encoding="utf-8",
            )

    def _run(self):
        return subprocess.run(
            [
                sys.executable,
                str(AGGREGATOR),
                str(self.out),
                self.run_id,
                "--store-root",
                str(self.store_root),
            ],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )

    def test_aggregate_passes_when_children_match_selected_rootfs(self):
        result = self._run()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)

        aggregate = json.loads((self.out / "summary.json").read_text(encoding="utf-8"))
        self.assertTrue(aggregate["passed"])
        self.assertEqual(aggregate["benchmarks_passed"], len(BENCHMARKS))
        self.assertEqual(aggregate["current_rootfs"]["store_id"], self.store_id)
        for record in aggregate["records"]:
            self.assertTrue(record["current_selection_checks_passed"])
            self.assertEqual(record["current_selection_errors"], [])

    def test_aggregate_rejects_child_plan_with_stale_image_identity(self):
        plan_path = self.out / "jupyter" / "insula-plan.json"
        plan = json.loads(plan_path.read_text(encoding="utf-8"))
        plan["image_id"] = "sha256:stale"
        plan_path.write_text(json.dumps(plan) + "\n", encoding="utf-8")

        result = self._run()
        self.assertNotEqual(result.returncode, 0, result.stderr + result.stdout)

        aggregate = json.loads((self.out / "summary.json").read_text(encoding="utf-8"))
        self.assertFalse(aggregate["passed"])
        stale = next(record for record in aggregate["records"] if record["name"] == "jupyter")
        self.assertFalse(stale["current_selection_checks_passed"])
        self.assertTrue(
            any("insula-plan.json:image_id" in error for error in stale["current_selection_errors"])
        )

    def test_embedded_summary_plan_is_optional(self):
        summary_path = self.out / "bridge" / "summary.json"
        summary = json.loads(summary_path.read_text(encoding="utf-8"))
        del summary["plan"]
        summary_path.write_text(json.dumps(summary) + "\n", encoding="utf-8")

        result = self._run()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)

        aggregate = json.loads((self.out / "summary.json").read_text(encoding="utf-8"))
        bridge = next(record for record in aggregate["records"] if record["name"] == "bridge")
        self.assertTrue(bridge["current_selection_checks_passed"])

    def test_aggregate_rejects_child_plan_with_stale_rootfs_tree_manifest(self):
        plan_path = self.out / "lsp" / "insula-plan.json"
        plan = json.loads(plan_path.read_text(encoding="utf-8"))
        plan["rootfs_tree_manifest"] = {
            "kind": "soma_rootfs_tree_manifest_v1",
            "tree_hash_sha256": "stale-tree",
            "manifest_sha256": "manifest-sha",
        }
        plan_path.write_text(json.dumps(plan) + "\n", encoding="utf-8")

        result = self._run()
        self.assertNotEqual(result.returncode, 0, result.stderr + result.stdout)

        aggregate = json.loads((self.out / "summary.json").read_text(encoding="utf-8"))
        lsp = next(record for record in aggregate["records"] if record["name"] == "lsp")
        self.assertFalse(lsp["current_selection_checks_passed"])
        self.assertTrue(
            any(
                "insula-plan.json:rootfs_tree_manifest" in error
                for error in lsp["current_selection_errors"]
            )
        )

    def test_aggregate_rejects_module_batch_plan_without_soma_authority(self):
        plan_path = self.out / "jupyter" / "insula-plan.json"
        plan = json.loads(plan_path.read_text(encoding="utf-8"))
        del plan["soma_insula_authority"]
        plan_path.write_text(json.dumps(plan) + "\n", encoding="utf-8")

        result = self._run()
        self.assertNotEqual(result.returncode, 0, result.stderr + result.stdout)

        aggregate = json.loads((self.out / "summary.json").read_text(encoding="utf-8"))
        jupyter = next(record for record in aggregate["records"] if record["name"] == "jupyter")
        self.assertFalse(jupyter["current_selection_checks_passed"])
        self.assertTrue(
            any(
                "insula-plan.json:soma_insula_authority" in error
                for error in jupyter["current_selection_errors"]
            )
        )

    def test_aggregate_rejects_daemon_plan_without_soma_authority(self):
        plan_path = self.out / "bridge-daemon" / "insula-plan.json"
        plan = json.loads(plan_path.read_text(encoding="utf-8"))
        del plan["soma_insula_authority"]
        plan_path.write_text(json.dumps(plan) + "\n", encoding="utf-8")

        result = self._run()
        self.assertNotEqual(result.returncode, 0, result.stderr + result.stdout)

        aggregate = json.loads((self.out / "summary.json").read_text(encoding="utf-8"))
        daemon = next(record for record in aggregate["records"] if record["name"] == "bridge-daemon")
        self.assertFalse(daemon["current_selection_checks_passed"])
        self.assertTrue(
            any(
                "insula-plan.json:soma_insula_authority" in error
                for error in daemon["current_selection_errors"]
            )
        )


if __name__ == "__main__":
    unittest.main()

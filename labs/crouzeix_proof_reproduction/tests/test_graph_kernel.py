import unittest

from labs.crouzeix_proof_reproduction import graph_kernel


class GraphKernelTest(unittest.TestCase):
    def test_topological_order_is_deterministic(self):
        dependencies = {"c": ("b",), "a": (), "b": ("a",)}

        self.assertEqual(
            graph_kernel.topological_order(dependencies),
            ("a", "b", "c"),
        )

    def test_external_dependencies_are_not_emitted(self):
        dependencies = {"child": ("route-parent",)}

        self.assertEqual(
            graph_kernel.topological_order(
                dependencies, external_ids={"route-parent"}
            ),
            ("child",),
        )

    def test_unknown_dependency_is_rejected(self):
        with self.assertRaisesRegex(
            graph_kernel.GraphKernelError, "unknown dependency: missing"
        ):
            graph_kernel.topological_order({"child": ("missing",)})

    def test_cycle_is_typed(self):
        with self.assertRaisesRegex(graph_kernel.GraphKernelError, "cycle"):
            graph_kernel.topological_order({"a": ("b",), "b": ("a",)})

    def test_closure_is_dependency_first_and_unique(self):
        dependencies = {"root": ("b", "a"), "a": (), "b": ("a",)}

        self.assertEqual(
            graph_kernel.dependency_closure(
                ("root",), dependencies, excluded={"root"}
            ),
            ("a", "b"),
        )

    def test_closure_rejects_unknown_root(self):
        with self.assertRaisesRegex(
            graph_kernel.GraphKernelError, "unknown root: missing"
        ):
            graph_kernel.dependency_closure(("missing",), {"known": ()})


if __name__ == "__main__":
    unittest.main()

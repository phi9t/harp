from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import provider_independence  # noqa: E402


REPOSITORY_ROOT = LAB.parents[1]
LEAN_ROOT = REPOSITORY_ROOT / "formalization/lean"
LS_ROOTS = (
    "Crouzeix.LoristSchwenninger.MainTheorem",
    "Crouzeix.LoristSchwenninger.Consequences",
)
HARP_ROOTS = (
    "Crouzeix.Harp.FiniteHorizonPerturbation",
    "Crouzeix.Harp.FiniteAtomicL2Dilation",
    "Crouzeix.Harp.MainTheorem",
    "Crouzeix.Harp.Consequences",
)
LEGACY_FORBIDDEN = frozenset(
    {
        "Crouzeix.Jin.Terminal",
        "CrouzeixConjecture.FinalTheorems",
        "CrouzeixConjecture.RadialOuterReduction",
        "CrouzeixConjecture.HilbertSpace",
        "CrouzeixConjecture.HilbertSpectralSet",
    }
)


def write_module(lean_root: Path, module: str, source: str) -> None:
    path = lean_root.joinpath(*module.split(".")).with_suffix(".lean")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(source, encoding="utf-8")


class ActiveImportParserTests(unittest.TestCase):
    def test_accepts_single_prelude_after_optional_module_before_imports(self) -> None:
        sources = (
            "prelude\nimport Alpha.One\nnamespace Fixture\n",
            (
                "module\n"
                "-- the prelude header may be separated by comments\n"
                "prelude\n"
                "public import Alpha.One\n"
                "namespace Fixture\n"
            ),
        )

        for source in sources:
            with self.subTest(source=source):
                self.assertEqual(
                    provider_independence.parse_active_imports(source, "Fixture.Root"),
                    ("Alpha.One",),
                )

    def test_accepts_optional_module_and_plain_or_public_imports(self) -> None:
        source = """
module

public   import Alpha.One
public	import Alpha.Tabbed
import Alpha.Two Beta.Three

namespace Fixture
"""

        parsed = provider_independence.parse_active_imports(source, "Fixture.Root")

        self.assertEqual(
            parsed,
            ("Alpha.One", "Alpha.Tabbed", "Alpha.Two", "Beta.Three"),
        )

    def test_public_section_terminates_import_header_after_public_import(self) -> None:
        source = "module\npublic import Mathlib.X\npublic section\nnamespace Fixture\n"

        parsed = provider_independence.parse_active_imports(source, "Fixture.Root")

        self.assertEqual(parsed, ("Mathlib.X",))

    def test_ignores_imports_in_line_and_nested_block_comments(self) -> None:
        source = """
-- import Hidden.Line
/- import Hidden.Block
   /- public import Hidden.Nested -/
-/
import Visible.Direct -- import Hidden.Trailing
/- comment -/ public import Visible.AfterComment

namespace Fixture
-- import Hidden.Late
"""

        parsed = provider_independence.parse_active_imports(source, "Fixture.Root")

        self.assertEqual(parsed, ("Visible.Direct", "Visible.AfterComment"))

    def test_line_comments_end_at_cr_or_crlf(self) -> None:
        for newline in ("\r", "\r\n"):
            with self.subTest(newline=repr(newline)):
                source = (
                    f"-- import Hidden.Comment{newline}"
                    f"import Visible.Direct{newline}"
                    f"namespace Fixture{newline}"
                )
                self.assertEqual(
                    provider_independence.parse_active_imports(source, "Fixture.Root"),
                    ("Visible.Direct",),
                )

    def test_import_parsing_stops_at_first_non_header_command(self) -> None:
        source = """
import Visible.Header

namespace Fixture
import Hidden.Late
"""

        self.assertEqual(
            provider_independence.parse_active_imports(source, "Fixture.Root"),
            ("Visible.Header",),
        )
        self.assertEqual(
            provider_independence.parse_active_imports(
                "moduleName := 1\nimport Hidden.Late\n", "Fixture.Root"
            ),
            (),
        )

    def test_rejects_malformed_import_like_command(self) -> None:
        for command in (
            "import",
            "import Good.Module, Bad.Module",
            "public import",
            "public nope",
        ):
            with (
                self.subTest(command=command),
                self.assertRaisesRegex(
                    provider_independence.ProviderIndependenceError,
                    "malformed import.*Fixture.Root",
                ),
            ):
                provider_independence.parse_active_imports(
                    f"{command}\nnamespace Fixture\n", "Fixture.Root"
                )

    def test_rejects_malformed_or_repeated_module_command(self) -> None:
        for source in (
            "module Extra\nimport Alpha.One\n",
            "module\nmodule\nimport Alpha.One\n",
            "import Alpha.One\nmodule\n",
        ):
            with (
                self.subTest(source=source),
                self.assertRaisesRegex(
                    provider_independence.ProviderIndependenceError,
                    "malformed module.*Fixture.Root",
                ),
            ):
                provider_independence.parse_active_imports(source, "Fixture.Root")

    def test_rejects_malformed_repeated_or_late_prelude(self) -> None:
        for source in (
            "prelude Extra\nimport Alpha.One\n",
            "prelude\nprelude\nimport Alpha.One\n",
            "import Alpha.One\nprelude\nimport Hidden.Late\n",
        ):
            with (
                self.subTest(source=source),
                self.assertRaisesRegex(
                    provider_independence.ProviderIndependenceError,
                    "malformed prelude.*Fixture.Root",
                ),
            ):
                provider_independence.parse_active_imports(source, "Fixture.Root")

    def test_rejects_module_after_prelude(self) -> None:
        with self.assertRaisesRegex(
            provider_independence.ProviderIndependenceError,
            "malformed module.*Fixture.Root",
        ):
            provider_independence.parse_active_imports(
                "prelude\nmodule\nimport Alpha.One\n", "Fixture.Root"
            )

    def test_rejects_unmatched_block_comment_close(self) -> None:
        with self.assertRaisesRegex(
            provider_independence.ProviderIndependenceError,
            "unmatched block comment close.*Fixture.Root",
        ):
            provider_independence.parse_active_imports(
                "import Visible\n-/\n", "Fixture.Root"
            )

    def test_rejects_unterminated_block_comment(self) -> None:
        with self.assertRaisesRegex(
            provider_independence.ProviderIndependenceError,
            "unterminated block comment.*Fixture.Root",
        ):
            provider_independence.parse_active_imports(
                "import Visible\n/- never closed\n", "Fixture.Root"
            )


class ActiveSourceScannerTests(unittest.TestCase):
    def test_reports_danger_tokens_but_ignores_comments_and_identifiers(self) -> None:
        source = """
-- sorry admit unsafe native_decide implemented_by
/- axiom Hidden : Prop -/
theorem sorryNotToken : True := by trivial
theorem one : True := by sorry
theorem two : True := by admit
axiom exposed : Prop
unsafe def risky := 1
example : True := by native_decide
def impl := 1
  implemented_by impl
"""

        findings = provider_independence.scan_active_source(source, "Fixture.Root")

        self.assertEqual(
            tuple((finding.token, finding.line) for finding in findings.dangers),
            (
                ("sorry", 5),
                ("admit", 6),
                ("axiom", 7),
                ("unsafe", 8),
                ("native_decide", 9),
                ("implemented_by", 11),
            ),
        )

    def test_axiom_is_dangerous_only_at_line_start(self) -> None:
        source = """
def mentions := "axiom"
example : True := by
  have axiomValue : True := True.intro
  exact axiomValue
  axiom indentedButCommand : Prop
"""

        findings = provider_independence.scan_active_source(source, "Fixture.Root")

        self.assertEqual(
            tuple((finding.token, finding.line) for finding in findings.dangers),
            (("axiom", 6),),
        )

    def test_set_option_is_report_only(self) -> None:
        source = """
set_option maxHeartbeats 800000
-- set_option pp.universes true
example : True := by trivial
"""

        findings = provider_independence.scan_active_source(source, "Fixture.Root")

        self.assertEqual(findings.dangers, ())
        self.assertEqual(len(findings.set_options), 1)
        self.assertEqual(findings.set_options[0].line, 2)
        self.assertEqual(
            findings.set_options[0].source_line, "set_option maxHeartbeats 800000"
        )

    def test_ignores_tokens_and_comment_delimiters_inside_literals(self) -> None:
        source = r"""
def ordinary := "sorry admit axiom unsafe native_decide implemented_by set_option /- --"
def raw := r###"sorry set_option /- -- \""###
def slash := "/-"
theorem stillVisible : True := by sorry
def dash := "--"
set_option maxRecDepth 2000
def close := "-/"
"""

        findings = provider_independence.scan_active_source(source, "Fixture.Root")

        self.assertEqual(
            tuple((finding.token, finding.line) for finding in findings.dangers),
            (("sorry", 5),),
        )
        self.assertEqual(
            tuple((finding.token, finding.line) for finding in findings.set_options),
            (("set_option", 7),),
        )

    def test_scans_active_terms_inside_interpolated_strings(self) -> None:
        source = """
def simple : String := s!"literal sorry {(by exact sorry : String)}"
def nested : String :=
  s!"outer {s!"inner {(by exact admit : String)}"}"
def braces : String := s!"{({ value := (by exact unsafe : String) }).value}"
"""

        findings = provider_independence.scan_active_source(source, "Fixture.Root")

        self.assertEqual(
            tuple((finding.token, finding.line) for finding in findings.dangers),
            (("sorry", 2), ("admit", 4), ("unsafe", 5)),
        )

    def test_raw_string_marker_must_start_a_token(self) -> None:
        source = """
def sorryr (value : String) := value
#check sorryr"literal /- set_option"
"""

        findings = provider_independence.scan_active_source(source, "Fixture.Root")

        self.assertEqual(findings.dangers, ())
        self.assertEqual(findings.set_options, ())

    def test_ignores_danger_words_inside_unicode_and_escaped_identifiers(self) -> None:
        source = """
def αsorry := 1
def «sorry» := 2
def unsafeValue := 3
axiomα := 4
axiom! := 5
def «axiom» := 6
"""

        findings = provider_independence.scan_active_source(source, "Fixture.Root")

        self.assertEqual(findings.dangers, ())

    def test_escaped_interpolation_braces_are_literal(self) -> None:
        source = """
def literal : String := s!"{{sorry and admit}}"
def mixed : String := s!"{{unsafe}} {(by exact sorry : String)}"
"""

        findings = provider_independence.scan_active_source(source, "Fixture.Root")

        self.assertEqual(
            tuple((finding.token, finding.line) for finding in findings.dangers),
            (("sorry", 3),),
        )

    def test_rejects_unterminated_string_char_and_raw_string_literals(self) -> None:
        cases = (
            ('def value := "open', "unterminated string literal"),
            ("def value := '\n", "unterminated character literal"),
            ('def value := r##"open"#', "unterminated raw string literal"),
        )
        for source, message in cases:
            with (
                self.subTest(source=source),
                self.assertRaisesRegex(
                    provider_independence.ProviderIndependenceError,
                    f"{message}.*Fixture.Root",
                ),
            ):
                provider_independence.scan_active_source(source, "Fixture.Root")

    def test_scanner_rejects_unterminated_block_comment(self) -> None:
        with self.assertRaisesRegex(
            provider_independence.ProviderIndependenceError,
            "unterminated block comment.*Fixture.Root",
        ):
            provider_independence.scan_active_source(
                "theorem safe : True := by trivial\n/- open", "Fixture.Root"
            )

    def test_scans_after_line_comment_with_cr_or_crlf(self) -> None:
        for newline in ("\r", "\r\n"):
            with self.subTest(newline=repr(newline)):
                findings = provider_independence.scan_active_source(
                    f"-- hidden sorry{newline}example : True := by sorry{newline}",
                    "Fixture.Root",
                )
                self.assertEqual(
                    tuple(
                        (finding.token, finding.line) for finding in findings.dangers
                    ),
                    (("sorry", 2),),
                )


class ProviderIndependencePolicyTests(unittest.TestCase):
    def audit(
        self,
        modules: dict[str, str],
        roots: tuple[str, ...] = ("Route.Main",),
        forbidden_modules: frozenset[str] = frozenset(),
        forbidden_prefixes: tuple[str, ...] = (),
    ) -> provider_independence.ProviderIndependenceReport:
        with tempfile.TemporaryDirectory() as directory:
            lean_root = Path(directory)
            for module, source in modules.items():
                write_module(lean_root, module, source)
            return provider_independence.audit_provider_independence(
                lean_root,
                roots,
                forbidden_modules,
                forbidden_prefixes=forbidden_prefixes,
            )

    def test_computes_transitive_local_closure_from_explicit_roots(self) -> None:
        report = self.audit(
            {
                "Route.Main": "import Route.Middle External.Library\n",
                "Route.Middle": "public import Route.Leaf\n",
                "Route.Leaf": "theorem safe : True := by trivial\n",
                "Route.Unrelated": "theorem unused : True := by sorry\n",
            }
        )

        self.assertEqual(report.modules, ("Route.Leaf", "Route.Main", "Route.Middle"))
        self.assertEqual(report.set_options, ())

    def test_rejects_unknown_root(self) -> None:
        with self.assertRaisesRegex(
            provider_independence.ProviderIndependenceError,
            "root module does not exist: Missing.Root",
        ):
            self.audit({"Route.Main": ""}, roots=("Missing.Root",))

    def test_rejects_forbidden_root(self) -> None:
        with self.assertRaisesRegex(
            provider_independence.ProviderIndependenceError,
            "forbidden root module: Route.Main",
        ):
            self.audit(
                {"Route.Main": ""},
                forbidden_modules=frozenset({"Route.Main"}),
            )

    def test_rejects_direct_forbidden_import(self) -> None:
        with self.assertRaisesRegex(
            provider_independence.ProviderIndependenceError,
            "forbidden module.*Provider.Terminal.*Route.Main",
        ):
            self.audit(
                {
                    "Route.Main": "import Provider.Terminal\n",
                    "Provider.Terminal": "",
                },
                forbidden_modules=frozenset({"Provider.Terminal"}),
            )

    def test_rejects_forbidden_import_after_prelude(self) -> None:
        with self.assertRaisesRegex(
            provider_independence.ProviderIndependenceError,
            "forbidden module.*Provider.Terminal.*Route.Main",
        ):
            self.audit(
                {
                    "Route.Main": "prelude\nimport Provider.Terminal\n",
                    "Provider.Terminal": "",
                },
                forbidden_modules=frozenset({"Provider.Terminal"}),
            )

    def test_rejects_transitive_forbidden_import(self) -> None:
        with self.assertRaisesRegex(
            provider_independence.ProviderIndependenceError,
            "forbidden module.*Provider.Terminal.*Route.Middle",
        ):
            self.audit(
                {
                    "Route.Main": "import Route.Middle\n",
                    "Route.Middle": "import Provider.Terminal\n",
                    "Provider.Terminal": "",
                },
                forbidden_modules=frozenset({"Provider.Terminal"}),
            )

    def test_exact_forbidden_check_allows_similarly_named_module(self) -> None:
        report = self.audit(
            {
                "Route.Main": "import Provider.TerminalHelper\n",
                "Provider.TerminalHelper": "",
            },
            forbidden_modules=frozenset({"Provider.Terminal"}),
        )

        self.assertIn("Provider.TerminalHelper", report.modules)

    def test_forbidden_prefix_rejects_namespace_but_not_similar_prefix(self) -> None:
        with self.assertRaisesRegex(
            provider_independence.ProviderIndependenceError,
            "forbidden module.*Crouzeix.Jin.Support",
        ):
            self.audit(
                {
                    "Route.Main": "import Crouzeix.Jin.Support\n",
                    "Crouzeix.Jin.Support": "",
                },
                forbidden_prefixes=("Crouzeix.Jin",),
            )

        report = self.audit(
            {
                "Route.Main": "import Crouzeix.Jingle.Support\n",
                "Crouzeix.Jingle.Support": "",
            },
            forbidden_prefixes=("Crouzeix.Jin",),
        )
        self.assertIn("Crouzeix.Jingle.Support", report.modules)

    def test_rejects_missing_transitive_managed_local_import(self) -> None:
        with self.assertRaisesRegex(
            provider_independence.ProviderIndependenceError,
            "local module does not exist: CrouzeixConjecture.Missing",
        ):
            self.audit(
                {
                    "Route.Main": "import Route.Middle\n",
                    "Route.Middle": "import CrouzeixConjecture.Missing\n",
                }
            )

    def test_rejects_danger_token_anywhere_in_transitive_closure(self) -> None:
        with self.assertRaisesRegex(
            provider_independence.ProviderIndependenceError,
            "danger token sorry.*Route.Leaf:1",
        ):
            self.audit(
                {
                    "Route.Main": "import Route.Leaf\n",
                    "Route.Leaf": "theorem unfinished : True := by sorry\n",
                }
            )

    def test_report_is_deterministic_and_contains_set_options(self) -> None:
        report = self.audit(
            {
                "Route.Zed": "set_option maxRecDepth 2000\n",
                "Route.Main": (
                    "import Route.Zed Route.Alpha\n"
                    "set_option maxHeartbeats 800000 in\n"
                    "theorem safe : True := by trivial\n"
                ),
                "Route.Alpha": "set_option pp.universes true\n",
            }
        )

        self.assertEqual(report.modules, ("Route.Alpha", "Route.Main", "Route.Zed"))
        self.assertEqual(
            tuple((item.module, item.line) for item in report.set_options),
            (("Route.Alpha", 1), ("Route.Main", 2), ("Route.Zed", 1)),
        )

    def test_rejects_root_beneath_symlinked_namespace_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            workspace = Path(directory)
            lean_root = workspace / "lean"
            external = workspace / "external"
            write_module(external, "Route.Main", "")
            lean_root.mkdir()
            (lean_root / "Route").symlink_to(
                external / "Route", target_is_directory=True
            )

            with self.assertRaisesRegex(
                provider_independence.ProviderIndependenceError,
                "symlink.*Route.Main",
            ):
                provider_independence.audit_provider_independence(
                    lean_root, ("Route.Main",), frozenset()
                )

    def test_rejects_import_beneath_symlinked_namespace_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            workspace = Path(directory)
            lean_root = workspace / "lean"
            external = workspace / "external"
            write_module(lean_root, "Route.Main", "import Escaped.Leaf\n")
            write_module(external, "Escaped.Leaf", "")
            (lean_root / "Escaped").symlink_to(
                external / "Escaped", target_is_directory=True
            )

            with self.assertRaisesRegex(
                provider_independence.ProviderIndependenceError,
                "symlink.*Escaped.Leaf",
            ):
                provider_independence.audit_provider_independence(
                    lean_root, ("Route.Main",), frozenset()
                )

    def test_rejects_symlinked_lean_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            workspace = Path(directory)
            external = workspace / "external"
            write_module(external, "Route.Main", "")
            lean_root = workspace / "lean"
            lean_root.symlink_to(external, target_is_directory=True)

            with self.assertRaisesRegex(
                provider_independence.ProviderIndependenceError,
                "Lean source root cannot be a symlink",
            ):
                provider_independence.audit_provider_independence(
                    lean_root, ("Route.Main",), frozenset()
                )


class RepositoryProviderIndependenceTests(unittest.TestCase):
    def test_repository_audits_cover_full_terminal_surfaces(self) -> None:
        self.assertEqual(
            LS_ROOTS,
            (
                "Crouzeix.LoristSchwenninger.MainTheorem",
                "Crouzeix.LoristSchwenninger.Consequences",
            ),
        )
        self.assertEqual(
            HARP_ROOTS,
            (
                "Crouzeix.Harp.FiniteHorizonPerturbation",
                "Crouzeix.Harp.FiniteAtomicL2Dilation",
                "Crouzeix.Harp.MainTheorem",
                "Crouzeix.Harp.Consequences",
            ),
        )

    def test_current_lorist_schwenninger_terminal_modules_are_provider_independent(
        self,
    ) -> None:
        report = provider_independence.audit_provider_independence(
            LEAN_ROOT,
            LS_ROOTS,
            LEGACY_FORBIDDEN | frozenset(HARP_ROOTS),
            forbidden_prefixes=("Crouzeix.Jin",),
        )

        self.assertTrue(set(LS_ROOTS).issubset(report.modules))
        self.assertNotIn("CrouzeixConjecture.HilbertSpace", report.modules)
        self.assertIn("CrouzeixConjecture.HilbertSpaceCore", report.modules)

    def test_current_harp_reconstruction_modules_are_provider_independent(
        self,
    ) -> None:
        report = provider_independence.audit_provider_independence(
            LEAN_ROOT,
            HARP_ROOTS,
            LEGACY_FORBIDDEN | frozenset(LS_ROOTS),
            forbidden_prefixes=("Crouzeix.Jin",),
        )

        self.assertTrue(set(HARP_ROOTS).issubset(report.modules))
        self.assertNotIn("Crouzeix.LoristSchwenninger.MainTheorem", report.modules)
        self.assertIn("Crouzeix.LoristSchwenninger.Scalar", report.modules)
        self.assertEqual(
            tuple(
                (item.module, item.line, item.source_line)
                for item in report.set_options
            ),
            (
                (
                    "Crouzeix.LoristSchwenninger.BoundaryEmbedding",
                    18,
                    "set_option maxHeartbeats 800000",
                ),
            ),
        )


if __name__ == "__main__":
    unittest.main()

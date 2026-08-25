"""Dependency-neutral Lorist--Schwenninger route contract."""

from __future__ import annotations

from dataclasses import dataclass


SCHEMA_VERSION = "crouzeix-ls-source-graph/v1"
SOURCE_ID = "LS-ARXIV-V1"
SOURCE_IDENTITY = "arxiv:2608.03841v1"
ROUTE_ID = "lorist-schwenninger"
AGGREGATE_MODULE = "CrouzeixLoristSchwenninger"
ALLOWED_AXIOMS = ("Classical.choice", "Quot.sound", "propext")
LS_PROVIDER_FORBIDDEN_PREFIXES = ("Crouzeix.Harp", "Crouzeix.Jin")
HARP_PROVIDER_FORBIDDEN_PREFIXES = ("Crouzeix.Jin",)
FORBIDDEN_PREFIXES = LS_PROVIDER_FORBIDDEN_PREFIXES
FORBIDDEN_MODULES = (
    "CrouzeixConjecture.MainPerturbationReduction",
    "CrouzeixConjecture.CompletionStatement",
    "CrouzeixConjecture.CompletionDiagonalization",
    "CrouzeixConjecture.PositiveRealCompletion",
    "CrouzeixConjecture.FinalTheorems",
    "CrouzeixConjecture.HilbertSpace",
    "CrouzeixConjecture.HilbertSpectralSet",
    "CrouzeixConjecture.RadialOuterReduction",
)


def provider_policy(
    route_id: str,
    forbidden_modules: tuple[str, ...] = (),
) -> tuple[tuple[str, ...], tuple[str, ...]]:
    if route_id == ROUTE_ID:
        return (
            tuple(sorted({*FORBIDDEN_MODULES, *forbidden_modules})),
            FORBIDDEN_PREFIXES,
        )
    if route_id == "harp":
        return (
            tuple(sorted(set(forbidden_modules))),
            HARP_PROVIDER_FORBIDDEN_PREFIXES,
        )
    raise ValueError(f"unknown provider policy route: {route_id}")


@dataclass(frozen=True)
class LSNodeContract:
    node_id: str
    declaration: str
    build_target: str
    dependencies: tuple[str, ...]
    legacy_dependencies: tuple[str, ...]
    role: str
    source_locator: str
    statement_sha256: str
    legacy_graph_lean_name: str | None = None
    legacy_status: str | None = None
    legacy_receipt_sha256: str | None = None
    legacy_blocked_reason: str | None = None
    legacy_build_target: str | None = None

    @property
    def accepted_graph_lean_names(self) -> frozenset[str]:
        return frozenset(
            name
            for name in (self.declaration, self.legacy_graph_lean_name)
            if name is not None
        )


NODES = (
    LSNodeContract(
        node_id="ls-equation-one-terminal-bound",
        declaration=(
            "CrouzeixConjecture.LoristSchwenninger.DilationData."
            "perturbation_mul_target_power_norm_le"
        ),
        build_target=("formalization/lean/Crouzeix/LoristSchwenninger/Dilation.lean"),
        dependencies=(),
        legacy_dependencies=(),
        role="intermediate",
        source_locator=("arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99"),
        statement_sha256=(
            "f560ccaca9499f62c8cc30fe1a93338d1024101e129b439fbc880df29cb23311"
        ),
        legacy_graph_lean_name=(
            "CrouzeixConjecture.LoristSchwenninger."
            "perturbation_mul_target_power_norm_le"
        ),
        legacy_status="passed",
        legacy_receipt_sha256=(
            "e596548fe22a946e5bef0d5f942a99d64da3579c76dbd5afd0a4e46de33317cb"
        ),
        legacy_build_target=(
            "formalization/lean/Crouzeix/LoristSchwenninger/Perturbation.lean"
        ),
    ),
    LSNodeContract(
        node_id="ls-power-recurrence",
        declaration=(
            "CrouzeixConjecture.LoristSchwenninger.DilationData."
            "equation_three_lower_bound"
        ),
        build_target=(
            "formalization/lean/Crouzeix/LoristSchwenninger/OperatorRecurrence.lean"
        ),
        dependencies=("ls-equation-one-terminal-bound",),
        legacy_dependencies=("ls-equation-one-terminal-bound",),
        role="intermediate",
        source_locator=("arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L74-L90"),
        statement_sha256=(
            "0b80e8d8bf6e2f5f9d3e3c0fb7e3d2e3f9c9ec7b5ecb392a6d211c7ef7ac0f3d"
        ),
        legacy_graph_lean_name=(
            "CrouzeixConjecture.LoristSchwenninger.PowerRecurrenceStatement"
        ),
        legacy_status="blocked",
        legacy_blocked_reason=(
            "Operator recurrence construction remains to be formalized: derive "
            "the lower bound on Re <E_1 T x, x> from the commuting perturbation "
            "family and terminal product boundedness."
        ),
    ),
    LSNodeContract(
        node_id="ls-scalar-contradiction",
        declaration=("CrouzeixConjecture.LoristSchwenninger.scalar_endpoint_le_two"),
        build_target=("formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean"),
        dependencies=(),
        legacy_dependencies=(),
        role="intermediate",
        source_locator=("arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98"),
        statement_sha256=(
            "966cabe377c00b287b67e3a96de1ab53a12b8c4d35d130c26ae2bc899557f9fa"
        ),
        legacy_status="passed",
        legacy_receipt_sha256=(
            "ad13640a3a8676aa6dd7a2bfdb9f5c1975a0fdbee037cf0555767c8e40549781"
        ),
        legacy_build_target=(
            "formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean"
        ),
    ),
    LSNodeContract(
        node_id="ls-perturbation-lemma",
        declaration=(
            "CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two"
        ),
        build_target=(
            "formalization/lean/Crouzeix/LoristSchwenninger/PerturbationLemma.lean"
        ),
        dependencies=("ls-power-recurrence", "ls-scalar-contradiction"),
        legacy_dependencies=("ls-power-recurrence", "ls-scalar-contradiction"),
        role="intermediate",
        source_locator=("arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99"),
        statement_sha256=(
            "0d38ea668014d371f14b3ffb7d5e5bdce7417a2aef2cfd81f663257e72d79d86"
        ),
        legacy_graph_lean_name=(
            "CrouzeixConjecture.LoristSchwenninger.PerturbationLemma"
        ),
        legacy_status="blocked",
        legacy_blocked_reason=(
            "Full LS perturbation lemma awaits the scalar recurrence proof over "
            "a top singular vector."
        ),
    ),
    LSNodeContract(
        node_id="ls-double-layer-realization",
        declaration=(
            "CrouzeixConjecture.LoristSchwenninger."
            "norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary"
        ),
        build_target=(
            "formalization/lean/Crouzeix/LoristSchwenninger/ConcreteDilation.lean"
        ),
        dependencies=("ls-perturbation-lemma",),
        legacy_dependencies=("ls-perturbation-lemma",),
        role="intermediate",
        source_locator=("arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124"),
        statement_sha256=(
            "be32b0c50689d78d037f7359bcd03ab51a0e8ef06b8c5e64d533fbdb2cfaa280"
        ),
        legacy_graph_lean_name=(
            "CrouzeixConjecture.LoristSchwenninger.DoubleLayerRealization"
        ),
        legacy_status="blocked",
        legacy_blocked_reason=(
            "Double-layer realization remains to be formalized: construct the LS "
            "application data from the positive boundary density, multiplication "
            "by f, and the functional calculus of alpha(f^n)(A)."
        ),
    ),
    LSNodeContract(
        node_id="ls-terminal-crouzeix",
        declaration="CrouzeixConjecture.loristSchwenningerMainTheorem",
        build_target=(
            "formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean"
        ),
        dependencies=("ls-double-layer-realization",),
        legacy_dependencies=(
            "ls-perturbation-lemma",
            "ls-double-layer-realization",
        ),
        role="terminal",
        source_locator=("arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128"),
        statement_sha256=(
            "3a53ccdfc2b6639917cdd774c8d9a2293d690c5c2ef4cf02657e0fd752544a67"
        ),
        legacy_graph_lean_name=(
            "CrouzeixConjecture.LoristSchwenninger.CrouzeixTerminal"
        ),
        legacy_status="blocked",
        legacy_blocked_reason=(
            "Terminal LS theorem awaits the full perturbation lemma and "
            "double-layer realization."
        ),
    ),
)
BY_ID = {node.node_id: node for node in NODES}
NODE_ORDER = tuple(node.node_id for node in NODES)

_BODY_AUDIT_PREFIX = "formalization/lean/Crouzeix/LoristSchwenninger"
THEOREM_BODY_AUDITS: dict[str, dict[str, object]] = {
    "ls-power-recurrence": {
        "path": f"{_BODY_AUDIT_PREFIX}/OperatorRecurrence.lean",
        "theorem": "equation_three_lower_bound",
        "required_calls": (
            "recurrence_lower_bound",
            "recurrence_difference_lower_bound",
        ),
        "forbidden_calls": (
            "jinMainTheorem",
            "harpFiniteHorizonMainTheorem",
        ),
    },
    "ls-perturbation-lemma": {
        "path": f"{_BODY_AUDIT_PREFIX}/PerturbationLemma.lean",
        "theorem": "norm_target_le_two",
        "required_calls": (
            "equation_three_lower_bound",
            "displacementSq_le",
            "scalar_endpoint_le_two",
        ),
        "forbidden_calls": (
            "jinMainTheorem",
            "harpFiniteHorizonMainTheorem",
        ),
    },
    "ls-double-layer-realization": {
        "path": f"{_BODY_AUDIT_PREFIX}/ConcreteDilation.lean",
        "theorem": "norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary",
        "required_calls": (
            "dilationDataOfParametricPolynomial",
            "norm_target_le_two",
        ),
        "forbidden_calls": (
            "jinMainTheorem",
            "harpFiniteHorizonMainTheorem",
        ),
    },
    "ls-terminal-crouzeix": {
        "path": f"{_BODY_AUDIT_PREFIX}/MainTheorem.lean",
        "theorem": "loristSchwenningerMainTheorem",
        "required_calls": (
            "norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary",
            "norm_polynomialEval_le_of_tendsto",
            "tendsto_maxPolynomialModulusOnSet_of_outerApproximation",
        ),
        "forbidden_calls": (
            "jinMainTheorem",
            "harpFiniteHorizonMainTheorem",
            "loristSchwenningerFiniteMatrixMainTheorem",
        ),
    },
}

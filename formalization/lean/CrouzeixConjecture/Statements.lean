module

public import CrouzeixConjecture.Definitions

@[expose] public section

noncomputable section

open scoped Matrix.Norms.L2Operator

namespace CrouzeixConjecture

variable {n : Type*} [Fintype n] [DecidableEq n]

/-- The maximum modulus appearing in the manuscript.  It is represented as the supremum of
the same set of values; compactness and nonemptiness will supply the attaining point. -/
def maxPolynomialModulusOnNumericalRange (A : SquareMatrix n) (p : Polynomial ℂ) : ℝ :=
  sSup ((fun z : ℂ ↦ ‖Polynomial.eval z p‖) '' numericalRange A)

/-- The polynomial specialization of `eq:main-bound`. -/
def PolynomialCrouzeixBound (A : SquareMatrix n) (p : Polynomial ℂ) : Prop :=
  ‖polynomialEval p A‖ ≤ 2 * maxPolynomialModulusOnNumericalRange A p

/-- The polynomial specialization of the manuscript's main theorem. `Nonempty n` records the
positive-dimension convention needed for a nonempty numerical range. -/
def MainTheoremStatement [Nonempty n] : Prop :=
  ∀ (A : SquareMatrix n) (p : Polynomial ℂ), PolynomialCrouzeixBound A p

/-- The finite-matrix theorem needed to transport the polynomial bound to Hilbert spaces.
Only standard finite index types are required because the transport uses `stdOrthonormalBasis`. -/
def FiniteMatrixMainTheoremStatement : Prop :=
  ∀ (d : ℕ) [Nonempty (Fin d)], MainTheoremStatement (n := Fin d)

end CrouzeixConjecture

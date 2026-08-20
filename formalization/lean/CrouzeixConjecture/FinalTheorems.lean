module

public import CrouzeixConjecture.HolomorphicConsequences

@[expose] public section

noncomputable section

open scoped Matrix Matrix.Norms.L2Operator

namespace CrouzeixConjecture

variable {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]

/-- The ported upstream polynomial specialization of the manuscript's main
theorem. Harp's terminal source-map declaration is assembled separately in
`Crouzeix.Jin.Terminal`. -/
theorem jinFinalCrouzeixConjecture (A : SquareMatrix n) (p : Polynomial ℂ) :
    ‖polynomialEval p A‖ ≤
      2 * maxPolynomialModulusOnNumericalRange A p :=
  polynomialCrouzeixBound_of_holomorphicCrouzeixBound A p

/-- The manuscript's rational spectral-set discussion, kept as a separate corollary.  Pole
freeness is required on exactly the numerical range, and the same constant `2` and induced
Euclidean matrix norm occur in the conclusion. -/
theorem crouzeixRationalSpectralSetCorollary :
    RationalSpectralSetCorollaryStatement (n := n) :=
  fun A r hfree ↦ holomorphicCrouzeixRationalBound A r hfree

/-- Pointwise form of the separate rational corollary. -/
theorem crouzeixRationalBound
    (A : SquareMatrix n) (r : RatFunc ℂ)
    (hfree : RationalPoleFreeOn r (numericalRange A)) :
    ‖rationalMatrixEval r A‖ ≤
      2 * maxRationalModulusOnNumericalRange A r :=
  holomorphicCrouzeixRationalBound A r hfree

end CrouzeixConjecture

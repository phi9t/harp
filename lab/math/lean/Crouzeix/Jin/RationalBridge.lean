module

public import CrouzeixConjecture.HolomorphicConsequences

@[expose] public section

noncomputable section

open scoped Matrix Matrix.Norms.L2Operator

namespace CrouzeixConjecture

variable {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]

/-- Route-local source-correspondent bridge to Jin's finite rational
specialization of the holomorphic Crouzeix bound. -/
theorem jinHolomorphicCrouzeixRationalBound
    (A : SquareMatrix n) (r : RatFunc ℂ)
    (hfree : RationalPoleFreeOn r (numericalRange A)) :
    ‖rationalMatrixEval r A‖ ≤
      2 * maxRationalModulusOnNumericalRange A r :=
  holomorphicCrouzeixRationalBound A r hfree

end CrouzeixConjecture

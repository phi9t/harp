import Mathlib.Algebra.Polynomial.Eval.Defs
import Mathlib.Analysis.Complex.Basic
import Mathlib.Data.Matrix.Mul

/- Harp-native Crouzeix route scaffolding.

This module intentionally contains no imported Jin terminal theorem. It starts
with the first source-mapped Jin interface: the maximum polynomial modulus on a
matrix numerical range.
-/

noncomputable section

namespace CrouzeixConjecture

variable {n : Type*} [Fintype n]

abbrev SquareMatrix (n : Type*) := Matrix n n ℂ

/-- Numerical range of a finite complex matrix, as the values of the quadratic
form on unit vectors. -/
def numericalRange (A : SquareMatrix n) : Set ℂ :=
  {z | ∃ x : n → ℂ, ‖x‖ = 1 ∧ z = dotProduct (star x) (A.mulVec x)}

/-- The maximum modulus appearing in Jin's manuscript and Lean source map,
represented as the supremum of polynomial moduli on the numerical range. -/
def maxPolynomialModulusOnNumericalRange (A : SquareMatrix n) (p : Polynomial ℂ) : ℝ :=
  sSup ((fun z : ℂ => ‖p.eval z‖) '' numericalRange A)

end CrouzeixConjecture

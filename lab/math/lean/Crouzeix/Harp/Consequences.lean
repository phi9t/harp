import Crouzeix.Harp.MainTheorem
import CrouzeixConjecture.RationalApproximation
import CrouzeixConjecture.HilbertSpaceCore
import CrouzeixConjecture.HilbertSpectralSetCore

/-!
Provider-clean consequences of the Harp finite-horizon finite-matrix theorem.

This module supplies the Harp theorem to the neutral finite-matrix,
rational-approximation, and Hilbert-space adapters without importing another
terminal theorem provider.
-/

noncomputable section

open scoped InnerProductSpace Matrix Matrix.Norms.L2Operator

namespace CrouzeixConjecture

/-- The Harp finite-horizon theorem uniformly supplies every standard finite
matrix dimension used by the Hilbert-space transport. -/
theorem harpFiniteHorizonFiniteMatrixMainTheorem :
    FiniteMatrixMainTheoremStatement :=
  fun d => Harp.harpFiniteHorizonMainTheorem (n := Fin d)

variable {n : Type} [Fintype n] [DecidableEq n] [Nonempty n]

/-- The finite-matrix rational spectral-set consequence of the Harp
finite-horizon polynomial theorem. -/
theorem harpFiniteHorizonRationalSpectralSetCorollary :
    RationalSpectralSetCorollaryStatement (n := n) :=
  rationalSpectralSetCorollary_of_mainTheorem
    Harp.harpFiniteHorizonMainTheorem

variable {H : Type*} [NormedAddCommGroup H] [InnerProductSpace Complex H]

/-- The Harp finite-horizon finite-matrix theorem transported to every nonzero
complex Hilbert space. -/
theorem harpFiniteHorizonHilbertSpacePolynomialCrouzeix
    [CompleteSpace H] [Nontrivial H]
    (A : H →L[Complex] H) (p : Polynomial Complex) :
    norm (operatorPolynomialEval p A) <=
      2 * supPolynomialModulusOnOperatorNumericalRange A p :=
  hilbertSpacePolynomialCrouzeix_of_mainTheorem
    harpFiniteHorizonFiniteMatrixMainTheorem A p

/-- The Hilbert-space rational spectral-set statement supplied by the Harp
finite-horizon finite-matrix theorem. -/
theorem harpFiniteHorizonHilbertSpaceRationalSpectralSet
    [CompleteSpace H] [Nontrivial H] :
    HilbertRationalSpectralSetStatement (H := H) :=
  hilbertSpaceRationalSpectralSet_of_mainTheorem
    harpFiniteHorizonFiniteMatrixMainTheorem

/-- The closed numerical range of a bounded operator is a two-spectral set,
using the Harp finite-horizon theorem as the finite-matrix provider. -/
theorem harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet
    [CompleteSpace H] [Nontrivial H] (A : H →L[Complex] H) :
    ClosedOperatorNumericalRangeIsTwoSpectralSet A :=
  closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem
    harpFiniteHorizonFiniteMatrixMainTheorem A

end CrouzeixConjecture

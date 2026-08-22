import Crouzeix.LoristSchwenninger.MainTheorem
import CrouzeixConjecture.RationalApproximation
import CrouzeixConjecture.HilbertSpaceCore
import CrouzeixConjecture.HilbertSpectralSetCore

/-!
Provider-clean consequences of the Lorist--Schwenninger finite-matrix theorem.

This module supplies the Lorist--Schwenninger theorem to the neutral finite-matrix,
rational-approximation, and Hilbert-space adapters without importing another
terminal theorem provider.
-/

noncomputable section

open scoped InnerProductSpace Matrix Matrix.Norms.L2Operator

namespace CrouzeixConjecture

/-- The Lorist--Schwenninger theorem uniformly supplies every standard finite
matrix dimension used by the Hilbert-space transport. -/
theorem loristSchwenningerFiniteMatrixMainTheorem :
    FiniteMatrixMainTheoremStatement :=
  fun d => loristSchwenningerMainTheorem (n := Fin d)

variable {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]

/-- The finite-matrix rational spectral-set consequence of the
Lorist--Schwenninger polynomial theorem. -/
theorem loristSchwenningerRationalSpectralSetCorollary :
    RationalSpectralSetCorollaryStatement (n := n) :=
  rationalSpectralSetCorollary_of_mainTheorem
    loristSchwenningerMainTheorem

variable {H : Type*} [NormedAddCommGroup H] [InnerProductSpace ℂ H]

/-- The Lorist--Schwenninger finite-matrix theorem transported to every nonzero
complex Hilbert space. -/
theorem loristSchwenningerHilbertSpacePolynomialCrouzeix
    [CompleteSpace H] [Nontrivial H]
    (A : H →L[ℂ] H) (p : Polynomial ℂ) :
    ‖operatorPolynomialEval p A‖ ≤
      2 * supPolynomialModulusOnOperatorNumericalRange A p :=
  hilbertSpacePolynomialCrouzeix_of_mainTheorem
    loristSchwenningerFiniteMatrixMainTheorem A p

/-- The Hilbert-space rational spectral-set statement supplied by the
Lorist--Schwenninger finite-matrix theorem. -/
theorem loristSchwenningerHilbertSpaceRationalSpectralSet
    [CompleteSpace H] [Nontrivial H] :
    HilbertRationalSpectralSetStatement (H := H) :=
  hilbertSpaceRationalSpectralSet_of_mainTheorem
    loristSchwenningerFiniteMatrixMainTheorem

/-- The closed numerical range of a bounded operator is a `2`-spectral set,
using the Lorist--Schwenninger theorem as the finite-matrix provider. -/
theorem loristSchwenningerClosedOperatorNumericalRange_isTwoSpectralSet
    [CompleteSpace H] [Nontrivial H] (A : H →L[ℂ] H) :
    ClosedOperatorNumericalRangeIsTwoSpectralSet A :=
  closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem
    loristSchwenningerFiniteMatrixMainTheorem A

end CrouzeixConjecture

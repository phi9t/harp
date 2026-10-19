import Crouzeix.LoristSchwenninger.Consequences
import Crouzeix.Harp.MainTheorem
import CrouzeixConjecture.FinalTheorems

namespace CrouzeixTextbook.Part06
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def lorist_schwenninger_main := @loristSchwenningerMainTheorem
set_option linter.defProp false in
def lorist_schwenninger_finite_matrix := @loristSchwenningerFiniteMatrixMainTheorem
set_option linter.defProp false in
def lorist_schwenninger_rational := @loristSchwenningerRationalSpectralSetCorollary
set_option linter.defProp false in
def lorist_schwenninger_hilbert := @loristSchwenningerHilbertSpacePolynomialCrouzeix
set_option linter.defProp false in
def lorist_schwenninger_two_spectral_set := @loristSchwenningerClosedOperatorNumericalRange_isTwoSpectralSet
set_option linter.defProp false in
def jin_final_comparator := @jinFinalCrouzeixConjecture

/-- The three checked terminal routes, recorded side by side.  This is a
comparison theorem: each conjunct retains its own terminal proof. -/
theorem three_route_terminal_bundle {n : Type}
    [Fintype n] [DecidableEq n] [Nonempty n] :
    MainTheoremStatement (n := n) ∧ MainTheoremStatement (n := n) ∧
      MainTheoremStatement (n := n) := by
  exact ⟨fun A p => jinFinalCrouzeixConjecture A p,
    loristSchwenningerMainTheorem,
    CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem⟩

namespace Exercises.Chapter35

theorem exercise_01_solution {n : Type*}
    [Fintype n] [DecidableEq n] [Nonempty n]
    (hmain : MainTheoremStatement (n := n)) (A : SquareMatrix n)
    (p : Polynomial ℂ) : PolynomialCrouzeixBound A p := by
  exact hmain A p

theorem exercise_02_solution
    (hmain : ∀ (d : ℕ) [Nonempty (Fin d)], MainTheoremStatement (n := Fin d)) :
    FiniteMatrixMainTheoremStatement := by
  exact hmain

theorem exercise_03_solution {n : Type*}
    [Fintype n] [DecidableEq n] [Nonempty n]
    (hmain : MainTheoremStatement (n := n)) :
    RationalSpectralSetCorollaryStatement (n := n) := by
  intro A r hfree
  exact (rationalSpectralSetCorollary_of_mainTheorem hmain A r hfree).trans_eq rfl

theorem exercise_04_solution {H : Type*}
    [NormedAddCommGroup H] [InnerProductSpace ℂ H]
    [CompleteSpace H] [Nontrivial H]
    (hfinite : FiniteMatrixMainTheoremStatement)
    (A : H →L[ℂ] H) (p : Polynomial ℂ) :
    ‖operatorPolynomialEval p A‖ ≤
      2 * supPolynomialModulusOnOperatorNumericalRange A p := by
  exact (hilbertSpacePolynomialCrouzeix_of_mainTheorem hfinite A p).trans_eq rfl

theorem exercise_05_solution {H : Type*}
    [NormedAddCommGroup H] [InnerProductSpace ℂ H]
    [CompleteSpace H] [Nontrivial H]
    (hfinite : FiniteMatrixMainTheoremStatement) (A : H →L[ℂ] H) :
    ClosedOperatorNumericalRangeIsTwoSpectralSet A := by
  rcases closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem hfinite A with
    ⟨hcompact, hspectrum, hrational⟩
  exact ⟨hcompact, hspectrum, hrational⟩

theorem exercise_06_solution {n : Type*}
    [Fintype n] [DecidableEq n] [Nonempty n]
    (A : SquareMatrix n) (p : Polynomial ℂ)
    (jinConclusion lsConclusion harpConclusion : Prop)
    (hJinNormalized : jinConclusion = PolynomialCrouzeixBound A p)
    (hLsNormalized : lsConclusion = PolynomialCrouzeixBound A p)
    (hHarpNormalized : harpConclusion = PolynomialCrouzeixBound A p)
    (hJin : jinConclusion) (hLs : lsConclusion) (hHarp : harpConclusion) :
    PolynomialCrouzeixBound A p ∧ PolynomialCrouzeixBound A p ∧
      PolynomialCrouzeixBound A p := by
  exact ⟨hJinNormalized ▸ hJin, hLsNormalized ▸ hLs, hHarpNormalized ▸ hHarp⟩

end Exercises.Chapter35

end
end CrouzeixTextbook.Part06

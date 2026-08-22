module

public import CrouzeixConjecture.HilbertSpectralSetCore
public import CrouzeixConjecture.HilbertSpace

@[expose] public section

noncomputable section

open Filter Set
open scoped InnerProductSpace Topology

namespace CrouzeixConjecture

variable {H : Type*} [NormedAddCommGroup H] [InnerProductSpace ℂ H]

private theorem jinFiniteMatrixMainTheorem : FiniteMatrixMainTheoremStatement :=
  fun d => jinFinalCrouzeixConjecture (n := Fin d)

section RationalApproximation

variable [CompleteSpace H] [Nontrivial H]

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem norm_operatorPolynomialEval_le_of_forall_closedNumericalRange
    (A : H →L[ℂ] H) (p : Polynomial ℂ) (C : ℝ)
    (hC : ∀ z ∈ closedOperatorNumericalRange A, ‖Polynomial.eval z p‖ ≤ C) :
    ‖operatorPolynomialEval p A‖ ≤ 2 * C :=
  norm_operatorPolynomialEval_le_of_forall_closedNumericalRange_of_mainTheorem
    jinFiniteMatrixMainTheorem A p C hC

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem cauchySeq_operatorPolynomialEval_of_tendstoUniformlyOn
    (A : H →L[ℂ] H) (q : ℕ → Polynomial ℂ) (f : ℂ → ℂ)
    (hq : TendstoUniformlyOn (fun N z ↦ Polynomial.eval z (q N)) f atTop
      (closedOperatorNumericalRange A)) :
    CauchySeq (fun N ↦ operatorPolynomialEval (q N) A) :=
  cauchySeq_operatorPolynomialEval_of_tendstoUniformlyOn_of_mainTheorem
    jinFiniteMatrixMainTheorem A q f hq

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem operatorRationalApproximants_tendsto
    (A : H →L[ℂ] H) (r : RatFunc ℂ)
    (hfree : RationalPoleFreeOn r (closedOperatorNumericalRange A)) :
    Tendsto (fun N ↦ operatorPolynomialEval (operatorRationalApproximants A r N) A)
      atTop (nhds (operatorRationalEval r A)) :=
  operatorRationalApproximants_tendsto_of_mainTheorem
    jinFiniteMatrixMainTheorem A r hfree

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem tendsto_operatorPolynomialEval_sub_zero_of_same_uniform_limit
    (A : H →L[ℂ] H) (q s : ℕ → Polynomial ℂ) (f : ℂ → ℂ)
    (hq : TendstoUniformlyOn (fun N z ↦ Polynomial.eval z (q N)) f atTop
      (closedOperatorNumericalRange A))
    (hs : TendstoUniformlyOn (fun N z ↦ Polynomial.eval z (s N)) f atTop
      (closedOperatorNumericalRange A)) :
    Tendsto (fun N ↦ operatorPolynomialEval (q N) A - operatorPolynomialEval (s N) A)
      atTop (nhds 0) :=
  tendsto_operatorPolynomialEval_sub_zero_of_same_uniform_limit_of_mainTheorem
    jinFiniteMatrixMainTheorem A q s f hq hs

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem tendsto_operatorPolynomialEval_of_tendstoUniformlyOn_polynomial
    (A : H →L[ℂ] H) (q : ℕ → Polynomial ℂ) (p : Polynomial ℂ)
    (hq : TendstoUniformlyOn (fun N z ↦ Polynomial.eval z (q N))
      (fun z ↦ Polynomial.eval z p) atTop (closedOperatorNumericalRange A)) :
    Tendsto (fun N ↦ operatorPolynomialEval (q N) A) atTop
      (nhds (operatorPolynomialEval p A)) :=
  tendsto_operatorPolynomialEval_of_tendstoUniformlyOn_polynomial_of_mainTheorem
    jinFiniteMatrixMainTheorem A q p hq

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem tendsto_operatorPolynomialEval_of_rational_approximation
    (A : H →L[ℂ] H) (r : RatFunc ℂ)
    (hfree : RationalPoleFreeOn r (closedOperatorNumericalRange A))
    (q : ℕ → Polynomial ℂ)
    (hq : TendstoUniformlyOn (fun N z ↦ Polynomial.eval z (q N))
      (rationalScalarEval r) atTop (closedOperatorNumericalRange A)) :
    Tendsto (fun N ↦ operatorPolynomialEval (q N) A) atTop
      (nhds (operatorRationalEval r A)) :=
  tendsto_operatorPolynomialEval_of_rational_approximation_of_mainTheorem
    jinFiniteMatrixMainTheorem A r hfree q hq

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem hilbertSpaceRationalCrouzeix
    (A : H →L[ℂ] H) (r : RatFunc ℂ)
    (hfree : RationalPoleFreeOn r (closedOperatorNumericalRange A)) :
    ‖operatorRationalEval r A‖ ≤
      2 * supRationalModulusOnClosedOperatorNumericalRange A r :=
  hilbertSpaceRationalCrouzeix_of_mainTheorem jinFiniteMatrixMainTheorem A r hfree

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem spectrum_subset_closedOperatorNumericalRange (A : H →L[ℂ] H) :
    spectrum ℂ A ⊆ closedOperatorNumericalRange A :=
  spectrum_subset_closedOperatorNumericalRange_of_mainTheorem jinFiniteMatrixMainTheorem A

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem operatorPolynomialEval_denom_isUnit_of_rationalPoleFreeOn
    (A : H →L[ℂ] H) (r : RatFunc ℂ)
    (hfree : RationalPoleFreeOn r (closedOperatorNumericalRange A)) :
    IsUnit (operatorPolynomialEval r.denom A) :=
  operatorPolynomialEval_denom_isUnit_of_rationalPoleFreeOn_of_mainTheorem
    jinFiniteMatrixMainTheorem A r hfree

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem operatorRationalEval_eq_num_mul_inverse_denom
    (A : H →L[ℂ] H) (r : RatFunc ℂ)
    (hfree : RationalPoleFreeOn r (closedOperatorNumericalRange A)) :
    operatorRationalEval r A =
      operatorPolynomialEval r.num A * Ring.inverse (operatorPolynomialEval r.denom A) :=
  operatorRationalEval_eq_num_mul_inverse_denom_of_mainTheorem
    jinFiniteMatrixMainTheorem A r hfree

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem hilbertSpaceRationalSpectralSet : HilbertRationalSpectralSetStatement (H := H) :=
  hilbertSpaceRationalSpectralSet_of_mainTheorem jinFiniteMatrixMainTheorem

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem closedOperatorNumericalRange_isTwoSpectralSet (A : H →L[ℂ] H) :
    ClosedOperatorNumericalRangeIsTwoSpectralSet A :=
  closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem
    jinFiniteMatrixMainTheorem A

/-- Compatibility wrapper using the existing Jin finite-matrix theorem provider. -/
theorem operatorRationalEval_algebraMap_polynomial
    (A : H →L[ℂ] H) (p : Polynomial ℂ) :
    operatorRationalEval (algebraMap (Polynomial ℂ) (RatFunc ℂ) p) A =
      operatorPolynomialEval p A :=
  operatorRationalEval_algebraMap_polynomial_of_mainTheorem
    jinFiniteMatrixMainTheorem A p

end RationalApproximation

end CrouzeixConjecture

import CrouzeixConjecture.PositiveRealCompletion
import CrouzeixConjecture.FinalTheorems
import CrouzeixConjecture.RadialOuterReduction
import CrouzeixConjecture.RationalApproximation
import CrouzeixConjecture.QTransferAlgebra
import Mathlib.Topology.Separation.Connected
namespace CrouzeixTextbook.Part06
open CrouzeixConjecture Filter Set
open scoped ComplexOrder InnerProductSpace Matrix Matrix.Norms.L2Operator Topology

noncomputable section

/-- Public CFT-32-001 theorem. The proof provider constructs the positive
square root, transports the Chapter 31 source inequality through the two
Gramian congruences, runs the eigenvector contradiction, and transfers the
result back through the polar unitary. -/
theorem completion_implies_norm_two
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (S : SquareMatrix n) (hS : IsUnit S) (lambda : n → ℂ)
    (hlambda : ∀ i, ‖lambda i‖ ≤ 1) (d : ℂ → n → ℂ) (hd0 : d 0 = 0)
    (hKernel : IsPositiveMatrixKernelOn openUnitDisk
      (matrixHerglotzKernel
        (completionKernelModel (completionGramMatrix S) lambda d))) :
    ‖completionDiagonalizableMatrix S lambda‖ ≤ 2 := by
  have hProvider : ‖completionDiagonalizableMatrix S lambda‖ ≤ 2 :=
    norm_completionDiagonalizableMatrix_le_two_of_positiveKernelModel
      S hS lambda hlambda d hd0 hKernel
  exact hProvider

set_option linter.defProp false in
def jin_polynomial_constant_two :=
  @polynomialCrouzeixBound_of_holomorphicCrouzeixBound

/-- The theorem-kind provider for the bundled rational spectral-set
statement, instantiated from the pointwise holomorphic rational bound. -/
theorem jin_rational_spectral_set_provider
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n] :
    RationalSpectralSetCorollaryStatement (n := n) := by
  intro A r hfree
  have hProvider := holomorphicCrouzeixRationalBound A r hfree
  exact hProvider

set_option linter.defProp false in
def jin_rational_spectral_set := @jin_rational_spectral_set_provider
set_option linter.defProp false in
def jin_rational_constant_two := @holomorphicCrouzeixRationalBound
set_option linter.defProp false in
def holomorphic_constant_two := @holomorphicCrouzeixBound
set_option linter.defProp false in
def polynomial_from_holomorphic := @polynomialCrouzeixBound_of_holomorphicCrouzeixBound

namespace Exercises.Chapter32

/-- Extract the norm estimate from the Gramian anticommutator inequality
without invoking the positive-kernel completion endpoint. -/
theorem exercise_01_solution
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    {M : ℝ} (hM : 0 ≤ M) (C : SquareMatrix n)
    (hbound : ∀ k : ℕ, ‖C ^ k‖ ≤ M)
    (hineq : (4 • (gramian 2 C - gramian 4 C) -
      (gramian 2 C - gramian 4 C) * gramian 4 C -
      gramian 4 C * (gramian 2 C - gramian 4 C)).PosSemidef) : ‖C‖ ≤ 2 := by
  have hfourSmul :
      (4 : ℕ) • (gramian 2 C - gramian 4 C) =
        (4 : ℂ) • (gramian 2 C - gramian 4 C) := by
    ext i j
    norm_num [Matrix.smul_apply]
  rw [hfourSmul] at hineq
  have hfour :=
    four_sub_conjTranspose_mul_self_posSemidef_of_gramian_inequality
      hM C hbound hineq
  exact matrix_norm_le_two_of_four_sub_conjTranspose_mul_self_posSemidef C hfour

/-- Separate the zero and positive polynomial normalization branches. -/
theorem exercise_02_solution
    {n : Type*} [Fintype n] [DecidableEq n]
    (A : SquareMatrix n) (p : Polynomial ℂ) (M : ℝ)
    (hM : M = maxPolynomialModulusOnNumericalRange A p) :
    (M = 0 → polynomialEval p A = 0) ∧
      (0 < M →
        ‖polynomialEval (((M : ℂ)⁻¹) • p) A‖ ≤ 2 →
        ‖polynomialEval p A‖ ≤ 2 * M) := by
  classical
  cases isEmpty_or_nonempty n with
  | inl hempty =>
      letI : IsEmpty n := hempty
      constructor
      · intro _
        exact Subsingleton.elim _ _
      · intro _ _
        have hzero : polynomialEval p A = 0 := Subsingleton.elim _ _
        rw [hzero, norm_zero]
        positivity
  | inr hnonempty =>
      letI : Nonempty n := hnonempty
      constructor
      · intro hzero
        have hvanish : ∀ z ∈ numericalRange A, p.eval z = 0 := by
          intro z hz
          apply norm_eq_zero.mp
          apply le_antisymm
          · calc
              ‖p.eval z‖ ≤ maxPolynomialModulusOnNumericalRange A p :=
                norm_polynomial_eval_le_maxOnNumericalRange A p hz
              _ = M := hM.symm
              _ = 0 := hzero
          · exact norm_nonneg _
        by_cases hp : p = 0
        · simp [hp, polynomialEval]
        have hfinite : (numericalRange A).Finite :=
          (Polynomial.finite_setOf_isRoot hp).subset (by
            intro z hz
            exact hvanish z hz)
        have hpreconnected : IsPreconnected (numericalRange A) :=
          (numericalRange_convex A).isPreconnected
        have hsubsingleton : (numericalRange A).Subsingleton := by
          intro z hz w hw
          by_contra hzw
          have hinfinite : (numericalRange A).Infinite :=
            hpreconnected.infinite_of_nontrivial ⟨z, hz, w, hw, hzw⟩
          exact hinfinite hfinite
        obtain ⟨z, hz⟩ := numericalRange_nonempty A
        have hmatrix : A = z • (1 : SquareMatrix n) := by
          let B : SquareMatrix n := A - z • (1 : SquareMatrix n)
          have hquadratic : ∀ x : EuclideanVector n,
              ⟪x, euclideanOperator B x⟫_ℂ = 0 := by
            intro x
            by_cases hx : x = 0
            · simp [hx]
            let u : EuclideanVector n := (‖x‖ : ℂ)⁻¹ • x
            have hxnorm : 0 < ‖x‖ := norm_pos_iff.mpr hx
            have hu : ‖u‖ = 1 := by
              simp [u, norm_smul, Complex.norm_real,
                inv_mul_cancel₀ (ne_of_gt hxnorm)]
            have huW : ⟪u, euclideanOperator A u⟫_ℂ ∈ numericalRange A :=
              ⟨u, hu, rfl⟩
            have huvalue : ⟪u, euclideanOperator A u⟫_ℂ = z :=
              hsubsingleton huW hz
            have huquadratic : ⟪u, euclideanOperator B u⟫_ℂ = 0 := by
              have hopsub :
                  euclideanOperator (A - z • (1 : SquareMatrix n)) =
                    euclideanOperator A -
                      euclideanOperator (z • (1 : SquareMatrix n)) := by
                exact map_sub (euclideanOperator (n := n)) A
                  (z • (1 : SquareMatrix n))
              rw [show B = A - z • (1 : SquareMatrix n) by rfl,
                hopsub, sub_apply]
              have hscalar :
                  euclideanOperator (z • (1 : SquareMatrix n)) u = z • u := by
                simp
              rw [inner_sub_right, hscalar, inner_smul_right, huvalue]
              simp [hu]
            have hxrepr : x = (‖x‖ : ℂ) • u := by
              simp [u, smul_smul, ne_of_gt hxnorm]
            rw [hxrepr, map_smul, inner_smul_left, inner_smul_right,
              huquadratic, mul_zero]
            simp
          have hlinear : (euclideanOperator B).toLinearMap = 0 := by
            rw [← inner_map_self_eq_zero]
            intro x
            rw [← inner_conj_symm]
            simp [hquadratic x]
          have hoperator : euclideanOperator B = 0 := by
            apply ContinuousLinearMap.ext
            intro x
            exact LinearMap.congr_fun hlinear x
          have hB : B = 0 := by
            have := congrArg (euclideanOperator (n := n)).symm hoperator
            simpa using this
          exact sub_eq_zero.mp hB
        rw [hmatrix, polynomialEval, ← Algebra.algebraMap_eq_smul_one]
        rw [Polynomial.aeval_algebraMap_apply_eq_algebraMap_eval,
          hvanish z hz, map_zero]
      · intro hMpos hnormalized
        have heval :
            polynomialEval (((M : ℂ)⁻¹) • p) A =
              ((M : ℂ)⁻¹) • polynomialEval p A := by
          simp [polynomialEval, Algebra.smul_def]
        rw [heval, norm_smul, norm_inv, Complex.norm_real,
          Real.norm_eq_abs, abs_of_pos hMpos] at hnormalized
        calc
          ‖polynomialEval p A‖ = M * (M⁻¹ * ‖polynomialEval p A‖) := by
            field_simp
          _ ≤ M * 2 := mul_le_mul_of_nonneg_left hnormalized hMpos.le
          _ = 2 * M := by ring

/-- Pole avoidance supplies the exact open holomorphy domain used by the
rational functional calculus. -/
theorem exercise_03_solution (r : RatFunc ℂ) (s : Set ℂ)
    (hfree : RationalPoleFreeOn r s) :
    IsOpen (rationalPoleSet r)ᶜ ∧ s ⊆ (rationalPoleSet r)ᶜ ∧
      DifferentiableOn ℂ (rationalScalarEval r) (rationalPoleSet r)ᶜ := by
  refine ⟨isOpen_compl_rationalPoleSet r,
    (rationalPoleFreeOn_iff_subset_compl r s).mp hfree, ?_⟩
  exact differentiableOn_rationalScalarEval r _
    (rationalPoleFreeOn_compl_rationalPoleSet r)

/-- Separate the zero and positive rational normalization branches. -/
theorem exercise_04_solution
    {n : Type*} [Fintype n] [DecidableEq n]
    (A : SquareMatrix n) (r : RatFunc ℂ)
    (hfree : RationalPoleFreeOn r (numericalRange A)) (M : ℝ)
    (hM : M = maxRationalModulusOnNumericalRange A r) :
    (M = 0 → rationalMatrixEval r A = 0) ∧
      (0 < M →
        ‖rationalMatrixEval (((M : ℂ)⁻¹) • r) A‖ ≤ 2 →
        ‖rationalMatrixEval r A‖ ≤ 2 * M) := by
  classical
  cases isEmpty_or_nonempty n with
  | inl hempty =>
      letI : IsEmpty n := hempty
      constructor
      · intro _
        exact Subsingleton.elim _ _
      · intro _ _
        have hzero : rationalMatrixEval r A = 0 := Subsingleton.elim _ _
        rw [hzero, norm_zero]
        positivity
  | inr hnonempty =>
      letI : Nonempty n := hnonempty
      constructor
      · intro hzero
        have hscalar : ∀ z ∈ numericalRange A, rationalScalarEval r z = 0 := by
          intro z hz
          apply norm_eq_zero.mp
          apply le_antisymm
          · calc
              ‖rationalScalarEval r z‖ ≤
                  maxRationalModulusOnNumericalRange A r :=
                norm_rationalScalarEval_le_maxRationalModulusOnNumericalRange
                  A r hfree hz
              _ = M := hM.symm
              _ = 0 := hzero
          · exact norm_nonneg _
        have hnum : ∀ z ∈ numericalRange A, r.num.eval z = 0 := by
          intro z hz
          have hdenom : r.denom.eval z ≠ 0 :=
            (rationalPoleFreeOn_iff r (numericalRange A)).mp hfree z hz
          simpa [rationalScalarEval, hdenom] using hscalar z hz
        have hnumeval : polynomialEval r.num A = 0 := by
          by_cases hp : r.num = 0
          · simp [hp, polynomialEval]
          have hfinite : (numericalRange A).Finite :=
            (Polynomial.finite_setOf_isRoot hp).subset (by
              intro z hz
              exact hnum z hz)
          have hpreconnected : IsPreconnected (numericalRange A) :=
            (numericalRange_convex A).isPreconnected
          have hsubsingleton : (numericalRange A).Subsingleton := by
            intro z hz w hw
            by_contra hzw
            have hinfinite : (numericalRange A).Infinite :=
              hpreconnected.infinite_of_nontrivial ⟨z, hz, w, hw, hzw⟩
            exact hinfinite hfinite
          obtain ⟨z, hz⟩ := numericalRange_nonempty A
          have hmatrix : A = z • (1 : SquareMatrix n) := by
            let B : SquareMatrix n := A - z • (1 : SquareMatrix n)
            have hquadratic : ∀ x : EuclideanVector n,
                ⟪x, euclideanOperator B x⟫_ℂ = 0 := by
              intro x
              by_cases hx : x = 0
              · simp [hx]
              let u : EuclideanVector n := (‖x‖ : ℂ)⁻¹ • x
              have hxnorm : 0 < ‖x‖ := norm_pos_iff.mpr hx
              have hu : ‖u‖ = 1 := by
                simp [u, norm_smul, Complex.norm_real,
                  inv_mul_cancel₀ (ne_of_gt hxnorm)]
              have huW : ⟪u, euclideanOperator A u⟫_ℂ ∈ numericalRange A :=
                ⟨u, hu, rfl⟩
              have huvalue : ⟪u, euclideanOperator A u⟫_ℂ = z :=
                hsubsingleton huW hz
              have huquadratic : ⟪u, euclideanOperator B u⟫_ℂ = 0 := by
                have hopsub :
                    euclideanOperator (A - z • (1 : SquareMatrix n)) =
                      euclideanOperator A -
                        euclideanOperator (z • (1 : SquareMatrix n)) := by
                  exact map_sub (euclideanOperator (n := n)) A
                    (z • (1 : SquareMatrix n))
                rw [show B = A - z • (1 : SquareMatrix n) by rfl,
                  hopsub, sub_apply]
                have hscalar :
                    euclideanOperator (z • (1 : SquareMatrix n)) u = z • u := by
                  simp
                rw [inner_sub_right, hscalar, inner_smul_right, huvalue]
                simp [hu]
              have hxrepr : x = (‖x‖ : ℂ) • u := by
                simp [u, smul_smul, ne_of_gt hxnorm]
              rw [hxrepr, map_smul, inner_smul_left, inner_smul_right,
                huquadratic, mul_zero]
              simp
            have hlinear : (euclideanOperator B).toLinearMap = 0 := by
              rw [← inner_map_self_eq_zero]
              intro x
              rw [← inner_conj_symm]
              simp [hquadratic x]
            have hoperator : euclideanOperator B = 0 := by
              apply ContinuousLinearMap.ext
              intro x
              exact LinearMap.congr_fun hlinear x
            have hB : B = 0 := by
              have := congrArg (euclideanOperator (n := n)).symm hoperator
              simpa using this
            exact sub_eq_zero.mp hB
          rw [hmatrix, polynomialEval, ← Algebra.algebraMap_eq_smul_one]
          rw [Polynomial.aeval_algebraMap_apply_eq_algebraMap_eval,
            hnum z hz, map_zero]
        simp [rationalMatrixEval, hnumeval]
      · intro hMpos hnormalized
        have hscale :
            rationalMatrixEval (((M : ℂ)⁻¹) • r) A =
              ((M : ℂ)⁻¹) • rationalMatrixEval r A := by
          let c : ℂ := (M : ℂ)⁻¹
          have hdisplayedEq :
              c • r = algebraMap (Polynomial ℂ) (RatFunc ℂ) (c • r.num) /
                algebraMap (Polynomial ℂ) (RatFunc ℂ) r.denom := by
            rw [RatFunc.smul_eq_C_smul, Algebra.smul_def]
            calc
              (algebraMap (Polynomial ℂ) (RatFunc ℂ)) (Polynomial.C c) * r =
                  (algebraMap (Polynomial ℂ) (RatFunc ℂ)) (Polynomial.C c) *
                    ((algebraMap (Polynomial ℂ) (RatFunc ℂ)) r.num /
                      (algebraMap (Polynomial ℂ) (RatFunc ℂ)) r.denom) := by
                        rw [RatFunc.num_div_denom]
              _ = (algebraMap (Polynomial ℂ) (RatFunc ℂ)) (c • r.num) /
                  (algebraMap (Polynomial ℂ) (RatFunc ℂ)) r.denom := by
                rw [Polynomial.smul_eq_C_mul, map_mul]
                ring
          have hdisplayedFree :
              RationalPoleFreeOn (c • r) (numericalRange A) := by
            rw [hdisplayedEq]
            apply rationalPoleFreeOn_fraction_of_denominator_ne_zero
            intro z hz
            exact (rationalPoleFreeOn_iff r (numericalRange A)).mp hfree z hz
          have hreduced :=
            polynomialEval_denom_isUnit_of_rationalPoleFreeOn_numericalRange
              (c • r) A hdisplayedFree
          have hdenom :=
            polynomialEval_denom_isUnit_of_rationalPoleFreeOn_numericalRange
              r A hfree
          rw [show ((M : ℂ)⁻¹ • r = c • r) by rfl,
            rationalMatrixEval_fraction (c • r) (c • r.num) r.denom A
              hdisplayedEq hreduced hdenom]
          simp [c, rationalMatrixEval, polynomialEval, map_smul]
        rw [hscale,
          norm_smul, norm_inv, Complex.norm_real, Real.norm_eq_abs,
          abs_of_pos hMpos] at hnormalized
        calc
          ‖rationalMatrixEval r A‖ = M * (M⁻¹ * ‖rationalMatrixEval r A‖) := by
            field_simp
          _ ≤ M * 2 := mul_le_mul_of_nonneg_left hnormalized hMpos.le
          _ = 2 * M := by ring

/-- Record both limits in their mathematical order: first the matrix
approximation, then the maxima on shrinking outer domains. -/
theorem exercise_05_solution
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (A : SquareMatrix n) {U : Set ℂ} (hUopen : IsOpen U)
    (hWU : numericalRange A ⊆ U) {f : ℂ → ℂ}
    (hf : DifferentiableOn ℂ f U) :
    Tendsto (simpleSpectrumHolomorphicEval A f) atTop
      (nhds (holomorphicMatrixEval A f)) ∧
    ∃ N : ℕ, Tendsto
      (fun k ↦ maxFunctionModulusOnSet
        (closure (parallelOuterDomain (numericalRange A) (k + N))) f)
      atTop (nhds (maxFunctionModulusOnSet (numericalRange A) f)) := by
  let K := numericalRange A
  have hKcompact : IsCompact K := isCompact_numericalRange A
  have hKne : K.Nonempty := numericalRange_nonempty A
  obtain ⟨epsilon, hepsilon, hbuffer⟩ :=
    hKcompact.exists_cthickening_subset_open hUopen hWU
  let C := Metric.cthickening epsilon K
  have hCcompact : IsCompact C := hKcompact.cthickening
  have hfC : ContinuousOn f C := hf.continuousOn.mono hbuffer
  have heventually : ∀ᶠ k in atTop, outerApproximationRadius k < epsilon :=
    (tendsto_order.1 tendsto_outerApproximationRadius).2 epsilon hepsilon
  obtain ⟨N, hN⟩ := eventually_atTop.1 heventually
  refine ⟨tendsto_simpleSpectrumHolomorphicEval_of_differentiableOn_neighborhood
    A hUopen hWU hf, ⟨N, ?_⟩⟩
  apply tendsto_maxFunctionModulusOnSet_of_outerApproximation
    hKcompact hKne hCcompact hfC
  · intro k
    exact (parallelOuterDomain_data hKcompact (numericalRange_convex A) hKne
      (k + N)).closure_isCompact
  · intro k
    exact (parallelOuterDomain_data hKcompact (numericalRange_convex A) hKne
      (k + N)).contains.trans subset_closure
  · intro k
    rw [parallelOuterDomain_closure]
    exact Metric.cthickening_mono (hN (k + N) (by omega)).le K
  · exact tendsto_outerApproximationRadius.comp (tendsto_add_atTop_nat N)
  · intro k z hz
    exact parallelOuterDomain_closure_near hKcompact (k + N) hz

/-- Only after the holomorphic limit is complete do polynomial evaluation and
the two maximum-modulus notations become definitionally identical. -/
theorem exercise_06_solution
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (A : SquareMatrix n) (p : Polynomial ℂ) :
    holomorphicMatrixEval A (fun z ↦ Polynomial.eval z p) = polynomialEval p A ∧
      maxFunctionModulusOnSet (numericalRange A) (fun z ↦ Polynomial.eval z p) =
        maxPolynomialModulusOnNumericalRange A p := by
  exact ⟨holomorphicMatrixEval_polynomial A p, rfl⟩

end Exercises.Chapter32

end
end CrouzeixTextbook.Part06

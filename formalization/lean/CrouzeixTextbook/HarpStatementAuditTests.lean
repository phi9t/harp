import CrouzeixTextbook.HarpStatementAudit

noncomputable section
open scoped ComplexConjugate InnerProductSpace Matrix Matrix.Norms.L2Operator
open CrouzeixConjecture
namespace CrouzeixTextbook.HarpStatementAuditTests
open HarpStatementAudit

-- Public audit declarations must remain available to compile clients.
#check HarpStatementAudit.textbookBound.{0}
#check HarpStatementAudit.textbookBound_iff_mainTheoremStatement
#check HarpStatementAudit.polynomialModulusImage_nonempty
#check HarpStatementAudit.polynomialModulusImage_bddAbove
#check HarpStatementAudit.polynomialModulusImage_isCompact
#check HarpStatementAudit.polynomialModulusImage_maximum

variable {n : Type} [Fintype n] [DecidableEq n] [Nonempty n]

/-- The zero matrix has numerical range exactly `{0}` in positive dimension. -/
theorem zeroMatrix_range : numericalRange (0 : Matrix n n ℂ) = {0} := by
  ext z
  constructor
  · rintro ⟨x, hx, rfl⟩
    simp [euclideanOperator]
  · intro hz
    obtain ⟨w, hw⟩ := numericalRange_nonempty (0 : Matrix n n ℂ)
    obtain ⟨x, hx, hform⟩ := hw
    refine ⟨x, hx, ?_⟩
    simp [euclideanOperator, Set.mem_singleton_iff.mp hz]

theorem zeroMatrix_bound (p : Polynomial ℂ) :
    PolynomialCrouzeixBound (0 : Matrix n n ℂ) p := by
  have heval : Polynomial.aeval (0 : Matrix n n ℂ) p =
      algebraMap ℂ (Matrix n n ℂ) (p.eval 0) := by
    simpa using p.aeval_algebraMap_apply_eq_algebraMap_eval
      (A := Matrix n n ℂ) 0
  simp only [PolynomialCrouzeixBound, polynomialEval, heval,
    maxPolynomialModulusOnNumericalRange, zeroMatrix_range, Set.image_singleton,
    csSup_singleton, norm_algebraMap, norm_one, mul_one]
  linarith [norm_nonneg (p.eval 0)]

/-- Constants have both operator norm and numerical-range maximum `‖c‖`. -/
theorem constantPolynomial_values (A : Matrix n n ℂ) (c : ℂ) :
    ‖polynomialEval (Polynomial.C c) A‖ = ‖c‖ ∧
      maxPolynomialModulusOnNumericalRange A (Polynomial.C c) = ‖c‖ := by
  constructor
  · simp [polynomialEval]
  · obtain ⟨z, hz, hmax⟩ := exists_maxPolynomialModulusOnNumericalRange A (Polynomial.C c)
    simpa using hmax.symm

theorem constantPolynomial_bound (A : Matrix n n ℂ) (c : ℂ) :
    PolynomialCrouzeixBound A (Polynomial.C c) := by
  obtain ⟨hnorm, hmax⟩ := constantPolynomial_values A c
  unfold PolynomialCrouzeixBound
  rw [hnorm, hmax]
  linarith [norm_nonneg c]

/-- Every one-dimensional matrix is scalar, so polynomial evaluation is scalar evaluation. -/
theorem oneDimensional_eval (A : Matrix (Fin 1) (Fin 1) ℂ) (p : Polynomial ℂ) :
    Polynomial.aeval A p =
      algebraMap ℂ (Matrix (Fin 1) (Fin 1) ℂ) (p.eval (A 0 0)) := by
  have hA : A = algebraMap ℂ (Matrix (Fin 1) (Fin 1) ℂ) (A 0 0) := by
    ext i j
    fin_cases i
    fin_cases j
    simp [Algebra.algebraMap_eq_smul_one]
  conv_lhs => rw [hA]
  exact p.aeval_algebraMap_apply_eq_algebraMap_eval (A 0 0)

theorem oneDimensional_range (A : Matrix (Fin 1) (Fin 1) ℂ) :
    numericalRange A = {A 0 0} := by
  ext z
  constructor
  · rintro ⟨x, hx, rfl⟩
    have hA : A = (A 0 0) • (1 : Matrix (Fin 1) (Fin 1) ℂ) := by
      ext i j
      fin_cases i
      fin_cases j
      simp
    change ⟪x, euclideanOperator A x⟫_ℂ = A 0 0
    calc
      _ = ⟪x, euclideanOperator ((A 0 0) • (1 : Matrix (Fin 1) (Fin 1) ℂ)) x⟫_ℂ :=
        congrArg (fun B ↦ ⟪x, euclideanOperator B x⟫_ℂ) hA
      _ = A 0 0 := by simp [inner_smul_right, inner_self_eq_norm_sq_to_K, hx]
  · intro hz
    subst z
    refine ⟨EuclideanSpace.single 0 1, by simp, ?_⟩
    simp [euclideanOperator, EuclideanSpace.inner_eq_star_dotProduct, dotProduct]

theorem oneDimensional_bound (A : Matrix (Fin 1) (Fin 1) ℂ) (p : Polynomial ℂ) :
    PolynomialCrouzeixBound A p := by
  simp only [PolynomialCrouzeixBound, polynomialEval, oneDimensional_eval,
    norm_algebraMap, maxPolynomialModulusOnNumericalRange, oneDimensional_range,
    Set.image_singleton, csSup_singleton, norm_one, mul_one]
  linarith [norm_nonneg (p.eval (A 0 0))]

/-- Vanishing makes the actual nonempty image `{0}`; no empty-supremum convention is used. -/
theorem vanishing_maximum (A : Matrix n n ℂ) (p : Polynomial ℂ)
    (hp : ∀ z ∈ numericalRange A, p.eval z = 0) :
    maxPolynomialModulusOnNumericalRange A p = 0 := by
  obtain ⟨z, hz, hmax⟩ := exists_maxPolynomialModulusOnNumericalRange A p
  rw [hp z hz, norm_zero] at hmax
  exact hmax.symm

/-- Conditional consequence of the audited proposition, not an independent terminal proof. -/
theorem vanishing_eval_of_textbookBound (h : textbookBound.{0})
    (A : Matrix n n ℂ) (p : Polynomial ℂ)
    (hp : ∀ z ∈ numericalRange A, p.eval z = 0) :
    Polynomial.aeval A p = 0 := by
  have hb := (textbookBound_iff_mainTheoremStatement.mp h) n A p
  change ‖Polynomial.aeval A p‖ ≤ 2 * maxPolynomialModulusOnNumericalRange A p at hb
  rw [vanishing_maximum A p hp, mul_zero] at hb
  exact norm_eq_zero.mp (le_antisymm hb (norm_nonneg _))

/-- A genuinely nonzero square-zero matrix. Its norm is not zero. -/
def nilpotentTwo : Matrix (Fin 2) (Fin 2) ℂ := !![0, 1; 0, 0]

theorem nilpotentTwo_nonzero : nilpotentTwo ≠ 0 := by
  intro h
  have := congrArg (fun A : Matrix (Fin 2) (Fin 2) ℂ ↦ A 0 1) h
  norm_num [nilpotentTwo] at this

theorem nilpotentTwo_square : nilpotentTwo * nilpotentTwo = 0 := by
  ext i j
  fin_cases i <;> fin_cases j <;> norm_num [nilpotentTwo, Matrix.mul_apply, Fin.sum_univ_two]

theorem nilpotentTwo_action (x : EuclideanSpace ℂ (Fin 2)) :
    euclideanOperator nilpotentTwo x = WithLp.toLp 2 ![x 1, 0] := by
  ext i
  fin_cases i
  · simp [nilpotentTwo, euclideanOperator]
    rfl
  · simp [nilpotentTwo, euclideanOperator]

theorem nilpotentTwo_norm : ‖euclideanOperator nilpotentTwo‖ = 1 := by
  have ha (x : EuclideanSpace ℂ (Fin 2)) :
      ‖euclideanOperator nilpotentTwo x‖ = ‖x 1‖ := by
    rw [nilpotentTwo_action, EuclideanSpace.norm_eq]
    simp [Fin.sum_univ_two, Real.sqrt_sq (norm_nonneg (x 1))]
  apply le_antisymm
  · apply ContinuousLinearMap.opNorm_le_bound _ zero_le_one
    intro x
    rw [one_mul, ha]
    exact PiLp.norm_apply_le x 1
  · have h := (euclideanOperator nilpotentTwo).le_opNorm
      (EuclideanSpace.single 1 1)
    simpa [ha] using h

/- Exact compiler correspondence tests. The altered propositions below are
well-typed before rejection; they are not asserted to be false. In particular,
quantifier reordering is logically equivalent to the target. Adding the terminal
statement as a premise makes a tautology. Neither accepts the target proof verbatim. -/

def reorderedQuantifiers : Prop :=
  ∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n]
    (p : Polynomial ℂ) (A : Matrix n n ℂ), PolynomialCrouzeixBound A p

def addedTerminalPremise : Prop :=
  ∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n],
    MainTheoremStatement (n := n) → MainTheoremStatement (n := n)

def missingUnitCondition : Prop :=
  ∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n]
    (A : Matrix n n ℂ) (p : Polynomial ℂ),
    ‖euclideanOperator (Polynomial.aeval A p)‖ ≤
      2 * sSup ((fun z : ℂ ↦ ‖p.eval z‖) ''
        {z : ℂ | ∃ x : EuclideanSpace ℂ n, ⟪x, euclideanOperator A x⟫_ℂ = z})

def wrongFrobeniusNorm : Prop :=
  ∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n]
    (A : Matrix n n ℂ) (p : Polynomial ℂ),
    Real.sqrt (∑ i, ∑ j, ‖Polynomial.aeval A p i j‖ ^ 2) ≤
      2 * sSup (polynomialModulusImage A p)

theorem exactCorrespondence_positiveControl (h : textbookBound.{0}) :
    ∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n]
      (A : Matrix n n ℂ) (p : Polynomial ℂ), PolynomialCrouzeixBound A p := h

theorem exactCorrespondence_rejectsReordered (h : textbookBound.{0}) : True := by
  have _ : textbookBound.{0} := h
  fail_if_success have _ : reorderedQuantifiers := h
  trivial

theorem exactCorrespondence_rejectsPremise (h : textbookBound.{0}) : True := by
  have _ : textbookBound.{0} := h
  fail_if_success have _ : addedTerminalPremise := h
  trivial

theorem exactCorrespondence_rejectsMissingUnit (h : textbookBound.{0}) : True := by
  have _ : textbookBound.{0} := h
  fail_if_success have _ : missingUnitCondition := h
  trivial

theorem exactCorrespondence_rejectsWrongNorm (h : textbookBound.{0}) : True := by
  have _ : textbookBound.{0} := h
  fail_if_success have _ : wrongFrobeniusNorm := h
  trivial

theorem reorderedQuantifiers_iff : reorderedQuantifiers ↔ textbookBound.{0} := by
  constructor
  · intro h n _ _ _ A p
    exact h n p A
  · intro h n _ _ _ p A
    exact h n A p

/-- Adding the terminal statement as its own premise makes the result tautological. -/
theorem addedTerminalPremise_true : addedTerminalPremise := fun _ _ _ _ h ↦ h

/-- The unit vector `(1/√2,1/√2)` witnesses the sharp factor two for `X`. -/
theorem nilpotentTwo_X_bound : PolynomialCrouzeixBound nilpotentTwo Polynomial.X := by
  let s : ℝ := Real.sqrt 2 / 2
  have hs : 0 ≤ s := by positivity
  have hs2 : s ^ 2 = 1 / 2 := by
    dsimp [s]
    nlinarith [Real.sq_sqrt (show (0 : ℝ) ≤ 2 by norm_num)]
  let x : EuclideanSpace ℂ (Fin 2) := WithLp.toLp 2 ![(s : ℂ), (s : ℂ)]
  have hx2 : ‖x‖ ^ 2 = 1 := by
    norm_num [EuclideanSpace.norm_sq_eq, Fin.sum_univ_two, x,
      Complex.norm_real, Real.norm_eq_abs, abs_of_nonneg hs, hs2]
  have hx : ‖x‖ = 1 := by nlinarith [norm_nonneg x]
  have hform : ⟪x, euclideanOperator nilpotentTwo x⟫_ℂ = (1 / 2 : ℂ) := by
    rw [nilpotentTwo_action]
    simp only [PiLp.inner_apply, Fin.sum_univ_two, x,
      Matrix.cons_val_zero, Matrix.cons_val_one,
      RCLike.inner_apply, zero_mul, add_zero, Complex.conj_ofReal]
    rw [← Complex.ofReal_mul]
    rw [show s * s = (1 / 2 : ℝ) by nlinarith [hs2]]
    norm_num
  have hm := norm_polynomial_eval_le_maxOnNumericalRange nilpotentTwo Polynomial.X
    (show (1 / 2 : ℂ) ∈ numericalRange nilpotentTwo from ⟨x, hx, hform⟩)
  norm_num at hm
  change ‖polynomialEval Polynomial.X nilpotentTwo‖ ≤ _
  have hn : ‖polynomialEval Polynomial.X nilpotentTwo‖ = 1 := by
    simpa [polynomialEval, matrix_norm_eq_euclidean_operator_norm] using nilpotentTwo_norm
  rw [hn]
  linarith

private theorem norm_diagonal_two (a b : ℂ) :
    ‖Matrix.diagonal ![a, b]‖ = max ‖a‖ ‖b‖ := by
  rw [Matrix.l2_opNorm_diagonal]
  apply le_antisymm
  · apply (pi_norm_le_iff_of_nonneg (le_trans (norm_nonneg a) (le_max_left _ _))).mpr
    intro i
    fin_cases i
    · exact le_max_left _ _
    · exact le_max_right _ _
  · exact max_le (norm_le_pi_norm ![a, b] 0) (norm_le_pi_norm ![a, b] 1)

/-- These formulas make the comparison norm explicit, independently of scoped instances. -/
def maxEntryNormTwo (A : Matrix (Fin 2) (Fin 2) ℂ) : ℝ :=
  max (max ‖A 0 0‖ ‖A 0 1‖) (max ‖A 1 0‖ ‖A 1 1‖)

def frobeniusNormSqTwo (A : Matrix (Fin 2) (Fin 2) ℂ) : ℝ :=
  ∑ i, ∑ j, ‖A i j‖ ^ 2

def rankOneTwo : Matrix (Fin 2) (Fin 2) ℂ := !![1, 1; 0, 0]
def weightedShiftTwo : Matrix (Fin 2) (Fin 2) ℂ := !![0, 2; 1, 0]

theorem rankOneTwo_nonnormal : rankOneTwo * rankOneTwoᴴ ≠ rankOneTwoᴴ * rankOneTwo := by
  intro h
  have := congrArg (fun A : Matrix (Fin 2) (Fin 2) ℂ ↦ A 0 0) h
  norm_num [rankOneTwo, Matrix.mul_apply, Fin.sum_univ_two] at this

theorem weightedShiftTwo_nonnormal :
    weightedShiftTwo * weightedShiftTwoᴴ ≠ weightedShiftTwoᴴ * weightedShiftTwo := by
  intro h
  have := congrArg (fun A : Matrix (Fin 2) (Fin 2) ℂ ↦ A 0 0) h
  norm_num [weightedShiftTwo, Matrix.mul_apply, Fin.sum_univ_two, map_ofNat] at this

/-- For this rank-one nonnormal matrix, the operator and Frobenius norms agree;
both differ from the maximum entry norm. Squaring avoids irrational notation. -/
theorem rankOneTwo_norms :
    ‖euclideanOperator rankOneTwo‖ ^ 2 = 2 ∧
      maxEntryNormTwo rankOneTwo = 1 ∧ frobeniusNormSqTwo rankOneTwo = 2 := by
  have hprod : rankOneTwo * rankOneTwoᴴ = Matrix.diagonal ![2, 0] := by
    ext i j
    fin_cases i <;> fin_cases j <;>
      norm_num [rankOneTwo, Matrix.mul_apply, Fin.sum_univ_two, Matrix.diagonal]
  have hnorm : ‖rankOneTwo‖ * ‖rankOneTwo‖ = 2 := by
    rw [← CStarRing.norm_self_mul_star, Matrix.star_eq_conjTranspose, hprod, norm_diagonal_two]
    norm_num
  refine ⟨?_, ?_, ?_⟩
  · simpa [pow_two, matrix_norm_eq_euclidean_operator_norm] using hnorm
  · norm_num [maxEntryNormTwo, rankOneTwo]
  · norm_num [frobeniusNormSqTwo, rankOneTwo, Fin.sum_univ_two]

/-- Here the Euclidean norm is two, while the Frobenius norm is `√5`.
The maximum entry norm happens to agree with the Euclidean norm. -/
theorem weightedShiftTwo_norms :
    ‖euclideanOperator weightedShiftTwo‖ = 2 ∧
      maxEntryNormTwo weightedShiftTwo = 2 ∧ frobeniusNormSqTwo weightedShiftTwo = 5 := by
  have hprod : weightedShiftTwo * weightedShiftTwoᴴ = Matrix.diagonal ![4, 1] := by
    ext i j
    fin_cases i <;> fin_cases j <;>
      norm_num [weightedShiftTwo, Matrix.mul_apply, Fin.sum_univ_two, Matrix.diagonal, map_ofNat]
  have hnorm : ‖weightedShiftTwo‖ * ‖weightedShiftTwo‖ = 4 := by
    rw [← CStarRing.norm_self_mul_star, Matrix.star_eq_conjTranspose, hprod, norm_diagonal_two]
    norm_num
  refine ⟨?_, ?_, ?_⟩
  · rw [← matrix_norm_eq_euclidean_operator_norm]
    nlinarith [norm_nonneg weightedShiftTwo]
  · norm_num [maxEntryNormTwo, weightedShiftTwo]
  · norm_num [frobeniusNormSqTwo, weightedShiftTwo, Fin.sum_univ_two]

theorem nilpotentTwo_X_maximum :
    maxPolynomialModulusOnNumericalRange nilpotentTwo Polynomial.X = 1 / 2 := by
  have hnorm : ‖polynomialEval Polynomial.X nilpotentTwo‖ = 1 := by
    simpa [polynomialEval, matrix_norm_eq_euclidean_operator_norm] using nilpotentTwo_norm
  have hlower := nilpotentTwo_X_bound
  change ‖polynomialEval Polynomial.X nilpotentTwo‖ ≤ _ at hlower
  rw [hnorm] at hlower
  obtain ⟨z, ⟨x, hx, rfl⟩, hmax⟩ :=
    exists_maxPolynomialModulusOnNumericalRange nilpotentTwo Polynomial.X
  have hcoords : ‖x 0‖ ^ 2 + ‖x 1‖ ^ 2 = 1 := by
    have := EuclideanSpace.norm_sq_eq x
    simpa [Fin.sum_univ_two, hx] using this.symm
  have hform : ‖⟪x, euclideanOperator nilpotentTwo x⟫_ℂ‖ = ‖x 0‖ * ‖x 1‖ := by
    rw [nilpotentTwo_action]
    simp [PiLp.inner_apply, Fin.sum_univ_two, RCLike.inner_apply, mul_comm]
  simp only [Polynomial.eval_X, hform] at hmax
  nlinarith [sq_nonneg (‖x 0‖ - ‖x 1‖)]

-- Kernel axiom closures are printed for inspection by the local audit gate.
#print axioms textbookBound_iff_mainTheoremStatement
#print axioms polynomialModulusImage_nonempty
#print axioms polynomialModulusImage_bddAbove
#print axioms polynomialModulusImage_isCompact
#print axioms polynomialModulusImage_maximum
#print axioms zeroMatrix_range
#print axioms zeroMatrix_bound
#print axioms constantPolynomial_values
#print axioms constantPolynomial_bound
#print axioms oneDimensional_eval
#print axioms oneDimensional_range
#print axioms oneDimensional_bound
#print axioms vanishing_maximum
#print axioms vanishing_eval_of_textbookBound
#print axioms nilpotentTwo_nonzero
#print axioms nilpotentTwo_square
#print axioms nilpotentTwo_action
#print axioms nilpotentTwo_norm
#print axioms nilpotentTwo_X_bound
#print axioms nilpotentTwo_X_maximum
#print axioms rankOneTwo_nonnormal
#print axioms rankOneTwo_norms
#print axioms weightedShiftTwo_nonnormal
#print axioms weightedShiftTwo_norms
#print axioms exactCorrespondence_positiveControl
#print axioms exactCorrespondence_rejectsReordered
#print axioms exactCorrespondence_rejectsPremise
#print axioms exactCorrespondence_rejectsMissingUnit
#print axioms exactCorrespondence_rejectsWrongNorm
#print axioms reorderedQuantifiers_iff
#print axioms addedTerminalPremise_true

end CrouzeixTextbook.HarpStatementAuditTests

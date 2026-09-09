import CrouzeixConjecture.Euclidean
import MathematicalFoundations.CauchySchwarz.Elementary
import MathematicalFoundations.CauchySchwarz.Equality
import MathematicalFoundations.CauchySchwarz.Quadratic
import MathematicalFoundations.Orthogonality

namespace CrouzeixTextbook.Part02

open CrouzeixConjecture
open MathematicalFoundations.Orthogonality (lineProjection)
open scoped Matrix
open scoped Matrix.Norms.L2Operator

set_option linter.defProp false

section RealGeometry

variable {n : ℕ}

/-- The real pairing is symmetric. -/
theorem real_inner_comm (x y : Fin n → ℝ) : x ⬝ᵥ y = y ⬝ᵥ x :=
  dotProduct_comm x y

/-- The real pairing is additive in its first argument. -/
theorem real_inner_add_left (x y z : Fin n → ℝ) : (x + y) ⬝ᵥ z = x ⬝ᵥ z + y ⬝ᵥ z :=
  add_dotProduct x y z

/-- The real pairing is homogeneous in its first argument. -/
theorem real_inner_smul_left (c : ℝ) (x y : Fin n → ℝ) : (c • x) ⬝ᵥ y = c * (x ⬝ᵥ y) :=
  smul_dotProduct c x y

/-- The real pairing is positive semidefinite. -/
theorem real_inner_self_nonneg (x : Fin n → ℝ) : 0 ≤ x ⬝ᵥ x := by
  simpa [dotProduct, pow_two] using
    TextbookBench.Elementary.sum_sq_nonneg Finset.univ x

/-- Positive definiteness: only the zero vector has vanishing self-pairing. -/
theorem real_inner_self_eq_zero_iff (x : Fin n → ℝ) : x ⬝ᵥ x = 0 ↔ x = 0 :=
  dotProduct_self_eq_zero

/-- CFT-07-001: every vector splits into its line projection and the residual. -/
theorem projection_residual_decomposition (u x : Fin n → ℝ) :
    x = lineProjection u x + (x - lineProjection u x) := by
  have h : lineProjection u x + (x - lineProjection u x) = x := by abel
  exact h.symm

/-- CFT-07-002: the residual is orthogonal to the line it was projected onto. -/
theorem projection_residual_orthogonal {u x : Fin n → ℝ} (hu : u ≠ 0) :
    (x - lineProjection u x) ⬝ᵥ u = 0 := by
  have hdenom : u ⬝ᵥ u ≠ 0 := fun h => hu (dotProduct_self_eq_zero.mp h)
  rw [sub_dotProduct, lineProjection, smul_dotProduct, smul_eq_mul,
    real_inner_comm x u]
  field_simp
  ring

/-- Pythagoras for the projection splitting. -/
theorem projection_pythagoras {u : Fin n → ℝ} (hu : u ≠ 0) (x : Fin n → ℝ) :
    x ⬝ᵥ x = lineProjection u x ⬝ᵥ lineProjection u x
      + (x - lineProjection u x) ⬝ᵥ (x - lineProjection u x) := by
  have hres : (x - lineProjection u x) ⬝ᵥ u = 0 := projection_residual_orthogonal hu
  obtain ⟨c, hc⟩ : ∃ c : ℝ, lineProjection u x = c • u := ⟨x ⬝ᵥ u / (u ⬝ᵥ u), rfl⟩
  have hcross : lineProjection u x ⬝ᵥ (x - lineProjection u x) = 0 := by
    nth_rewrite 1 [hc]
    rw [smul_dotProduct, smul_eq_mul, real_inner_comm u (x - lineProjection u x),
      hres, mul_zero]
  have hsplit : x = lineProjection u x + (x - lineProjection u x) := by abel
  calc x ⬝ᵥ x
      = (lineProjection u x + (x - lineProjection u x)) ⬝ᵥ
          (lineProjection u x + (x - lineProjection u x)) := by rw [← hsplit]
    _ = lineProjection u x ⬝ᵥ lineProjection u x
          + (x - lineProjection u x) ⬝ᵥ (x - lineProjection u x) := by
        rw [add_dotProduct, dotProduct_add, dotProduct_add, hcross,
          real_inner_comm (x - lineProjection u x) (lineProjection u x), hcross]
        ring

/-- The projection is the closest point of the line: any other coefficient is worse. -/
theorem projection_minimizes {u : Fin n → ℝ} (hu : u ≠ 0) (x : Fin n → ℝ) (t : ℝ) :
    (x - lineProjection u x) ⬝ᵥ (x - lineProjection u x)
      ≤ (x - t • u) ⬝ᵥ (x - t • u) := by
  set p := lineProjection u x with hp
  set r := x - p with hr
  have hres : r ⬝ᵥ u = 0 := projection_residual_orthogonal hu
  have hsub : x - t • u = r + (p - t • u) := by rw [hr]; abel
  have hpu : ∃ s : ℝ, p = s • u := ⟨x ⬝ᵥ u / (u ⬝ᵥ u), by rw [hp, lineProjection]⟩
  obtain ⟨s, hs⟩ := hpu
  have hcross : r ⬝ᵥ (p - t • u) = 0 := by
    rw [hs, ← sub_smul, dotProduct_smul, smul_eq_mul, hres, mul_zero]
  rw [hsub, add_dotProduct, dotProduct_add, dotProduct_add, hcross,
    real_inner_comm (p - t • u) r, hcross]
  have := real_inner_self_nonneg (p - t • u)
  linarith

/-- Projecting onto the zero line returns zero, so the orthogonality card genuinely
needs its nonvanishing hypothesis. -/
theorem lineProjection_zero (x : Fin n → ℝ) : lineProjection (0 : Fin n → ℝ) x = 0 := by
  simp [lineProjection]

/-- Cauchy-Schwarz in the pairing notation of this chapter. The proof is the
maintained quadratic route of the foundations laboratory, reused rather than
reproved. -/
theorem cauchy_schwarz_real (x y : Fin n → ℝ) :
    (x ⬝ᵥ y) ^ 2 ≤ (x ⬝ᵥ x) * (y ⬝ᵥ y) := by
  simpa [dotProduct, pow_two] using
    (TextbookBench.cauchySchwarzQuadratic : ∀ (m : ℕ) (a b : Fin m → ℝ), _) n x y

/-- The equality case, also reused from the maintained laboratory. -/
theorem cauchy_schwarz_real_equality_iff (x y : Fin n → ℝ) :
    (x ⬝ᵥ y) ^ 2 = (x ⬝ᵥ x) * (y ⬝ᵥ y) ↔ x = 0 ∨ ∃ t : ℝ, y = fun i => t * x i := by
  simpa [dotProduct, pow_two] using
    TextbookBench.cauchySchwarzEqualityIffProportional n x y

end RealGeometry

section HermitianBoundary

/-- CFT-07-003: the Hilbert-space pairing of a matrix action is the conjugated
coordinate pairing. -/
def adjoint_coordinate_identity := @inner_euclideanOperator_eq_star_dotProduct

/-- CFT-07-004: conjugate transpose transports to the Hilbert-space adjoint. -/
def adjoint_matrix_is_conjugate_transpose := @euclideanOperator_conjTranspose

/-- CFT-07-005: the matrix norm of this development is the induced operator norm. -/
def matrix_norm_is_operator_norm := @matrix_norm_eq_euclidean_operator_norm

/-- CFT-07-006: norms are nonnegative. -/
theorem norm_is_nonnegative {E : Type*} [SeminormedAddGroup E] (x : E) : 0 ≤ ‖x‖ := by
  have h : 0 ≤ ‖x‖ + ‖x‖ := by
    calc (0 : ℝ) = ‖x - x‖ := by simp
      _ ≤ ‖x‖ + ‖x‖ := by simp
  linarith

/-- Over the complex numbers the transpose and the conjugate transpose differ, so
the algebraic dual pullback of Chapter 4 is not the metric adjoint. -/
theorem transpose_ne_conjTranspose_complex :
    (!![Complex.I, 0; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ).transpose ≠
      (!![Complex.I, 0; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ).conjTranspose := by
  intro h
  have hentry := congrFun (congrFun h 0) 0
  simp [Matrix.transpose_apply, Matrix.conjTranspose_apply] at hentry
  simp [Complex.ext_iff] at hentry
  linarith

end HermitianBoundary

namespace Exercises.Chapter07

/-- CFT-07-E01. -/
theorem exercise_01_solution (x : Fin 2 → ℝ) :
    x = lineProjection ![1, 0] x + (x - lineProjection ![1, 0] x) ∧
      lineProjection ![1, 0] ![3, 4] = ![3, 0] := by
  refine ⟨by abel, ?_⟩
  ext i
  fin_cases i <;>
    simp [lineProjection, dotProduct, Fin.sum_univ_two]

/-- CFT-07-E02. -/
theorem exercise_02_solution :
    (![3, 4] - lineProjection ![1, 0] ![3, 4] : Fin 2 → ℝ) ⬝ᵥ ![1, 0] = 0 ∧
      (![3, 4] : Fin 2 → ℝ) ⬝ᵥ ![3, 4] = 25 := by
  constructor
  · simp [lineProjection, dotProduct, Fin.sum_univ_two]
  · simp [dotProduct, Fin.sum_univ_two]
    norm_num

/-- CFT-07-E03. -/
theorem exercise_03_solution {n : ℕ} (x y : Fin n → ℝ) (hy : y ≠ 0) :
    (x ⬝ᵥ y) ^ 2 ≤ (x ⬝ᵥ x) * (y ⬝ᵥ y) ∧
      (x - lineProjection y x) ⬝ᵥ y = 0 ∧
      x ⬝ᵥ x = lineProjection y x ⬝ᵥ lineProjection y x
        + (x - lineProjection y x) ⬝ᵥ (x - lineProjection y x) :=
  ⟨cauchy_schwarz_real x y, projection_residual_orthogonal hy,
    projection_pythagoras hy x⟩

/-- CFT-07-E04. -/
theorem exercise_04_solution {n : ℕ} (x y : Fin n → ℝ) :
    ((x ⬝ᵥ y) ^ 2 = (x ⬝ᵥ x) * (y ⬝ᵥ y) ↔ x = 0 ∨ ∃ t : ℝ, y = fun i => t * x i) ∧
      ∀ t : ℝ, ((t • x) ⬝ᵥ x) ^ 2 = ((t • x) ⬝ᵥ (t • x)) * (x ⬝ᵥ x) := by
  refine ⟨cauchy_schwarz_real_equality_iff x y, fun t => ?_⟩
  simp only [smul_dotProduct, dotProduct_smul, smul_eq_mul]
  ring

/-- CFT-07-E05. -/
theorem exercise_05_solution :
    lineProjection (0 : Fin 2 → ℝ) ![3, 4] = 0 ∧
      (!![Complex.I, 0; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ).transpose ≠
        (!![Complex.I, 0; 0, 0] : Matrix (Fin 2) (Fin 2) ℂ).conjTranspose :=
  ⟨lineProjection_zero _, transpose_ne_conjTranspose_complex⟩

/-- CFT-07-E06. -/
theorem exercise_06_solution {n : Type*} [Fintype n] [DecidableEq n]
    (A : SquareMatrix n) :
    0 ≤ ‖A‖ ∧ ‖A‖ = ‖euclideanOperator A‖ :=
  ⟨norm_nonneg A, matrix_norm_eq_euclidean_operator_norm A⟩

end Exercises.Chapter07

end CrouzeixTextbook.Part02

#check CrouzeixTextbook.Part02.Exercises.Chapter07.exercise_01_solution
#check CrouzeixTextbook.Part02.Exercises.Chapter07.exercise_02_solution
#check CrouzeixTextbook.Part02.Exercises.Chapter07.exercise_03_solution
#check CrouzeixTextbook.Part02.Exercises.Chapter07.exercise_04_solution
#check CrouzeixTextbook.Part02.Exercises.Chapter07.exercise_05_solution
#check CrouzeixTextbook.Part02.Exercises.Chapter07.exercise_06_solution

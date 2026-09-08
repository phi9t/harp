import CrouzeixTextbook.Part01.Chapter01
import Mathlib.LinearAlgebra.Determinant
import Mathlib.LinearAlgebra.Matrix.Charpoly.Coeff
import Mathlib.LinearAlgebra.Matrix.NonsingularInverse
import Mathlib.LinearAlgebra.Matrix.Trace

namespace CrouzeixTextbook.Part01

/-- CFT-05-001: determinant is multiplicative. -/
theorem determinant_multiplicative {𝕜 n : Type*} [CommRing 𝕜]
    [Fintype n] [DecidableEq n] (A B : Matrix n n 𝕜) :
    (A * B).det = A.det * B.det :=
  Matrix.det_mul A B

/-- CFT-05-002: diagonal determinant is the product of diagonal entries. -/
theorem determinant_diagonal {𝕜 n : Type*} [CommRing 𝕜]
    [Fintype n] [DecidableEq n] (d : n → 𝕜) :
    (Matrix.diagonal d).det = ∏ i, d i :=
  Matrix.det_diagonal

/-- CFT-05-003: determinant is unchanged by invertible conjugation. -/
theorem determinant_similarity {𝕜 n : Type*} [CommRing 𝕜]
    [Fintype n] [DecidableEq n] (S : (Matrix n n 𝕜)ˣ) (A : Matrix n n 𝕜) :
    (S.val * A * (↑(S⁻¹) : Matrix n n 𝕜)).det = A.det :=
  Matrix.det_units_conj S A

/-- CFT-05-004: trace is cyclic for two rectangular factors. -/
theorem trace_cyclic {𝕜 m n : Type*} [AddCommMonoid 𝕜] [CommMagma 𝕜]
    [Fintype m] [Fintype n] (A : Matrix m n 𝕜) (B : Matrix n m 𝕜) :
    (A * B).trace = (B * A).trace :=
  Matrix.trace_mul_comm A B

/-- CFT-05-005: trace is unchanged by invertible conjugation. -/
theorem trace_similarity {𝕜 n : Type*} [CommSemiring 𝕜]
    [Fintype n] [DecidableEq n] (S : (Matrix n n 𝕜)ˣ) (A : Matrix n n 𝕜) :
    (S.val * A * (↑(S⁻¹) : Matrix n n 𝕜)).trace = A.trace :=
  Matrix.trace_units_conj S A

/-- CFT-05-006: diagonal trace is the sum of diagonal entries. -/
theorem trace_diagonal {𝕜 n : Type*} [AddCommMonoid 𝕜]
    [Fintype n] [DecidableEq n] (d : n → 𝕜) :
    (Matrix.diagonal d).trace = ∑ i, d i :=
  Matrix.trace_diagonal d

section DeterminantSupport

variable {𝕜 n : Type*} [CommRing 𝕜] [Fintype n] [DecidableEq n]

/-- Normalization: the identity operator scales volume by one. -/
theorem determinant_normalized : (1 : Matrix n n 𝕜).det = 1 :=
  Matrix.det_one

/-- Alternation: two equal rows force the determinant to vanish. -/
theorem determinant_repeated_row {A : Matrix n n 𝕜} {i j : n}
    (hij : i ≠ j) (h : A i = A j) : A.det = 0 :=
  Matrix.det_zero_of_row_eq hij h

/-- Antisymmetry: permuting the rows multiplies the determinant by the sign. -/
theorem determinant_permuted (σ : Equiv.Perm n) (A : Matrix n n 𝕜) :
    (A.submatrix σ id).det = Equiv.Perm.sign σ * A.det :=
  Matrix.det_permute σ A

/-- Additivity in one row; the other rows are held fixed. -/
theorem determinant_row_additive (A : Matrix n n 𝕜) (j : n) (u v : n → 𝕜) :
    (A.updateRow j (u + v)).det = (A.updateRow j u).det + (A.updateRow j v).det :=
  Matrix.det_updateRow_add A j u v

/-- Homogeneity in one row; this is the second half of multilinearity. -/
theorem determinant_row_homogeneous (A : Matrix n n 𝕜) (j : n) (s : 𝕜) (u : n → 𝕜) :
    (A.updateRow j (s • u)).det = s * (A.updateRow j u).det :=
  Matrix.det_updateRow_smul A j s u

/-- The Leibniz expansion that the normalization above pins down uniquely. -/
theorem determinant_leibniz (A : Matrix n n 𝕜) :
    A.det = ∑ σ : Equiv.Perm n, Equiv.Perm.sign σ • ∏ i, A (σ i) i :=
  Matrix.det_apply A

/-- Transposition leaves the determinant unchanged, so rows and columns agree. -/
theorem determinant_transpose (A : Matrix n n 𝕜) : A.transpose.det = A.det :=
  Matrix.det_transpose A

/-- Invertibility criterion: a square matrix is a unit exactly when its determinant is. -/
theorem determinant_unit_criterion (A : Matrix n n 𝕜) :
    IsUnit A ↔ IsUnit A.det :=
  Matrix.isUnit_iff_isUnit_det A

/-- Over a field the criterion becomes the familiar nonvanishing test. -/
theorem determinant_field_criterion {K : Type*} [Field K] (A : Matrix n n K) :
    IsUnit A ↔ A.det ≠ 0 := by
  rw [Matrix.isUnit_iff_isUnit_det, isUnit_iff_ne_zero]

end DeterminantSupport

section ExteriorAction

variable {A M ι : Type*} [CommRing A] [AddCommGroup M] [Module A M]
  [Fintype ι] [DecidableEq ι]

/-- Top-degree exterior action. Every alternating `ι`-form on `M` is rescaled by
`LinearMap.det T` when its inputs are pushed through `T`. The proof uses only the
one-dimensionality of the space of top alternating forms. -/
theorem alternating_form_scaled_by_determinant (e : Module.Basis ι A M)
    (f : M [⋀^ι]→ₗ[A] A) (T : M →ₗ[A] M) (v : ι → M) :
    f (T ∘ v) = LinearMap.det T * f v := by
  have h := f.eq_smul_basis_det e
  calc f (T ∘ v) = (f e • e.det) (T ∘ v) := by rw [← h]
    _ = f e * e.det (T ∘ v) := by simp
    _ = f e * (LinearMap.det T * e.det v) := by rw [e.det_comp]
    _ = LinearMap.det T * (f e • e.det) v := by simp; ring
    _ = LinearMap.det T * f v := by rw [← h]

end ExteriorAction

section TraceSupport

variable {𝕜 n : Type*} [CommRing 𝕜] [Fintype n] [DecidableEq n]

omit [DecidableEq n] in
/-- Trace is linear; determinant is not. -/
theorem trace_linear (A B : Matrix n n 𝕜) (c : 𝕜) :
    (A + c • B).trace = A.trace + c * B.trace := by
  rw [Matrix.trace_add, Matrix.trace_smul, smul_eq_mul]

/-- Cyclicity is strictly weaker than multiplicativity. -/
theorem trace_not_multiplicative :
    ∃ A B : Matrix (Fin 2) (Fin 2) ℝ, (A * B).trace ≠ A.trace * B.trace :=
  ⟨1, 1, by norm_num [Matrix.trace_fin_two]⟩

/-- First-order volume expansion in dimension two: the linear coefficient is the trace. -/
theorem determinant_one_add_smul_fin_two (t : 𝕜) (A : Matrix (Fin 2) (Fin 2) 𝕜) :
    (1 + t • A).det = 1 + t * A.trace + t ^ 2 * A.det := by
  simp [Matrix.det_fin_two, Matrix.trace_fin_two]
  ring

end TraceSupport

section RunningFamily

/-- Determinant of the cumulative running family. -/
theorem runningFamily_det (lam α : ℝ) : (runningFamily lam α).det = lam ^ 2 := by
  simp [runningFamily, Matrix.det_fin_two_of]
  ring

/-- Trace of the cumulative running family. -/
theorem runningFamily_trace (lam α : ℝ) : (runningFamily lam α).trace = 2 * lam := by
  simp [runningFamily, Matrix.trace_fin_two_of]
  ring

/-- Both scalars are blind to the shear parameter that changes the family's geometry. -/
theorem runningFamily_scalars_ignore_shear (lam α β : ℝ) :
    (runningFamily lam α).det = (runningFamily lam β).det ∧
    (runningFamily lam α).trace = (runningFamily lam β).trace := by
  rw [runningFamily_det, runningFamily_det, runningFamily_trace, runningFamily_trace]
  exact ⟨rfl, rfl⟩

end RunningFamily

namespace Exercises.Chapter05

/-- CFT-05-E01. -/
theorem exercise_01_solution (a b : ℝ) :
    (!![a, b; a, b] : Matrix (Fin 2) (Fin 2) ℝ).det = 0 ∧
    (!![a, b; 1, 2] : Matrix (Fin 2) (Fin 2) ℝ).det =
      -(!![1, 2; a, b] : Matrix (Fin 2) (Fin 2) ℝ).det := by
  refine ⟨?_, ?_⟩
  · simp [Matrix.det_fin_two_of]
    ring
  · simp [Matrix.det_fin_two_of]
    ring

/-- CFT-05-E02. -/
theorem exercise_02_solution :
    (!![2, 5, -1; 0, 3, 4; 0, 0, -2] : Matrix (Fin 3) (Fin 3) ℝ).det = -12 ∧
    (!![2, 5, -1; 0, 3, 4; 0, 0, -2] : Matrix (Fin 3) (Fin 3) ℝ).trace = 3 := by
  refine ⟨?_, ?_⟩
  · simp [Matrix.det_fin_three]
    norm_num
  · simp [Matrix.trace_fin_three]

/-- CFT-05-E03. -/
theorem exercise_03_solution {𝕜 n : Type*} [CommRing 𝕜] [Fintype n] [DecidableEq n]
    (S R A : Matrix n n 𝕜) (h : R * S = 1) :
    (S * A * R).det = A.det := by
  have hRS : R.det * S.det = 1 := by
    have hdet := congrArg Matrix.det h
    rwa [Matrix.det_mul, Matrix.det_one] at hdet
  calc (S * A * R).det = S.det * A.det * R.det := by
        rw [Matrix.det_mul, Matrix.det_mul]
    _ = R.det * S.det * A.det := by ring
    _ = A.det := by rw [hRS, one_mul]

/-- CFT-05-E04. -/
theorem exercise_04_solution {𝕜 m n : Type*} [CommRing 𝕜] [Fintype m] [Fintype n]
    (A : Matrix m n 𝕜) (B : Matrix n m 𝕜) :
    (A * B).trace = ∑ i, ∑ j, A i j * B j i ∧
      (B * A).trace = ∑ j, ∑ i, B j i * A i j ∧
      (A * B).trace = (B * A).trace := by
  have hAB : (A * B).trace = ∑ i, ∑ j, A i j * B j i := by
    simp [Matrix.trace, Matrix.diag, Matrix.mul_apply]
  have hBA : (B * A).trace = ∑ j, ∑ i, B j i * A i j := by
    simp [Matrix.trace, Matrix.diag, Matrix.mul_apply]
  refine ⟨hAB, hBA, ?_⟩
  rw [hAB, hBA, Finset.sum_comm]
  exact Finset.sum_congr rfl fun j _ => Finset.sum_congr rfl fun i _ => mul_comm _ _

/-- CFT-05-E05. -/
theorem exercise_05_solution {𝕜 n : Type*} [CommRing 𝕜] [Fintype n] [DecidableEq n]
    (A B C : Matrix n n 𝕜) :
    (A * B * C).trace = (B * C * A).trace ∧
      (B * C * A).trace = (C * A * B).trace ∧
      ∃ X Y Z : Matrix (Fin 2) (Fin 2) ℝ, (X * Y * Z).trace ≠ (X * Z * Y).trace := by
  refine ⟨?_, ?_, ?_⟩
  · rw [Matrix.mul_assoc, Matrix.trace_mul_comm]
  · rw [Matrix.mul_assoc, Matrix.trace_mul_comm]
  · refine ⟨!![1, 0; 0, 0], !![0, 1; 0, 0], !![0, 0; 1, 0], ?_⟩
    norm_num [Matrix.trace_fin_two, Matrix.mul_fin_two]

/-- CFT-05-E06. -/
theorem exercise_06_solution {𝕜 n : Type*} [CommRing 𝕜] [Fintype n] [DecidableEq n]
    (S R A : Matrix n n 𝕜) (h : R * S = 1) :
    (S * A * R).trace = A.trace := by
  calc (S * A * R).trace = (R * (S * A)).trace := by rw [Matrix.trace_mul_comm]
    _ = (R * S * A).trace := by rw [Matrix.mul_assoc]
    _ = A.trace := by rw [h, Matrix.one_mul]

end Exercises.Chapter05

end CrouzeixTextbook.Part01

#check CrouzeixTextbook.Part01.Exercises.Chapter05.exercise_01_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter05.exercise_02_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter05.exercise_03_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter05.exercise_04_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter05.exercise_05_solution
#check CrouzeixTextbook.Part01.Exercises.Chapter05.exercise_06_solution

import Mathlib.LinearAlgebra.Matrix.Charpoly.Coeff
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

end CrouzeixTextbook.Part01

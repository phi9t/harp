import Mathlib.Algebra.BigOperators.Fin
import Mathlib.Data.Matrix.Basic

/-!
Finite-coordinate autodiff notation contracts.

This first phase formalizes the algebraic objects named by the JAX autodiff
cookbook: component gradients, JVPs, VJPs, and Hessian-vector products.  It
does not model JAX tracing, floating-point execution, pytrees, or analytic
differentiability.
-/

namespace AutodiffGeometry

open BigOperators

variable {R : Type*}

/-- A finite coordinate vector. -/
abbrev Vec (n : Nat) (R : Type*) := Fin n -> R

/-- A finite coordinate matrix with output rows and input columns. -/
abbrev Mat (m n : Nat) (R : Type*) := Fin m -> Fin n -> R

/-- `grad(f, i)` corresponds to the `i`th component of the gradient. -/
def gradArg {n : Nat} (gradient : Vec n R) (i : Fin n) : R :=
  gradient i

/-- The component selected by `grad(f, i)` is exactly the `i`th gradient coordinate. -/
theorem grad_arg_eq_component {n : Nat} (gradient : Vec n R) (i : Fin n) :
    gradArg gradient i = gradient i := by
  rfl

variable [CommSemiring R]

/-- Dot product for finite coordinate vectors. -/
def dot {n : Nat} (x y : Vec n R) : R :=
  ∑ i, x i * y i

/-- Matrix-vector multiplication in the convention `J x` maps input tangents to outputs. -/
def matVec {m n : Nat} (A : Mat m n R) (x : Vec n R) : Vec m R :=
  fun i => ∑ j, A i j * x j

/-- Transpose of a finite coordinate matrix. -/
def transpose {m n : Nat} (A : Mat m n R) : Mat n m R :=
  fun j i => A i j

/-- A JVP applies the derivative matrix to an input tangent. -/
def jvp {m n : Nat} (jacobian : Mat m n R) (tangent : Vec n R) : Vec m R :=
  matVec jacobian tangent

/-- A VJP applies the transpose derivative matrix to an output cotangent. -/
def vjp {m n : Nat} (jacobian : Mat m n R) (cotangent : Vec m R) : Vec n R :=
  matVec (transpose jacobian) cotangent

/-- A Hessian-vector product applies the Hessian matrix to a vector. -/
def hvp {n : Nat} (hessian : Mat n n R) (v : Vec n R) : Vec n R :=
  matVec hessian v

/-- A JVP is matrix-vector multiplication by the Jacobian. -/
theorem jvp_eq_matVec {m n : Nat} (jacobian : Mat m n R) (tangent : Vec n R) :
    jvp jacobian tangent = matVec jacobian tangent := by
  rfl

/-- A VJP is matrix-vector multiplication by the transposed Jacobian. -/
theorem vjp_eq_transpose_matVec {m n : Nat} (jacobian : Mat m n R) (cotangent : Vec m R) :
    vjp jacobian cotangent = matVec (transpose jacobian) cotangent := by
  rfl

/-- A Hessian-vector product is matrix-vector multiplication by the Hessian. -/
theorem hvp_eq_matVec {n : Nat} (hessian : Mat n n R) (v : Vec n R) :
    hvp hessian v = matVec hessian v := by
  rfl

/-- JVP and VJP are dual under the finite-coordinate dot product. -/
theorem dot_jvp_eq_dot_vjp {m n : Nat} (jacobian : Mat m n R)
    (tangent : Vec n R) (cotangent : Vec m R) :
    dot (jvp jacobian tangent) cotangent = dot tangent (vjp jacobian cotangent) := by
  classical
  simp [dot, jvp, vjp, matVec, transpose, Finset.sum_mul, Finset.mul_sum]
  rw [Finset.sum_comm]
  simp [mul_left_comm, mul_comm]

end AutodiffGeometry

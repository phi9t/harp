import AutodiffGeometry.FiniteCoordinates

namespace CrouzeixTextbook.Part02

open scoped Matrix

set_option linter.defProp false in
def gradient_component_contract := @AutodiffGeometry.grad_arg_eq_component
set_option linter.defProp false in
def derivative_action_is_jvp := @AutodiffGeometry.jvp_eq_matVec
set_option linter.defProp false in
def derivative_pullback_is_vjp := @AutodiffGeometry.vjp_eq_transpose_matVec
set_option linter.defProp false in
def hessian_vector_action := @AutodiffGeometry.hvp_eq_matVec
set_option linter.defProp false in
def jvp_vjp_duality := @AutodiffGeometry.dot_jvp_eq_dot_vjp

theorem linear_approximation_chain_kernel {R : Type*} [Semiring R]
    {m n p : Nat} (A : Matrix (Fin m) (Fin n) R) (B : Matrix (Fin p) (Fin m) R)
    (v : Fin n → R) : B *ᵥ (A *ᵥ v) = (B * A) *ᵥ v :=
  Matrix.mulVec_mulVec v B A

end CrouzeixTextbook.Part02

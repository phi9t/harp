import CrouzeixTextbook.Part02.Chapter10
import Mathlib.Analysis.Calculus.Deriv.Mul
import Mathlib.Analysis.Normed.Module.FiniteDimension

namespace CrouzeixTextbook.Part02

open scoped Matrix

section Frechet

variable {E F G : Type*}
  [NormedAddCommGroup E] [NormedSpace ℝ E]
  [NormedAddCommGroup F] [NormedSpace ℝ F]
  [NormedAddCommGroup G] [NormedSpace ℝ G]

/-- The Fréchet derivative is unique: two bounded linear maps that both approximate
`f` to first order at `x` are equal. This is what makes "the" derivative a
definition rather than a choice. -/
theorem frechet_derivative_unique {f : E → F} {f' g' : E →L[ℝ] F} {x : E}
    (hf : HasFDerivAt f f' x) (hg : HasFDerivAt f g' x) : f' = g' :=
  hf.unique hg

/-- A bounded linear map is its own derivative at every point: its first-order
approximation is exact, so the remainder is identically zero. -/
theorem continuous_linear_map_hasFDerivAt (L : E →L[ℝ] F) (x : E) :
    HasFDerivAt (fun w => L w) L x :=
  L.hasFDerivAt

/-- The chain rule: derivatives compose as bounded linear maps. -/
theorem frechet_chain_rule {f : E → F} {g : F → G} {f' : E →L[ℝ] F} {g' : F →L[ℝ] G}
    {x : E} (hg : HasFDerivAt g g' (f x)) (hf : HasFDerivAt f f' x) :
    HasFDerivAt (g ∘ f) (g'.comp f') x :=
  hg.comp x hf

end Frechet

section CoordinateDerivatives

variable {m n p : ℕ}

/-- The Jacobian read as a bounded linear map on coordinate space. In finite
dimensions every linear map is bounded, so this carries no analytic hypothesis. -/
noncomputable def jacobianAction (A : Matrix (Fin m) (Fin n) ℝ) :
    (Fin n → ℝ) →L[ℝ] (Fin m → ℝ) :=
  LinearMap.toContinuousLinearMap A.mulVecLin

@[simp]
theorem jacobianAction_apply (A : Matrix (Fin m) (Fin n) ℝ) (v : Fin n → ℝ) :
    jacobianAction A v = A *ᵥ v := rfl

/-- CFT-11-001: a gradient's `i`th component is what the covector reports on the
`i`th coordinate direction. The pairing with a standard basis vector selects a
component, so "the `i`th partial derivative" and "the `i`th entry of the gradient"
name the same number. -/
theorem gradient_component_contract (g : Fin n → ℝ) (i : Fin n) :
    g ⬝ᵥ Pi.single i 1 = g i ∧
      HasDerivAt (fun t : ℝ => g ⬝ᵥ (t • Pi.single i 1)) (g i) 0 := by
  have hsel : g ⬝ᵥ Pi.single i 1 = g i := by
    simp [dotProduct, Pi.single_apply, Finset.sum_ite_eq']
  refine ⟨hsel, ?_⟩
  have h : (fun t : ℝ => g ⬝ᵥ (t • Pi.single i 1)) = fun t : ℝ => t * g i := by
    funext t
    rw [dotProduct_smul, smul_eq_mul, hsel]
  rw [h]
  simpa using (hasDerivAt_id (0 : ℝ)).mul_const (g i)

/-- CFT-11-002: the derivative of a matrix action is that action, and evaluating it
on a tangent is the Jacobian-vector product. Forward mode never forms the Jacobian;
this is the identity that says it does not have to. -/
theorem derivative_action_is_jvp (A : Matrix (Fin m) (Fin n) ℝ) (x v : Fin n → ℝ) :
    HasFDerivAt (fun w : Fin n → ℝ => A *ᵥ w) (jacobianAction A) x ∧
      jacobianAction A v = A *ᵥ v :=
  ⟨(jacobianAction A).hasFDerivAt, rfl⟩

/-- CFT-11-003: pulling a cotangent back through the derivative is the transpose
action. This is the vector-Jacobian product of reverse mode. -/
theorem derivative_pullback_is_vjp (A : Matrix (Fin m) (Fin n) ℝ) (y : Fin m → ℝ) :
    y ᵥ* A = A.transpose *ᵥ y ∧ jacobianAction A.transpose y = A.transpose *ᵥ y :=
  ⟨transpose_coordinate_action A y, rfl⟩

/-- CFT-11-005: forward and reverse mode are adjoint, and the transpose is the
*only* matrix for which that holds. The uniqueness clause is why reverse mode
computes the adjoint rather than merely something adjoint-like. -/
theorem jvp_vjp_duality (A : Matrix (Fin m) (Fin n) ℝ) :
    (∀ (v : Fin n → ℝ) (y : Fin m → ℝ),
        (A *ᵥ v) ⬝ᵥ y = v ⬝ᵥ (A.transpose *ᵥ y)) ∧
      ∀ B : Matrix (Fin n) (Fin m) ℝ,
        (∀ (v : Fin n → ℝ) (y : Fin m → ℝ), (A *ᵥ v) ⬝ᵥ y = v ⬝ᵥ (B *ᵥ y)) →
          B = A.transpose := by
  refine ⟨fun v y => bilinear_pairing_duality A v y, fun B hB => ?_⟩
  ext i j
  have h := hB (Pi.single i 1) (Pi.single j 1)
  rw [bilinear_pairing_duality] at h
  simpa [dotProduct, Matrix.mulVec, Pi.single_apply, Finset.sum_ite_eq',
    Matrix.transpose_apply] using h.symm

/-- CFT-11-006: composing two coordinate derivative actions is the action of the
matrix product. This is the chain rule once the derivatives are in coordinates. -/
theorem linear_approximation_chain_kernel {R : Type*} [Semiring R]
    {m n p : Nat} (A : Matrix (Fin m) (Fin n) R) (B : Matrix (Fin p) (Fin m) R)
    (v : Fin n → R) : B *ᵥ (A *ᵥ v) = (B * A) *ᵥ v :=
  Matrix.mulVec_mulVec v B A

/-- The chain rule in coordinates is matrix multiplication: the Fréchet composite of
two matrix actions has derivative the action of the product. This is the theorem the
chapter exists to state. -/
theorem jacobianAction_comp (A : Matrix (Fin m) (Fin n) ℝ)
    (B : Matrix (Fin p) (Fin m) ℝ) :
    (jacobianAction B).comp (jacobianAction A) = jacobianAction (B * A) := by
  ext v i
  simp [Matrix.mulVec_mulVec]

theorem chain_rule_in_coordinates_is_matrix_product
    (A : Matrix (Fin m) (Fin n) ℝ) (B : Matrix (Fin p) (Fin m) ℝ) (x : Fin n → ℝ) :
    HasFDerivAt ((fun w : Fin m → ℝ => B *ᵥ w) ∘ (fun w : Fin n → ℝ => A *ᵥ w))
      (jacobianAction (B * A)) x := by
  rw [← jacobianAction_comp A B]
  exact frechet_chain_rule ((jacobianAction B).hasFDerivAt) ((jacobianAction A).hasFDerivAt)

end CoordinateDerivatives

section Hessian

variable {n : ℕ}

/-- The quadratic form attached to a matrix, in coordinates. -/
noncomputable def quadraticForm (H : Matrix (Fin n) (Fin n) ℝ) (x : Fin n → ℝ) : ℝ :=
  (H *ᵥ x) ⬝ᵥ x

/-- For a symmetric `H` the two cross terms of the quadratic form's expansion agree,
so the first-order term is `2 ⟪Hx, w⟫`. Symmetry is exactly what collapses them. -/
theorem symmetric_cross_terms (H : Matrix (Fin n) (Fin n) ℝ)
    (hH : H.transpose = H) (x w : Fin n → ℝ) :
    (H *ᵥ x) ⬝ᵥ w + (H *ᵥ w) ⬝ᵥ x = 2 * ((H *ᵥ x) ⬝ᵥ w) := by
  have h : (H *ᵥ w) ⬝ᵥ x = (H *ᵥ x) ⬝ᵥ w := by
    rw [bilinear_pairing_duality, hH, dotProduct_comm]
  rw [h]; ring

/-- CFT-11-004: for a symmetric `H` the gradient of the quadratic form is `2Hx`, and
the derivative of that gradient sends a direction `v` to `2Hv`. The Hessian-vector
product is a matrix action, which is why it costs one linear solve and not `n`. -/
theorem hessian_vector_action (H : Matrix (Fin n) (Fin n) ℝ)
    (hH : H.transpose = H) (x v : Fin n → ℝ) :
    (∀ w : Fin n → ℝ,
        (H *ᵥ x) ⬝ᵥ w + (H *ᵥ w) ⬝ᵥ x = ((2 : ℝ) • (H *ᵥ x)) ⬝ᵥ w) ∧
      HasFDerivAt (fun z : Fin n → ℝ => (2 : ℝ) • (H *ᵥ z))
        (jacobianAction ((2 : ℝ) • H)) x ∧
      jacobianAction ((2 : ℝ) • H) v = (2 : ℝ) • (H *ᵥ v) := by
  have hfun : (fun z : Fin n → ℝ => (2 : ℝ) • (H *ᵥ z))
      = fun z : Fin n → ℝ => ((2 : ℝ) • H) *ᵥ z := by
    funext z; rw [Matrix.smul_mulVec]
  refine ⟨fun w => ?_, ?_, ?_⟩
  · rw [symmetric_cross_terms H hH x w, smul_dotProduct, smul_eq_mul]
  · rw [hfun]
    exact (jacobianAction ((2 : ℝ) • H)).hasFDerivAt
  · rw [jacobianAction_apply, Matrix.smul_mulVec]

end Hessian

section DirectionalButNotFrechet

/-- The standard cusp: homogeneous of degree one, hence directionally
differentiable in every direction at the origin, but its directional derivative is
not additive. -/
noncomputable def coneCusp (p : Fin 2 → ℝ) : ℝ :=
  if p 0 = 0 ∧ p 1 = 0 then 0 else (p 0) ^ 3 / ((p 0) ^ 2 + (p 1) ^ 2)

/-- `coneCusp` is positively and negatively homogeneous of degree one, so along every
line through the origin it is exactly linear. -/
theorem coneCusp_homogeneous (t : ℝ) (p : Fin 2 → ℝ) :
    coneCusp (t • p) = t * coneCusp p := by
  by_cases hp : p 0 = 0 ∧ p 1 = 0
  · have h0 : (t • p) 0 = 0 := by simp [hp.1]
    have h1 : (t • p) 1 = 0 := by simp [hp.2]
    simp [coneCusp, hp, h0, h1]
  · by_cases ht : t = 0
    · subst ht
      simp [coneCusp]
    · have hden : (p 0) ^ 2 + (p 1) ^ 2 ≠ 0 := by
        intro h
        exact hp ⟨by nlinarith [sq_nonneg (p 0), sq_nonneg (p 1)],
          by nlinarith [sq_nonneg (p 0), sq_nonneg (p 1)]⟩
      have hne : ¬ ((t • p) 0 = 0 ∧ (t • p) 1 = 0) := by
        rintro ⟨h0, h1⟩
        simp only [Pi.smul_apply, smul_eq_mul, mul_eq_zero] at h0 h1
        exact hp ⟨h0.resolve_left ht, h1.resolve_left ht⟩
      have ht2 : t ^ 2 ≠ 0 := pow_ne_zero 2 ht
      have hd : ((t • p) 0) ^ 2 + ((t • p) 1) ^ 2
          = t ^ 2 * ((p 0) ^ 2 + (p 1) ^ 2) := by
        simp only [Pi.smul_apply, smul_eq_mul]; ring
      have hnum : ((t • p) 0) ^ 3 = t ^ 2 * (t * (p 0) ^ 3) := by
        simp only [Pi.smul_apply, smul_eq_mul]; ring
      rw [coneCusp, coneCusp, if_neg hne, if_neg hp, hd, hnum,
        mul_div_mul_left _ _ ht2, mul_div_assoc]

/-- The directional derivative of `coneCusp` at the origin is `coneCusp` itself. -/
theorem coneCusp_hasDerivAt_along (v : Fin 2 → ℝ) :
    HasDerivAt (fun t : ℝ => coneCusp (t • v)) (coneCusp v) 0 := by
  have h : (fun t : ℝ => coneCusp (t • v)) = fun t : ℝ => t * coneCusp v := by
    funext t; exact coneCusp_homogeneous t v
  rw [h]
  simpa using (hasDerivAt_id (0 : ℝ)).mul_const (coneCusp v)

/-- The directional derivative is not additive: it reports `1` along `e₀`, `0` along
`e₁`, and `1/2` along their sum. -/
theorem coneCusp_not_additive :
    coneCusp ![1, 1] ≠ coneCusp ![1, 0] + coneCusp ![0, 1] := by
  norm_num [coneCusp]

/-- Hence `coneCusp` has no Fréchet derivative at the origin, even though every
directional derivative exists. Directional differentiability in every direction is
strictly weaker than differentiability. -/
theorem coneCusp_not_frechet_differentiable :
    ¬ ∃ L : (Fin 2 → ℝ) →L[ℝ] ℝ, HasFDerivAt coneCusp L 0 := by
  rintro ⟨L, hL⟩
  have key : ∀ v : Fin 2 → ℝ, L v = coneCusp v := by
    intro v
    have hline : HasFDerivAt (fun t : ℝ => t • v)
        (ContinuousLinearMap.smulRight (1 : ℝ →L[ℝ] ℝ) v) (0 : ℝ) :=
      (ContinuousLinearMap.smulRight (1 : ℝ →L[ℝ] ℝ) v).hasFDerivAt
    have h1 : HasFDerivAt (coneCusp ∘ fun t : ℝ => t • v)
        (L.comp (ContinuousLinearMap.smulRight (1 : ℝ →L[ℝ] ℝ) v)) (0 : ℝ) :=
      HasFDerivAt.comp (0 : ℝ) (by simpa using hL) hline
    have h2 : HasFDerivAt (coneCusp ∘ fun t : ℝ => t • v)
        (ContinuousLinearMap.smulRight (1 : ℝ →L[ℝ] ℝ) (coneCusp v)) (0 : ℝ) :=
      hasDerivAt_iff_hasFDerivAt.mp (coneCusp_hasDerivAt_along v)
    have heq := h1.unique h2
    have happ := congrArg (fun T : ℝ →L[ℝ] ℝ => T 1) heq
    simpa using happ
  have hsum : (![1, 1] : Fin 2 → ℝ) = ![1, 0] + ![0, 1] := by
    ext i; fin_cases i <;> simp
  apply coneCusp_not_additive
  rw [← key, ← key, ← key, hsum, map_add]

end DirectionalButNotFrechet

namespace Exercises.Chapter11

/-- CFT-11-E01. -/
theorem exercise_01_solution {E F : Type*}
    [NormedAddCommGroup E] [NormedSpace ℝ E] [NormedAddCommGroup F] [NormedSpace ℝ F]
    (L : E →L[ℝ] F) (x : E) :
    HasFDerivAt (fun w => L w) L x ∧
      HasFDerivAt (fun w : E => w) (ContinuousLinearMap.id ℝ E) x :=
  ⟨L.hasFDerivAt, (ContinuousLinearMap.id ℝ E).hasFDerivAt⟩

/-- CFT-11-E02. -/
theorem exercise_02_solution {n : ℕ} (x w : Fin n → ℝ) :
    ((1 : Matrix (Fin n) (Fin n) ℝ) *ᵥ x) ⬝ᵥ w
        + ((1 : Matrix (Fin n) (Fin n) ℝ) *ᵥ w) ⬝ᵥ x
      = 2 * (x ⬝ᵥ w) := by
  rw [Matrix.one_mulVec, Matrix.one_mulVec, dotProduct_comm w x]
  ring

/-- CFT-11-E03. -/
theorem exercise_03_solution {E F : Type*}
    [NormedAddCommGroup E] [NormedSpace ℝ E] [NormedAddCommGroup F] [NormedSpace ℝ F]
    {f : E → F} {f' g' : E →L[ℝ] F} {x : E}
    (hf : HasFDerivAt f f' x) (hg : HasFDerivAt f g' x) :
    f' = g' ∧ ∀ v : E, f' v = g' v := by
  have h := hf.unique hg
  exact ⟨h, fun v => by rw [h]⟩

/-- CFT-11-E04. -/
theorem exercise_04_solution {m n p : ℕ}
    (A : Matrix (Fin m) (Fin n) ℝ) (B : Matrix (Fin p) (Fin m) ℝ) (x v : Fin n → ℝ) :
    HasFDerivAt ((fun w : Fin m → ℝ => B *ᵥ w) ∘ (fun w : Fin n → ℝ => A *ᵥ w))
        (jacobianAction (B * A)) x ∧
      B *ᵥ (A *ᵥ v) = (B * A) *ᵥ v :=
  ⟨chain_rule_in_coordinates_is_matrix_product A B x,
    linear_approximation_chain_kernel A B v⟩

/-- CFT-11-E05. -/
theorem exercise_05_solution :
    (∀ (t : ℝ) (p : Fin 2 → ℝ), coneCusp (t • p) = t * coneCusp p) ∧
      coneCusp ![1, 1] ≠ coneCusp ![1, 0] + coneCusp ![0, 1] ∧
      ¬ ∃ L : (Fin 2 → ℝ) →L[ℝ] ℝ, HasFDerivAt coneCusp L 0 :=
  ⟨coneCusp_homogeneous, coneCusp_not_additive, coneCusp_not_frechet_differentiable⟩

/-- CFT-11-E06. -/
theorem exercise_06_solution {m n : ℕ} (A : Matrix (Fin m) (Fin n) ℝ)
    (v : Fin n → ℝ) (y : Fin m → ℝ) :
    (A *ᵥ v) ⬝ᵥ y = v ⬝ᵥ (A.transpose *ᵥ y) ∧
      ∀ B : Matrix (Fin n) (Fin m) ℝ,
        (∀ (v : Fin n → ℝ) (y : Fin m → ℝ), (A *ᵥ v) ⬝ᵥ y = v ⬝ᵥ (B *ᵥ y)) →
          B = A.transpose :=
  ⟨(jvp_vjp_duality A).1 v y, (jvp_vjp_duality A).2⟩

end Exercises.Chapter11

end CrouzeixTextbook.Part02

import CrouzeixConjecture.CanonicalParallelRadialGeometry
import CrouzeixConjecture.ConvexProjection
import Mathlib.Analysis.Calculus.Deriv.Mul
import Mathlib.Analysis.Calculus.TangentCone.Real

namespace CrouzeixTextbook.Part05
open CrouzeixConjecture Filter Set
open scoped InnerProductSpace Topology

noncomputable section

/-- Existence and uniqueness of the metric projection are separate claims:
closedness gives completeness and hence a minimizer, while convexity makes it unique. -/
theorem convex_projection
    (K : Set ℂ) (hKne : K.Nonempty) (hKclosed : IsClosed K)
    (hKconvex : Convex ℝ K) (z : ℂ) :
    ∃! p : ℂ, p ∈ K ∧ ∀ w ∈ K, ‖z - p‖ ≤ ‖z - w‖ := by
  rcases exists_norm_eq_iInf_of_complete_convex
      hKne hKclosed.isComplete hKconvex z with ⟨p, hpK, hpiInf⟩
  refine ⟨p, ?_, ?_⟩
  · refine ⟨hpK, ?_⟩
    intro w hw
    rw [hpiInf]
    exact ciInf_le ⟨0, Set.forall_mem_range.2 fun _ ↦ norm_nonneg _⟩
      (⟨w, hw⟩ : K)
  · rintro q ⟨hqK, hqmin⟩
    letI : Nonempty K := hKne.to_subtype
    have hqiInf : ‖z - q‖ = ⨅ w : K, ‖z - w‖ := by
      apply le_antisymm
      · exact le_ciInf fun (w : K) ↦ hqmin w w.property
      · exact ciInf_le ⟨0, Set.forall_mem_range.2 fun _ ↦ norm_nonneg _⟩
          (⟨q, hqK⟩ : K)
    have hpvar : ∀ w ∈ K, ⟪z - p, w - p⟫_ℝ ≤ 0 :=
      (norm_eq_iInf_iff_real_inner_le_zero hKconvex hpK).mp hpiInf
    have hqvar : ∀ w ∈ K, ⟪z - q, w - q⟫_ℝ ≤ 0 :=
      (norm_eq_iInf_iff_real_inner_le_zero hKconvex hqK).mp hqiInf
    have hself : ⟪p - q, p - q⟫_ℝ ≤ 0 := by
      calc
        ⟪p - q, p - q⟫_ℝ =
            ⟪z - p, q - p⟫_ℝ + ⟪z - q, p - q⟫_ℝ := by
          simp only [inner_sub_left, inner_sub_right]
          ring
        _ ≤ 0 := add_nonpos (hpvar q hqK) (hqvar p hpK)
    exact (sub_eq_zero.mp (real_inner_self_nonpos.mp hself)).symm

/-- The segment first-variation inequality characterizing the projection. -/
theorem convex_projection_variational
    (K : Set ℂ) (hKne : K.Nonempty) (hKcompact : IsCompact K)
    (hKconvex : Convex ℝ K) (z w : ℂ) (hw : w ∈ K) :
    ⟪z - convexProjection K hKne hKcompact hKconvex z,
      w - convexProjection K hKne hKcompact hKconvex z⟫_ℝ ≤ 0 :=
  CrouzeixConjecture.convexProjection_variational K hKne hKcompact hKconvex z w hw

/-- Adding the two projection inequalities and applying Cauchy--Schwarz gives
the one-Lipschitz estimate. -/
theorem convex_projection_nonexpansive
    (K : Set ℂ) (hKne : K.Nonempty) (hKcompact : IsCompact K)
    (hKconvex : Convex ℝ K) (x y : ℂ) :
    ‖convexProjection K hKne hKcompact hKconvex x -
        convexProjection K hKne hKcompact hKconvex y‖ ≤ ‖x - y‖ :=
  norm_convexProjection_sub_le K hKne hKcompact hKconvex x y

/-- The canonical radius is positive and at most one, so its parallel body is
an honest outer neighborhood inside one fixed thickening. -/
theorem outer_approximation_radius (k : ℕ) :
    0 < outerApproximationRadius k ∧ outerApproximationRadius k ≤ 1 := by
  exact ⟨outerApproximationRadius_pos k, outerApproximationRadius_le_one k⟩

/-- Radius decay controls the Hausdorff error uniformly for every compact
source set, and all approximants remain in one fixed compact neighborhood. -/
theorem outer_radius_tends_to_zero
    (K : Set ℂ) (hKcompact : IsCompact K) :
    Tendsto outerApproximationRadius atTop (nhds 0) ∧
      Tendsto (fun k ↦ Metric.hausdorffDist
        (closure (parallelOuterDomain K k)) K) atTop (nhds 0) ∧
      ∀ k, closure (parallelOuterDomain K k) ⊆ Metric.cthickening 1 K := by
  exact ⟨tendsto_outerApproximationRadius,
    tendsto_hausdorffDist_parallelOuterDomain_closure hKcompact,
    parallelOuterDomain_closure_subset_fixedNeighborhood K⟩

/-- The inverse-gauge radial parametrization of a positive parallel body is
regular: its tangent never vanishes.  The imaginary component of the polar
factor is the strictly positive radius. -/
theorem parallel_radial_tangent_ne_zero
    (K : Set ℂ) (hKne : K.Nonempty) (hKcompact : IsCompact K)
    (hKconvex : Convex ℝ K) {c : ℂ} (hc : c ∈ K)
    {r : ℝ} (hr : 0 < r) (t : ℝ) :
    (parallelPositivePeriodicRadialData
      K hKne hKcompact hKconvex hc hr).tangent t ≠ 0 := by
  rw [parallelPositivePeriodicRadialData_tangent_formula]
  intro hzero
  have hfactor :
      (((deriv (𝕜 := ℝ) (parallelGaugeRadius K c r) t : ℝ) : ℂ) +
        Complex.I * (parallelGaugeRadius K c r t : ℂ)) = 0 :=
    (mul_eq_zero.mp hzero).resolve_right (parallelRadialDirection_ne_zero t)
  have himag := congrArg Complex.im hfactor
  simp only [Complex.add_im, Complex.ofReal_im, Complex.I_mul_im,
    Complex.ofReal_re, zero_add, Complex.zero_im] at himag
  exact (parallelGaugeRadius_pos K hKcompact hc hr t).ne' himag

/-- Consequently the arclength speed used in the oriented boundary package is
strictly positive, not merely nonnegative. -/
theorem parallel_radial_speed_pos
    (K : Set ℂ) (hKne : K.Nonempty) (hKcompact : IsCompact K)
    (hKconvex : Convex ℝ K) {c : ℂ} (hc : c ∈ K)
    {r : ℝ} (hr : 0 < r) (t : ℝ) :
    0 < ‖(parallelPositivePeriodicRadialData
      K hKne hKcompact hKconvex hc hr).tangent t‖ :=
  norm_pos_iff.mpr
    (parallel_radial_tangent_ne_zero K hKne hKcompact hKconvex hc hr t)

/-- The exact positively oriented radial `C¹` boundary package used by the
Cauchy and double-layer constructions; no raw-body `C∞` claim is made. -/
theorem parallel_outer_domain_data
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n] :
    CanonicalParallelOrientedRadialBoundaryStatement (n := n) :=
  canonicalParallelOrientedRadialBoundaryStatement

namespace Exercises.Chapter25

/-- A nonempty closed convex subset of the complex plane has a nearest point. -/
theorem exercise_01_solution
    (K : Set ℂ) (hKne : K.Nonempty) (hKclosed : IsClosed K)
    (hKconvex : Convex ℝ K) (z : ℂ) :
    ∃ p : ℂ, p ∈ K ∧ ∀ w ∈ K, ‖z - p‖ ≤ ‖z - w‖ := by
  rcases exists_norm_eq_iInf_of_complete_convex
      hKne hKclosed.isComplete hKconvex z with ⟨p, hpK, hpiInf⟩
  refine ⟨p, hpK, ?_⟩
  intro w hw
  rw [hpiInf]
  exact ciInf_le ⟨0, Set.forall_mem_range.2 fun _ ↦ norm_nonneg _⟩
    (⟨w, hw⟩ : K)

/-- Projection onto a centered closed disk is radial outside and the identity
inside. -/
theorem exercise_02_solution
    (r : ℝ) (hr : 0 < r) (x : ℂ)
    (hne : (Metric.closedBall (0 : ℂ) r).Nonempty)
    (hcompact : IsCompact (Metric.closedBall (0 : ℂ) r))
    (hconvex : Convex ℝ (Metric.closedBall (0 : ℂ) r)) :
    convexProjection (Metric.closedBall (0 : ℂ) r) hne hcompact hconvex x =
      if ‖x‖ ≤ r then x else ((r / ‖x‖ : ℝ) : ℂ) * x := by
  let K : Set ℂ := Metric.closedBall 0 r
  let p := convexProjection K hne hcompact hconvex x
  by_cases hx : ‖x‖ ≤ r
  · have hxK : x ∈ K := by simpa [K, Metric.mem_closedBall, dist_zero_right]
    have hpK : p ∈ K := convexProjection_mem K hne hcompact hconvex x
    have hpvar : ⟪x - p, x - p⟫_ℝ ≤ 0 := by
      exact (norm_eq_iInf_iff_real_inner_le_zero hconvex hpK).mp
        (norm_sub_convexProjection_eq_iInf K hne hcompact hconvex x) x hxK
    have hxp : x = p := sub_eq_zero.mp (real_inner_self_nonpos.mp hpvar)
    have hpx : p = x := hxp.symm
    simpa [hx, p] using hpx
  · have hxgt : r < ‖x‖ := lt_of_not_ge hx
    have hxnorm_pos : 0 < ‖x‖ := lt_trans hr hxgt
    have hxnorm : ‖x‖ ≠ 0 := ne_of_gt hxnorm_pos
    let q : ℂ := ((r / ‖x‖ : ℝ) : ℂ) * x
    have hqnorm : ‖q‖ = r := by
      simp only [q, norm_mul, Complex.norm_real]
      rw [Real.norm_eq_abs, abs_of_pos (div_pos hr hxnorm_pos)]
      field_simp
    have hqK : q ∈ K := by
      simp [K, Metric.mem_closedBall, dist_zero_right, hqnorm]
    have hqvar : ∀ w ∈ K, ⟪x - q, w - q⟫_ℝ ≤ 0 := by
      intro w hw
      have hwnorm : ‖w‖ ≤ r := by
        simpa [K, Metric.mem_closedBall, dist_zero_right] using hw
      have hinner := real_inner_le_norm x w
      have hscale_nonneg : 0 ≤ 1 - r / ‖x‖ := by
        apply sub_nonneg.mpr
        exact (div_le_one hxnorm_pos).2 hxgt.le
      have hinner_bound : ⟪x, w⟫_ℝ ≤ ‖x‖ * r :=
        le_trans hinner (mul_le_mul_of_nonneg_left hwnorm (norm_nonneg x))
      change ⟪x - (r / ‖x‖) • x, w - (r / ‖x‖) • x⟫_ℝ ≤ 0
      rw [show x - (r / ‖x‖) • x = (1 - r / ‖x‖) • x by module]
      simp only [real_inner_smul_left, inner_sub_right, real_inner_smul_right,
        real_inner_self_eq_norm_mul_norm]
      have hcancel : (r / ‖x‖) * (‖x‖ * ‖x‖) = ‖x‖ * r := by
        field_simp
      calc
        (1 - r / ‖x‖) * ⟪x, w⟫_ℝ -
            r / ‖x‖ * ((1 - r / ‖x‖) * (‖x‖ * ‖x‖)) =
            (1 - r / ‖x‖) *
              (⟪x, w⟫_ℝ - (r / ‖x‖) * (‖x‖ * ‖x‖)) := by ring
        _ = (1 - r / ‖x‖) * (⟪x, w⟫_ℝ - ‖x‖ * r) := by rw [hcancel]
        _ ≤ 0 := mul_nonpos_of_nonneg_of_nonpos hscale_nonneg
          (sub_nonpos.mpr hinner_bound)
    have hpK : p ∈ K := convexProjection_mem K hne hcompact hconvex x
    have hpvar : ∀ w ∈ K, ⟪x - p, w - p⟫_ℝ ≤ 0 :=
      (norm_eq_iInf_iff_real_inner_le_zero hconvex hpK).mp
        (norm_sub_convexProjection_eq_iInf K hne hcompact hconvex x)
    have hqp : ⟪q - p, q - p⟫_ℝ ≤ 0 := by
      calc
        ⟪q - p, q - p⟫_ℝ =
            ⟪x - q, p - q⟫_ℝ + ⟪x - p, q - p⟫_ℝ := by
          simp only [inner_sub_left, inner_sub_right]
          ring
        _ ≤ 0 := add_nonpos (hqvar p hpK) (hpvar q hqK)
    have hqp_eq : q = p := sub_eq_zero.mp (real_inner_self_nonpos.mp hqp)
    simp only [K, p, q] at hqp_eq
    simp [hx, ← hqp_eq]

/-- Two variational inequalities force the two candidates to coincide. -/
theorem exercise_03_solution
    (x p q : ℂ)
    (hp : ⟪x - p, q - p⟫_ℝ ≤ 0)
    (hq : ⟪x - q, p - q⟫_ℝ ≤ 0) : p = q := by
  have hself : ⟪p - q, p - q⟫_ℝ ≤ 0 := by
    calc
      ⟪p - q, p - q⟫_ℝ =
          ⟪x - p, q - p⟫_ℝ + ⟪x - q, p - q⟫_ℝ := by
        simp only [inner_sub_left, inner_sub_right]
        ring
      _ ≤ 0 := add_nonpos hp hq
  exact sub_eq_zero.mp (real_inner_self_nonpos.mp hself)

/-- Add the two variational inequalities, then cancel the common distance
after Cauchy--Schwarz; the zero-distance branch is handled separately. -/
theorem exercise_04_solution
    (x y px py : ℂ)
    (hx : ⟪x - px, py - px⟫_ℝ ≤ 0)
    (hy : ⟪y - py, px - py⟫_ℝ ≤ 0) :
    ‖px - py‖ ≤ ‖x - y‖ := by
  have hfirm : ⟪px - py, px - py⟫_ℝ ≤ ⟪x - y, px - py⟫_ℝ := by
    have hid :
        ⟪x - y, px - py⟫_ℝ - ⟪px - py, px - py⟫_ℝ =
          -⟪x - px, py - px⟫_ℝ - ⟪y - py, px - py⟫_ℝ := by
      simp only [inner_sub_left, inner_sub_right]
      ring
    linarith
  have hprod : ‖px - py‖ * ‖px - py‖ ≤ ‖x - y‖ * ‖px - py‖ := by
    calc
      ‖px - py‖ * ‖px - py‖ = ⟪px - py, px - py⟫_ℝ :=
        (real_inner_self_eq_norm_mul_norm (px - py)).symm
      _ ≤ ⟪x - y, px - py⟫_ℝ := hfirm
      _ ≤ ‖x - y‖ * ‖px - py‖ := real_inner_le_norm _ _
  by_cases hz : ‖px - py‖ = 0
  · simpa only [hz] using norm_nonneg (x - y)
  · exact le_of_mul_le_mul_right hprod
      (lt_of_le_of_ne (norm_nonneg _) (Ne.symm hz))

/-- A corner with unequal one-sided tangents is not differentiable. -/
theorem exercise_05_solution
    (v w : ℂ) (hvw : v ≠ w) :
    ¬ DifferentiableAt ℝ (fun t : ℝ => if t ≤ 0 then t • v else t • w) 0 := by
  intro hdiff
  let f : ℝ → ℂ := fun t ↦ if t ≤ 0 then t • v else t • w
  have hleft : HasDerivWithinAt f v (Iic 0) 0 := by
    have hbase : HasDerivWithinAt (fun t : ℝ ↦ t • v) v (Iic 0) 0 := by
      simpa using
        ((hasDerivAt_id (𝕜 := ℝ) (x := 0)).smul_const v).hasDerivWithinAt
    apply hbase.congr
    · intro t ht
      change t ≤ 0 at ht
      simp [f, ht]
    · simp [f]
  have hright : HasDerivWithinAt f w (Ici 0) 0 := by
    have hbase : HasDerivWithinAt (fun t : ℝ ↦ t • w) w (Ici 0) 0 := by
      simpa using
        ((hasDerivAt_id (𝕜 := ℝ) (x := 0)).smul_const w).hasDerivWithinAt
    apply hbase.congr
    · intro t ht
      change 0 ≤ t at ht
      rcases lt_or_eq_of_le ht with htpos | rfl
      · simp [f, not_le.mpr htpos]
      · simp [f]
    · simp [f]
  have hglobal : DifferentiableAt ℝ f 0 := by simpa only [f] using hdiff
  have hvderiv : derivWithin f (Iic 0) 0 = v :=
    hleft.derivWithin (uniqueDiffWithinAt_Iic 0)
  have hwderiv : derivWithin f (Ici 0) 0 = w :=
    hright.derivWithin (uniqueDiffWithinAt_Ici 0)
  have hleft_global : derivWithin f (Iic 0) 0 = deriv f 0 :=
    hglobal.hasDerivAt.hasDerivWithinAt.derivWithin (uniqueDiffWithinAt_Iic 0)
  have hright_global : derivWithin f (Ici 0) 0 = deriv f 0 :=
    hglobal.hasDerivAt.hasDerivWithinAt.derivWithin (uniqueDiffWithinAt_Ici 0)
  apply hvw
  rw [← hvderiv, hleft_global, ← hright_global, hwderiv]

/-- Build the canonical oriented radial `C¹` package without invoking the
preassembled existence theorem. -/
theorem exercise_06_solution
    (n : Type) [Fintype n] [DecidableEq n] [Nonempty n] :
    CanonicalParallelOrientedRadialBoundaryStatement (n := n) := by
  intro A k
  unfold HasOrientedRadialConvexBoundary
  let c : ℂ := Classical.choose (numericalRange_nonempty A)
  have hc : c ∈ numericalRange A := Classical.choose_spec (numericalRange_nonempty A)
  let R := parallelPositivePeriodicRadialData
    (numericalRange A) (numericalRange_nonempty A) (isCompact_numericalRange A)
      (numericalRange_convex A) hc (outerApproximationRadius_pos k)
  exact ⟨R, c, ⟨orientedRadialConvexBoundary_thickening
    (numericalRange A) (numericalRange_nonempty A) (isCompact_numericalRange A)
      (numericalRange_convex A) hc (outerApproximationRadius_pos k)⟩⟩

end Exercises.Chapter25

end
end CrouzeixTextbook.Part05

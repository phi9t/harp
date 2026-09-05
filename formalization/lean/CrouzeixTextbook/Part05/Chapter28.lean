import CrouzeixConjecture.HolomorphicDoubleLayer

namespace CrouzeixTextbook.Part05
open CrouzeixConjecture MeasureTheory
open scoped BoundedContinuousFunction ComplexOrder Matrix Matrix.Norms.L2Operator

noncomputable section

/-- Holomorphy on a neighborhood of a compact, oriented radial domain gives
the complete Cauchy family on one common contour. -/
theorem power_cauchy_formula
    {n : Type*} [Fintype n] [DecidableEq n]
    (R : PositivePeriodicRadialData) (c : ℂ) {Omega V : Set ℂ}
    (G : R.OrientedRadialConvexBoundary c Omega)
    (hVopen : IsOpen V) (hclosureCompact : IsCompact (closure Omega))
    (hclosure : closure Omega ⊆ V)
    {f : ℂ → ℂ} (hf : DifferentiableOn ℂ f V)
    (hbound : ∀ z ∈ closure Omega, ‖f z‖ ≤ 1)
    (B : SquareMatrix n) (hB : SimpleDiagonalization B)
    (hWB : numericalRange B ⊆ Omega) :
    HasParametricPowerCauchyFormula
      (G.parametricBoundary R c) contourParameterMeasure B
      (parametricContractiveBoundaryFunctionOfContinuousOn
        (G.parametricBoundary R c) f
        (hf.continuousOn.mono hclosure) hbound)
      (hB.functionEval f) := by
  have hfamily :=
    G.hasParametricPowerCauchyFormula_of_holomorphic_of_simpleDiagonalization
    R c hVopen hclosureCompact hclosure hf hbound B hB hWB
  intro m
  exact hfamily m

/-- The zeroth member of a complete power family is exactly the mass-one
normalization. -/
theorem power_cauchy_mass_one
    {i n : Type*} [TopologicalSpace i] [MeasurableSpace i]
    [Fintype n] [DecidableEq n]
    {mu : Measure i} {Omega : Set ℂ}
    {Gamma : ParametricConvexBoundary (i := i) Omega}
    {B : SquareMatrix n} {f : ContractiveBoundaryFunction i}
    {T : SquareMatrix n}
    (hCauchy : HasParametricPowerCauchyFormula Gamma mu B f T) :
    ∫ x, parametricBoundaryFirstPart Gamma B x ∂mu =
      (1 : SquareMatrix n) := by
  simpa only [pow_zero, one_smul] using hCauchy 0

/-- Continuous scalar data bounded by one on the closed domain produces the
normalized boundary function, with both its evaluation law and contractivity
visible at this interface. -/
theorem contractive_boundary_function
    {i : Type*} [TopologicalSpace i] [CompactSpace i] {Omega : Set ℂ}
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (f : ℂ → ℂ) (hf : ContinuousOn f (closure Omega))
    (hbound : ∀ z ∈ closure Omega, ‖f z‖ ≤ 1) :
    ∃ h : ContractiveBoundaryFunction i,
      (∀ x, h.function x = f (Gamma.point x)) ∧
      ‖h.function‖ ≤ 1 := by
  let h := parametricContractiveBoundaryFunctionOfContinuousOn
    Gamma f hf hbound
  refine ⟨h, ?_, ?_⟩
  · intro x
    rfl
  · exact (BoundedContinuousFunction.norm_le (f := h.function) (by norm_num)).2
      h.norm_le_one

/-- The whole power family determines one companion function in the algebra
of `B`, and the direct Cayley identity holds at every point of the disk. -/
theorem power_cayley_companion
    {i n : Type*} [TopologicalSpace i] [CompactSpace i]
    [MeasurableSpace i] [OpensMeasurableSpace i]
    [Fintype n] [DecidableEq n] [Nonempty n]
    {mu : Measure i} [IsFiniteMeasure mu] {Omega : Set ℂ}
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (f : ContractiveBoundaryFunction i) (T : SquareMatrix n)
    (hCauchy : HasParametricPowerCauchyFormula Gamma mu B f T)
    (hspectrum : matrixSpectrum T ⊆ closedUnitDisk) :
    ∃ g : ℂ → SquareMatrix n,
      (∀ z ∈ unitDisk, g z ∈ generatedAlgebra B) ∧
      (∀ z ∈ unitDisk,
        (rePart (doubleLayerCayleySeries
          (parametricPositiveBoundaryDensityOfMass
            Gamma B hWB (power_cauchy_mass_one hCauchy)) f z)).PosSemidef) ∧
      ∀ z ∈ unitDisk,
        (2 : ℂ) • doubleLayerCayleySeries
            (parametricPositiveBoundaryDensityOfMass
              Gamma B hWB (power_cauchy_mass_one hCauchy)) f z =
          matrixCayleyTransform z T + (g z)ᴴ := by
  refine ⟨parametricPowerCayleyCompanion Gamma mu B f, ?_, ?_⟩
  · intro z hz
    exact parametricPowerCayleyCompanion_mem_generatedAlgebra
      Gamma B hWB f z hz
  · constructor
    · intro z hz
      exact doubleLayerCayleySeries_rePart_posSemidef _ f z hz
    · intro z hz
      exact parametric_direct_cayley_identity_of_powerCauchy
        Gamma B hWB f T hCauchy hspectrum z hz

/-- The Cayley series built from the complete family is a positive-real
completion of the prescribed target. -/
theorem positive_completion_from_power_family
    {i n : Type*} [TopologicalSpace i] [CompactSpace i]
    [MeasurableSpace i] [OpensMeasurableSpace i]
    [Fintype n] [DecidableEq n] [Nonempty n]
    {mu : Measure i} [IsFiniteMeasure mu] {Omega : Set ℂ}
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (f : ContractiveBoundaryFunction i) (T : SquareMatrix n)
    (hCauchy : HasParametricPowerCauchyFormula Gamma mu B f T)
    (hspectrum : matrixSpectrum T ⊆ closedUnitDisk) :
    ∃ H : ℂ → SquareMatrix n, IsPositiveRealCompletion B T H := by
  let D : PositiveBoundaryDensity (n := n) mu :=
    parametricPositiveBoundaryDensityOfMass
      Gamma B hWB (power_cauchy_mass_one hCauchy)
  obtain ⟨g, hg, hpositive, hidentity⟩ :=
    power_cayley_companion Gamma B hWB f T hCauchy hspectrum
  refine ⟨doubleLayerCayleySeries D f, ?_⟩
  apply isPositiveRealCompletion_of_direct_cayley_identity
    B T (doubleLayerCayleySeries D f) g hspectrum
  · exact doubleLayerCayleySeries_analyticOnNhd D f
  · exact doubleLayerCayleySeries_zero D f
  · exact hpositive
  · exact hg
  · exact hidentity

/-- Simultaneous Cauchy control supplies a positive-real completion, fed here
to the Jin provider whose terminal implication is deferred to Chapters 30--32. -/
theorem power_family_norm_two
    {i n : Type*} [TopologicalSpace i] [CompactSpace i]
    [MeasurableSpace i] [OpensMeasurableSpace i]
    [Fintype n] [DecidableEq n] [Nonempty n]
    {mu : Measure i} [IsFiniteMeasure mu] {Omega : Set ℂ}
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B T : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (f : ContractiveBoundaryFunction i)
    (hCauchy : HasParametricPowerCauchyFormula Gamma mu B f T)
    (hB : SimpleDiagonalization B) (lambda : n → ℂ)
    (htarget : T = innerConjugation hB.changeBasis (Matrix.diagonal lambda))
    (hlambda : ∀ j, ‖lambda j‖ ≤ 1) :
    ‖T‖ ≤ 2 := by
  have hspectrum : matrixSpectrum T ⊆ closedUnitDisk := by
    rw [htarget, matrixSpectrum, AlgEquiv.spectrum_eq, spectrum_diagonal]
    rintro z ⟨j, rfl⟩
    simpa [closedUnitDisk, Metric.mem_closedBall, dist_eq_norm] using hlambda j
  obtain ⟨H, hH⟩ := positive_completion_from_power_family
    Gamma B hWB f T hCauchy hspectrum
  exact positiveRealCompletionStatement B T H hB lambda htarget hlambda hH

namespace Exercises.Chapter28

theorem exercise_01_solution :
    ∀ (i n : Type) [TopologicalSpace i] [CompactSpace i]
      [MeasurableSpace i] [OpensMeasurableSpace i]
      [Fintype n] [DecidableEq n] [Nonempty n]
      (mu : Measure i) [IsFiniteMeasure mu] (Omega : Set ℂ)
      (Gamma : ParametricConvexBoundary (i := i) Omega)
      (B : SquareMatrix n) (f : ContractiveBoundaryFunction i)
      (T : SquareMatrix n),
      HasParametricPowerCauchyFormula Gamma mu B f T ↔
        ∀ m : ℕ, ∫ x, (f.function x) ^ m •
          parametricBoundaryFirstPart Gamma B x ∂mu = T ^ m := by
  intro i n _ _ _ _ _ _ _ mu _ Omega Gamma B f T
  constructor
  · intro h m
    exact h m
  · intro h m
    exact h m

theorem exercise_02_solution :
    ∀ (i : Type) [TopologicalSpace i]
      (f : ContractiveBoundaryFunction i),
      ∀ m : ℕ, ∀ x : i, ‖f.function x ^ m‖ ≤ 1 := by
  intro i _ f m x
  rw [norm_pow]
  exact pow_le_one₀ (norm_nonneg (f.function x)) (f.norm_le_one x)

theorem exercise_03_solution :
    ∀ z w : ℂ, ‖z * w‖ < 1 →
      cayleyTransform z w =
        1 + 2 * ∑' m : ℕ, (z * w) ^ (m + 1) := by
  intro z w hzw
  let u : ℂ := z * w
  have hunorm : ‖u‖ < 1 := by simpa [u] using hzw
  have htail : ∑' m : ℕ, u ^ (m + 1) = u * (1 - u)⁻¹ := by
    rw [show (fun m : ℕ ↦ u ^ (m + 1)) =
        fun m : ℕ ↦ u * u ^ m by
      funext m
      rw [pow_succ']]
    rw [tsum_mul_left, tsum_geometric_of_norm_lt_one hunorm]
  have hne : 1 - u ≠ 0 := by
    intro hu
    have huone : u = 1 := (sub_eq_zero.mp hu).symm
    rw [huone, norm_one] at hunorm
    exact (lt_irrefl 1 hunorm).elim
  rw [cayleyTransform, show z * w = u by rfl, htail]
  rw [inv_eq_one_div]
  field_simp
  ring

theorem exercise_04_solution :
    ∀ (i n : Type) [TopologicalSpace i] [CompactSpace i]
      [MeasurableSpace i] [OpensMeasurableSpace i]
      [Fintype n] [DecidableEq n] [Nonempty n]
      (mu : Measure i) [IsFiniteMeasure mu] (Omega : Set ℂ)
      (Gamma : ParametricConvexBoundary (i := i) Omega)
      (B : SquareMatrix n) (f : ContractiveBoundaryFunction i)
      (T : SquareMatrix n),
      HasParametricPowerCauchyFormula Gamma mu B f T →
        ∫ x, parametricBoundaryFirstPart Gamma B x ∂mu =
          (1 : SquareMatrix n) := by
  intro i n _ _ _ _ _ _ _ mu _ Omega Gamma B f T hCauchy
  simpa only [pow_zero, one_smul] using hCauchy 0

theorem exercise_05_solution :
    ∀ (i n : Type) [TopologicalSpace i] [CompactSpace i]
      [MeasurableSpace i] [OpensMeasurableSpace i]
      [Fintype n] [DecidableEq n] [Nonempty n]
      (mu : Measure i) [IsFiniteMeasure mu] (Omega : Set ℂ)
      (Gamma : ParametricConvexBoundary (i := i) Omega)
      (B : SquareMatrix n) (f : ContractiveBoundaryFunction i)
      (T : SquareMatrix n),
      HasParametricPowerCauchyFormula Gamma mu B f T →
        (∀ m : ℕ, ∫ x, (f.function x) ^ m •
          parametricBoundaryFirstPart Gamma B x ∂mu = T ^ m) ∧
        ∫ x, parametricBoundaryFirstPart Gamma B x ∂mu =
          (1 : SquareMatrix n) := by
  intro i n _ _ _ _ _ _ _ mu _ Omega Gamma B f T hCauchy
  constructor
  · intro m
    exact hCauchy m
  · simpa only [pow_zero, one_smul] using hCauchy 0

theorem exercise_06_solution :
    ∀ (i n : Type) [TopologicalSpace i] [CompactSpace i]
      [MeasurableSpace i] [OpensMeasurableSpace i]
      [Fintype n] [DecidableEq n] [Nonempty n]
      (mu : Measure i) [IsFiniteMeasure mu] (Omega : Set ℂ)
      (Gamma : ParametricConvexBoundary (i := i) Omega)
      (B T : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
      (f : ContractiveBoundaryFunction i),
      HasParametricPowerCauchyFormula Gamma mu B f T →
        ∀ (hB : SimpleDiagonalization B) (lambda : n → ℂ),
          T = innerConjugation hB.changeBasis (Matrix.diagonal lambda) →
          (∀ j, ‖lambda j‖ ≤ 1) → ‖T‖ ≤ 2 := by
  intro i n _ _ _ _ _ _ _ mu _ Omega Gamma B T hWB f hCauchy
    hB lambda htarget hlambda
  have hspectrum : matrixSpectrum T ⊆ closedUnitDisk := by
    rw [htarget, matrixSpectrum, AlgEquiv.spectrum_eq, spectrum_diagonal]
    rintro z ⟨j, rfl⟩
    simpa [closedUnitDisk, Metric.mem_closedBall, dist_eq_norm] using hlambda j
  obtain ⟨H, hcompletion⟩ := positive_completion_from_power_family
    Gamma B hWB f T hCauchy hspectrum
  exact positiveRealCompletionStatement
    B T H hB lambda htarget hlambda hcompletion

end Exercises.Chapter28

/-- The sharpness matrix `A₀,₂` is the two-dimensional weighted shift. -/
def aZeroTwo : SquareMatrix (Fin 2) :=
  !![0, 2; 0, 0]

/-- The complete power sequence of `A₀,₂` is `I, A₀,₂, 0, 0, …`. -/
theorem a_zero_two_power_truncation :
    aZeroTwo ^ 0 = 1 ∧
    aZeroTwo ^ 1 = aZeroTwo ∧
    ∀ m : ℕ, 2 ≤ m → aZeroTwo ^ m = 0 := by
  constructor
  · simp
  constructor
  · simp
  · intro m hm
    obtain ⟨k, rfl⟩ := Nat.exists_eq_add_of_le hm
    rw [pow_add]
    have hsquare : aZeroTwo ^ 2 = 0 := by
      ext i j
      fin_cases i <;> fin_cases j <;>
        norm_num [aZeroTwo, Matrix.mul_apply, pow_two]
    rw [hsquare, zero_mul]

end
end CrouzeixTextbook.Part05

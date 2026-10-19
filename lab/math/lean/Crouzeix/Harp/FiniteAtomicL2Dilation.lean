import Crouzeix.Harp.FiniteAtomicDilation
import Crouzeix.LoristSchwenninger.CompressionMoments
import Mathlib.LinearAlgebra.Complex.FiniteDimensional
import Mathlib.MeasureTheory.Integral.Bochner.SumMeasure
import Mathlib.MeasureTheory.Function.LpSeminorm.Count
import Mathlib.MeasureTheory.Measure.Count

/-!
Finite atomic `L²` dilations for the Harp finite-horizon argument.

Cubature weights are absorbed into a positive matrix density on a finite
discrete space equipped with counting measure.  The Lorist--Schwenninger
boundary embedding and multiplier can then be reused without introducing a
second finite-dimensional dilation model.
-/

noncomputable section

namespace CrouzeixConjecture
namespace Harp

open MeasureTheory Set
open scoped BoundedContinuousFunction ComplexOrder Matrix MatrixOrder
  Matrix.Norms.L2Operator

universe u

variable {i : Type u} {n : Type} [TopologicalSpace i] [CompactSpace i]
  [MeasurableSpace i] [OpensMeasurableSpace i]
  [Fintype n] [DecidableEq n] [Nonempty n]
  {μ : Measure i} [IsFiniteMeasure μ] {Ω : Set ℂ}

/-- A finite family of positive weighted matrices, regarded as a positive
boundary density for counting measure. -/
def finiteAtomicPositiveBoundaryDensity
    {ι : Type} [TopologicalSpace ι] [MeasurableSpace ι]
    [OpensMeasurableSpace ι] [MeasurableSingletonClass ι] [Fintype ι]
    (density : ι → SquareMatrix n) (weights : ι → ℝ)
    (hpositive : ∀ j, (density j).PosSemidef)
    (hweights : ∀ j, 0 ≤ weights j)
    (hmass : ∑ j, weights j • density j =
      (2 : ℂ) • (1 : SquareMatrix n)) :
    PositiveBoundaryDensity (n := n) (Measure.count : Measure ι) where
  density := fun j ↦ weights j • density j
  integrable_density := Integrable.of_finite
  posSemidef_ae := Measure.ae_count_iff.mpr fun j ↦
    (hpositive j).smul (hweights j)
  mass_eq_two_one := by
    simpa only [MeasureTheory.integral_count] using hmass

/-- The polynomial boundary function restricted to a finite discrete family
of cubature nodes. -/
def finiteAtomicPolynomialBoundaryFunction
    {ι : Type} [TopologicalSpace ι] [DiscreteTopology ι] [CompactSpace ι]
    (nodes : ι → i) (Gamma : ParametricConvexBoundary (i := i) Ω)
    (q : Polynomial ℂ) : ι →ᵇ ℂ :=
  BoundedContinuousFunction.mkOfCompact
    ⟨fun j ↦ parametricPolynomialBoundaryFunction Gamma q (nodes j),
      continuous_of_discreteTopology⟩

omit [MeasurableSpace i] [OpensMeasurableSpace i] in
@[simp]
theorem finiteAtomicPolynomialBoundaryFunction_apply
    {ι : Type} [TopologicalSpace ι] [DiscreteTopology ι] [CompactSpace ι]
    (nodes : ι → i) (Gamma : ParametricConvexBoundary (i := i) Ω)
    (q : Polynomial ℂ) (j : ι) :
    finiteAtomicPolynomialBoundaryFunction nodes Gamma q j =
      parametricPolynomialBoundaryFunction Gamma q (nodes j) :=
  rfl

/-- Cubature at power zero turns the positively weighted sampled
double-layer matrices into a normalized boundary density for counting
measure. -/
def finiteAtomicDoubleLayerDensity
    {ι : Type} [TopologicalSpace ι] [MeasurableSpace ι]
    [OpensMeasurableSpace ι] [MeasurableSingletonClass ι] [Fintype ι]
    (Gamma : ParametricConvexBoundary (i := i) Ω)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Ω)
    (nodes : ι → i) (weights : ι → ℝ)
    (hweights : ∀ j, 0 < weights j)
    (hmass : ∑ j, weights j • parametricDoubleLayerDensity Gamma B (nodes j) =
      (2 : ℂ) • (1 : SquareMatrix n)) :
    PositiveBoundaryDensity (n := n) (Measure.count : Measure ι) :=
  finiteAtomicPositiveBoundaryDensity
    (fun j ↦ parametricDoubleLayerDensity Gamma B (nodes j)) weights
    (fun j ↦ parametricDoubleLayerDensity_posSemidef Gamma B hWB (nodes j))
    (fun j ↦ (hweights j).le) hmass

omit [Nonempty n] in
/-- The zeroth cubature moment is exactly the mass identity needed by the
finite counting-measure density. -/
theorem finiteAtomicDoubleLayer_mass_eq_two_one
    {ι : Type} [Fintype ι] (N : ℕ)
    (Gamma : ParametricConvexBoundary (i := i) Ω)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Ω)
    (q : Polynomial ℂ) (hCauchy : HasParametricPolynomialCauchyFormula Gamma μ B)
    (nodes : ι → i) (weights : ι → ℝ)
    (hmoments : ∀ k : Fin (N + 2),
      ∑ j, weights j • matrixMomentObservable N Gamma B q (nodes j) k =
        ∫ x, matrixMomentObservable N Gamma B q x k ∂μ) :
    ∑ j, weights j • parametricDoubleLayerDensity Gamma B (nodes j) =
      (2 : ℂ) • (1 : SquareMatrix n) := by
  have hzero := hmoments (0 : Fin (N + 2))
  have hzero' :
      ∑ j, weights j • parametricDoubleLayerDensity Gamma B (nodes j) =
        ∫ x, parametricDoubleLayerDensity Gamma B x ∂μ := by
    simpa [matrixMomentObservable] using hzero
  have hmass :=
    (parametricPositiveBoundaryDensity Gamma B hWB hCauchy).mass_eq_two_one
  change (∫ x, parametricDoubleLayerDensity Gamma B x ∂μ) =
    (2 : ℂ) • (1 : SquareMatrix n) at hmass
  exact hzero'.trans hmass

omit [Nonempty n] in
/-- Every cubature moment in the sampled horizon gives the same boundary
map value for the finite counting-measure density as for the original
boundary density. -/
theorem finiteAtomic_boundaryPhi_eq_of_moments
    {ι : Type} [TopologicalSpace ι] [DiscreteTopology ι]
    [CompactSpace ι] [MeasurableSpace ι] [BorelSpace ι]
    [MeasurableSingletonClass ι] [Fintype ι]
    (N : ℕ) (Gamma : ParametricConvexBoundary (i := i) Ω)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Ω)
    (q : Polynomial ℂ) (hCauchy : HasParametricPolynomialCauchyFormula Gamma μ B)
    (nodes : ι → i) (weights : ι → ℝ) (hweights : ∀ j, 0 < weights j)
    (hmoments : ∀ l : Fin (N + 2),
      ∑ j, weights j • matrixMomentObservable N Gamma B q (nodes j) l =
        ∫ x, matrixMomentObservable N Gamma B q x l ∂μ)
    (k : ℕ) (hk : k ≤ N + 1) :
    boundaryPhiCLM
        (finiteAtomicDoubleLayerDensity Gamma B hWB nodes weights hweights
          (finiteAtomicDoubleLayer_mass_eq_two_one N Gamma B hWB q hCauchy
            nodes weights hmoments))
        ((finiteAtomicPolynomialBoundaryFunction nodes Gamma q) ^ k) =
      boundaryPhiCLM
        (parametricPositiveBoundaryDensity Gamma B hWB hCauchy)
        ((parametricPolynomialBoundaryFunction Gamma q) ^ k) := by
  rw [boundaryPhiCLM_apply, boundaryPhi, boundaryPhiCLM_apply, boundaryPhi]
  apply congrArg ((2 : ℂ)⁻¹ • ·)
  have hmoment := hmoments (⟨k, by omega⟩ : Fin (N + 2))
  rw [MeasureTheory.integral_count]
  change
    (∑ j, ((finiteAtomicPolynomialBoundaryFunction nodes Gamma q) ^ k) j •
      (weights j • parametricDoubleLayerDensity Gamma B (nodes j))) =
      ∫ x, ((parametricPolynomialBoundaryFunction Gamma q) ^ k) x •
        parametricDoubleLayerDensity Gamma B x ∂μ
  calc
    _ = ∑ j, weights j •
        (((parametricPolynomialBoundaryFunction Gamma q) ^ k) (nodes j) •
          parametricDoubleLayerDensity Gamma B (nodes j)) := by
      apply Finset.sum_congr rfl
      intro j _
      rw [BoundedContinuousFunction.pow_apply,
        finiteAtomicPolynomialBoundaryFunction_apply,
        ← BoundedContinuousFunction.pow_apply]
      exact smul_comm _ _ _
    _ = _ := by
      simpa [matrixMomentObservable, BoundedContinuousFunction.pow_apply] using hmoment

omit [Nonempty n] in
/-- The finite counting-measure boundary embedding and diagonal boundary
multiplier compress to the original boundary moments throughout the requested
horizon. -/
theorem finiteAtomic_compression_eq_of_moments
    {ι : Type} [TopologicalSpace ι] [DiscreteTopology ι]
    [CompactSpace ι] [MeasurableSpace ι] [BorelSpace ι]
    [MeasurableSingletonClass ι] [Fintype ι]
    (N : ℕ) (Gamma : ParametricConvexBoundary (i := i) Ω)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Ω)
    (q : Polynomial ℂ) (hCauchy : HasParametricPolynomialCauchyFormula Gamma μ B)
    (nodes : ι → i) (weights : ι → ℝ) (hweights : ∀ j, 0 < weights j)
    (hmoments : ∀ l : Fin (N + 2),
      ∑ j, weights j • matrixMomentObservable N Gamma B q (nodes j) l =
        ∫ x, matrixMomentObservable N Gamma B q x l ∂μ)
    (k : ℕ) (hk : k ≤ N + 1) :
    let D := finiteAtomicDoubleLayerDensity Gamma B hWB nodes weights hweights
      (finiteAtomicDoubleLayer_mass_eq_two_one N Gamma B hWB q hCauchy
        nodes weights hmoments)
    let h := finiteAtomicPolynomialBoundaryFunction nodes Gamma q
    (ContinuousLinearMap.adjoint
        (LoristSchwenninger.boundaryEmbedding D).toContinuousLinearMap).comp
      (((LoristSchwenninger.bcfMulL
          (mu := (Measure.count : Measure ι)) (n := n) h) ^ k).comp
        (LoristSchwenninger.boundaryEmbedding D).toContinuousLinearMap) =
      euclideanOperator
        (boundaryPhiCLM
          (parametricPositiveBoundaryDensity Gamma B hWB hCauchy)
          ((parametricPolynomialBoundaryFunction Gamma q) ^ k)) := by
  dsimp only
  rw [LoristSchwenninger.boundaryEmbedding_adjoint_comp_bcfMulL_pow_comp_boundaryEmbedding]
  exact congrArg euclideanOperator
    (finiteAtomic_boundaryPhi_eq_of_moments N Gamma B hWB q hCauchy
      nodes weights hweights hmoments k hk)

/-- A small package hiding the horizon-dependent Hilbert space and its
instances while exposing exactly the witness consumed by the finite-horizon
perturbation theorem. -/
structure FiniteAtomicL2DilationWitness
    {E : Type*} [NormedAddCommGroup E] [InnerProductSpace ℂ E]
    [FiniteDimensional ℂ E] [CompleteSpace E] [Nontrivial E]
    (core : CommutingPerturbationData E) (N : ℕ) where
  nodeType : Type
  [fintypeNodeType : Fintype nodeType]
  [measurableSpaceNodeType : MeasurableSpace nodeType]
  [discreteMeasurableSpaceNodeType : DiscreteMeasurableSpace nodeType]
  K : Type
  [normedAddCommGroup : NormedAddCommGroup K]
  [innerProductSpace : InnerProductSpace ℂ K]
  [completeSpace : CompleteSpace K]
  [finiteDimensional : FiniteDimensional ℂ K]
  modelEquiv : K ≃ₗ[ℂ]
    (nodeType →₂[(Measure.count : Measure nodeType)] E)
  nodeCount : ℕ
  nodeCount_eq_card : nodeCount = Fintype.card nodeType
  nodeCountBound : nodeCount ≤
    2 * (N + 2) * Module.finrank ℂ E ^ 2 + 1
  dilationFinrank : Module.finrank ℂ K = nodeCount * Module.finrank ℂ E
  data : FiniteHorizonDilationData core K N

/-- Cubature data in a `Type`-valued package, so its existentially chosen
finite node type can be used to define the horizon-dependent `L²` space. -/
structure FiniteAtomicMatrixMomentCubatureData (N : ℕ)
    (Gamma : ParametricConvexBoundary (i := i) Ω)
    (μ : Measure i) (B : SquareMatrix n) (q : Polynomial ℂ) where
  nodeType : Type
  [fintypeNodeType : Fintype nodeType]
  nodes : nodeType → i
  weights : nodeType → ℝ
  nodeCountBound : Fintype.card nodeType ≤
    Module.finrank ℝ (Fin (N + 2) → SquareMatrix n) + 1
  weights_pos : ∀ j, 0 < weights j
  weightSum : ∑ j, weights j = μ.real Set.univ
  moments : ∀ k : Fin (N + 2),
    ∑ j, weights j • matrixMomentObservable N Gamma B q (nodes j) k =
      ∫ x, matrixMomentObservable N Gamma B q x k ∂μ

omit [Nonempty n] in
/-- The proposition-valued cubature theorem yields a nonempty dependent data
package; classical choice may then select its finite node type. -/
theorem finiteAtomicMatrixMomentCubatureData_nonempty (N : ℕ)
    (Gamma : ParametricConvexBoundary (i := i) Ω)
    (μ : Measure i) [IsFiniteMeasure μ] [NeZero μ]
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Ω)
    (q : Polynomial ℂ) :
    Nonempty (FiniteAtomicMatrixMomentCubatureData N Gamma μ B q) := by
  obtain ⟨ι, hι, nodes, weights, hcard, hweights, hweights_sum, hmoments⟩ :=
    exists_positive_matrix_moment_cubature N Gamma μ B hWB q
  exact ⟨{
    nodeType := ι
    fintypeNodeType := hι
    nodes := nodes
    weights := weights
    nodeCountBound := hcard
    weights_pos := hweights
    weightSum := hweights_sum
    moments := hmoments }⟩

/-- On a finite discrete space, the counting-measure `L²` model is linearly
equivalent to the full function space. -/
def countL2LinearEquiv
    {ι E : Type*} [Finite ι] [MeasurableSpace ι]
    [DiscreteMeasurableSpace ι]
    [NormedAddCommGroup E] [NormedSpace ℂ E] :
    (ι → E) ≃ₗ[ℂ] (ι →₂[(Measure.count : Measure ι)] E) := by
  let toLp : (ι → E) →ₗ[ℂ] (ι →₂[(Measure.count : Measure ι)] E) :=
    { toFun := fun f ↦ (MemLp.of_discrete (f := f)).toLp f
      map_add' := fun f g ↦ (MemLp.toLp_add
        (MemLp.of_discrete (f := f))
        (MemLp.of_discrete (f := g))).symm
      map_smul' := fun c f ↦ (MemLp.toLp_const_smul c
        (MemLp.of_discrete (f := f))).symm }
  apply LinearEquiv.ofBijective toLp
  constructor
  · intro f g hfg
    funext j
    have hae : f =ᵐ[(Measure.count : Measure ι)] g :=
      (MemLp.toLp_eq_toLp_iff
        (MemLp.of_discrete (f := f)) (MemLp.of_discrete (f := g))).mp hfg
    exact Measure.ae_count_iff.mp hae j
  · intro f
    refine ⟨fun j ↦ f j, ?_⟩
    apply Lp.ext
    exact (MemLp.coeFn_toLp (MemLp.of_discrete (f := fun j ↦ f j))).trans
      (Filter.Eventually.of_forall fun _ ↦ rfl)

/-- The `L²` space of vector-valued functions on a finite discrete space with
counting measure is finite-dimensional. -/
theorem finiteDimensional_countL2
    {ι E : Type*} [Fintype ι] [MeasurableSpace ι]
    [DiscreteMeasurableSpace ι]
    [NormedAddCommGroup E] [NormedSpace ℂ E] [FiniteDimensional ℂ E] :
    FiniteDimensional ℂ (ι →₂[(Measure.count : Measure ι)] E) :=
  countL2LinearEquiv.finiteDimensional

/-- The finite counting-measure `L²` model has one copy of the coefficient
space for each node. -/
theorem finrank_countL2
    {ι E : Type*} [Fintype ι] [MeasurableSpace ι]
    [DiscreteMeasurableSpace ι]
    [NormedAddCommGroup E] [NormedSpace ℂ E] [FiniteDimensional ℂ E] :
    Module.finrank ℂ (ι →₂[(Measure.count : Measure ι)] E) =
      Fintype.card ι * Module.finrank ℂ E := by
  rw [← countL2LinearEquiv.finrank_eq, Module.finrank_pi_fintype]
  simp

/-- Positive matrix-moment cubature through `N + 1` produces a finite
counting-measure `L²` dilation witness at horizon `N`. -/
def finiteAtomicL2DilationWitness (N : ℕ)
    (Gamma : ParametricConvexBoundary (i := i) Ω)
    (μ : Measure i) [IsFiniteMeasure μ] [NeZero μ]
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Ω)
    (q : Polynomial ℂ)
    (hq : ∀ z ∈ closure Ω, ‖Polynomial.eval z q‖ ≤ 1)
    (hCauchy : HasParametricPolynomialCauchyFormula Gamma μ B) :
    FiniteAtomicL2DilationWitness
      (finiteHorizonPolynomialCore Gamma μ B hWB q hq) N := by
  let cubature := Classical.choice
    (finiteAtomicMatrixMomentCubatureData_nonempty N Gamma μ B hWB q)
  let ι := cubature.nodeType
  letI : Fintype ι := cubature.fintypeNodeType
  letI : TopologicalSpace ι := ⊥
  letI : DiscreteTopology ι := discreteTopology_bot ι
  letI : MeasurableSpace ι := ⊤
  letI : DiscreteMeasurableSpace ι := inferInstance
  letI : OpensMeasurableSpace ι := inferInstance
  letI : BorelSpace ι := inferInstance
  let nodes := cubature.nodes
  let weights := cubature.weights
  have hweights := cubature.weights_pos
  have hmoments := cubature.moments
  let h := finiteAtomicPolynomialBoundaryFunction nodes Gamma q
  let D := finiteAtomicDoubleLayerDensity Gamma B hWB nodes weights hweights
    (finiteAtomicDoubleLayer_mass_eq_two_one N Gamma B hWB q hCauchy
      nodes weights hmoments)
  let V := (LoristSchwenninger.boundaryEmbedding D).toContinuousLinearMap
  let Q := LoristSchwenninger.bcfMulL
    (mu := (Measure.count : Measure ι)) (n := n) h
  let P := polynomialEval q B
  let core := finiteHorizonPolynomialCore Gamma μ B hWB q hq
  letI : FiniteDimensional ℂ
      (ι →₂[(Measure.count : Measure ι)] EuclideanVector n) :=
    finiteDimensional_countL2
  refine {
    nodeType := ι
    fintypeNodeType := inferInstance
    measurableSpaceNodeType := inferInstance
    discreteMeasurableSpaceNodeType := inferInstance
    K := ι →₂[(Measure.count : Measure ι)] EuclideanVector n
    modelEquiv := LinearEquiv.refl ℂ _
    nodeCount := Fintype.card ι
    nodeCount_eq_card := rfl
    nodeCountBound := by
      calc
        Fintype.card ι ≤
            Module.finrank ℝ (Fin (N + 2) → SquareMatrix n) + 1 :=
          cubature.nodeCountBound
        _ = 2 * (N + 2) * Module.finrank ℂ (EuclideanVector n) ^ 2 + 1 := by
          rw [Module.finrank_pi_fintype]
          simp only [Finset.sum_const, Finset.card_univ, Fintype.card_fin, nsmul_eq_mul]
          rw [finrank_real_of_complex, Module.finrank_matrix]
          simp [finrank_euclideanSpace]
          ring
    dilationFinrank := finrank_countL2
    data := {
      V := V
      Q := Q
      isometry := ?_
      contraction := ?_
      perturbation_eq := ?_ } }
  · exact (LoristSchwenninger.boundaryEmbedding D).isometry
  · apply LoristSchwenninger.bcfMulL_norm_le_one h
    apply (BoundedContinuousFunction.norm_le zero_le_one).2
    intro j
    simpa [h, finiteAtomicPolynomialBoundaryFunction_apply,
      parametricPolynomialBoundaryFunction_apply] using
      hq (Gamma.point (nodes j))
        (frontier_subset_closure (Gamma.supported (nodes j)).boundary_point)
  · intro k hk
    have hcompression := finiteAtomic_compression_eq_of_moments
      N Gamma B hWB q hCauchy nodes weights hweights hmoments k hk
    have hcompressionAdjoint :
        ContinuousLinearMap.adjoint
            ((ContinuousLinearMap.adjoint V).comp ((Q ^ k).comp V)) =
          ContinuousLinearMap.adjoint
            (euclideanOperator
              (boundaryPhiCLM
                (parametricPositiveBoundaryDensity Gamma B hWB hCauchy)
                ((parametricPolynomialBoundaryFunction Gamma q) ^ k))) :=
      congrArg ContinuousLinearMap.adjoint hcompression
    rw [ContinuousLinearMap.adjoint_comp, ContinuousLinearMap.adjoint_comp,
      ContinuousLinearMap.adjoint_adjoint] at hcompressionAdjoint
    have hadjointPow :
        ContinuousLinearMap.adjoint (Q ^ k) =
          (ContinuousLinearMap.adjoint Q) ^ k := by
      rw [← ContinuousLinearMap.star_eq_adjoint, star_pow,
        ContinuousLinearMap.star_eq_adjoint]
    rw [hadjointPow, ← euclideanOperator_conjTranspose] at hcompressionAdjoint
    have hcompressed :
        LoristSchwenninger.compressedAdjointPower V Q k =
          euclideanOperator
            (boundaryPhiCLM
              (parametricPositiveBoundaryDensity Gamma B hWB hCauchy)
              ((parametricPolynomialBoundaryFunction Gamma q) ^ k))ᴴ := by
      apply ContinuousLinearMap.ext
      intro x
      have hx := congrArg
        (fun A : EuclideanVector n →L[ℂ] EuclideanVector n ↦ A x)
        hcompressionAdjoint
      simpa only [LoristSchwenninger.compressedAdjointPower,
        ContinuousLinearMap.comp_apply] using hx
    have hCauchyPower :
        ∫ x, ((parametricPolynomialBoundaryFunction Gamma q) ^ k) x •
            parametricBoundaryFirstPart Gamma B x ∂μ = P ^ k := by
      simpa only [P, BoundedContinuousFunction.pow_apply] using
        LoristSchwenninger.parametricPolynomialPowerCauchy
          Gamma q B hCauchy k
    have hdouble := two_smul_boundaryPhi_parametric_eq
      Gamma B hWB hCauchy
      ((parametricPolynomialBoundaryFunction Gamma q) ^ k)
      (P ^ k) hCauchyPower
    have hdoubleAdjoint := congrArg Matrix.conjTranspose hdouble
    have hdoubleOperator :
        (2 : ℂ) • euclideanOperator
            (boundaryPhiCLM
              (parametricPositiveBoundaryDensity Gamma B hWB hCauchy)
              ((parametricPolynomialBoundaryFunction Gamma q) ^ k))ᴴ =
          (euclideanOperator Pᴴ) ^ k +
            euclideanOperator
              (parametricBoundaryCompanion Gamma μ B
                ((parametricPolynomialBoundaryFunction Gamma q) ^ k)) := by
      have hmatrix :
          (2 : ℂ) •
              (boundaryPhiCLM
                (parametricPositiveBoundaryDensity Gamma B hWB hCauchy)
                ((parametricPolynomialBoundaryFunction Gamma q) ^ k))ᴴ =
            (Pᴴ) ^ k +
              parametricBoundaryCompanion Gamma μ B
                ((parametricPolynomialBoundaryFunction Gamma q) ^ k) := by
        simpa only [Matrix.conjTranspose_smul, Matrix.conjTranspose_add,
          Matrix.conjTranspose_pow, Matrix.conjTranspose_conjTranspose,
          map_ofNat, star_ofNat] using hdoubleAdjoint
      simpa only [map_smul, map_add, map_pow] using
        congrArg (fun A : SquareMatrix n ↦ euclideanOperator A) hmatrix
    have htargetAdjoint :
        ContinuousLinearMap.adjoint (euclideanOperator P) =
          euclideanOperator Pᴴ :=
      (euclideanOperator_conjTranspose P).symm
    change euclideanOperator
        (parametricBoundaryCompanion Gamma μ B
          ((parametricPolynomialBoundaryFunction Gamma q) ^ k)) =
      LoristSchwenninger.doubledCompressedAdjointPower V Q k -
        (ContinuousLinearMap.adjoint (euclideanOperator P)) ^ k
    rw [LoristSchwenninger.doubledCompressedAdjointPower, hcompressed,
      htargetAdjoint, hdoubleOperator]
    abel

end Harp
end CrouzeixConjecture

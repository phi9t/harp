module

public import CrouzeixConjecture.RadialContour
public import CrouzeixConjecture.DoubleLayerBoundary

@[expose] public section

noncomputable section

open MeasureTheory Set
open scoped Matrix Matrix.Norms.L2Operator

namespace CrouzeixConjecture

namespace PositivePeriodicRadialData

namespace OrientedRadialConvexBoundary

/-- The geometric oriented radial boundary therefore supplies the complete double-layer
provider required by the simple-spectrum perturbation argument. -/
theorem hasDoubleLayerCompletionProvider_of_simpleDiagonalization
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (R : PositivePeriodicRadialData) (c : ℂ) {Omega : Set ℂ}
    (G : OrientedRadialConvexBoundary R c Omega)
    (B : SquareMatrix n) (hB : SimpleDiagonalization B)
    (hWB : numericalRange B ⊆ Omega) :
    HasDoubleLayerCompletionProvider B (closure Omega) := by
  exact hasDoubleLayerCompletionProvider_of_parametricBoundary
    (G.parametricBoundary R c) B hWB
    (G.hasParametricPolynomialCauchyFormula_of_simpleDiagonalization
      R c B hB hWB)

end OrientedRadialConvexBoundary

end PositivePeriodicRadialData

end CrouzeixConjecture

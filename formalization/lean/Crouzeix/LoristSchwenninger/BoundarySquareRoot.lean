import CrouzeixConjecture.DoubleLayerPositiveMap

/-!
The canonical matrix square root of Harp's positive boundary density.

`PositiveBoundaryDensity` is positive semidefinite only almost everywhere.
The continuous-functional-calculus square root is nevertheless nonnegative at
every point (it is zero outside the nonnegative cone), while its square agrees
with the density on the almost-everywhere positivity set.
-/

noncomputable section

open MeasureTheory
open scoped ComplexOrder MatrixOrder

namespace CrouzeixConjecture
namespace LoristSchwenninger

variable {i n : Type*} [MeasurableSpace i]
  [Fintype n] [DecidableEq n]
variable {mu : Measure i}

/-- The canonical positive square root of the boundary density. -/
def boundarySquareRoot
    (D : PositiveBoundaryDensity (n := n) mu) (z : i) : SquareMatrix n :=
  CFC.sqrt (D.density z)

/-- The canonical square root is positive semidefinite at every point. -/
theorem boundarySquareRoot_posSemidef
    (D : PositiveBoundaryDensity (n := n) mu) (z : i) :
    (boundarySquareRoot D z).PosSemidef := by
  change (CFC.sqrt (D.density z)).PosSemidef
  exact Matrix.nonneg_iff_posSemidef.mp (CFC.sqrt_nonneg (D.density z))

/-- On the almost-everywhere set where the boundary density is positive
semidefinite, the canonical square root multiplied by itself recovers the
density. -/
theorem boundarySquareRoot_mul_self_ae
    (D : PositiveBoundaryDensity (n := n) mu) :
    ∀ᵐ z ∂mu,
      boundarySquareRoot D z * boundarySquareRoot D z = D.density z := by
  filter_upwards [D.posSemidef_ae] with z hz
  exact CFC.sqrt_mul_sqrt_self (D.density z) hz.nonneg

end LoristSchwenninger
end CrouzeixConjecture

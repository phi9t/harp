import CrouzeixConjecture.NumericalRange

/-!
An expanded specification of Harp's finite-matrix target. This module proves
correspondence and well-definedness of the maximum, not the terminal inequality.
Local compilation of this module is non-hermetic verification.
-/

noncomputable section

open scoped InnerProductSpace Matrix Matrix.Norms.L2Operator
open CrouzeixConjecture

namespace CrouzeixTextbook.HarpStatementAudit

universe u

/-- The matrix polynomial bound with every mathematical operand exposed.
The norm is on continuous linear maps of Euclidean space. The supremum runs
over polynomial moduli at the quadratic forms of unit Euclidean vectors.
This definition does not refer to `MainTheoremStatement`. -/
def textbookBound : Prop :=
  ∀ (n : Type u) [Fintype n] [DecidableEq n] [Nonempty n]
    (A : Matrix n n ℂ) (p : Polynomial ℂ),
    ‖(Matrix.toEuclideanCLM (𝕜 := ℂ) (n := n)) (Polynomial.aeval A p)‖ ≤
      2 * sSup ((fun z : ℂ ↦ ‖Polynomial.eval z p‖) ''
        {z : ℂ | ∃ x : EuclideanSpace ℂ n, ‖x‖ = 1 ∧
          ⟪x, (Matrix.toEuclideanCLM (𝕜 := ℂ) (n := n)) A x⟫_ℂ = z})

/-- Exact semantic correspondence, uniformly over finite nonempty index types.
Neither direction supplies a proof of either terminal proposition. -/
theorem textbookBound_iff_mainTheoremStatement :
    textbookBound.{u} ↔
      ∀ (n : Type u) [Fintype n] [DecidableEq n] [Nonempty n],
        MainTheoremStatement (n := n) := by
  rfl

variable {n : Type u} [Fintype n] [DecidableEq n]

/-- The explicit real-valued image whose supremum occurs in the target. -/
def polynomialModulusImage (A : Matrix n n ℂ) (p : Polynomial ℂ) : Set ℝ :=
  (fun z : ℂ ↦ ‖Polynomial.eval z p‖) ''
    {z : ℂ | ∃ x : EuclideanSpace ℂ n, ‖x‖ = 1 ∧
      ⟪x, (Matrix.toEuclideanCLM (𝕜 := ℂ) (n := n)) A x⟫_ℂ = z}

theorem polynomialModulusImage_nonempty [Nonempty n]
    (A : Matrix n n ℂ) (p : Polynomial ℂ) :
    (polynomialModulusImage A p).Nonempty :=
  (numericalRange_nonempty A).image _

theorem polynomialModulusImage_isCompact
    (A : Matrix n n ℂ) (p : Polynomial ℂ) :
    IsCompact (polynomialModulusImage A p) :=
  (isCompact_numericalRange A).image p.continuous.norm

theorem polynomialModulusImage_bddAbove
    (A : Matrix n n ℂ) (p : Polynomial ℂ) :
    BddAbove (polynomialModulusImage A p) :=
  (polynomialModulusImage_isCompact A p).bddAbove

/-- Nonemptiness and compactness justify calling this supremum a maximum. -/
theorem polynomialModulusImage_maximum [Nonempty n]
    (A : Matrix n n ℂ) (p : Polynomial ℂ) :
    ∃ x : EuclideanSpace ℂ n, ‖x‖ = 1 ∧
      ‖p.eval ⟪x, (Matrix.toEuclideanCLM (𝕜 := ℂ) (n := n)) A x⟫_ℂ‖ =
        sSup (polynomialModulusImage A p) := by
  obtain ⟨z, ⟨x, hx, rfl⟩, hz⟩ :=
    exists_maxPolynomialModulusOnNumericalRange A p
  exact ⟨x, hx, hz⟩

end CrouzeixTextbook.HarpStatementAudit

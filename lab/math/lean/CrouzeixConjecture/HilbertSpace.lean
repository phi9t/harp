module

public import CrouzeixConjecture.HilbertSpaceCore
public import CrouzeixConjecture.FinalTheorems

@[expose] public section

noncomputable section

namespace CrouzeixConjecture

variable {H : Type*} [NormedAddCommGroup H] [InnerProductSpace ℂ H]

/-- The finite-matrix theorem transported to an arbitrary nonzero finite-dimensional complex
Hilbert space, using the existing Jin theorem provider. -/
theorem finiteDimensionalHilbertPolynomialCrouzeix
    [FiniteDimensional ℂ H] [Nontrivial H]
    (A : H →L[ℂ] H) (p : Polynomial ℂ) :
    ‖operatorPolynomialEval p A‖ ≤
      2 * supPolynomialModulusOnOperatorNumericalRange A p :=
  finiteDimensionalHilbertPolynomialCrouzeix_of_mainTheorem
    (fun d => crouzeixConjecture (n := Fin d)) A p

/-- The manuscript's Hilbert-space consequence, using the existing Jin theorem provider. -/
theorem hilbertSpacePolynomialCrouzeix [CompleteSpace H] [Nontrivial H]
    (A : H →L[ℂ] H) (p : Polynomial ℂ) :
    ‖operatorPolynomialEval p A‖ ≤
      2 * supPolynomialModulusOnOperatorNumericalRange A p :=
  hilbertSpacePolynomialCrouzeix_of_mainTheorem
    (fun d => crouzeixConjecture (n := Fin d)) A p

end CrouzeixConjecture

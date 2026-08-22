import Crouzeix.LoristSchwenninger.Dilation

/-!
The real scalar sequence in the Lorist--Schwenninger recurrence.

Mathlib's complex inner product is conjugate-linear in its first argument and
linear in its second. Thus the source's operator-theory expression
`Re inner (E_n T^n x) x` is represented below as
`re (inner x ((E_n T^n) x))`; the two complex inner products are conjugates
and have the same real part.
-/

noncomputable section

namespace CrouzeixConjecture
namespace LoristSchwenninger

variable {E K : Type*}
variable [NormedAddCommGroup E] [InnerProductSpace Complex E]
  [FiniteDimensional Complex E] [CompleteSpace E] [Nontrivial E]
variable [NormedAddCommGroup K] [InnerProductSpace Complex K] [CompleteSpace K]

/-- The source scalar
`m_n = Re inner (E_n T^n x) x`, in mathlib's inner-product orientation. -/
def recurrenceScalar (data : DilationData (E := E) (K := K))
    (x : E) (n : Nat) : Real :=
  Complex.re (inner Complex x ((data.perturbation n * data.T ^ n) x))

/-- For a unit vector, the LS scalar sequence has the uniform absolute bound
needed by `bounded_recurrence_terminal_tendsto_zero`. -/
theorem recurrenceScalar_abs_le
    (data : DilationData (E := E) (K := K)) {x : E}
    (hx : norm x = 1) (n : Nat) :
    abs (recurrenceScalar data x n) <=
      data.bound * (2 + data.bound) := by
  let A := data.perturbation n * data.T ^ n
  calc
    abs (recurrenceScalar data x n) =
        abs (Complex.re (inner Complex x (A x))) := rfl
    _ <= norm (inner Complex x (A x)) := Complex.abs_re_le_norm _
    _ <= norm x * norm (A x) := norm_inner_le_norm _ _
    _ = norm (A x) := by rw [hx, one_mul]
    _ <= norm A * norm x := A.le_opNorm x
    _ = norm A := by rw [hx, mul_one]
    _ <= data.bound * (2 + data.bound) :=
      data.perturbation_mul_target_power_norm_le n

omit [Nontrivial E] in
/-- The first LS scalar is exactly the scalar appearing in Equation 3. -/
@[simp]
theorem recurrenceScalar_one
    (data : DilationData (E := E) (K := K)) (x : E) :
    recurrenceScalar data x 1 =
      Complex.re (inner Complex x (data.perturbation 1 (data.T x))) := by
  simp [recurrenceScalar, mul_apply_eq_comp]

end LoristSchwenninger
end CrouzeixConjecture

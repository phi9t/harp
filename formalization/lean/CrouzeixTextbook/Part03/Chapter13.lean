import CrouzeixConjecture.FunctionMaximum

namespace CrouzeixTextbook.Part03
open CrouzeixConjecture

noncomputable section

set_option linter.defProp false in
def max_modulus_on_compact_set := @maxFunctionModulusOnSet
set_option linter.defProp false in
def compact_maximum_exists := @exists_maxFunctionModulusOnSet
set_option linter.defProp false in
def pointwise_norm_le_maximum := @norm_function_le_maxFunctionModulusOnSet
set_option linter.defProp false in
def compact_maximum_nonnegative := @maxFunctionModulusOnSet_nonneg
set_option linter.defProp false in
def compact_maximum_monotone := @maxFunctionModulusOnSet_mono
set_option linter.defProp false in
def outer_maxima_converge := @tendsto_maxFunctionModulusOnSet_of_outerApproximation

namespace Exercises.Chapter13

/-- CFT-13-E01. -/
theorem exercise_01_solution (s : Set ℂ) (f : ℂ → ℂ) (hne : s.Nonempty) :
    maxFunctionModulusOnSet s f = sSup ((fun z : ℂ => ‖f z‖) '' s) ∧
      ((fun z : ℂ => ‖f z‖) '' s).Nonempty :=
  ⟨rfl, hne.image _⟩

/-- CFT-13-E02. -/
theorem exercise_02_solution (s : Set ℂ) (hne : s.Nonempty) (c : ℂ) :
    maxFunctionModulusOnSet s (fun _ => c) = ‖c‖ := by
  have himage : (fun z : ℂ => ‖(fun _ : ℂ => c) z‖) '' s = {‖c‖} :=
    hne.image_const ‖c‖
  rw [maxFunctionModulusOnSet, himage, csSup_singleton]

/-- CFT-13-E03. -/
theorem exercise_03_solution {s : Set ℂ} {f : ℂ → ℂ}
    (hs : IsCompact s) (hne : s.Nonempty) (hf : ContinuousOn f s) :
    IsGreatest ((fun z : ℂ => ‖f z‖) '' s) (maxFunctionModulusOnSet s f) := by
  obtain ⟨z, hz, hmax⟩ := exists_maxFunctionModulusOnSet hs hne hf
  refine ⟨⟨z, hz, hmax⟩, ?_⟩
  rintro _ ⟨w, hw, rfl⟩
  exact norm_function_le_maxFunctionModulusOnSet hs hne hf hw

/-- CFT-13-E04. -/
theorem exercise_04_solution {s : Set ℂ} {f : ℂ → ℂ}
    (hs : IsCompact s) (hne : s.Nonempty) (hf : ContinuousOn f s) :
    (∃ z ∈ s, ‖f z‖ = maxFunctionModulusOnSet s f) ∧
      0 ≤ maxFunctionModulusOnSet s f := by
  obtain ⟨z, hz, hmax⟩ := exists_maxFunctionModulusOnSet hs hne hf
  refine ⟨⟨z, hz, hmax⟩, ?_⟩
  rw [← hmax]
  exact norm_nonneg _

/-- CFT-13-E05. -/
theorem exercise_05_solution (f : ℂ → ℂ) :
    maxFunctionModulusOnSet (∅ : Set ℂ) f = 0 := by
  rw [maxFunctionModulusOnSet, Set.image_empty, Real.sSup_empty]

/-- CFT-13-E06. -/
theorem exercise_06_solution {s t : Set ℂ} {f : ℂ → ℂ}
    (hs : IsCompact s) (hsne : s.Nonempty)
    (ht : IsCompact t) (htne : t.Nonempty)
    (hf : ContinuousOn f t) (hst : s ⊆ t) :
    maxFunctionModulusOnSet s f ≤ maxFunctionModulusOnSet t f ∧
      0 ≤ maxFunctionModulusOnSet s f :=
  ⟨maxFunctionModulusOnSet_mono hs hsne ht htne hf hst,
    maxFunctionModulusOnSet_nonneg hs hsne (hf.mono hst)⟩

end Exercises.Chapter13

end
end CrouzeixTextbook.Part03

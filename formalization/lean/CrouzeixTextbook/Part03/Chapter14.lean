import CrouzeixConjecture.MatrixPowerSeries

namespace CrouzeixTextbook.Part03
open CrouzeixConjecture
open scoped ENNReal Matrix Matrix.Norms.L2Operator

noncomputable section

set_option linter.defProp false in
def matrix_power_series := @matrixPowerSeries
set_option linter.defProp false in
def matrix_power_series_coefficient := @matrixPowerSeries_apply
set_option linter.defProp false in
def matrix_power_series_radius := @one_le_matrixPowerSeries_radius
set_option linter.defProp false in
def matrix_power_series_sum := @matrixPowerSeriesSum
set_option linter.defProp false in
def matrix_power_series_analytic := @matrixPowerSeriesSum_analyticOnNhd_unitDisk
set_option linter.defProp false in
def matrix_power_series_converges := @matrixPowerSeries_hasSum

namespace Exercises.Chapter14

open Finset

/-- CFT-14-E01. -/
theorem exercise_01_solution {n : Type*} [Fintype n] [DecidableEq n]
    (a : ℕ → SquareMatrix n) (m : ℕ) (z : ℂ) :
    matrixPowerSeries a m (fun _ => z) = z ^ m • a m ∧
      matrixPowerSeriesSum a 0 = a 0 :=
  ⟨matrixPowerSeries_apply a m z, matrixPowerSeriesSum_zero a⟩

/-- CFT-14-E02. -/
theorem exercise_02_solution {n : Type*} [Fintype n] [DecidableEq n]
    (T : SquareMatrix n) (N : ℕ) :
    (∑ i ∈ range N, T ^ i) * (1 - T) = 1 - T ^ N :=
  geom_sum_mul_neg T N

/-- CFT-14-E03. -/
theorem exercise_03_solution {n : Type*} [Fintype n] [DecidableEq n]
    (a : ℕ → SquareMatrix n) (C : ℝ) (ha : ∀ m, ‖a m‖ ≤ C) :
    (1 : ENNReal) ≤ (matrixPowerSeries a).radius ∧
      AnalyticOnNhd ℂ (matrixPowerSeriesSum a) unitDisk :=
  ⟨one_le_matrixPowerSeries_radius a C ha,
    matrixPowerSeriesSum_analyticOnNhd_unitDisk a C ha⟩

/-- CFT-14-E04. -/
theorem exercise_04_solution {n : Type*} [Fintype n] [DecidableEq n]
    (a : ℕ → SquareMatrix n) (C : ℝ) (ha : ∀ m, ‖a m‖ ≤ C)
    {z : ℂ} (hz : z ∈ unitDisk) :
    HasSum (fun m => z ^ m • a m) (matrixPowerSeriesSum a z) ∧
      ∑' m, z ^ m • a m = matrixPowerSeriesSum a z := by
  have h := matrixPowerSeries_hasSum a C ha hz
  exact ⟨h, h.tsum_eq⟩

/-- CFT-14-E05. -/
theorem exercise_05_solution :
    (1 : ℂ) ∉ unitDisk ∧ (0 : ℂ) ∈ unitDisk := by
  constructor
  · simp [unitDisk]
  · simp [unitDisk]

/-- CFT-14-E06. -/
theorem exercise_06_solution {n : Type*} [Fintype n] [DecidableEq n]
    (B : SquareMatrix n) :
    (∀ m : ℕ, ‖(fun _ : ℕ => B) m‖ ≤ ‖B‖) ∧
      (1 : ENNReal) ≤ (matrixPowerSeries (fun _ => B)).radius :=
  ⟨fun _ => le_rfl,
    one_le_matrixPowerSeries_radius (fun _ => B) ‖B‖ (fun _ => le_rfl)⟩

end Exercises.Chapter14

end
end CrouzeixTextbook.Part03

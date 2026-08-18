import Mathlib

/-!
# NNG4 introductory stable worlds

This module proves Harp-owned counterparts of the ordinary theorem statements
in the stable worlds imported by upstream NNG4 `Game.lean` at commit
`727e4d219838eeb7f3945d2e9a0539f244d50540`.

The upstream `Power/L10FLT.lean` endpoint is intentionally excluded here:
NNG4 presents that level with the hidden `xyzzy` tactic, implemented by an
axiom-like macro in the game. Harp records that as a boundary, not as a proof
obligation satisfied by this early phase.
-/

namespace NNG4Intro

namespace Tutorial

theorem l01_rfl (x q : Nat) : 37 * x + q = 37 * x + q := by
  rfl

theorem l02_rw (x y : Nat) (h : y = x + 7) : 2 * y = 2 * (x + 7) := by
  rw [h]

theorem l03_two_eq_ss0 : 2 = Nat.succ (Nat.succ 0) := by
  rfl

theorem l04_rw_backwards : 2 = Nat.succ (Nat.succ 0) := by
  rfl

theorem l05_add_zero (a b c : Nat) : a + (b + 0) + (c + 0) = a + b + c := by
  omega

theorem l06_add_zero2 (a b c : Nat) : a + (b + 0) + (c + 0) = a + b + c := by
  omega

theorem l07_succ_eq_add_one (n : Nat) : Nat.succ n = n + 1 := by
  omega

theorem l08_two_add_two : (2 : Nat) + 2 = 4 := by
  norm_num

end Tutorial

namespace Addition

theorem l01_zero_add (n : Nat) : 0 + n = n := by
  omega

theorem l02_succ_add (a b : Nat) : Nat.succ a + b = Nat.succ (a + b) := by
  omega

theorem l03_add_comm (a b : Nat) : a + b = b + a := by
  omega

theorem l04_add_assoc (a b c : Nat) : a + b + c = a + (b + c) := by
  omega

theorem l05_add_right_comm (a b c : Nat) : a + b + c = a + c + b := by
  omega

end Addition

namespace Multiplication

theorem l01_mul_one (m : Nat) : m * 1 = m := by
  omega

theorem l02_zero_mul (m : Nat) : 0 * m = 0 := by
  omega

theorem l03_succ_mul (a b : Nat) : Nat.succ a * b = a * b + b := by
  rw [Nat.succ_mul]

theorem l04_mul_comm (a b : Nat) : a * b = b * a := by
  ring

theorem l05_one_mul (m : Nat) : 1 * m = m := by
  omega

theorem l06_two_mul (m : Nat) : 2 * m = m + m := by
  ring

theorem l07_mul_add (a b c : Nat) : a * (b + c) = a * b + a * c := by
  ring

theorem l08_add_mul (a b c : Nat) : (a + b) * c = a * c + b * c := by
  ring

theorem l09_mul_assoc (a b c : Nat) : (a * b) * c = a * (b * c) := by
  ring

end Multiplication

namespace Power

theorem l01_zero_pow_zero : (0 : Nat) ^ 0 = 1 := by
  norm_num

theorem l02_zero_pow_succ (m : Nat) : (0 : Nat) ^ Nat.succ m = 0 := by
  simp

theorem l03_pow_one (a : Nat) : a ^ 1 = a := by
  simp

theorem l04_one_pow (m : Nat) : (1 : Nat) ^ m = 1 := by
  simp

theorem l05_pow_two (a : Nat) : a ^ 2 = a * a := by
  ring

theorem l06_pow_add (a m n : Nat) : a ^ (m + n) = a ^ m * a ^ n := by
  rw [pow_add]

theorem l07_mul_pow (a b n : Nat) : (a * b) ^ n = a ^ n * b ^ n := by
  rw [mul_pow]

theorem l08_pow_pow (a m n : Nat) : (a ^ m) ^ n = a ^ (m * n) := by
  rw [pow_mul]

theorem l09_add_sq (a b : Nat) : (a + b) ^ 2 = a ^ 2 + b ^ 2 + 2 * a * b := by
  ring

end Power

namespace Implication

theorem l01_exact (x y z : Nat) (h1 : x + y = 37) (_h2 : 3 * x + z = 42) :
    x + y = 37 := by
  exact h1

theorem l02_exact2 (x y : Nat) (h : 0 + x = 0 + y + 2) : x = y + 2 := by
  omega

theorem l03_apply (x y : Nat) (h1 : x = 37) (h2 : x = 37 → y = 42) : y = 42 := by
  exact h2 h1

theorem l04_succ_inj (x : Nat) (h : x + 1 = 4) : x = 3 := by
  omega

theorem l05_succ_inj2 (x : Nat) (h : x + 1 = 4) : x = 3 := by
  omega

theorem l06_intro (x : Nat) : x = 37 → x = 37 := by
  intro h
  exact h

theorem l07_intro2 (x y : Nat) : x + 1 = y + 1 → x = y := by
  omega

theorem l08_ne (x y : Nat) (h1 : x = y) (h2 : x ≠ y) : False := by
  exact h2 h1

theorem l09_zero_ne_one : (0 : Nat) ≠ 1 := by
  norm_num

theorem l10_one_ne_zero : (1 : Nat) ≠ 0 := by
  norm_num

theorem l11_two_add_two_ne_five :
    Nat.succ (Nat.succ 0) + Nat.succ (Nat.succ 0) ≠
      Nat.succ (Nat.succ (Nat.succ (Nat.succ (Nat.succ 0)))) := by
  norm_num

end Implication

namespace AdvancedAddition

theorem l01_add_right_cancel (a b n : Nat) : a + n = b + n → a = b := by
  omega

theorem l02_add_left_cancel (a b n : Nat) : n + a = n + b → a = b := by
  omega

theorem l03_add_left_eq_self (x y : Nat) : x + y = y → x = 0 := by
  omega

theorem l04_add_right_eq_self (x y : Nat) : x + y = x → y = 0 := by
  omega

theorem l05_add_right_eq_zero (a b : Nat) : a + b = 0 → a = 0 := by
  omega

theorem l06_add_left_eq_zero (a b : Nat) : a + b = 0 → b = 0 := by
  omega

end AdvancedAddition

namespace LessOrEqual

theorem l01_le_refl (x : Nat) : x ≤ x := by
  omega

theorem l02_zero_le (x : Nat) : 0 ≤ x := by
  omega

theorem l03_le_succ_self (x : Nat) : x ≤ Nat.succ x := by
  omega

theorem l04_le_trans (x y z : Nat) (hxy : x ≤ y) (hyz : y ≤ z) : x ≤ z := by
  omega

theorem l05_le_zero (x : Nat) (hx : x ≤ 0) : x = 0 := by
  omega

theorem l06_le_antisymm (x y : Nat) (hxy : x ≤ y) (hyx : y ≤ x) : x = y := by
  omega

theorem l07_or_symm (x y : Nat) (h : x = 37 ∨ y = 42) : y = 42 ∨ x = 37 := by
  exact h.symm

theorem l08_le_total (x y : Nat) : x ≤ y ∨ y ≤ x := by
  omega

theorem l09_succ_le_succ (x y : Nat) (hx : Nat.succ x ≤ Nat.succ y) : x ≤ y := by
  omega

theorem l10_le_one (x : Nat) (hx : x ≤ 1) : x = 0 ∨ x = 1 := by
  omega

theorem l11_le_two (x : Nat) (hx : x ≤ 2) : x = 0 ∨ x = 1 ∨ x = 2 := by
  omega

end LessOrEqual

namespace AdvancedMultiplication

theorem l01_mul_le_mul_right (a b t : Nat) (h : a ≤ b) : a * t ≤ b * t := by
  exact Nat.mul_le_mul_right t h

theorem l02_mul_left_ne_zero (a b : Nat) (h : a * b ≠ 0) : b ≠ 0 := by
  intro hb
  exact h (by simp [hb])

theorem l03_eq_succ_of_ne_zero (a : Nat) (ha : a ≠ 0) : ∃ n, a = Nat.succ n := by
  cases a with
  | zero => contradiction
  | succ n => exact ⟨n, rfl⟩

theorem l04_one_le_of_ne_zero (a : Nat) (ha : a ≠ 0) : 1 ≤ a := by
  omega

theorem l05_le_mul_right (a b : Nat) (h : a * b ≠ 0) : a ≤ a * b := by
  have hb : 1 ≤ b := by
    apply l04_one_le_of_ne_zero
    intro hb
    exact h (by simp [hb])
  nlinarith [Nat.mul_le_mul_left a hb]

theorem l06_mul_right_eq_one (x y : Nat) (h : x * y = 1) : x = 1 := by
  exact Nat.eq_one_of_mul_eq_one_right h

theorem l07_mul_ne_zero (a b : Nat) (ha : a ≠ 0) (hb : b ≠ 0) : a * b ≠ 0 := by
  exact Nat.mul_ne_zero ha hb

theorem l08_mul_eq_zero (a b : Nat) (h : a * b = 0) : a = 0 ∨ b = 0 := by
  by_cases ha : a = 0
  · exact Or.inl ha
  · right
    by_contra hb
    exact Nat.mul_ne_zero ha hb h

theorem l09_mul_left_cancel (a b c : Nat) (ha : a ≠ 0) (h : a * b = a * c) : b = c := by
  exact Nat.mul_left_cancel (Nat.pos_of_ne_zero ha) h

theorem l10_mul_right_eq_self (a b : Nat) (ha : a ≠ 0) (h : a * b = a) : b = 1 := by
  have : a * b = a * 1 := by simpa using h
  exact Nat.mul_left_cancel (Nat.pos_of_ne_zero ha) this

end AdvancedMultiplication

namespace Algorithm

theorem l01_add_left_comm (a b c : Nat) : a + (b + c) = b + (a + c) := by
  omega

theorem l02_add_algo1 (a b c d : Nat) : a + b + (c + d) = a + c + d + b := by
  omega

theorem l03_add_algo2 (a b c d e f g h : Nat) :
    (d + f) + (h + (a + c)) + (g + e + b) = a + b + c + d + e + f + g + h := by
  omega

theorem l04_add_algo3 (a b c d e f g h : Nat) :
    (d + f) + (h + (a + c)) + (g + e + b) = a + b + c + d + e + f + g + h := by
  omega

theorem l05_pred (a b : Nat) (h : Nat.succ a = Nat.succ b) : a = b := by
  omega

theorem l06_succ_ne_zero (a : Nat) : Nat.succ a ≠ 0 := by
  omega

theorem l07_succ_ne_succ (m n : Nat) (h : m ≠ n) : Nat.succ m ≠ Nat.succ n := by
  omega

theorem l08_decide : (20 : Nat) + 20 = 40 := by
  norm_num

theorem l09_decide2 : (2 : Nat) + 2 ≠ 5 := by
  norm_num

end Algorithm

def provenStableLevelCount : Nat := 77

theorem provenStableLevelCount_eq : provenStableLevelCount = 77 := by
  rfl

def excludedMagicEndpointCount : Nat := 1

theorem excludedMagicEndpointCount_eq : excludedMagicEndpointCount = 1 := by
  rfl

end NNG4Intro

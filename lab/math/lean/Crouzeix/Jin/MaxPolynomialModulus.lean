module

public import CrouzeixConjecture.Statements

@[expose] public section

/- Harp-native Crouzeix route scaffolding.

This module intentionally contains no imported Jin terminal theorem. It starts
with the first source-mapped Jin interface: the maximum polynomial modulus on a
matrix numerical range.
-/

noncomputable section

namespace CrouzeixConjecture

open scoped Matrix.Norms.L2Operator

variable {n : Type*} [Fintype n] [DecidableEq n]

end CrouzeixConjecture

import Lean

/- Deliberately restricted dependencies for imported-body audit regressions.
This file is outside the production Lean library. -/
theorem importedForbidden : True := True.intro
theorem importedAlias : True := importedForbidden
axiom importedUntrusted : True
theorem importedHiddenAxiom : True := importedUntrusted

/-- warning: declaration uses `sorry` -/
#guard_msgs in
theorem importedSorry : True := by sorry

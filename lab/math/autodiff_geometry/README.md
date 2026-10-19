# Autodiff Geometry Lean formalization

This Lean library is the first finite-coordinate phase for the autodiff
geometry packet. It formalizes algebraic contracts named by the JAX autodiff
cookbook:

- `gradArg gradient i` selects the `i`th component of a finite-coordinate
  gradient, matching the cookbook's `grad(f, i)` notation boundary;
- `jvp jacobian tangent` is Jacobian-vector multiplication;
- `vjp jacobian cotangent` is multiplication by the transposed Jacobian;
- `hvp hessian v` is Hessian-vector multiplication; and
- `dot_jvp_eq_dot_vjp` proves finite-coordinate JVP/VJP duality.

The project does not model JAX tracing, floating-point behavior, pytrees,
complex differentiation, or analytic differentiability. Those remain roadmap
items in `knowledge/autodiff_geometry/formalization_roadmap.md`.

The toolchain and dependency graph are pinned by the shared Lake root in
`../lean/`, so proof iteration reuses the same mathlib cache as the other Harp
Lean libraries. Use `mise run lean-autodiff` for focused builds, or
`mise run lean-all` to warm and check the full shared root. The compatibility
verifier remains `scripts/check_autodiff_geometry_lean.sh`.

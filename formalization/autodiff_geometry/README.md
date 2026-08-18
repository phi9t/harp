# Autodiff Geometry Lean formalization

This standalone Lean project is the first finite-coordinate phase for the
autodiff geometry packet. It formalizes algebraic contracts named by the JAX
autodiff cookbook:

- `gradArg gradient i` selects the `i`th component of a finite-coordinate
  gradient, matching the cookbook's `grad(f, i)` notation boundary;
- `jvp jacobian tangent` is Jacobian-vector multiplication;
- `vjp jacobian cotangent` is multiplication by the transposed Jacobian;
- `hvp hessian v` is Hessian-vector multiplication; and
- `dot_jvp_eq_dot_vjp` proves finite-coordinate JVP/VJP duality.

The project does not model JAX tracing, floating-point behavior, pytrees,
complex differentiation, or analytic differentiability. Those remain roadmap
items in `knowledge/autodiff_geometry/formalization_roadmap.md`.

The toolchain is pinned by `lean-toolchain`, and the verifier is
`scripts/check_autodiff_geometry_lean.sh`.

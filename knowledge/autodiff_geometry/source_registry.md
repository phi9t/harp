---
id: autodiff-geometry-source-registry
title: Autodiff geometry source registry
type: source-registry
status: active
created: 2026-08-18
updated: 2026-08-18
tags: [autodiff, jax, sources, provenance]
confidence: medium
---

# Autodiff geometry source registry

Back to the [[knowledge/autodiff_geometry/autodiff_geometry_index|packet index]].

## Source classes and boundary

The first phase uses public first-party sources and immutable Git identities
where available. It records textbook links as notation-lineage evidence, not as
permission to reproduce book content.

| ID | Identity | Stability | Local artifact | Claim ceiling |
|---|---|---|---|---|
| `JAX-AUTODIFF-COOKBOOK` | JAX documentation notebook `docs/notebooks/autodiff_cookbook.ipynb` at Git commit `0eb0f676ba22b15b9dfe29518ab0d40277b7138d`; rendered `latest` page observed 2026-08-18 with HTTP 200 and ETag `W/"97c466ee4490ce1618af9352d85ff6e6"` | The rendered `latest` page is mutable; the Git commit is immutable. | No vendored bytes in this phase. | Cookbook notation claims, examples, and API-level descriptions of `grad`, `argnums`, `jvp`, `vjp`, `jacfwd`, `jacrev`, and Hessian-vector products. Not implementation verification. |
| `JAX-API-SOURCE` | JAX source file `jax/_src/api.py` at Git commit `0eb0f676ba22b15b9dfe29518ab0d40277b7138d` | Immutable Git commit. | No vendored bytes in this phase. | Existence and location of public transformation definitions such as `grad`, `value_and_grad`, `jacfwd`, `jacrev`, `hessian`, `jvp`, and `vjp`. Not semantic equivalence to Lean. |
| `SPIVAK-CALCULUS-ON-MANIFOLDS-REPO` | Public GitHub repository `zongpingding/Calculus_On_Manifolds_Michael_Spivak`, default branch `main`, observed at `ac453ea3c0a43fe2bc3ee5b144bee5d9d1cf174f` | Repository is mutable; observed commit is immutable. | No vendored bytes in this phase. | Bibliographic/source availability for Spivak notation lineage only. This packet does not reproduce or rely on the scan text. |
| `SICM-COURSE-PAGE` | Gerald Jay Sussman's MIT CSAIL course page for 6.946/6.5160, observed 2026-08-18; page links SICM second edition and an HTML mechanics book mirror. | Mutable dated web observation. | No vendored bytes in this phase. | Public-course pointer to SICM second edition and HTML reading route. Not a source for quoted mechanics definitions. |
| `MITPRESS-SICM` | MIT Press URL linked by the JAX cookbook for *Structure and Interpretation of Classical Mechanics*, second edition. | Browser-facing page returned HTTP 403 to plain `curl` on 2026-08-18. | No vendored bytes. | Dated pointer only; not local page evidence. |
| `MITPRESS-FDG` | MIT Press URL linked by the JAX cookbook for *Functional Differential Geometry*. | Browser-facing page returned HTTP 403 to plain `curl` on 2026-08-18. | No vendored bytes. | Dated pointer only; not local page evidence. |

## Capture notes

The JAX cookbook rendered page was available at:

`https://docs.jax.dev/en/latest/notebooks/autodiff_cookbook.html`

The immutable notebook source used for claim locators was:

`https://raw.githubusercontent.com/jax-ml/jax/0eb0f676ba22b15b9dfe29518ab0d40277b7138d/docs/notebooks/autodiff_cookbook.ipynb`

The API source locator used for function definitions was:

`https://raw.githubusercontent.com/jax-ml/jax/0eb0f676ba22b15b9dfe29518ab0d40277b7138d/jax/_src/api.py`

## Cannot-prove boundaries

- The JAX docs describe the public API and notation contract; they do not prove
  Harp's Lean statements.
- The Lean project proves finite-coordinate algebraic contracts; it does not
  validate JAX's Python implementation or floating-point execution.
- The Spivak repository is a public source pointer. This packet does not assert
  redistribution rights for the underlying book scan.
- MIT Press SICM and FDG pages were not captured by plain `curl`; claims about
  those books stay at the level of JAX's own bibliographic pointer until a later
  evidence capture succeeds.

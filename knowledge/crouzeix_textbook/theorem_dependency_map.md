---
id: cft-theorem-dependency-map
title: Theorem dependency map
type: reference
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, mathematics, lean]
confidence: high
canonical: theorem_dependency_map.md
---

# Theorem dependency map

This page is the stable reader route for the pedagogical dependency report. It
does not maintain a second graph or copy the derived edge table. The sole
structured authority is each theorem row's `pedagogical_prerequisites` field in
`content/crouzeix_textbook/coverage.json`.

## Reading the generated ledger

Run `mise run crouzeix-textbook-publication`. The Rust publisher validates the
graph together with the book contracts, compiles a fresh Lean receipt, switches
one immutable generation, and checks the selected generation.

The validated pointer is
`atlas/src/content/generated/crouzeix_textbook/publication/current.json`. Its
generation identifies `pedagogical_dependency_ledger.md`, which gives the
direct prerequisites for every CFT theorem identity. Follow edges backward to
recover what a chapter expects and forward through the chapter routes to see
where an idea is used.

## Two dependency notions

Pedagogical prerequisites describe the order in which the book teaches ideas.
They are not Lean kernel dependencies. The same Rust publication contains a
separate `kernel_dependency_ledger.md` derived from the compiled receipt. The
distinction matters most after the common Crouzeix machinery branches into the
Jin and Lorist--Schwenninger routes.

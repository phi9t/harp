---
id: cft-exercise-index
title: Exercise index
type: reference
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, mathematics, lean]
confidence: high
canonical: exercise_index.md
---

# Exercise index

This page is the stable reader route for the generated exercise report. It
does not duplicate the exercise roster. The structured authority is
`content/crouzeix_textbook/exercises.json`, cross-validated against the
coverage contract and the chapter anchors.

## Reading the generated ledger

Run `mise run crouzeix-textbook-publication`. The task compiles a fresh receipt,
publishes one immutable ledger generation, and checks it before static
consumption.

The validated pointer is
`atlas/src/content/generated/crouzeix_textbook/publication/current.json`. Its
generation identifies `exercise_ledger.md`, which records exercise identity,
chapter, kind, difficulty, prerequisite skills, starter location, and any
distinct checked Lean solution. An absent solution remains visibly incomplete;
it is never replaced by the theorem being studied.

## Finding an exercise in the book

Use the CFT exercise identity from the generated ledger to follow the exact
chapter anchor. The [[knowledge/crouzeix_textbook/crouzeix_textbook_index|book index]]
gives the chapter route, while
[[knowledge/crouzeix_textbook/status_and_scope|status and scope]] explains what
the presence or absence of a formal solution certifies.

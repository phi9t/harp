# Narrative theorem-card validator review

Date: 2026-09-07. Base: `4a72888fd9b09d86e958da35074e86c08615a257`.
Candidate is uncommitted in `codex/harp-mathematics-spec`.

## Scope and behavior

Task 4a repairs the publication integration mismatch in the existing Markdown
owner. `check` and therefore `generate_correspondence` now accept the approved
continuous narrative proof format. No public API, manifest, chapter-specific
exception, theorem promotion, Lean source, or canonical chapter prose changes
belong to this patch.

The existing ten-heading workshop requirements and diagnostic code remain.
Cards without workshop subsection headings may instead supply explicit bold
Statement, Proof, Boundary, and Formal correspondence labels. Each must occur
exactly once and have nonempty parsed content. The approved combined “Boundary
and Lean provider(s)” form supplies both of those fields. Formal correspondence
must contain an inline Lean-style declaration identifier or a `.lean` code
link; unrelated prose, a bare “Lean”, or an arbitrary general link does not
suffice. This checks the presence of an inspectable provider, not mathematical
correctness or declaration resolution.

The parser recognizes paragraph-leading labels, including Statement immediately
after a bold theorem title. Code, fenced or indented examples, quoted text,
inline label mentions, and heading text cannot supply narrative labels.
Labels must contain plain bold text; breaks, HTML, and nested inline markup
invalidate a candidate label instead of being silently dropped. Quoted headings
cannot contribute workshop subsections or select the workshop format. Dotted
provider identifiers require nonempty components beginning with a letter or
underscore, followed by letters, digits, underscores, or apostrophes.
Unrelated leading bold labels end the previous field, so their contents cannot
fill an empty required field. Both formats use the canonical heading identities
to delimit a card, including a nested registered card. Unregistered deeper
detail headings remain within the current card. Motivation, historical context,
and ML discussion remain chapter-level editorial concerns covered by separate
foundations fixtures.

## TDD evidence

Before production edits,
`narrative_accepts_current_approved_column_rule_through_public_check` failed
through the public `check` API on the actual Chapter 1 column-rule narrative:
`crouzeix-textbook.markdown.theorem-card`, with all ten workshop subsection
counts zero. That fixture now passes both `check` and
`generate_correspondence`.

A second red run established that an H2 narrative missing its proof could
borrow the ten workshop headings of the next registered H3 card. The
`narrative_cannot_borrow_a_nested_registered_workshop_card` regression failed
because `check` incorrectly returned success. Sharing the canonical card
boundary fixed this without relaxing any workshop requirement.

## Quality-review repair

Independent quality review identified three parser gaps. Before repairing
production code, the expanded public-check suite was run with
`cargo test --profile test-small -p harp --test foundations_narrative`:
10 tests passed and 4 failed. The failures established:

- A Statement-only narrative became incorrectly acceptable after appending
  all ten workshop headings inside a block quote.
- A complete narrative was incorrectly rejected after adding a quoted Proof
  heading; the quoted heading selected workshop validation.
- All six malformed sole provider identifiers were accepted: `1.bad`,
  `foo..bar`, `foo.`, `.foo`, `foo.1bar`, and `foo.'bar`.
- All seven interrupted Proof labels were accepted: a soft break, two hard
  break spellings, an HTML comment, empty HTML emphasis tags, nested Markdown
  emphasis, and an inline link inserted into the label.

The repair excludes quoted headings from subsection counting without changing
canonical heading-identity iteration. It validates dotted identifier components
and rejects non-text events within bold labels. The exact same 14-test suite
then passed; five plausible qualified or underscored identifiers remain
accepted. These are syntax and presence checks, not a new Lean name resolver.

## Focused verification

Commands ran under `PATH=/opt/homebrew/bin:$PATH /Users/bytedance/.local/bin/mise exec -- sh -c '…'`
with `CARGO_TARGET_DIR="$HARP_TARGET_DIR"`, resolving to the existing
worktree-local `.build/harp-target`.

- `cargo test --profile test-small -p harp --test foundations_narrative`:
  14 passed, 0 failed; 66 temporary fixture scenarios include current prose,
  combined fields, nested detail headings, missing/empty/duplicate fields,
  code and quote impersonation, sibling/nested-card borrowing, and invalid
  provider content, quoted workshop impersonation, malformed identifiers,
  and interrupted labels.
- `cargo test --profile test-small -p harp --test crouzeix_textbook prose_anchor`:
  6 passed, 0 failed.
- `cargo test --profile test-small -p harp --test crouzeix_textbook theorem_card`:
  3 passed, 0 failed (one overlaps the preceding filter).
- `rustfmt --check --edition 2021 crates/harp/src/crouzeix_textbook/markdown.rs crates/harp/tests/foundations_narrative.rs`:
  passed.
- `cargo clippy --profile test-small -p harp --lib --test foundations_narrative -- -D warnings`:
  passed.
- `git diff --check -- crates/harp/src/crouzeix_textbook/markdown.rs`:
  passed.

No Lean/Lake command, dependency-cache maintenance, full repository gate,
commit, or push was performed. Publication remains the parent workstream's
integration responsibility after the chapter candidate is frozen.

## Frozen candidate

SHA-256:

- `crates/harp/src/crouzeix_textbook/markdown.rs`:
  `6a732511b4fb3766dfa171c59522e5a033a1c140e918188cc28255d2a5013927`.
- `crates/harp/tests/foundations_narrative.rs`:
  `b2680bda23d725c10d42595f553d231d0d2cab0152f806c99a7f058e9b693485`.

## Independent acceptance

The specification reviewer independently ran the initial public-check,
anchor and workshop tests and passed the bounded implementation. After the
three quality findings were repaired, the same quality reviewer checked
the final hashes and independently ran all 14 narrative tests, six anchor
tests, three workshop-filter tests, and eight original reproduction/control
cases against the compiled public API. All passed, with no remaining
findings. Final publication and repository-wide acceptance remain separate
gates.

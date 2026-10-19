# Completion Contract

## Owned State

- CAPF provider list and ordering.
- Replacement start/end range.
- Completion collection and metadata properties.
- Completion overlay or ghost-text frontend state.
- Trigger context, cancellation state, and accept path.

Primary local sources:

- `/opt/soma/emacs/share/emacs/30.2/lisp/minibuffer.el.gz:2918-3003`
- `/opt/soma/emacs/share/emacs/30.2/lisp/minibuffer.el.gz:2451-2504`
- `/opt/soma/emacs/share/emacs/30.2/lisp/completion-preview.el.gz:221-236`
- `/opt/soma/emacs/share/emacs/30.2/lisp/completion-preview.el.gz:380-450`
- `/opt/soma/emacs/share/emacs/30.2/lisp/completion-preview.el.gz:452-539`
- `/private-source/home/operator/workspace/devx/doom/ai-completion.el:1-74`

## Required Session Context

- Buffer identity and text-owner identity.
- Point and replacement range.
- Prefix/suffix snapshot.
- `buffer-chars-modified-tick` and additional interaction state relevant to the
  frontend.
- Active provider, overlay, and exit function.
- CAPF provider order and `:exclusive` property at the time completion was
  requested.

## Faithful Command Paths

- CAPF providers must return a valid `(START END COLLECTION . PROPS)` tuple.
- Provider ordering and `:exclusive` behavior matter.
- Ghost-text accept paths must run the frontend's intended insert/exit logic.
- DevX config maps Copilot acceptance keys and Tabbymacs ghost-text acceptance;
  those are user-profile specifics, not generic Emacs defaults.

## Async Work

- Model requests and cancellation.
- LSP/kernel CAPF queries.
- Provider exit functions with arbitrary side effects.

## Stale Handles

- Overlay after text changes.
- Replacement range after user edits.
- Provider cache after buffer modification.
- Backend mode switch between Copilot and Tabbymacs.

## Completion Evidence

- Suggestion displayed only if request context still matches.
- Acceptance applies exactly the intended replacement.
- Undo groups the accepted insertion sensibly.
- Provider exit function status is correct for full or partial acceptance.
- No suffix duplication or unrelated rewrite.

## Negative Cases

- Marker alive but prefix/suffix stale.
- Provider range changed after request but suggestion accepted.
- TAB steals indentation/snippet/CAPF behavior unpredictably.
- Ghost text accepted after backend was disabled or switched.

Probe `fim-stale-acceptance` showed the core stale-acceptance rule: a marker can
remain live while buffer text and tick change, so acceptance must be rejected.

Probe `capf-provider-exclusivity` showed that CAPF sources are not a generic
merged list. An exclusive first provider can own the attempt and block fallback;
only `:exclusive no` allows the next provider to answer after a failed prefix
match.

Probe `capf-exit-function-acceptance` showed that a text-equivalent insertion
is not necessarily a faithful completion acceptance. A CAPF provider can attach
an `:exit-function` through `completion-extra-properties`; the ordinary
completion path calls it with the accepted string and status, while a raw
replacement can produce the same final buffer text with no provider side
effect. Model ghost-text acceptance must therefore report whether it is using
CAPF/preview acceptance or a separate overlay path, and must explicitly account
for provider exit-function behavior.

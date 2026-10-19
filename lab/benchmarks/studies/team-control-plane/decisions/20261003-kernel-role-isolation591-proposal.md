# Proposed bounded role-isolation repair591

Commit read: `9055b9976d429229e5d42b2e86eb04dbc30da5bf`.
Status: assessed valid; implementation awaiting explicit repair-budget extension.

Whole-branch591 returned one Medium finding after the existing three repair iterations. Read-only author assessment and root query inspection confirm status admits same-principal cross-role episode records while exact inspection denies them. Public grants permit principal reuse across roles; current tests use different principals and miss this case.

Proposed scope: require authenticated role equality for non-Owner episode execution and episode-native operation summary SQL, before ordering/33-row limits. Preserve Owner visibility, current-role principal/scope filters, legacy authorization semantics and existing bounded output. No schema/provider/workflow changes.

Proposed proof: real public Worker execution/native operation with Worker and Verifier grants sharing principal and scope. Verifier summaries must omit Worker execution/thread/turn/effect identifiers and exact inspection must refuse them. Worker and Owner retain expected visibility. Add boundary records showing unauthorized rows cannot consume32visible slots or affect overflow flags. Establish a meaningful red, apply the minimal query fix, prove green, run fullPG/Temporal and repository gates/strict validation, freeze, two-axis review and one fresh whole-branch review.

Request: one additional bounded repair/review iteration for this finding only, without resetting the earlier three-iteration history. If that round does not converge, stop implementation and report findings for a new owner decision. No patch or runtime has been launched under this proposal; main remains unchanged.

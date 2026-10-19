# Third bounded jj19 repair: exported reachable history

Wholebranch630 (`264756e..ba5dfa3`, session20281) terminates1 with a Medium
private-file export finding. Root confirmed freeze_candidate validates the
current tree while prepare_integration exports reachable history. A private
path committed and then deleted can remain recoverable in the bundle. Earlier
conformance and gate passes do not override this finding; acceptance/landing
remain withheld.

Repair3/3 uses a fresh isolated writer tree at `d819a94`, preserving the original
100 evidence records and repair1/2 inputs. Validate the actual exported history
against the qualified private/disallowed-path capture contract before publishing
a bundle/ref. Direct export cannot bypass validation with stale/forged candidate
data. Tests deliberately commit synthetic private content, descend/delete and
require refusal. Preserve clean-candidate exact verification and ordinary
Git interoperability; no shared rewrite, pruning or magical secret detection.

Only private disposable jj fixtures and necessary focused tooling checks are
admitted. No Cargo/PG/Temporal/provider/Trae/shared-main/remote operation or
commit by the author. Root retains fresh conformance, branch review and gated
landing. If this final round is uncertain, return to architectural assessment;
do not relabel incomplete privacy qualification as accepted.

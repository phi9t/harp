# Ticket 14: worktree compatibility shim

Status: implemented and locally verified; independent acceptance remains with the lead.
Implementation base: ff66dd24df0f90ae85c3f1dab286cd3b0c156c8c.

`tools/agents/wt.py` now exports the legacy API from
`tools/collab/worktree_compat.py`. Existing flags and exit codes remain.
Help identifies the interface as deprecated. Creation and removal use a
nonblocking repository lock. Metadata corruption and identity mismatches are
refusals instead of silently empty metadata. Creation writes metadata atomically.

`--mainline REF` and `git config collab.mainline REF` select the comparison target.
Status keeps local ahead/behind fields and adds remote_mainline, remote_ahead,
and remote_behind. An upstream or existing origin tracking ref supplies remote
comparison; status performs no network fetch.

## Authorized compatibility correction

The original GC test expected newly created worker/merged and worker/old trees
to be deleted merely because neither had commits ahead. The lead explicitly
prioritized story 38 and authorized correcting this unsafe expectation.
The revised test requires both trees and branches to survive. Other legacy
assertions and all other tests remain unchanged. Artificially aging a working
Attempt does not make it terminal. New metadata records state working, and GC
requires an explicitly terminal state before considering cleanliness and merge.

Red evidence: the corrected legacy test failed on the original implementation,
listing worker/merged and worker/old where the expected candidate set was empty.
The new public test also showed the original GC deleting its fresh directory.
A concurrent-creation regression executed the original CLI from the fixed base
in a disposable repository and observed exits 0 and 64 instead of success 0
and refusal 1. The migrated implementation returns 0 and 1.

## Verification

```sh
python3 tools/temp_root.py -- python3 tools/agents/test_wt.py
# 12 tests passed, with the one authorized regression correction
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_cleanup.py -v
# 8 tests passed
```

The public CLI checks also establish corrupt metadata refusal, deprecation help,
configured local mainline selection, and divergence against a newer remote ref.
The same-target creation test starts two real CLI processes. Exactly one succeeds.
The retained finish API removes clean merged trees, including when invoked from
inside the tree. Dirty or locked worktrees and changed branch identities refuse.

No shared branch was moved. No real workspace GC was applied. The lead owns the
full repository gate, independent conformance review, and final acceptance.

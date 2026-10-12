# 25 archive-python-compat: Restore archives on the supported host Python versions

Type: task
Status: claimed
Owner: trae
Blocked by: 
Claimed: 2026-10-12T17:37:34Z expires 2026-10-13T17:37:34Z

Triage: needs-triage

## Context

Draft follow-up requested on 2026-10-12. Mainline commit e69d9ce introduced `TarFile.extract(..., filter="fully_trusted")`. Combined source 365876ed fails four existing archive/sweep tests on the host Python with `TypeError: unexpected keyword argument filter`. See evidence/mainline-combined-validation.json. No fix was made during the snapshot landing.

## Acceptance
- [ ] Archive restoration works on the repository-supported Python versions, including this host Python, while preserving the intended explicit extraction policy.
- [ ] Existing dirty-archive, restore-verification and sweep tests pass through public CLIs with modes, symlinks and external-target preservation intact.
- [ ] The final combined Collab suite passes all 170 current tests and the follow-up qualification ticket records this regression.

## Comments
- 2026-10-12T17:37:34Z trae: claimed until 2026-10-13T17:37:34Z

# 25 archive-python-compat: Restore archives on the supported host Python versions

Type: task
Status: claimed
Owner: trae
Blocked by: 
Claimed: 2026-10-12T17:37:34Z expires 2026-10-13T17:37:34Z

Triage: ready-for-agent

## Context

Draft follow-up requested on 2026-10-12. Mainline commit e69d9ce introduced `TarFile.extract(..., filter="fully_trusted")`. Combined source 365876ed fails four existing archive/sweep tests on the host Python with `TypeError: unexpected keyword argument filter`. See evidence/mainline-combined-validation.json. No fix was made during the snapshot landing.

## Acceptance
- [x] Archive restoration works on the repository-supported Python versions, including this host Python, while preserving the intended explicit extraction policy.
- [x] Existing dirty-archive, restore-verification and sweep tests pass through public CLIs with modes, symlinks and external-target preservation intact.
- [x] The final combined Collab suite passes all 170 current tests and the follow-up qualification ticket records this regression.

## Comments
- 2026-10-12T17:37:34Z trae: claimed until 2026-10-13T17:37:34Z
- 2026-10-12T17:51:00Z trae: added Python-version capability check for `TarFile.extract(filter=...)`; archive restoration still validates member paths, symlink traversal and restored inventory. Evidence: `python3 -m unittest discover -s tools/collab/tests -p test_cleanup.py -v` passed 9 tests; `python3 -m unittest discover -s tools/collab/tests -p test_sweep_repair.py -v` passed 3 tests; `python3 -m unittest discover -s tools/collab/tests -p test_collab.py -k sweep -v` passed 7 tests; full `python3 -m unittest discover -s tools/collab/tests -v` passed 170 tests.

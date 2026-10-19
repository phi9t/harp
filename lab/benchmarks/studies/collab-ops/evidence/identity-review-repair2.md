# Bookkeeping identity repair

The independent branch review found that the private bookkeeping clone lost
repository-local commit identity. A landing could therefore publish code and
then repeatedly fail its tracker record operation, or attribute it to a different
global identity.

The locked preflight now validates the original repository's effective author
and committer identity before publication. A durable Attempt event binds that
identity for the private bookkeeping commit and later recovery. Standalone
landings that need no record commit do not require this extra preparation.

Two public CLI regressions were red before the repair: the record commit used
the global identity instead of the repository-local identity, and missing local
identity allowed code publication. Both now pass. The complete lifecycle file
passed all 10 tests, including interrupted record recovery and override exports.

Validation command:
`python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_lifecycle_ops.py -v`

This focused evidence does not replace the final integration gate or independent
review of the combined repair.

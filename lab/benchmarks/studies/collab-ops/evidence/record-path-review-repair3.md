# Bookkeeping path containment repair

The separate branch defect review found that candidate-controlled evidence
symlinks could redirect post-landing record writes into host files outside the
private bookkeeping checkout.

Locked landing preflight now checks every selected record file and ancestor in
the exact Integration tree. Symlinked or non-regular targets are refused before
code publication. Recovery repeats that check before record work. Record append
operations create and open each directory through its parent descriptor with
`O_NOFOLLOW`, then open an unshared regular file without following links. The
private workspace root and lexical containment are validated, and appended
records are flushed and synced before their commit.

Four public CLI regressions were red before the repair: symlinked acceptance,
landing and override files, and an evidence directory symlink. They now pass and
prove the external sentinel bytes and trunk remain unchanged. The existing ten
lifecycle cases also pass, including normal resolution, overrides, identity
validation and interrupted record recovery.

Commands:

```sh
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_record_paths.py -v
python3 tools/temp_root.py -- python3 -m unittest discover -s tools/collab/tests -p test_lifecycle_ops.py -v
```

Initial discovery accidentally collected the imported lifecycle test class too;
the test module now imports its module, and the fresh path suite contains exactly
four cases. Final integration and independent review remain separately recorded.

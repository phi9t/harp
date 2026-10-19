`cachekey.build_cache_key(source, workspace)` must return a deterministic cache
key for a file inside a workspace. The key must be:

```
<relative-posix-path>:<sha256-of-file-bytes>
```

`source` and `workspace` may be strings or `pathlib.Path` objects. Resolve both
paths before comparing them, reject sources outside the workspace with
`ValueError`, and always use `/` as the separator in the relative path. The key
must not depend on the absolute temporary directory that contains the workspace.
Fix `cachekey.py`; the project uses uv offline and has no dependencies.

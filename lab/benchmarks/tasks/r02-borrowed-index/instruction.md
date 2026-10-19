`parse_assignment(line)` in `src/lib.rs` parses one `key=value` assignment
from an index file. It must trim surrounding whitespace, lowercase the key,
preserve the trimmed value, reject lines without `=`, reject an empty key, and
return an owned `Entry` that remains usable after the input buffer is dropped.

Fix `src/lib.rs`. The crate has no dependencies and is built with
`cargo test --offline`.

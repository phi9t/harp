`OrderId` has one canonical wire/text form: `ord-` followed by the numeric ID
with at least four decimal digits, left-padded with zeroes when needed. IDs
larger than four digits must not be truncated.

`Display`, `serialize_order_id`, and `parse_order_id` must all use that same
canonical form. Fix `src/lib.rs`. The crate has no dependencies and is built
with `cargo test --offline`.

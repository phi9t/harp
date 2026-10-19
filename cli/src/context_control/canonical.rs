use serde::Serialize;
use sha2::{Digest as _, Sha256};

use crate::AppError;

pub fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, AppError> {
    serde_json::to_vec(value).map_err(|error| {
        AppError::external(
            "release.serialization",
            format!("could not serialize canonical JSON: {error}"),
        )
    })
}

pub fn json_bytes_with_newline<T: Serialize>(value: &T) -> Result<Vec<u8>, AppError> {
    let mut bytes = json_bytes(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn sha256_id(bytes: &[u8]) -> String {
    format!("sha256-{}", sha256_hex(bytes))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use serde::Serialize;

    use super::*;

    #[derive(Serialize)]
    struct OrderedFixture {
        map: BTreeMap<String, u8>,
        set: BTreeSet<String>,
    }

    #[test]
    fn canonical_json_is_compact_ordered_and_optionally_newline_terminated() {
        let fixture = OrderedFixture {
            map: BTreeMap::from([("z".to_owned(), 2), ("a".to_owned(), 1)]),
            set: BTreeSet::from(["z".to_owned(), "a".to_owned()]),
        };

        let compact = json_bytes(&fixture).unwrap();
        assert_eq!(compact, br#"{"map":{"a":1,"z":2},"set":["a","z"]}"#);

        let with_newline = json_bytes_with_newline(&fixture).unwrap();
        assert_eq!(with_newline, [compact, b"\n".to_vec()].concat());
    }

    #[test]
    fn sha256_helpers_have_stable_lowercase_wire_forms() {
        assert_eq!(
            sha256_hex(b"harp"),
            "c7fa84b7672cc02e760f04c8d92c603bfe63d76632a817da2675f9d3cb7a198a"
        );
        assert_eq!(
            sha256_id(b"harp"),
            "sha256-c7fa84b7672cc02e760f04c8d92c603bfe63d76632a817da2675f9d3cb7a198a"
        );
    }
}

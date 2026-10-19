use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{valid_sha256, validate_bounded_string, ContractResult};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ArtifactRef {
    #[schemars(
        length(min = 82, max = 82),
        regex(pattern = r"^artifact://sha256/[0-9a-f]{64}$")
    )]
    pub uri: String,
    #[schemars(length(min = 64, max = 64), regex(pattern = r"^[0-9a-f]{64}$"))]
    pub sha256: String,
    #[schemars(length(min = 1, max = 256))]
    pub media_type: String,
    pub size_bytes: u64,
    #[schemars(length(min = 1, max = 256))]
    pub logical_schema: Option<String>,
    #[schemars(length(max = 256), inner(length(min = 1, max = 1024)))]
    pub permitted_ranges: Option<Vec<String>>,
}

impl ArtifactRef {
    pub fn sha256(
        digest: impl Into<String>,
        media_type: impl Into<String>,
        size_bytes: u64,
    ) -> ContractResult<Self> {
        let digest = digest.into();
        if !valid_sha256(&digest) {
            return Err(crate::invalid(
                "sha256",
                "must contain exactly 64 lowercase hexadecimal characters",
            ));
        }

        let artifact = Self {
            uri: format!("artifact://sha256/{digest}"),
            sha256: digest,
            media_type: media_type.into(),
            size_bytes,
            logical_schema: None,
            permitted_ranges: None,
        };
        artifact.validate()?;
        Ok(artifact)
    }

    pub fn validate(&self) -> ContractResult<()> {
        if !valid_sha256(&self.sha256) {
            return Err(crate::invalid(
                "sha256",
                "must contain exactly 64 lowercase hexadecimal characters",
            ));
        }
        if self.uri != format!("artifact://sha256/{}", self.sha256) {
            return Err(crate::invalid(
                "uri",
                "must match the artifact SHA-256 digest",
            ));
        }
        validate_bounded_string("mediaType", &self.media_type, 256, false)?;
        if let Some(logical_schema) = &self.logical_schema {
            validate_bounded_string("logicalSchema", logical_schema, 256, false)?;
        }
        if let Some(ranges) = &self.permitted_ranges {
            if ranges.len() > 256 {
                return Err(crate::invalid(
                    "permittedRanges",
                    "must contain at most 256 entries",
                ));
            }
            for range in ranges {
                validate_bounded_string("permittedRanges", range, 1024, false)?;
            }
        }
        Ok(())
    }
}

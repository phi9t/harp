use std::fmt;
use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::{Uuid, Version};

use crate::{invalid, ContractResult};

macro_rules! uuid_id {
    ($name:ident, $field:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, JsonSchema)]
        #[serde(transparent)]
        pub struct $name(#[schemars(extend("format" = "uuid"))] String);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7().to_string())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = crate::ContractError;

            fn from_str(value: &str) -> ContractResult<Self> {
                let uuid = Uuid::parse_str(value)
                    .map_err(|_| invalid($field, "must be a canonical UUIDv7 string"))?;
                if uuid.get_version() != Some(Version::SortRand)
                    || uuid.hyphenated().to_string() != value
                {
                    return Err(invalid($field, "must be a canonical UUIDv7 string"));
                }
                Ok(Self(value.to_owned()))
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                value.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

uuid_id!(RunId, "runId");
uuid_id!(AttemptId, "attemptId");
uuid_id!(OperationId, "operationId");

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct TaskId(
    #[schemars(
        length(min = 1, max = 128),
        regex(pattern = r"^(?!\.{1,2}$)[A-Za-z0-9_.-]{1,128}$")
    )]
    String,
);

impl fmt::Display for TaskId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for TaskId {
    type Err = crate::ContractError;

    fn from_str(value: &str) -> ContractResult<Self> {
        if value.is_empty() {
            return Err(invalid("taskId", "must not be empty"));
        }
        if value.len() > 128 {
            return Err(invalid("taskId", "must be at most 128 bytes"));
        }
        if value.trim() != value {
            return Err(invalid(
                "taskId",
                "must not contain leading or trailing whitespace",
            ));
        }
        if matches!(value, "." | "..") {
            return Err(invalid("taskId", "must not be '.' or '..'"));
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        {
            return Err(invalid(
                "taskId",
                "may contain only ASCII letters, numbers, '_', '-', and '.'",
            ));
        }
        Ok(Self(value.to_owned()))
    }
}

impl<'de> Deserialize<'de> for TaskId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

macro_rules! opaque_id {
    ($name:ident, $field:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, JsonSchema)]
        #[serde(transparent)]
        pub struct $name(#[schemars(length(min = 1, max = 256))] String);

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = crate::ContractError;

            fn from_str(value: &str) -> ContractResult<Self> {
                if value.is_empty() {
                    return Err(invalid($field, "must not be empty"));
                }
                if value.len() > 256 {
                    return Err(invalid($field, "must be at most 256 bytes"));
                }
                if value.chars().any(char::is_control) {
                    return Err(invalid($field, "must not contain control characters"));
                }
                Ok(Self(value.to_owned()))
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                value.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

opaque_id!(ThreadId, "threadId");
opaque_id!(TurnId, "turnId");
opaque_id!(ExternalSessionId, "externalSessionId");

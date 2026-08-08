use serde::Deserialize;

use crate::error::AppError;

pub fn parse_strict_bounded(
    bytes: &[u8],
    max_bytes: usize,
    size_code: &'static str,
    parse_code: &'static str,
    label: &str,
) -> Result<serde_json::Value, AppError> {
    if bytes.len() > max_bytes {
        return Err(AppError::invalid_input(
            size_code,
            format!("{label} exceeds {max_bytes} bytes"),
        ));
    }
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = serde_json::Value::deserialize(&mut deserializer).map_err(|error| {
        AppError::invalid_input(parse_code, format!("invalid {label}: {error}"))
    })?;
    deserializer.end().map_err(|error| {
        AppError::invalid_input(parse_code, format!("invalid {label}: {error}"))
    })?;
    Ok(value)
}

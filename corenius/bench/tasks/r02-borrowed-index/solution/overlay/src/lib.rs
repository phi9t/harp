#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    pub key: String,
    pub value: String,
}

pub fn parse_assignment(line: &str) -> Result<Entry, String> {
    let trimmed = line.trim();
    let (key, value) = trimmed
        .split_once('=')
        .ok_or_else(|| "missing '=' separator".to_string())?;
    let key = key.trim().to_ascii_lowercase();
    if key.is_empty() {
        return Err("empty key".to_string());
    }
    Ok(Entry {
        key,
        value: value.trim().to_string(),
    })
}

#[derive(Debug, PartialEq, Eq)]
pub struct Entry<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

pub fn parse_assignment(line: &str) -> Result<Entry<'_>, String> {
    let trimmed = line.trim();
    let (key, value) = trimmed
        .split_once('=')
        .ok_or_else(|| "missing '=' separator".to_string())?;
    let key = key.trim().to_ascii_lowercase();
    if key.is_empty() {
        return Err("empty key".to_string());
    }
    let value = value.trim();
    Ok(Entry {
        key: key.as_str(),
        value,
    })
}

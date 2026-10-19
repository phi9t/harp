#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub id: u32,
    pub payload: String,
}

#[cfg(feature = "json")]
pub fn encode_packet(packet: &Packet) -> String {
    format!(
        "{{\"id\":{},\"payload\":\"{}\"}}",
        packet.id,
        escape_json(&packet.payload)
    )
}

#[cfg(not(feature = "json"))]
pub fn encode_packet(packet: &Packet) -> String {
    format!("{}|{}", packet.id, packet.payload)
}

#[cfg(feature = "json")]
fn escape_json(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            _ => escaped.push(ch),
        }
    }
    escaped
}

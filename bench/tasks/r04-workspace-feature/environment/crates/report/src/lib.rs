use codec::Packet;

pub fn render_report(id: u32, payload: &str) -> String {
    codec::encode_packet(&Packet {
        id,
        payload: payload.to_string(),
    })
}

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderId(u32);

impl OrderId {
    pub fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub fn raw(self) -> u32 {
        self.0
    }
}

impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ord-{:04}", self.0)
    }
}

pub fn serialize_order_id(id: OrderId) -> String {
    format!("ORDER{}", id.raw())
}

pub fn parse_order_id(text: &str) -> Option<OrderId> {
    let raw = text.strip_prefix("ord-")?;
    raw.parse::<u32>().ok().map(OrderId::new)
}

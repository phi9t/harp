use r03_trait_format::{parse_order_id, serialize_order_id, OrderId};

#[test]
fn display_and_serializer_use_the_same_canonical_form() {
    let id = OrderId::new(42);

    assert_eq!(id.to_string(), "ord-0042");
    assert_eq!(serialize_order_id(id), "ord-0042");
}

#[test]
fn serialized_id_round_trips_through_parser() {
    let id = OrderId::new(7);

    assert_eq!(parse_order_id(&serialize_order_id(id)), Some(id));
}

#[test]
fn large_ids_are_not_truncated() {
    let id = OrderId::new(12_345);

    assert_eq!(id.to_string(), "ord-12345");
    assert_eq!(serialize_order_id(id), "ord-12345");
}

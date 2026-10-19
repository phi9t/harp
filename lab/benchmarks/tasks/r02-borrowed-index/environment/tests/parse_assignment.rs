use r02_borrowed_index::parse_assignment;

#[test]
fn parsed_entry_survives_after_input_buffer_is_dropped() {
    let entry = {
        let line = String::from("  Mode = FAST  ");
        parse_assignment(&line).expect("assignment should parse")
    };

    assert_eq!(entry.key, "mode");
    assert_eq!(entry.value, "FAST");
}

#[test]
fn rejects_malformed_assignments() {
    assert!(parse_assignment("missing separator").unwrap_err().contains("="));
    assert!(parse_assignment(" = value").unwrap_err().contains("key"));
}

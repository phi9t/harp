use r01_countdown_division::apply_div;

#[test]
fn accepts_exact_positive_division() {
    assert_eq!(apply_div(84, 7), Some(12));
}

#[test]
fn rejects_non_exact_or_non_positive_intermediates() {
    assert_eq!(apply_div(7, 2), None);
    assert_eq!(apply_div(-8, 2), None);
    assert_eq!(apply_div(8, 0), None);
    assert_eq!(apply_div(0, 5), None);
}

pub fn apply_div(lhs: i64, rhs: i64) -> Option<i64> {
    if rhs == 0 {
        return None;
    }
    Some(lhs / rhs)
}

#[cfg(test)]
mod tests {
    use super::apply_div;

    #[test]
    fn accepts_exact_positive_division() {
        assert_eq!(apply_div(84, 7), Some(12));
    }

    #[test]
    fn rejects_non_exact_or_non_positive_intermediates() {
        assert_eq!(apply_div(7, 2), None);
        assert_eq!(apply_div(-8, 2), None);
        assert_eq!(apply_div(8, 0), None);
    }
}

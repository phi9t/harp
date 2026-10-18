pub fn apply_div(lhs: i64, rhs: i64) -> Option<i64> {
    if rhs == 0 || lhs <= 0 || rhs <= 0 || lhs % rhs != 0 {
        return None;
    }
    Some(lhs / rhs)
}

pub fn apply_div(lhs: i64, rhs: i64) -> Option<i64> {
    if rhs == 0 {
        return None;
    }
    Some(lhs / rhs)
}

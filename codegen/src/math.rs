pub fn align_up(x: u64, y: u64) -> u64 {
    if x == 0 {
        0
    } else {
        (1 + (x - 1) / y) * y
    }
}

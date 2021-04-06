pub fn wrap(n: f32, max: usize) -> f32 {
    if n < 0.0 {
        n + max as f32
    } else if n >= max as f32 {
        n - max as f32
    } else {
        n
    }
}
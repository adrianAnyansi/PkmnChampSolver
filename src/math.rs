/// Basic functions

/// Multiplies an integer by a float and floors the result to the nearest integer.
/// Used to calculate many stat modifications and damage rolls
pub fn mult_and_round(a: i32, b: f64) -> i32 {
    (a as f64 * b).round() as i32
}

pub fn div_and_floor(a: i32, b: i32) -> i32 {
    (a as f64 / b as f64).round() as i32
}
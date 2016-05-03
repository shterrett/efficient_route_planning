#[cfg(test)]
pub fn floats_nearly_eq(float_1: f64, float_2: f64) -> bool {
    (float_1 - float_2).abs() < 0.0001
}

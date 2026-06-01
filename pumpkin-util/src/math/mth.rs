use std::sync::LazyLock;

const SIN_SCALE: f64 = 10430.378350470453; // 65536 / 2PI
const SIN_MASK: i64 = 65535;

static SIN: LazyLock<[f32; 65536]> =
    LazyLock::new(|| std::array::from_fn(|i| (i as f64 / SIN_SCALE).sin() as f32));

#[must_use]
pub fn sin(value: f64) -> f32 {
    SIN[((value * SIN_SCALE) as i64 & SIN_MASK) as usize]
}

#[must_use]
pub fn cos(value: f64) -> f32 {
    SIN[((value * SIN_SCALE + 16384.0) as i64 & SIN_MASK) as usize]
}

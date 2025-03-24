use num_traits::{Float, One, PrimInt, Zero};

pub mod boundingbox;
pub mod experience;
pub mod float_provider;
pub mod int_provider;
pub mod position;
pub mod vector2;
pub mod vector3;
pub mod vertical_surface_type;
pub mod voxel_shape;

pub fn wrap_degrees(degrees: f32) -> f32 {
    let mut var1 = degrees % 360.0;
    if var1 >= 180.0 {
        var1 -= 360.0;
    }

    if var1 < -180.0 {
        var1 += 360.0;
    }

    var1
}

pub fn squared_magnitude(a: f64, b: f64, c: f64) -> f64 {
    c.mul_add(c, a.mul_add(a, b * b))
}

pub fn magnitude(a: f64, b: f64, c: f64) -> f64 {
    squared_magnitude(a, b, c).sqrt()
}

/// Converts a world coordinate to the corresponding chunk-section coordinate.
// TODO: This probably shouldn't be placed here
pub const fn get_section_cord(coord: i32) -> i32 {
    coord >> 4
}

const MULTIPLY_DE_BRUIJN_BIT_POSITION: [u8; 32] = [
    0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7, 26,
    12, 18, 6, 11, 5, 10, 9,
];

/// Maximum return value: 31
pub const fn ceil_log2(value: u32) -> u8 {
    let value = if value.is_power_of_two() {
        value
    } else {
        smallest_encompassing_power_of_two(value)
    };

    MULTIPLY_DE_BRUIJN_BIT_POSITION[(((value as usize) * 125613361) >> 27) & 31]
}

/// Maximum return value: 30
pub const fn floor_log2(value: u32) -> u8 {
    ceil_log2(value) - if value.is_power_of_two() { 0 } else { 1 }
}

pub const fn smallest_encompassing_power_of_two(value: u32) -> u32 {
    let mut i = value - 1;
    i |= i >> 1;
    i |= i >> 2;
    i |= i >> 4;
    i |= i >> 8;
    i |= i >> 16;
    i + 1
}

#[inline]
pub fn floor_div<T>(x: T, y: T) -> T
where
    T: PrimInt + Zero + One,
{
    let div = x / y;
    if (x ^ y) < T::zero() && div * y != x {
        div - T::one()
    } else {
        div
    }
}

#[inline]
pub fn square<T>(n: T) -> T
where
    T: Float,
{
    n * n
}

#[inline]
pub fn floor_mod<T>(x: T, y: T) -> T
where
    T: PrimInt + Zero,
{
    let rem = x % y;
    if (x ^ y) < T::zero() && rem != T::zero() {
        rem + y
    } else {
        rem
    }
}

#[inline]
pub fn map<T>(value: T, old_start: T, old_end: T, new_start: T, new_end: T) -> T
where
    T: Float,
{
    lerp(lerp_progress(value, old_start, old_end), new_start, new_end)
}

#[inline]
pub fn lerp<T>(delta: T, start: T, end: T) -> T
where
    T: Float,
{
    start + delta * (end - start)
}

#[inline]
pub fn lerp_progress<T>(value: T, start: T, end: T) -> T
where
    T: Float,
{
    (value - start) / (end - start)
}

pub fn clamped_lerp(start: f64, end: f64, delta: f64) -> f64 {
    if delta < 0f64 {
        start
    } else if delta > 1f64 {
        end
    } else {
        lerp(delta, start, end)
    }
}

#[inline]
pub fn clamped_map(value: f64, old_start: f64, old_end: f64, new_start: f64, new_end: f64) -> f64 {
    clamped_lerp(new_start, new_end, lerp_progress(value, old_start, old_end))
}

pub fn lerp2(delta_x: f64, delta_y: f64, x0y0: f64, x1y0: f64, x0y1: f64, x1y1: f64) -> f64 {
    lerp(
        delta_y,
        lerp(delta_x, x0y0, x1y0),
        lerp(delta_x, x0y1, x1y1),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn lerp3(
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    x0y0z0: f64,
    x1y0z0: f64,
    x0y1z0: f64,
    x1y1z0: f64,
    x0y0z1: f64,
    x1y0z1: f64,
    x0y1z1: f64,
    x1y1z1: f64,
) -> f64 {
    lerp(
        delta_z,
        lerp2(delta_x, delta_y, x0y0z0, x1y0z0, x0y1z0, x1y1z0),
        lerp2(delta_x, delta_y, x0y0z1, x1y0z1, x0y1z1, x1y1z1),
    )
}

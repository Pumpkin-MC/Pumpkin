use num_traits::{Float, One, PrimInt, Zero};
use std::sync::LazyLock;

pub mod atomic_f32;
pub mod bit_storage;
pub mod block_box;
pub mod boundingbox;
pub mod bounds;
pub mod euler_angle;
pub mod experience;
pub mod float_provider;
pub mod int_provider;
pub mod pool;
pub mod position;
pub mod vector2;
pub mod vector3;
pub mod vertical_surface_type;

const SIN_SCALE: f64 = 10430.378350470453;
const SIN_MASK: i64 = 65535;

static SIN: LazyLock<[f32; 65536]> =
    LazyLock::new(|| std::array::from_fn(|i| (f64::from(i as u32) / SIN_SCALE).sin() as f32));

#[must_use]
pub fn sin(value: f32) -> f32 {
    SIN[((value as f64 * SIN_SCALE) as i64 & SIN_MASK) as usize]
}

#[must_use]
pub fn cos(value: f32) -> f32 {
    SIN[(((value as f64 * SIN_SCALE) + 16384.0) as i64 & SIN_MASK) as usize]
}

#[must_use]
pub fn wrap_degrees(degrees: f32) -> f32 {
    let mut normalized_angle = degrees % 360.0;
    if normalized_angle >= 180.0 {
        normalized_angle -= 360.0;
    }
    if normalized_angle < -180.0 {
        normalized_angle += 360.0;
    }
    normalized_angle
}

#[must_use]
pub fn wrap_degrees_f64(degrees: f64) -> f64 {
    let mut normalized_angle = degrees % 360.0;
    if normalized_angle >= 180.0 {
        normalized_angle -= 360.0;
    }
    if normalized_angle < -180.0 {
        normalized_angle += 360.0;
    }
    normalized_angle
}

#[must_use]
pub const fn wrap_degrees_i32(degrees: i32) -> i32 {
    let mut normalized_angle = degrees % 360;
    if normalized_angle >= 180 {
        normalized_angle -= 360;
    }
    if normalized_angle < -180 {
        normalized_angle += 360;
    }
    normalized_angle
}

#[must_use]
pub fn wrap_degrees_90(angle: f32) -> f32 {
    let mut normalized_angle = angle % 90.0;
    if normalized_angle >= 45.0 {
        normalized_angle -= 90.0;
    }
    if normalized_angle < -45.0 {
        normalized_angle += 90.0;
    }
    normalized_angle
}

#[must_use]
pub fn clamp_angle(value: f32, mean: f32, delta: f32) -> f32 {
    let i = subtract_angles(value, mean);
    let j = i.clamp(-delta, delta);
    mean - j
}

#[must_use]
pub fn subtract_angles(start: f32, end: f32) -> f32 {
    wrap_degrees(end - start)
}

#[must_use]
pub fn degrees_difference(from_angle: f32, to_angle: f32) -> f32 {
    wrap_degrees(to_angle - from_angle)
}

#[must_use]
pub fn degrees_difference_abs(angle_a: f32, angle_b: f32) -> f32 {
    degrees_difference(angle_a, angle_b).abs()
}

#[must_use]
pub fn rotate_if_necessary(base_angle: f32, target_angle: f32, max_angle_diff: f32) -> f32 {
    let delta_angle = degrees_difference(base_angle, target_angle);
    let delta_angle_clamped = delta_angle.clamp(-max_angle_diff, max_angle_diff);
    target_angle - delta_angle_clamped
}

#[must_use]
pub fn approach(current: f32, target: f32, increment: f32) -> f32 {
    let increment = increment.abs();
    if current < target {
        (current + increment).clamp(current, target)
    } else {
        (current - increment).clamp(target, current)
    }
}

#[must_use]
pub fn approach_degrees(current: f32, target: f32, increment: f32) -> f32 {
    let difference = degrees_difference(current, target);
    approach(current, current + difference, increment)
}

#[must_use]
pub const fn squared_magnitude(a: f64, b: f64, c: f64) -> f64 {
    c.mul_add(c, a.mul_add(a, b * b))
}

#[inline]
#[must_use]
pub fn fast_inv_sqrt(value: f64) -> f64 {
    let half = 0.5 * value;
    let bits = value.to_bits();
    let approx_bits = 6910469410427058090u64.wrapping_sub(bits >> 1);
    let approx = f64::from_bits(approx_bits);
    approx * (1.5 - half * approx * approx)
}

#[inline]
#[must_use]
pub fn fast_inv_cube_root(x: f32) -> f32 {
    let mut i = x.to_bits() as i32;
    i = 1419967116i32.wrapping_sub(i / 3);
    let mut y = f32::from_bits(i as u32);
    y = 0.6666667 * y + 1.0 / (3.0 * y * y * x);
    0.6666667 * y + 1.0 / (3.0 * y * y * x)
}

#[must_use]
pub fn magnitude(a: f64, b: f64, c: f64) -> f64 {
    squared_magnitude(a, b, c).sqrt()
}

#[must_use]
pub fn length_squared_2d(x: f64, y: f64) -> f64 {
    x * x + y * y
}

#[must_use]
pub fn length_squared_2d_f32(x: f32, y: f32) -> f32 {
    x * x + y * y
}

#[must_use]
pub fn length_2d(x: f64, y: f64) -> f64 {
    x.hypot(y)
}

#[must_use]
pub fn length_2d_f32(x: f32, y: f32) -> f32 {
    x.hypot(y)
}

#[must_use]
pub fn length_squared_3d(x: f64, y: f64, z: f64) -> f64 {
    x * x + y * y + z * z
}

#[must_use]
pub fn length_squared_3d_f32(x: f32, y: f32, z: f32) -> f32 {
    x * x + y * y + z * z
}

#[must_use]
pub fn length_3d(x: f64, y: f64, z: f64) -> f64 {
    (x * x + y * y + z * z).sqrt()
}

#[must_use]
pub fn length_3d_f32(x: f32, y: f32, z: f32) -> f32 {
    (x * x + y * y + z * z).sqrt()
}

#[must_use]
pub const fn get_section_cord(coord: i32) -> i32 {
    coord >> 4
}

const MULTIPLY_DE_BRUIJN_BIT_POSITION: [u8; 32] = [
    0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7, 26,
    12, 18, 6, 11, 5, 10, 9,
];

#[must_use]
pub const fn ceil_log2(value: u32) -> u8 {
    let value = if value.is_power_of_two() {
        value
    } else {
        smallest_encompassing_power_of_two(value)
    };

    MULTIPLY_DE_BRUIJN_BIT_POSITION[((((value as u64) * 125613361) >> 27) & 31) as usize]
}

#[must_use]
pub const fn floor_log2(value: u32) -> u8 {
    ceil_log2(value) - if value.is_power_of_two() { 0 } else { 1 }
}

#[must_use]
pub const fn smallest_encompassing_power_of_two(value: u32) -> u32 {
    value.next_power_of_two()
}

#[inline]
pub fn floor_div<T: PrimInt + Zero + One>(x: T, y: T) -> T {
    let div = x / y;
    let rem = x % y;

    if (x ^ y) < T::zero() && rem != T::zero() {
        div - T::one()
    } else {
        div
    }
}

#[inline]
#[must_use]
pub const fn square_f64(n: f64) -> f64 {
    n * n
}

#[inline]
#[must_use]
pub const fn square_f32(n: f32) -> f32 {
    n * n
}

#[inline]
pub fn floor_mod<T: PrimInt + Zero>(x: T, y: T) -> T {
    let rem = x % y;
    if (x ^ y) < T::zero() && rem != T::zero() {
        rem + y
    } else {
        rem
    }
}

#[inline]
pub fn map<T: Float>(value: T, old_start: T, old_end: T, new_start: T, new_end: T) -> T {
    lerp(lerp_progress(value, old_start, old_end), new_start, new_end)
}

#[inline]
pub fn lerp<T: Float>(delta: T, start: T, end: T) -> T {
    start + delta * (end - start)
}

#[inline]
pub fn lerp_progress<T: Float>(value: T, start: T, end: T) -> T {
    (value - start) / (end - start)
}

#[inline]
pub fn inverse_lerp<T: Float>(value: T, min: T, max: T) -> T {
    (value - min) / (max - min)
}

#[must_use]
pub fn clamped_lerp<T: Float>(start: T, end: T, delta: T) -> T {
    if delta < T::zero() {
        start
    } else if delta > T::one() {
        end
    } else {
        lerp(delta, start, end)
    }
}

#[inline]
#[must_use]
pub fn clamped_map<T: Float>(value: T, old_start: T, old_end: T, new_start: T, new_end: T) -> T {
    clamped_lerp(new_start, new_end, lerp_progress(value, old_start, old_end))
}

#[must_use]
pub fn lerp2<T: Float>(delta_x: T, delta_y: T, x0y0: T, x1y0: T, x0y1: T, x1y1: T) -> T {
    lerp(
        delta_y,
        lerp(delta_x, x0y0, x1y0),
        lerp(delta_x, x0y1, x1y1),
    )
}

#[expect(clippy::too_many_arguments)]
#[must_use]
pub fn lerp3<T: Float>(
    delta_x: T,
    delta_y: T,
    delta_z: T,
    x0y0z0: T,
    x1y0z0: T,
    x0y1z0: T,
    x1y1z0: T,
    x0y0z1: T,
    x1y0z1: T,
    x0y1z1: T,
    x1y1z1: T,
) -> T {
    lerp(
        delta_z,
        lerp2(delta_x, delta_y, x0y0z0, x1y0z0, x0y1z0, x1y1z0),
        lerp2(delta_x, delta_y, x0y0z1, x1y0z1, x0y1z1, x1y1z1),
    )
}

#[must_use]
pub fn catmullrom(alpha: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    0.5 * (2.0 * p1
        + (p2 - p0) * alpha
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * alpha * alpha
        + (3.0 * p1 - p0 - 3.0 * p2 + p3) * alpha * alpha * alpha)
}

#[must_use]
pub fn smoothstep(x: f32) -> f32 {
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

#[must_use]
pub fn smoothstep_derivative(x: f32) -> f32 {
    30.0 * x * x * (x - 1.0) * (x - 1.0)
}

#[must_use]
pub fn triangle_wave(index: f32, period: f32) -> f32 {
    ((index % period - period * 0.5).abs() - period * 0.25) / (period * 0.25)
}

#[must_use]
pub fn rot_lerp(a: f32, from: f32, to: f32) -> f32 {
    from + a * wrap_degrees(to - from)
}

#[must_use]
pub fn rot_lerp_f64(a: f64, from: f64, to: f64) -> f64 {
    from + a * wrap_degrees_f64(to - from)
}

#[must_use]
pub fn rot_lerp_rad(a: f32, from: f32, to: f32) -> f32 {
    let mut diff = to - from;
    while diff < -std::f32::consts::PI {
        diff += std::f32::consts::TAU;
    }
    while diff >= std::f32::consts::PI {
        diff -= std::f32::consts::TAU;
    }
    from + a * diff
}

#[must_use]
pub fn positive_ceil_div(input: i32, divisor: i32) -> i32 {
    -floor_div(-input, divisor)
}

#[must_use]
pub fn positive_ceil_div_i64(input: i64, divisor: i64) -> i64 {
    -floor_div(-input, divisor)
}

#[must_use]
pub fn round_toward(input: i32, multiple: i32) -> i32 {
    positive_ceil_div(input, multiple) * multiple
}

#[must_use]
pub fn round_toward_i64(input: i64, multiple: i64) -> i64 {
    positive_ceil_div_i64(input, multiple) * multiple
}

#[must_use]
pub fn quantize(value: f64, quantize_resolution: i32) -> i32 {
    (value / quantize_resolution as f64).floor() as i32 * quantize_resolution
}

#[must_use]
pub const fn get_seed(x: i32, y: i32, z: i32) -> i64 {
    let mut seed =
        ((x as i64).wrapping_mul(3129871)) ^ ((z as i64).wrapping_mul(116129781)) ^ (y as i64);
    seed = seed
        .wrapping_mul(seed)
        .wrapping_mul(42317861)
        .wrapping_add(seed.wrapping_mul(11));
    seed >> 16
}

#[must_use]
pub fn polynomial_rolling_hash(signatures: &[Box<[u8]>]) -> u8 {
    let mut i: i32 = 1;

    for signature in signatures {
        i = i.wrapping_mul(31).wrapping_add(java_array_hash(signature));
    }

    let b = (i & 0xFF) as u8;
    if b == 0 { 1 } else { b }
}

fn java_array_hash(data: &[u8]) -> i32 {
    let mut hash: i32 = 1;
    for &byte in data {
        let signed_byte = byte as i32;
        hash = hash.wrapping_mul(31).wrapping_add(signed_byte);
    }
    hash
}

#[must_use]
pub fn java_string_hash(string: &str) -> i32 {
    let mut result = 0i32;
    for char_encoding in string.encode_utf16() {
        result = 31i32
            .wrapping_mul(result)
            .wrapping_add(char_encoding as i32);
    }
    result
}

macro_rules! vector_codec_impl {
    ($vector:ty, $number:literal, $($components:ident),+ ) => {
        impl<T> From<&$vector> for Vec<T> where T: Clone {
            fn from(value: &$vector) -> Self {
                vec![
                    $( value.$components.clone(), )+
                ]
            }
        }

        impl<T> FlatTryFrom<Vec<T>> for $vector {
            fn flat_try_from(value: Vec<T>) -> DataResult<Self> {
                validate_fixed_size(value, $number).flat_map(|v| {
                    if let Ok(arr) = v.try_into() {
                        let [ $( $components, )+ ]: [T; $number] = arr;
                        DataResult::new_success(Self { $( $components, )+ })
                    } else {
                        DataResult::new_error(format!("Expected {} elements", $number))
                    }
                })
            }
        }

        impl<T> Encode for $vector where T: Encode + Clone {
            fn encode<O: DynamicOps>(&self, ops: &'static O, prefix: O::Value) -> DataResult<O::Value> {
                Vec::<T>::from(self).encode(ops, prefix)
            }
        }

        impl<T> Decode for $vector where T: Decode {
            fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
                <Vec<T>>::decode(input, ops).flat_map(|(s, p)| Self::flat_try_from(s).map(|v| (v, p)))
            }
        }
    };
}
pub(crate) use vector_codec_impl;

#[test]
#[expect(clippy::unicode_not_nfc)]
fn java_hash() {
    let values = [
        ("", 0, 1),
        ("1", 49, 80),
        ("TEST", 2571410, 3494931),
        ("TEST1", 79713759, 108342910),
        ("TEST0123456789", 506557463, 2014109272),
        (
            " !\"#$%&'()*+,-./0123456789:\
            ;<=>?@ABCDEFGHIJKLMNOPQRST\
            UVWXYZ[\\]^_`abcdefghijklm\
            nopqrstuvwxyz{|}~¡¢£¤¥¦§¨©\
            ª«¬®¯°±²³´µ¶·¸¹º»¼½¾¿ÀÁÂÃÄ\
            ÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞ\
            ßàáâãäåæçèéêëìíîïðñòóôõö÷øùúûüýþ",
            -1992287231i32,
            -1606003975i32,
        ),
        ("求同存异", 847053876, 1709557670),
        (
            "אבְּרֵאשִׁ֖ית בָּרָ֣א אֱלֹהִ֑ים אֵ֥ת הַשָּׁמַ֖יִם וְאֵ֥ת הָאָֽרֶץ:",
            1372570871,
            -396640725i32,
        ),
        ("संस्कृत-", 1748614838, -187482695i32),
        ("minecraft:offset", -920384768i32, 432924929),
    ];

    for (string, value, _) in values {
        assert_eq!(java_string_hash(string), value);
    }

    for (string, _, value) in values {
        assert_eq!(java_array_hash(string.as_bytes()), value);
    }
}

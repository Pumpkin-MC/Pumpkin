use num_traits::{Float, One, PrimInt, Zero};

pub mod atomic_f32;
pub mod block_box;
pub mod boundingbox;
pub mod euler_angle;
pub mod experience;
pub mod float_provider;
pub mod int_provider;
pub mod pool;
pub mod position;
pub mod vector2;
pub mod vector3;
pub mod vertical_surface_type;

/// Wraps an angle in degrees to the range [-180, 180).
///
/// # Arguments
/// - `degrees` – The angle in degrees to wrap.
///
/// # Returns
/// The wrapped angle.
#[must_use]
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

/// Clamps an angle toward a mean within a delta.
///
/// # Arguments
/// - `value` – The angle to clamp.
/// - `mean` – The reference angle.
/// - `delta` – Maximum deviation from the mean.
///
/// # Returns
/// The clamped angle.
#[must_use]
pub fn clamp_angle(value: f32, mean: f32, delta: f32) -> f32 {
    let i = subtract_angles(value, mean);
    let j = i.clamp(-delta, delta);
    mean - j
}

/// Subtracts two angles, returning the difference wrapped to [-180, 180).
///
/// # Arguments
/// - `start` – Starting angle.
/// - `end` – Ending angle.
///
/// # Returns
/// The wrapped difference.
#[must_use]
pub fn subtract_angles(start: f32, end: f32) -> f32 {
    wrap_degrees(end - start)
}

/// Returns the squared magnitude of a 3D vector.
///
/// # Arguments
/// - `a` – X component.
/// - `b` – Y component.
/// - `c` – Z component.
///
/// # Returns
/// `a² + b² + c²`
#[must_use]
pub fn squared_magnitude(a: f64, b: f64, c: f64) -> f64 {
    c.mul_add(c, a.mul_add(a, b * b))
}

/// Returns the size (length) of a 3D vector.
///
/// # Arguments
/// - `a` – X component.
/// - `b` – Y component.
/// - `c` – Z component.
///
/// # Returns
/// The Euclidean length.
#[must_use]
pub fn magnitude(a: f64, b: f64, c: f64) -> f64 {
    squared_magnitude(a, b, c).sqrt()
}

/// Converts a world coordinate to a chunk-section coordinate.
///
/// # Arguments
/// - `coord` – The world coordinate.
///
/// # Returns
/// The corresponding chunk-section coordinate.
// TODO: This probably shouldn't be placed here
#[must_use]
pub const fn get_section_cord(coord: i32) -> i32 {
    coord >> 4
}

const MULTIPLY_DE_BRUIJN_BIT_POSITION: [u8; 32] = [
    0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7, 26,
    12, 18, 6, 11, 5, 10, 9,
];

/// Returns the smallest power-of-two exponent ≥ log2(value).
///
/// # Arguments
/// - `value` – The input integer.
///
/// # Returns
/// Maximum return value: 31
#[must_use]
pub const fn ceil_log2(value: u32) -> u8 {
    let value = if value.is_power_of_two() {
        value
    } else {
        smallest_encompassing_power_of_two(value)
    };

    MULTIPLY_DE_BRUIJN_BIT_POSITION[((((value as u64) * 125613361) >> 27) & 31) as usize]
}

/// Returns the largest power-of-two exponent ≤ log2(value).
///
/// # Arguments
/// - `value` – The input integer.
///
/// # Returns
/// Maximum return value: 30
#[must_use]
pub const fn floor_log2(value: u32) -> u8 {
    ceil_log2(value) - if value.is_power_of_two() { 0 } else { 1 }
}

/// Returns the smallest power of two greater than or equal to the input.
///
/// # Arguments
/// - `value` – The input integer.
///
/// # Returns
/// The next power of two.
#[must_use]
pub const fn smallest_encompassing_power_of_two(value: u32) -> u32 {
    value.next_power_of_two()
}

/// Computes floor division of integers.
///
/// # Arguments
/// - `x` – Dividend.
/// - `y` – Divisor.
///
/// # Returns
/// The floor of x / y.
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

/// Squares a floating-point value.
///
/// # Arguments
/// - `n` – The value to square.
///
/// # Returns
/// n²
#[inline]
pub fn square<T: Float>(n: T) -> T {
    n * n
}

/// Computes a floor modulo operation.
///
/// # Arguments
/// - `x` – Dividend.
/// - `y` – Divisor.
///
/// # Returns
/// Result of x mod y.
#[inline]
pub fn floor_mod<T: PrimInt + Zero>(x: T, y: T) -> T {
    let rem = x % y;
    if (x ^ y) < T::zero() && rem != T::zero() {
        rem + y
    } else {
        rem
    }
}

/// Maps a value from one range to another.
///
/// # Arguments
/// - `value` – Value in old range.
/// - `old_start` – Start of old range.
/// - `old_end` – End of old range.
/// - `new_start` – Start of new range.
/// - `new_end` – End of new range.
///
/// # Returns
/// Value in new range.
#[inline]
pub fn map<T: Float>(value: T, old_start: T, old_end: T, new_start: T, new_end: T) -> T {
    lerp(lerp_progress(value, old_start, old_end), new_start, new_end)
}

/// Performs linear interpolation.
///
/// # Arguments
/// - `delta` – Progress factor [0,1].
/// - `start` – Start value.
/// - `end` – End value.
///
/// # Returns
/// Interpolated value.
#[inline]
pub fn lerp<T: Float>(delta: T, start: T, end: T) -> T {
    start + delta * (end - start)
}

/// Returns the interpolation progress of a value in a range.
///
/// # Arguments
/// - `value` – Value to map.
/// - `start` – Start of range.
/// - `end` – End of range.
///
/// # Returns
/// Fractional progress of value between start and end.
#[inline]
pub fn lerp_progress<T: Float>(value: T, start: T, end: T) -> T {
    (value - start) / (end - start)
}

/// Performs a clamped linear interpolation between two values.
///
/// # Arguments
/// - `start` – Start value.
/// - `end` – End value.
/// - `delta` – Interpolation factor; clamped to [0,1].
///
/// # Returns
/// Interpolated value, clamped to the range [start, end].
#[must_use]
pub fn clamped_lerp(start: f64, end: f64, delta: f64) -> f64 {
    if delta < 0.0 {
        start
    } else if delta > 1.0 {
        end
    } else {
        lerp(delta, start, end)
    }
}

/// Maps a value from one range to another with clamping.
///
/// # Arguments
/// - `value` – Value in old range.
/// - `old_start` – Start of old range.
/// - `old_end` – End of old range.
/// - `new_start` – Start of new range.
/// - `new_end` – End of new range.
///
/// # Returns
/// Value in a new range, clamped to [`new_start`, `new_end`].
#[inline]
#[must_use]
pub fn clamped_map(value: f64, old_start: f64, old_end: f64, new_start: f64, new_end: f64) -> f64 {
    clamped_lerp(new_start, new_end, lerp_progress(value, old_start, old_end))
}

/// Performs bilinear interpolation.
///
/// # Arguments
/// - `delta_x` – Interpolation factor along X axis [0,1].
/// - `delta_y` – Interpolation factor along Y axis [0,1].
/// - `x0y0` – Value at (0,0).
/// - `x1y0` – Value at (1,0).
/// - `x0y1` – Value at (0,1).
/// - `x1y1` – Value at (1,1).
///
/// # Returns
/// Interpolated value.
#[must_use]
pub fn lerp2(delta_x: f64, delta_y: f64, x0y0: f64, x1y0: f64, x0y1: f64, x1y1: f64) -> f64 {
    lerp(
        delta_y,
        lerp(delta_x, x0y0, x1y0),
        lerp(delta_x, x0y1, x1y1),
    )
}

/// Performs trilinear interpolation.
///
/// # Arguments
/// - `delta_x` – Interpolation factor along X axis [0,1].
/// - `delta_y` – Interpolation factor along Y axis [0,1].
/// - `delta_z` – Interpolation factor along Z axis [0,1].
/// - `x0y0z0` – Value at (0,0,0).
/// - `x1y0z0` – Value at (1,0,0).
/// - `x0y1z0` – Value at (0,1,0).
/// - `x1y1z0` – Value at (1,1,0).
/// - `x0y0z1` – Value at (0,0,1).
/// - `x1y0z1` – Value at (1,0,1).
/// - `x0y1z1` – Value at (0,1,1).
/// - `x1y1z1` – Value at (1,1,1).
///
/// # Returns
/// Interpolated value.
#[expect(clippy::too_many_arguments)]
#[must_use]
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

/// Calculates a polynomial rolling hash for an array of byte slices.
///
/// # Arguments
/// - `signatures` – Slice of byte arrays to hash.
///
/// # Returns
/// A single-byte hash (never 0).
#[must_use]
pub fn polynomial_rolling_hash(signatures: &[Box<[u8]>]) -> u8 {
    let mut i: i32 = 1;

    for signature in signatures {
        i = i.wrapping_mul(31).wrapping_add(java_array_hash(signature)); // NOTE: Wrap to prevent multiplication overflow.
    }

    let b = (i & 0xFF) as u8; // NOTE: Take the least significant byte.
    if b == 0 { 1 } else { b } // NOTE: Ensure the checksum is never zero.
}

/// Computes a Java-style hash code for a byte array.
///
/// # Arguments
/// - `data` – Byte array to hash.
///
/// # Returns
/// Hash code as i32.
fn java_array_hash(data: &[u8]) -> i32 {
    let mut hash: i32 = 1;
    for &byte in data {
        let signed_byte = byte as i32;
        hash = hash.wrapping_mul(31).wrapping_add(signed_byte);
    }
    hash
}

/// Computes a Java-style hash code for a string.
///
/// # Arguments
/// - `string` – String to hash.
///
/// # Returns
/// Hash code as i32.
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

/// Tests the Java-style string and array hash implementations.
///
/// This verifies that `java_string_hash` and `java_array_hash` produce the expected
/// results for various strings, including ASCII, Unicode, and edge cases.
///
/// # Panics
/// - If any computed hash does not match the expected value.
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
        // NOTE: This might look weird because Hebrew is text is right to left.
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

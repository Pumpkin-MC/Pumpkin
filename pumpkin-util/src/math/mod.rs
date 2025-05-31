use num_traits::{Float, One, PrimInt, Zero};

pub mod boundingbox;
pub mod experience;
pub mod float_provider;
pub mod int_provider;
pub mod pool;
pub mod position;
pub mod vector2;
pub mod vector3;
pub mod vertical_surface_type;

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

    MULTIPLY_DE_BRUIJN_BIT_POSITION[((((value as u64) * 125613361) >> 27) & 31) as usize]
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

/// Calculates a Polynomial Rolling Hash
/// Mojang's checksum algorithm for previous messages
pub fn polynomial_rolling_hash(signatures: &[Box<[u8]>]) -> u8 {
    let mut i: i32 = 1;

    for signature in signatures.iter() {
        i = i.wrapping_mul(31).wrapping_add(java_array_hash(signature)); // Wrap to prevent multiplication overflow
    }

    let b = (i & 0xFF) as u8; // Take the least significant byte.
    if b == 0 { 1 } else { b } // Ensure the checksum is never zero.
}

/// Arrays.hashCode() and String.hashCode() have similar but different implementations.
fn java_array_hash(data: &[u8]) -> i32 {
    let mut hash: i32 = 1;
    for &byte in data {
        let signed_byte = byte as i32;
        hash = hash.wrapping_mul(31).wrapping_add(signed_byte);
    }
    hash
}

pub fn java_string_hash(string: &str) -> i32 {
    let mut result = 0i32;
    for char_encoding in string.encode_utf16() {
        result = 31i32
            .wrapping_mul(result)
            .wrapping_add(char_encoding as i32);
    }
    result
}

#[test]
fn test_java_hash() {
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
        // This might look weird because hebrew is text is right to left
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

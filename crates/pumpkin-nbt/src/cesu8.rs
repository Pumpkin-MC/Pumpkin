use std::borrow::Cow;

use crate::Error;

/// Converts a Java Modified UTF-8 (CESU-8) byte slice into a Rust UTF-8 string.
///
/// Fast-paths valid standard UTF-8 slices that do not contain null bytes (`\0`),
/// which avoids heap allocations and benefits from standard library SIMD validation.
///
/// # Errors
/// Returns [`Error::Cesu8DecodingError`] if the bytes are not valid Java CESU-8.
pub fn from_java_cesu8(bytes: &[u8]) -> Result<Cow<'_, str>, Error> {
    if let Ok(s) = std::str::from_utf8(bytes)
        && !bytes.contains(&0)
    {
        return Ok(Cow::Borrowed(s));
    }
    decode_java_cesu8(bytes).map(Cow::Owned)
}

/// Converts a Rust UTF-8 string into Java Modified UTF-8 (CESU-8) bytes.
///
/// Fast-paths strings without null bytes or 4-byte UTF-8 sequences (`b >= 0xF0`),
/// returning a borrowed slice of the existing string without allocating.
#[must_use]
pub fn to_java_cesu8(s: &str) -> Cow<'_, [u8]> {
    if s.as_bytes().iter().any(|&b| b == 0 || b >= 0xF0) {
        Cow::Owned(encode_java_cesu8(s))
    } else {
        Cow::Borrowed(s.as_bytes())
    }
}

fn decode_java_cesu8(bytes: &[u8]) -> Result<String, Error> {
    let mut out = String::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i];
        if b0 == 0 || b0 >= 0xF0 {
            return Err(Error::Cesu8DecodingError);
        }
        if b0 < 0x80 {
            out.push(b0 as char);
            i += 1;
        } else if (b0 & 0xE0) == 0xC0 {
            if i + 1 >= bytes.len() {
                return Err(Error::Cesu8DecodingError);
            }
            let b1 = bytes[i + 1];
            if (b1 & 0xC0) != 0x80 {
                return Err(Error::Cesu8DecodingError);
            }
            let cp = (u32::from(b0 & 0x1F) << 6) | u32::from(b1 & 0x3F);
            if cp == 0 {
                out.push('\0');
            } else if cp < 0x80 {
                return Err(Error::Cesu8DecodingError);
            } else {
                let c = char::from_u32(cp).ok_or(Error::Cesu8DecodingError)?;
                out.push(c);
            }
            i += 2;
        } else if (b0 & 0xF0) == 0xE0 {
            if i + 2 >= bytes.len() {
                return Err(Error::Cesu8DecodingError);
            }
            let b1 = bytes[i + 1];
            let b2 = bytes[i + 2];
            if (b1 & 0xC0) != 0x80 || (b2 & 0xC0) != 0x80 {
                return Err(Error::Cesu8DecodingError);
            }
            let cp =
                (u32::from(b0 & 0x0F) << 12) | (u32::from(b1 & 0x3F) << 6) | u32::from(b2 & 0x3F);
            if (0xD800..=0xDBFF).contains(&cp) {
                if i + 5 >= bytes.len() {
                    return Err(Error::Cesu8DecodingError);
                }
                let b3 = bytes[i + 3];
                let b4 = bytes[i + 4];
                let b5 = bytes[i + 5];
                if b3 != 0xED || (b4 & 0xC0) != 0x80 || (b5 & 0xC0) != 0x80 {
                    return Err(Error::Cesu8DecodingError);
                }
                let low = (u32::from(b3 & 0x0F) << 12)
                    | (u32::from(b4 & 0x3F) << 6)
                    | u32::from(b5 & 0x3F);
                if !(0xDC00..=0xDFFF).contains(&low) {
                    return Err(Error::Cesu8DecodingError);
                }
                let full_cp = 0x10000 + (((cp - 0xD800) << 10) | (low - 0xDC00));
                let c = char::from_u32(full_cp).ok_or(Error::Cesu8DecodingError)?;
                out.push(c);
                i += 6;
            } else if (0xDC00..=0xDFFF).contains(&cp) || cp < 0x800 {
                return Err(Error::Cesu8DecodingError);
            } else {
                let c = char::from_u32(cp).ok_or(Error::Cesu8DecodingError)?;
                out.push(c);
                i += 3;
            }
        } else {
            return Err(Error::Cesu8DecodingError);
        }
    }
    Ok(out)
}

fn encode_java_cesu8(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() + 4);
    for c in s.chars() {
        let cp = c as u32;
        if cp == 0 {
            out.push(0xC0);
            out.push(0x80);
        } else if cp < 0x80 {
            out.push(cp as u8);
        } else if cp < 0x800 {
            out.push((0xC0 | (cp >> 6)) as u8);
            out.push((0x80 | (cp & 0x3F)) as u8);
        } else if cp < 0x10000 {
            out.push((0xE0 | (cp >> 12)) as u8);
            out.push((0x80 | ((cp >> 6) & 0x3F)) as u8);
            out.push((0x80 | (cp & 0x3F)) as u8);
        } else {
            let offset = cp - 0x10000;
            let high = 0xD800 + (offset >> 10);
            let low = 0xDC00 + (offset & 0x3FF);
            out.push((0xE0 | (high >> 12)) as u8);
            out.push((0x80 | ((high >> 6) & 0x3F)) as u8);
            out.push((0x80 | (high & 0x3F)) as u8);
            out.push((0xE0 | (low >> 12)) as u8);
            out.push((0x80 | ((low >> 6) & 0x3F)) as u8);
            out.push((0x80 | (low & 0x3F)) as u8);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cesu8_fast_path_roundtrip() {
        // Standard ASCII
        let ascii = "minecraft:stone";
        let encoded = to_java_cesu8(ascii);
        assert!(matches!(encoded, Cow::Borrowed(_)));
        let decoded = from_java_cesu8(&encoded).unwrap();
        assert_eq!(decoded, ascii);

        // 2-byte UTF-8
        let utf8_2 = "café";
        let encoded = to_java_cesu8(utf8_2);
        assert!(matches!(encoded, Cow::Borrowed(_)));
        let decoded = from_java_cesu8(&encoded).unwrap();
        assert_eq!(decoded, utf8_2);

        // 3-byte UTF-8
        let utf8_3 = "中文测试";
        let encoded = to_java_cesu8(utf8_3);
        assert!(matches!(encoded, Cow::Borrowed(_)));
        let decoded = from_java_cesu8(&encoded).unwrap();
        assert_eq!(decoded, utf8_3);

        // 4-byte UTF-8 (emoji)
        let emoji = "Hello 😀 World";
        let encoded = to_java_cesu8(emoji);
        assert!(matches!(encoded, Cow::Owned(_)));
        let decoded = from_java_cesu8(&encoded).unwrap();
        assert_eq!(decoded, emoji);

        // Null byte in string
        let with_null = "Hello\0World";
        let encoded = to_java_cesu8(with_null);
        assert!(matches!(encoded, Cow::Owned(_)));
        let decoded = from_java_cesu8(&encoded).unwrap();
        assert_eq!(decoded, with_null);

        // Verify exact byte encoding of null and supplementary characters
        assert_eq!(to_java_cesu8("\0").as_ref(), &[0xC0, 0x80]);
        assert_eq!(
            to_java_cesu8("\u{10401}").as_ref(),
            &[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x81]
        );
        assert_eq!(
            from_java_cesu8(&[0xED, 0xA0, 0x81, 0xED, 0xB0, 0x81]).unwrap(),
            "\u{10401}"
        );
    }
}

//! Deserialization from Java Edition, unnamed network, and Bedrock NBT.

use std::borrow::Cow;
use std::io::{Cursor, Seek, SeekFrom};

use crate::{Error, io};
use io::Read;

/// Result type returned by NBT deserialization operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Byte source used by NBT read helpers.
///
/// Implementations may return borrowed strings and byte arrays when the
/// underlying storage permits it.
pub trait NbtDataSource<'a> {
    /// Reads one unsigned byte.
    fn read_u8(&mut self) -> Result<u8>;
    /// Fills `buf` with bytes from the source.
    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()>;
    /// Moves the current position by `offset` bytes.
    fn seek_relative(&mut self, offset: i64) -> Result<()>;
    /// Reads and decodes a string payload of `len` bytes.
    fn read_string(&mut self, len: usize) -> Result<Cow<'a, str>>;
    /// Reads a byte-array payload of `len` elements.
    fn read_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>>;
}

/// Adapts a [`Read`] and [`Seek`] stream into an [`NbtDataSource`].
pub struct NbtStreamReader<R>(
    /// Wrapped input stream.
    pub R,
);

impl<'a, R: Read + Seek> NbtDataSource<'a> for NbtStreamReader<R> {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.0.read_exact(&mut buf).map_err(Error::Incomplete)?;
        Ok(buf[0])
    }

    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        self.0.read_exact(buf).map_err(Error::Incomplete)
    }

    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        self.0
            .seek(SeekFrom::Current(offset))
            .map_err(Error::Incomplete)?;
        Ok(())
    }

    fn read_string(&mut self, len: usize) -> Result<Cow<'a, str>> {
        let mut buf = vec![0u8; len];
        self.0.read_exact(&mut buf).map_err(Error::Incomplete)?;
        let string = cesu8::from_java_cesu8(&buf).map_err(|_| Error::Cesu8DecodingError)?;
        Ok(Cow::Owned(string.into_owned()))
    }

    fn read_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>> {
        let mut buf = vec![0u8; len];
        self.0.read_exact(&mut buf).map_err(Error::Incomplete)?;
        let i8_buf: Vec<i8> = buf.into_iter().map(|b| b as i8).collect();
        Ok(Cow::Owned(i8_buf))
    }
}

impl<'a> NbtDataSource<'a> for Cursor<&'a [u8]> {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).map_err(Error::Incomplete)?;
        Ok(buf[0])
    }

    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        self.read_exact(buf).map_err(Error::Incomplete)
    }

    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        self.seek(SeekFrom::Current(offset))
            .map_err(Error::Incomplete)?;
        Ok(())
    }

    fn read_string(&mut self, len: usize) -> Result<Cow<'a, str>> {
        let pos = self.position() as usize;
        let data_len = self.get_ref().len();
        if pos + len > data_len {
            return Err(Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected EOF",
            )));
        }
        self.set_position((pos + len) as u64);
        let data = self.get_ref();
        let slice = &data[pos..pos + len];
        if let Ok(s) = std::str::from_utf8(slice) {
            Ok(Cow::Borrowed(s))
        } else {
            let string = cesu8::from_java_cesu8(slice).map_err(|_| Error::Cesu8DecodingError)?;
            Ok(string)
        }
    }

    fn read_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>> {
        let pos = self.position() as usize;
        let data_len = self.get_ref().len();
        if pos + len > data_len {
            return Err(Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected EOF",
            )));
        }
        self.set_position((pos + len) as u64);
        let data = self.get_ref();
        let slice = &data[pos..pos + len];
        // SAFETY: `slice` is a valid byte slice of length `len`. `u8` and `i8` have identical size, alignment (1 byte), and valid value representations.
        let i8_slice = unsafe { std::slice::from_raw_parts(slice.as_ptr().cast::<i8>(), len) };
        Ok(Cow::Borrowed(i8_slice))
    }
}

impl<'a, S: NbtDataSource<'a> + ?Sized> NbtDataSource<'a> for &mut S {
    fn read_u8(&mut self) -> Result<u8> {
        (**self).read_u8()
    }
    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        (**self).read_bytes(buf)
    }
    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        (**self).seek_relative(offset)
    }
    fn read_string(&mut self, len: usize) -> Result<Cow<'a, str>> {
        (**self).read_string(len)
    }
    fn read_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>> {
        (**self).read_byte_array(len)
    }
}

impl<'a> NbtDataSource<'a> for Cursor<Vec<u8>> {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf).map_err(Error::Incomplete)?;
        Ok(buf[0])
    }

    fn read_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        self.read_exact(buf).map_err(Error::Incomplete)
    }

    fn seek_relative(&mut self, offset: i64) -> Result<()> {
        self.seek(SeekFrom::Current(offset))
            .map_err(Error::Incomplete)?;
        Ok(())
    }

    fn read_string(&mut self, len: usize) -> Result<Cow<'a, str>> {
        let pos = self.position() as usize;
        let data_len = self.get_ref().len();
        if pos + len > data_len {
            return Err(Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected EOF",
            )));
        }
        self.set_position((pos + len) as u64);
        let data = self.get_ref();
        let slice = &data[pos..pos + len];
        let string = cesu8::from_java_cesu8(slice).map_err(|_| Error::Cesu8DecodingError)?;
        Ok(Cow::Owned(string.into_owned()))
    }

    fn read_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>> {
        let pos = self.position() as usize;
        let data_len = self.get_ref().len();
        if pos + len > data_len {
            return Err(Error::Incomplete(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected EOF",
            )));
        }
        self.set_position((pos + len) as u64);
        let data = self.get_ref();
        let slice = &data[pos..pos + len];
        // SAFETY: `slice` is a valid byte slice of length `len`. `u8` and `i8` have identical size, alignment (1 byte), and valid value representations.
        let i8_slice = unsafe { std::slice::from_raw_parts(slice.as_ptr().cast::<i8>(), len) };
        Ok(Cow::Owned(i8_slice.to_vec()))
    }
}

/// Format-specific primitive reader used by the NBT parser.
pub trait NbtReadHelper<'a> {
    /// Underlying byte source.
    type Reader: NbtDataSource<'a>;

    /// Returns the underlying byte source.
    fn reader(&mut self) -> &mut Self::Reader;

    /// Advances by `count` bytes.
    fn skip_bytes(&mut self, count: i64) -> Result<()> {
        self.reader().seek_relative(count)
    }
    /// Advances past an unsigned byte.
    fn skip_u8(&mut self) -> Result<()> {
        self.skip_bytes(1)
    }
    /// Advances past a signed byte.
    fn skip_i8(&mut self) -> Result<()> {
        self.skip_bytes(1)
    }
    /// Advances past a 16-bit signed integer.
    fn skip_i16(&mut self) -> Result<()> {
        self.skip_bytes(2)
    }
    /// Advances past a 32-bit signed integer.
    fn skip_i32(&mut self) -> Result<()> {
        self.skip_bytes(4)
    }
    /// Advances past a 64-bit signed integer.
    fn skip_i64(&mut self) -> Result<()> {
        self.skip_bytes(8)
    }
    /// Advances past a 32-bit floating-point number.
    fn skip_f32(&mut self) -> Result<()> {
        self.skip_bytes(4)
    }
    /// Advances past a 64-bit floating-point number.
    fn skip_f64(&mut self) -> Result<()> {
        self.skip_bytes(8)
    }
    /// Advances past a length-prefixed string.
    fn skip_string(&mut self) -> Result<()>;

    /// Reads an unsigned byte.
    fn get_u8(&mut self) -> Result<u8>;
    /// Reads a signed byte.
    fn get_i8(&mut self) -> Result<i8>;
    /// Reads a 16-bit signed integer.
    fn get_i16(&mut self) -> Result<i16>;
    /// Reads a 32-bit signed integer.
    fn get_i32(&mut self) -> Result<i32>;
    /// Reads a 64-bit signed integer.
    fn get_i64(&mut self) -> Result<i64>;
    /// Reads an array of 32-bit signed integers.
    fn get_i32_array(&mut self, len: usize) -> Result<Vec<i32>> {
        let mut values = Vec::with_capacity(len.min(4096));
        for _ in 0..len {
            values.push(self.get_i32()?);
        }
        Ok(values)
    }
    /// Reads an array of 64-bit signed integers.
    fn get_i64_array(&mut self, len: usize) -> Result<Vec<i64>> {
        let mut values = Vec::with_capacity(len.min(4096));
        for _ in 0..len {
            values.push(self.get_i64()?);
        }
        Ok(values)
    }
    /// Reads a 32-bit floating-point number.
    fn get_f32(&mut self) -> Result<f32>;
    /// Reads a 64-bit floating-point number.
    fn get_f64(&mut self) -> Result<f64>;
    /// Reads a length-prefixed string.
    fn get_string(&mut self) -> Result<Cow<'a, str>>;
    /// Reads a byte array with the supplied element count.
    fn get_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>>;
}

/// Reads Java Edition NBT primitives using big-endian numeric encoding.
pub struct NbtReadHelperJava<D> {
    reader: D,
}

impl<D> NbtReadHelperJava<D> {
    /// Creates a Java Edition reader over `r`.
    pub const fn new(r: D) -> Self {
        Self { reader: r }
    }
}

/// Reads Bedrock network NBT primitives using little-endian and variable-length encoding.
pub struct NbtReadHelperBedrock<D> {
    reader: D,
}

impl<D> NbtReadHelperBedrock<D> {
    /// Creates a Bedrock network reader over `r`.
    pub const fn new(r: D) -> Self {
        Self { reader: r }
    }
}

impl<'a, D: NbtDataSource<'a>> NbtReadHelperJava<D> {
    fn get_string_len(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.reader.read_bytes(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }
}

impl<'a, D: NbtDataSource<'a>> NbtReadHelper<'a> for NbtReadHelperJava<D> {
    type Reader = D;

    fn reader(&mut self) -> &mut D {
        &mut self.reader
    }

    fn skip_string(&mut self) -> Result<()> {
        let len = self.get_string_len()? as i64;
        self.skip_bytes(len)
    }

    fn get_u8(&mut self) -> Result<u8> {
        self.reader.read_u8()
    }
    fn get_i8(&mut self) -> Result<i8> {
        Ok(self.reader.read_u8()? as i8)
    }
    fn get_i16(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.reader.read_bytes(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }
    fn get_i32(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.reader.read_bytes(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }
    fn get_i64(&mut self) -> Result<i64> {
        let mut buf = [0u8; 8];
        self.reader.read_bytes(&mut buf)?;
        Ok(i64::from_be_bytes(buf))
    }
    fn get_i32_array(&mut self, len: usize) -> Result<Vec<i32>> {
        let byte_len = len
            .checked_mul(std::mem::size_of::<i32>())
            .ok_or(Error::LargeLength(len))?;
        let mut values = Vec::<i32>::with_capacity(len);
        // SAFETY: The vector has capacity for `byte_len` bytes, and `u8` accepts
        // every bit pattern. Its length remains zero until the read succeeds.
        let bytes =
            unsafe { std::slice::from_raw_parts_mut(values.as_mut_ptr().cast::<u8>(), byte_len) };
        self.reader.read_bytes(bytes)?;
        // SAFETY: Every byte of all `len` elements was initialized by `read_bytes`,
        // and every bit pattern is a valid `i32`.
        unsafe { values.set_len(len) };
        for value in &mut values {
            *value = value.to_be();
        }
        Ok(values)
    }
    fn get_i64_array(&mut self, len: usize) -> Result<Vec<i64>> {
        let byte_len = len
            .checked_mul(std::mem::size_of::<i64>())
            .ok_or(Error::LargeLength(len))?;
        let mut values = Vec::<i64>::with_capacity(len);
        // SAFETY: The vector has capacity for `byte_len` bytes, and `u8` accepts
        // every bit pattern. Its length remains zero until the read succeeds.
        let bytes =
            unsafe { std::slice::from_raw_parts_mut(values.as_mut_ptr().cast::<u8>(), byte_len) };
        self.reader.read_bytes(bytes)?;
        // SAFETY: Every byte of all `len` elements was initialized by `read_bytes`,
        // and every bit pattern is a valid `i64`.
        unsafe { values.set_len(len) };
        for value in &mut values {
            *value = value.to_be();
        }
        Ok(values)
    }
    fn get_f32(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.reader.read_bytes(&mut buf)?;
        Ok(f32::from_be_bytes(buf))
    }
    fn get_f64(&mut self) -> Result<f64> {
        let mut buf = [0u8; 8];
        self.reader.read_bytes(&mut buf)?;
        Ok(f64::from_be_bytes(buf))
    }

    fn get_string(&mut self) -> Result<Cow<'a, str>> {
        let len = self.get_string_len()? as usize;
        self.reader.read_string(len)
    }

    fn get_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>> {
        self.reader.read_byte_array(len)
    }
}

impl<'a, D: NbtDataSource<'a>> NbtReadHelperBedrock<D> {
    fn get_u8(&mut self) -> Result<u8> {
        self.reader.read_u8()
    }

    fn get_var_u32(&mut self) -> Result<u32> {
        let mut val = 0;
        for i in 0..5 {
            let byte = self.get_u8()?;
            val |= (u32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(val);
            }
        }
        Err(Error::VarIntTooLarge)
    }

    fn get_var_i32(&mut self) -> Result<i32> {
        let val = self.get_var_u32()?;
        Ok(((val >> 1) as i32) ^ -((val as i32) & 1))
    }

    fn get_var_u64(&mut self) -> Result<u64> {
        let mut val = 0;
        for i in 0..10 {
            let byte = self.get_u8()?;
            val |= (u64::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(val);
            }
        }
        Err(Error::VarLongTooLarge)
    }

    fn get_var_i64(&mut self) -> Result<i64> {
        let val = self.get_var_u64()?;
        Ok(((val >> 1) as i64) ^ -((val as i64) & 1))
    }

    fn get_string_len(&mut self) -> Result<u32> {
        self.get_var_u32()
    }
}

impl<'a, D: NbtDataSource<'a>> NbtReadHelper<'a> for NbtReadHelperBedrock<D> {
    type Reader = D;

    fn reader(&mut self) -> &mut D {
        &mut self.reader
    }

    fn skip_string(&mut self) -> Result<()> {
        let len = self.get_string_len()? as i64;
        self.skip_bytes(len)
    }

    fn get_u8(&mut self) -> Result<u8> {
        self.reader.read_u8()
    }
    fn get_i8(&mut self) -> Result<i8> {
        Ok(self.reader.read_u8()? as i8)
    }
    fn get_i16(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.reader.read_bytes(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }
    fn get_i32(&mut self) -> Result<i32> {
        self.get_var_i32()
    }
    fn get_i64(&mut self) -> Result<i64> {
        self.get_var_i64()
    }
    fn get_f32(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.reader.read_bytes(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }
    fn get_f64(&mut self) -> Result<f64> {
        let mut buf = [0u8; 8];
        self.reader.read_bytes(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    fn get_string(&mut self) -> Result<Cow<'a, str>> {
        let len = self.get_string_len()? as usize;
        self.reader.read_string(len)
    }

    fn get_byte_array(&mut self, len: usize) -> Result<Cow<'a, [i8]>> {
        self.reader.read_byte_array(len)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{NbtReadHelper, NbtReadHelperJava};

    #[test]
    fn java_numeric_arrays_decode_big_endian_values() {
        let ints = [i32::MIN, -1, 0, 1, i32::MAX];
        let int_bytes: Vec<u8> = ints.iter().flat_map(|value| value.to_be_bytes()).collect();
        let mut reader = NbtReadHelperJava::new(Cursor::new(int_bytes.as_slice()));
        assert_eq!(reader.get_i32_array(ints.len()).unwrap(), ints);

        let longs = [i64::MIN, -1, 0, 1, i64::MAX];
        let long_bytes: Vec<u8> = longs.iter().flat_map(|value| value.to_be_bytes()).collect();
        let mut reader = NbtReadHelperJava::new(Cursor::new(long_bytes.as_slice()));
        assert_eq!(reader.get_i64_array(longs.len()).unwrap(), longs);
    }
}

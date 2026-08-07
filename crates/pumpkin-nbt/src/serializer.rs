use std::io::Write;

use crate::Error;

/// Result type returned by NBT serialization operations.
pub type Result<T> = std::result::Result<T, Error>;

macro_rules! define_write_number_be {
    ($name:ident, $type:ty) => {
        fn $name(&mut self, value: $type) -> Result<()> {
            let buf = value.to_be_bytes();
            self.writer.write_all(&buf).map_err(Error::Incomplete)?;
            Ok(())
        }
    };
}

macro_rules! define_write_number_le {
    ($name:ident, $type:ty) => {
        fn $name(&mut self, value: $type) -> Result<()> {
            let buf = value.to_le_bytes();
            self.writer.write_all(&buf).map_err(Error::Incomplete)?;
            Ok(())
        }
    };
}

/// Format-specific primitive writer used by the NBT serializer.
pub trait NbtWriteHelper {
    /// Underlying output writer.
    type Writer: Write;

    /// Returns the underlying output writer.
    fn writer(&mut self) -> &mut Self::Writer;
    /// Writes an unsigned byte.
    fn write_u8(&mut self, value: u8) -> Result<()>;
    /// Writes a signed byte.
    fn write_i8(&mut self, value: i8) -> Result<()>;
    /// Writes a 16-bit signed integer.
    fn write_i16(&mut self, value: i16) -> Result<()>;
    /// Writes a 32-bit signed integer.
    fn write_i32(&mut self, value: i32) -> Result<()>;
    /// Writes a 64-bit signed integer.
    fn write_i64(&mut self, value: i64) -> Result<()>;
    /// Writes a 32-bit floating-point number.
    fn write_f32(&mut self, value: f32) -> Result<()>;
    /// Writes a 64-bit floating-point number.
    fn write_f64(&mut self, value: f64) -> Result<()>;
    /// Writes a length-prefixed string.
    fn write_string(&mut self, value: &str) -> Result<()>;

    /// Writes an unmodified byte slice.
    fn write_slice(&mut self, value: &[u8]) -> Result<()> {
        self.writer().write_all(value).map_err(Error::Incomplete)?;
        Ok(())
    }
}

/// Writes Java Edition NBT primitives using big-endian numeric encoding.
pub struct NbtWriteHelperJava<W: Write> {
    writer: W,
}

impl<W: Write> NbtWriteHelperJava<W> {
    /// Creates a Java Edition writer over `w`.
    pub const fn new(w: W) -> Self {
        Self { writer: w }
    }
}

impl<W: Write> NbtWriteHelperJava<W> {
    define_write_number_be!(write_string_len, u16);
}

impl<W: Write> NbtWriteHelper for NbtWriteHelperJava<W> {
    type Writer = W;

    fn writer(&mut self) -> &mut Self::Writer {
        &mut self.writer
    }

    define_write_number_be!(write_u8, u8);
    define_write_number_be!(write_i8, i8);
    define_write_number_be!(write_i16, i16);
    define_write_number_be!(write_i32, i32);
    define_write_number_be!(write_i64, i64);
    define_write_number_be!(write_f32, f32);
    define_write_number_be!(write_f64, f64);

    fn write_string(&mut self, value: &str) -> Result<()> {
        let java_string = cesu8::to_java_cesu8(value);
        let len = java_string.len();
        if len > u16::MAX as usize {
            return Err(Error::LargeLength(len));
        }

        self.write_string_len(len as u16)?;
        self.writer
            .write_all(&java_string)
            .map_err(Error::Incomplete)?;
        Ok(())
    }
}

/// Writes Bedrock network NBT primitives using little-endian and variable-length encoding.
pub struct NbtWriteHelperBedrock<W: Write> {
    writer: W,
}

impl<W: Write> NbtWriteHelperBedrock<W> {
    /// Creates a Bedrock network writer over `w`.
    pub const fn new(w: W) -> Self {
        Self { writer: w }
    }
}

impl<W: Write> NbtWriteHelperBedrock<W> {
    fn write_var_u32(&mut self, mut value: u32) -> Result<()> {
        // LEB128
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.write_u8(byte)?;
            if value == 0 {
                break;
            }
        }
        Ok(())
    }

    fn write_var_i32(&mut self, value: i32) -> Result<()> {
        // ZigZag
        self.write_var_u32(((value << 1) ^ (value >> 31)) as u32)
    }

    fn write_var_u64(&mut self, mut value: u64) -> Result<()> {
        // LEB128
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.write_u8(byte)?;
            if value == 0 {
                break;
            }
        }
        Ok(())
    }

    fn write_var_i64(&mut self, value: i64) -> Result<()> {
        // ZigZag
        self.write_var_u64(((value << 1) ^ (value >> 63)) as u64)
    }

    fn write_string_len(&mut self, value: u32) -> Result<()> {
        self.write_var_u32(value)
    }
}

impl<W: Write> NbtWriteHelper for NbtWriteHelperBedrock<W> {
    type Writer = W;

    fn writer(&mut self) -> &mut Self::Writer {
        &mut self.writer
    }

    define_write_number_le!(write_u8, u8);
    define_write_number_le!(write_i8, i8);
    define_write_number_le!(write_i16, i16);
    fn write_i32(&mut self, value: i32) -> Result<()> {
        self.write_var_i32(value)
    }
    fn write_i64(&mut self, value: i64) -> Result<()> {
        self.write_var_i64(value)
    }
    define_write_number_le!(write_f32, f32);
    define_write_number_le!(write_f64, f64);

    fn write_string(&mut self, value: &str) -> Result<()> {
        let bedrock_string = value.as_bytes();
        let len = bedrock_string.len();
        if len > u32::MAX as usize {
            return Err(Error::LargeLength(len));
        }

        self.write_string_len(len as u32)?;
        self.writer
            .write_all(bedrock_string)
            .map_err(Error::Incomplete)?;
        Ok(())
    }
}

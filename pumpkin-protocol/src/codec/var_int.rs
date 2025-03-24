use std::{
    io::{ErrorKind, Read, Write},
    num::NonZeroUsize,
    ops::Deref,
};

use crate::ser::{NetworkReadExt, ReadingError, WritingError};

use super::Codec;
use bytes::BufMut;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{SeqAccess, Visitor},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub type VarIntType = i32;

/**
 * A variable-length integer type used by the Minecraft network protocol.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VarInt(pub VarIntType);

impl Codec<Self> for VarInt {
    /// The maximum number of bytes a `VarInt` can occupy.
    const MAX_SIZE: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(5) };

    /// Returns the exact number of bytes this VarInt will write when
    /// [`Encode::encode`] is called, assuming no error occurs.
    fn written_size(&self) -> usize {
        match self.0 {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }

    fn encode(&self, write: &mut impl Write) -> Result<(), WritingError> {
        let (simd, bytes_needed) = self.encode_simd();
        write
            .write_all(&simd.to_le_bytes()[..bytes_needed as usize])
            .map_err(WritingError::IoError)?;
        Ok(())
    }

    fn decode(read: &mut impl Read) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.get_u8_be()?;
            val |= (i32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(VarInt(val));
            }
        }
        Err(ReadingError::TooLarge("VarInt".to_string()))
    }
}

impl VarInt {
    // Adapted from VarInt-Simd encode
    // https://github.com/as-com/varint-simd/blob/0f468783da8e181929b01b9c6e9f741c1fe09825/src/encode/mod.rs#L71
    pub fn encode_simd(&self) -> (u64, u32) {
        let simd = self.0 as u64;

        let stage1 = (simd & 0x000000000000007f)
            | ((simd & 0x0000000000003f80) << 1)
            | ((simd & 0x00000000001fc000) << 2)
            | ((simd & 0x000000000fe00000) << 3)
            | ((simd & 0x00000000f0000000) << 4);

        let leading = stage1.leading_zeros();

        let unused_bytes = (leading - 1) >> 3;
        let bytes_needed = 8 - unused_bytes;

        // set all but the last MSBs
        let msbs = 0x8080808080808080;
        let msbmask = 0xffffffffffffffff >> (((8 - bytes_needed + 1) << 3) - 1);

        (stage1 | (msbs & msbmask), unused_bytes)
    }

    pub async fn decode_async(read: &mut (impl AsyncRead + Unpin)) -> Result<Self, ReadingError> {
        let mut val = 0;
        for i in 0..Self::MAX_SIZE.get() {
            let byte = read.read_u8().await.map_err(|err| {
                if i == 0 && matches!(err.kind(), ErrorKind::UnexpectedEof) {
                    ReadingError::CleanEOF("VarInt".to_string())
                } else {
                    ReadingError::Incomplete(err.to_string())
                }
            })?;
            val |= (i32::from(byte) & 0x7F) << (i * 7);
            if byte & 0x80 == 0 {
                return Ok(VarInt(val));
            }
        }
        Err(ReadingError::TooLarge("VarInt".to_string()))
    }

    pub async fn encode_async(
        &self,
        write: &mut (impl AsyncWrite + Unpin),
    ) -> Result<(), WritingError> {
        let (simd, bytes_needed) = self.encode_simd();
        write
            .write_all(&simd.to_le_bytes()[..bytes_needed as usize])
            .await
            .map_err(WritingError::IoError)?;
        Ok(())
    }
}

impl From<i32> for VarInt {
    fn from(value: i32) -> Self {
        VarInt(value)
    }
}

impl From<u32> for VarInt {
    fn from(value: u32) -> Self {
        VarInt(value as i32)
    }
}

impl From<u8> for VarInt {
    fn from(value: u8) -> Self {
        VarInt(value as i32)
    }
}

impl From<usize> for VarInt {
    fn from(value: usize) -> Self {
        VarInt(value as i32)
    }
}

impl From<VarInt> for i32 {
    fn from(value: VarInt) -> Self {
        value.0
    }
}

impl AsRef<i32> for VarInt {
    fn as_ref(&self) -> &i32 {
        &self.0
    }
}

impl Deref for VarInt {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for VarInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut value = self.0 as u32;
        let mut buf = Vec::new();

        while value > 0x7F {
            buf.put_u8(value as u8 | 0x80);
            value >>= 7;
        }

        buf.put_u8(value as u8);

        serializer.serialize_bytes(&buf)
    }
}

impl<'de> Deserialize<'de> for VarInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VarIntVisitor;

        impl<'de> Visitor<'de> for VarIntVisitor {
            type Value = VarInt;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid VarInt encoded in a byte sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut val = 0;
                for i in 0..VarInt::MAX_SIZE.get() {
                    if let Some(byte) = seq.next_element::<u8>()? {
                        val |= (i32::from(byte) & 0b01111111) << (i * 7);
                        if byte & 0b10000000 == 0 {
                            return Ok(VarInt(val));
                        }
                    } else {
                        break;
                    }
                }
                Err(serde::de::Error::custom("VarInt was too large"))
            }
        }

        deserializer.deserialize_seq(VarIntVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::VarInt;
    use crate::{codec::Codec, ser::ReadingError};
    use std::io::Cursor;

    #[test]
    fn test_varint_encode_decode() {
        let test_cases = [
            0,
            1,
            127,
            128,
            255,
            2147,
            16384,
            0x7FFF_FFFF,
            -1,
            -2147483648,
        ];

        for &value in &test_cases {
            let varint = VarInt(value);
            let mut buffer = Vec::new();
            varint.encode(&mut buffer).unwrap();

            let mut reader = Cursor::new(buffer);
            let decoded = VarInt::decode(&mut reader).unwrap();

            assert_eq!(decoded.0, value);
            assert_eq!(reader.position() as usize, varint.written_size());
        }
    }

    #[test]
    fn test_varint_max_size() {
        let mut buffer = [0u8; 5];
        // Max varint (5 bytes)
        let max_varint = VarInt(0x7FFF_FFFF);
        max_varint.encode(&mut &mut buffer[..]).unwrap();

        let mut reader = Cursor::new(&buffer[..]);
        assert_eq!(VarInt::decode(&mut reader).unwrap().0, 0x7FFF_FFFF);

        // Long varint (6 bytes)
        let invalid_data = [0x80, 0x80, 0x80, 0x80, 0x80, 0x00];
        let mut reader = Cursor::new(&invalid_data[..]);
        match VarInt::decode(&mut reader) {
            Err(ReadingError::TooLarge(_)) => (),
            _ => panic!("Expected TooLarge error"),
        }
    }

    #[tokio::test]
    async fn test_varint_async() {
        let varint = VarInt(32767);
        let mut buffer = Vec::new();
        varint.encode_async(&mut buffer).await.unwrap();

        let mut reader = Cursor::new(buffer);
        let decoded = VarInt::decode_async(&mut reader).await.unwrap();
        assert_eq!(decoded.0, 32767);
    }
}

use std::io::{Error, Read};

use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use crate::{codec::var_uint::VarUInt, serial::PacketRead};

impl PacketRead for bool {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        Ok(buf[0] != 0)
    }
}

impl PacketRead for i8 {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        Ok(buf[0] as Self)
    }
}

impl PacketRead for i16 {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl PacketRead for i32 {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }

    fn read_be<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_be_bytes(buf))
    }
}

impl PacketRead for i64 {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl PacketRead for u8 {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl PacketRead for u16 {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl PacketRead for u32 {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl PacketRead for u64 {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl PacketRead for f32 {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl PacketRead for f64 {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_le_bytes(buf))
    }
}

impl<T: PacketRead, const N: usize> PacketRead for [T; N] {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        #[allow(clippy::uninit_assumed_init)]
        let mut buf: [T; N] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        for i in &mut buf {
            *i = T::read(reader)?;
        }
        Ok(buf)
    }
}

impl PacketRead for String {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let vec = Vec::read(reader)?;
        Ok(unsafe { String::from_utf8_unchecked(vec) })
    }
}

impl PacketRead for Vec<u8> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        #[allow(clippy::uninit_vec)]
        {
            let len = VarUInt::read(reader)?.0 as _;
            let mut buf = Vec::with_capacity(len);
            unsafe {
                buf.set_len(len);
            }
            reader.read_exact(&mut buf)?;
            Ok(buf)
        }
    }
}

impl<T: PacketRead> PacketRead for Vector3<T> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            x: T::read(reader)?,
            y: T::read(reader)?,
            z: T::read(reader)?,
        })
    }
}

impl<T: PacketRead> PacketRead for Vector2<T> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            x: T::read(reader)?,
            y: T::read(reader)?,
        })
    }
}

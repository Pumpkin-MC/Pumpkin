use std::io::{Error, Read};

use crate::{codec::var_uint::VarUInt, serial::PacketRead};

impl PacketRead for bool {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        Ok(if buf[0] == 0 { false } else { true })
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
        use std::mem::{MaybeUninit, transmute_copy};
        let mut buf: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..N {
            buf[i] = MaybeUninit::new(T::read(reader)?);
        }
        let initialized = unsafe { transmute_copy::<[MaybeUninit<T>; N], [T; N]>(&buf) };
        Ok(initialized)
    }
}

impl PacketRead for String {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let len = VarUInt::read(reader)?.0 as _;
        let mut buf = vec![0; len];
        reader.read_exact(&mut buf)?;
        Ok(unsafe { String::from_utf8_unchecked(buf) })
    }
}

impl PacketRead for Vec<u8> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let len = VarUInt::read(reader)?.0 as _;
        let mut buf = vec![0; len];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }
}

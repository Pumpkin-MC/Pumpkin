use std::{
    borrow::Cow,
    io::{Error, ErrorKind, Read},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
use uuid::Uuid;

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::{PacketRead, PacketReadSlice, read_str_slice},
};

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

    fn read_be<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0; size_of::<Self>()];
        reader.read_exact(&mut buf)?;
        Ok(Self::from_be_bytes(buf))
    }
}

impl PacketRead for u32 {
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

impl PacketRead for u64 {
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
        let mut buf: [std::mem::MaybeUninit<T>; N] =
            std::array::from_fn(|_| std::mem::MaybeUninit::uninit());
        for i in 0..N {
            match T::read(reader) {
                Ok(val) => {
                    buf[i].write(val);
                }
                Err(err) => {
                    for elem in &mut buf[..i] {
                        // SAFETY: Only elements 0..i were initialized in previous iterations and need to be dropped on error.
                        unsafe {
                            elem.assume_init_drop();
                        }
                    }
                    return Err(err);
                }
            }
        }
        // SAFETY: All N elements were successfully initialized in the loop above.
        Ok(buf.map(|elem| unsafe { elem.assume_init() }))
    }
}

impl PacketRead for String {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        const MAX_STRING_LENGTH: usize = 32767;

        let len = VarUInt::read(reader)?.0 as usize;

        if len > MAX_STRING_LENGTH {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("String length {len} exceeds maximum of {MAX_STRING_LENGTH}"),
            ));
        }

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;

        Self::from_utf8(buf)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 sequence"))
    }
}

impl<T: PacketRead> PacketRead for Vec<T> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let len = VarUInt::read(reader)?.0 as _;
        let mut buf = Self::with_capacity(len);
        for _ in 0..len {
            buf.push(T::read(reader)?);
        }
        Ok(buf)
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

impl PacketRead for BlockPos {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self(Vector3 {
            x: VarInt::read(reader)?.0,
            y: VarInt::read(reader)?.0,
            z: VarInt::read(reader)?.0,
        }))
    }
}

impl PacketRead for SocketAddr {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        match u8::read(reader)? {
            4 => {
                let ip = u32::read_be(reader)?;
                let port = u16::read_be(reader)?;
                Ok(Self::V4(SocketAddrV4::new(Ipv4Addr::from(ip), port)))
            }
            6 => {
                // Addr family
                u16::read(reader)?;
                let port = u16::read_be(reader)?;
                let flowinfo = u32::read_be(reader)?;

                let mut ip = [0; 16];
                reader.read_exact(&mut ip)?;
                let ip = Ipv6Addr::from(ip);

                let scope_id = u32::read_be(reader)?;

                Ok(Self::V6(SocketAddrV6::new(ip, port, flowinfo, scope_id)))
            }
            _ => Err(Error::other("Invalid socket address version")),
        }
    }
}

impl PacketRead for Uuid {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut bytes = [0; 16];
        reader.read_exact(&mut bytes)?;
        Ok(Self::from_bytes(bytes))
    }
}

impl<T: PacketRead> PacketRead for Option<T> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        bool::read(reader)?.then(|| T::read(reader)).transpose()
    }
}

impl PacketRead for Cow<'_, str> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self::Owned(String::read(reader)?))
    }
}

impl<'a> PacketReadSlice<'a> for bool {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if buf.is_empty() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "expected bool byte"));
        }
        let b = buf[0];
        *buf = &buf[1..];
        Ok(b != 0)
    }
}

impl<'a> PacketReadSlice<'a> for u8 {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if buf.is_empty() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "expected u8"));
        }
        let b = buf[0];
        *buf = &buf[1..];
        Ok(b)
    }
}

impl<'a> PacketReadSlice<'a> for i8 {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        u8::read_slice(buf).map(|b| b as Self)
    }
}

impl<'a> PacketReadSlice<'a> for i16 {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if buf.len() < 2 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "expected i16"));
        }
        let (bytes, rest) = buf.split_at(2);
        *buf = rest;
        let arr = bytes
            .try_into()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid i16 slice"))?;
        Ok(Self::from_le_bytes(arr))
    }
}

impl<'a> PacketReadSlice<'a> for i32 {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if buf.len() < 4 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "expected i32"));
        }
        let (bytes, rest) = buf.split_at(4);
        *buf = rest;
        let arr = bytes
            .try_into()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid i32 slice"))?;
        Ok(Self::from_le_bytes(arr))
    }
}

impl<'a> PacketReadSlice<'a> for i64 {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if buf.len() < 8 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "expected i64"));
        }
        let (bytes, rest) = buf.split_at(8);
        *buf = rest;
        let arr = bytes
            .try_into()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid i64 slice"))?;
        Ok(Self::from_le_bytes(arr))
    }
}

impl<'a> PacketReadSlice<'a> for u16 {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if buf.len() < 2 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "expected u16"));
        }
        let (bytes, rest) = buf.split_at(2);
        *buf = rest;
        let arr = bytes
            .try_into()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid u16 slice"))?;
        Ok(Self::from_le_bytes(arr))
    }
}

impl<'a> PacketReadSlice<'a> for u32 {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if buf.len() < 4 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "expected u32"));
        }
        let (bytes, rest) = buf.split_at(4);
        *buf = rest;
        let arr = bytes
            .try_into()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid u32 slice"))?;
        Ok(Self::from_le_bytes(arr))
    }
}

impl<'a> PacketReadSlice<'a> for u64 {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if buf.len() < 8 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "expected u64"));
        }
        let (bytes, rest) = buf.split_at(8);
        *buf = rest;
        let arr = bytes
            .try_into()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid u64 slice"))?;
        Ok(Self::from_le_bytes(arr))
    }
}

impl<'a> PacketReadSlice<'a> for f32 {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if buf.len() < 4 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "expected f32"));
        }
        let (bytes, rest) = buf.split_at(4);
        *buf = rest;
        let arr = bytes
            .try_into()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid f32 slice"))?;
        Ok(Self::from_le_bytes(arr))
    }
}

impl<'a> PacketReadSlice<'a> for f64 {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if buf.len() < 8 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "expected f64"));
        }
        let (bytes, rest) = buf.split_at(8);
        *buf = rest;
        let arr = bytes
            .try_into()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid f64 slice"))?;
        Ok(Self::from_le_bytes(arr))
    }
}

impl<'a> PacketReadSlice<'a> for &'a str {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        read_str_slice(buf)
    }
}

impl<'a, T: PacketReadSlice<'a>> PacketReadSlice<'a> for Option<T> {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        bool::read_slice(buf)?
            .then(|| T::read_slice(buf))
            .transpose()
    }
}

impl<'a> PacketReadSlice<'a> for uuid::Uuid {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        if buf.len() < 16 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "expected Uuid"));
        }
        let (bytes, rest) = buf.split_at(16);
        *buf = rest;
        let arr = bytes
            .try_into()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid Uuid slice"))?;
        Ok(Self::from_bytes(arr))
    }
}

impl<'a> PacketReadSlice<'a> for crate::codec::var_int::VarInt {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Self::read(buf)
    }
}

impl<'a> PacketReadSlice<'a> for crate::codec::var_uint::VarUInt {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Self::read(buf)
    }
}

impl<'a> PacketReadSlice<'a> for crate::codec::var_ulong::VarULong {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Self::read(buf)
    }
}

impl<'a> PacketReadSlice<'a> for Cow<'a, str> {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        Ok(Self::Borrowed(read_str_slice(buf)?))
    }
}

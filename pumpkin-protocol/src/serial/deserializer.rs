use std::{
    io::{Error, Read},
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
use uuid::Uuid;

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
        #[expect(clippy::uninit_assumed_init)]
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
        Self::from_utf8(vec).map_err(|e| Error::other(format!("Invalid UTF-8: {e}")))
    }
}

impl PacketRead for Vec<u8> {
    #[expect(clippy::read_zero_byte_vec)]
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        // Cap at 1 MB to prevent OOM from a malicious length prefix.
        const MAX_VEC_LENGTH: u32 = 1024 * 1024;
        let len = VarUInt::read(reader)?.0;
        if len > MAX_VEC_LENGTH {
            return Err(Error::other(format!(
                "Vec<u8> length {len} exceeds maximum ({MAX_VEC_LENGTH})"
            )));
        }
        let len = len as usize;
        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;
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

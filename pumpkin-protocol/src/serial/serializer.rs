use std::{
    io::{Error, Write},
    net::SocketAddr,
};

use pumpkin_util::{
    GameMode,
    math::{position::BlockPos, vector3::Vector3},
};

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};

impl PacketWrite for bool {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&if *self { [1] } else { [0] }).map(|_| ())
    }
}

impl PacketWrite for i8 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_le_bytes()).map(|_| ())
    }
}

impl PacketWrite for i16 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_le_bytes()).map(|_| ())
    }
}

impl PacketWrite for i32 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_le_bytes()).map(|_| ())
    }

    fn write_be<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_be_bytes()).map(|_| ())
    }
}

impl PacketWrite for i64 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_le_bytes()).map(|_| ())
    }

    fn write_be<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_be_bytes()).map(|_| ())
    }
}

impl PacketWrite for u8 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_le_bytes()).map(|_| ())
    }
}

impl PacketWrite for u16 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_le_bytes()).map(|_| ())
    }

    fn write_be<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_be_bytes()).map(|_| ())
    }
}

impl PacketWrite for u32 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_le_bytes()).map(|_| ())
    }

    fn write_be<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_be_bytes()).map(|_| ())
    }
}

impl PacketWrite for u64 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_le_bytes()).map(|_| ())
    }

    fn write_be<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_be_bytes()).map(|_| ())
    }
}

impl PacketWrite for f32 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_le_bytes()).map(|_| ())
    }
}

impl PacketWrite for f64 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write(&self.to_le_bytes()).map(|_| ())
    }
}

impl<T: PacketWrite, const N: usize> PacketWrite for [T; N] {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

impl<T: PacketWrite> PacketWrite for Vec<T> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        for item in self {
            item.write(writer)?;
        }
        Ok(())
    }
}

impl PacketWrite for String {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(self.len() as _).write(writer)?;
        writer.write_all(self.as_bytes())
    }
}

impl<T: PacketWrite> PacketWrite for Vector3<T> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.x.write(writer)?;
        self.y.write(writer)?;
        self.z.write(writer)
    }
}

impl PacketWrite for BlockPos {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(self.0.x).write(writer)?;
        VarInt(self.0.y).write(writer)?;
        VarInt(self.0.z).write(writer)
    }
}

impl<T: PacketWrite> PacketWrite for Option<T> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        match self {
            Self::Some(value) => {
                true.write(writer)?;
                value.write(writer)
            }
            Self::None => false.write(writer),
        }
    }
}

impl PacketWrite for SocketAddr {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        match self {
            // version, addr, port
            Self::V4(addr) => {
                4u8.write(writer)?;
                writer.write_all(&addr.ip().octets())?;
                addr.port().write_be(writer)
            }
            // version, addr_family, port, flow_info, addr, scope_id
            Self::V6(addr) => {
                6u8.write(writer)?;
                10u16.write(writer)?;
                addr.port().write_be(writer)?;
                addr.flowinfo().write_be(writer)?;
                writer.write_all(&addr.ip().octets())?;
                addr.scope_id().write_be(writer)
            }
        }
    }
}

impl PacketWrite for GameMode {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(match self {
            Self::Survival => 0,
            Self::Creative => 1,
            Self::Adventure => 2,
            // I have no idea why
            Self::Spectator => 6,
        })
        .write(writer)
    }
}

use crate::java::legacy::ids::clientbound;
use crate::java::legacy::{LegacyPacket, LegacyRead, LegacyWrite, legacy_ids};
use crate::java::legacy::{LegacyReadExt, LegacyWriteExt};
use crate::{
    VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::version::JavaMinecraftVersion;
use uuid::Uuid;

/// Spawns a painting entity in the world for versions <= 1.18.2.
/// In 1.19+, paintings are spawned via the unified `CSpawnEntity` packet. `variant` is
/// already the client's motive id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSpawnPainting {
    pub entity_id: VarInt,
    pub uuid: Uuid,
    pub title: String,
    pub variant: VarInt,
    pub location: BlockPos,
    pub direction: u8,
}

legacy_ids! { CSpawnPainting => clientbound::play::SPAWN_PAINTING }

impl CSpawnPainting {
    #[must_use]
    pub const fn new(
        entity_id: VarInt,
        uuid: Uuid,
        title: String,
        variant: VarInt,
        location: BlockPos,
        direction: u8,
    ) -> Self {
        Self {
            entity_id,
            uuid,
            title,
            variant,
            location,
            direction,
        }
    }
}

impl LegacyWrite for CSpawnPainting {
    fn write_legacy(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&self.entity_id)?;
        if *version >= JavaMinecraftVersion::V_1_9 {
            write.write_uuid(&self.uuid)?;
        }
        if *version >= JavaMinecraftVersion::V_1_13 {
            write.write_var_int(&self.variant)?;
        } else {
            write.write_string_bounded(&self.title, 13)?;
        }

        if *version >= JavaMinecraftVersion::V_1_8 {
            write.write_block_pos_legacy(&self.location, version)?;
            write.write_u8(self.direction)?;
        } else {
            write.write_i32_be(self.location.0.x)?;
            write.write_i32_be(self.location.0.y)?;
            write.write_i32_be(self.location.0.z)?;
            write.write_i32_be(self.direction as i32)?;
        }

        Ok(())
    }
}

impl<'a> LegacyRead<'a> for CSpawnPainting {
    fn read_legacy(
        bytebuf: &mut &'a [u8],
        version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        let entity_id = bytebuf.get_var_int()?;
        let uuid = if *version >= JavaMinecraftVersion::V_1_9 {
            bytebuf.get_uuid()?
        } else {
            Uuid::nil()
        };

        let (title, variant) = if *version >= JavaMinecraftVersion::V_1_13 {
            let variant = bytebuf.get_var_int()?;
            (String::new(), variant)
        } else {
            let title = bytebuf.get_str()?.to_string();
            (title, VarInt(0))
        };

        let (location, direction) = if *version >= JavaMinecraftVersion::V_1_8 {
            let loc = bytebuf.get_block_pos_legacy(version)?;
            let dir = bytebuf.get_u8()?;
            (loc, dir)
        } else {
            let x = bytebuf.get_i32_be()?;
            let y = bytebuf.get_i32_be()?;
            let z = bytebuf.get_i32_be()?;
            let dir = bytebuf.get_i32_be()? as u8;
            (BlockPos::new(x, y, z), dir)
        };

        Ok(Self {
            entity_id,
            uuid,
            title,
            variant,
            location,
            direction,
        })
    }
}

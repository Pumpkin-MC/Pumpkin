use std::io::{Read, Write};

use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_id_remap::remap_entity_id_for_version;
use pumpkin_data::packet::clientbound::play::SPAWN_LIVING_ENTITY;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::{
    ClientPacket, ServerPacket, VarInt,
    codec::lp_vector_3d::LpVector3d,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

const ROTATION_FACTOR: f32 = 256.0 / 360.0;
const VELOCITY_FACTOR: f64 = 8000.0;

#[must_use]
#[expect(clippy::too_many_lines)]
pub fn remap_living_mob_type_for_version(entity_id: u16, version: JavaMinecraftVersion) -> u16 {
    if version >= JavaMinecraftVersion::V_1_14 {
        return remap_entity_id_for_version(entity_id, version);
    }

    if version >= JavaMinecraftVersion::V_1_11 {
        // 1.11 - 1.13 separate entity IDs
        if entity_id == EntityType::ELDER_GUARDIAN.id {
            return 4;
        } else if entity_id == EntityType::WITHER_SKELETON.id {
            return 5;
        } else if entity_id == EntityType::STRAY.id {
            return 6;
        } else if entity_id == EntityType::HUSK.id {
            return 23;
        } else if entity_id == EntityType::ZOMBIE_VILLAGER.id {
            return 27;
        } else if entity_id == EntityType::SKELETON_HORSE.id {
            return 28;
        } else if entity_id == EntityType::ZOMBIE_HORSE.id {
            return 29;
        } else if entity_id == EntityType::DONKEY.id {
            return 31;
        } else if entity_id == EntityType::MULE.id {
            return 32;
        } else if entity_id == EntityType::EVOKER.id {
            return 34;
        } else if entity_id == EntityType::VEX.id {
            return 35;
        } else if entity_id == EntityType::VINDICATOR.id {
            return 36;
        } else if entity_id == EntityType::ILLUSIONER.id {
            return 37;
        } else if entity_id == EntityType::LLAMA.id || entity_id == EntityType::TRADER_LLAMA.id {
            return 103;
        } else if entity_id == EntityType::PARROT.id {
            return 105;
        }
    }

    // 1.7.10 - 1.10 (and shared legacy base IDs for 1.11 - 1.12)
    if entity_id == EntityType::CREEPER.id {
        50
    } else if entity_id == EntityType::SKELETON.id
        || entity_id == EntityType::WITHER_SKELETON.id
        || entity_id == EntityType::STRAY.id
        || entity_id == EntityType::BOGGED.id
        || entity_id == EntityType::PARCHED.id
    {
        51
    } else if entity_id == EntityType::SPIDER.id {
        52
    } else if entity_id == EntityType::GIANT.id {
        53
    } else if entity_id == EntityType::ZOMBIE.id
        || entity_id == EntityType::DROWNED.id
        || entity_id == EntityType::HUSK.id
        || entity_id == EntityType::ZOMBIE_VILLAGER.id
    {
        54
    } else if entity_id == EntityType::SLIME.id {
        55
    } else if entity_id == EntityType::GHAST.id || entity_id == EntityType::HAPPY_GHAST.id {
        56
    } else if entity_id == EntityType::ZOMBIFIED_PIGLIN.id
        || entity_id == EntityType::PIGLIN.id
        || entity_id == EntityType::PIGLIN_BRUTE.id
    {
        57
    } else if entity_id == EntityType::ENDERMAN.id || entity_id == EntityType::CREAKING.id {
        58
    } else if entity_id == EntityType::CAVE_SPIDER.id {
        59
    } else if entity_id == EntityType::SILVERFISH.id {
        60
    } else if entity_id == EntityType::BLAZE.id || entity_id == EntityType::BREEZE.id {
        61
    } else if entity_id == EntityType::MAGMA_CUBE.id {
        62
    } else if entity_id == EntityType::ENDER_DRAGON.id {
        63
    } else if entity_id == EntityType::WITHER.id {
        64
    } else if entity_id == EntityType::BAT.id
        || entity_id == EntityType::VEX.id
        || entity_id == EntityType::ALLAY.id
        || entity_id == EntityType::BEE.id
        || entity_id == EntityType::PARROT.id
    {
        65
    } else if entity_id == EntityType::WITCH.id {
        66
    } else if entity_id == EntityType::ENDERMITE.id {
        67
    } else if entity_id == EntityType::GUARDIAN.id || entity_id == EntityType::ELDER_GUARDIAN.id {
        68
    } else if entity_id == EntityType::SHULKER.id {
        69
    } else if entity_id == EntityType::PIG.id
        || entity_id == EntityType::HOGLIN.id
        || entity_id == EntityType::ZOGLIN.id
        || entity_id == EntityType::STRIDER.id
    {
        90
    } else if entity_id == EntityType::SHEEP.id
        || entity_id == EntityType::GOAT.id
        || entity_id == EntityType::SNIFFER.id
        || entity_id == EntityType::ARMADILLO.id
    {
        91
    } else if entity_id == EntityType::COW.id || entity_id == EntityType::PANDA.id {
        92
    } else if entity_id == EntityType::CHICKEN.id {
        93
    } else if entity_id == EntityType::SQUID.id
        || entity_id == EntityType::GLOW_SQUID.id
        || entity_id == EntityType::DOLPHIN.id
        || entity_id == EntityType::COD.id
        || entity_id == EntityType::SALMON.id
        || entity_id == EntityType::PUFFERFISH.id
        || entity_id == EntityType::TROPICAL_FISH.id
        || entity_id == EntityType::TADPOLE.id
        || entity_id == EntityType::AXOLOTL.id
        || entity_id == EntityType::FROG.id
        || entity_id == EntityType::NAUTILUS.id
    {
        94
    } else if entity_id == EntityType::WOLF.id || entity_id == EntityType::FOX.id {
        95
    } else if entity_id == EntityType::MOOSHROOM.id {
        96
    } else if entity_id == EntityType::SNOW_GOLEM.id {
        97
    } else if entity_id == EntityType::OCELOT.id || entity_id == EntityType::CAT.id {
        98
    } else if entity_id == EntityType::IRON_GOLEM.id
        || entity_id == EntityType::COPPER_GOLEM.id
        || entity_id == EntityType::RAVAGER.id
        || entity_id == EntityType::WARDEN.id
    {
        99
    } else if entity_id == EntityType::HORSE.id
        || entity_id == EntityType::DONKEY.id
        || entity_id == EntityType::MULE.id
        || entity_id == EntityType::ZOMBIE_HORSE.id
        || entity_id == EntityType::SKELETON_HORSE.id
        || entity_id == EntityType::CAMEL.id
        || entity_id == EntityType::LLAMA.id
        || entity_id == EntityType::TRADER_LLAMA.id
    {
        100
    } else if entity_id == EntityType::RABBIT.id {
        if version <= JavaMinecraftVersion::V_1_7_6 {
            93 // Chicken fallback in 1.7.10
        } else {
            101
        }
    } else if entity_id == EntityType::POLAR_BEAR.id {
        if version < JavaMinecraftVersion::V_1_10 {
            92 // Cow fallback in < 1.10
        } else {
            102
        }
    } else if entity_id == EntityType::VILLAGER.id
        || entity_id == EntityType::WANDERING_TRADER.id
        || entity_id == EntityType::PILLAGER.id
        || entity_id == EntityType::VINDICATOR.id
        || entity_id == EntityType::EVOKER.id
        || entity_id == EntityType::ILLUSIONER.id
    {
        120
    } else {
        54
    }
}

#[java_packet(SPAWN_LIVING_ENTITY)]
#[derive(Clone, Debug, PartialEq)]
pub struct CSpawnLivingEntity {
    pub entity_id: VarInt,
    pub entity_uuid: uuid::Uuid,
    pub r#type: VarInt,
    pub position: Vector3<f64>,
    pub yaw: u8,
    pub pitch: u8,
    pub head_yaw: u8,
    pub velocity: LpVector3d,
    pub metadata: Option<Box<[u8]>>,
}

impl CSpawnLivingEntity {
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        entity_id: VarInt,
        entity_uuid: uuid::Uuid,
        r#type: VarInt,
        position: Vector3<f64>,
        pitch: f32,
        yaw: f32,
        head_yaw: f32,
        velocity: Vector3<f64>,
        metadata: Option<Box<[u8]>>,
    ) -> Self {
        Self {
            entity_id,
            entity_uuid,
            r#type,
            position,
            pitch: (pitch * ROTATION_FACTOR).floor() as u8,
            yaw: (yaw.rem_euclid(360.0) * ROTATION_FACTOR).floor() as u8,
            head_yaw: (head_yaw.rem_euclid(360.0) * ROTATION_FACTOR).floor() as u8,
            velocity: LpVector3d(velocity),
            metadata,
        }
    }

    #[must_use]
    pub fn pitch_degrees(&self) -> f32 {
        (self.pitch as i8 as f32) / ROTATION_FACTOR
    }

    #[must_use]
    pub fn yaw_degrees(&self) -> f32 {
        (self.yaw as i8 as f32) / ROTATION_FACTOR
    }

    #[must_use]
    pub fn head_yaw_degrees(&self) -> f32 {
        (self.head_yaw as i8 as f32) / ROTATION_FACTOR
    }

    pub fn read_packet_data(
        mut read: impl Read,
        version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        let v1_9 = *version >= JavaMinecraftVersion::V_1_9;
        let v1_11 = *version >= JavaMinecraftVersion::V_1_11;
        let v1_15 = *version >= JavaMinecraftVersion::V_1_15;

        let entity_id = read.get_var_int()?;

        let (entity_uuid, r#type, position) = if v1_9 {
            let entity_uuid = read.get_uuid()?;
            let type_id = if v1_11 {
                read.get_var_int()?
            } else {
                VarInt(i32::from(read.get_u8()?))
            };
            let position = Vector3::new(read.get_f64_be()?, read.get_f64_be()?, read.get_f64_be()?);
            (entity_uuid, type_id, position)
        } else {
            let entity_uuid = uuid::Uuid::nil();
            let type_id = VarInt(i32::from(read.get_u8()?));
            let position = Vector3::new(
                f64::from(read.get_i32_be()?) / 32.0,
                f64::from(read.get_i32_be()?) / 32.0,
                f64::from(read.get_i32_be()?) / 32.0,
            );
            (entity_uuid, type_id, position)
        };

        let yaw = read.get_u8()?;
        let pitch = read.get_u8()?;
        let head_yaw = read.get_u8()?;

        let vel_x = f64::from(read.get_i16_be()?) / VELOCITY_FACTOR;
        let vel_y = f64::from(read.get_i16_be()?) / VELOCITY_FACTOR;
        let vel_z = f64::from(read.get_i16_be()?) / VELOCITY_FACTOR;
        let velocity = LpVector3d(Vector3::new(vel_x, vel_y, vel_z));

        let metadata = if v1_15 {
            None
        } else {
            let mut meta_bytes = Vec::new();
            read.read_to_end(&mut meta_bytes)
                .map_err(|e| ReadingError::Message(e.to_string()))?;
            if meta_bytes.is_empty() {
                None
            } else {
                Some(meta_bytes.into_boxed_slice())
            }
        };

        Ok(Self {
            entity_id,
            entity_uuid,
            r#type,
            position,
            yaw,
            pitch,
            head_yaw,
            velocity,
            metadata,
        })
    }
}

impl<'a> ServerPacket<'a> for CSpawnLivingEntity {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Self::read_packet_data(bytebuf, version)
    }
}

impl ClientPacket for CSpawnLivingEntity {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let v1_9 = *version >= JavaMinecraftVersion::V_1_9;
        let v1_11 = *version >= JavaMinecraftVersion::V_1_11;
        let v1_15 = *version >= JavaMinecraftVersion::V_1_15;

        write.write_var_int(&self.entity_id)?;

        let remapped_type = remap_living_mob_type_for_version(self.r#type.0 as u16, *version);

        if v1_9 {
            write.write_uuid(&self.entity_uuid)?;
            if v1_11 {
                write.write_var_int(&VarInt(remapped_type as i32))?;
            } else {
                write.write_u8(remapped_type as u8)?;
            }
            write.write_f64_be(self.position.x)?;
            write.write_f64_be(self.position.y)?;
            write.write_f64_be(self.position.z)?;
        } else {
            write.write_u8(remapped_type as u8)?;
            write.write_i32_be((self.position.x * 32.0).floor() as i32)?;
            write.write_i32_be((self.position.y * 32.0).floor() as i32)?;
            write.write_i32_be((self.position.z * 32.0).floor() as i32)?;
        }

        write.write_u8(self.yaw)?;
        write.write_u8(self.pitch)?;
        write.write_u8(self.head_yaw)?;

        self.velocity.write_legacy(&mut write)?;

        if !v1_15 {
            if let Some(metadata) = &self.metadata {
                write.write_slice(metadata)?;
            } else if v1_9 {
                write.write_u8(0xFF)?;
            } else {
                // In <= 1.8 (specifically 1.7.10), DataWatcher requires at least one entry
                // so that `readWatchedObjectsFromPacketBuffer` returns a non-null List,
                // avoiding a NullPointerException in `S0FPacketSpawnMob.func_149027_c()`.
                // (0 << 5) | 0 = 0 (type: byte, index: 0 flags), 0 (value), 127 (terminator)
                write.write_slice(&[0x00, 0x00, 127])?;
            }
        }

        Ok(())
    }
}

use std::io::Write;

use pumpkin_data::{
    packet::clientbound::play::LEVEL_PARTICLES, particle_id_remap::remap_particle_id_for_version,
};
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};

/// Spawns a cluster of particles at a specific location.
///
/// This is the most versatile visual packet in the protocol. It allows for
/// precise control over particle density, spread, and speed. It can also
/// carry extra data for complex particles like redstone dust (color) or
/// block/item breaking (textures).
#[java_packet(LEVEL_PARTICLES)]
#[derive(Clone, Debug, PartialEq)]
pub struct CParticle<'a> {
    /// If true, the particle renders even if the client's "Particles"
    /// setting is set to "Minimal".
    pub force_spawn: bool,
    /// If true, the distance at which particles are visible is significantly
    /// increased (from 256 to 65536 blocks). Often used for massive events.
    pub important: bool,
    /// The absolute center position of the particle cluster.
    pub position: Vector3<f64>,
    /// The maximum distance from the center that particles can spawn.
    pub offset: Vector3<f32>,
    /// The velocity or "spread" speed of the particles.
    pub max_speed: f32,
    /// The total number of particles to spawn in this cluster.
    pub particle_count: i32,
    /// The ID of the particle type (e.g., `minecraft:flame`).
    pub particle_id: VarInt,
    /// Extra data required by specific particles (e.g., block states for
    /// `block` particles or RGB values for `dust`).
    pub data: &'a [u8],
}

impl<'a> CParticle<'a> {
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        force_spawn: bool,
        important: bool,
        position: Vector3<f64>,
        offset: Vector3<f32>,
        max_speed: f32,
        particle_count: i32,
        particle_id: VarInt,
        data: &'a [u8],
    ) -> Self {
        Self {
            force_spawn,
            important,
            position,
            offset,
            max_speed,
            particle_count,
            particle_id,
            data,
        }
    }
}

#[must_use]
pub const fn particle_name_for_v1_7(particle: pumpkin_data::particle::Particle) -> &'static str {
    use pumpkin_data::particle::Particle::{
        AngryVillager, Block, BlockCrumble, BlockMarker, Bubble, BubbleColumnUp, BubblePop,
        CampfireSignalSmoke, Cloud, Composter, Crit, DamageIndicator, DragonBreath,
        DrippingDripstoneLava, DrippingDripstoneWater, DrippingLava, DrippingWater, Dust,
        DustColorTransition, DustPillar, DustPlume, Effect, Enchant, EnchantedHit, EntityEffect,
        Explosion, ExplosionEmitter, FallingDust, Firework, Fishing, Flame, HappyVillager, Heart,
        InstantEffect, Item, ItemSlime, ItemSnowball, LargeSmoke, Lava, Mycelium, Note, Poof,
        Portal, Rain, ReversePortal, SmallFlame, Snowflake, SoulFireFlame, Splash, SweepAttack,
        TotemOfUndying, Underwater, Witch,
    };
    match particle {
        ExplosionEmitter => "hugeexplosion",
        Explosion => "largeexplode",
        Poof => "explode",
        Firework => "fireworksSpark",
        Bubble | BubblePop | BubbleColumnUp => "bubble",
        Splash => "splash",
        Fishing => "wake",
        Underwater => "suspended",
        Crit | DamageIndicator | SweepAttack => "crit",
        EnchantedHit => "magicCrit",
        LargeSmoke | CampfireSignalSmoke => "largesmoke",
        InstantEffect => "spell",
        EntityEffect => "mobSpell",
        Effect => "mobSpellAmbient",
        Witch | TotemOfUndying | DragonBreath => "witchMagic",
        DrippingWater | DrippingDripstoneWater => "dripWater",
        DrippingLava | DrippingDripstoneLava => "dripLava",
        AngryVillager => "angryVillager",
        HappyVillager | Composter => "happyVillager",
        Mycelium => "townaura",
        Note => "note",
        Portal | ReversePortal => "portal",
        Enchant => "enchantmenttable",
        Flame | SmallFlame | SoulFireFlame => "flame",
        Lava => "lava",
        Cloud => "cloud",
        Dust | DustColorTransition | DustPillar | DustPlume => "reddust",
        ItemSnowball | Snowflake => "snowballpoof",
        ItemSlime => "slime",
        Heart => "heart",
        BlockMarker => "barrier",
        Rain => "droplet",
        Item => "iconcrack_",
        Block | BlockCrumble => "blockcrack_",
        FallingDust => "blockdust_",
        _ => "smoke",
    }
}

#[must_use]
pub fn particle_id_from_1_7_name(name: &str) -> i32 {
    match name {
        "explode" => pumpkin_data::particle::Particle::Poof as i32,
        "largeexplode" => pumpkin_data::particle::Particle::Explosion as i32,
        "hugeexplosion" => pumpkin_data::particle::Particle::ExplosionEmitter as i32,
        "fireworksSpark" => pumpkin_data::particle::Particle::Firework as i32,
        "bubble" => pumpkin_data::particle::Particle::Bubble as i32,
        "splash" => pumpkin_data::particle::Particle::Splash as i32,
        "wake" => pumpkin_data::particle::Particle::Fishing as i32,
        "suspended" | "depthsuspend" => pumpkin_data::particle::Particle::Underwater as i32,
        "crit" => pumpkin_data::particle::Particle::Crit as i32,
        "magicCrit" => pumpkin_data::particle::Particle::EnchantedHit as i32,
        "smoke" => pumpkin_data::particle::Particle::Smoke as i32,
        "largesmoke" => pumpkin_data::particle::Particle::LargeSmoke as i32,
        "spell" | "instantSpell" => pumpkin_data::particle::Particle::InstantEffect as i32,
        "mobSpell" => pumpkin_data::particle::Particle::EntityEffect as i32,
        "mobSpellAmbient" => pumpkin_data::particle::Particle::Effect as i32,
        "witchMagic" => pumpkin_data::particle::Particle::Witch as i32,
        "dripWater" => pumpkin_data::particle::Particle::DrippingWater as i32,
        "dripLava" => pumpkin_data::particle::Particle::DrippingLava as i32,
        "angryVillager" => pumpkin_data::particle::Particle::AngryVillager as i32,
        "happyVillager" => pumpkin_data::particle::Particle::HappyVillager as i32,
        "townaura" => pumpkin_data::particle::Particle::Mycelium as i32,
        "note" => pumpkin_data::particle::Particle::Note as i32,
        "portal" => pumpkin_data::particle::Particle::Portal as i32,
        "enchantmenttable" => pumpkin_data::particle::Particle::Enchant as i32,
        "flame" => pumpkin_data::particle::Particle::Flame as i32,
        "lava" => pumpkin_data::particle::Particle::Lava as i32,
        "cloud" => pumpkin_data::particle::Particle::Cloud as i32,
        "reddust" => pumpkin_data::particle::Particle::Dust as i32,
        "snowballpoof" | "snowshovel" => pumpkin_data::particle::Particle::ItemSnowball as i32,
        "slime" => pumpkin_data::particle::Particle::ItemSlime as i32,
        "heart" => pumpkin_data::particle::Particle::Heart as i32,
        "barrier" => pumpkin_data::particle::Particle::BlockMarker as i32,
        "droplet" => pumpkin_data::particle::Particle::Rain as i32,
        _ => pumpkin_data::particle::Particle::from_name(name).map_or(0, |p| p as i32),
    }
}

pub(super) fn particle_id_for_version(
    particle_id: VarInt,
    version: JavaMinecraftVersion,
) -> VarInt {
    u16::try_from(particle_id.0).map_or(particle_id, |particle_id| {
        VarInt(i32::from(remap_particle_id_for_version(
            particle_id,
            version,
        )))
    })
}

impl ClientPacket for CParticle<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        if *version <= JavaMinecraftVersion::V_1_7_6 {
            let name = pumpkin_data::particle::Particle::from_id(self.particle_id.0 as u16)
                .map_or("smoke", particle_name_for_v1_7);
            write.write_string_bounded(name, 64)?;
        } else if *version < JavaMinecraftVersion::V_1_20_5 {
            let remapped_id =
                remap_particle_id_for_version(self.particle_id.0 as u16, *version) as i32;
            if *version >= JavaMinecraftVersion::V_1_19 {
                write.write_var_int(&VarInt(remapped_id))?;
            } else {
                write.write_i32_be(remapped_id)?;
            }
        }

        if *version >= JavaMinecraftVersion::V_1_8 {
            write.write_bool(self.important)?;
        }
        if *version >= JavaMinecraftVersion::V_1_21_4 {
            write.write_bool(self.force_spawn)?;
        }

        if *version >= JavaMinecraftVersion::V_1_15 {
            write.write_f64_be(self.position.x)?;
            write.write_f64_be(self.position.y)?;
            write.write_f64_be(self.position.z)?;
        } else {
            write.write_f32_be(self.position.x as f32)?;
            write.write_f32_be(self.position.y as f32)?;
            write.write_f32_be(self.position.z as f32)?;
        }

        write.write_f32_be(self.offset.x)?;
        write.write_f32_be(self.offset.y)?;
        write.write_f32_be(self.offset.z)?;

        write.write_f32_be(self.max_speed)?;
        write.write_i32_be(self.particle_count)?;

        if *version >= JavaMinecraftVersion::V_1_20_5 {
            let remapped_id =
                remap_particle_id_for_version(self.particle_id.0 as u16, *version) as i32;
            write.write_var_int(&VarInt(remapped_id))?;
        }
        write.write_slice(self.data)?;

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CParticle<'a> {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let (particle_id, important, force_spawn) = if *version <= JavaMinecraftVersion::V_1_7_6 {
            let name = bytebuf.get_str_bounded_borrowed(64)?;
            let id = particle_id_from_1_7_name(name);
            (VarInt(id), false, false)
        } else if *version < JavaMinecraftVersion::V_1_20_5 {
            let id = if *version >= JavaMinecraftVersion::V_1_19 {
                bytebuf.get_var_int()?
            } else {
                VarInt(bytebuf.get_i32_be()?)
            };
            let important = bytebuf.get_bool()?;
            (id, important, false)
        } else {
            let important = bytebuf.get_bool()?;
            let force_spawn = if *version >= JavaMinecraftVersion::V_1_21_4 {
                bytebuf.get_bool()?
            } else {
                false
            };
            (VarInt(0), important, force_spawn)
        };

        let position = if *version >= JavaMinecraftVersion::V_1_15 {
            Vector3::new(
                bytebuf.get_f64_be()?,
                bytebuf.get_f64_be()?,
                bytebuf.get_f64_be()?,
            )
        } else {
            Vector3::new(
                f64::from(bytebuf.get_f32_be()?),
                f64::from(bytebuf.get_f32_be()?),
                f64::from(bytebuf.get_f32_be()?),
            )
        };

        let offset = Vector3::new(
            bytebuf.get_f32_be()?,
            bytebuf.get_f32_be()?,
            bytebuf.get_f32_be()?,
        );
        let max_speed = bytebuf.get_f32_be()?;
        let particle_count = bytebuf.get_i32_be()?;

        let (particle_id, data) = if *version >= JavaMinecraftVersion::V_1_20_5 {
            let id = bytebuf.get_var_int()?;
            let remaining = bytebuf.read_remaining_slice_borrowed(usize::MAX)?;
            (id, remaining)
        } else {
            let remaining = bytebuf.read_remaining_slice_borrowed(usize::MAX)?;
            (particle_id, remaining)
        };

        Ok(Self {
            force_spawn,
            important,
            position,
            offset,
            max_speed,
            particle_count,
            particle_id,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Seek, SeekFrom};

    use pumpkin_data::particle::Particle;
    use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

    use crate::{ClientPacket, VarInt};

    use super::CParticle;

    fn encoded_particle_id(version: JavaMinecraftVersion) -> VarInt {
        let packet = CParticle::new(
            false,
            false,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            0.0,
            1,
            VarInt(Particle::ExplosionEmitter as i32),
            &[],
        );
        let mut bytes = Vec::new();
        packet.write_packet_data(&mut bytes, &version).unwrap();

        let mut cursor = Cursor::new(bytes);
        cursor.seek(SeekFrom::Start(46)).unwrap();
        VarInt::decode(&mut cursor).unwrap()
    }

    #[test]
    fn particle_id_remaps_for_1_21_11() {
        assert_eq!(
            encoded_particle_id(JavaMinecraftVersion::V_1_21_11),
            VarInt(22)
        );
    }

    #[test]
    fn particle_id_stays_latest_for_26_2() {
        assert_eq!(
            encoded_particle_id(JavaMinecraftVersion::V_26_2),
            VarInt(29)
        );
    }

    #[test]
    fn particle_encoding_legacy_1_7_string() {
        use crate::ser::NetworkReadSliceExt;

        let packet = CParticle::new(
            false,
            false,
            Vector3::new(1.0, 2.0, 3.0),
            Vector3::new(0.1, 0.2, 0.3),
            0.5,
            10,
            VarInt(Particle::ExplosionEmitter as i32),
            &[],
        );
        let mut bytes = Vec::new();
        packet
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_1_7_6)
            .unwrap();

        let mut slice = bytes.as_slice();
        let name = slice.get_str_borrowed().unwrap();
        assert_eq!(name, "hugeexplosion");
    }

    #[test]
    fn particle_encoding_1_8_int_id() {
        use crate::ser::NetworkReadExt;

        let packet = CParticle::new(
            false,
            false,
            Vector3::new(1.0, 2.0, 3.0),
            Vector3::new(0.1, 0.2, 0.3),
            0.5,
            10,
            VarInt(Particle::ExplosionEmitter as i32),
            &[],
        );
        let mut bytes = Vec::new();
        packet
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_1_8)
            .unwrap();

        let mut slice = bytes.as_slice();
        let id = slice.get_i32_be().unwrap();
        assert_eq!(id, 18);
    }

    #[test]
    fn particle_encoding_1_19_varint_id() {
        let packet = CParticle::new(
            false,
            false,
            Vector3::new(1.0, 2.0, 3.0),
            Vector3::new(0.1, 0.2, 0.3),
            0.5,
            10,
            VarInt(Particle::ExplosionEmitter as i32),
            &[],
        );
        let mut bytes = Vec::new();
        packet
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_1_19)
            .unwrap();

        let mut cursor = Cursor::new(bytes);
        let id = VarInt::decode(&mut cursor).unwrap();
        assert_eq!(id, VarInt(22));
    }
}

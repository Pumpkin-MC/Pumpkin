use std::io::Write;

use pumpkin_data::packet::clientbound::play::LEVEL_PARTICLES;
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

impl ClientPacket for CParticle<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        // The particle moved back to the front of the packet in 26.3
        write.write_var_int(&self.particle_id)?;
        write.write_slice(self.data)?;

        write.write_bool(self.important)?;
        write.write_bool(self.force_spawn)?;

        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;

        write.write_f32_be(self.offset.x)?;
        write.write_f32_be(self.offset.y)?;
        write.write_f32_be(self.offset.z)?;

        write.write_f32_be(self.max_speed)?;
        // Since 26.3 the speed is set per axis and the count is a var int, followed by the
        // randomization type, 0 being the default one.
        write.write_f32_be(self.max_speed)?;
        write.write_f32_be(self.max_speed)?;
        write.write_var_int(&VarInt(self.particle_count))?;
        write.write_var_int(&VarInt(0))?;

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CParticle<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let (_particle_id, important, force_spawn) = {
            let important = bytebuf.get_bool()?;
            let force_spawn = bytebuf.get_bool()?;
            (VarInt(0), important, force_spawn)
        };

        let position = Vector3::new(
            bytebuf.get_f64_be()?,
            bytebuf.get_f64_be()?,
            bytebuf.get_f64_be()?,
        );

        let offset = Vector3::new(
            bytebuf.get_f32_be()?,
            bytebuf.get_f32_be()?,
            bytebuf.get_f32_be()?,
        );
        let max_speed = bytebuf.get_f32_be()?;
        let particle_count = bytebuf.get_i32_be()?;

        let (particle_id, data) = {
            let id = bytebuf.get_var_int()?;
            let remaining = bytebuf.read_remaining_slice_borrowed(usize::MAX)?;
            (id, remaining)
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
mod tests {}

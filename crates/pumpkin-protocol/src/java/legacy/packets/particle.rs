//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/particle.rs`.

use crate::java::client::play::CParticle;
use crate::java::legacy::LegacyWrite;
use crate::{
    VarInt,
    ser::{NetworkWriteExt, WritingError},
};
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Write;

impl LegacyWrite for CParticle<'_> {
    fn write_legacy(
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
            if *version >= JavaMinecraftVersion::V_1_19 {
                write.write_var_int(&self.particle_id)?;
            } else {
                write.write_i32_be(self.particle_id.0)?;
            }
        } else if *version >= JavaMinecraftVersion::V_26_3 {
            // The particle moved back to the front of the packet in 26.3
            write.write_var_int(&self.particle_id)?;
            write.write_slice(self.data)?;
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
        if *version >= JavaMinecraftVersion::V_26_3 {
            // Since 26.3 the speed is set per axis and the count is a var int, followed by the
            // randomization type, 0 being the default one.
            write.write_f32_be(self.max_speed)?;
            write.write_f32_be(self.max_speed)?;
            write.write_var_int(&VarInt(self.particle_count))?;
            write.write_var_int(&VarInt(0))?;
            return Ok(());
        }
        write.write_i32_be(self.particle_count)?;

        if *version >= JavaMinecraftVersion::V_1_20_5 {
            write.write_var_int(&self.particle_id)?;
        }
        write.write_slice(self.data)?;

        Ok(())
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

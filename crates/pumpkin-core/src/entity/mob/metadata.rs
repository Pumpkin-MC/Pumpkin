use pumpkin_data::dye_color::DyeColor;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_data::tracked_data::{self, TrackedData};
use pumpkin_protocol::VarInt;
use pumpkin_protocol::bedrock::client::set_actor_data::{
    MetadataValue, SyncedActorDataList, entity_data_flag, entity_data_key,
};
use pumpkin_protocol::java::client::play::{CSetEntityMetadata, Metadata};
use pumpkin_util::version::JavaMinecraftVersion;

use crate::entity::Entity;
use crate::entity::passive::villager::VillagerData;

/// A single typed Java metadata property ready for binary serialization.
#[derive(Clone)]
pub enum JavaMetadataEntry {
    Bool(TrackedData, bool),
    Byte(TrackedData, i8),
    VarInt(TrackedData, i32),
    Int(TrackedData, i32),
    Float(TrackedData, f32),
    VillagerData(TrackedData, VillagerData),
}

impl JavaMetadataEntry {
    pub fn write(&self, writer: &mut Vec<u8>, version: &JavaMinecraftVersion) {
        match self {
            Self::Bool(tracked, val) => {
                let _ = Metadata::new(*tracked, *val).write(writer, version);
            }
            Self::Byte(tracked, val) => {
                let _ = Metadata::new(*tracked, *val).write(writer, version);
            }
            Self::VarInt(tracked, val) => {
                let _ = Metadata::new(*tracked, VarInt(*val)).write(writer, version);
            }
            Self::Int(tracked, val) => {
                let _ = Metadata::new(*tracked, *val).write(writer, version);
            }
            Self::Float(tracked, val) => {
                let _ = Metadata::new(*tracked, *val).write(writer, version);
            }
            Self::VillagerData(tracked, val) => {
                let _ = Metadata::new(*tracked, *val).write(writer, version);
            }
        }
    }
}

/// Unified mob metadata container that abstracts entity visual properties across
/// Java Edition (tracked entity data) and Bedrock Edition (`SyncedActorDataList`).
#[derive(Clone, Default)]
pub struct MobMetadata {
    bedrock_data: SyncedActorDataList,
    java_entries: Vec<JavaMetadataEntry>,
}

impl MobMetadata {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether the mob is a baby.
    /// Updates Java `ageable_mob::DATA_BABY_ID` and Bedrock `BABY` flag + `SCALE` (0.55).
    #[must_use]
    pub fn baby(mut self, is_baby: bool) -> Self {
        if is_baby {
            self.bedrock_data
                .set_actor_flag(entity_data_flag::BABY, true);
            self.bedrock_data
                .set(entity_data_key::SCALE, MetadataValue::Float(0.55));
            self.java_entries.push(JavaMetadataEntry::Bool(
                tracked_data::ageable_mob::DATA_BABY_ID,
                true,
            ));
        }
        self
    }

    /// Sets a custom scale for Bedrock clients.
    #[must_use]
    pub fn scale(mut self, scale: f32) -> Self {
        self.bedrock_data
            .set(entity_data_key::SCALE, MetadataValue::Float(scale));
        self
    }

    /// Sets whether the mob has AI disabled.
    #[must_use]
    pub fn no_ai(mut self, no_ai: bool) -> Self {
        if no_ai {
            self.bedrock_data
                .set_actor_flag(entity_data_flag::NO_AI, true);
            self.java_entries.push(JavaMetadataEntry::Byte(
                tracked_data::mob::DATA_MOB_FLAGS,
                1,
            ));
        }
        self
    }

    /// Sets whether the mob is saddled (pigs, striders, horses, camels).
    #[must_use]
    pub fn saddled(mut self, saddled: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::SADDLED, saddled);
        self
    }

    /// Sets whether the mob is tamed (wolves, cats, parrots).
    #[must_use]
    pub fn tamed(mut self, tamed: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::TAMED, tamed);
        if tamed {
            self.java_entries.push(JavaMetadataEntry::Byte(
                tracked_data::tamable_animal::DATA_FLAGS_ID,
                4,
            ));
        }
        self
    }

    /// Sets whether the mob is sitting (tamed wolves, cats, parrots).
    #[must_use]
    pub fn sitting(mut self, sitting: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::SITTING, sitting);
        self
    }

    /// Sets whether the mob is interested / begging (wolves).
    #[must_use]
    pub fn interested(mut self, interested: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::INTERESTED, interested);
        self.java_entries.push(JavaMetadataEntry::Bool(
            tracked_data::wolf::INTERESTED_ID,
            interested,
        ));
        self
    }

    /// Sets whether the mob is angry (wolves, endermen).
    #[must_use]
    pub fn angry(mut self, angry: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::ANGRY, angry);
        self
    }

    /// Sets the collar color for tamed mobs (wolves, cats). Each species keeps it in its own
    /// `tracked` slot.
    #[must_use]
    pub fn collar_color(mut self, tracked: TrackedData, color: DyeColor) -> Self {
        self.bedrock_data.set(
            entity_data_key::COLOR_INDEX,
            MetadataValue::Byte(color.id() as i8),
        );
        self.java_entries
            .push(JavaMetadataEntry::VarInt(tracked, color.id() as i32));
        self
    }

    /// Sets sheep wool color and sheared state.
    #[must_use]
    pub fn sheep_wool(mut self, color: DyeColor, is_sheared: bool) -> Self {
        self.bedrock_data.set(
            entity_data_key::COLOR_INDEX,
            MetadataValue::Byte(color.id() as i8),
        );
        self.bedrock_data
            .set_actor_flag(entity_data_flag::SHEARED, is_sheared);
        let packed_byte = (if is_sheared { 0x10 } else { 0 }) | (color.id() & 0x0F);
        self.java_entries.push(JavaMetadataEntry::Byte(
            tracked_data::sheep::WOOL_ID,
            packed_byte as i8,
        ));
        self
    }

    /// Sets creeper charged/powered state.
    #[must_use]
    pub fn charged(mut self, charged: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::POWERED, charged);
        self.bedrock_data
            .set_actor_flag(entity_data_flag::CHARGED, charged);
        self.java_entries.push(JavaMetadataEntry::Bool(
            tracked_data::creeper::CHARGED,
            charged,
        ));
        self
    }

    /// Sets creeper ignited state.
    #[must_use]
    pub fn ignited(mut self, ignited: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::IGNITED, ignited);
        self.java_entries.push(JavaMetadataEntry::Bool(
            tracked_data::creeper::IS_IGNITED,
            ignited,
        ));
        self
    }

    /// Sets creeper fuse speed (-1 idle, 1 counting down).
    #[must_use]
    pub fn fuse_speed(mut self, fuse_speed: i32) -> Self {
        self.bedrock_data.set(
            entity_data_key::SWELL_DIRECTION,
            MetadataValue::Int(fuse_speed),
        );
        self.java_entries.push(JavaMetadataEntry::VarInt(
            tracked_data::creeper::DATA_SWELL_DIR,
            fuse_speed,
        ));
        self
    }

    /// Sets slime / magma cube size.
    #[must_use]
    pub fn slime_size(mut self, size: i32) -> Self {
        self.bedrock_data
            .set(entity_data_key::SCALE, MetadataValue::Float(size as f32));
        self.java_entries
            .push(JavaMetadataEntry::Int(tracked_data::slime::ID_SIZE, size));
        self
    }

    /// Sets villager data and experience using villager tracked data.
    #[must_use]
    pub fn villager_data(self, data: VillagerData, xp: i32) -> Self {
        self.villager_data_with_tracked(tracked_data::villager::DATA_VILLAGER_DATA, data, xp)
    }

    /// Sets zombie villager data and experience using zombie villager tracked data.
    #[must_use]
    pub fn zombie_villager_data(self, data: VillagerData, xp: i32) -> Self {
        self.villager_data_with_tracked(tracked_data::zombie_villager::DATA_VILLAGER_DATA, data, xp)
    }

    /// Sets villager / zombie villager data and experience with specified tracked data key.
    #[must_use]
    pub fn villager_data_with_tracked(
        mut self,
        tracked: TrackedData,
        data: VillagerData,
        xp: i32,
    ) -> Self {
        const PROFESSIONS: [i32; 15] = [0, 8, 11, 6, 7, 1, 2, 4, 12, 5, 13, 14, 3, 10, 9];
        const REGIONS: [i32; 7] = [1, 2, 0, 3, 4, 5, 6];

        let profession_id = usize::try_from(data.profession.0)
            .ok()
            .and_then(|id| PROFESSIONS.get(id))
            .copied()
            .unwrap_or_default();
        let region_id = usize::try_from(data.r#type.0)
            .ok()
            .and_then(|id| REGIONS.get(id))
            .copied()
            .unwrap_or_default();

        self.bedrock_data
            .set(entity_data_key::VARIANT, MetadataValue::Int(profession_id));
        self.bedrock_data
            .set(entity_data_key::MARK_VARIANT, MetadataValue::Int(region_id));
        self.bedrock_data.set(
            entity_data_key::TRADE_TIER,
            MetadataValue::Int(data.level.0.saturating_sub(1)),
        );
        self.bedrock_data
            .set(entity_data_key::MAX_TRADE_TIER, MetadataValue::Int(4));
        self.bedrock_data
            .set(entity_data_key::TRADE_EXPERIENCE, MetadataValue::Int(xp));

        self.java_entries
            .push(JavaMetadataEntry::VillagerData(tracked, data));
        self
    }

    /// Sets whether a zombie villager is converting.
    #[must_use]
    pub fn converting(mut self, converting: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::CONVERTING, converting);
        self.bedrock_data
            .set_actor_flag(entity_data_flag::SHAKING, converting);
        self.java_entries.push(JavaMetadataEntry::Bool(
            tracked_data::zombie_villager::DATA_CONVERTING_ID,
            converting,
        ));
        self
    }

    /// Sets enderman screaming state.
    #[must_use]
    pub fn screaming(mut self, screaming: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::ANGRY, screaming);
        self.java_entries.push(JavaMetadataEntry::Bool(
            tracked_data::enderman::DATA_CREEPY,
            screaming,
        ));
        self
    }

    /// Sets bat roosting state.
    #[must_use]
    pub fn bat_roosting(mut self, roosting: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::RESTING, roosting);
        self.java_entries.push(JavaMetadataEntry::Byte(
            tracked_data::bat::DATA_ID_FLAGS,
            i8::from(roosting),
        ));
        self
    }

    /// Sets blaze on fire / charged state.
    #[must_use]
    pub fn blaze_on_fire(mut self, on_fire: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::CHARGED, on_fire);
        self.bedrock_data
            .set_actor_flag(entity_data_flag::ON_FIRE, on_fire);
        self.java_entries.push(JavaMetadataEntry::Byte(
            tracked_data::blaze::DATA_FLAGS_ID,
            i8::from(on_fire),
        ));
        self
    }

    /// Sets spider climbing state.
    #[must_use]
    pub fn spider_climbing(mut self, climbing: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::WALL_CLIMBING, climbing);
        self.java_entries.push(JavaMetadataEntry::Byte(
            tracked_data::spider::DATA_FLAGS_ID,
            i8::from(climbing),
        ));
        self
    }

    /// Sets iron golem crackiness level (0 to 3).
    #[must_use]
    pub fn iron_golem_crack(mut self, crack_level: u8) -> Self {
        let health_ratio = match crack_level {
            0 => 100,
            1 => 75,
            2 => 50,
            _ => 25,
        };
        self.bedrock_data.set(
            entity_data_key::STRUCTURAL_INTEGRITY,
            MetadataValue::Int(health_ratio),
        );
        self
    }

    /// Sets chested status (donkeys, mules, llamas).
    #[must_use]
    pub fn chested(mut self, chested: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::CHESTED, chested);
        self.java_entries.push(JavaMetadataEntry::Bool(
            tracked_data::trader_llama::DATA_ID_CHEST,
            chested,
        ));
        self
    }

    /// Sets shivering / shaking status (striders out of lava, skeletons in powder snow).
    #[must_use]
    pub fn shivering(mut self, shivering: bool) -> Self {
        self.bedrock_data
            .set_actor_flag(entity_data_flag::SHAKING, shivering);
        self.java_entries.push(JavaMetadataEntry::Bool(
            tracked_data::strider::SUFFOCATING,
            shivering,
        ));
        self
    }

    /// Raw Bedrock flag helper.
    #[must_use]
    pub fn with_bedrock_flag(mut self, flag_index: u32, value: bool) -> Self {
        self.bedrock_data.set_actor_flag(flag_index, value);
        self
    }

    /// Raw Bedrock data property helper.
    #[must_use]
    pub fn with_bedrock_data(mut self, key: u32, value: MetadataValue) -> Self {
        self.bedrock_data.set(key, value);
        self
    }

    /// Raw Java metadata entry helper.
    #[must_use]
    pub fn with_java_entry(mut self, entry: JavaMetadataEntry) -> Self {
        self.java_entries.push(entry);
        self
    }

    /// Returns the Bedrock actor data list.
    #[must_use]
    pub fn to_bedrock_actor_data(&self) -> SyncedActorDataList {
        self.bedrock_data.clone()
    }

    /// Serializes all Java metadata entries for the target Java version, ending with 255.
    #[must_use]
    pub fn to_java_spawn_metadata(&self, version: JavaMinecraftVersion) -> Option<Box<[u8]>> {
        if version < JavaMinecraftVersion::V_1_9 || self.java_entries.is_empty() {
            return None;
        }
        let mut buf = Vec::new();
        for entry in &self.java_entries {
            entry.write(&mut buf, &version);
        }
        buf.push(255);
        Some(buf.into_boxed_slice())
    }

    /// Broadcasts the updated metadata to both Java and Bedrock viewers.
    pub fn send_to_viewers(&self, entity: &Entity) {
        if !self.bedrock_data.0.is_empty() {
            entity.send_bedrock_actor_data(&self.bedrock_data);
        }
        if let Some(java_bytes) = self.to_java_spawn_metadata(CURRENT_MC_VERSION) {
            let meta_packet = CSetEntityMetadata::new(entity.entity_id.into(), java_bytes);
            entity
                .world
                .load()
                .broadcast_to_chunk(entity.chunk_pos.load(), &meta_packet);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baby_metadata_generation() {
        let meta = MobMetadata::new().baby(true);
        let bedrock = meta.to_bedrock_actor_data();
        if let Some(MetadataValue::Int64(flags)) = bedrock.0.get(&entity_data_key::FLAGS) {
            assert_eq!(
                *flags & (1i64 << entity_data_flag::BABY),
                1i64 << entity_data_flag::BABY
            );
        } else {
            panic!("FLAGS missing");
        }
        assert_eq!(
            bedrock.0.get(&entity_data_key::SCALE).map(|v| match v {
                MetadataValue::Float(f) => *f,
                _ => 0.0,
            }),
            Some(0.55)
        );

        let java = meta
            .to_java_spawn_metadata(JavaMinecraftVersion::V_26_3)
            .expect("java metadata");
        assert!(!java.is_empty());
        assert_eq!(*java.last().unwrap(), 255);
    }

    #[test]
    fn sheep_wool_metadata_generation() {
        let meta = MobMetadata::new().sheep_wool(DyeColor::Red, true);
        let bedrock = meta.to_bedrock_actor_data();
        assert_eq!(
            bedrock
                .0
                .get(&entity_data_key::COLOR_INDEX)
                .map(|v| match v {
                    MetadataValue::Byte(b) => *b,
                    _ => -1,
                }),
            Some(DyeColor::Red.id() as i8)
        );
        if let Some(MetadataValue::Int64(flags)) = bedrock.0.get(&entity_data_key::FLAGS) {
            assert_eq!(
                *flags & (1i64 << entity_data_flag::SHEARED),
                1i64 << entity_data_flag::SHEARED
            );
        } else {
            panic!("FLAGS missing");
        }
    }

    #[test]
    fn creeper_metadata_generation() {
        let meta = MobMetadata::new().charged(true).ignited(true).fuse_speed(1);
        let bedrock = meta.to_bedrock_actor_data();
        if let Some(MetadataValue::Int64(flags)) = bedrock.0.get(&entity_data_key::FLAGS) {
            assert_eq!(
                *flags & (1i64 << entity_data_flag::CHARGED),
                1i64 << entity_data_flag::CHARGED
            );
            assert_eq!(
                *flags & (1i64 << entity_data_flag::IGNITED),
                1i64 << entity_data_flag::IGNITED
            );
        } else {
            panic!("FLAGS missing");
        }
        assert_eq!(
            bedrock
                .0
                .get(&entity_data_key::SWELL_DIRECTION)
                .map(|v| match v {
                    MetadataValue::Int(i) => *i,
                    _ => 0,
                }),
            Some(1)
        );
    }

    #[test]
    fn slime_size_metadata_generation() {
        let meta = MobMetadata::new().slime_size(4);
        let bedrock = meta.to_bedrock_actor_data();
        assert_eq!(
            bedrock.0.get(&entity_data_key::SCALE).map(|v| match v {
                MetadataValue::Float(f) => *f,
                _ => 0.0,
            }),
            Some(4.0)
        );
    }
}

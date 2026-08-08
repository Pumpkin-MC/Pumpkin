use crossbeam::atomic::AtomicCell;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::entity::Entity;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::EntityBase;
use crate::world::loot::{LootContextParameters, LootTableExt};
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::GameMode;

pub struct VehicleEntity {
    pub entity: Entity,
    pub hurt_time: AtomicI32,
    pub hurt_dir: AtomicI32,
    pub damage: AtomicCell<f32>,
}

impl VehicleEntity {
    pub const fn new(entity: Entity) -> Self {
        Self {
            entity,
            hurt_time: AtomicI32::new(0),
            hurt_dir: AtomicI32::new(1),
            damage: AtomicCell::new(0.0),
        }
    }

    pub fn tick(&self) {
        let current_hurt = self.hurt_time.load(Ordering::Relaxed);
        if current_hurt > 0 {
            self.hurt_time.store(current_hurt - 1, Ordering::Relaxed);
        }

        let current_damage = self.damage.load();
        if current_damage > 0.0 {
            self.damage.store(current_damage - 1.0);
        }
    }

    pub fn set_damage(&self, damage: f32) {
        self.damage.store(damage);
    }

    pub fn get_damage(&self) -> f32 {
        self.damage.load()
    }

    pub fn set_hurt_time(&self, hurt_time: i32) {
        self.hurt_time.store(hurt_time, Ordering::Relaxed);
    }

    pub fn get_hurt_time(&self) -> i32 {
        self.hurt_time.load(Ordering::Relaxed)
    }

    pub fn set_hurt_dir(&self, hurt_dir: i32) {
        self.hurt_dir.store(hurt_dir, Ordering::Relaxed);
    }

    pub fn get_hurt_dir(&self) -> i32 {
        self.hurt_dir.load(Ordering::Relaxed)
    }

    pub fn send_wobble_metadata(&self) {
        self.entity.send_meta_data(
            &[
                Metadata::new(
                    TrackedData::DAMAGE_WOBBLE_TICKS,
                    MetaDataType::INTEGER,
                    VarInt(self.get_hurt_time()),
                ),
                Metadata::new(
                    TrackedData::DAMAGE_WOBBLE_SIDE,
                    MetaDataType::INTEGER,
                    VarInt(self.get_hurt_dir()),
                ),
                Metadata::new(
                    TrackedData::ID_HURT,
                    MetaDataType::INT,
                    VarInt(self.get_hurt_time()),
                ),
                Metadata::new(
                    TrackedData::ID_HURTDIR,
                    MetaDataType::INT,
                    VarInt(self.get_hurt_dir()),
                ),
            ],
            None,
        );
        self.entity.send_meta_data(
            &[
                Metadata::new(
                    TrackedData::DAMAGE_WOBBLE_STRENGTH,
                    MetaDataType::FLOAT,
                    self.get_damage(),
                ),
                Metadata::new(
                    TrackedData::ID_DAMAGE,
                    MetaDataType::FLOAT,
                    self.get_damage(),
                ),
            ],
            None,
        );
    }

    pub async fn kill_and_drop_self(&self) {
        let world = self.entity.world.load();
        let entity_drops = world.level_info.load().game_rules.entity_drops;

        if entity_drops && let Some(loot_table) = &self.entity.entity_type.loot_table {
            let pos = self.entity.block_pos.load();
            let is_raining = world.is_raining().await;
            let is_thundering = world.is_thundering().await;
            let params = LootContextParameters {
                is_raining: Some(is_raining),
                is_thundering: Some(is_thundering),
                world_time: world.level_info.load().day_time as u64,
                ..Default::default()
            };
            for stack in loot_table.get_loot(params) {
                world.drop_stack(&pos, stack).await;
            }
        }

        self.entity.remove().await;
    }

    pub async fn damage_with_context(&self, amount: f32, source: Option<&dyn EntityBase>) -> bool {
        if !self.entity.is_alive() {
            return true;
        }

        let new_strength = self.apply_damage_wobble(amount);

        let is_creative = source
            .and_then(|s| s.get_player())
            .is_some_and(|p| p.gamemode.load() == GameMode::Creative);

        if is_creative || new_strength > 40.0 {
            if is_creative {
                self.entity.remove().await;
            } else {
                self.kill_and_drop_self().await;
            }
        }

        true
    }

    /// Applies the standard minecart damage wobble without destroying the vehicle.
    /// TNT minecarts use this before deciding whether damage primes or breaks them.
    pub fn apply_damage_wobble(&self, amount: f32) -> f32 {
        let current_side = self.get_hurt_dir();
        self.set_hurt_dir(-current_side);
        self.set_hurt_time(10);
        self.entity.velocity_dirty.store(true, Ordering::SeqCst);

        let current_strength = self.get_damage();
        let new_strength = current_strength + amount * 10.0;
        self.set_damage(new_strength);

        self.send_wobble_metadata();
        new_strength
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::meta_data_type::MetaDataType;
    use pumpkin_data::tracked_data::TrackedData;
    use pumpkin_protocol::codec::var_int::VarInt;
    use pumpkin_protocol::java::client::play::Metadata;
    use pumpkin_util::version::JavaMinecraftVersion;

    fn wobble_integer_metadata(version: JavaMinecraftVersion) -> Vec<u8> {
        let mut bytes = Vec::new();
        for metadata in [
            Metadata::new(
                TrackedData::DAMAGE_WOBBLE_TICKS,
                MetaDataType::INTEGER,
                VarInt(10),
            ),
            Metadata::new(
                TrackedData::DAMAGE_WOBBLE_SIDE,
                MetaDataType::INTEGER,
                VarInt(-1),
            ),
            Metadata::new(TrackedData::ID_HURT, MetaDataType::INT, VarInt(10)),
            Metadata::new(TrackedData::ID_HURTDIR, MetaDataType::INT, VarInt(-1)),
        ] {
            metadata.write(&mut bytes, &version).unwrap();
        }
        bytes
    }

    #[test]
    fn wobble_integers_serialize_for_legacy_and_current_clients() {
        let expected = vec![8, 1, 10, 9, 1, 0xff, 0xff, 0xff, 0xff, 0x0f];

        assert_eq!(
            wobble_integer_metadata(JavaMinecraftVersion::V_1_21_11),
            expected
        );
        assert_eq!(
            wobble_integer_metadata(JavaMinecraftVersion::V_26_2),
            expected
        );
    }
}

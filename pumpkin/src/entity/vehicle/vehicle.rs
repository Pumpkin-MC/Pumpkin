use crossbeam::atomic::AtomicCell;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::entity::Entity;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::EntityBase;
use crate::world::loot::{LootContextParameters, LootTableExt};
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
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
        self.entity.send_meta_data(&[
            Metadata::new(
                TrackedData::ID_HURT,
                MetaDataType::INTEGER,
                self.get_hurt_time(),
            ),
            Metadata::new(
                TrackedData::ID_HURTDIR,
                MetaDataType::INTEGER,
                self.get_hurt_dir(),
            ),
        ]);
        self.entity.send_meta_data(&[Metadata::new(
            TrackedData::ID_DAMAGE,
            MetaDataType::FLOAT,
            self.get_damage(),
        )]);
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

        let current_side = self.get_hurt_dir();
        self.set_hurt_dir(-current_side);
        self.set_hurt_time(10);
        self.entity.velocity_dirty.store(true, Ordering::SeqCst);

        let current_strength = self.get_damage();
        let new_strength = current_strength + amount * 10.0;
        self.set_damage(new_strength);

        self.send_wobble_metadata();

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
}

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;

use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity};
use crate::server::Server;
use crate::world::loot::{LootContextParameters, LootTableExt};
use pumpkin_data::damage::DamageType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;

pub struct BoatEntity {
    entity: Entity,
    damage_wobble_ticks: AtomicI32,
    damage_wobble_side: AtomicI32,
    damage_wobble_strength: AtomicCell<f32>,
    ticks_underwater: AtomicCell<f32>,
    left_paddle_moving: AtomicBool,
    right_paddle_moving: AtomicBool,
}

impl BoatEntity {
    pub const fn new(entity: Entity) -> Self {
        Self {
            entity,
            damage_wobble_ticks: AtomicI32::new(0),
            damage_wobble_side: AtomicI32::new(1),
            damage_wobble_strength: AtomicCell::new(0.0),
            ticks_underwater: AtomicCell::new(0.0),
            left_paddle_moving: AtomicBool::new(false),
            right_paddle_moving: AtomicBool::new(false),
        }
    }

    pub async fn set_paddles(&self, left: bool, right: bool) {
        self.left_paddle_moving.store(left, Ordering::Relaxed);
        self.right_paddle_moving.store(right, Ordering::Relaxed);

        self.entity
            .send_meta_data(&[
                Metadata::new(
                    TrackedData::DATA_LEFT_PADDLE_MOVING,
                    MetaDataType::Boolean,
                    left,
                ),
                Metadata::new(
                    TrackedData::DATA_RIGHT_PADDLE_MOVING,
                    MetaDataType::Boolean,
                    right,
                ),
            ])
            .await;
    }

    async fn send_wobble_metadata(&self) {
        self.entity
            .send_meta_data(&[
                Metadata::new(
                    TrackedData::DATA_DAMAGE_WOBBLE_TICKS,
                    MetaDataType::Integer,
                    VarInt(self.damage_wobble_ticks.load(Ordering::Relaxed)),
                ),
                Metadata::new(
                    TrackedData::DATA_DAMAGE_WOBBLE_SIDE,
                    MetaDataType::Integer,
                    VarInt(self.damage_wobble_side.load(Ordering::Relaxed)),
                ),
            ])
            .await;
        self.entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_DAMAGE_WOBBLE_STRENGTH,
                MetaDataType::Float,
                self.damage_wobble_strength.load(),
            )])
            .await;
    }

    async fn kill_and_drop_self(&self) {
        let world = self.entity.world.load();
        let entity_drops = world.level_info.load().game_rules.entity_drops;

        if entity_drops && let Some(loot_table) = &self.entity.entity_type.loot_table {
            let pos = self.entity.block_pos.load();
            let params = LootContextParameters::default();
            for stack in loot_table.get_loot(params) {
                world.drop_stack(&pos, stack).await;
            }
        }

        self.entity.remove().await;
    }
}

impl NBTStorage for BoatEntity {}

impl EntityBase for BoatEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn tick<'a>(
        &'a self,
        _caller: Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let ticks = self.damage_wobble_ticks.load(Ordering::Relaxed);
            if ticks > 0 {
                self.damage_wobble_ticks.store(ticks - 1, Ordering::Relaxed);
            }

            let strength = self.damage_wobble_strength.load();
            if strength > 0.0 {
                self.damage_wobble_strength.store(strength - 1.0);
            }

            let underwater = self.ticks_underwater.load();
            if self.entity.touching_water.load(Ordering::Relaxed) {
                self.ticks_underwater.store((underwater + 1.0).min(60.0));
            } else if underwater > 0.0 {
                self.ticks_underwater.store((underwater - 1.0).max(0.0));
            }
        })
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.send_wobble_metadata().await;
        })
    }

    fn can_hit(&self) -> bool {
        self.entity.is_alive()
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if !self.entity.is_alive() {
                return true;
            }

            let current_side = self.damage_wobble_side.load(Ordering::Relaxed);
            self.damage_wobble_side
                .store(-current_side, Ordering::Relaxed);
            self.damage_wobble_ticks.store(10, Ordering::Relaxed);
            self.entity.velocity_dirty.store(true, Ordering::SeqCst);

            let current_strength = self.damage_wobble_strength.load();
            let new_strength = current_strength + amount * 10.0;
            self.damage_wobble_strength.store(new_strength);

            self.send_wobble_metadata().await;

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
        })
    }

    fn interact<'a>(
        &'a self,
        player: &'a Player,
        _item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if player.living_entity.entity.sneaking.load(Ordering::Relaxed) {
                return false;
            }

            if self.ticks_underwater.load() >= 60.0 {
                return false;
            }

            if self.entity.passengers.lock().await.len() >= 2 {
                return false;
            }

            if player.living_entity.entity.has_vehicle().await {
                return false;
            }

            let world = self.entity.world.load();
            let Some(vehicle) = world.get_entity_by_id(self.entity.entity_id) else {
                return false;
            };

            let Some(passenger) = world.get_player_by_id(player.entity_id()) else {
                return false;
            };

            self.entity
                .add_passenger(vehicle, passenger as Arc<dyn EntityBase>)
                .await;

            true
        })
    }

    fn set_paddle_state(&self, left: bool, right: bool) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.set_paddles(left, right).await;
        })
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }
}

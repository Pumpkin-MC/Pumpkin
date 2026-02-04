use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;

use crate::entity::item::ItemEntity;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity};
use crate::server::Server;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;

pub struct BoatEntity {
    entity: Entity,
    damage_wobble_ticks: AtomicI32,
    damage_wobble_side: AtomicI32,
    damage_wobble_strength: AtomicCell<f32>,
}

impl BoatEntity {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            damage_wobble_ticks: AtomicI32::new(0),
            damage_wobble_side: AtomicI32::new(1),
            damage_wobble_strength: AtomicCell::new(0.0),
        }
    }

    fn entity_to_item(entity_type: &EntityType) -> &'static Item {
        match entity_type.id {
            val if val == EntityType::OAK_BOAT.id => &Item::OAK_BOAT,
            val if val == EntityType::OAK_CHEST_BOAT.id => &Item::OAK_CHEST_BOAT,
            val if val == EntityType::SPRUCE_BOAT.id => &Item::SPRUCE_BOAT,
            val if val == EntityType::SPRUCE_CHEST_BOAT.id => &Item::SPRUCE_CHEST_BOAT,
            val if val == EntityType::BIRCH_BOAT.id => &Item::BIRCH_BOAT,
            val if val == EntityType::BIRCH_CHEST_BOAT.id => &Item::BIRCH_CHEST_BOAT,
            val if val == EntityType::JUNGLE_BOAT.id => &Item::JUNGLE_BOAT,
            val if val == EntityType::JUNGLE_CHEST_BOAT.id => &Item::JUNGLE_CHEST_BOAT,
            val if val == EntityType::ACACIA_BOAT.id => &Item::ACACIA_BOAT,
            val if val == EntityType::ACACIA_CHEST_BOAT.id => &Item::ACACIA_CHEST_BOAT,
            val if val == EntityType::DARK_OAK_BOAT.id => &Item::DARK_OAK_BOAT,
            val if val == EntityType::DARK_OAK_CHEST_BOAT.id => &Item::DARK_OAK_CHEST_BOAT,
            val if val == EntityType::MANGROVE_BOAT.id => &Item::MANGROVE_BOAT,
            val if val == EntityType::MANGROVE_CHEST_BOAT.id => &Item::MANGROVE_CHEST_BOAT,
            val if val == EntityType::CHERRY_BOAT.id => &Item::CHERRY_BOAT,
            val if val == EntityType::CHERRY_CHEST_BOAT.id => &Item::CHERRY_CHEST_BOAT,
            val if val == EntityType::PALE_OAK_BOAT.id => &Item::PALE_OAK_BOAT,
            val if val == EntityType::PALE_OAK_CHEST_BOAT.id => &Item::PALE_OAK_CHEST_BOAT,
            val if val == EntityType::BAMBOO_RAFT.id => &Item::BAMBOO_RAFT,
            val if val == EntityType::BAMBOO_CHEST_RAFT.id => &Item::BAMBOO_CHEST_RAFT,
            _ => &Item::OAK_BOAT,
        }
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
        let pos = self.entity.pos.load();
        let entity_drops = world.level_info.load().game_rules.entity_drops;

        if entity_drops {
            let item = Self::entity_to_item(self.entity.entity_type);
            let item_stack = ItemStack::new(1, item);

            let item_entity = Entity::new(world.clone(), pos, &EntityType::ITEM);
            let item_entity = Arc::new(ItemEntity::new(item_entity, item_stack).await);
            world.spawn_entity(item_entity).await;
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
            self.damage_wobble_side.store(-current_side, Ordering::Relaxed);
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

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }
}

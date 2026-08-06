use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering::Relaxed},
};

use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use rand::{RngExt, rng};

use crate::entity::{
    Entity, EntityBaseFuture, NBTStorage, NbtFuture,
    item::ItemEntity,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
    player::Player,
};
use crate::world::game_event::{GameEventContext, emit_game_event};

/// `Bogged.java` (extends `AbstractSkeleton`, implements `Shearable`).
pub struct BoggedSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
    /// Vanilla `Bogged.DATA_SHEARED` synced data / `sheared` save tag.
    sheared: AtomicBool,
}

impl BoggedSkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity);
        let bogged = Self {
            entity,
            sheared: AtomicBool::new(false),
        };
        Arc::new(bogged)
    }

    pub fn is_sheared(&self) -> bool {
        self.sheared.load(Relaxed)
    }

    pub fn set_sheared(&self, sheared: bool) {
        if self.sheared.swap(sheared, Relaxed) != sheared {
            self.entity.mob_entity.living_entity.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::SHEARED,
                    MetaDataType::BOOLEAN,
                    sheared,
                )],
                None,
            );
        }
    }

    /// Vanilla `Bogged::readyForShearing`: `!isSheared()`.
    pub fn ready_for_shearing(&self) -> bool {
        !self.is_sheared()
    }

    /// Vanilla `Bogged::shear` + `Bogged::spawnShearedMushrooms`: plays the shear sound, drops
    /// mushrooms via the `shearing/bogged` loot table (2 rolls, each an even pick between brown
    /// and red mushroom, 1 item apiece -- no shearing-loot-table infrastructure exists in
    /// `pumpkin-data`, so this is hardcoded like `Sheep::shear` hardcodes its wool drop), and
    /// marks the bogged sheared.
    pub async fn shear(&self, player: &Arc<Player>) {
        let entity = &self.entity.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();

        world.play_sound(Sound::EntityBoggedShear, SoundCategory::Players, &pos);

        for _ in 0..2 {
            let mushroom = if rng().random_bool(0.5) {
                &Item::BROWN_MUSHROOM
            } else {
                &Item::RED_MUSHROOM
            };
            let velocity = Vector3::new(
                (rand::random::<f64>() - 0.5) * 0.1,
                rand::random::<f64>() * 0.05 + 0.2,
                (rand::random::<f64>() - 0.5) * 0.1,
            );
            let drop_pos = pos + Vector3::new(0.0, f64::from(entity.height()), 0.0);
            let item_entity = Arc::new(ItemEntity::new_with_velocity(
                Entity::new(
                    world.clone(),
                    drop_pos,
                    &pumpkin_data::entity::EntityType::ITEM,
                ),
                ItemStack::new(1, mushroom),
                velocity,
                10,
            ));
            world.spawn_entity(item_entity).await;
        }

        self.set_sheared(true);
        emit_game_event(
            &world,
            GameEvent::Shear,
            pos,
            GameEventContext::of_entity(player.clone()),
        )
        .await;
    }
}

impl NBTStorage for BoggedSkeletonEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.entity.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_bool("sheared", self.is_sheared());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.entity
                .mob_entity
                .living_entity
                .read_nbt_non_mut(nbt)
                .await;
            self.sheared
                .store(nbt.get_bool("sheared").unwrap_or(false), Relaxed);
        })
    }
}

impl Mob for BoggedSkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = &self.entity.mob_entity.living_entity.entity;
            if entity.age.load(Relaxed) < 0 {
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::SHEARED,
                    MetaDataType::BOOLEAN,
                    self.is_sheared(),
                )],
                None,
            );
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if item_stack.is_shears() && self.ready_for_shearing() {
                self.shear(player).await;
                let _ = item_stack.damage_item(1);
                return true;
            }

            self.entity
                .mob_entity
                .mob_interact(player, item_stack, self.can_be_leashed())
                .await
        })
    }
}

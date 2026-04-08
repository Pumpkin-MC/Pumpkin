use crate::{entity::EntityBaseFuture, server::Server};
use core::f32;
use pumpkin_data::data_component_impl::DamageResistantImpl;
use pumpkin_data::data_component_impl::DamageResistantType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{damage::DamageType, meta_data_type::MetaDataType, tracked_data::TrackedData};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::{
    codec::item_stack_seralizer::ItemStackSerializer,
    java::client::play::{CTakeItemEntity, Metadata},
};
use pumpkin_util::math::atomic_f32::AtomicF32;
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering::{AcqRel, Relaxed};

use std::sync::{
    Arc,
    atomic::{
        AtomicBool, AtomicU8, AtomicU32,
        Ordering::{self},
    },
};
use tokio::sync::Mutex;

use super::{Entity, EntityBase, NBTStorage, living::LivingEntity, player::Player};

pub struct ItemEntity {
    entity: Entity,
    item_age: AtomicU32,
    // These cannot be atomic values because we mutate their state based on what they are; we run
    // into the ABA problem
    item_stack: Mutex<ItemStack>,
    pickup_delay: AtomicU8,
    health: AtomicF32,
    never_despawn: AtomicBool,
    never_pickup: AtomicBool,
}

impl ItemEntity {
    fn item_stack_is_fire_immune(item_stack: &ItemStack) -> bool {
        item_stack
            .get_data_component::<DamageResistantImpl>()
            .is_some_and(|res| res.res_type == DamageResistantType::Fire)
    }

    fn sync_fire_immunity_from_stack(&self, item_stack: &ItemStack) {
        self.entity.fire_immune.store(
            Self::item_stack_is_fire_immune(item_stack),
            Ordering::Relaxed,
        );
    }

    pub async fn new(entity: Entity, item_stack: ItemStack) -> Self {
        entity
            .set_velocity(Vector3::new(
                rand::random::<f64>().mul_add(0.2, -0.1),
                0.2,
                rand::random::<f64>().mul_add(0.2, -0.1),
            ))
            .await;
        entity.yaw.store(rand::random::<f32>() * 360.0);

        entity.fire_immune.store(
            Self::item_stack_is_fire_immune(&item_stack),
            Ordering::Relaxed,
        );

        Self {
            entity,
            item_stack: Mutex::new(item_stack),
            item_age: AtomicU32::new(0),
            pickup_delay: AtomicU8::new(10), // Vanilla pickup delay is 10 ticks
            health: AtomicF32::new(5.0),
            never_despawn: AtomicBool::new(false),
            never_pickup: AtomicBool::new(false),
        }
    }

    pub async fn new_with_velocity(
        entity: Entity,
        item_stack: ItemStack,
        velocity: Vector3<f64>,
        pickup_delay: u8,
    ) -> Self {
        entity.set_velocity(velocity).await;
        entity.yaw.store(rand::random::<f32>() * 360.0);

        entity.fire_immune.store(
            Self::item_stack_is_fire_immune(&item_stack),
            Ordering::Relaxed,
        );

        Self {
            entity,
            item_stack: Mutex::new(item_stack),
            item_age: AtomicU32::new(0),
            pickup_delay: AtomicU8::new(pickup_delay), // Vanilla pickup delay is 10 ticks
            health: AtomicF32::new(5.0),
            never_despawn: AtomicBool::new(false),
            never_pickup: AtomicBool::new(false),
        }
    }

    async fn can_merge(&self) -> bool {
        if self.never_pickup.load(Ordering::Relaxed) || self.entity.removed.load(Ordering::Relaxed)
        {
            return false;
        }

        let item_stack = self.item_stack.lock().await;

        item_stack.item_count < item_stack.get_max_stack_size()
    }

    async fn try_merge(&self) {
        let bounding_box = self.entity.bounding_box.load().expand(0.5, 0.0, 0.5);

        let world = self.entity.world.load();
        let entities = world.entities.load();
        let items = entities.iter().filter_map(|entity: &Arc<dyn EntityBase>| {
            entity.clone().get_item_entity().filter(|item| {
                item.entity.entity_id != self.entity.entity_id
                    && !item.never_despawn.load(Ordering::Relaxed)
                    && item.entity.bounding_box.load().intersects(&bounding_box)
            })
        });

        for item in items {
            if item.can_merge().await {
                self.try_merge_with(&item).await;

                if self.entity.removed.load(Ordering::SeqCst) {
                    break;
                }
            }
        }
    }

    async fn try_merge_with(&self, other: &Self) {
        let self_stack = self.item_stack.lock().await;

        let other_stack = other.item_stack.lock().await;

        if !self_stack.are_equal(&other_stack)
            || self_stack.item_count + other_stack.item_count > self_stack.get_max_stack_size()
        {
            return;
        }

        let (target, mut stack1, source, mut stack2) =
            if other_stack.item_count < self_stack.item_count {
                (self, self_stack, other, other_stack)
            } else {
                (other, other_stack, self, self_stack)
            };

        // Vanilla code adds a .min(64). Not needed with Vanilla item data

        let max_size = stack1.get_max_stack_size();

        let j = stack2.item_count.min(max_size - stack1.item_count);

        stack1.increment(j);

        stack2.decrement(j);

        let empty1 = stack1.item_count == 0;

        let empty2 = stack2.item_count == 0;

        drop(stack1);

        drop(stack2);

        let never_despawn = source.never_despawn.load(Ordering::Relaxed);

        target.never_despawn.store(never_despawn, Ordering::Relaxed);

        if !never_despawn {
            let age = target
                .item_age
                .load(Ordering::Relaxed)
                .min(source.item_age.load(Ordering::Relaxed));

            target.item_age.store(age, Ordering::Relaxed);
        }

        let never_pickup = source.never_pickup.load(Ordering::Relaxed);

        target.never_pickup.store(never_pickup, Ordering::Relaxed);

        if !never_pickup {
            let source_delay = source.pickup_delay.load(Ordering::Relaxed);
            target
                .pickup_delay
                .fetch_max(source_delay, Ordering::Relaxed);
        }

        if empty1 {
            target.entity.remove().await;
        } else {
            target.init_data_tracker().await;
        }

        if empty2 {
            source.entity.remove().await;
        } else {
            source.init_data_tracker().await;
        }
    }

    fn decrement_pickup_delay(&self) {
        self.pickup_delay
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
                Some(val.saturating_sub(1))
            })
            .ok();
    }

    fn apply_fluid_drag_or_gravity(&self, mut velo: Vector3<f64>) -> Vector3<f64> {
        let entity = &self.entity;

        if entity.touching_water.load(Ordering::SeqCst) && entity.water_height.load() > 0.1 {
            velo.x *= 0.99;
            velo.z *= 0.99;
            if velo.y < 0.06 {
                velo.y += 5.0e-4;
            }
        } else if entity.touching_lava.load(Ordering::SeqCst) && entity.lava_height.load() > 0.1 {
            velo.x *= 0.95;
            velo.z *= 0.95;
            if velo.y < 0.06 {
                velo.y += 5.0e-4;
            }
        } else {
            velo.y -= <Self as EntityBase>::get_gravity(self);
        }

        velo
    }

    async fn update_no_clip_and_push_out(&self) {
        let entity = &self.entity;
        let pos = entity.pos.load();
        let bounding_box = entity.bounding_box.load();

        let no_clip = !entity
            .world
            .load()
            .is_space_empty(bounding_box.expand(-1.0e-7, -1.0e-7, -1.0e-7))
            .await;

        entity.no_clip.store(no_clip, Ordering::Relaxed);

        if no_clip {
            entity
                .push_out_of_blocks(Vector3::new(
                    pos.x,
                    f64::midpoint(bounding_box.min.y, bounding_box.max.y),
                    pos.z,
                ))
                .await;
        }
    }

    async fn should_tick_move(&self, move_velo: Vector3<f64>) -> Option<bool> {
        let entity = &self.entity;

        let mut tick_move = !entity.on_ground.load(Ordering::SeqCst)
            || move_velo.horizontal_length_squared() > 1.0e-5;

        if !tick_move {
            let Ok(item_age) = i32::try_from(self.item_age.load(Ordering::Relaxed)) else {
                entity.remove().await;
                return None;
            };

            tick_move = (item_age + entity.entity_id) % 4 == 0;
        }

        Some(tick_move)
    }

    async fn move_and_apply_friction(
        &self,
        caller: &Arc<dyn EntityBase>,
        server: &Server,
        move_velo: Vector3<f64>,
    ) {
        let entity = &self.entity;

        entity.move_entity(caller.clone(), move_velo).await;
        entity.tick_block_collisions(caller, server).await;

        let mut friction = 0.98;
        let on_ground = entity.on_ground.load(Ordering::SeqCst);

        let mut velo = entity.velocity.load();
        if on_ground {
            let block_affecting_velo = entity.get_block_with_y_offset(0.999_999).await.1;
            friction *= f64::from(block_affecting_velo.slipperiness) * 0.98;
        }

        velo = velo.multiply(friction, 0.98, friction);

        if on_ground && velo.y < 0.0 {
            velo.y = 0.0;
        }

        entity.velocity.store(velo);
    }

    async fn process_age_and_merge(&self) -> bool {
        if self.never_despawn.load(Ordering::Relaxed) {
            return true;
        }

        let entity = &self.entity;
        let age = self.item_age.fetch_add(1, Ordering::Relaxed) + 1;

        if age >= 6000 {
            entity.remove().await;
            return false;
        }

        let n = if entity
            .last_pos
            .load()
            .sub(&entity.pos.load())
            .length_squared()
            == 0.0
        {
            40
        } else {
            2
        };

        if age.is_multiple_of(n) && self.can_merge().await {
            self.try_merge().await;
        }

        true
    }

    async fn sync_motion_if_dirty(
        &self,
        caller: &Arc<dyn EntityBase>,
        original_velo: Vector3<f64>,
    ) {
        let entity = &self.entity;

        entity.update_fluid_state(caller).await;

        let velocity_dirty = entity.velocity_dirty.swap(false, Ordering::SeqCst)
            || entity.touching_water.load(Ordering::SeqCst)
            || entity.touching_lava.load(Ordering::SeqCst)
            || entity.velocity.load().sub(&original_velo).length_squared() > 0.1;

        if velocity_dirty {
            entity.send_pos_rot().await;
            entity.send_velocity().await;
        }
    }
}

impl NBTStorage for ItemEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> super::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.write_nbt(nbt).await;

            let item_stack = self.item_stack.lock().await;
            let mut item_nbt = NbtCompound::new();
            item_stack.write_item_stack(&mut item_nbt);
            nbt.put_compound("Item", item_nbt);

            nbt.put_int("Age", self.item_age.load(Ordering::Relaxed) as i32);
            nbt.put_short(
                "PickupDelay",
                i16::from(self.pickup_delay.load(Ordering::Relaxed)),
            );
            nbt.put("Health", NbtTag::Float(self.health.load(Relaxed)));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> super::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.read_nbt_non_mut(nbt).await;

            if let Some(item_nbt) = nbt.get_compound("Item")
                && let Some(item_stack) = ItemStack::read_item_stack(item_nbt)
            {
                self.sync_fire_immunity_from_stack(&item_stack);
                *self.item_stack.lock().await = item_stack;
            }

            self.item_age.store(
                nbt.get_int("Age").unwrap_or(0).max(0) as u32,
                Ordering::Relaxed,
            );
            self.pickup_delay.store(
                nbt.get_short("PickupDelay")
                    .unwrap_or(0)
                    .clamp(0, u8::MAX as i16) as u8,
                Ordering::Relaxed,
            );
            self.health
                .store(nbt.get_float("Health").unwrap_or(5.0).max(0.0), Relaxed);
        })
    }
}

impl EntityBase for ItemEntity {
    fn tick<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.entity;
            self.decrement_pickup_delay();

            let original_velo = entity.velocity.load();
            entity
                .velocity
                .store(self.apply_fluid_drag_or_gravity(original_velo));

            self.update_no_clip_and_push_out().await;

            let move_velo = entity.velocity.load(); // In case push_out_of_blocks modifies it

            let Some(tick_move) = self.should_tick_move(move_velo).await else {
                return;
            };

            if tick_move {
                self.move_and_apply_friction(&caller, server, move_velo)
                    .await;
            }

            if self.process_age_and_merge().await {
                self.sync_motion_if_dirty(&caller, original_velo).await;
            }
        })
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {
            self.entity
                .send_meta_data(&[Metadata::new(
                    TrackedData::ITEM,
                    MetaDataType::ITEM_STACK,
                    &ItemStackSerializer::from(self.item_stack.lock().await.clone()),
                )])
                .await;
        })
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            // Check if entity is fire_immune
            let is_fire_damage = damage_type == DamageType::IN_FIRE
                || damage_type == DamageType::ON_FIRE
                || damage_type == DamageType::LAVA;
            if is_fire_damage && self.entity.fire_immune.load(Ordering::Relaxed) {
                return false;
            }

            // Thread safe damage application
            loop {
                let current = self.health.load(Relaxed);
                let new = current - amount;
                if self
                    .health
                    .compare_exchange(current, new, AcqRel, Relaxed)
                    .is_ok()
                {
                    if new <= 0.0 {
                        self.entity.remove().await;
                    }
                    return true;
                }
            }
        })
    }

    fn on_player_collision<'a>(&'a self, player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {
            if self.pickup_delay.load(Ordering::Relaxed) > 0
                || player.living_entity.health.load() <= 0.0
                || player.is_spectator()
            {
                return;
            }

            if player
                .inventory
                .insert_stack_anywhere(&mut *self.item_stack.lock().await)
                .await
                || player.is_creative()
            {
                player
                    .client
                    .enqueue_packet(&CTakeItemEntity::new(
                        self.entity.entity_id.into(),
                        player.entity_id().into(),
                        self.item_stack.lock().await.item_count.into(),
                    ))
                    .await;
                player
                    .current_screen_handler
                    .lock()
                    .await
                    .lock()
                    .await
                    .send_content_updates()
                    .await;
                if self.item_stack.lock().await.is_empty() {
                    self.entity.remove().await;
                } else {
                    // Update entity
                    self.init_data_tracker().await;
                }
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn get_item_entity(self: Arc<Self>) -> Option<Arc<ItemEntity>> {
        Some(self)
    }

    fn get_gravity(&self) -> f64 {
        0.04
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::registry::default_registry;
    use crate::world::World;
    use arc_swap::ArcSwap;
    use pumpkin_config::world::LevelConfig;
    use pumpkin_data::{dimension::Dimension, entity::EntityType, item::Item};
    use pumpkin_util::{math::vector3::Vector3, world_seed::Seed};
    use pumpkin_world::{level::Level, world_info::LevelData};
    use tempfile::tempdir;

    fn test_world() -> Arc<World> {
        let temp_dir = tempdir().unwrap();
        let block_registry = default_registry();
        let level = Level::from_root_folder(
            &LevelConfig::default(),
            temp_dir.keep(),
            block_registry.clone(),
            0,
            Dimension::OVERWORLD,
        );
        let level_info = Arc::new(ArcSwap::new(Arc::new(LevelData::default(Seed(0)))));

        Arc::new(World::load(
            level,
            level_info,
            Dimension::OVERWORLD,
            block_registry,
            std::sync::Weak::new(),
        ))
    }

    #[tokio::test]
    async fn item_entity_nbt_round_trips_uuid_and_item_payload() {
        let world = test_world();
        let uuid = uuid::Uuid::new_v4();
        let entity = Entity::from_uuid(
            uuid,
            world.clone(),
            Vector3::new(10.0, 64.0, -5.0),
            &EntityType::ITEM,
        );
        let item_entity = ItemEntity::new_with_velocity(
            entity,
            ItemStack::new(3, &Item::DIAMOND),
            Vector3::new(0.25, 0.5, -0.125),
            7,
        )
        .await;
        item_entity.item_age.store(42, Ordering::Relaxed);
        item_entity.health.store(3.5, Relaxed);

        let mut nbt = NbtCompound::new();
        item_entity.write_nbt(&mut nbt).await;

        assert!(
            nbt.get("UUID").is_some(),
            "item entity nbt should include UUID"
        );
        let item_nbt = nbt
            .get_compound("Item")
            .expect("item entity nbt should include item payload");
        assert_eq!(item_nbt.get_string("id"), Some("minecraft:diamond"));
        assert_eq!(item_nbt.get_int("count"), Some(3));
        assert_eq!(nbt.get_int("Age"), Some(42));
        assert_eq!(nbt.get_short("PickupDelay"), Some(7));
        assert_eq!(nbt.get_float("Health"), Some(3.5));

        let reloaded = ItemEntity::new_with_velocity(
            Entity::from_uuid(uuid, world, Vector3::new(0.0, 0.0, 0.0), &EntityType::ITEM),
            ItemStack::EMPTY.clone(),
            Vector3::new(0.0, 0.0, 0.0),
            0,
        )
        .await;
        reloaded.read_nbt_non_mut(&nbt).await;

        let restored_stack = reloaded.item_stack.lock().await.clone();
        assert_eq!(restored_stack.item.id, Item::DIAMOND.id);
        assert_eq!(restored_stack.item_count, 3);
        assert_eq!(reloaded.item_age.load(Ordering::Relaxed), 42);
        assert_eq!(reloaded.pickup_delay.load(Ordering::Relaxed), 7);
        assert_eq!(reloaded.health.load(Relaxed), 3.5);
        assert_eq!(
            reloaded.entity.entity_uuid, uuid,
            "reloaded item entity should keep the base UUID path"
        );
    }
}

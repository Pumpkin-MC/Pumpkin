use crate::entity::player::statistics::StatisticCategory;
use crate::{entity::EntityBaseFuture, server::Server};
use core::f32;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::DamageResistantImpl;
use pumpkin_data::data_component_impl::DamageResistantType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::client::CAddItemActor;
use pumpkin_protocol::bedrock::network_item::ItemStackWrapper;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::{CSetEntityMetadata, Metadata};
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

use super::{Entity, EntityBase, NBTStorage, NbtFuture, living::LivingEntity, player::Player};

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

const ITEM_UPDATE_INTERVAL: u32 = 20;

impl ItemEntity {
    pub fn new(entity: Entity, item_stack: ItemStack) -> Self {
        entity.velocity.store(Vector3::new(
            rand::random::<f64>().mul_add(0.2, -0.1),
            0.2,
            rand::random::<f64>().mul_add(0.2, -0.1),
        ));
        entity.yaw.store(rand::random::<f32>() * 360.0);

        // Set fire immunity for certain items
        if let Some(res) = item_stack.get_data_component::<DamageResistantImpl>()
            && res.res_type == DamageResistantType::Fire
        {
            entity.fire_immune.store(true, Ordering::Relaxed);
        }

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

    pub fn new_with_velocity(
        entity: Entity,
        item_stack: ItemStack,
        velocity: Vector3<f64>,
        pickup_delay: u8,
    ) -> Self {
        entity.velocity.store(velocity);
        entity.yaw.store(rand::random::<f32>() * 360.0);

        // Set fire immunity for certain items
        if let Some(res) = item_stack.get_data_component::<DamageResistantImpl>()
            && res.res_type == DamageResistantType::Fire
        {
            entity.fire_immune.store(true, Ordering::Relaxed);
        }

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

    /// Creates an `ItemEntity` for restoring from NBT without random velocity.
    /// The velocity and position will be set by `Entity::read_nbt_non_mut`.
    pub fn new_for_restore(entity: Entity) -> Self {
        Self {
            entity,
            item_stack: Mutex::new(ItemStack::new(1, &pumpkin_data::item::Item::AIR)),
            item_age: AtomicU32::new(0),
            pickup_delay: AtomicU8::new(10),
            health: AtomicF32::new(5.0),
            never_despawn: AtomicBool::new(false),
            never_pickup: AtomicBool::new(false),
        }
    }

    pub const fn get_item_stack(&self) -> &Mutex<ItemStack> {
        &self.item_stack
    }

    pub const fn get_entity(&self) -> &Entity {
        &self.entity
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
        // Always lock in entity_id order to prevent deadlock when two
        // items try to merge with each other concurrently.
        let (low, high) = if self.entity.entity_id < other.entity.entity_id {
            (self, other)
        } else {
            (other, self)
        };

        let low_stack = low.item_stack.lock().await;
        let high_stack = high.item_stack.lock().await;

        let (self_stack, other_stack) = if self.entity.entity_id < other.entity.entity_id {
            (low_stack, high_stack)
        } else {
            (high_stack, low_stack)
        };

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

        let mut event = crate::plugin::api::events::entity::item_merge::ItemMergeEvent {
            entity_id: target.entity.entity_id,
            target_id: source.entity.entity_id,
            cancelled: false,
        };
        if let Some(server) = self.entity.world.load().server.upgrade() {
            server.plugin_manager.fire(&server, &mut event).await;
        }
        if event.cancelled {
            return;
        }

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
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
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

    fn update_no_clip_and_push_out(&self) {
        let entity = &self.entity;
        let pos = entity.pos.load();
        let bounding_box = entity.bounding_box.load();

        let no_clip = !entity
            .world
            .load()
            .is_space_empty(bounding_box.expand(-1.0e-7, -1.0e-7, -1.0e-7));

        entity.no_clip.store(no_clip, Ordering::Relaxed);

        if no_clip {
            entity.push_out_of_blocks(Vector3::new(
                pos.x,
                f64::midpoint(bounding_box.min.y, bounding_box.max.y),
                pos.z,
            ));
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

    async fn move_and_apply_friction<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
        move_velo: Vector3<f64>,
    ) {
        let entity = &self.entity;

        entity.move_entity(caller, move_velo).await;
        entity.tick_block_collisions(caller, server).await;

        let mut friction = 0.98;
        let on_ground = entity.on_ground.load(Ordering::SeqCst);

        let mut velo = entity.velocity.load();
        if on_ground {
            let block_affecting_velo = entity.get_block_with_y_offset(0.999_999).1;
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
            let mut despawn_event =
                crate::plugin::api::events::entity::item_despawn::ItemDespawnEvent::new(
                    entity.entity_id,
                );
            if let Some(server) = entity.world.load().server.upgrade() {
                server
                    .plugin_manager
                    .fire(&server, &mut despawn_event)
                    .await;
            }
            if !despawn_event.cancelled {
                entity.remove().await;
                return false;
            }
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

    async fn sync_motion_if_dirty<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        original_velo: Vector3<f64>,
    ) {
        let entity = &self.entity;

        entity.update_fluid_state(caller).await;

        let velocity_dirty = entity.velocity_dirty.swap(false, Ordering::SeqCst)
            || entity.touching_water.load(Ordering::SeqCst)
            || entity.touching_lava.load(Ordering::SeqCst)
            || entity.velocity.load().sub(&original_velo).length_squared() > 0.1;
        let moved = entity.pos.load() != entity.last_sent_pos.load();
        let position_dirty = moved
            && self
                .item_age
                .load(Ordering::Relaxed)
                .is_multiple_of(ITEM_UPDATE_INTERVAL);

        if position_dirty || velocity_dirty {
            entity.send_pos_rot();
        } else if moved {
            entity.send_bedrock_pos();
        }
        if velocity_dirty {
            entity.send_velocity();
        }
    }
}

impl NBTStorage for ItemEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.write_nbt(nbt).await;

            let item = self.item_stack.lock().await;
            let mut item_compound = NbtCompound::new();
            item.write_item_stack(&mut item_compound);
            nbt.put_compound("Item", item_compound);

            nbt.put_short("Age", self.item_age.load(Ordering::Relaxed) as i16);
            nbt.put_short(
                "PickupDelay",
                self.pickup_delay.load(Ordering::Relaxed) as i16,
            );
            nbt.put_short("Health", self.health.load(Relaxed) as i16);
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.entity.read_nbt_non_mut(nbt).await;

            // Restore the item stack from the "Item" compound
            if let Some(item_compound) = nbt.get_compound("Item")
                && let Some(stack) = ItemStack::read_item_stack(item_compound)
            {
                *self.item_stack.lock().await = stack;
            }

            // Vanilla stores Age as a short
            self.item_age
                .store(nbt.get_short("Age").unwrap_or(0) as u32, Ordering::Relaxed);

            // Vanilla stores PickupDelay as a short
            if let Some(delay) = nbt.get_short("PickupDelay") {
                self.pickup_delay.store(delay as u8, Ordering::Relaxed);
            }

            // Vanilla stores Health as a short
            if let Some(health) = nbt.get_short("Health") {
                self.health.store(health as f32, Relaxed);
            }
        })
    }
}

impl EntityBase for ItemEntity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.entity;
            self.decrement_pickup_delay();

            let original_velo = entity.velocity.load();
            entity
                .velocity
                .store(self.apply_fluid_drag_or_gravity(original_velo));

            self.update_no_clip_and_push_out();

            let move_velo = entity.velocity.load(); // In case push_out_of_blocks modifies it

            let Some(tick_move) = self.should_tick_move(move_velo).await else {
                return;
            };

            if tick_move {
                self.move_and_apply_friction(caller, server, move_velo)
                    .await;
            }

            if self.process_age_and_merge().await {
                self.sync_motion_if_dirty(caller, original_velo).await;
            }
        })
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {
            self.entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::item::ITEM,
                    &ItemStackSerializer::from(self.item_stack.lock().await.clone()),
                )],
                None,
            );
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

            let (item_id, count_before) = {
                let stack = self.item_stack.lock().await;
                (stack.item.id, stack.item_count)
            };

            let inserted = {
                let mut stack = self.item_stack.lock().await;
                player.inventory.insert_stack_anywhere(&mut stack).await
            };

            if inserted || player.is_creative() {
                let (count_after, is_empty) = {
                    let stack = self.item_stack.lock().await;
                    (stack.item_count, stack.is_empty())
                };

                let amount_picked_up = if player.is_creative() {
                    count_before
                } else {
                    count_before - count_after
                };

                if amount_picked_up > 0 {
                    player
                        .increment_stat(
                            StatisticCategory::PickedUp,
                            item_id as i32,
                            amount_picked_up as i32,
                        )
                        .await;
                }

                player
                    .living_entity
                    .pickup(&self.entity, amount_picked_up.into());

                player
                    .current_screen_handler
                    .lock()
                    .await
                    .lock()
                    .await
                    .send_content_updates()
                    .await;

                if is_empty {
                    self.entity.remove().await;
                } else {
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

    fn send_bedrock_spawn_packet<'a>(
        &'a self,
        client: &'a crate::net::bedrock::BedrockClient,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.entity;
            let runtime_id = entity.entity_id as u64;
            let item_stack = self.item_stack.lock().await;
            let packet = CAddItemActor {
                entity_unique_id: VarLong(runtime_id as i64),
                entity_runtime_id: VarULong(runtime_id),
                item: ItemStackWrapper::from(&*item_stack),
                position: entity.pos.load().to_f32_lossy(),
                velocity: entity.velocity.load().to_f32_lossy(),
                metadata: entity.bedrock_metadata(),
                from_fishing: false,
            };
            if let Ok(data) = client.serialize_packet(&packet) {
                client.send_game_packet(data).await;
            }
        })
    }

    fn send_java_spawn_packet<'a>(
        &'a self,
        client: &'a crate::net::java::JavaClient,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let spawn_packet = self.entity.create_spawn_packet();
            if let Ok(data) = client.serialize_packet(&spawn_packet) {
                client.enqueue_packet(data).await;
            }

            if client.version.load() >= CURRENT_MC_VERSION {
                let metadata = Metadata::new(
                    pumpkin_data::tracked_data::item::ITEM,
                    ItemStackSerializer::from(self.item_stack.lock().await.clone()),
                );
                let mut data = Vec::new();
                if metadata.write(&mut data, &client.version.load()).is_ok() {
                    data.push(255);
                    let meta_packet =
                        CSetEntityMetadata::new(self.entity.entity_id.into(), data.into());
                    if let Ok(meta_data) = client.serialize_packet(&meta_packet) {
                        client.enqueue_packet(meta_data).await;
                    }
                }
            }
        })
    }
}

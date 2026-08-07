use crate::entity::player::statistics::StatisticCategory;
use crate::{entity::EntityBaseFuture, server::Server};
use core::f32;
use pumpkin_data::data_component_impl::DamageResistantImpl;
use pumpkin_data::data_component_impl::DamageResistantType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{damage::DamageType, meta_data_type::MetaDataType, tracked_data::TrackedData};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::client::CAddItemActor;
use pumpkin_protocol::bedrock::network_item::ItemStackWrapper;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::atomic_f32::AtomicF32;
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering::{AcqRel, Relaxed};

use std::sync::{
    Arc,
    atomic::{
        AtomicBool, AtomicI32, AtomicU8, AtomicU32,
        Ordering::{self},
    },
};
use tokio::sync::Mutex;

use super::{Entity, EntityBase, NBTStorage, NbtFuture, living::LivingEntity, player::Player};

/// Vanilla `ItemEntity.setUnlimitedLifetime` sentinel: an item with this age
/// never increments (`ItemEntity.java` tick: `if (this.age != -32768)`), so it
/// never reaches the despawn threshold.
const INFINITE_LIFETIME_AGE: i32 = -32768;

pub struct ItemEntity {
    entity: Entity,
    /// Vanilla `ItemEntity.age`: a plain signed counter, not a magnitude.
    /// Negative starting values (e.g. -6000 for extended lifetime) still
    /// increment every tick toward the 6000 despawn threshold; only
    /// `INFINITE_LIFETIME_AGE` is exempt.
    item_age: AtomicI32,
    merge_tick: AtomicU32,
    // These cannot be atomic values because we mutate their state based on what they are; we run
    // into the ABA problem
    item_stack: Mutex<ItemStack>,
    pickup_delay: AtomicU8,
    health: AtomicF32,
    never_pickup: AtomicBool,
}

impl ItemEntity {
    pub fn new(entity: Entity, item_stack: ItemStack) -> Self {
        entity.velocity.store(Vector3::new(
            rand::random::<f64>().mul_add(0.2, -0.1),
            0.2,
            rand::random::<f64>().mul_add(0.2, -0.1),
        ));
        entity.yaw.store(rand::random::<f32>() * 360.0);

        Self::update_fire_immunity(&entity, &item_stack);

        Self {
            entity,
            item_stack: Mutex::new(item_stack),
            item_age: AtomicI32::new(0),
            merge_tick: AtomicU32::new(0),
            pickup_delay: AtomicU8::new(10), // Vanilla pickup delay is 10 ticks
            health: AtomicF32::new(5.0),
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

        Self::update_fire_immunity(&entity, &item_stack);

        Self {
            entity,
            item_stack: Mutex::new(item_stack),
            item_age: AtomicI32::new(0),
            merge_tick: AtomicU32::new(0),
            pickup_delay: AtomicU8::new(pickup_delay), // Vanilla pickup delay is 10 ticks
            health: AtomicF32::new(5.0),
            never_pickup: AtomicBool::new(false),
        }
    }

    /// Creates an `ItemEntity` for restoring from NBT without random velocity.
    /// The velocity and position will be set by `Entity::read_nbt_non_mut`.
    pub fn new_for_restore(entity: Entity) -> Self {
        Self {
            entity,
            item_stack: Mutex::new(ItemStack::new(1, &pumpkin_data::item::Item::AIR)),
            item_age: AtomicI32::new(0),
            merge_tick: AtomicU32::new(0),
            pickup_delay: AtomicU8::new(10),
            health: AtomicF32::new(5.0),
            never_pickup: AtomicBool::new(false),
        }
    }

    /// Vanilla derives fire immunity from the held stack on every query
    /// (`ItemEntity.fireImmune`), so it has to be refreshed whenever the stack
    /// is replaced -- notably when an entity is restored from NBT.
    fn update_fire_immunity(entity: &Entity, item_stack: &ItemStack) {
        let immune = item_stack
            .get_data_component::<DamageResistantImpl>()
            .is_some_and(|res| res.res_type == DamageResistantType::Fire);

        entity.fire_immune.store(immune, Ordering::Relaxed);
    }

    pub const fn get_item_stack(&self) -> &Mutex<ItemStack> {
        &self.item_stack
    }

    pub const fn get_entity(&self) -> &Entity {
        &self.entity
    }

    /// Vanilla `ItemEntity.hasPickUpDelay`.
    pub fn has_pickup_delay(&self) -> bool {
        self.pickup_delay.load(Ordering::Relaxed) > 0
    }

    async fn can_merge(&self) -> bool {
        let age = self.item_age.load(Ordering::Relaxed);
        if self.never_pickup.load(Ordering::Relaxed)
            || self.entity.removed.load(Ordering::Relaxed)
            || age == INFINITE_LIFETIME_AGE
            || age >= 6_000
            || self.pickup_delay.load(Ordering::Relaxed) == u8::MAX
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

        if !Self::are_mergeable_stacks(&self_stack, &other_stack) {
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

        // Vanilla: `toItem.age = Math.min(toItem.age, fromItem.age)`, unconditionally.
        // `INFINITE_LIFETIME_AGE` (-32768) is always the smallest legal age, so the
        // plain min already propagates the "never despawn" sentinel to the target.
        let age = target
            .item_age
            .load(Ordering::Relaxed)
            .min(source.item_age.load(Ordering::Relaxed));

        target.item_age.store(age, Ordering::Relaxed);

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

    fn are_mergeable_stacks(first: &ItemStack, second: &ItemStack) -> bool {
        first.item_count + second.item_count <= first.get_max_stack_size()
            && first.are_items_and_components_equal(second)
    }

    fn decrement_pickup_delay(&self) {
        self.pickup_delay
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
                Some(Self::next_pickup_delay(val))
            })
            .ok();
    }

    const fn next_pickup_delay(value: u8) -> u8 {
        if value == 0 || value == u8::MAX {
            value
        } else {
            value - 1
        }
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

    fn should_tick_move(&self, move_velo: Vector3<f64>) -> bool {
        let entity = &self.entity;

        let mut tick_move = !entity.on_ground.load(Ordering::SeqCst)
            || move_velo.horizontal_length_squared() > 1.0e-5;

        if !tick_move {
            let item_age = self.item_age.load(Ordering::Relaxed);
            tick_move = (item_age + entity.entity_id) % 4 == 0;
        }

        tick_move
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
            friction *= f64::from(block_affecting_velo.slipperiness);
        }

        velo = velo.multiply(friction, 0.98, friction);

        if on_ground && velo.y < 0.0 {
            velo.y = 0.0;
        }

        entity.velocity.store(velo);
    }

    async fn process_age_and_merge(&self) -> bool {
        let entity = &self.entity;
        let merge_tick = self.merge_tick.fetch_add(1, Ordering::Relaxed) + 1;

        // Vanilla: `if (this.age != -32768) { this.age++; }` -- every other age,
        // including negative extended-lifetime starts, increments normally.
        // A single fetch_update (rather than load-then-branch-then-fetch_add)
        // is required here: `try_merge_with` can store INFINITE_LIFETIME_AGE
        // into this same counter from another entity's tick concurrently, and
        // a load/store split could race past it and increment it away.
        let age = self
            .item_age
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |age| {
                (age != INFINITE_LIFETIME_AGE).then_some(age + 1)
            })
            .map_or(INFINITE_LIFETIME_AGE, |prev| prev + 1);

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

        if merge_tick.is_multiple_of(n) && self.can_merge().await {
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
            || entity.velocity.load().sub(&original_velo).length_squared() > 0.01;

        if velocity_dirty {
            entity.send_pos_rot();
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

            // Vanilla: `output.putShort("Age", (short)this.age)` -- a plain cast,
            // no special-casing for the sentinel.
            nbt.put_short("Age", self.item_age.load(Ordering::Relaxed) as i16);

            // `u8::MAX` is this implementation's "never pick up" sentinel;
            // vanilla spells the same thing as 32767.
            let pickup_delay = match self.pickup_delay.load(Ordering::Relaxed) {
                u8::MAX => i16::MAX,
                delay => i16::from(delay),
            };
            nbt.put_short("PickupDelay", pickup_delay);
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
                Self::update_fire_immunity(&self.entity, &stack);
                *self.item_stack.lock().await = stack;
            }

            // Vanilla: `this.age = input.getShortOr("Age", (short)0)`. Negative
            // values are legitimate active states (-32768 never despawns, -6000
            // is the extended-lifetime start) and must round-trip as-is.
            let age = nbt.get_short("Age").unwrap_or(0);
            self.item_age.store(i32::from(age), Ordering::Relaxed);

            // Vanilla stores PickupDelay as a short where 32767 means "never".
            // Truncating instead of saturating would turn e.g. 300 into 44.
            if let Some(delay) = nbt.get_short("PickupDelay") {
                // `delay >= i16::MAX` is the "never pick up" sentinel check (vanilla's 32767).
                // clippy flags `>=` against a max value as redundant since `i16` can't exceed
                // it, but `>=` documents intent (at-or-past the sentinel) better than `==` and
                // is kept deliberately rather than narrowed to an exact-match comparison.
                #[allow(clippy::absurd_extreme_comparisons)]
                let delay = if delay >= i16::MAX {
                    u8::MAX
                } else {
                    delay.clamp(0, i16::from(u8::MAX - 1)) as u8
                };
                self.pickup_delay.store(delay, Ordering::Relaxed);
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

            if self.item_stack.lock().await.is_empty() {
                entity.remove().await;
                return;
            }

            self.decrement_pickup_delay();

            let original_velo = entity.velocity.load();
            entity
                .velocity
                .store(self.apply_fluid_drag_or_gravity(original_velo));

            self.update_no_clip_and_push_out();

            let move_velo = entity.velocity.load(); // In case push_out_of_blocks modifies it

            if self.should_tick_move(move_velo) {
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
                    TrackedData::ITEM,
                    MetaDataType::ITEM_STACK,
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
            client.send_game_packet(&packet).await;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ItemEntity;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;

    #[test]
    fn item_merge_ignores_stack_counts() {
        let first = ItemStack::new(3, &Item::ARROW);
        let second = ItemStack::new(5, &Item::ARROW);

        assert!(ItemEntity::are_mergeable_stacks(&first, &second));
    }

    #[test]
    fn item_merge_rejects_different_components() {
        let first = ItemStack::new(3, &Item::ARROW);
        let second = ItemStack::new(1, &Item::SPECTRAL_ARROW);

        assert!(!ItemEntity::are_mergeable_stacks(&first, &second));
    }

    #[test]
    fn permanent_pickup_delay_is_not_decremented() {
        assert_eq!(ItemEntity::next_pickup_delay(u8::MAX), u8::MAX);
        assert_eq!(ItemEntity::next_pickup_delay(1), 0);
    }
}

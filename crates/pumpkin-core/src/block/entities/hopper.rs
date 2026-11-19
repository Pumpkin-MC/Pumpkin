use crate::block::blocks::chests::chest_container;
use crate::block::blocks::composter::ComposterBlock;
use crate::block::entities::BlockEntity;
use crate::block::entities::decorated_pot::DecoratedPotBlockEntity;
use crate::entity::EntityBase;
use crate::entity::item::ItemEntity;
use crate::plugin::api::events::inventory::hopper_inventory_search::HopperInventorySearchEvent;
use crate::plugin::api::events::inventory::inventory_move_item::InventoryMoveItemEvent;
use crate::plugin::api::events::inventory::inventory_pickup_item::InventoryPickupItemEvent;
use crate::world::World;
use pumpkin_data::block_properties::{
    BlockProperties, ChestLikeProperties, FacingHopper, HopperLikeProperties,
};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockId, BlockStateId};
use pumpkin_inventory::{Clearable, Inventory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::any::Any;
use std::array::from_fn;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Vanilla `Hopper.SUCK_AABB`: from the hopper's bowl up to the top of the block above.
const SUCK_AABB: BoundingBox = BoundingBox::new_array([0.0, 11.0 / 16.0, 0.0], [1.0, 2.0, 1.0]);

pub struct HopperBlockEntity {
    pub position: BlockPos,
    pub items: RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    pub dirty: AtomicBool,
    pub comparator_dirty: AtomicBool,
    pub cooldown_time: AtomicI32,
    pub ticked_game_time: AtomicI64,
    pub metadata: super::ContainerMetadata,
}

#[must_use]
pub const fn to_direction(facing: FacingHopper) -> BlockDirection {
    match facing {
        FacingHopper::Down => BlockDirection::Down,
        FacingHopper::North => BlockDirection::North,
        FacingHopper::South => BlockDirection::South,
        FacingHopper::West => BlockDirection::West,
        FacingHopper::East => BlockDirection::East,
    }
}

#[must_use]
pub fn to_offset(facing: &FacingHopper) -> Vector3<i32> {
    to_direction(*facing).to_offset()
}

/// Properties of one state snapshot, `None` for any other block. `from_state_id` parses whatever
/// it is handed, so only the block id can reject a replacement state.
fn hopper_properties(block: BlockId, state_id: BlockStateId) -> Option<HopperLikeProperties> {
    (block == BlockId::HOPPER).then(|| HopperLikeProperties::from_state_id(state_id))
}

impl BlockEntity for HopperBlockEntity {
    fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put(
            "TransferCooldown",
            NbtTag::Int(self.cooldown_time.load(Ordering::Relaxed)),
        );
        self.write_inventory_nbt(nbt, true);
        self.metadata.write_nbt(nbt);
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let mut hopper = Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
            cooldown_time: AtomicI32::from(
                nbt.get_numeric_int("TransferCooldown")
                    .unwrap_or(Self::NO_COOLDOWN_TIME),
            ),
            ticked_game_time: AtomicI64::new(0),
            metadata: super::ContainerMetadata::from_nbt(nbt),
        };

        pumpkin_inventory::sync_read_items_from_nbt(
            nbt,
            hopper
                .items
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );

        hopper
    }

    fn tick(&self, world: &Arc<World>) {
        self.ticked_game_time
            .store(world.get_world_age(), Ordering::Relaxed);
        // The block entity outlives its block by a tick when another Rayon worker replaces it,
        // so guard like `trial_spawner.rs::tick` does. One snapshot for id and state: a second
        // read could already be the replacement, and the pair would not belong together.
        let (block, state) = world.get_block_and_state(&self.position);
        let Some(properties) = hopper_properties(block.id, state.id) else {
            return;
        };
        // Vanilla `pushItemsTick`: atomic wrapping decrement floored at 0, move once it hits 0.
        let previous = self
            .cooldown_time
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                Some(current.wrapping_sub(1).max(0))
            })
            .unwrap_or_else(|current| current);
        if previous.wrapping_sub(1) <= 0
            && properties.enabled
            && let Some(entity) = world.get_block_entity(&self.position)
            && let Some(hopper) = entity.as_any().downcast_ref::<Self>()
        {
            hopper.try_move_items(properties, world);
        }
    }

    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn is_comparator_dirty(&self) -> bool {
        self.comparator_dirty.load(Ordering::Relaxed)
    }

    fn clear_comparator_dirty(&self) {
        self.comparator_dirty.store(false, Ordering::Relaxed);
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn bedrock_block_actor_data(
        &self,
        _state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        Some(crate::block::entities::bedrock::build_container_actor(
            "Hopper",
            self.position,
            None,
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn apply_item_components(&self, stack: &ItemStack) {
        self.metadata.apply_item_components(stack);
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::apply_container_items_to_slice(stack, &mut *items);
    }

    fn collect_item_components(&self, stack: &mut ItemStack) {
        self.metadata.collect_name_only(stack);
    }

    fn collect_pick_item_components(&self, stack: &mut ItemStack) {
        self.metadata.collect_item_components(stack);
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::collect_container_items_from_slice(&*items, stack);
    }

    fn get_lock(&self) -> Option<pumpkin_data::data_component_impl::LockImpl> {
        self.metadata.get_lock()
    }

    fn get_custom_name(&self) -> Option<pumpkin_util::text::TextComponent> {
        self.metadata.get_custom_name()
    }
}

impl HopperBlockEntity {
    pub const INVENTORY_SIZE: usize = 5;
    pub const ID: &'static str = "minecraft:hopper";
    /// Vanilla `HopperBlockEntity.MOVE_ITEM_SPEED`.
    pub const MOVE_ITEM_SPEED: i32 = 8;
    /// Vanilla `HopperBlockEntity.NO_COOLDOWN_TIME`.
    const NO_COOLDOWN_TIME: i32 = -1;

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
            cooldown_time: AtomicI32::new(Self::NO_COOLDOWN_TIME),
            ticked_game_time: AtomicI64::new(0),
            metadata: super::ContainerMetadata::default(),
        }
    }

    fn read_items(&self) -> RwLockReadGuard<'_, [ItemStack; Self::INVENTORY_SIZE]> {
        self.items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write_items(&self) -> RwLockWriteGuard<'_, [ItemStack; Self::INVENTORY_SIZE]> {
        self.items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Vanilla `HopperBlockEntity.getContainerAt`: the container hoppers, droppers and crafters see.
    #[must_use]
    pub fn get_container_at(world: &Arc<World>, pos: &BlockPos) -> Option<Arc<dyn Inventory>> {
        Self::get_container_at_coords(
            world,
            pos,
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + 0.5,
            f64::from(pos.0.z) + 0.5,
        )
    }

    /// The block container at `pos`, else a random container entity around `x`, `y`, `z`.
    #[must_use]
    pub fn get_container_at_coords(
        world: &Arc<World>,
        pos: &BlockPos,
        x: f64,
        y: f64,
        z: f64,
    ) -> Option<Arc<dyn Inventory>> {
        Self::get_block_container(world, pos).or_else(|| Self::get_entity_container(world, x, y, z))
    }

    /// Vanilla `HopperBlockEntity.getBlockContainer`.
    fn get_block_container(world: &Arc<World>, pos: &BlockPos) -> Option<Arc<dyn Inventory>> {
        let (block, state) = world.get_block_and_state(pos);
        // `WorldlyContainerHolder`: the composter has no block entity.
        if block == &Block::COMPOSTER {
            return Some(ComposterBlock::get_container(state.id, world, pos));
        }
        // Checked first like vanilla, a miss here skips the block entity lookup.
        if !state.has_block_entity() {
            return None;
        }
        let entity = world.get_block_entity(pos)?;
        if ChestLikeProperties::handles_block_id(block.id) {
            chest_container(world, block, pos, state.id, &entity)
        } else if let Ok(pot) = Arc::downcast::<DecoratedPotBlockEntity>(entity.clone()) {
            // Vanilla `DecoratedPotBlockEntity.getTheItem` unpacks before hopper slot access.
            DecoratedPotBlockEntity::unpack_loot_table(&pot, world);
            pot.get_container()
        } else {
            // TODO vanilla `RandomizableContainerBlockEntity.getItem` unpacks barrels, shulker boxes,
            // dispensers, droppers, hoppers and crafters. Only chests unpack in `chest_container`.
            entity.get_container()
        }
    }

    /// Vanilla `HopperBlockEntity.getEntityContainer`: a random one of the container entities
    /// (chest boat, chest or hopper minecart) in the box around the point.
    fn get_entity_container(world: &World, x: f64, y: f64, z: f64) -> Option<Arc<dyn Inventory>> {
        let search_box = BoundingBox::new(
            Vector3::new(x - 0.5, y - 0.5, z - 0.5),
            Vector3::new(x + 0.5, y + 0.5, z + 0.5),
        );
        let candidates: Vec<Arc<dyn EntityBase>> = world
            .get_entities_at_box(&search_box)
            .into_iter()
            .filter(|entity| entity.get_entity().is_alive() && entity.has_container_inventory())
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let index = rand::rng().random_range(0..candidates.len());
        candidates[index].get_container_inventory()
    }

    /// Vanilla `getSlots`: a `WorldlyContainer` names the slots of a face, any other container
    /// offers all of its slots.
    fn slot_count(container: &dyn Inventory, direction: BlockDirection) -> usize {
        if container.is_sided() {
            container.get_slots_for_face(direction).len()
        } else {
            container.size()
        }
    }

    /// The `n`th slot of [`Self::slot_count`].
    fn slot_at(container: &dyn Inventory, direction: BlockDirection, n: usize) -> usize {
        if container.is_sided() {
            container.get_slots_for_face(direction)[n]
        } else {
            n
        }
    }

    /// Vanilla `isFullContainer`.
    fn is_full_container(container: &dyn Inventory, direction: BlockDirection) -> bool {
        (0..Self::slot_count(container, direction)).all(|n| {
            let item = container.get_stack(Self::slot_at(container, direction, n));
            item.item_count >= item.get_max_stack_size()
        })
    }

    /// Vanilla `canPlaceItemInContainer`.
    fn can_place_item_in_container(
        container: &dyn Inventory,
        stack: &ItemStack,
        slot: usize,
        direction: Option<BlockDirection>,
    ) -> bool {
        container.is_valid_slot_for(slot, stack)
            && (!container.is_sided()
                || container.can_place_item_through_face(slot, stack, direction))
    }

    /// Vanilla `canTakeItemFromContainer`.
    fn can_take_item_from_container(
        into: &dyn Inventory,
        from: &dyn Inventory,
        stack: &ItemStack,
        slot: usize,
        direction: BlockDirection,
    ) -> bool {
        from.can_transfer_to(into, slot, stack)
            && (!from.is_sided() || from.can_take_item_through_face(slot, stack, direction))
    }

    /// Vanilla `suckInItems` from a container: the first slot of its bottom face that gives an item.
    ///
    /// `Err` is an item that could not go back into a full slot after a failed offer, the caller
    /// drops it so it is never lost.
    pub fn try_take_from_source(
        into: &dyn Inventory,
        source: &dyn Inventory,
    ) -> Result<bool, ItemStack> {
        let direction = BlockDirection::Down;
        for n in 0..Self::slot_count(source, direction) {
            let slot = Self::slot_at(source, direction, n);
            let item = source.get_stack(slot);
            if !item.is_empty()
                && Self::can_take_item_from_container(into, source, &item, slot, direction)
                && Self::transfer_one(source, slot, &item, into)?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Takes one `approved` item out of `source` and offers it to `into`, restoring it on failure.
    fn transfer_one(
        source: &dyn Inventory,
        slot: usize,
        approved: &ItemStack,
        into: &dyn Inventory,
    ) -> Result<bool, ItemStack> {
        let Some(one_item) = Self::take_one(source, slot, approved) else {
            return Ok(false);
        };
        if Self::add_one_item(source, into, &one_item) {
            // Furnace XP stays stored: vanilla only pays it out to a player or on break.
            source.mark_dirty();
            return Ok(true);
        }
        Self::put_back(source, slot, one_item).map_or(Ok(false), Err)
    }

    fn try_move_items(&self, state: HopperLikeProperties, world: &Arc<World>) {
        // Emptiness before the cooldown check, so a just inserted item isn't ejected this tick.
        let has_items = !self.is_empty();
        if self.cooldown_time.load(Ordering::Relaxed) <= 0 && state.enabled {
            let mut success = has_items && self.eject_items(world, state.facing);
            if !self.inventory_full() {
                success |= self.suck_in_items(world);
            }
            if success {
                // Vanilla `setCooldown(8)`: overrides a cooldown an insert set during this move.
                self.cooldown_time
                    .store(Self::MOVE_ITEM_SPEED, Ordering::Relaxed);
                self.mark_dirty();
            }
        }
    }

    fn inventory_full(&self) -> bool {
        self.read_items()
            .iter()
            .all(|item| !item.is_empty() && item.item_count == item.get_max_stack_size())
    }

    #[allow(clippy::too_many_lines)]
    fn suck_in_items(&self, world: &Arc<World>) -> bool {
        let pos_up = &self.position.up();
        if world.cancelled_by_plugin(|| HopperInventorySearchEvent::new(self.position, *pos_up)) {
            return false;
        }

        if let Some(container) = Self::get_container_at(world, pos_up) {
            // An empty container has nothing to take, skip reading every slot.
            if container.is_empty() {
                return false;
            }
            let direction = BlockDirection::Down;
            for n in 0..Self::slot_count(container.as_ref(), direction) {
                let i = Self::slot_at(container.as_ref(), direction, n);
                let item = container.get_stack(i);
                if item.is_empty()
                    || !Self::can_take_item_from_container(
                        self,
                        container.as_ref(),
                        &item,
                        i,
                        direction,
                    )
                {
                    continue;
                }
                if world.cancelled_by_plugin(|| {
                    InventoryMoveItemEvent::new(
                        *pos_up,
                        self.position,
                        item.item.registry_key.to_string(),
                        1,
                    )
                }) {
                    continue;
                }
                // Again after the event: a handler may have filled this hopper, and taking a
                // record that can't fit would stop the song for nothing.
                if !Self::can_take_item_from_container(
                    self,
                    container.as_ref(),
                    &item,
                    i,
                    direction,
                ) {
                    continue;
                }
                match Self::transfer_one(container.as_ref(), i, &item, self) {
                    Ok(true) => return true,
                    Ok(false) => {}
                    Err(leftover) => {
                        // Concurrent insert filled the slot, drop the item, never void it.
                        let pos = pos_up.to_centered_f64();
                        world.scatter_stack(pos.x, pos.y, pos.z, leftover);
                    }
                }
            }
            return false;
        }
        let (block, state) = world.get_block_and_state(pos_up);
        if !state.is_full_cube() || block.has_tag(&tag::Block::MINECRAFT_DOES_NOT_BLOCK_HOPPERS) {
            let entities = world.get_entities_at_box(&SUCK_AABB.at_pos(self.position));
            for entity_base in entities {
                let Some(item_entity) = entity_base.get_item_entity() else {
                    continue;
                };
                let (is_empty, item_id) = {
                    let stack = item_entity
                        .get_item_stack()
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    (stack.is_empty(), stack.item.registry_key)
                };
                if is_empty
                    || world.cancelled_by_plugin(|| {
                        InventoryPickupItemEvent::new(
                            self.position,
                            item_entity.get_entity().entity_id,
                            item_id.to_string(),
                        )
                    })
                {
                    continue;
                }
                if Self::pickup_item_entity(self, item_entity) {
                    return true;
                }
            }
        }
        false
    }

    /// Vanilla `removeItem(slot, 1)`. `None` when the slot no longer holds `approved`.
    /// Taken before the offer, holding the source lock would deadlock two facing hoppers.
    pub(crate) fn take_one(
        container: &dyn Inventory,
        slot: usize,
        approved: &ItemStack,
    ) -> Option<ItemStack> {
        let mut one_item = None;
        container.update_stack(slot, &mut |current| {
            if current.is_empty() || !current.are_items_and_components_equal(approved) {
                return false;
            }
            one_item = Some(current.split(1));
            true
        });
        // Like vanilla (MC-175090) the composter empties on the take, even if the insert then fails.
        if one_item.is_some() {
            container.mark_dirty();
        }
        one_item
    }

    /// Undoes [`Self::take_one`] under the slot lock, returns the item if the slot has no room.
    pub(crate) fn put_back(
        container: &dyn Inventory,
        slot: usize,
        item: ItemStack,
    ) -> Option<ItemStack> {
        let mut item = Some(item);
        container.update_stack(slot, &mut |current| {
            let Some(one_item) = item.take() else {
                return false;
            };
            if current.is_empty() {
                *current = one_item;
                return true;
            }
            if current.are_items_and_components_equal(&one_item)
                && current.item_count < current.get_max_stack_size()
            {
                current.increment(1);
                return true;
            }
            item = Some(one_item);
            false
        });
        container.mark_dirty();
        item
    }

    /// `facing` comes from the live block state, the block entity keeps no copy that could go stale.
    fn eject_items(&self, world: &Arc<World>, facing: FacingHopper) -> bool {
        let direction = to_direction(facing);
        let opposite_dir = direction.opposite();
        let target_pos = self.position.offset(direction.to_offset());
        let Some(container) = Self::get_container_at(world, &target_pos) else {
            return false;
        };
        if Self::is_full_container(container.as_ref(), opposite_dir) {
            return false;
        }
        for slot in 0..Self::INVENTORY_SIZE {
            let item = self.get_stack(slot);
            if item.is_empty() {
                continue;
            }
            if world.cancelled_by_plugin(|| {
                InventoryMoveItemEvent::new(
                    self.position,
                    target_pos,
                    item.item.registry_key.to_string(),
                    1,
                )
            }) {
                continue;
            }
            // Vanilla `ejectItems`: remove the item before offering it to the target, restore it on failure.
            let Some(one_item) = Self::take_one(self, slot, &item) else {
                continue;
            };
            if Self::add_item(
                Some(self),
                container.as_ref(),
                one_item.clone(),
                Some(opposite_dir),
            )
            .is_empty()
            {
                return true;
            }
            if let Some(leftover) = Self::put_back(self, slot, one_item) {
                // Slot is someone else's now and full -> drop as an item entity, never void.
                let pos = self.position.to_centered_f64();
                world.scatter_stack(pos.x, pos.y, pos.z, leftover);
            }
        }
        false
    }

    pub fn add_one_item(from: &dyn Inventory, to: &dyn Inventory, item: &ItemStack) -> bool {
        Self::add_item(Some(from), to, item.copy_with_count(1), None).is_empty()
    }

    /// Moves a whole item stack of an item entity into `into`, true once all of it moved.
    pub fn pickup_item_entity(into: &dyn Inventory, item_entity: &ItemEntity) -> bool {
        item_entity.insert_into(|stack| Self::add_item(None, into, stack, None))
    }

    /// Vanilla `HopperBlockEntity.addItem(from, container, stack, direction)`: returns what did
    /// not fit. A `WorldlyContainer` only offers the slots of the face `direction` names.
    pub fn add_item(
        from: Option<&dyn Inventory>,
        to: &dyn Inventory,
        mut stack: ItemStack,
        direction: Option<BlockDirection>,
    ) -> ItemStack {
        let face = direction.filter(|_| to.is_sided());
        let count = face.map_or_else(|| to.size(), |face| Self::slot_count(to, face));
        for n in 0..count {
            if stack.is_empty() {
                break;
            }
            let slot = face.map_or(n, |face| Self::slot_at(to, face, n));
            Self::try_move_in_item(from, to, &mut stack, slot, direction);
        }
        stack
    }

    /// Vanilla `HopperBlockEntity.tryMoveInItem`.
    fn try_move_in_item(
        from: Option<&dyn Inventory>,
        to: &dyn Inventory,
        stack: &mut ItemStack,
        slot: usize,
        direction: Option<BlockDirection>,
    ) {
        if !Self::can_place_item_in_container(to, stack, slot, direction) {
            return;
        }
        // Vanilla `wasEmpty` only matters for a hopper target. Read under the same lock as the
        // insert and the cooldown, so two sources filling one empty hopper can't both see it empty.
        if let Some(hopper) = to.as_any().downcast_ref::<Self>() {
            let mut items = hopper.write_items();
            let was_empty = items.iter().all(ItemStack::is_empty);
            let Some(dst) = items.get_mut(slot) else {
                return;
            };
            if !Self::merge_into_slot(dst, stack, hopper.get_max_count_per_stack()) {
                return;
            }
            if was_empty {
                hopper.set_cooldown_after_insert(from);
            }
        } else if !to.update_stack(slot, &mut |dst| {
            Self::merge_into_slot(dst, stack, to.get_max_count_per_stack())
        }) {
            // One lock for read and write: a parallel write would otherwise be overwritten by a stale copy.
            return;
        }
        to.mark_dirty();
    }

    /// The cooldown half of vanilla `tryMoveInItem`, for a hopper that was empty. Called under its
    /// items lock, see `try_move_items`.
    fn set_cooldown_after_insert(&self, from: Option<&dyn Inventory>) {
        // A hopper that already ticked skips one tick, one that hasn't counts it down: both move together.
        let cooldown = match from.and_then(|from| from.as_any().downcast_ref::<Self>()) {
            Some(from_hopper)
                if self.ticked_game_time.load(Ordering::Relaxed)
                    >= from_hopper.ticked_game_time.load(Ordering::Relaxed) =>
            {
                Self::MOVE_ITEM_SPEED - 1
            }
            _ => Self::MOVE_ITEM_SPEED,
        };
        // Vanilla `isOnCustomCooldown`: a cooldown above `MOVE_ITEM_SPEED` is kept.
        let _ = self
            .cooldown_time
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                (current <= Self::MOVE_ITEM_SPEED).then_some(cooldown)
            });
    }

    /// The slot half of vanilla `tryMoveInItem`. Returns whether anything moved.
    ///
    /// An empty slot takes at most `max_count` items, what the container allows per slot. Vanilla
    /// `setItem` would cut the excess off, here it stays in `stack`.
    fn merge_into_slot(dst: &mut ItemStack, stack: &mut ItemStack, max_count: u8) -> bool {
        if dst.is_empty() {
            *dst = if stack.item_count > max_count {
                stack.split(max_count)
            } else {
                std::mem::replace(stack, ItemStack::EMPTY.clone())
            };
            return true;
        }
        if !dst.are_items_and_components_equal(stack) {
            return false;
        }
        let moved = stack
            .item_count
            .min(stack.get_max_stack_size().saturating_sub(dst.item_count));
        stack.decrement(moved);
        dst.increment(moved);
        moved > 0
    }
}

impl Inventory for HopperBlockEntity {
    fn size(&self) -> usize {
        Self::INVENTORY_SIZE
    }

    fn is_empty(&self) -> bool {
        let items = self.read_items();
        items.iter().all(ItemStack::is_empty)
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        let items = self.read_items();
        items
            .get(slot)
            .cloned()
            .unwrap_or_else(|| ItemStack::EMPTY.clone())
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        let mut items = self.write_items();
        let Some(slot_item) = items.get_mut(slot) else {
            return ItemStack::EMPTY.clone();
        };
        let removed = std::mem::replace(slot_item, ItemStack::EMPTY.clone());
        self.mark_dirty();
        removed
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        let mut items = self.write_items();
        let Some(slot_item) = items.get_mut(slot) else {
            return ItemStack::EMPTY.clone();
        };
        let res = if !slot_item.is_empty() && amount > 0 {
            slot_item.split(amount)
        } else {
            ItemStack::EMPTY.clone()
        };
        self.mark_dirty();
        res
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        let mut items = self.write_items();
        if let Some(slot_item) = items.get_mut(slot) {
            *slot_item = stack;
            self.mark_dirty();
        }
    }

    fn update_stack(&self, slot: usize, update: &mut dyn FnMut(&mut ItemStack) -> bool) -> bool {
        pumpkin_inventory::update_locked_stack(&self.items, slot, update)
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
        self.comparator_dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for HopperBlockEntity {
    fn clear(&self) {
        let mut items = self.write_items();
        items.fill_with(|| ItemStack::EMPTY.clone());
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::{Block, BlockDirection, item::Item};

    #[test]
    fn hopper_state_yields_its_properties() {
        let properties =
            hopper_properties(Block::HOPPER.id, Block::HOPPER.default_state.id).unwrap();

        assert!(properties.enabled);
        assert_eq!(properties.facing, FacingHopper::Down);
    }

    #[test]
    fn disabled_hopper_state_is_read_as_disabled() {
        let disabled = HopperLikeProperties {
            facing: FacingHopper::North,
            enabled: false,
        }
        .to_state_id(&Block::HOPPER);

        let properties = hopper_properties(Block::HOPPER.id, disabled).unwrap();

        assert!(!properties.enabled);
        assert_eq!(properties.facing, FacingHopper::North);
    }

    /// Every one of these decodes cleanly through `from_state_id` into some `facing`/`enabled`.
    /// Nothing about the state marks it as foreign, so rejection has to come from the id.
    #[test]
    fn replacement_state_yields_no_properties() {
        for replacement in [Block::AIR, Block::CHEST, Block::DROPPER, Block::PISTON] {
            assert!(
                hopper_properties(replacement.id, replacement.default_state.id).is_none(),
                "{} was accepted as a hopper",
                replacement.name
            );
        }
    }

    /// The other direction: a hopper state under a foreign id is still foreign. Pins that the id
    /// decides, not a range check on the state.
    #[test]
    fn hopper_state_id_under_another_block_yields_no_properties() {
        assert!(hopper_properties(Block::CHEST.id, Block::HOPPER.default_state.id).is_none());
    }

    fn diamond() -> ItemStack {
        ItemStack::new(1, &Item::DIAMOND)
    }

    fn hopper_holding(stack: ItemStack) -> HopperBlockEntity {
        let hopper = HopperBlockEntity::new(BlockPos::new(0, 0, 0));
        hopper.set_stack(0, stack);
        hopper
    }

    #[test]
    fn take_one_splits_a_single_item_off() {
        let hopper = hopper_holding(ItemStack::new(10, &Item::DIAMOND));

        let one_item = HopperBlockEntity::take_one(&hopper, 0, &diamond()).unwrap();

        assert_eq!(one_item.item_count, 1);
        assert_eq!(hopper.get_stack(0).item_count, 9);
    }

    #[test]
    fn take_one_on_an_empty_slot_extracts_nothing() {
        let hopper = hopper_holding(ItemStack::EMPTY.clone());

        assert!(HopperBlockEntity::take_one(&hopper, 0, &diamond()).is_none());
    }

    #[test]
    fn take_one_on_a_missing_slot_extracts_nothing() {
        let hopper = hopper_holding(ItemStack::new(10, &Item::DIAMOND));

        assert!(
            HopperBlockEntity::take_one(&hopper, HopperBlockEntity::INVENTORY_SIZE, &diamond())
                .is_none()
        );
        assert_eq!(hopper.get_stack(0).item_count, 10);
    }

    /// The event let a diamond through, but the slot holds dirt by now.
    #[test]
    fn take_one_skips_a_slot_swapped_after_the_event() {
        let hopper = hopper_holding(ItemStack::new(64, &Item::DIRT));

        assert!(HopperBlockEntity::take_one(&hopper, 0, &diamond()).is_none());
        assert_eq!(hopper.get_stack(0).item_count, 64);
    }

    /// Same item, but a renamed diamond is not the stack the event approved.
    #[test]
    fn take_one_skips_a_slot_with_other_components() {
        let mut named = ItemStack::new(10, &Item::DIAMOND);
        named.set_data_component(pumpkin_data::data_component_impl::CustomNameImpl {
            name: pumpkin_util::text::TextComponent::text("Named"),
        });
        let hopper = hopper_holding(named.clone());

        assert!(HopperBlockEntity::take_one(&hopper, 0, &diamond()).is_none());
        let current = hopper.get_stack(0);
        assert_eq!(current.item_count, 10);
        assert!(current.are_items_and_components_equal(&named));
    }

    #[test]
    fn untouched_slot_gets_the_item_back() {
        let hopper = hopper_holding(ItemStack::new(10, &Item::DIAMOND));
        let one_item = HopperBlockEntity::take_one(&hopper, 0, &diamond()).unwrap();

        assert!(HopperBlockEntity::put_back(&hopper, 0, one_item).is_none());
        assert_eq!(hopper.get_stack(0).item_count, 10);
    }

    /// The slot says 3 because someone took 6 while the offer was out, only the one item returns.
    #[test]
    fn changed_count_takes_back_only_the_one_item() {
        let hopper = hopper_holding(ItemStack::new(10, &Item::DIAMOND));
        let one_item = HopperBlockEntity::take_one(&hopper, 0, &diamond()).unwrap();
        hopper.set_stack(0, ItemStack::new(3, &Item::DIAMOND));

        assert!(HopperBlockEntity::put_back(&hopper, 0, one_item).is_none());
        assert_eq!(hopper.get_stack(0).item_count, 4);
    }

    /// A foreign item cannot absorb the one item and must not be overwritten, so nothing fits and
    /// the item comes back out.
    #[test]
    fn foreign_item_in_the_slot_is_left_alone() {
        let hopper = hopper_holding(ItemStack::new(1, &Item::DIAMOND));
        let one_item = HopperBlockEntity::take_one(&hopper, 0, &diamond()).unwrap();
        hopper.set_stack(0, ItemStack::new(64, &Item::DIRT));

        let leftover = HopperBlockEntity::put_back(&hopper, 0, one_item).unwrap();

        assert_eq!(leftover.get_item().id, Item::DIAMOND.id);
        assert_eq!(leftover.item_count, 1);
        let current = hopper.get_stack(0);
        assert_eq!(current.get_item().id, Item::DIRT.id);
        assert_eq!(current.item_count, 64);
    }

    /// Emptied in the meantime, so there is room and nothing to overwrite.
    #[test]
    fn emptied_slot_takes_the_single_item() {
        let hopper = hopper_holding(ItemStack::new(10, &Item::DIAMOND));
        let one_item = HopperBlockEntity::take_one(&hopper, 0, &diamond()).unwrap();
        hopper.set_stack(0, ItemStack::EMPTY.clone());

        assert!(HopperBlockEntity::put_back(&hopper, 0, one_item).is_none());
        assert_eq!(hopper.get_stack(0).item_count, 1);
    }

    /// Same item but no room left. Incrementing would push it past `get_max_stack_size`.
    #[test]
    fn full_slot_of_the_same_item_hands_the_item_back() {
        let hopper = hopper_holding(ItemStack::new(10, &Item::DIAMOND));
        let one_item = HopperBlockEntity::take_one(&hopper, 0, &diamond()).unwrap();
        let max = ItemStack::new(1, &Item::DIAMOND).get_max_stack_size();
        hopper.set_stack(0, ItemStack::new(max, &Item::DIAMOND));

        let leftover = HopperBlockEntity::put_back(&hopper, 0, one_item).unwrap();

        assert_eq!(leftover.item_count, 1);
        assert_eq!(hopper.get_stack(0).item_count, max);
    }

    /// A dropped stack goes in whole, like vanilla `addItem(Container, ItemEntity)`.
    #[test]
    fn add_item_moves_a_whole_stack_into_one_slot() {
        let hopper = hopper_holding(ItemStack::EMPTY.clone());

        let leftover =
            HopperBlockEntity::add_item(None, &hopper, ItemStack::new(64, &Item::DIAMOND), None);

        assert!(leftover.is_empty());
        assert_eq!(hopper.get_stack(0).item_count, 64);
        assert!(hopper.get_stack(1).is_empty());
    }

    /// Tops up the partial stack, and what's left comes back when every other slot is taken.
    #[test]
    fn add_item_returns_what_does_not_fit() {
        let hopper = hopper_holding(ItemStack::new(60, &Item::DIAMOND));
        for slot in 1..HopperBlockEntity::INVENTORY_SIZE {
            hopper.set_stack(slot, ItemStack::new(64, &Item::DIRT));
        }

        let leftover =
            HopperBlockEntity::add_item(None, &hopper, ItemStack::new(10, &Item::DIAMOND), None);

        assert_eq!(hopper.get_stack(0).item_count, 64);
        assert_eq!(leftover.get_item().id, Item::DIAMOND.id);
        assert_eq!(leftover.item_count, 6);
    }

    #[test]
    fn furnace_worldly_container_respects_faces_fuel_and_empty_bucket() {
        use crate::block::entities::furnace::FurnaceBlockEntity;
        let furnace = FurnaceBlockEntity::new(BlockPos::new(0, 64, 0));
        let dest_hopper = HopperBlockEntity::new(BlockPos::new(0, 63, 0));

        // UP inserts into input slot 0
        let leftover = HopperBlockEntity::add_item(
            None,
            &furnace,
            ItemStack::new(2, &Item::RAW_IRON),
            Some(BlockDirection::Up),
        );
        assert!(leftover.is_empty());
        assert_eq!(furnace.get_stack(0).item_count, 2);

        // NORTH (side) rejects non-fuel in slot 1, accepts coal and empty bucket
        let rejected = HopperBlockEntity::add_item(
            None,
            &furnace,
            ItemStack::new(1, &Item::RAW_IRON),
            Some(BlockDirection::North),
        );
        assert_eq!(rejected.item_count, 1);

        let fuel = HopperBlockEntity::add_item(
            None,
            &furnace,
            ItemStack::new(3, &Item::COAL),
            Some(BlockDirection::North),
        );
        assert!(fuel.is_empty());
        assert_eq!(furnace.get_stack(1).item_count, 3);

        // DOWN does NOT pull coal out of fuel slot 1
        assert!(matches!(
            HopperBlockEntity::try_take_from_source(&dest_hopper, &furnace),
            Ok(false)
        ));

        // DOWN DOES pull an empty bucket out of fuel slot 1 (after lava bucket consumed)
        furnace.set_stack(1, ItemStack::new(1, &Item::BUCKET));
        assert!(matches!(
            HopperBlockEntity::try_take_from_source(&dest_hopper, &furnace),
            Ok(true)
        ));
        assert!(furnace.get_stack(1).is_empty());
        assert_eq!(dest_hopper.get_stack(0).item.id, Item::BUCKET.id);
    }

    #[test]
    fn brewing_stand_worldly_container_sides_and_glass_bottle_extraction() {
        use crate::block::entities::brewing_stand::BrewingStandBlockEntity;
        let stand = BrewingStandBlockEntity::new(BlockPos::new(0, 64, 0));
        let dest_hopper = HopperBlockEntity::new(BlockPos::new(0, 63, 0));

        // UP accepts brewing reagent (Nether Wart) in slot 3, rejects non-reagents (Oak Planks)
        let bad_top = HopperBlockEntity::add_item(
            None,
            &stand,
            ItemStack::new(1, &Item::OAK_PLANKS),
            Some(BlockDirection::Up),
        );
        assert_eq!(bad_top.item_count, 1);

        let good_top = HopperBlockEntity::add_item(
            None,
            &stand,
            ItemStack::new(2, &Item::NETHER_WART),
            Some(BlockDirection::Up),
        );
        assert!(good_top.is_empty());
        assert_eq!(stand.get_stack(3).item_count, 2);

        // EAST accepts Blaze Powder into slot 4 and 1x Glass Bottle per bottle slot 0..=2
        let blaze = HopperBlockEntity::add_item(
            None,
            &stand,
            ItemStack::new(4, &Item::BLAZE_POWDER),
            Some(BlockDirection::East),
        );
        assert!(blaze.is_empty());
        assert_eq!(stand.get_stack(4).item_count, 4);

        for expected_slot in 0..3 {
            let leftover = HopperBlockEntity::add_item(
                None,
                &stand,
                ItemStack::new(1, &Item::GLASS_BOTTLE),
                Some(BlockDirection::East),
            );
            assert!(leftover.is_empty());
            assert_eq!(stand.get_stack(expected_slot).item_count, 1);
        }
        let fourth_bottle = HopperBlockEntity::add_item(
            None,
            &stand,
            ItemStack::new(1, &Item::GLASS_BOTTLE),
            Some(BlockDirection::East),
        );
        assert_eq!(fourth_bottle.item_count, 1);

        // Clear bottle slots; DOWN does NOT extract Nether Wart from slot 3,
        // but DOES extract leftover Glass Bottle from slot 3 (e.g. after Dragon's Breath).
        stand.set_stack(0, ItemStack::EMPTY.clone());
        stand.set_stack(1, ItemStack::EMPTY.clone());
        stand.set_stack(2, ItemStack::EMPTY.clone());
        assert!(matches!(
            HopperBlockEntity::try_take_from_source(&dest_hopper, &stand),
            Ok(false)
        ));

        stand.set_stack(3, ItemStack::new(1, &Item::GLASS_BOTTLE));
        assert!(matches!(
            HopperBlockEntity::try_take_from_source(&dest_hopper, &stand),
            Ok(true)
        ));
        assert!(stand.get_stack(3).is_empty());
        assert_eq!(dest_hopper.get_stack(0).item.id, Item::GLASS_BOTTLE.id);
    }

    #[test]
    fn single_item_and_filtered_containers_enforce_26_3_rules() {
        use crate::block::entities::chiseled_bookshelf::ChiseledBookshelfBlockEntity;
        use crate::block::entities::jukebox::JukeboxBlockEntity;
        use crate::block::entities::lectern::LecternBlockEntity;
        use crate::block::entities::shulker_box::ShulkerBoxBlockEntity;

        // Shulker box rejects nested shulker boxes
        let shulker = ShulkerBoxBlockEntity::new(BlockPos::new(0, 64, 0));
        let nested = HopperBlockEntity::add_item(
            None,
            &shulker,
            ItemStack::new(1, &Item::SHULKER_BOX),
            Some(BlockDirection::Up),
        );
        assert_eq!(nested.item_count, 1);

        // Chiseled bookshelf rejects non-books and caps each slot at 1 book
        let shelf = ChiseledBookshelfBlockEntity::new(BlockPos::new(0, 64, 0));
        assert_eq!(
            HopperBlockEntity::add_item(None, &shelf, ItemStack::new(1, &Item::DIAMOND), None)
                .item_count,
            1
        );
        let books_left =
            HopperBlockEntity::add_item(None, &shelf, ItemStack::new(7, &Item::BOOK), None);
        assert_eq!(books_left.item_count, 1);
        for slot in 0..6 {
            assert_eq!(shelf.get_stack(slot).item_count, 1);
        }

        // Jukebox accepts 1 music disc, rejects non-discs
        let jukebox = JukeboxBlockEntity::new(BlockPos::new(0, 64, 0));
        assert_eq!(
            HopperBlockEntity::add_item(None, &jukebox, ItemStack::new(1, &Item::STONE), None)
                .item_count,
            1
        );
        assert!(
            HopperBlockEntity::add_item(
                None,
                &jukebox,
                ItemStack::new(1, &Item::MUSIC_DISC_CAT),
                None,
            )
            .is_empty()
        );
        assert_eq!(
            HopperBlockEntity::add_item(
                None,
                &jukebox,
                ItemStack::new(1, &Item::MUSIC_DISC_13),
                None,
            )
            .item_count,
            1
        );

        // Lectern is NOT exposed as a world container (`BlockEntity::get_inventory` == None)
        let lectern: Arc<dyn BlockEntity> =
            Arc::new(LecternBlockEntity::new(BlockPos::new(0, 64, 0)));
        assert!(lectern.get_inventory().is_none());
    }

    #[test]
    fn empty_hopper_transfer_cooldown_uses_ticked_game_time() {
        let src = HopperBlockEntity::new(BlockPos::new(0, 65, 0));
        let dst_already_ticked = HopperBlockEntity::new(BlockPos::new(0, 64, 0));
        src.ticked_game_time.store(100, Ordering::Relaxed);
        dst_already_ticked
            .ticked_game_time
            .store(100, Ordering::Relaxed);

        assert!(
            HopperBlockEntity::add_item(
                Some(&src),
                &dst_already_ticked,
                ItemStack::new(1, &Item::IRON_INGOT),
                Some(BlockDirection::Up),
            )
            .is_empty()
        );
        assert_eq!(dst_already_ticked.cooldown_time.load(Ordering::Relaxed), 7);

        let dst_not_yet_ticked = HopperBlockEntity::new(BlockPos::new(0, 64, 0));
        dst_not_yet_ticked
            .ticked_game_time
            .store(99, Ordering::Relaxed);
        assert!(
            HopperBlockEntity::add_item(
                Some(&src),
                &dst_not_yet_ticked,
                ItemStack::new(1, &Item::IRON_INGOT),
                Some(BlockDirection::Up),
            )
            .is_empty()
        );
        assert_eq!(dst_not_yet_ticked.cooldown_time.load(Ordering::Relaxed), 8);
    }
}

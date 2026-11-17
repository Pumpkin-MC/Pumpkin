use crate::block::blocks::composter::ComposterBlock;
use crate::block::entities::BlockEntity;
use crate::entity::EntityBase;
use crate::world::World;
use crate::world::loot::{LootContextParameters, fill_chest_inventory_handle};
use pumpkin_data::block_properties::{FacingHopper, HopperLikeProperties};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::loot_table::get_loot_table;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockId, BlockStateId};
use pumpkin_inventory::{Clearable, Inventory, sync_write_items_to_nbt};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::any::Any;
use std::array::from_fn;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64};

pub struct HopperBlockEntity {
    pub position: BlockPos,
    pub items: RwLock<[ItemStack; Self::INVENTORY_SIZE]>,
    pub dirty: AtomicBool,
    pub comparator_dirty: AtomicBool,
    pub facing: FacingHopper,
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

/// One item taken out of a slot, with what the slot held before and after the removal.
///
/// A compare-and-swap: the offer runs without the source lock (holding it across
/// [`HopperBlockEntity::add_one_item`] deadlocks two hoppers facing each other), so the rollback
/// has to compare against `remainder` to tell an untouched slot from someone else's write.
struct Extraction {
    one_item: ItemStack,
    snapshot: ItemStack,
    remainder: ItemStack,
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
            facing: FacingHopper::Down,
            cooldown_time: AtomicI32::from(nbt.get_int("TransferCooldown").unwrap_or(-1)),
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
        if self.cooldown_time.fetch_sub(1, Ordering::Relaxed) <= 0 {
            self.cooldown_time.store(0, Ordering::Relaxed);
            if properties.enabled
                && let Some(entity) = world.get_block_entity(&self.position)
                && let Some(hopper) = entity.as_any().downcast_ref::<Self>()
            {
                hopper.try_move_items(properties, world);
            }
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

    fn set_block_state(&mut self, block_state: BlockStateId) {
        // TODO !!!IMPORTANT!!! set block state when loading the chunk
        self.facing = HopperLikeProperties::from_state_id(block_state).facing;
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

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put(
            "TransferCooldown",
            NbtTag::Int(self.cooldown_time.load(Ordering::Relaxed)),
        );
        if let Ok(items) = self.items.try_read() {
            sync_write_items_to_nbt(items.as_slice(), &mut nbt);
        }
        Some(nbt)
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

    #[must_use]
    pub fn new(position: BlockPos, facing: FacingHopper) -> Self {
        Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
            facing,
            cooldown_time: AtomicI32::new(-1),
            ticked_game_time: AtomicI64::new(0),
            metadata: super::ContainerMetadata::default(),
        }
    }
    fn try_move_items(&self, state: HopperLikeProperties, world: &Arc<World>) {
        if self.cooldown_time.load(Ordering::Relaxed) <= 0 && state.enabled {
            let mut success = if self.is_empty() {
                false
            } else {
                self.eject_items(state.facing, world)
            };
            if !self.inventory_full() {
                success |= self.suck_in_items(world);
            }
            if success {
                self.cooldown_time.store(8, Ordering::Relaxed);
                self.mark_dirty();
            }
        }
    }

    fn inventory_full(&self) -> bool {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for item in items.iter() {
            if item.is_empty() || item.item_count != item.get_max_stack_size() {
                return false;
            }
        }
        true
    }

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

    fn get_block_container(world: &Arc<World>, pos: &BlockPos) -> Option<Arc<dyn Inventory>> {
        let (block, state) = world.get_block_and_state(pos);
        if block == &Block::COMPOSTER {
            return Some(ComposterBlock::get_container(state.id, world, pos));
        }
        if block.has_tag(&tag::Block::C_CHESTS_WOODEN)
            || block.has_tag(&tag::Block::MINECRAFT_COPPER_CHESTS)
        {
            return crate::block::blocks::chests::get_chest_inventory(
                world,
                Some(world),
                pos,
                state.id,
                true,
            );
        }
        let entity = world.get_block_entity(pos)?;
        entity.set_world(Arc::downgrade(world));
        if let Some((loot_key, seed)) = entity.take_loot_table()
            && let Some(table) = get_loot_table(&loot_key)
            && let Some(inv) = entity.clone().get_inventory()
        {
            fill_chest_inventory_handle(
                &inv,
                &crate::world::loot::LootTableHandle::Static(table),
                seed,
                &LootContextParameters {
                    world: Some(world),
                    position: Some(pos.to_f64()),
                    ..Default::default()
                },
            );
            inv.mark_dirty();
            return Some(inv);
        }
        entity.get_inventory()
    }

    fn get_entity_container(
        world: &Arc<World>,
        x: f64,
        y: f64,
        z: f64,
    ) -> Option<Arc<dyn Inventory>> {
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

    fn is_full_container(container: &dyn Inventory, direction: BlockDirection) -> bool {
        if container.is_sided() {
            for &slot in container.get_slots_for_face(direction) {
                let item = container.get_stack(slot);
                let max_count = container
                    .get_max_count_per_stack()
                    .min(item.get_max_stack_size());
                if item.is_empty() || item.item_count < max_count {
                    return false;
                }
            }
        } else {
            for slot in 0..container.size() {
                let item = container.get_stack(slot);
                let max_count = container
                    .get_max_count_per_stack()
                    .min(item.get_max_stack_size());
                if item.is_empty() || item.item_count < max_count {
                    return false;
                }
            }
        }
        true
    }

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

    fn can_accept_one_item(into: &dyn Inventory, stack: &ItemStack) -> bool {
        let mut probe = stack.clone();
        probe.set_count(1);
        for slot in 0..into.size() {
            if !Self::can_place_item_in_container(into, &probe, slot, None) {
                continue;
            }
            let current = into.get_stack(slot);
            if current.is_empty() {
                return true;
            }
            let max_count = into
                .get_max_count_per_stack()
                .min(probe.get_max_stack_size());
            if current.item_count < max_count && current.are_items_and_components_equal(&probe) {
                return true;
            }
        }
        false
    }

    pub fn try_take_from_source(into: &dyn Inventory, source: &dyn Inventory) -> bool {
        let direction = BlockDirection::Down;
        if source.is_sided() {
            for &slot in source.get_slots_for_face(direction) {
                if Self::try_take_in_item_from_slot(into, source, slot, direction) {
                    return true;
                }
            }
        } else {
            for slot in 0..source.size() {
                if Self::try_take_in_item_from_slot(into, source, slot, direction) {
                    return true;
                }
            }
        }
        false
    }

    fn try_take_in_item_from_slot(
        into: &dyn Inventory,
        source: &dyn Inventory,
        slot: usize,
        direction: BlockDirection,
    ) -> bool {
        let item = source.get_stack(slot);
        if item.is_empty()
            || !Self::can_take_item_from_container(into, source, &item, slot, direction)
            || !Self::can_accept_one_item(into, &item)
        {
            return false;
        }
        let one_item = source.remove_stack_specific(slot, 1);
        if one_item.is_empty() {
            return false;
        }
        let leftover = Self::add_item(Some(source), into, one_item, None);
        if leftover.is_empty() {
            source.mark_dirty();
            return true;
        }
        let current = source.get_stack(slot);
        if current.is_empty() {
            source.set_stack(slot, leftover);
        } else if current.are_items_and_components_equal(&leftover)
            && current.item_count < current.get_max_stack_size()
        {
            let mut restored = current;
            restored.item_count += leftover.item_count;
            source.set_stack(slot, restored);
        }
        false
    }

    pub fn pickup_item_entity(
        into: &dyn Inventory,
        item_entity: &crate::entity::item::ItemEntity,
    ) -> bool {
        let copy = {
            let stack = item_entity
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if stack.is_empty() {
                return false;
            }
            stack.clone()
        };
        let original_count = copy.item_count;
        let remaining = Self::add_item(None, into, copy, None);
        if remaining.is_empty() {
            *item_entity
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = ItemStack::EMPTY.clone();
            item_entity.get_entity().remove();
            true
        } else if remaining.item_count != original_count {
            let mut stack = item_entity
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *stack = remaining;
            true
        } else {
            false
        }
    }

    fn suck_in_items(&self, world: &Arc<World>) -> bool {
        let pos_up = &self.position.up();
        let mut search_event = crate::plugin::api::events::inventory::hopper_inventory_search::HopperInventorySearchEvent::new(
            self.position,
            *pos_up,
        );
        if let Some(server) = world.server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut search_event);
        }
        if search_event.cancelled {
            return false;
        }

        if let Some(container) = Self::get_container_at(world, pos_up) {
            return Self::try_take_from_source(self, container.as_ref());
        }

        let (block, state) = world.get_block_and_state(pos_up);
        let is_blocked =
            state.is_full_cube() && !block.has_tag(&tag::Block::MINECRAFT_DOES_NOT_BLOCK_HOPPERS);
        if !is_blocked {
            let base = self.position.to_f64();
            let search_box = BoundingBox::new(
                Vector3::new(base.x, base.y + 0.6875, base.z),
                Vector3::new(base.x + 1.0, base.y + 2.0, base.z + 1.0),
            );
            for entity_base in world.get_entities_at_box(&search_box) {
                if !entity_base.get_entity().is_alive() {
                    continue;
                }
                if let Some(item_entity) = entity_base.get_item_entity() {
                    let (is_empty, registry_key) = {
                        let stack = item_entity
                            .get_item_stack()
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        (stack.is_empty(), stack.item.registry_key.to_string())
                    };
                    if !is_empty {
                        let mut pickup_event =
                            crate::plugin::api::events::inventory::inventory_pickup_item::InventoryPickupItemEvent::new(
                                self.position,
                                item_entity.get_entity().entity_id,
                                registry_key,
                            );
                        if let Some(server) = world.server.upgrade() {
                            server
                                .plugin_manager
                                .fire_blocking(&server, &mut pickup_event);
                        }
                        if pickup_event.cancelled {
                            continue;
                        }
                        if Self::pickup_item_entity(self, item_entity) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Splits one item off `slot`. One lock for read and write, so the snapshot is the state the
    /// removal really happened on and not an older one. `None` when the slot is empty by then.
    fn take_one(&self, slot: usize) -> Option<Extraction> {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if items[slot].is_empty() {
            return None;
        }
        let snapshot = items[slot].clone();
        let one_item = items[slot].split(1);
        let remainder = items[slot].clone();
        self.mark_dirty();
        Some(Extraction {
            one_item,
            snapshot,
            remainder,
        })
    }

    /// Undoes [`Self::take_one`] after a failed offer, handing the item back when the slot has no
    /// room for it any more.
    ///
    /// The snapshot only fits a slot nobody wrote to, so it is restored on a match and dropped on
    /// a mismatch, writing it anyway would undo the other write, in either direction.
    fn put_back(&self, slot: usize, extraction: Extraction) -> Option<ItemStack> {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.mark_dirty();
        let current = &mut items[slot];
        if current.are_equal(&extraction.remainder) {
            *current = extraction.snapshot;
            return None;
        }
        if current.is_empty() {
            *current = extraction.one_item;
            return None;
        }
        if current.are_items_and_components_equal(&extraction.one_item)
            && current.item_count < current.get_max_stack_size()
        {
            current.item_count += 1;
            return None;
        }
        Some(extraction.one_item)
    }

    fn eject_items(&self, facing: FacingHopper, world: &Arc<World>) -> bool {
        let direction = to_direction(facing);
        let opposite_dir = direction.opposite();
        let target_pos = self.position.offset(direction.to_offset());

        if let Some(container) = Self::get_container_at(world, &target_pos) {
            if Self::is_full_container(container.as_ref(), opposite_dir) {
                return false;
            }
            for slot in 0..Self::INVENTORY_SIZE {
                let item = self.get_stack(slot);
                if item.is_empty() {
                    continue;
                }
                let mut move_event = crate::plugin::api::events::inventory::inventory_move_item::InventoryMoveItemEvent::new(
                    self.position,
                    target_pos,
                    item.item.registry_key.to_string(),
                    1,
                );
                if let Some(server) = world.server.upgrade() {
                    server
                        .plugin_manager
                        .fire_blocking(&server, &mut move_event);
                }
                if move_event.cancelled {
                    continue;
                }
                // Vanilla `HopperBlockEntity.ejectItems`: actually remove the item from the
                // hopper before offering it to the target, restoring it on failure. Reading a
                // clone and never writing back left the source stack untouched, duplicating
                // the item into the target while the hopper kept its full stack.
                let Some(extraction) = self.take_one(slot) else {
                    // Emptied while the event was firing.
                    continue;
                };
                if Self::add_item(
                    Some(self),
                    container.as_ref(),
                    extraction.one_item.clone(),
                    Some(opposite_dir),
                )
                .is_empty()
                {
                    return true;
                }
                if let Some(leftover) = self.put_back(slot, extraction) {
                    // Slot is someone else's now and full -> dropping beats overwriting or voiding.
                    let pos = self.position.to_centered_f64();
                    world.scatter_stack(pos.x, pos.y, pos.z, leftover);
                }
            }
        }
        false
    }

    pub fn add_one_item(from: &dyn Inventory, to: &dyn Inventory, item: &ItemStack) -> bool {
        Self::add_item(Some(from), to, item.clone(), None).is_empty()
    }

    pub fn add_item(
        from: Option<&dyn Inventory>,
        to: &dyn Inventory,
        mut stack: ItemStack,
        direction: Option<BlockDirection>,
    ) -> ItemStack {
        if to.is_sided()
            && let Some(dir) = direction
        {
            for &slot in to.get_slots_for_face(dir) {
                if stack.is_empty() {
                    break;
                }
                stack = Self::try_move_in_item(from, to, stack, slot, direction);
            }
            return stack;
        }

        let size = to.size();
        for slot in 0..size {
            if stack.is_empty() {
                break;
            }
            stack = Self::try_move_in_item(from, to, stack, slot, direction);
        }
        stack
    }

    fn try_move_in_item(
        from: Option<&dyn Inventory>,
        to: &dyn Inventory,
        mut stack: ItemStack,
        slot: usize,
        direction: Option<BlockDirection>,
    ) -> ItemStack {
        if !Self::can_place_item_in_container(to, &stack, slot, direction) {
            return stack;
        }

        let mut current = to.get_stack(slot);
        let was_empty = to.is_empty();
        let max_count = to.get_max_count_per_stack().min(stack.get_max_stack_size());
        let mut success = false;

        if current.is_empty() {
            if stack.item_count > max_count {
                to.set_stack(slot, stack.split(max_count));
            } else {
                to.set_stack(slot, stack);
                stack = ItemStack::EMPTY.clone();
            }
            success = true;
        } else if current.item_count < max_count && current.are_items_and_components_equal(&stack) {
            let space = max_count - current.item_count;
            let count = stack.item_count.min(space);
            if count > 0 {
                stack.decrement(count);
                current.item_count += count;
                to.set_stack(slot, current);
                success = true;
            }
        }

        if success {
            if was_empty
                && let Some(hopper) = to.as_any().downcast_ref::<Self>()
                && hopper.cooldown_time.load(Ordering::Relaxed) <= 8
            {
                let skip_tick = i32::from(
                    from.and_then(|src| src.as_any().downcast_ref::<Self>())
                        .is_some_and(|from_hopper| {
                            hopper.ticked_game_time.load(Ordering::Relaxed)
                                >= from_hopper.ticked_game_time.load(Ordering::Relaxed)
                        }),
                );
                hopper.cooldown_time.store(8 - skip_tick, Ordering::Relaxed);
            }
            to.mark_dirty();
        }

        stack
    }
}

impl Inventory for HopperBlockEntity {
    fn size(&self) -> usize {
        Self::INVENTORY_SIZE
    }

    fn is_empty(&self) -> bool {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.iter().all(ItemStack::is_empty)
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items[slot].clone()
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let removed = std::mem::replace(&mut items[slot], ItemStack::EMPTY.clone());
        self.mark_dirty();
        removed
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let res = if !items[slot].is_empty() && amount > 0 {
            items[slot].split(amount)
        } else {
            ItemStack::EMPTY.clone()
        };
        self.mark_dirty();
        res
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items[slot] = stack;
        self.mark_dirty();
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
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.fill_with(|| ItemStack::EMPTY.clone());
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::{Block, item::Item};

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

    fn hopper_holding(stack: ItemStack) -> HopperBlockEntity {
        let hopper = HopperBlockEntity::new(BlockPos::new(0, 0, 0), FacingHopper::Down);
        hopper.set_stack(0, stack);
        hopper
    }

    #[test]
    fn take_one_splits_a_single_item_off() {
        let hopper = hopper_holding(ItemStack::new(10, &Item::DIAMOND));

        let extraction = hopper.take_one(0).unwrap();

        assert_eq!(extraction.one_item.item_count, 1);
        assert_eq!(extraction.snapshot.item_count, 10);
        assert_eq!(extraction.remainder.item_count, 9);
        assert_eq!(hopper.get_stack(0).item_count, 9);
    }

    #[test]
    fn take_one_on_an_empty_slot_extracts_nothing() {
        let hopper = hopper_holding(ItemStack::EMPTY.clone());

        assert!(hopper.take_one(0).is_none());
    }

    #[test]
    fn untouched_slot_gets_the_snapshot_back() {
        let hopper = hopper_holding(ItemStack::new(10, &Item::DIAMOND));
        let extraction = hopper.take_one(0).unwrap();

        assert!(hopper.put_back(0, extraction).is_none());
        assert_eq!(hopper.get_stack(0).item_count, 10);
    }

    /// The snapshot says 10, the slot says 3 because someone took 7 while the offer was out.
    /// Restoring the snapshot would conjure those 7 back, so only the one item returns.
    #[test]
    fn changed_count_takes_back_one_item_not_the_snapshot() {
        let hopper = hopper_holding(ItemStack::new(10, &Item::DIAMOND));
        let extraction = hopper.take_one(0).unwrap();
        hopper.set_stack(0, ItemStack::new(3, &Item::DIAMOND));

        assert!(hopper.put_back(0, extraction).is_none());
        assert_eq!(hopper.get_stack(0).item_count, 4);
    }

    /// A foreign item cannot absorb the one item and must not be overwritten, so nothing fits and
    /// the item comes back out.
    #[test]
    fn foreign_item_in_the_slot_is_left_alone() {
        let hopper = hopper_holding(ItemStack::new(1, &Item::DIAMOND));
        let extraction = hopper.take_one(0).unwrap();
        hopper.set_stack(0, ItemStack::new(64, &Item::DIRT));

        let leftover = hopper.put_back(0, extraction).unwrap();

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
        let extraction = hopper.take_one(0).unwrap();
        hopper.set_stack(0, ItemStack::EMPTY.clone());

        assert!(hopper.put_back(0, extraction).is_none());
        assert_eq!(hopper.get_stack(0).item_count, 1);
    }

    /// Same item but no room left. Incrementing would push it past `get_max_stack_size`.
    #[test]
    fn full_slot_of_the_same_item_hands_the_item_back() {
        let hopper = hopper_holding(ItemStack::new(10, &Item::DIAMOND));
        let extraction = hopper.take_one(0).unwrap();
        let max = ItemStack::new(1, &Item::DIAMOND).get_max_stack_size();
        hopper.set_stack(0, ItemStack::new(max, &Item::DIAMOND));

        let leftover = hopper.put_back(0, extraction).unwrap();

        assert_eq!(leftover.item_count, 1);
        assert_eq!(hopper.get_stack(0).item_count, max);
    }

    #[test]
    fn furnace_worldly_container_respects_faces_fuel_and_empty_bucket() {
        use crate::block::entities::furnace::FurnaceBlockEntity;
        let furnace = FurnaceBlockEntity::new(BlockPos::new(0, 64, 0));
        let dest_hopper = HopperBlockEntity::new(BlockPos::new(0, 63, 0), FacingHopper::Down);

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
        assert!(!HopperBlockEntity::try_take_from_source(
            &dest_hopper,
            &furnace
        ));

        // DOWN DOES pull an empty bucket out of fuel slot 1 (after lava bucket consumed)
        furnace.set_stack(1, ItemStack::new(1, &Item::BUCKET));
        assert!(HopperBlockEntity::try_take_from_source(
            &dest_hopper,
            &furnace
        ));
        assert!(furnace.get_stack(1).is_empty());
        assert_eq!(dest_hopper.get_stack(0).item.id, Item::BUCKET.id);
    }

    #[test]
    fn brewing_stand_worldly_container_sides_and_glass_bottle_extraction() {
        use crate::block::entities::brewing_stand::BrewingStandBlockEntity;
        let stand = BrewingStandBlockEntity::new(BlockPos::new(0, 64, 0));
        let dest_hopper = HopperBlockEntity::new(BlockPos::new(0, 63, 0), FacingHopper::Down);

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
        assert!(!HopperBlockEntity::try_take_from_source(
            &dest_hopper,
            &stand
        ));

        stand.set_stack(3, ItemStack::new(1, &Item::GLASS_BOTTLE));
        assert!(HopperBlockEntity::try_take_from_source(
            &dest_hopper,
            &stand
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
        let src = HopperBlockEntity::new(BlockPos::new(0, 65, 0), FacingHopper::Down);
        let dst_already_ticked =
            HopperBlockEntity::new(BlockPos::new(0, 64, 0), FacingHopper::Down);
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

        let dst_not_yet_ticked =
            HopperBlockEntity::new(BlockPos::new(0, 64, 0), FacingHopper::Down);
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

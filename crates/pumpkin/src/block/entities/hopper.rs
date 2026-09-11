use crate::block::entities::BlockEntity;
use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::world::World;
use pumpkin_data::block_properties::{FacingHopper, HopperLikeProperties};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{BlockId, BlockStateId};
use pumpkin_inventory::{Clearable, Inventory, sync_write_items_to_nbt};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
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
    pub facing: FacingHopper,
    pub cooldown_time: AtomicI32,
    pub ticked_game_time: AtomicI64,
}

#[must_use]
pub fn to_offset(facing: &FacingHopper) -> Vector3<i32> {
    match facing {
        FacingHopper::Down => (0, -1, 0),
        FacingHopper::North => (0, 0, -1),
        FacingHopper::South => (0, 0, 1),
        FacingHopper::West => (-1, 0, 0),
        FacingHopper::East => (1, 0, 0),
    }
    .into()
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
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let mut hopper = Self {
            position,
            items: RwLock::new(from_fn(|_| ItemStack::EMPTY.clone())),
            dirty: AtomicBool::new(false),
            facing: FacingHopper::Down,
            cooldown_time: AtomicI32::from(nbt.get_int("TransferCooldown").unwrap_or(-1)),
            ticked_game_time: AtomicI64::new(0),
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
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
            facing,
            cooldown_time: AtomicI32::new(-1),
            ticked_game_time: AtomicI64::new(0),
        }
    }
    fn try_move_items(&self, state: HopperLikeProperties, world: &Arc<World>) {
        if self.cooldown_time.load(Ordering::Relaxed) <= 0 && state.enabled {
            let mut success = if self.is_empty() {
                false
            } else {
                self.eject_items(world)
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

    #[allow(clippy::too_many_lines)]
    fn suck_in_items(&self, world: &Arc<World>) -> bool {
        // TODO getEntityContainer
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

        if let Some(entity) = world.get_block_entity(pos_up)
            && let Some(container) = entity.clone().get_inventory()
        {
            // TODO check WorldlyContainer
            for i in 0..container.size() {
                let mut item = container.get_stack(i);
                if !item.is_empty() && container.can_transfer_to(self, i, &item) {
                    //TODO WorldlyContainer
                    let _backup = item.clone();
                    let one_item = item.split(1);
                    if Self::add_one_item(container.as_ref(), self, &one_item) {
                        container.set_stack(i, item);
                        // If extracting from furnace output slot (index 2), drop XP as orbs
                        let furnace_output_slot: usize = 2;
                        if i == furnace_output_slot
                            && let Some(experience_container) =
                                entity.clone().to_experience_container()
                        {
                            let xp = experience_container.extract_experience();
                            if xp > 0 {
                                let pos = self.position.to_f64();
                                ExperienceOrbEntity::spawn(world, pos, xp as u32);
                            }
                        }
                        return true;
                    }
                }
            }
            return false;
        }
        let (block, state) = world.get_block_and_state(pos_up);
        if !(state.is_solid() && block.has_tag(&tag::Block::MINECRAFT_DOES_NOT_BLOCK_HOPPERS)) {
            let pos_up_f = pos_up.to_f64();
            let search_box = pumpkin_util::math::boundingbox::BoundingBox::new(
                pos_up_f,
                pos_up_f.add_raw(1.0, 1.0, 1.0),
            );
            let entities = world.get_entities_at_box(&search_box);
            for entity_base in entities {
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
                        let (backup, one_item, is_empty) = {
                            let mut stack = item_entity
                                .get_item_stack()
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            if stack.is_empty() {
                                continue;
                            }
                            let backup = stack.clone();
                            let one_item = stack.split(1);
                            let is_empty = stack.is_empty();
                            (backup, one_item, is_empty)
                        };
                        if Self::add_one_item(self, self, &one_item) {
                            if is_empty {
                                item_entity.get_entity().remove();
                            }
                            return true;
                        }
                        let mut stack = item_entity
                            .get_item_stack()
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        *stack = backup;
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

    fn eject_items(&self, world: &Arc<World>) -> bool {
        // TODO getEntityContainer

        if let Some(entity) = world.get_block_entity(&self.position.offset(to_offset(&self.facing)))
            && let Some(container) = entity.get_inventory()
        {
            // TODO check WorldlyContainer
            let mut is_full = true;
            for i in 0..container.size() {
                let item = container.get_stack(i);
                if item.item_count < item.get_max_stack_size() {
                    is_full = false;
                    break;
                }
            }
            if is_full {
                return false;
            }
            let target_pos = self.position.offset(to_offset(&self.facing));
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
                    // Emptied while the event was firing. An empty stack takes `add_one_item`'s
                    // `dst.is_empty()` branch and reports a transfer that never happened.
                    continue;
                };
                if Self::add_one_item(self, container.as_ref(), &extraction.one_item) {
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
        let mut success = false;
        let to_empty = to.is_empty();
        for j in 0..to.size() {
            if to.is_valid_slot_for(j, item) {
                let mut dst = to.get_stack(j);
                if dst.is_empty() {
                    dst = item.clone();
                    to.set_stack(j, dst);
                    success = true;
                } else if dst.item_count < dst.get_max_stack_size() && dst.item == item.item {
                    // TODO check Components equal
                    dst.item_count += 1;
                    to.set_stack(j, dst);
                    success = true;
                }
                if success {
                    if to_empty
                        && let Some(hopper) = to.as_any().downcast_ref::<Self>()
                        && hopper.cooldown_time.load(Ordering::Relaxed) <= 8
                    {
                        if let Some(from_hopper) = from.as_any().downcast_ref::<Self>() {
                            if from_hopper.cooldown_time.load(Ordering::Relaxed)
                                >= hopper.cooldown_time.load(Ordering::Relaxed)
                            {
                                hopper.cooldown_time.store(7, Ordering::Relaxed);
                            } else {
                                hopper.cooldown_time.store(8, Ordering::Relaxed);
                            }
                        } else {
                            hopper.cooldown_time.store(8, Ordering::Relaxed);
                        }
                    }
                    to.mark_dirty();
                    return true;
                }
            }
        }
        false
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
}

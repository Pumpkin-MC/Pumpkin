use super::BlockEntity;
use crate::world::World;
use crate::world::loot::{LootContextParameters, generate_loot_from_handle, place_items_in_slots};
use pumpkin_data::data_component_impl::{
    ContainerImpl, ContainerLootImpl, DataComponentImpl, PotDecorationsImpl,
};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::{Clearable, Inventory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;
use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

pub struct DecoratedPotBlockEntity {
    pub position: BlockPos,
    pub decorations: Mutex<PotDecorationsImpl>,
    pub item: Mutex<Option<ItemStack>>,
    pub loot_table: Mutex<Option<String>>,
    pub loot_table_seed: AtomicI64,
    pub dirty: AtomicBool,
    pub comparator_dirty: AtomicBool,
}

// TODO vanilla `getTheItem`/`setTheItem`/`splitTheItem` unpack the loot table on every slot access.
// Hopper `get_container` unpacks once; other inventory reads still skip it.
impl BlockEntity for DecoratedPotBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let decorations = nbt
            .get("sherds")
            .and_then(PotDecorationsImpl::read_data)
            .unwrap_or(PotDecorationsImpl::EMPTY);
        let loot_table = nbt.get_string("LootTable").map(ToString::to_string);
        let loot_table_seed = nbt.get_long("LootTableSeed").unwrap_or(0);
        let item = if loot_table.is_some() {
            None
        } else {
            nbt.get_compound("item")
                .and_then(ItemStack::read_item_stack)
                .filter(|s| !s.is_empty())
        };
        Self {
            position,
            decorations: Mutex::new(decorations),
            item: Mutex::new(item),
            loot_table: Mutex::new(loot_table),
            loot_table_seed: AtomicI64::new(loot_table_seed),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(decorations) = self.decorations.lock()
            && !decorations.is_empty()
        {
            nbt.put("sherds", decorations.write_data());
        }
        let loot_key = self
            .loot_table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if let Some(key) = loot_key {
            nbt.put_string("LootTable", key);
            let seed = self.loot_table_seed.load(Ordering::Relaxed);
            if seed != 0 {
                nbt.put_long("LootTableSeed", seed);
            }
        } else if let Ok(item) = self.item.lock()
            && let Some(it) = item.as_ref()
            && let Some(it_nbt) = it.to_nbt()
        {
            nbt.put_compound("item", it_nbt);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(decorations) = self.decorations.try_lock()
            && !decorations.is_empty()
        {
            nbt.put("sherds", decorations.write_data());
        }
        let has_loot = self
            .loot_table
            .try_lock()
            .map_or(true, |guard| guard.is_some());
        if !has_loot
            && let Ok(item) = self.item.try_lock()
            && let Some(ref it) = *item
            && let Some(it_nbt) = it.to_nbt()
        {
            nbt.put_compound("item", it_nbt);
        }
        Some(nbt)
    }

    fn bedrock_block_actor_data(
        &self,
        _state_id: pumpkin_data::BlockStateId,
    ) -> Option<NbtCompound> {
        let tags = self.decorations.lock().ok().map(|dec| {
            [&dec.back, &dec.left, &dec.right, &dec.front]
                .into_iter()
                .map(|side| {
                    let name = side.as_ref().map_or_else(
                        || "minecraft:brick".to_string(),
                        |s| format!("minecraft:{}", s.item.registry_key),
                    );
                    NbtTag::String(name.into())
                })
                .collect::<Vec<_>>()
        });
        Some(build_decorated_pot_actor(self.position, tags.as_deref()))
    }

    fn collect_item_components(&self, stack: &mut ItemStack) {
        let decorations = self
            .decorations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        stack.set_data_component(decorations);
    }

    fn apply_item_components(&self, stack: &ItemStack) {
        let decorations = stack
            .get_data_component::<PotDecorationsImpl>()
            .cloned()
            .unwrap_or(PotDecorationsImpl::EMPTY);
        *self
            .decorations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = decorations;

        if let Some(loot) = stack.get_data_component::<ContainerLootImpl>() {
            *self
                .loot_table
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(loot.loot_table.clone());
            self.loot_table_seed.store(loot.seed, Ordering::Relaxed);
        }
        if let Some(container) = stack.get_data_component::<ContainerImpl>()
            && let Some((_, it)) = container.items.first()
            && !it.is_empty()
        {
            *self
                .item
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(it.clone());
        }
    }

    fn loot_table_key(&self) -> Option<String> {
        self.loot_table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn has_loot_table(&self) -> bool {
        self.loot_table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    fn on_block_replaced(self: Arc<Self>, world: &Arc<World>, position: &BlockPos) {
        Self::unpack_loot_table(&self, world);
        let inventory: Arc<dyn Inventory> = self;
        world.scatter_inventory(position, &inventory);
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn is_comparator_dirty(&self) -> bool {
        self.comparator_dirty.load(Ordering::Relaxed)
    }

    fn clear_comparator_dirty(&self) {
        self.comparator_dirty.store(false, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DecoratedPotBlockEntity {
    pub const ID: &'static str = "minecraft:decorated_pot";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            decorations: Mutex::new(PotDecorationsImpl::EMPTY),
            item: Mutex::new(None),
            loot_table: Mutex::new(None),
            loot_table_seed: AtomicI64::new(0),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
        }
    }

    /// Takes the pending loot-table key and seed.
    fn take_loot_table(&self) -> Option<(String, i64)> {
        let mut guard = self
            .loot_table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard
            .take()
            .map(|key| (key, self.loot_table_seed.swap(0, Ordering::Relaxed)))
    }

    pub fn unpack_loot_table(this: &Arc<Self>, world: &Arc<World>) {
        if let Some((loot_key, seed)) = this.take_loot_table()
            && let Some(handle) = world.get_loot_table(&loot_key)
        {
            // Generate without the item lock, then check the slot and place under one lock so a
            // concurrently inserted item is kept. Like vanilla `LootTable.fill` with no empty
            // slot, the loot is dropped with a warning.
            let generated = generate_loot_from_handle(
                &handle,
                seed,
                &LootContextParameters {
                    world: Some(world),
                    // TODO vanilla `RandomizableContainer.unpackLootTable` ORIGIN is `Vec3.atCenterOf`.
                    position: Some(this.position.to_f64()),
                    ..Default::default()
                },
            );
            let mut item = this
                .item
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut slots = [item.take().unwrap_or_else(|| ItemStack::EMPTY.clone())];
            place_items_in_slots(&mut slots, generated, seed);
            let [slot] = slots;
            *item = (!slot.is_empty()).then_some(slot);
            drop(item);
            this.mark_dirty();
        }
    }

    pub fn get_decorations(&self) -> PotDecorationsImpl {
        self.decorations
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn get_item(&self) -> Option<ItemStack> {
        self.item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn take_item(&self) -> Option<ItemStack> {
        let taken = self
            .item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if taken.is_some() {
            self.mark_dirty();
        }
        taken
    }

    /// Inserts up to `count` items from `stack`, returning `Some((new_count, max_stack_size))` on success.
    pub fn try_insert_item(&self, stack: &mut ItemStack, count: u8) -> Option<(u8, u8)> {
        if stack.is_empty() || count == 0 {
            return None;
        }
        let mut item_guard = self
            .item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(existing) = item_guard.as_mut() {
            if existing.is_empty() {
                let max_stack = stack.get_max_stack_size();
                let insert_count = count.min(stack.item_count).min(max_stack);
                let mut inserted = stack.clone();
                inserted.item_count = insert_count;
                *item_guard = Some(inserted);
                stack.item_count -= insert_count;
                drop(item_guard);
                self.mark_dirty();
                return Some((insert_count, max_stack));
            }
            if existing.are_items_and_components_equal(stack) {
                let max_stack = existing.get_max_stack_size();
                let space = max_stack.saturating_sub(existing.item_count);
                let add = count.min(space).min(stack.item_count);
                if add > 0 {
                    existing.item_count += add;
                    stack.item_count -= add;
                    let new_count = existing.item_count;
                    drop(item_guard);
                    self.mark_dirty();
                    return Some((new_count, max_stack));
                }
            }
            None
        } else {
            let max_stack = stack.get_max_stack_size();
            let insert_count = count.min(stack.item_count).min(max_stack);
            let mut inserted = stack.clone();
            inserted.item_count = insert_count;
            *item_guard = Some(inserted);
            stack.item_count -= insert_count;
            drop(item_guard);
            self.mark_dirty();
            Some((insert_count, max_stack))
        }
    }

    pub fn get_comparator_output(&self) -> u8 {
        self.item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .map_or(0, |item| {
                if item.item_count == 0 {
                    0
                } else {
                    // Vanilla scales by the item's own stack size, not a fixed 64.
                    // `try_insert_item` may hold more than one stack, so cap the ratio if full.
                    let max_count = f32::from(item.get_max_stack_size());
                    let filled = (f32::from(item.item_count) / max_count).min(1.0);
                    1 + (filled * 14.0).floor() as u8
                }
            })
    }
}

impl Inventory for DecoratedPotBlockEntity {
    fn size(&self) -> usize {
        1
    }

    fn is_empty(&self) -> bool {
        self.item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .is_none_or(ItemStack::is_empty)
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        if slot != 0 {
            return ItemStack::EMPTY.clone();
        }
        self.item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
            .unwrap_or_else(|| ItemStack::EMPTY.clone())
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        if slot != 0 {
            return ItemStack::EMPTY.clone();
        }
        let removed = self
            .item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .unwrap_or_else(|| ItemStack::EMPTY.clone());
        self.mark_dirty();
        removed
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        if slot != 0 || amount == 0 {
            return ItemStack::EMPTY.clone();
        }
        let mut guard = self
            .item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(existing) = guard.as_mut() else {
            return ItemStack::EMPTY.clone();
        };
        let split = existing.split(amount);
        if existing.is_empty() {
            *guard = None;
        }
        drop(guard);
        self.mark_dirty();
        split
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        if slot != 0 {
            return;
        }
        *self
            .item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            (!stack.is_empty()).then_some(stack);
        self.mark_dirty();
    }

    fn update_stack(&self, slot: usize, update: &mut dyn FnMut(&mut ItemStack) -> bool) -> bool {
        if slot != 0 {
            return false;
        }
        // One lock for read and write, a parallel insert can't be overwritten.
        let mut guard = self
            .item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut stack = guard.take().unwrap_or_else(|| ItemStack::EMPTY.clone());
        let changed = update(&mut stack);
        *guard = (!stack.is_empty()).then_some(stack);
        changed
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
        self.comparator_dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for DecoratedPotBlockEntity {
    fn clear(&self) {
        *self
            .item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        self.mark_dirty();
    }
}

/// Builds the Bedrock block-actor NBT for decorated pots.
#[must_use]
pub fn build_decorated_pot_actor(position: BlockPos, sherds: Option<&[NbtTag]>) -> NbtCompound {
    let default_brick = NbtTag::String("minecraft:brick".into());
    let sherd_tags = if let Some(tags) = sherds
        && tags.len() >= 4
    {
        tags.iter().take(4).cloned().collect()
    } else {
        vec![
            default_brick.clone(),
            default_brick.clone(),
            default_brick.clone(),
            default_brick,
        ]
    };

    super::bedrock::BedrockBlockActorBuilder::new("DecoratedPot", position)
        .put_list("sherds", sherd_tags)
        .finish()
}

#[cfg(test)]
mod tests {
    use pumpkin_data::item::Item;

    use super::*;

    #[test]
    fn decorated_pot_actor_builder() {
        let pos = BlockPos::new(2, 64, 2);
        let sherds = vec![
            NbtTag::String("minecraft:archer_pottery_sherd".into()),
            NbtTag::String("minecraft:brick".into()),
            NbtTag::String("minecraft:prize_pottery_sherd".into()),
            NbtTag::String("minecraft:miner_pottery_sherd".into()),
        ];
        let nbt = build_decorated_pot_actor(pos, Some(&sherds));
        assert_eq!(nbt.get_string("id"), Some("DecoratedPot"));
        let list = nbt.get_list("sherds").expect("sherds missing");
        assert_eq!(list.len(), 4);
        assert_eq!(
            list[0].extract_string(),
            Some("minecraft:archer_pottery_sherd")
        );
        assert_eq!(list[1].extract_string(), Some("minecraft:brick"));
    }

    #[test]
    fn decorated_pot_26_3_compound_and_legacy_list_nbt_roundtrip() {
        let pos = BlockPos::new(1, 70, 3);
        let mut sherds_compound = NbtCompound::new();
        let mut back_template = NbtCompound::new();
        back_template.put_string("id", "minecraft:flow_pottery_sherd".to_string());
        sherds_compound.put_compound("back", back_template);
        sherds_compound.put_string("front", "minecraft:brick".to_string());

        let mut nbt = NbtCompound::new();
        nbt.put_compound("sherds", sherds_compound);
        nbt.put_string(
            "LootTable",
            "minecraft:pots/trial_chambers/corridor".to_string(),
        );
        nbt.put_long("LootTableSeed", 42);

        let pot = DecoratedPotBlockEntity::from_nbt(&nbt, pos);
        assert!(pot.has_loot_table());
        let decorations = pot.get_decorations();
        assert_eq!(
            decorations.back.as_ref().map(|s| s.item.id),
            Some(Item::FLOW_POTTERY_SHERD.id)
        );
        assert!(decorations.left.is_none());
        assert!(decorations.right.is_none());
        assert_eq!(
            decorations.front.as_ref().map(|s| s.item.id),
            Some(Item::BRICK.id)
        );

        let mut written = NbtCompound::new();
        pot.write_nbt(&mut written);
        assert_eq!(
            written.get_string("LootTable"),
            Some("minecraft:pots/trial_chambers/corridor")
        );
        assert_eq!(written.get_long("LootTableSeed"), Some(42));
        let written_sherds = written
            .get_compound("sherds")
            .expect("26.3 sherds compound");
        assert!(written_sherds.get_compound("back").is_some());
        assert!(written_sherds.get("left").is_none());
        assert!(written_sherds.get("right").is_none());
        assert!(written_sherds.get_compound("front").is_some());

        // Stored items must not be copied into ContainerImpl on block drop.
        pot.set_stack(0, ItemStack::new(5, &Item::EMERALD));
        let mut dropped = ItemStack::new(1, &Item::DECORATED_POT);
        pot.collect_item_components(&mut dropped);
        assert_eq!(
            dropped.get_data_component::<PotDecorationsImpl>(),
            Some(&decorations)
        );
        assert!(
            dropped
                .get_data_component::<pumpkin_data::data_component_impl::ContainerImpl>()
                .is_none_or(|c| c.items.is_empty())
        );

        // Legacy 4-string List fallback.
        let mut legacy_nbt = NbtCompound::new();
        legacy_nbt.put_list(
            "sherds",
            vec![
                NbtTag::String("minecraft:archer_pottery_sherd".into()),
                NbtTag::String("minecraft:brick".into()),
                NbtTag::String("minecraft:prize_pottery_sherd".into()),
                NbtTag::String("minecraft:miner_pottery_sherd".into()),
            ],
        );
        let legacy_pot = DecoratedPotBlockEntity::from_nbt(&legacy_nbt, pos);
        let legacy_dec = legacy_pot.get_decorations();
        assert_eq!(
            legacy_dec.back.as_ref().map(|s| s.item.id),
            Some(Item::ARCHER_POTTERY_SHERD.id)
        );
        assert_eq!(
            legacy_dec.left.as_ref().map(|s| s.item.id),
            Some(Item::BRICK.id)
        );
        assert_eq!(
            legacy_dec.right.as_ref().map(|s| s.item.id),
            Some(Item::PRIZE_POTTERY_SHERD.id)
        );
        assert_eq!(
            legacy_dec.front.as_ref().map(|s| s.item.id),
            Some(Item::MINER_POTTERY_SHERD.id)
        );
    }
}

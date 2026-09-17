use super::{
    BlockEntity,
    components::{BlockEntityComponents, ComponentFields, ComponentMap, set_component},
};
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{ContainerImpl, ContainerLootImpl};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;

pub struct DecoratedPotBlockEntity {
    pub position: BlockPos,
    pub components: BlockEntityComponents,
    pub item: Mutex<Option<ItemStack>>,
    pub pending_loot: Mutex<Option<ContainerLootImpl>>,
}

impl BlockEntity for DecoratedPotBlockEntity {
    /// Returns pot decorations and retained additions without exposing its pending loot as a component.
    fn component_state(&self) -> Option<&BlockEntityComponents> {
        Some(&self.components)
    }

    /// Excludes the live single-slot container from retained item additions.
    fn consumed_components(&self) -> &'static [DataComponent] {
        &[DataComponent::Container]
    }

    /// Takes the pot's pending loot without consuming an unrelated retained item component.
    fn take_loot_table(&self) -> Option<(String, i64)> {
        self.pending_loot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .map(|loot| (loot.loot_table, loot.seed))
    }

    /// Reports whether the pot has an unopened loot table stored in block NBT.
    fn has_loot_table(&self) -> bool {
        self.pending_loot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    /// Adds the current contained item without unpacking a pending loot table.
    fn collect_implicit_components(&self, components: &mut ComponentMap) {
        let items = self
            .get_item()
            .filter(|item| !item.is_empty())
            .map(|item| (0, item))
            .into_iter()
            .collect();
        set_component(components, Box::new(ContainerImpl { items }));
    }

    /// Restores the first container slot while discarding out-of-range item slots.
    fn apply_implicit_components(&self, stack: &ItemStack) {
        *self
            .item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = stack
            .get_data_component::<ContainerImpl>()
            .and_then(|container| {
                container
                    .items
                    .iter()
                    .find(|(slot, _)| *slot == 0)
                    .map(|(_, item)| item.clone())
            });
    }

    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    /// Loads ordered pot decorations, retained additions, and the contained item.
    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let components = BlockEntityComponents::from_nbt(nbt, Self::COMPONENT_FIELDS);
        let pending_loot = nbt.get_string("LootTable").map(|key| ContainerLootImpl {
            loot_table: key.to_owned(),
            seed: nbt.get_long("LootTableSeed").unwrap_or(0),
        });
        let item = pending_loot
            .is_none()
            .then(|| {
                nbt.get_compound("item")
                    .and_then(ItemStack::read_item_stack)
            })
            .flatten();
        Self {
            position,
            components,
            item: Mutex::new(item),
            pending_loot: Mutex::new(pending_loot),
        }
    }

    /// Saves decorations and item state while preserving deferred loot without opening it.
    fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.components.write_nbt(nbt);
        if !self.write_pending_loot(nbt)
            && let Ok(item) = self.item.lock()
            && let Some(it) = item.as_ref()
        {
            let mut it_nbt = NbtCompound::new();
            it.write_item_stack(&mut it_nbt);
            nbt.put_compound("item", it_nbt);
        }
    }

    /// Encodes the visible pot decorations and item state for chunk updates.
    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.components.write_nbt(&mut nbt);
        if !self.write_pending_loot(&mut nbt)
            && let Ok(item) = self.item.try_lock()
            && let Some(ref it) = *item
        {
            let mut it_nbt = NbtCompound::new();
            it.write_item_stack(&mut it_nbt);
            nbt.put_compound("item", it_nbt);
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DecoratedPotBlockEntity {
    const COMPONENT_FIELDS: ComponentFields = &[(DataComponent::PotDecorations, "sherds")];
    pub const ID: &'static str = "minecraft:decorated_pot";

    /// Creates an empty undecorated pot at the supplied position.
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            components: BlockEntityComponents::new(Self::COMPONENT_FIELDS),
            item: Mutex::new(None),
            pending_loot: Mutex::new(None),
        }
    }

    /// Persists the pending key and seed, returning whether the contained item must be omitted.
    fn write_pending_loot(&self, nbt: &mut NbtCompound) -> bool {
        let loot = self
            .pending_loot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        loot.as_ref().is_some_and(|loot| {
            nbt.put_string("LootTable", loot.loot_table.clone());
            if loot.seed != 0 {
                nbt.put_long("LootTableSeed", loot.seed);
            }
            true
        })
    }

    pub fn get_item(&self) -> Option<ItemStack> {
        self.item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn take_item(&self) -> Option<ItemStack> {
        self.item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }

    pub fn try_insert_item(&self, stack: &mut ItemStack, count: u8) -> bool {
        let mut item_guard = self
            .item
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(existing) = item_guard.as_mut() {
            if existing.item.id == stack.item.id {
                // Vanilla gates on the item's own max stack size, not a fixed 64.
                // Saturate so a count loaded from NBT above that limit cannot underflow.
                let space = existing
                    .get_max_stack_size()
                    .saturating_sub(existing.item_count);
                let add = count.min(space).min(stack.item_count);
                if add > 0 {
                    existing.item_count += add;
                    stack.item_count -= add;
                    return true;
                }
            }
            false
        } else {
            let insert_count = count.min(stack.item_count).min(stack.get_max_stack_size());
            let mut inserted = stack.clone();
            inserted.item_count = insert_count;
            *item_guard = Some(inserted);
            stack.item_count -= insert_count;
            true
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

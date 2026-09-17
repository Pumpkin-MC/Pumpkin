use std::sync::Mutex;

use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{ContainerLootImpl, DataComponentImpl, read_data};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};

/// A snapshot of the components available to a block's loot functions.
pub type ComponentMap = Vec<(DataComponent, Box<dyn DataComponentImpl>)>;

/// Persistent fields consumed by one block-entity family when placing an item.
pub type ComponentFields = &'static [(DataComponent, &'static str)];

pub const CONTAINER_FIELDS: ComponentFields = &[
    (DataComponent::CustomName, "CustomName"),
    (DataComponent::Lock, "lock"),
];
pub const RANDOMIZABLE_CONTAINER_FIELDS: ComponentFields = &[
    (DataComponent::CustomName, "CustomName"),
    (DataComponent::Lock, "lock"),
    (DataComponent::ContainerLoot, "LootTable"),
];

/// Owns retained item additions and scalar implicit fields without a position cache.
///
/// The component registry bounds the number of entries. Dynamic fields such as inventories
/// remain in their existing owner and replace retained values during collection.
pub struct BlockEntityComponents {
    values: Mutex<ComponentValues>,
    fields: ComponentFields,
}

struct ComponentValues {
    retained: ComponentMap,
    implicit: ComponentMap,
}

impl BlockEntityComponents {
    /// Creates empty component storage with the family's persistent field mapping.
    #[must_use]
    pub const fn new(fields: ComponentFields) -> Self {
        Self {
            values: Mutex::new(ComponentValues {
                retained: Vec::new(),
                implicit: Vec::new(),
            }),
            fields,
        }
    }

    /// Loads retained additions and implicit fields, giving the implicit fields precedence.
    pub fn from_nbt(nbt: &NbtCompound, fields: ComponentFields) -> Self {
        let mut retained = Vec::new();
        let mut implicit = Vec::new();
        if let Some(components) = nbt.get_compound("components") {
            for (name, tag) in &components.child_tags {
                if let Some(id) = DataComponent::try_from_name(name)
                    && let Some(value) = read_data(id, tag)
                {
                    set_component(&mut retained, value);
                }
            }
        }
        for &(id, key) in fields {
            if id == DataComponent::ContainerLoot {
                if let Some(loot_table) = nbt.get_string(key) {
                    set_component(
                        &mut implicit,
                        Box::new(ContainerLootImpl {
                            loot_table: loot_table.to_owned(),
                            seed: nbt.get_long("LootTableSeed").unwrap_or(0),
                        }),
                    );
                }
            } else if let Some(tag) = nbt.get(key).or_else(|| match id {
                DataComponent::CustomName => nbt.get(if key == "CustomName" {
                    "custom_name"
                } else {
                    "CustomName"
                }),
                DataComponent::Bees => nbt.get("Bees"),
                _ => None,
            }) {
                if let Some(value) = read_data(id, tag) {
                    set_component(&mut implicit, value);
                } else {
                    tracing::warn!(
                        "Invalid block entity component {} in field {key}",
                        id.to_name()
                    );
                }
            }
        }
        Self {
            values: Mutex::new(ComponentValues { retained, implicit }),
            fields,
        }
    }

    /// Copies typed values without serializing NBT or unpacking deferred loot.
    pub fn collect(&self) -> ComponentMap {
        let stored = self
            .values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut values = stored.retained.clone();
        for &(id, _) in self.fields {
            if let Some((_, value)) = stored.implicit.iter().find(|(key, _)| *key == id) {
                set_component(&mut values, value.clone());
            } else if matches!(
                id,
                DataComponent::BannerPatterns | DataComponent::PotDecorations | DataComponent::Bees
            ) {
                if let Some(value) = read_data(id, &NbtTag::List(Vec::new())) {
                    set_component(&mut values, value);
                }
            } else if !matches!(id, DataComponent::Lock | DataComponent::ContainerLoot) {
                values.retain(|(key, _)| *key != id);
            }
        }
        values
    }

    /// Replaces state from effective implicit values and unconsumed item additions.
    pub fn apply(&self, stack: &ItemStack, consumed: &[DataComponent]) {
        let mut retained = Vec::new();
        let mut implicit = Vec::new();
        for (id, value) in &stack.patch {
            if !matches!(
                id,
                DataComponent::BlockEntityData | DataComponent::BlockState
            ) && !consumed.contains(id)
                && !self.fields.iter().any(|(field, _)| field == id)
                && let Some(value) = value
            {
                set_component(&mut retained, value.clone());
            }
        }
        for &(id, _) in self.fields {
            let value = match stack.patch.iter().find(|(key, _)| *key == id) {
                Some((_, value)) => value.clone(),
                None => stack
                    .item
                    .components
                    .iter()
                    .find(|(key, _)| *key == id)
                    .map(|(_, value)| value.clone_dyn()),
            };
            if let Some(value) = value {
                set_component(&mut implicit, value);
            }
        }
        *self
            .values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            ComponentValues { retained, implicit };
    }

    /// Saves mapped implicit fields and unconsumed additions using vanilla's NBT layout.
    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_nbt_with_retained(nbt, true);
    }

    /// Writes all mapped custom fields, including locks, without retained item additions.
    pub fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        self.write_nbt_with_retained(nbt, false);
    }

    /// Writes one consistent snapshot of implicit fields and optionally the retained component map.
    fn write_nbt_with_retained(&self, nbt: &mut NbtCompound, include_retained: bool) {
        let values = self
            .values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for &(id, key) in self.fields {
            if let Some((_, value)) = values.implicit.iter().find(|(field, _)| *field == id) {
                if let Some(loot) = value.as_any().downcast_ref::<ContainerLootImpl>() {
                    nbt.put_string(key, loot.loot_table.clone());
                    if loot.seed != 0 {
                        nbt.put_long("LootTableSeed", loot.seed);
                    }
                } else {
                    nbt.put(key, value.write_data());
                }
            }
        }
        if include_retained && !values.retained.is_empty() {
            let mut retained = NbtCompound::new();
            for (id, value) in &values.retained {
                retained.put(id.to_name(), value.write_data());
            }
            nbt.put_compound("components", retained);
        }
    }

    /// Writes the render fields used by container chunk updates, without deferred loot or locks.
    pub fn write_client_nbt(&self, nbt: &mut NbtCompound) {
        let values = self
            .values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for &(id, key) in self.fields {
            if !matches!(id, DataComponent::Lock | DataComponent::ContainerLoot)
                && let Some((_, value)) =
                    values.implicit.iter().find(|(existing, _)| *existing == id)
            {
                nbt.put(key, value.write_data());
            }
        }
    }

    /// Takes a pending loot key and seed atomically, leaving other components intact.
    pub fn take_loot_table(&self) -> Option<(String, i64)> {
        let mut values = self
            .values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let index = values
            .implicit
            .iter()
            .position(|(id, _)| *id == DataComponent::ContainerLoot)?;
        let (_, value) = values.implicit.remove(index);
        let loot = value.as_any().downcast_ref::<ContainerLootImpl>()?;
        Some((loot.loot_table.clone(), loot.seed))
    }

    /// Reports whether deferred loot remains available without generating or consuming it.
    pub fn has_loot_table(&self) -> bool {
        self.values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .implicit
            .iter()
            .any(|(id, _)| *id == DataComponent::ContainerLoot)
    }

    /// Reads one typed value under the component lock; the callback must not re-enter this state.
    pub fn with_component<T: DataComponentImpl + 'static, R>(
        &self,
        read: impl FnOnce(&T) -> R,
    ) -> Option<R> {
        let values = self
            .values
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let id = T::get_enum();
        let value = values
            .implicit
            .iter()
            .find(|(key, _)| *key == id)
            .or_else(|| {
                if self.fields.iter().any(|(field, _)| *field == id)
                    && !matches!(id, DataComponent::Lock | DataComponent::ContainerLoot)
                {
                    None
                } else {
                    values.retained.iter().find(|(key, _)| *key == id)
                }
            })?;
        let (_, value) = value;
        value.as_any().downcast_ref::<T>().map(read)
    }
}

/// Inserts one typed component, replacing an earlier value of the same registry type.
pub fn set_component(components: &mut ComponentMap, value: Box<dyn DataComponentImpl>) {
    let id = value.get_self_enum();
    if let Some((_, existing)) = components.iter_mut().find(|(key, _)| *key == id) {
        *existing = value;
    } else {
        components.push((id, value));
    }
}

/// Supplies the common owned state and current inventory for container block entities.
macro_rules! impl_container_components {
    () => {
        /// Returns this container's persistent scalar fields and retained component additions.
        fn component_state(
            &self,
        ) -> Option<&$crate::block::entities::components::BlockEntityComponents> {
            Some(&self.components)
        }

        /// Marks inventory contents as implicit so placement does not retain a stale copy.
        fn consumed_components(&self) -> &'static [pumpkin_data::data_component::DataComponent] {
            &[pumpkin_data::data_component::DataComponent::Container]
        }

        /// Snapshots current slots without opening the container or generating deferred loot.
        fn collect_implicit_components(
            &self,
            components: &mut $crate::block::entities::components::ComponentMap,
        ) {
            let items = self
                .items
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let items = items
                .iter()
                .enumerate()
                .filter(|(_, item)| !item.is_empty())
                .map(|(slot, item)| (slot as u8, item.clone()))
                .collect();
            $crate::block::entities::components::set_component(
                components,
                Box::new(pumpkin_data::data_component_impl::ContainerImpl { items }),
            );
        }

        /// Restores valid inventory slots and clears absent contents before marking the container dirty.
        fn apply_implicit_components(&self, stack: &pumpkin_data::item_stack::ItemStack) {
            let mut items = self
                .items
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            items.fill_with(|| pumpkin_data::item_stack::ItemStack::EMPTY.clone());
            if let Some(container) =
                stack.get_data_component::<pumpkin_data::data_component_impl::ContainerImpl>()
            {
                for (slot, item) in &container.items {
                    if let Some(target) = items.get_mut(usize::from(*slot)) {
                        *target = item.clone();
                    }
                }
            }
            drop(items);
            pumpkin_inventory::Inventory::mark_dirty(self);
        }
    };
}

pub(crate) use impl_container_components;

#[cfg(test)]
mod tests;

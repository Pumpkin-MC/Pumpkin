//! Runtime items registered by plugins on top of the generated vanilla items.
//!
//! The generated `Item` table is compile time only. Items added here get the ids
//! `Item::vanilla_count() + index`, in registration order. The generated lookups
//! (`Item::from_id`, `Item::from_registry_key`) fall back to this registry after their static
//! match misses, so every caller that maps an id or key to an `Item` also sees dynamic items.
//!
//! Registration is append only and closes with [`Item::freeze_dynamic_registry`], which the server
//! calls before it accepts connections. Entries are leaked so they can be `&'static Item`.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::{LazyLock, PoisonError, RwLock};

use crate::data_component::DataComponent;
use crate::data_component_impl::{
    DamageImpl, DataComponentImpl, EnchantmentsImpl, ItemModelImpl, ItemNameImpl, LoreImpl,
    MaxDamageImpl, MaxStackSizeImpl, Rarity, RarityImpl, RepairCostImpl,
};
use crate::item::Item;

/// Namespace of the generated items. It is implicit in their `registry_key`.
const VANILLA_NAMESPACE: &str = "minecraft";
const DEFAULT_MAX_STACK_SIZE: u8 = 64;
/// Item ids are `u16` on the network and in `Item`.
const MAX_ITEM_ID: usize = u16::MAX as usize;

/// Description of an item to add to the registry.
pub struct ItemRegistration {
    /// Full resource location, `namespace:path`. The `minecraft` namespace is reserved.
    pub key: String,
    /// Default components of the item. Components that are not listed get vanilla style defaults.
    pub components: Vec<(DataComponent, Box<dyn DataComponentImpl>)>,
    /// Shortcut for the `MaxStackSize` component. Wins over a listed `MaxStackSize` component.
    pub max_stack_size: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemRegistrationError {
    /// The key is not a valid `namespace:path` resource location.
    InvalidKey(String),
    /// The `minecraft` namespace belongs to the generated items.
    ReservedNamespace(String),
    /// An item with this key exists already with another definition.
    Duplicate(String),
    /// A component was listed twice.
    DuplicateComponent(&'static str),
    /// The registry is closed, see [`Item::freeze_dynamic_registry`].
    Frozen,
    /// There are no ids left.
    Full,
}

impl fmt::Display for ItemRegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey(key) => {
                write!(
                    f,
                    "'{key}' is not a valid item key, expected namespace:path"
                )
            }
            Self::ReservedNamespace(key) => {
                write!(
                    f,
                    "'{key}' uses the reserved '{VANILLA_NAMESPACE}' namespace"
                )
            }
            Self::Duplicate(key) => write!(
                f,
                "item '{key}' is already registered with another definition"
            ),
            Self::DuplicateComponent(name) => {
                write!(f, "component '{name}' is listed more than once")
            }
            Self::Frozen => write!(f, "items can only be registered before players connect"),
            Self::Full => write!(f, "the item registry is full"),
        }
    }
}

impl std::error::Error for ItemRegistrationError {}

fn is_valid_namespace(namespace: &str) -> bool {
    !namespace.is_empty()
        && namespace
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.'))
}

fn is_valid_path(path: &str) -> bool {
    !path.is_empty()
        && path
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'/'))
}

fn split_key(key: &str) -> Result<(&str, &str), ItemRegistrationError> {
    let (namespace, path) = key
        .split_once(':')
        .ok_or_else(|| ItemRegistrationError::InvalidKey(key.to_string()))?;
    if !is_valid_namespace(namespace) || !is_valid_path(path) {
        return Err(ItemRegistrationError::InvalidKey(key.to_string()));
    }
    if namespace == VANILLA_NAMESPACE {
        return Err(ItemRegistrationError::ReservedNamespace(key.to_string()));
    }
    Ok((namespace, path))
}

/// Vanilla ids are `0..count`.
static VANILLA_COUNT: LazyLock<u16> = LazyLock::new(|| {
    let mut count = 0u16;
    while Item::from_vanilla_id(count).is_some() {
        count += 1;
    }
    count
});

/// Append only item table. The global instance backs `Item::from_id` and friends.
pub struct ItemRegistry {
    frozen: bool,
    items: Vec<&'static Item>,
    by_key: HashMap<&'static str, &'static Item>,
}

impl ItemRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            frozen: false,
            items: Vec::new(),
            by_key: HashMap::new(),
        }
    }

    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    #[must_use]
    pub const fn is_frozen(&self) -> bool {
        self.frozen
    }

    /// Adds an item. Registering a key again with an identical definition returns the existing
    /// item, even when the registry is closed, so a plugin can be reloaded. A different definition
    /// for a known key is [`ItemRegistrationError::Duplicate`].
    pub fn register(
        &mut self,
        registration: ItemRegistration,
    ) -> Result<&'static Item, ItemRegistrationError> {
        let ItemRegistration {
            key,
            components,
            max_stack_size,
        } = registration;
        let (namespace, path) = split_key(&key)?;
        let components =
            with_default_components(&key, namespace, path, components, max_stack_size)?;

        if let Some(existing) = self.by_key.get(key.as_str()) {
            return if same_components(existing.components, &components) {
                Ok(existing)
            } else {
                Err(ItemRegistrationError::Duplicate(key))
            };
        }
        if self.frozen {
            return Err(ItemRegistrationError::Frozen);
        }
        let id = usize::from(*VANILLA_COUNT) + self.items.len();
        if id > MAX_ITEM_ID {
            return Err(ItemRegistrationError::Full);
        }

        let leaked_components: Vec<(DataComponent, &'static dyn DataComponentImpl)> = components
            .into_iter()
            .map(|(id, component)| (id, &*Box::leak(component)))
            .collect();
        let registry_key: &'static str = Box::leak(key.into_boxed_str());
        let item: &'static Item = Box::leak(Box::new(Item {
            id: id as u16,
            registry_key,
            components: Box::leak(leaked_components.into_boxed_slice()),
        }));
        self.items.push(item);
        self.by_key.insert(registry_key, item);
        Ok(item)
    }

    #[must_use]
    pub fn get_by_id(&self, id: u16) -> Option<&'static Item> {
        let index = usize::from(id).checked_sub(usize::from(*VANILLA_COUNT))?;
        self.items.get(index).copied()
    }

    #[must_use]
    pub fn get_by_key(&self, key: &str) -> Option<&'static Item> {
        self.by_key.get(key).copied()
    }

    /// Registered items in registration (and so id) order.
    #[must_use]
    pub fn items(&self) -> &[&'static Item] {
        &self.items
    }
}

impl Default for ItemRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn same_components(
    existing: &[(DataComponent, &'static dyn DataComponentImpl)],
    new: &[(DataComponent, Box<dyn DataComponentImpl>)],
) -> bool {
    existing.len() == new.len()
        && new.iter().all(|(id, component)| {
            existing
                .iter()
                .find(|(existing_id, _)| existing_id == id)
                .is_some_and(|(_, existing)| existing.equal(component.as_ref()))
        })
}

/// Adds what vanilla gives every item and what the item behaviour code reads without checking
/// for presence (most of all `MaxStackSize`, which defaults to 1 when absent on a stack).
fn with_default_components(
    key: &str,
    namespace: &str,
    path: &str,
    mut components: Vec<(DataComponent, Box<dyn DataComponentImpl>)>,
    max_stack_size: Option<u8>,
) -> Result<Vec<(DataComponent, Box<dyn DataComponentImpl>)>, ItemRegistrationError> {
    for (index, (id, _)) in components.iter().enumerate() {
        if components[..index].iter().any(|(other, _)| other == id) {
            return Err(ItemRegistrationError::DuplicateComponent(id.to_name()));
        }
    }

    if let Some(size) = max_stack_size {
        components.retain(|(id, _)| *id != DataComponent::MaxStackSize);
        components.push((
            DataComponent::MaxStackSize,
            MaxStackSizeImpl { size }.to_dyn(),
        ));
    }

    let has = |components: &[(DataComponent, Box<dyn DataComponentImpl>)], id: DataComponent| {
        components.iter().any(|(existing, _)| *existing == id)
    };

    if !has(&components, DataComponent::MaxStackSize) {
        // A damageable item cannot stack.
        let size = if has(&components, DataComponent::MaxDamage) {
            1
        } else {
            DEFAULT_MAX_STACK_SIZE
        };
        components.push((
            DataComponent::MaxStackSize,
            MaxStackSizeImpl { size }.to_dyn(),
        ));
    }
    if !has(&components, DataComponent::ItemName) {
        components.push((
            DataComponent::ItemName,
            ItemNameImpl {
                name: Cow::Owned(format!("item.{namespace}.{}", path.replace('/', "."))),
            }
            .to_dyn(),
        ));
    }
    if has(&components, DataComponent::MaxDamage) && !has(&components, DataComponent::Damage) {
        components.push((DataComponent::Damage, DamageImpl { damage: 0 }.to_dyn()));
    }
    if !has(&components, DataComponent::Enchantments) {
        components.push((
            DataComponent::Enchantments,
            EnchantmentsImpl {
                enchantment: Cow::Borrowed(&[]),
            }
            .to_dyn(),
        ));
    }
    if !has(&components, DataComponent::ItemModel) {
        components.push((
            DataComponent::ItemModel,
            ItemModelImpl {
                id: Cow::Owned(key.to_string()),
            }
            .to_dyn(),
        ));
    }
    if !has(&components, DataComponent::Lore) {
        components.push((DataComponent::Lore, LoreImpl { lines: Vec::new() }.to_dyn()));
    }
    if !has(&components, DataComponent::Rarity) {
        components.push((
            DataComponent::Rarity,
            RarityImpl {
                rarity: Rarity::Common,
            }
            .to_dyn(),
        ));
    }
    if !has(&components, DataComponent::RepairCost) {
        components.push((DataComponent::RepairCost, RepairCostImpl::DEFAULT.to_dyn()));
    }
    Ok(components)
}

static REGISTRY: LazyLock<RwLock<ItemRegistry>> =
    LazyLock::new(|| RwLock::new(ItemRegistry::new()));

fn read_registry() -> std::sync::RwLockReadGuard<'static, ItemRegistry> {
    REGISTRY.read().unwrap_or_else(PoisonError::into_inner)
}

fn write_registry() -> std::sync::RwLockWriteGuard<'static, ItemRegistry> {
    REGISTRY.write().unwrap_or_else(PoisonError::into_inner)
}

impl Item {
    /// Number of generated items. Dynamic items get the ids `vanilla_count() + index`.
    #[must_use]
    pub fn vanilla_count() -> u16 {
        *VANILLA_COUNT
    }

    /// Registers an item in the global registry and returns it. See [`ItemRegistry::register`].
    pub fn register_dynamic(
        registration: ItemRegistration,
    ) -> Result<&'static Self, ItemRegistrationError> {
        write_registry().register(registration)
    }

    /// Closes the global registry. Later registrations fail with [`ItemRegistrationError::Frozen`].
    pub fn freeze_dynamic_registry() {
        write_registry().freeze();
    }

    #[must_use]
    pub fn is_dynamic_registry_frozen() -> bool {
        read_registry().is_frozen()
    }

    /// Items registered at runtime, in id order.
    #[must_use]
    pub fn dynamic_items() -> Vec<&'static Self> {
        read_registry().items().to_vec()
    }

    /// Whether this item was registered at runtime instead of being generated.
    #[must_use]
    pub fn is_dynamic(&self) -> bool {
        self.id >= *VANILLA_COUNT
    }

    /// The full resource location, `minecraft:` prefixed for vanilla items.
    #[must_use]
    pub fn resource_location(&self) -> Cow<'static, str> {
        if self.is_dynamic() {
            Cow::Borrowed(self.registry_key)
        } else {
            Cow::Owned(format!("{VANILLA_NAMESPACE}:{}", self.registry_key))
        }
    }

    /// The item tags for the client tag sync (generated plus runtime). `None` when there are no
    /// runtime item tags, so the generated tables can be sent as they are.
    #[must_use]
    pub fn network_tags() -> Option<Vec<(String, Vec<u16>)>> {
        crate::dynamic_tag::merged_tags(crate::tag::RegistryKey::Item, |key| {
            Self::from_registry_key(key).map(|item| item.id)
        })
    }

    /// Fallback of the generated `from_id` for ids past the vanilla range.
    #[must_use]
    pub fn from_dynamic_id(id: u16) -> Option<&'static Self> {
        read_registry().get_by_id(id)
    }

    /// Fallback of the generated `from_registry_key` for keys that are not vanilla. Only
    /// namespaced keys can match, vanilla keys never carry a namespace here.
    #[must_use]
    pub fn from_dynamic_registry_key(key: &str) -> Option<&'static Self> {
        if !key.contains(':') {
            return None;
        }
        read_registry().get_by_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_stack::ItemStack;

    fn registration(key: &str) -> ItemRegistration {
        ItemRegistration {
            key: key.to_string(),
            components: Vec::new(),
            max_stack_size: None,
        }
    }

    #[test]
    fn vanilla_ids_are_contiguous() {
        let count = Item::vanilla_count();
        assert!(count > 1000);
        assert!(Item::from_vanilla_id(count - 1).is_some());
        for id in count..=u16::MAX {
            assert!(Item::from_vanilla_id(id).is_none(), "gap at {id}");
        }
    }

    #[test]
    fn ids_follow_vanilla_count_in_registration_order() {
        let mut registry = ItemRegistry::new();
        let first = registry.register(registration("test_order:first")).unwrap();
        let second = registry
            .register(registration("test_order:second"))
            .unwrap();
        assert_eq!(first.id, Item::vanilla_count());
        assert_eq!(second.id, Item::vanilla_count() + 1);
        assert_eq!(registry.get_by_id(first.id), Some(first));
        assert_eq!(registry.get_by_key("test_order:second"), Some(second));
        assert!(first.is_dynamic());
        assert!(!Item::DIAMOND.is_dynamic());
        assert_eq!(first.resource_location(), "test_order:first");
        assert_eq!(Item::DIAMOND.resource_location(), "minecraft:diamond");
        assert!(registry.get_by_id(Item::vanilla_count() + 2).is_none());
    }

    #[test]
    fn rejects_bad_keys_duplicates_and_frozen() {
        let mut registry = ItemRegistry::new();
        for bad in ["nocolon", "Upper:case", "ns:", ":path", "ns:sp ace"] {
            assert_eq!(
                registry.register(registration(bad)).err(),
                Some(ItemRegistrationError::InvalidKey(bad.to_string())),
            );
        }
        assert_eq!(
            registry.register(registration("minecraft:thing")).err(),
            Some(ItemRegistrationError::ReservedNamespace(
                "minecraft:thing".to_string()
            )),
        );
        let first = registry.register(registration("test_dup:a")).unwrap();
        // The same definition is the same item, a different one is a conflict.
        assert_eq!(
            registry.register(registration("test_dup:a")).unwrap(),
            first
        );
        let mut different = registration("test_dup:a");
        different.max_stack_size = Some(8);
        assert_eq!(
            registry.register(different).err(),
            Some(ItemRegistrationError::Duplicate("test_dup:a".to_string())),
        );
        let mut twice = registration("test_dup:b");
        twice.components = vec![
            (
                DataComponent::Rarity,
                RarityImpl {
                    rarity: Rarity::Rare,
                }
                .to_dyn(),
            ),
            (
                DataComponent::Rarity,
                RarityImpl {
                    rarity: Rarity::Epic,
                }
                .to_dyn(),
            ),
        ];
        assert_eq!(
            registry.register(twice).err(),
            Some(ItemRegistrationError::DuplicateComponent(
                "minecraft:rarity"
            )),
        );
        registry.freeze();
        assert_eq!(
            registry.register(registration("test_dup:c")).err(),
            Some(ItemRegistrationError::Frozen),
        );
        // Re-registering a known item still works for plugin reloads.
        assert_eq!(
            registry.register(registration("test_dup:a")).unwrap(),
            first
        );
        assert_eq!(registry.items().len(), 1);
    }

    #[test]
    fn defaults_and_explicit_components() {
        let mut registry = ItemRegistry::new();
        let plain = registry.register(registration("test_comp:plain")).unwrap();
        let stack = ItemStack::new(1, plain);
        assert_eq!(stack.get_max_stack_size(), 64);
        assert_eq!(
            stack.get_data_component::<ItemNameImpl>().unwrap().name,
            "item.test_comp.plain"
        );

        let mut sword = registration("test_comp:sword");
        sword.components = vec![
            (
                DataComponent::MaxDamage,
                MaxDamageImpl { max_damage: 2000 }.to_dyn(),
            ),
            (
                DataComponent::Rarity,
                RarityImpl {
                    rarity: Rarity::Rare,
                }
                .to_dyn(),
            ),
        ];
        let sword = registry.register(sword).unwrap();
        let stack = ItemStack::new(1, sword);
        assert_eq!(stack.get_max_stack_size(), 1);
        assert_eq!(stack.get_max_damage(), Some(2000));
        assert_eq!(
            stack.get_data_component::<RarityImpl>().unwrap().rarity,
            Rarity::Rare
        );

        let mut sized = registration("test_comp:sized");
        sized.max_stack_size = Some(16);
        sized.components = vec![(
            DataComponent::MaxStackSize,
            MaxStackSizeImpl { size: 2 }.to_dyn(),
        )];
        let sized = registry.register(sized).unwrap();
        assert_eq!(ItemStack::new(1, sized).get_max_stack_size(), 16);
        assert_eq!(
            sized
                .components
                .iter()
                .filter(|(id, _)| *id == DataComponent::MaxStackSize)
                .count(),
            1
        );
    }

    #[test]
    fn global_lookups_and_nbt_round_trip() {
        let item = Item::register_dynamic(registration("test_global:gem")).unwrap();
        assert_eq!(Item::from_id(item.id), Some(item));
        assert_eq!(Item::from_registry_key("test_global:gem"), Some(item));
        assert_eq!(Item::from_registry_key("gem"), None);
        assert_eq!(
            Item::from_registry_key("minecraft:diamond"),
            Some(&Item::DIAMOND)
        );
        assert!(Item::dynamic_items().contains(&item));
        assert_eq!(
            Item::register_dynamic(registration("test_global:gem")).unwrap(),
            item
        );
        assert_eq!(
            Item::register_dynamic(ItemRegistration {
                max_stack_size: Some(3),
                ..registration("test_global:gem")
            })
            .err(),
            Some(ItemRegistrationError::Duplicate(
                "test_global:gem".to_string()
            )),
        );

        let mut stack = ItemStack::new(7, item);
        stack.set_data_component(RarityImpl {
            rarity: Rarity::Epic,
        });
        let mut compound = pumpkin_nbt::compound::NbtCompound::new();
        stack.write_item_stack(&mut compound);
        assert_eq!(compound.get_string("id"), Some("test_global:gem"));
        let decoded = ItemStack::read_item_stack(&compound).expect("dynamic stack should decode");
        assert_eq!(decoded.item, item);
        assert_eq!(decoded.item_count, 7);
        assert_eq!(
            decoded.get_data_component::<RarityImpl>().unwrap().rarity,
            Rarity::Epic
        );

        // A plugin that is gone leaves an unknown item behind. It must not decode.
        let mut missing = pumpkin_nbt::compound::NbtCompound::new();
        missing.put_string("id", "test_global:removed".to_string());
        missing.put_int("count", 1);
        assert!(ItemStack::read_item_stack(&missing).is_none());
    }
}

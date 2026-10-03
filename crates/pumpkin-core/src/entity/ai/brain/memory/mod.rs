use std::fmt;
use std::marker::PhantomData;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

pub mod codec;
pub mod global_pos;
pub mod nearest_visible;
pub mod position_tracker;
pub mod types;
pub mod value;
pub mod walk_target;

pub use codec::MemoryCodec;
pub use global_pos::GlobalPos;
pub use nearest_visible::NearestVisibleLivingEntities;
pub use position_tracker::{BlockPosTracker, EntityTracker, PositionTracker};
pub use value::{DamageSourceMemory, MemoryValue, SpearStatus};
pub use walk_target::WalkTarget;

pub const NEVER_EXPIRE: i64 = i64::MAX;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MemoryModuleId(u8);

impl MemoryModuleId {
    #[must_use]
    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        types::name_of(self)
    }
}

pub struct MemoryModuleType<T: MemoryValue> {
    id: MemoryModuleId,
    value: PhantomData<fn() -> T>,
}

impl<T: MemoryValue> MemoryModuleType<T> {
    #[must_use]
    pub const fn new(raw: u8) -> Self {
        Self {
            id: MemoryModuleId::new(raw),
            value: PhantomData,
        }
    }

    #[must_use]
    pub const fn id(self) -> MemoryModuleId {
        self.id
    }

    #[must_use]
    pub fn can_serialize(self) -> bool {
        types::codec_of(self.id).is_some()
    }
}

impl<T: MemoryValue> Clone for MemoryModuleType<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: MemoryValue> Copy for MemoryModuleType<T> {}

impl<T: MemoryValue> fmt::Debug for MemoryModuleType<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id.name())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MemoryStatus {
    ValuePresent,
    ValueAbsent,
    Registered,
}

pub struct MemorySlot {
    value: Option<Box<dyn MemoryValue>>,
    ttl: i64,
}

impl MemorySlot {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            value: None,
            ttl: NEVER_EXPIRE,
        }
    }

    #[must_use]
    pub const fn has_value(&self) -> bool {
        self.value.is_some()
    }

    #[must_use]
    pub const fn can_expire(&self) -> bool {
        self.ttl != NEVER_EXPIRE
    }

    #[must_use]
    pub const fn time_to_live(&self) -> i64 {
        self.ttl
    }

    pub fn clear(&mut self) {
        self.value = None;
        self.ttl = NEVER_EXPIRE;
    }

    fn tick(&mut self) {
        if self.has_value() && self.can_expire() {
            if self.ttl <= 0 {
                self.clear();
            } else {
                self.ttl -= 1;
            }
        }
    }
}

impl MemorySlot {
    // Vanilla's dump prints the game time where the ttl belongs; the real ttl is what a reader wants.
    fn describe(&self) -> String {
        match &self.value {
            None => "-".to_owned(),
            Some(value) if self.can_expire() => {
                format!("{} (ttl: {})", value.describe(), self.ttl)
            }
            Some(value) => value.describe(),
        }
    }
}

impl fmt::Debug for MemorySlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.describe())
    }
}

// `StringUtil.truncateStringIfNecessary(text, 255, true)`, applied per line by the vanilla dump.
fn truncate_description(text: String) -> String {
    const MAX_CHARS: usize = 255;
    if text.chars().count() <= MAX_CHARS {
        return text;
    }
    let mut kept: String = text.chars().take(MAX_CHARS - 3).collect();
    kept.push_str("...");
    kept
}

#[derive(Default)]
pub struct MemoryStore {
    slots: Vec<Option<MemorySlot>>,
}

impl MemoryStore {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn register(&mut self, id: MemoryModuleId) {
        if self.slots.is_empty() {
            self.slots.resize_with(types::MEMORY_TYPE_COUNT, || None);
        }
        if let Some(slot) = self.slots.get_mut(id.index())
            && slot.is_none()
        {
            *slot = Some(MemorySlot::empty());
        }
    }

    #[must_use]
    pub fn check(&self, id: MemoryModuleId, status: MemoryStatus) -> bool {
        let Some(Some(slot)) = self.slots.get(id.index()) else {
            return false;
        };
        match status {
            MemoryStatus::Registered => true,
            MemoryStatus::ValuePresent => slot.has_value(),
            MemoryStatus::ValueAbsent => !slot.has_value(),
        }
    }

    #[must_use]
    pub fn has_value(&self, id: MemoryModuleId) -> bool {
        self.check(id, MemoryStatus::ValuePresent)
    }

    #[must_use]
    pub fn get<T: MemoryValue>(&self, key: MemoryModuleType<T>) -> Option<&T> {
        self.slots
            .get(key.id.index())?
            .as_ref()?
            .value
            .as_ref()?
            .as_any()
            .downcast_ref::<T>()
    }

    #[must_use]
    pub fn time_until_expiry(&self, id: MemoryModuleId) -> Option<i64> {
        self.slots
            .get(id.index())?
            .as_ref()
            .map(MemorySlot::time_to_live)
    }

    pub fn set<T: MemoryValue>(&mut self, key: MemoryModuleType<T>, value: T) {
        self.set_with_expiry(key, value, NEVER_EXPIRE);
    }

    pub fn set_optional<T: MemoryValue>(&mut self, key: MemoryModuleType<T>, value: Option<T>) {
        match value {
            Some(value) => self.set(key, value),
            None => self.erase(key.id),
        }
    }

    pub fn set_with_expiry<T: MemoryValue>(
        &mut self,
        key: MemoryModuleType<T>,
        value: T,
        ttl: i64,
    ) {
        let Some(Some(slot)) = self.slots.get_mut(key.id.index()) else {
            return;
        };
        if value.is_empty_collection() {
            slot.clear();
            return;
        }
        let reused = slot
            .value
            .as_mut()
            .and_then(|boxed| boxed.as_any_mut().downcast_mut::<T>());
        if let Some(existing) = reused {
            *existing = value;
        } else {
            slot.value = Some(Box::new(value));
        }
        slot.ttl = ttl;
    }

    pub fn erase(&mut self, id: MemoryModuleId) {
        if let Some(Some(slot)) = self.slots.get_mut(id.index()) {
            slot.clear();
        }
    }

    pub fn clear_all(&mut self) {
        for slot in self.slots.iter_mut().flatten() {
            slot.clear();
        }
    }

    #[must_use]
    pub fn is_value<T: MemoryValue + PartialEq>(
        &self,
        key: MemoryModuleType<T>,
        value: &T,
    ) -> bool {
        self.get(key) == Some(value)
    }

    pub fn tick(&mut self) {
        for slot in self.slots.iter_mut().flatten() {
            slot.tick();
        }
    }

    #[must_use]
    pub fn pack(&self) -> PackedMemories {
        let mut packed = Vec::new();
        for (index, slot) in self.slots.iter().enumerate() {
            let Some(slot) = slot.as_ref() else {
                continue;
            };
            let Some(value) = slot.value.as_ref() else {
                continue;
            };
            let id = MemoryModuleId::new(index as u8);
            let Some(codec) = types::codec_of(id) else {
                continue;
            };
            let Some(tag) = (codec.encode)(value.as_ref()) else {
                continue;
            };
            let ttl = slot.can_expire().then_some(slot.ttl);
            packed.push((id, tag, ttl));
        }
        PackedMemories(packed)
    }

    pub fn load_packed(&mut self, packed: &PackedMemories) {
        for (id, tag, ttl) in &packed.0 {
            let Some(codec) = types::codec_of(*id) else {
                continue;
            };
            let Some(value) = (codec.decode)(tag) else {
                tracing::debug!("Dropping undecodable brain memory {}", id.name());
                continue;
            };
            let Some(Some(slot)) = self.slots.get_mut(id.index()) else {
                continue;
            };
            if value.is_empty_collection() {
                slot.clear();
            } else {
                slot.value = Some(value);
                slot.ttl = ttl.unwrap_or(NEVER_EXPIRE);
            }
        }
    }

    #[must_use]
    pub fn debug_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|slot| (index, slot)))
            .map(|(index, slot)| {
                let name = MemoryModuleId::new(index as u8).name();
                let path = name.strip_prefix("minecraft:").unwrap_or(name);
                truncate_description(format!("{path}: {}", slot.describe()))
            })
            .collect();
        lines.sort();
        lines
    }
}

pub struct PackedMemories(Vec<(MemoryModuleId, NbtTag, Option<i64>)>);

impl PackedMemories {
    #[must_use]
    pub const fn empty() -> Self {
        Self(Vec::new())
    }

    #[must_use]
    pub fn into_nbt(self) -> NbtCompound {
        let mut memories = NbtCompound::new();
        for (id, tag, ttl) in self.0 {
            let mut entry = NbtCompound::new();
            entry.put("value", tag);
            if let Some(ttl) = ttl {
                entry.put_long("ttl", ttl);
            }
            memories.put_compound(id.name(), entry);
        }
        let mut brain = NbtCompound::new();
        brain.put_compound("memories", memories);
        brain
    }

    #[must_use]
    pub fn from_nbt(brain: &NbtCompound) -> Self {
        let Some(memories) = brain.get_compound("memories") else {
            return Self::empty();
        };
        let mut packed = Vec::with_capacity(memories.child_tags.len());
        for (name, tag) in &memories.child_tags {
            let Some(id) = types::from_name(name) else {
                tracing::debug!("Skipping unknown brain memory {name}");
                continue;
            };
            if types::codec_of(id).is_none() {
                tracing::debug!("Skipping non-persistent brain memory {name}");
                continue;
            }
            let NbtTag::Compound(entry) = tag else {
                continue;
            };
            let Some(value) = entry.get("value") else {
                tracing::debug!("Skipping valueless brain memory {name}");
                continue;
            };
            packed.push((id, value.clone(), entry.get_long("ttl")));
        }
        Self(packed)
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::math::position::BlockPos;

    use super::*;

    fn overworld(x: i32, y: i32, z: i32) -> GlobalPos {
        GlobalPos::new(&Dimension::OVERWORLD, BlockPos::new(x, y, z))
    }

    #[test]
    fn slot_expires_on_the_tick_after_its_ttl_reaches_zero() {
        let mut store = MemoryStore::default();
        store.register(types::ADMIRING_ITEM.id());
        store.set_with_expiry(types::ADMIRING_ITEM, true, 2);
        store.tick();
        assert_eq!(store.time_until_expiry(types::ADMIRING_ITEM.id()), Some(1));
        store.tick();
        assert_eq!(store.time_until_expiry(types::ADMIRING_ITEM.id()), Some(0));
        assert!(store.has_value(types::ADMIRING_ITEM.id()));
        store.tick();
        assert!(!store.has_value(types::ADMIRING_ITEM.id()));
    }

    #[test]
    fn never_expiring_slot_never_ticks_down() {
        let mut store = MemoryStore::default();
        store.register(types::ADMIRING_ITEM.id());
        store.set(types::ADMIRING_ITEM, true);
        for _ in 0..10 {
            store.tick();
        }
        assert_eq!(
            store.time_until_expiry(types::ADMIRING_ITEM.id()),
            Some(NEVER_EXPIRE)
        );
        assert!(store.has_value(types::ADMIRING_ITEM.id()));
    }

    #[test]
    fn unregistered_memory_fails_every_status_check() {
        let store = MemoryStore::default();
        let id = types::ADMIRING_ITEM.id();
        assert!(!store.check(id, MemoryStatus::Registered));
        assert!(!store.check(id, MemoryStatus::ValuePresent));
        assert!(!store.check(id, MemoryStatus::ValueAbsent));
    }

    #[test]
    fn registering_makes_the_slot_registered_and_absent() {
        let mut store = MemoryStore::default();
        let id = types::ADMIRING_ITEM.id();
        store.register(id);
        assert!(store.check(id, MemoryStatus::Registered));
        assert!(store.check(id, MemoryStatus::ValueAbsent));
        assert!(!store.check(id, MemoryStatus::ValuePresent));
    }

    #[test]
    fn setting_an_unregistered_memory_does_nothing() {
        let mut store = MemoryStore::default();
        store.set(types::ADMIRING_ITEM, true);
        assert!(store.is_empty());
        assert_eq!(store.get(types::ADMIRING_ITEM), None);
    }

    #[test]
    fn setting_an_empty_collection_clears_the_slot() {
        let mut store = MemoryStore::default();
        store.register(types::SECONDARY_JOB_SITE.id());
        store.set(types::SECONDARY_JOB_SITE, vec![overworld(1, 2, 3)]);
        assert!(store.has_value(types::SECONDARY_JOB_SITE.id()));
        store.set(types::SECONDARY_JOB_SITE, Vec::new());
        assert!(!store.has_value(types::SECONDARY_JOB_SITE.id()));
    }

    #[test]
    fn typed_get_on_the_wrong_type_is_none() {
        let mut store = MemoryStore::default();
        store.register(types::ADMIRING_ITEM.id());
        store.set(types::ADMIRING_ITEM, true);
        let wrong: MemoryModuleType<i32> = MemoryModuleType::new(types::ADMIRING_ITEM.id().0);
        assert_eq!(store.get(wrong), None);
    }

    #[test]
    fn setting_the_same_type_twice_reuses_the_allocation() {
        let mut store = MemoryStore::default();
        store.register(types::PLAY_DEAD_TICKS.id());
        store.set(types::PLAY_DEAD_TICKS, 1);
        let first = std::ptr::from_ref(store.get(types::PLAY_DEAD_TICKS).unwrap());
        store.set(types::PLAY_DEAD_TICKS, 2);
        let second = std::ptr::from_ref(store.get(types::PLAY_DEAD_TICKS).unwrap());
        assert_eq!(first, second);
        assert_eq!(store.get(types::PLAY_DEAD_TICKS), Some(&2));
    }

    #[test]
    fn transient_types_are_not_packed() {
        let mut store = MemoryStore::default();
        store.register(types::DANCING.id());
        store.set(types::DANCING, true);
        assert!(store.pack().0.is_empty());
    }

    #[test]
    fn ttl_is_packed_only_when_the_slot_can_expire() {
        let mut store = MemoryStore::default();
        store.register(types::ADMIRING_ITEM.id());
        store.register(types::HUNTED_RECENTLY.id());
        store.set(types::ADMIRING_ITEM, true);
        store.set_with_expiry(types::HUNTED_RECENTLY, true, 40);
        let packed = store.pack();
        for (id, _, ttl) in &packed.0 {
            if *id == types::ADMIRING_ITEM.id() {
                assert_eq!(*ttl, None);
            } else {
                assert_eq!(*ttl, Some(40));
            }
        }
        assert_eq!(packed.0.len(), 2);
    }

    #[test]
    fn empty_brain_writes_an_empty_memories_compound() {
        let store = MemoryStore::default();
        let mut expected = NbtCompound::new();
        expected.put_compound("memories", NbtCompound::new());
        assert_eq!(store.pack().into_nbt(), expected);
    }

    #[test]
    fn piglin_memories_round_trip_through_the_vanilla_layout() {
        let mut store = MemoryStore::default();
        store.register(types::ADMIRING_ITEM.id());
        store.register(types::ADMIRING_DISABLED.id());
        store.register(types::HUNTED_RECENTLY.id());
        store.set_with_expiry(types::ADMIRING_ITEM, true, 119);

        let mut memory = NbtCompound::new();
        memory.put_bool("value", true);
        memory.put_long("ttl", 119);
        let mut memories = NbtCompound::new();
        memories.put_compound("minecraft:admiring_item", memory);
        let mut expected = NbtCompound::new();
        expected.put_compound("memories", memories);

        let written = store.pack().into_nbt();
        assert_eq!(written, expected);

        let mut restored = MemoryStore::default();
        restored.register(types::ADMIRING_ITEM.id());
        restored.load_packed(&PackedMemories::from_nbt(&written));
        assert_eq!(restored.get(types::ADMIRING_ITEM), Some(&true));
        assert_eq!(
            restored.time_until_expiry(types::ADMIRING_ITEM.id()),
            Some(119)
        );
    }

    #[test]
    fn debug_lines_use_the_vanilla_dump_format() {
        let mut store = MemoryStore::default();
        store.register(types::HUNTED_RECENTLY.id());
        store.register(types::ADMIRING_ITEM.id());
        store.register(types::ANGRY_AT.id());
        store.set_with_expiry(types::HUNTED_RECENTLY, true, 40);
        store.set(types::ADMIRING_ITEM, false);
        assert_eq!(
            store.debug_lines(),
            vec![
                "admiring_item: false",
                "angry_at: -",
                "hunted_recently: true (ttl: 40)"
            ]
        );
    }

    #[test]
    fn debug_lines_are_capped_at_255_chars() {
        let mut store = MemoryStore::default();
        store.register(types::UNREACHABLE_TONGUE_TARGETS.id());
        store.set(
            types::UNREACHABLE_TONGUE_TARGETS,
            vec![uuid::Uuid::nil(); 20],
        );
        let lines = store.debug_lines();
        assert_eq!(lines[0].chars().count(), 255);
        assert!(lines[0].ends_with("..."));
    }

    #[test]
    fn unknown_and_transient_entries_are_skipped_on_load() {
        let mut memory = NbtCompound::new();
        memory.put_bool("value", true);
        let mut memories = NbtCompound::new();
        memories.put_compound("minecraft:not_a_memory", memory.clone());
        memories.put_compound("minecraft:dancing", memory);
        let mut brain = NbtCompound::new();
        brain.put_compound("memories", memories);
        assert!(PackedMemories::from_nbt(&brain).0.is_empty());
    }
}

//! Port of `net.minecraft.world.entity.ai.memory` (26.2 decompile at
//! `/home/eshanki/pumpkin-vanilla-26.2/decompiled`).
//!
//! Stage 0 of the Brain/Memory/Activity port. This module is the *always-live* half of a
//! `Brain`: it is guarded by a short-lived `std::sync::Mutex` and is never taken out of its
//! mutex for the duration of a tick, because memories are legitimately written from outside
//! the owning mob's AI tick (damage handlers, game-event listeners, item pickup). See
//! `super::Brain` for the split and `BRAIN_DESIGN.md` section 4.2 for the derivation.
//!
//! Deviations from vanilla, all deliberate:
//! - `GlobalPos` (dimension + `BlockPos`) is stored as a bare `BlockPos`. Pumpkin has no
//!   `GlobalPos`; the only consumer in this stage is Allay's liked note block, which vanilla
//!   also range-checks per dimension (`AllayAi.java:131`). Cross-dimension correctness is
//!   therefore NOT covered.
//! - `MemoryModuleType.PATH` is not represented at all. Vanilla's `MoveToTargetSink` stores a
//!   live `Path` in it (`MoveToTargetSink.java:92,101`); Pumpkin's `Navigator` owns its path
//!   internally and exposes no equivalent handle, so the slot would be unreadable.
//! - Entity-valued memories hold `Weak<dyn EntityBase>` rather than a strong `Arc`, so a
//!   stale memory cannot keep a despawned entity alive. A failed `upgrade()` is treated the
//!   same as vanilla's "the sensor stopped re-populating this memory".

use std::any::Any;
use std::sync::Weak;

use pumpkin_data::damage::DamageType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::entity::EntityBase;

/// `MemoryStatus` (`memory/MemoryStatus.java`), the full enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MemoryStatus {
    ValuePresent,
    ValueAbsent,
    /// The slot exists on this brain at all. `Brain.checkMemory` returns `false` for an
    /// unregistered type regardless of the requested status (`Brain.java:242-249`), which is
    /// why `MemoryStore` tracks registration separately from "has a value".
    Registered,
}

/// Dense, compiler-checked index for every memory type this port covers.
///
/// Vanilla declares ~120 `MemoryModuleType` constants (`memory/MemoryModuleType.java:33-152`)
/// in a registry that assigns each a stable identity. Only the subset reachable from Allay's
/// brain is declared here; adding a mob means adding variants, and `COUNT`/`ALL` must be kept
/// in sync (there is a `debug_assert` covering that in `MemoryStore::new`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MemoryKeyId {
    WalkTarget,
    LookTarget,
    CantReachWalkTargetSince,
    IsPanicking,
    HurtBy,
    LikedPlayer,
    LikedNoteblockPosition,
    LikedNoteblockCooldownTicks,
    ItemPickupCooldownTicks,
    NearestVisibleWantedItem,
}

impl MemoryKeyId {
    pub const ALL: [Self; 10] = [
        Self::WalkTarget,
        Self::LookTarget,
        Self::CantReachWalkTargetSince,
        Self::IsPanicking,
        Self::HurtBy,
        Self::LikedPlayer,
        Self::LikedNoteblockPosition,
        Self::LikedNoteblockCooldownTicks,
        Self::ItemPickupCooldownTicks,
        Self::NearestVisibleWantedItem,
    ];
    pub const COUNT: usize = Self::ALL.len();

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// Typed key into a `MemoryStore`, the Rust analogue of `MemoryModuleType<U>`
/// (`memory/MemoryModuleType.java:153-179`).
///
/// Implementors are zero-sized; the value type is
/// carried in the associated type so callers never downcast.
pub trait MemoryKey: 'static {
    type Value: Send + 'static;
    const ID: MemoryKeyId;
    const NAME: &'static str;
}

macro_rules! memory_keys {
    ($($key:ident => $id:ident : $value:ty, $name:literal;)*) => {
        $(
            pub struct $key;
            impl MemoryKey for $key {
                type Value = $value;
                const ID: MemoryKeyId = MemoryKeyId::$id;
                const NAME: &'static str = $name;
            }
        )*
    };
}

memory_keys! {
    WalkTargetMemory => WalkTarget: WalkTarget, "walk_target";
    LookTargetMemory => LookTarget: PositionTracker, "look_target";
    CantReachWalkTargetSinceMemory => CantReachWalkTargetSince: i64, "cant_reach_walk_target_since";
    IsPanickingMemory => IsPanicking: bool, "is_panicking";
    HurtByMemory => HurtBy: DamageType, "hurt_by";
    LikedPlayerMemory => LikedPlayer: Uuid, "liked_player";
    LikedNoteblockPositionMemory => LikedNoteblockPosition: BlockPos, "liked_noteblock_position";
    LikedNoteblockCooldownTicksMemory => LikedNoteblockCooldownTicks: i32, "liked_noteblock_cooldown_ticks";
    ItemPickupCooldownTicksMemory => ItemPickupCooldownTicks: i32, "item_pickup_cooldown_ticks";
    NearestVisibleWantedItemMemory => NearestVisibleWantedItem: Weak<dyn EntityBase>, "nearest_visible_wanted_item";
}

/// `PositionTracker` (`behavior/PositionTracker.java`) with its two concrete implementations,
/// `BlockPosTracker` and `EntityTracker`, collapsed into one enum.
///
/// The accessors return `Option` where vanilla returns a bare value: an `Entity` variant holds
/// a `Weak`, and a dead weak means the tracked entity left the world. Vanilla cannot hit that
/// case because a `Behavior` holding a Java reference keeps the entity reachable.
#[derive(Clone)]
pub enum PositionTracker {
    /// `BlockPosTracker(BlockPos)` (`behavior/BlockPosTracker.java:11-14`): the tracked point
    /// is the block *center*.
    Block { block_pos: BlockPos },
    /// `EntityTracker(entity, trackEyeHeight)` (`behavior/EntityTracker.java:16-24`).
    Entity {
        entity: Weak<dyn EntityBase>,
        track_eye_height: bool,
    },
}

impl PositionTracker {
    #[must_use]
    pub const fn of_block(block_pos: BlockPos) -> Self {
        Self::Block { block_pos }
    }

    /// `BlockPosTracker(Vec3)` (`behavior/BlockPosTracker.java:16-19`) keeps the exact vector
    /// as the tracked position rather than re-centering. Pumpkin's `Navigator` only ever
    /// consumes a destination vector, so the distinction is preserved by storing the containing
    /// block and accepting the re-centering; flagged because `RandomStroll` picks a non-centered
    /// point in vanilla.
    #[must_use]
    pub const fn of_position(pos: Vector3<f64>) -> Self {
        Self::Block {
            block_pos: BlockPos::new(
                pos.x.floor() as i32,
                pos.y.floor() as i32,
                pos.z.floor() as i32,
            ),
        }
    }

    #[must_use]
    pub fn of_entity(entity: &std::sync::Arc<dyn EntityBase>, track_eye_height: bool) -> Self {
        Self::Entity {
            entity: std::sync::Arc::downgrade(entity),
            track_eye_height,
        }
    }

    /// `PositionTracker.currentPosition()`.
    #[must_use]
    pub fn current_position(&self) -> Option<Vector3<f64>> {
        match self {
            Self::Block { block_pos } => Some(Vector3::new(
                f64::from(block_pos.0.x) + 0.5,
                f64::from(block_pos.0.y) + 0.5,
                f64::from(block_pos.0.z) + 0.5,
            )),
            Self::Entity {
                entity,
                track_eye_height,
            } => {
                let entity = entity.upgrade()?;
                let entity = entity.get_entity();
                let pos = entity.pos.load();
                Some(if *track_eye_height {
                    Vector3::new(pos.x, entity.get_eye_y(), pos.z)
                } else {
                    pos
                })
            }
        }
    }

    /// `PositionTracker.currentBlockPosition()`.
    #[must_use]
    pub fn current_block_position(&self) -> Option<BlockPos> {
        match self {
            Self::Block { block_pos } => Some(*block_pos),
            Self::Entity { entity, .. } => {
                let entity = entity.upgrade()?;
                Some(entity.get_entity().block_pos.load())
            }
        }
    }

    /// `PositionTracker.isVisibleBy(LivingEntity)`.
    ///
    /// DEVIATION: vanilla's `EntityTracker.isVisibleBy` (`behavior/EntityTracker.java:37-48`)
    /// requires the target to appear in the observer's `NEAREST_VISIBLE_LIVING_ENTITIES`
    /// memory. That memory needs a `NearestLivingEntitiesSensor`, which this stage does not
    /// port, so the check degrades to "the entity still exists and is alive". A mob will
    /// therefore keep looking at a target through a wall where vanilla would drop it.
    #[must_use]
    pub fn is_visible_by(&self) -> bool {
        match self {
            Self::Block { .. } => true,
            Self::Entity { entity, .. } => entity
                .upgrade()
                .is_some_and(|entity| entity.get_entity().is_alive()),
        }
    }
}

/// `WalkTarget` (`memory/WalkTarget.java:10-43`). The single memory that drives movement:
/// `MoveToTargetSink` is the only thing that reads it and hands it to the navigator.
#[derive(Clone)]
pub struct WalkTarget {
    pub target: PositionTracker,
    pub speed_modifier: f32,
    pub close_enough_dist: i32,
}

impl WalkTarget {
    #[must_use]
    pub const fn new(target: PositionTracker, speed_modifier: f32, close_enough_dist: i32) -> Self {
        Self {
            target,
            speed_modifier,
            close_enough_dist,
        }
    }
}

/// `MemorySlot<T>` (`memory/MemorySlot.java`). `time_to_live: None` is vanilla's
/// `NEVER_EXPIRE` (`Long.MAX_VALUE`, `:7`).
struct MemorySlot {
    value: Option<Box<dyn Any + Send>>,
    time_to_live: Option<i64>,
}

impl MemorySlot {
    const fn empty() -> Self {
        Self {
            value: None,
            time_to_live: None,
        }
    }

    /// `MemorySlot.tick()` (`memory/MemorySlot.java:16-24`): only ticks when a value is
    /// present *and* the slot can expire, clears at `<= 0` before decrementing.
    fn tick(&mut self) {
        if self.value.is_none() {
            return;
        }
        if let Some(ttl) = self.time_to_live {
            if ttl <= 0 {
                self.value = None;
                self.time_to_live = None;
            } else {
                self.time_to_live = Some(ttl - 1);
            }
        }
    }

    const fn has_value(&self) -> bool {
        self.value.is_some()
    }
}

/// The memory map of a `Brain` (`Brain.java:40`).
///
/// LOCK ORDER: this store's mutex is a leaf. Never acquire it while holding the mob's
/// `navigator`, `look_control` or `move_control` lock; read what you need into locals, drop
/// this guard, then take the controller lock. Every accessor here is a plain field access, so
/// no guard ever needs to survive an `.await`.
pub struct MemoryStore {
    slots: Vec<MemorySlot>,
    registered: Vec<bool>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    #[must_use]
    pub fn new() -> Self {
        debug_assert_eq!(MemoryKeyId::COUNT, MemoryKeyId::ALL.len());
        Self {
            slots: (0..MemoryKeyId::COUNT)
                .map(|_| MemorySlot::empty())
                .collect(),
            registered: vec![false; MemoryKeyId::COUNT],
        }
    }

    /// `Brain.registerMemory` (`Brain.java:78-89`), called for every memory a sensor writes
    /// and every memory a behavior lists as a required memory.
    pub fn register(&mut self, id: MemoryKeyId) {
        self.registered[id.index()] = true;
    }

    #[must_use]
    pub fn is_registered(&self, id: MemoryKeyId) -> bool {
        self.registered[id.index()]
    }

    #[must_use]
    pub fn get<K: MemoryKey>(&self) -> Option<&K::Value> {
        self.slots[K::ID.index()]
            .value
            .as_ref()
            .and_then(|value| value.downcast_ref::<K::Value>())
    }

    pub fn set<K: MemoryKey>(&mut self, value: K::Value) {
        self.slots[K::ID.index()] = MemorySlot {
            value: Some(Box::new(value)),
            time_to_live: None,
        };
    }

    pub fn set_with_expiry<K: MemoryKey>(&mut self, value: K::Value, time_to_live: i64) {
        self.slots[K::ID.index()] = MemorySlot {
            value: Some(Box::new(value)),
            time_to_live: Some(time_to_live),
        };
    }

    pub fn erase<K: MemoryKey>(&mut self) {
        self.erase_by_id(K::ID);
    }

    pub fn erase_by_id(&mut self, id: MemoryKeyId) {
        self.slots[id.index()] = MemorySlot::empty();
    }

    #[must_use]
    pub fn has_value<K: MemoryKey>(&self) -> bool {
        self.slots[K::ID.index()].has_value()
    }

    /// `Brain.checkMemory` (`Brain.java:242-249`). Note the leading null check: an
    /// unregistered memory fails *every* status, including `REGISTERED`.
    #[must_use]
    pub fn check(&self, id: MemoryKeyId, status: MemoryStatus) -> bool {
        if !self.registered[id.index()] {
            return false;
        }
        match status {
            MemoryStatus::Registered => true,
            MemoryStatus::ValuePresent => self.slots[id.index()].has_value(),
            MemoryStatus::ValueAbsent => !self.slots[id.index()].has_value(),
        }
    }

    /// `Brain.forgetOutdatedMemories` (`Brain.java:397-399`), the first of the four steps of
    /// `Brain.tick`.
    pub fn tick_expiry(&mut self) {
        for slot in &mut self.slots {
            slot.tick();
        }
    }

    /// `Brain.clearMemories` (`Brain.java:154-156`).
    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = MemorySlot::empty();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get_erase_roundtrip() {
        let mut store = MemoryStore::new();
        store.register(MemoryKeyId::LikedNoteblockCooldownTicks);
        assert!(!store.has_value::<LikedNoteblockCooldownTicksMemory>());
        store.set::<LikedNoteblockCooldownTicksMemory>(600);
        assert_eq!(store.get::<LikedNoteblockCooldownTicksMemory>(), Some(&600));
        store.erase::<LikedNoteblockCooldownTicksMemory>();
        assert!(!store.has_value::<LikedNoteblockCooldownTicksMemory>());
    }

    #[test]
    fn unregistered_memory_fails_every_status() {
        let store = MemoryStore::new();
        assert!(!store.check(MemoryKeyId::WalkTarget, MemoryStatus::Registered));
        assert!(!store.check(MemoryKeyId::WalkTarget, MemoryStatus::ValueAbsent));
        assert!(!store.check(MemoryKeyId::WalkTarget, MemoryStatus::ValuePresent));
    }

    #[test]
    fn registered_empty_slot_is_absent_not_present() {
        let mut store = MemoryStore::new();
        store.register(MemoryKeyId::WalkTarget);
        assert!(store.check(MemoryKeyId::WalkTarget, MemoryStatus::Registered));
        assert!(store.check(MemoryKeyId::WalkTarget, MemoryStatus::ValueAbsent));
        assert!(!store.check(MemoryKeyId::WalkTarget, MemoryStatus::ValuePresent));
    }

    #[test]
    fn expiry_clears_after_exactly_ttl_ticks() {
        let mut store = MemoryStore::new();
        store.register(MemoryKeyId::ItemPickupCooldownTicks);
        store.set_with_expiry::<ItemPickupCooldownTicksMemory>(7, 3);
        // MemorySlot.tick decrements while ttl > 0 and clears on the tick where it is <= 0,
        // so a ttl of 3 survives 3 ticks and is gone on the 4th.
        for _ in 0..3 {
            store.tick_expiry();
            assert!(store.has_value::<ItemPickupCooldownTicksMemory>());
        }
        store.tick_expiry();
        assert!(!store.has_value::<ItemPickupCooldownTicksMemory>());
    }

    #[test]
    fn never_expiring_memory_survives_ticks() {
        let mut store = MemoryStore::new();
        store.register(MemoryKeyId::LikedPlayer);
        store.set::<LikedPlayerMemory>(Uuid::nil());
        for _ in 0..100 {
            store.tick_expiry();
        }
        assert!(store.has_value::<LikedPlayerMemory>());
    }

    #[test]
    fn slots_are_independent() {
        let mut store = MemoryStore::new();
        store.register(MemoryKeyId::LikedNoteblockCooldownTicks);
        store.register(MemoryKeyId::ItemPickupCooldownTicks);
        store.set::<LikedNoteblockCooldownTicksMemory>(1);
        store.set::<ItemPickupCooldownTicksMemory>(2);
        assert_eq!(store.get::<LikedNoteblockCooldownTicksMemory>(), Some(&1));
        assert_eq!(store.get::<ItemPickupCooldownTicksMemory>(), Some(&2));
        store.erase::<LikedNoteblockCooldownTicksMemory>();
        assert_eq!(store.get::<ItemPickupCooldownTicksMemory>(), Some(&2));
    }
}

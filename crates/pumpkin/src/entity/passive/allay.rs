// Port of net.minecraft.world.entity.animal.allay.{Allay, AllayAi}, using the GameEvent/
// vibration engine in `crate::world::game_event`.
//
// Pumpkin has no Brain/Memory/Activity system (see PARITY.md), so `AllayAi`'s two
// activities (CORE / IDLE, each a `Brain` memory-gated `Behavior` list) are not portable
// as-is. This implementation keeps `LIKED_PLAYER` / `LIKED_NOTEBLOCK_POSITION` /
// `LIKED_NOTEBLOCK_COOLDOWN_TICKS` as plain fields on `AllayEntity`, ticked directly from
// `mob_tick`. Concretely ported: jukebox/note-block-triggered dancing (`setJukeboxPlaying`,
// `hearNoteblock`, `shouldStopDancing`), the duplication cooldown/predicate
// (`canDuplicate`/`duplicateAllay`), and the give/take/duplicate `mobInteract` branch.
// Explicitly NOT ported, with reasons:
//
// - `DATA_DANCING`/`DATA_CAN_DUPLICATE` client-synced booleans and the client-side
//   `holdingItemAnimationTicks`/`dancingAnimationTicks`/`spinningAnimationTicks` animation
//   state (Allay.java lines 80-81, 228-260, 393-404): Pumpkin's `Entity` has no generic
//   boolean tracked-data slot exposed to per-mob code the way vanilla's
//   `SynchedEntityData.Builder` does, and no client animation-state channel at all (same
//   gap noted in `warden.rs` for `AnimationState`). `is_dancing` below is server-side only:
//   an Allay here will duplicate/react correctly but the client will not visibly dance.
// - `AllayAi`'s item-carrying/delivery `Behavior`s (`GoToWantedItem`, `GoAndGiveItemsToTarget`,
//   `StayCloseToTarget`, `SensorType.NEAREST_ITEMS`): these depend on a "nearest wanted
//   item" sensor and a location-targeted walk-and-throw behavior, neither of which exist
//   in Pumpkin's Goal system (grepped `pumpkin/src/entity/ai/goal/` — no item-seeking or
//   deliver-to-position goal). `wantsToPickUp`/`allayConsidersItemEqual` (the item-match
//   predicate) and the single-slot inventory (`InventoryCarrier`) are ported as plain
//   methods for a future goal to call, but nothing currently drives an Allay to walk
//   toward a dropped item or to the liked note block to deposit it.
// - `hasNonMatchingPotion` (comparing `DataComponents.POTION_CONTENTS`): the general
//   item-equality check (`ItemStack::is_same_item`-equivalent) is ported; the potion-content
//   special case is left out pending confirmation `pumpkin_data::ItemStack` exposes potion
//   contents as a comparable data component the same way.
// - `LIKED_PLAYER`'s gamemode/distance liveness re-check (`AllayAi.getLikedPlayer`:
//   survival-or-creative and within 64 blocks) beyond what's needed for `mobInteract`.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::entity::player::Player;
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};
use crate::world::World;
use crate::world::game_event::{
    GameEventContext, GameEventFuture, GameEventListener, PositionSource,
};

/// `Allay.DUPLICATION_COOLDOWN_TICKS`
const DUPLICATION_COOLDOWN_TICKS: i64 = 6000;
/// `AllayAi.TIME_TO_FORGET_NOTEBLOCK`
const TIME_TO_FORGET_NOTEBLOCK_TICKS: i32 = 600;
/// `Allay.NUM_OF_DUPLICATION_HEARTS`. Currently unused: there is no heart-particle
/// broadcast channel wired up for this yet (see module doc comment on client sync gaps).
#[allow(dead_code)]
const NUM_OF_DUPLICATION_HEARTS: u32 = 3;
/// `Allay.MAX_NOTEBLOCK_DISTANCE` / the notification radius `GameEvent.JUKEBOX_PLAY` uses
/// (`shouldStopDancing` compares against `GameEvent.JUKEBOX_PLAY.value().notificationRadius()`,
/// which is 10 — see `crate::world::game_event::notification_radius`).
const JUKEBOX_DANCE_RADIUS: f64 = 10.0;

/// Represents an Allay, a passive, flying entity that can collect items for the player.
///
/// Wiki: <https://minecraft.wiki/w/Allay>
pub struct AllayEntity {
    pub mob_entity: MobEntity,
    is_dancing: AtomicBool,
    jukebox_pos: std::sync::Mutex<Option<BlockPos>>,
    liked_player: std::sync::Mutex<Option<Uuid>>,
    liked_noteblock_pos: std::sync::Mutex<Option<BlockPos>>,
    liked_noteblock_cooldown: std::sync::atomic::AtomicI32,
    duplication_cooldown: AtomicI64,
    listener_registered: AtomicBool,
    vibration_listener: std::sync::Mutex<Option<Arc<AllayVibrationListener>>>,
    jukebox_listener: std::sync::Mutex<Option<Arc<AllayJukeboxListener>>>,
}

impl AllayEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let allay = Self {
            mob_entity,
            is_dancing: AtomicBool::new(false),
            jukebox_pos: std::sync::Mutex::new(None),
            liked_player: std::sync::Mutex::new(None),
            liked_noteblock_pos: std::sync::Mutex::new(None),
            liked_noteblock_cooldown: std::sync::atomic::AtomicI32::new(0),
            duplication_cooldown: AtomicI64::new(0),
            listener_registered: AtomicBool::new(false),
            vibration_listener: std::sync::Mutex::new(None),
            jukebox_listener: std::sync::Mutex::new(None),
        };
        let mob_arc = Arc::new(allay);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
        goal_selector.add_goal(0, Box::new(SwimGoal::default()));
        goal_selector.add_goal(1, Box::new(WanderAroundGoal::new(1.0)));
        goal_selector.add_goal(
            2,
            LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
        );
        goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
        drop(goal_selector);

        let uuid = mob_arc.mob_entity.living_entity.entity.entity_uuid;
        *mob_arc.vibration_listener.lock().unwrap() = Some(Arc::new(AllayVibrationListener {
            allay: Arc::downgrade(&mob_arc),
            uuid,
        }));
        *mob_arc.jukebox_listener.lock().unwrap() = Some(Arc::new(AllayJukeboxListener {
            allay: Arc::downgrade(&mob_arc),
            uuid,
        }));

        mob_arc
    }

    #[must_use]
    pub fn is_dancing(&self) -> bool {
        self.is_dancing.load(Ordering::Relaxed)
    }

    /// `Allay.setDancing`, minus the `isEffectiveAi`/panic gate (Pumpkin has no panic-goal
    /// state check available here) and the client `DATA_DANCING` sync (see module doc
    /// comment).
    fn set_dancing(&self, dancing: bool) {
        self.is_dancing.store(dancing, Ordering::Relaxed);
    }

    /// `Allay.setJukeboxPlaying`
    fn set_jukebox_playing(&self, jukebox: BlockPos, is_playing: bool) {
        let mut jukebox_pos = self.jukebox_pos.lock().unwrap();
        if is_playing {
            if !self.is_dancing() {
                *jukebox_pos = Some(jukebox);
                drop(jukebox_pos);
                self.set_dancing(true);
            }
        } else if *jukebox_pos == Some(jukebox) || jukebox_pos.is_none() {
            *jukebox_pos = None;
            drop(jukebox_pos);
            self.set_dancing(false);
        }
    }

    /// `Allay.shouldStopDancing`, minus the "is it actually still a jukebox block" check
    /// (would need a block-state lookup call site outside `mob_tick`'s borrow shape; the
    /// distance check below is the one vanilla treats as authoritative in the common case
    /// of the jukebox simply running out of range or finishing its song, since
    /// `JukeboxSongPlayer` stops emitting `JUKEBOX_PLAY` once it does).
    fn should_stop_dancing(&self, my_pos: Vector3<f64>) -> bool {
        let jukebox_pos = *self.jukebox_pos.lock().unwrap();
        jukebox_pos.is_none_or(|pos| {
            let center = Vector3::new(
                f64::from(pos.0.x) + 0.5,
                f64::from(pos.0.y) + 0.5,
                f64::from(pos.0.z) + 0.5,
            );
            (center - my_pos).length_squared() > JUKEBOX_DANCE_RADIUS * JUKEBOX_DANCE_RADIUS
        })
    }

    /// `AllayAi.hearNoteblock`
    fn hear_noteblock(&self, pos: BlockPos) {
        let mut liked = self.liked_noteblock_pos.lock().unwrap();
        match *liked {
            None => {
                *liked = Some(pos);
                self.liked_noteblock_cooldown
                    .store(TIME_TO_FORGET_NOTEBLOCK_TICKS, Ordering::Relaxed);
            }
            Some(existing) if existing == pos => {
                self.liked_noteblock_cooldown
                    .store(TIME_TO_FORGET_NOTEBLOCK_TICKS, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    fn is_on_duplication_cooldown(&self) -> bool {
        self.duplication_cooldown.load(Ordering::Relaxed) > 0
    }

    fn reset_duplication_cooldown(&self) {
        self.duplication_cooldown
            .store(DUPLICATION_COOLDOWN_TICKS, Ordering::Relaxed);
    }

    /// `allayConsidersItemEqual`, minus the potion-content special case (see module doc
    /// comment).
    #[must_use]
    pub const fn considers_item_equal(a: &ItemStack, b: &ItemStack) -> bool {
        a.item.id == b.item.id
    }

    /// `Allay.duplicateAllay`
    async fn duplicate(&self) {
        let world = self.mob_entity.living_entity.entity.world.load_full();
        let pos = self.mob_entity.living_entity.entity.pos.load();
        let new_entity = Entity::new(world.clone(), pos, &EntityType::ALLAY);
        let clone = Self::new(new_entity);
        clone.reset_duplication_cooldown();
        self.reset_duplication_cooldown();
        world.spawn_entity(clone).await;
    }

    async fn register_listeners_once(&self) {
        if self.listener_registered.swap(true, Ordering::Relaxed) {
            return;
        }
        let vibration_listener = self.vibration_listener.lock().unwrap().clone();
        let jukebox_listener = self.jukebox_listener.lock().unwrap().clone();
        let world = self.mob_entity.living_entity.entity.world.load();
        if let Some(listener) = vibration_listener {
            world.register_game_event_listener(listener).await;
        }
        if let Some(listener) = jukebox_listener {
            world.register_game_event_listener(listener).await;
        }
    }
}

impl NBTStorage for AllayEntity {}

impl Mob for AllayEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.register_listeners_once().await;

            let age = self
                .mob_entity
                .living_entity
                .entity
                .age
                .load(Ordering::Relaxed);

            // `Allay.aiStep`: heal 1 HP every 10 ticks while alive.
            if age % 10 == 0 && self.mob_entity.living_entity.entity.is_alive() {
                self.mob_entity.living_entity.heal(1.0);
            }

            // `Allay.aiStep`'s `shouldStopDancing` check runs every 20 ticks.
            if self.is_dancing() && age % 20 == 0 {
                let pos = self.mob_entity.living_entity.entity.pos.load();
                if self.should_stop_dancing(pos) {
                    self.set_dancing(false);
                    *self.jukebox_pos.lock().unwrap() = None;
                }
            }

            // `Allay.updateDuplicationCooldown`
            if self.duplication_cooldown.load(Ordering::Relaxed) > 0 {
                self.duplication_cooldown.fetch_sub(1, Ordering::Relaxed);
            }

            // `CountDownCooldownTicks(LIKED_NOTEBLOCK_COOLDOWN_TICKS)`: once it reaches 0,
            // vanilla erases the memory entirely (see `AllayAi.shouldDepositItemsAtLikedNoteblock`
            // requiring the cooldown memory to be *present*).
            if self.liked_noteblock_cooldown.load(Ordering::Relaxed) > 0
                && self
                    .liked_noteblock_cooldown
                    .fetch_sub(1, Ordering::Relaxed)
                    <= 1
            {
                *self.liked_noteblock_pos.lock().unwrap() = None;
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let world = self.mob_entity.living_entity.entity.world.load_full();
            let held_slot = self
                .mob_entity
                .living_entity
                .entity_equipment
                .lock()
                .await
                .get(&EquipmentSlot::MAIN_HAND);
            let held = held_slot.lock().await.clone();
            let my_pos = self.mob_entity.living_entity.entity.pos.load();

            // `Allay.mobInteract`, dancing + DUPLICATES_ALLAYS + canDuplicate branch.
            if self.is_dancing()
                && item_stack
                    .item
                    .has_tag(&tag::Item::MINECRAFT_DUPLICATES_ALLAYS)
                && !self.is_on_duplication_cooldown()
            {
                self.duplicate().await;
                world.play_sound(
                    Sound::BlockAmethystBlockChime,
                    SoundCategory::Neutral,
                    &my_pos,
                );
                item_stack.decrement(1);
                return true;
            }

            // Empty-handed Allay + player holding an item: give it to the Allay.
            if held.is_empty() && !item_stack.is_empty() {
                let to_give = item_stack.copy_with_count(1);
                *held_slot.lock().await = to_give;
                item_stack.decrement(1);
                world.play_sound(Sound::EntityAllayItemGiven, SoundCategory::Neutral, &my_pos);
                *self.liked_player.lock().unwrap() = Some(player.get_entity().entity_uuid);
                return true;
            }

            // Allay holding an item + player empty-handed: take it back.
            if !held.is_empty() && item_stack.is_empty() {
                let taken =
                    std::mem::replace(&mut *held_slot.lock().await, ItemStack::EMPTY.clone());
                world.play_sound(Sound::EntityAllayItemTaken, SoundCategory::Neutral, &my_pos);
                *self.liked_player.lock().unwrap() = None;
                let mut taken = taken;
                if !player.inventory.insert_stack_anywhere(&mut taken).await {
                    player.drop_item(taken).await;
                }
                return true;
            }

            false
        })
    }
}

/// `Allay.VibrationUser` collapsed onto this project's `GameEventListener` trait.
struct AllayVibrationListener {
    allay: Weak<AllayEntity>,
    uuid: Uuid,
}

impl GameEventListener for AllayVibrationListener {
    fn listener_source(&self) -> PositionSource {
        PositionSource::Entity(self.uuid)
    }

    fn listener_radius(&self) -> i32 {
        // Allay.VibrationUser.VIBRATION_EVENT_LISTENER_RANGE
        16
    }

    fn handle_game_event<'a>(
        &'a self,
        _world: &'a Arc<World>,
        event: &'a GameEvent,
        _context: &'a GameEventContext,
        source_position: Vector3<f64>,
    ) -> GameEventFuture<'a> {
        Box::pin(async move {
            let Some(allay) = self.allay.upgrade() else {
                return false;
            };
            // GameEventTags.ALLAY_CAN_LISTEN is just `note_block_play`
            // (pumpkin-data/src/generated/tag.rs, GameEvent::MINECRAFT_ALLAY_CAN_LISTEN).
            if !matches!(event, GameEvent::NoteBlockPlay) {
                return false;
            }
            if allay.mob_entity.is_no_ai() {
                return false;
            }
            let pos = BlockPos::new(
                source_position.x.floor() as i32,
                source_position.y.floor() as i32,
                source_position.z.floor() as i32,
            );
            allay.hear_noteblock(pos);
            true
        })
    }
}

/// `Allay.JukeboxListener`.
struct AllayJukeboxListener {
    allay: Weak<AllayEntity>,
    uuid: Uuid,
}

impl GameEventListener for AllayJukeboxListener {
    fn listener_source(&self) -> PositionSource {
        PositionSource::Entity(self.uuid)
    }

    fn listener_radius(&self) -> i32 {
        // GameEvent.JUKEBOX_PLAY.value().notificationRadius() == 10
        10
    }

    fn handle_game_event<'a>(
        &'a self,
        _world: &'a Arc<World>,
        event: &'a GameEvent,
        _context: &'a GameEventContext,
        source_position: Vector3<f64>,
    ) -> GameEventFuture<'a> {
        Box::pin(async move {
            let Some(allay) = self.allay.upgrade() else {
                return false;
            };
            let pos = BlockPos::new(
                source_position.x.floor() as i32,
                source_position.y.floor() as i32,
                source_position.z.floor() as i32,
            );
            match event {
                GameEvent::JukeboxPlay => {
                    allay.set_jukebox_playing(pos, true);
                    true
                }
                GameEvent::JukeboxStopPlay => {
                    allay.set_jukebox_playing(pos, false);
                    true
                }
                _ => false,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplication_cooldown_constant_matches_vanilla() {
        assert_eq!(DUPLICATION_COOLDOWN_TICKS, 6000);
    }

    #[test]
    fn time_to_forget_noteblock_constant_matches_vanilla() {
        assert_eq!(TIME_TO_FORGET_NOTEBLOCK_TICKS, 600);
    }

    #[test]
    fn considers_item_equal_compares_by_item_id() {
        let a = ItemStack::new(1, &pumpkin_data::item::Item::DIAMOND);
        let b = ItemStack::new(3, &pumpkin_data::item::Item::DIAMOND);
        let c = ItemStack::new(1, &pumpkin_data::item::Item::EMERALD);
        assert!(AllayEntity::considers_item_equal(&a, &b));
        assert!(!AllayEntity::considers_item_equal(&a, &c));
    }
}

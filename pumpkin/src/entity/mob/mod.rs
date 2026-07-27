use super::{Entity, EntityBase, ai::pathfinder::Navigator, living::LivingEntity};
use crate::entity::EntityBaseFuture;
use crate::entity::ai::control::MoveControlTrait;
use crate::entity::ai::control::look_control::LookControl;
use crate::entity::ai::control::move_control::MoveControl;
use crate::entity::ai::goal::goal_selector::GoalSelector;
use crate::entity::player::Player;
use crate::world::World;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::position::BlockPos;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering};
use uuid::Uuid;

mod combat;
mod despawn;
mod entity_base;
mod interact;
mod spawn_rules;
mod sun_burn;

pub mod bat;
pub mod blaze;
pub mod breeze;
pub mod cave_spider;
pub mod creaking;
pub mod creeper;
pub mod elder_guardian;
pub mod enderman;
pub mod endermite;
pub mod equipment;
pub mod evoker;
pub mod ghast;
pub mod giant;
pub mod guardian;
pub mod hoglin;
pub mod illusioner;
pub mod magma_cube;
pub mod phantom;
pub mod piglin;
pub mod piglin_brute;
pub mod pillager;
pub mod ravager;
pub mod shulker;
pub mod silverfish;
pub mod skeleton;
pub mod slime;
pub mod spider;
pub mod vex;
pub mod vindicator;
pub mod warden;
pub mod witch;
pub mod zoglin;
pub mod zombie;
pub mod zombification;
pub mod zombified_piglin;

pub struct MobEntity {
    pub living_entity: LivingEntity,
    pub goals_selector: std::sync::Mutex<GoalSelector>,
    pub target_selector: std::sync::Mutex<GoalSelector>,
    pub navigator: std::sync::Mutex<Navigator>,
    pub target: tokio::sync::Mutex<Option<Arc<dyn EntityBase>>>,
    pub look_control: std::sync::Mutex<LookControl>,
    pub move_control: std::sync::Mutex<Box<dyn MoveControlTrait>>,
    pub position_target: AtomicCell<BlockPos>,
    pub position_target_range: AtomicI32,
    pub love_ticks: AtomicI32,
    pub breeding_cooldown: AtomicI32,
    pub breeder: AtomicCell<Option<Uuid>>,
    /// Vanilla `Zombie.canBreakDoors`. Kept on `MobEntity` so the shared
    /// `BreakDoorGoal` and the pathfinder door flag stay in sync.
    can_break_doors: AtomicBool,
    /// Set when `CanBreakDoors` came back from NBT, so the spawn roll in
    /// `finalizeSpawn` does not overwrite a saved value on chunk load.
    can_break_doors_loaded: AtomicBool,
    /// Vanilla `Mob.despawnCounter` — ticks spent far enough from players to roll despawn.
    despawn_counter: AtomicI32,
    mob_flags: AtomicU8,
    last_sent_yaw: AtomicU8,
    last_sent_pitch: AtomicU8,
    last_sent_head_yaw: AtomicU8,
}

impl MobEntity {
    const AI_DISABLED_FLAG: u8 = 1;
    const LEFT_HANDED_FLAG: u8 = 2;
    const ATTACKING_FLAG: u8 = 4;
    const CAN_PICK_UP_LOOT_FLAG: u8 = 8;

    #[must_use]
    pub fn new(entity: Entity) -> Self {
        Self {
            living_entity: LivingEntity::new(entity),
            goals_selector: std::sync::Mutex::new(GoalSelector::default()),
            target_selector: std::sync::Mutex::new(GoalSelector::default()),
            navigator: std::sync::Mutex::new(Navigator::default()),
            target: tokio::sync::Mutex::new(None),
            look_control: std::sync::Mutex::new(LookControl::default()),
            move_control: std::sync::Mutex::new(Box::new(MoveControl::default())),
            position_target: AtomicCell::new(BlockPos::ZERO),
            position_target_range: AtomicI32::new(-1),
            love_ticks: AtomicI32::new(0),
            breeding_cooldown: AtomicI32::new(0),
            breeder: AtomicCell::new(None),
            can_break_doors: AtomicBool::new(false),
            can_break_doors_loaded: AtomicBool::new(false),
            despawn_counter: AtomicI32::new(0),
            mob_flags: AtomicU8::new(0),
            last_sent_yaw: AtomicU8::new(0),
            last_sent_pitch: AtomicU8::new(0),
            last_sent_head_yaw: AtomicU8::new(0),
        }
    }

    /// Vanilla `Mob.getNoActionTime()` (Mob.java:265-267). Pumpkin tracks the
    /// same counter as `despawn_counter` (yarn `despawnCounter`); vanilla
    /// increments it in `checkDespawn` and resets it on damage/target changes.
    #[must_use]
    pub fn no_action_time(&self) -> i32 {
        self.despawn_counter.load(Relaxed)
    }

    pub fn is_in_position_target_range(&self) -> bool {
        self.is_in_position_target_range_pos(&self.living_entity.entity.block_pos.load())
    }

    pub fn is_in_position_target_range_pos(&self, block_pos: &BlockPos) -> bool {
        let position_target_range = self.position_target_range.load(Relaxed);
        if position_target_range == -1 {
            true
        } else {
            self.position_target.load().squared_distance(block_pos)
                < position_target_range * position_target_range
        }
    }

    pub fn set_attacking(&self, attacking: bool) {
        self.set_mob_flag(Self::ATTACKING_FLAG, attacking);
    }

    pub fn is_attacking(&self) -> bool {
        (self.mob_flags.load(Relaxed) & Self::ATTACKING_FLAG) != 0
    }

    pub fn set_left_handed(&self, left_handed: bool) {
        self.set_mob_flag(Self::LEFT_HANDED_FLAG, left_handed);
    }

    /// Vanilla `Zombie.canBreakDoors`.
    pub fn can_break_doors(&self) -> bool {
        self.can_break_doors.load(Relaxed)
    }

    /// Vanilla `Zombie.setCanBreakDoors` — also flips the navigation flag so the
    /// mob is willing to path through a closed wooden door.
    pub fn set_can_break_doors(&self, value: bool) {
        self.can_break_doors.store(value, Relaxed);
        if let Ok(mut navigator) = self.navigator.lock() {
            navigator.set_can_open_doors(value);
        }
    }

    /// Marks the door-breaking flag as restored from NBT.
    pub fn set_can_break_doors_from_nbt(&self, value: bool) {
        self.can_break_doors_loaded.store(true, Relaxed);
        self.set_can_break_doors(value);
    }

    pub fn can_break_doors_loaded(&self) -> bool {
        self.can_break_doors_loaded.load(Relaxed)
    }

    pub fn can_pick_up_loot(&self) -> bool {
        (self.mob_flags.load(Relaxed) & Self::CAN_PICK_UP_LOOT_FLAG) != 0
    }

    pub fn set_can_pick_up_loot(&self, value: bool) {
        self.set_mob_flag(Self::CAN_PICK_UP_LOOT_FLAG, value);
    }

    pub fn is_left_handed(&self) -> bool {
        (self.mob_flags.load(Relaxed) & Self::LEFT_HANDED_FLAG) != 0
    }

    pub fn set_no_ai(&self, no_ai: bool) {
        self.set_mob_flag(Self::AI_DISABLED_FLAG, no_ai);
    }

    pub fn is_no_ai(&self) -> bool {
        (self.mob_flags.load(Relaxed) & Self::AI_DISABLED_FLAG) != 0
    }

    fn set_mob_flag(&self, flag: u8, value: bool) {
        let old_b = self.mob_flags.load(Ordering::Relaxed);

        let new_b = if value { old_b | flag } else { old_b & !flag };

        if new_b != old_b {
            self.mob_flags.store(new_b, Ordering::Relaxed);

            self.living_entity.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::MOB_FLAGS_ID,
                    MetaDataType::BYTE,
                    new_b,
                )],
                None,
            );
        }
    }

    pub fn is_in_love(&self) -> bool {
        self.love_ticks.load(Relaxed) > 0
    }

    pub fn set_love_ticks(&self, ticks: i32, breeder: Option<Uuid>) {
        self.love_ticks.store(ticks, Relaxed);
        self.breeder.store(breeder);
    }

    pub fn reset_love_ticks(&self) {
        self.love_ticks.store(0, Relaxed);
    }

    pub fn is_breeding_ready(&self) -> bool {
        self.living_entity.entity.age.load(Relaxed) >= 0
            && self.breeding_cooldown.load(Relaxed) <= 0
    }
}

pub trait Mob: EntityBase + Send + Sync {
    fn get_random(&self) -> rand::rngs::ThreadRng {
        rand::rng()
    }

    fn get_max_look_yaw_change(&self) -> f32 {
        10.0
    }

    fn get_max_look_pitch_change(&self) -> f32 {
        40.0
    }

    fn get_max_head_rotation(&self) -> f32 {
        75.0
    }

    fn get_mob_entity(&self) -> &MobEntity;

    fn get_job_site(&self) -> Option<BlockPos> {
        None
    }

    fn get_home(&self) -> Option<BlockPos> {
        None
    }

    fn get_path_aware_entity(&self) -> Option<&dyn PathAwareEntity> {
        None
    }

    /// Whether this mob renders as a baby. Age-based by default; zombies use a
    /// permanent flag instead.
    fn is_mob_baby(&self) -> bool {
        self.get_mob_entity()
            .living_entity
            .entity
            .age
            .load(std::sync::atomic::Ordering::Relaxed)
            < 0
    }

    /// Whether this mob type participates in vanilla `Zombie.setCanBreakDoors`
    /// (ground-navigating zombies). Drowned navigate water and are excluded.
    fn supports_break_door_goal(&self) -> bool {
        false
    }

    /// Vanilla `Mob.requiresCustomPersistence`. Used by stateful conversions
    /// such as a curing zombie villager, which must survive normal despawn checks.
    fn requires_custom_persistence(&self) -> bool {
        false
    }

    /// Per-mob tick hook called each tick before AI runs. Override for mob-specific logic.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {})
    }

    fn post_tick(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {})
    }

    /// Called before damage is applied. Return `false` to cancel the damage entirely.
    /// Used by endermen to dodge projectiles via teleportation.
    fn pre_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        _source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async { true })
    }

    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        _source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {})
    }

    fn on_eating_grass(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {})
    }

    fn modify_incoming_damage(&self, amount: f32, _damage_type: DamageType) -> f32 {
        amount
    }

    fn can_attack_with_owner(&self, _target: &dyn EntityBase, _owner: &dyn EntityBase) -> bool {
        true
    }

    fn get_mob_gravity(&self) -> f64 {
        self.get_mob_entity().living_entity.get_gravity()
    }

    fn get_mob_y_velocity_drag(&self) -> Option<f64> {
        None
    }

    /// Set or clear the mob's target. Override to add side effects when targeting changes.
    fn set_mob_target(&self, target: Option<Arc<dyn EntityBase>>) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let mut mob_target = self.get_mob_entity().target.lock().await;
            *mob_target = target;
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move { self.get_mob_entity().mob_interact(player, item_stack).await })
    }

    fn mob_player_collision<'a>(&'a self, _player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {})
    }

    fn get_owner_uuid(&self) -> Option<Uuid> {
        None
    }

    fn is_sitting(&self) -> bool {
        false
    }

    fn get_base_experience_reward(&self) -> u32 {
        self.get_entity().entity_type.experience_reward
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let is_baby = self.is_mob_baby();
            if is_baby {
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
        })
    }

    fn mob_set_variant_name(&self, _name: &str) {}
}

#[expect(dead_code)]
const DEFAULT_PATHFINDING_FAVOR: f32 = 0.0;

pub trait PathAwareEntity: Mob + Send + Sync {
    fn get_pathfinding_favor(&self, _block_pos: BlockPos, _world: Arc<World>) -> f32 {
        0.0
    }

    // TODO: missing SpawnReason attribute
    fn can_spawn(&self, world: Arc<World>) -> bool {
        self.get_pathfinding_favor(
            self.get_mob_entity().living_entity.entity.block_pos.load(),
            world,
        ) >= 0.0
    }

    fn is_navigation<'a>(&'a self) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async {
            let navigator = self.get_mob_entity().navigator.lock().unwrap();
            !navigator.is_idle()
        })
    }

    // TODO: implement
    fn is_panicking(&self) -> bool {
        false
    }

    fn should_follow_leash(&self) -> bool {
        true
    }

    fn on_short_leash_tick(&self) {
        // TODO: implement
    }

    fn before_leash_tick(&self) {
        // TODO: implement
    }

    fn get_follow_leash_speed(&self) -> f32 {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time assertions that `MobEntity`'s public surface survived the
    // module split (constructor plus the state accessors kept in this file).
    const _: fn(Entity) -> MobEntity = MobEntity::new;
    const _: fn(&MobEntity, &BlockPos) -> bool = MobEntity::is_in_position_target_range_pos;
    const _: fn(&MobEntity, bool) = MobEntity::set_attacking;
    const _: fn(&MobEntity) -> bool = MobEntity::is_attacking;
    const _: fn(&MobEntity, bool) = MobEntity::set_can_break_doors;
    const _: fn(&MobEntity) -> bool = MobEntity::can_break_doors;
    const _: fn(&MobEntity) -> bool = MobEntity::can_break_doors_loaded;
    const _: fn(&MobEntity, bool) = MobEntity::set_can_pick_up_loot;
    const _: fn(&MobEntity) -> bool = MobEntity::can_pick_up_loot;
    const _: fn(&MobEntity, i32, Option<Uuid>) = MobEntity::set_love_ticks;
    const _: fn(&MobEntity) -> bool = MobEntity::is_in_love;
    const _: fn(&MobEntity) -> bool = MobEntity::is_breeding_ready;
}

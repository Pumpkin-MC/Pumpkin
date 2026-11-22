use std::sync::{
    Arc,
    atomic::{AtomicI32, AtomicU32, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::damage::DamageType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;
use rand::rngs::ThreadRng;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::brain::{
        Brain,
        memory::{PackedMemories, types},
    },
    custom_sound::CustomSound,
    mob::{Mob, MobEntity, drop_from_gift_loot_table},
    passive::animal::Animal,
    player::Player,
};
use crate::world::game_event::GameEventContext;

mod ai;

const ARMADILLO_BRUSH_LOOT_TABLE: &str = "minecraft:brush/armadillo";
const ARMADILLO_SHED_LOOT_TABLE: &str = "minecraft:gameplay/armadillo_shed";
/// Vanilla `20 * TimeUtil.SECONDS_PER_MINUTE * 5`.
const SCUTE_DROP_TICKS: i32 = 20 * 60 * 5;
/// How long a scare lasts after the sensor or a hit last saw the danger.
pub const SCARE_CHECK_INTERVAL: i32 = 80;
const SCARE_DISTANCE_HORIZONTAL: f64 = 7.0;
const SCARE_DISTANCE_VERTICAL: f64 = 2.0;
pub const BRUSH_DURABILITY_COST: i32 = 16;
/// Vanilla `getMaxHeadYRot` while it is not rolled up.
const MAX_HEAD_Y_ROT: f32 = 32.0;

fn pick_next_scute_drop_time(random: &mut ThreadRng) -> i32 {
    random.random_range(0..SCUTE_DROP_TICKS) + SCUTE_DROP_TICKS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum ArmadilloState {
    #[default]
    Idle = 0,
    Rolling = 1,
    Scared = 2,
    Unrolling = 3,
}

impl ArmadilloState {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Rolling,
            2 => Self::Scared,
            3 => Self::Unrolling,
            _ => Self::Idle,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn is_threatened(self) -> bool {
        !matches!(self, Self::Idle)
    }

    #[must_use]
    pub const fn animation_duration(self) -> u8 {
        match self {
            Self::Idle => 0,
            Self::Rolling => 10,
            Self::Scared => 50,
            Self::Unrolling => 30,
        }
    }

    #[must_use]
    pub const fn should_hide_in_shell(self, ticks_in_state: u32) -> bool {
        match self {
            Self::Idle => false,
            Self::Rolling => ticks_in_state > 5,
            Self::Scared => true,
            Self::Unrolling => ticks_in_state < 26,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Rolling => "rolling",
            Self::Scared => "scared",
            Self::Unrolling => "unrolling",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "rolling" => Self::Rolling,
            "scared" => Self::Scared,
            "unrolling" => Self::Unrolling,
            _ => Self::Idle,
        }
    }
}

/// Represents an Armadillo, a passive entity that can roll into a ball when threatened.
///
/// Wiki: <https://minecraft.wiki/w/Armadillo>
pub struct ArmadilloEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub state: AtomicI32,
    pub in_state_ticks: AtomicU32,
    pub scute_time: AtomicI32,
}

impl ArmadilloEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_can_float(true);
        Arc::new(Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            state: AtomicI32::new(ArmadilloState::Idle.id()),
            in_state_ticks: AtomicU32::new(0),
            scute_time: AtomicI32::new(pick_next_scute_drop_time(&mut rand::rng())),
        })
    }

    #[must_use]
    pub fn get_state(&self) -> ArmadilloState {
        ArmadilloState::from_id(self.state.load(Ordering::Relaxed))
    }

    pub fn switch_to_state(&self, state: ArmadilloState) {
        self.state.store(state.id(), Ordering::Relaxed);
        self.in_state_ticks.store(0, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::armadillo::ARMADILLO_STATE,
            VarInt(state.id()),
        );
    }

    #[must_use]
    pub fn is_scared(&self) -> bool {
        self.get_state() != ArmadilloState::Idle
    }

    #[must_use]
    pub fn should_hide_in_shell(&self) -> bool {
        self.get_state()
            .should_hide_in_shell(self.in_state_ticks.load(Ordering::Relaxed))
    }

    #[must_use]
    pub fn should_switch_to_scared_state(&self) -> bool {
        self.get_state() == ArmadilloState::Rolling
            && self.in_state_ticks.load(Ordering::Relaxed)
                > u32::from(ArmadilloState::Rolling.animation_duration())
    }

    /// Vanilla `canStayRolledUp`, with `panicking` given by the caller. Inside the brain it comes
    /// from the brain state: the brain is already borrowed there.
    #[must_use]
    pub fn can_stay_rolled_up_while(&self, panicking: bool) -> bool {
        let entity = self.get_entity();
        !panicking
            && !entity.is_in_liquid()
            && !entity.is_leashed()
            && !entity.has_vehicle()
            && !entity.has_passengers()
    }

    #[must_use]
    pub fn can_stay_rolled_up(&self) -> bool {
        self.can_stay_rolled_up_while(self.is_panicking())
    }

    fn emit_game_event(&self, event: GameEvent) {
        let entity = self.get_entity();
        entity.world.load().emit_game_event_with_context(
            event.name(),
            entity.pos.load(),
            &GameEventContext {
                source_entity: Some(self),
            },
        );
    }

    pub fn roll_up(&self) {
        if self.is_scared() {
            return;
        }
        self.mob_entity.stop_in_place();
        self.mob_entity.reset_love_ticks();
        self.emit_game_event(GameEvent::EntityAction);
        self.make_sound(Sound::EntityArmadilloRoll);
        self.switch_to_state(ArmadilloState::Rolling);
    }

    pub fn roll_out(&self) {
        if !self.is_scared() {
            return;
        }
        self.emit_game_event(GameEvent::EntityAction);
        self.make_sound(Sound::EntityArmadilloUnrollFinish);
        self.switch_to_state(ArmadilloState::Idle);
    }

    pub fn brush_off_scute(
        &self,
        _interacting_entity: Option<&dyn EntityBase>,
        tool: &ItemStack,
    ) -> bool {
        if self.is_baby() || !self.get_entity().is_alive() {
            return false;
        }
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        if let Some(table) = world.get_loot_table(ARMADILLO_BRUSH_LOOT_TABLE) {
            let params = crate::world::loot::LootContextParameters {
                world: Some(&world),
                position: Some(pos),
                this_entity: Some(self),
                tool: Some(tool.clone()),
                ..Default::default()
            };
            for drop_stack in crate::world::loot::generate_loot_from_handle(
                &table,
                self.get_random().random(),
                &params,
            ) {
                entity.spawn_at_location(drop_stack);
            }
        }
        world.play_sound(Sound::EntityArmadilloBrush, SoundCategory::Neutral, &pos);
        self.emit_game_event(GameEvent::EntityInteract);
        true
    }

    /// Vanilla `isScaredBy`: undead, whoever hurt it last, and players that sprint or ride.
    #[must_use]
    pub fn is_scared_by(&self, other: &dyn EntityBase) -> bool {
        let entity = self.get_entity();
        let other_entity = other.get_entity();
        let scare_box = entity.bounding_box.load().expand(
            SCARE_DISTANCE_HORIZONTAL,
            SCARE_DISTANCE_VERTICAL,
            SCARE_DISTANCE_HORIZONTAL,
        );
        if !scare_box.intersects(&other_entity.bounding_box.load()) {
            return false;
        }
        if other_entity
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_UNDEAD)
        {
            return true;
        }
        if self.mob_entity.living_entity.get_last_hurt_by_mob_id() == Some(other_entity.entity_id) {
            return true;
        }
        other.get_player().is_some_and(|player| {
            !player.is_spectator() && (other_entity.is_sprinting() || other_entity.has_vehicle())
        })
    }

    /// The scute timer of `customServerAiStep`.
    fn shed_scute(&self) {
        let entity = self.get_entity();
        // The timer keeps running below zero while loot is off or it is a baby.
        if !entity.is_alive()
            || self
                .scute_time
                .fetch_sub(1, Ordering::Relaxed)
                .wrapping_sub(1)
                > 0
            || !Mob::should_drop_loot(self)
        {
            return;
        }
        if drop_from_gift_loot_table(self, ARMADILLO_SHED_LOOT_TABLE, |drop_stack| {
            entity.spawn_at_location(drop_stack);
        }) {
            let mut random = self.get_random();
            let pitch = (random.random::<f32>() - random.random::<f32>()).mul_add(0.2, 1.0);
            self.play_entity_sound(Sound::EntityArmadilloScuteDrop, 1.0, pitch);
            self.emit_game_event(GameEvent::EntityPlace);
        }
        self.scute_time.store(
            pick_next_scute_drop_time(&mut self.get_random()),
            Ordering::Relaxed,
        );
    }
}

impl AgeableMob for ArmadilloEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for ArmadilloEntity {
    fn play_eating_sound(&self) {
        self.make_sound(Sound::EntityArmadilloEat);
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_ARMADILLO_FOOD)
    }

    fn can_fall_in_love(&self) -> bool {
        !self.mob_entity.is_in_love() && !self.is_scared()
    }
}

impl CustomSound for ArmadilloEntity {
    fn death_sound(&self) -> Option<Sound> {
        None
    }

    fn hurt_sound(&self) -> Option<Sound> {
        self.is_scared()
            .then_some(Sound::EntityArmadilloHurtReduced)
    }
}

impl Mob for ArmadilloEntity {
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityArmadilloStep, 0.15, 1.0);
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        if self.is_scared() {
            None
        } else {
            self.get_entity().entity_type.ambient_sound
        }
    }

    fn get_max_head_rotation(&self) -> f32 {
        if self.is_scared() {
            0.0
        } else {
            MAX_HEAD_Y_ROT
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("state", self.get_state().name().to_string());
        nbt.put_int("scute_time", self.scute_time.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(state_name) = nbt.get_string("state") {
            self.switch_to_state(ArmadilloState::from_name(state_name));
        } else if let Some(state_id) = nbt.get_int("state") {
            self.switch_to_state(ArmadilloState::from_id(state_id));
        }
        if let Some(scute_time) = nbt.get_int("scute_time") {
            self.scute_time.store(scute_time, Ordering::Relaxed);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    /// Vanilla `actuallyHurt`: a living attacker scares it, burning or freezing unrolls it.
    fn actually_hurt(
        &self,
        caller: &dyn EntityBase,
        damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
        damage: f32,
    ) {
        self.mob_entity.reset_love_ticks();
        self.mob_entity
            .living_entity
            .actually_hurt(caller, damage_type, source, cause, damage);
        if self.mob_entity.is_no_ai() || self.mob_entity.living_entity.is_dead_or_dying() {
            return;
        }
        if cause
            .or(source)
            .is_some_and(|attacker| attacker.get_living_entity().is_some())
        {
            self.mob_entity.with_brain(self, |brain| {
                brain.set_with_expiry(
                    types::DANGER_DETECTED_RECENTLY,
                    true,
                    i64::from(SCARE_CHECK_INTERVAL),
                );
            });
            if self.can_stay_rolled_up() {
                self.roll_up();
            }
        } else if damage_type.has_tag(&tag::DamageType::MINECRAFT_PANIC_ENVIRONMENTAL_CAUSES) {
            self.roll_out();
        }
    }

    fn modify_incoming_damage(&self, amount: f32, _damage_type: DamageType) -> f32 {
        if self.is_scared() {
            (amount - 1.0).max(0.0) / 2.0
        } else {
            amount
        }
    }

    /// Vanilla `Armadillo$1`: a rolled up armadillo does not turn its body.
    fn ticks_body_rotation(&self) -> bool {
        !self.is_scared()
    }

    /// The tail of vanilla `tick()`, after the look control has run.
    fn post_tick(&self) {
        if self.is_scared() {
            // `get_max_head_rotation` is 0 while scared, so the head stays on the body.
            let entity = self.get_entity();
            entity.head_yaw.store(entity.body_yaw.load());
        }
        self.in_state_ticks.fetch_add(1, Ordering::Relaxed);
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.mob_entity.tick_brain(self);
        self.mob_entity.with_brain(self, ai::update_activity);
        self.shed_scute();
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::armadillo::BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::armadillo::ARMADILLO_STATE,
            VarInt(self.get_state().id()),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if item_stack.item == &Item::BRUSH
            && self.brush_off_scute(Some(player.as_ref()), item_stack)
        {
            if player.gamemode.load() != pumpkin_util::GameMode::Creative {
                let _ = item_stack.damage_item(BRUSH_DURABILITY_COST);
            }
            return true;
        }
        if self.is_scared() {
            return false;
        }
        self.animal_interact(player, item_stack)
    }

    /// Vanilla returns `InteractionResult.FAIL` while scared, which also skips the ageable feeding.
    fn mob_interact_with_hand(
        &self,
        _player: &Arc<Player>,
        item_stack: &mut ItemStack,
        _hand: pumpkin_util::Hand,
    ) -> Option<bool> {
        (self.is_scared() && item_stack.item != &Item::BRUSH).then_some(false)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering::Relaxed;

    use pumpkin_data::entity::EntityType;
    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::ai::test_support::world;
    use crate::entity::mob::zombie::zombie::ZombieEntity;
    use crate::entity::passive::pig::PigEntity;
    use crate::world::World;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn armadillo(world: &Arc<World>) -> Arc<ArmadilloEntity> {
        let armadillo = ArmadilloEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::ARMADILLO,
        ));
        armadillo.get_entity().on_ground.store(true, Relaxed);
        armadillo
    }

    fn zombie(world: &Arc<World>, x: f64, y: f64) -> Arc<ZombieEntity> {
        ZombieEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, y, 0.5),
            &EntityType::ZOMBIE,
        ))
    }

    #[tokio::test]
    async fn the_scare_box_is_seven_blocks_wider_and_remembers_the_attacker() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let armadillo = armadillo(&world);
        let pig = PigEntity::new(Entity::new(
            world.clone(),
            Vector3::new(2.5, 64.0, 0.5),
            &EntityType::PIG,
        ));

        // The box is inflated by 7 blocks sideways and 2 up and down, so it reaches past the
        // bounding boxes: 7.65 sideways, and a zombie whose head is at 62.95 still overlaps it.
        let results = (
            armadillo.is_scared_by(zombie(&world, 8.0, 64.0).as_ref()),
            armadillo.is_scared_by(zombie(&world, 8.5, 64.0).as_ref()),
            armadillo.is_scared_by(zombie(&world, 2.5, 61.0).as_ref()),
            armadillo.is_scared_by(zombie(&world, 2.5, 60.0).as_ref()),
            armadillo.is_scared_by(pig.as_ref()),
        );
        armadillo
            .mob_entity
            .living_entity
            .set_last_hurt_by_mob(Some(pig.as_ref()));
        let remembered = armadillo.is_scared_by(pig.as_ref());
        world.level.shutdown().await;

        assert_eq!(results, (true, false, true, false, false));
        assert!(remembered);
        Ok(())
    }

    #[tokio::test]
    async fn a_baby_keeps_counting_down_until_it_can_shed() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let armadillo = armadillo(&world);
        armadillo.set_baby(true);
        armadillo.scute_time.store(1, Relaxed);

        armadillo.shed_scute();
        let as_baby = armadillo.scute_time.load(Relaxed);
        armadillo.set_baby(false);
        armadillo.shed_scute();
        let as_adult = armadillo.scute_time.load(Relaxed);
        world.level.shutdown().await;

        assert_eq!(as_baby, 0);
        assert!(as_adult >= SCUTE_DROP_TICKS);
        Ok(())
    }

    #[tokio::test]
    async fn a_scute_timer_read_from_nbt_at_the_integer_minimum_does_not_overflow() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let armadillo = armadillo(&world);
        armadillo.scute_time.store(i32::MIN, Relaxed);

        armadillo.shed_scute();
        let wrapped = armadillo.scute_time.load(Relaxed);
        world.level.shutdown().await;

        assert_eq!(wrapped, i32::MAX);
        Ok(())
    }

    #[tokio::test]
    async fn a_hit_by_a_living_attacker_stops_it_and_rolls_it_up_unless_it_dies() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let (killed, hurt) = (armadillo(&world), armadillo(&world));
        let attacker = zombie(&world, 1.5, 64.0);
        for armadillo in [&killed, &hurt] {
            armadillo
                .get_entity()
                .velocity
                .store(Vector3::new(0.3, 0.0, 0.3));
            armadillo.mob_entity.set_love_ticks(600, None);
        }

        for (armadillo, damage) in [(&killed, 100.0), (&hurt, 1.0)] {
            armadillo.actually_hurt(
                armadillo.as_ref(),
                DamageType::MOB_ATTACK,
                Some(attacker.as_ref()),
                Some(attacker.as_ref()),
                damage,
            );
        }
        let after_kill = (killed.get_state(), killed.get_entity().velocity.load());
        let after_hit = (hurt.get_state(), hurt.get_entity().velocity.load());
        let still_in_love = (killed.mob_entity.is_in_love(), hurt.mob_entity.is_in_love());
        world.level.shutdown().await;

        assert_eq!(
            after_kill,
            (ArmadilloState::Idle, Vector3::new(0.3, 0.0, 0.3))
        );
        assert_eq!(after_hit, (ArmadilloState::Rolling, Vector3::default()));
        assert_eq!(still_in_love, (false, false));
        Ok(())
    }

    #[tokio::test]
    async fn it_rolls_up_while_a_zombie_is_near_and_unrolls_after_it_is_gone() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let armadillo = armadillo(&world);
        let zombie = zombie(&world, 3.5, 64.0);
        let with_zombie: Vec<Arc<dyn EntityBase>> = vec![armadillo.clone(), zombie];
        let alone: Vec<Arc<dyn EntityBase>> = vec![armadillo.clone()];
        world.entities.store(Arc::new(with_zombie));

        let mut states = vec![armadillo.get_state()];
        let mut activity = None;
        for time in 0..220 {
            if time == 40 {
                world.entities.store(Arc::new(alone.clone()));
            }
            armadillo
                .mob_entity
                .with_brain(armadillo.as_ref(), |brain| {
                    brain.tick(&world, armadillo.as_ref(), time);
                    ai::update_activity(brain);
                    activity = brain.get_active_non_core_activity();
                });
            armadillo.in_state_ticks.fetch_add(1, Relaxed);
            if states.last() != Some(&armadillo.get_state()) {
                states.push(armadillo.get_state());
            }
        }
        world.level.shutdown().await;

        assert_eq!(
            states,
            vec![
                ArmadilloState::Idle,
                ArmadilloState::Rolling,
                ArmadilloState::Scared,
                ArmadilloState::Unrolling,
                ArmadilloState::Idle,
            ]
        );
        assert_eq!(activity, Some(Activity::Idle));
        Ok(())
    }
}

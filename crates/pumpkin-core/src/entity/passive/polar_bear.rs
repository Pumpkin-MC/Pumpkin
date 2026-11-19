use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag;
use pumpkin_util::math::position::BlockPos;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::behavior::neutral::apply_targets,
    ai::goal::{
        active_target::TargetCondition, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
        polar_bear_attack_players::PolarBearAttackPlayersGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity,
        neutral::{NeutralData, NeutralMob},
    },
    passive::animal::Animal,
    player::Player,
};

fn is_baby(mob: &dyn Mob) -> bool {
    mob.as_ageable().is_some_and(AgeableMob::is_baby)
}

const POLAR_BEAR_HUNTS: &[Hunt] =
    &[Hunt::new(4, TargetMatch::Type(&EntityType::FOX), true).when(TargetCondition::Adult)];

/// Ticks between two warning growls (`playWarningSound`).
const WARNING_SOUND_COOLDOWN: i32 = 40;
/// The bear rears up once the next attack is this close (`PolarBearMeleeAttackGoal`).
const STAND_UP_TICKS_BEFORE_ATTACK: i32 = 10;

pub struct PolarBearEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub neutral_data: NeutralData,
    pub standing: AtomicBool,
    warning_sound_ticks: AtomicI32,
}

/// Babies panic when hit; adults only on environmental damage like fire.
fn panic_causes(mob: &dyn Mob) -> &'static tag::Tag {
    if is_baby(mob) {
        &tag::DamageType::MINECRAFT_PANIC_CAUSES
    } else {
        &tag::DamageType::MINECRAFT_PANIC_ENVIRONMENTAL_CAUSES
    }
}

fn as_polar_bear(mob: &dyn Mob) -> Option<&PolarBearEntity> {
    mob.cast_any().downcast_ref::<PolarBearEntity>()
}

/// Vanilla `PolarBearMeleeAttackGoal.checkAndPerformAttack`: warns before it hits.
fn check_and_perform_attack(goal: &mut MeleeAttackGoal, mob: &dyn Mob, target: &dyn EntityBase) {
    let Some(bear) = as_polar_bear(mob) else {
        return;
    };
    if goal.can_perform_attack(mob, target) {
        goal.reset_attack_cooldown();
        if mob.do_hurt_target(target) {
            mob.on_attack(target);
        }
        bear.set_standing(false);
        return;
    }

    let reach = target.get_entity().entity_dimension.load().width + 3.0;
    let distance_sq = mob
        .get_entity()
        .pos
        .load()
        .squared_distance_to_vec(&target.get_entity().pos.load());
    if distance_sq < f64::from(reach * reach) {
        if goal.is_time_to_attack() {
            bear.set_standing(false);
            goal.reset_attack_cooldown();
        }
        if goal.cooldown <= STAND_UP_TICKS_BEFORE_ATTACK {
            bear.set_standing(true);
            bear.play_warning_sound();
        }
    } else {
        goal.reset_attack_cooldown();
        bear.set_standing(false);
    }
}

fn stop_standing(mob: &dyn Mob) {
    if let Some(bear) = as_polar_bear(mob) {
        bear.set_standing(false);
    }
}

impl PolarBearEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let polar_bear = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            neutral_data: NeutralData::default(),
            standing: AtomicBool::new(false),
            warning_sound_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(polar_bear);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                1,
                Box::new(
                    MeleeAttackGoal::new(1.25, true)
                        .checking(check_and_perform_attack)
                        .on_stop(stop_standing),
                ),
            );
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0).with_panic_tag(panic_causes));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.25)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));
        };

        {
            let target_selector = &mob_arc.mob_entity.target_selector;

            target_selector.add_goal(
                1,
                Box::new(
                    RevengeGoal::new(true)
                        .raising_alarm_when(is_baby)
                        .alerting_only(|other| !is_baby(other)),
                ),
            );
            target_selector.add_goal(2, PolarBearAttackPlayersGoal::new(&mob_arc.mob_entity));
            apply_targets(target_selector, &mob_arc.mob_entity, 3, 5, false);
            hunting::apply(target_selector, &mob_arc.mob_entity, POLAR_BEAR_HUNTS);
        };

        mob_arc
    }

    #[must_use]
    pub fn is_standing(&self) -> bool {
        self.standing.load(Ordering::Relaxed)
    }

    pub fn set_standing(&self, standing: bool) {
        self.standing.store(standing, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::polar_bear::DATA_STANDING_ID,
            standing,
        );
    }

    fn play_warning_sound(&self) {
        if self.warning_sound_ticks.load(Ordering::Relaxed) <= 0 {
            self.make_sound(Sound::EntityPolarBearWarning);
            self.warning_sound_ticks
                .store(WARNING_SOUND_COOLDOWN, Ordering::Relaxed);
        }
    }
}

impl AgeableMob for PolarBearEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn default_baby_spawn_chance(&self) -> f32 {
        crate::entity::ageable::POLAR_BEAR_BABY_SPAWN_CHANCE
    }
}

impl Animal for PolarBearEntity {
    fn is_food(&self, _item_stack: &ItemStack) -> bool {
        false
    }
}

impl NeutralMob for PolarBearEntity {
    fn get_neutral_data(&self) -> &NeutralData {
        &self.neutral_data
    }
}

impl Mob for PolarBearEntity {
    fn get_ambient_sound(&self) -> Option<Sound> {
        if AgeableMob::is_baby(self) {
            Some(Sound::EntityPolarBearAmbientBaby)
        } else {
            self.get_entity().entity_type.ambient_sound
        }
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        if self.warning_sound_ticks.load(Ordering::Relaxed) > 0 {
            self.warning_sound_ticks.fetch_sub(1, Ordering::Relaxed);
        }
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityPolarBearStep, 0.15, 1.0);
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn as_neutral(&self) -> Option<&dyn NeutralMob> {
        Some(self)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::polar_bear::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::polar_bear::DATA_STANDING_ID,
            self.is_standing(),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use crate::world::World;
    use pumpkin_util::math::vector3::Vector3;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    // Inside the stand-up range of 3 blocks past the target's width, but outside melee reach.
    const WARNING_RANGE: f64 = 4.0;
    const OUT_OF_RANGE: f64 = 8.0;

    fn bear_and_target(
        world: &Arc<World>,
        distance: f64,
    ) -> (Arc<PolarBearEntity>, Arc<PolarBearEntity>) {
        let spawn = |x| {
            PolarBearEntity::new(Entity::new(
                world.clone(),
                Vector3::new(x, 64.0, 0.5),
                &EntityType::POLAR_BEAR,
            ))
        };
        (spawn(0.5), spawn(0.5 + distance))
    }

    #[tokio::test]
    async fn rears_up_and_growls_in_the_last_ten_ticks_before_an_attack() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let (bear, target) = bear_and_target(&world, WARNING_RANGE);
        assert!(!bear.is_in_attack_range(target.as_ref()));
        let mut goal = MeleeAttackGoal::new(1.25, true);

        goal.cooldown = 11;
        check_and_perform_attack(&mut goal, bear.as_ref(), target.as_ref());
        assert!(!bear.is_standing());
        assert_eq!(bear.warning_sound_ticks.load(Ordering::Relaxed), 0);

        goal.cooldown = 10;
        check_and_perform_attack(&mut goal, bear.as_ref(), target.as_ref());
        assert!(bear.is_standing());
        assert_eq!(bear.warning_sound_ticks.load(Ordering::Relaxed), 40);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn growls_again_only_after_the_warning_cooldown() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let (bear, target) = bear_and_target(&world, WARNING_RANGE);
        let mut goal = MeleeAttackGoal::new(1.25, true);

        goal.cooldown = 10;
        check_and_perform_attack(&mut goal, bear.as_ref(), target.as_ref());
        for _ in 0..39 {
            bear.mob_tick(bear.as_ref());
        }
        assert_eq!(bear.warning_sound_ticks.load(Ordering::Relaxed), 1);

        bear.mob_tick(bear.as_ref());
        goal.cooldown = 10;
        check_and_perform_attack(&mut goal, bear.as_ref(), target.as_ref());
        assert_eq!(bear.warning_sound_ticks.load(Ordering::Relaxed), 40);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn stands_down_when_the_target_leaves_the_warning_range() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let (bear, target) = bear_and_target(&world, WARNING_RANGE);
        let mut goal = MeleeAttackGoal::new(1.25, true);

        goal.cooldown = 5;
        check_and_perform_attack(&mut goal, bear.as_ref(), target.as_ref());
        assert!(bear.is_standing());

        target
            .get_entity()
            .set_pos(Vector3::new(0.5 + OUT_OF_RANGE, 64.0, 0.5));
        check_and_perform_attack(&mut goal, bear.as_ref(), target.as_ref());
        assert!(!bear.is_standing());
        assert_eq!(goal.cooldown, goal.get_max_cooldown());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn stands_down_after_hitting_the_target() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let (bear, target) = bear_and_target(&world, 1.5);
        let mut goal = MeleeAttackGoal::new(1.25, true);
        goal.cooldown = 0;
        assert!(goal.can_perform_attack(bear.as_ref(), target.as_ref()));

        bear.set_standing(true);
        check_and_perform_attack(&mut goal, bear.as_ref(), target.as_ref());
        assert!(!bear.is_standing());
        assert_eq!(goal.cooldown, goal.get_max_cooldown());
        world.level.shutdown().await;
        Ok(())
    }
}

use std::sync::{
    Arc, PoisonError, Weak,
    atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_variant::PandaGene;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::{
        behavior::fleeing::{self, Flee},
        goal::{
            avoid_entity::not_spectator, follow_parent::FollowParentGoal,
            look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
            tempt::TemptGoal, wander_around::WanderAroundGoal,
        },
        target_match::TargetMatch,
    },
    mob::{
        Mob, MobEntity, PathAwareEntity,
        spawn::{EntitySpawnReason, SpawnGroupData},
    },
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;
use goals::{PandaBreedGoal, PandaGoal, PandaPanicGoal, attack_goal, hurt_by_target_goal};
use move_control::PandaMoveControl;

mod goals;
mod move_control;

/// Vanilla `PandaAvoidGoal` x2: a worried panda keeps clear of players and monsters.
const PANDA_FLEES: &[Flee] = &[
    Flee::new(6, TargetMatch::Type(&EntityType::PLAYER), 8.0, 2.0, 2.0)
        .when(is_worried_and_can_perform_action)
        .matching(not_spectator),
    Flee::new(6, TargetMatch::Monster, 4.0, 2.0, 2.0)
        .when(is_worried_and_can_perform_action)
        .matching(not_spectator),
];

/// Vanilla `Mob.rotlerp`: moves `from` towards `to` by at most `max` degrees and leaves the
/// result signed. `MoveControl::rotlerp` wraps into 0 to 360 instead, which would turn a pitch
/// of -45 into 315 and `set_pitch` would clamp that to +90.
fn rotlerp(from: f32, to: f32, max: f32) -> f32 {
    from + pumpkin_util::math::wrap_degrees(to - from).clamp(-max, max)
}

pub const FLAG_SNEEZE: u8 = 2;
pub const FLAG_ROLL: u8 = 4;
pub const FLAG_SIT: u8 = 8;
pub const FLAG_ON_BACK: u8 = 16;

/// Speed modifier of vanilla's `PandaAttackGoal`.
const ATTACK_SPEED_MODIFIER: f64 = 1.2;
/// Vanilla `Panda.TOTAL_UNHAPPY_TIME`: ticks a panda sulks when there is no bamboo to breed near.
const TOTAL_UNHAPPY_TIME: i32 = 32;
/// Unhappy counter values at which the panda whines about the missing bamboo.
const UNHAPPY_SOUND_TICKS: [i32; 2] = [29, 14];
/// Vanilla turns the panda to its target by up to this many degrees per axis while it sulks.
const UNHAPPY_LOOK_ANGLE: f32 = 90.0;
/// Vanilla `isThundering` needs the thunder level above this.
const THUNDERING_LEVEL: f32 = 0.9;

const GENE_RANDOM_BOUND: i32 = 16;
const MUTATION_CHANCE: i32 = 32;
const WEAK_MAX_HEALTH: f32 = 10.0;
const LAZY_MOVEMENT_SPEED: f32 = 0.07;

/// A recessive main gene only shows when the hidden gene is the same one.
fn get_variant_from_genes(main_gene: PandaGene, hidden_gene: PandaGene) -> PandaGene {
    if main_gene.recessive() && main_gene != hidden_gene {
        PandaGene::Normal
    } else {
        main_gene
    }
}

/// `next_int(bound)` stands for vanilla's `RandomSource.nextInt`.
fn get_random_gene(mut next_int: impl FnMut(i32) -> i32) -> PandaGene {
    match next_int(GENE_RANDOM_BOUND) {
        0 => PandaGene::Lazy,
        1 => PandaGene::Worried,
        2 => PandaGene::Playful,
        4 => PandaGene::Aggressive,
        3 | 5..=8 => PandaGene::Weak,
        9 | 10 => PandaGene::Brown,
        _ => PandaGene::Normal,
    }
}

fn next_boolean(next_int: &mut impl FnMut(i32) -> i32) -> bool {
    next_int(2) != 0
}

/// Vanilla `getOneOfGenesRandomly` for a panda's (main, hidden) genes.
fn get_one_of_genes_randomly(
    genes: (PandaGene, PandaGene),
    next_int: &mut impl FnMut(i32) -> i32,
) -> PandaGene {
    if next_boolean(next_int) {
        genes.0
    } else {
        genes.1
    }
}

/// Vanilla `setGeneFromParents`: the (main, hidden) genes of a foal of two pandas.
fn genes_from_parents(
    parent1: (PandaGene, PandaGene),
    parent2: (PandaGene, PandaGene),
    mut next_int: impl FnMut(i32) -> i32,
) -> (PandaGene, PandaGene) {
    let (first, second) = if next_boolean(&mut next_int) {
        (parent1, parent2)
    } else {
        (parent2, parent1)
    };
    let mut main_gene = get_one_of_genes_randomly(first, &mut next_int);
    let mut hidden_gene = get_one_of_genes_randomly(second, &mut next_int);
    if next_int(MUTATION_CHANCE) == 0 {
        main_gene = get_random_gene(&mut next_int);
    }
    if next_int(MUTATION_CHANCE) == 0 {
        hidden_gene = get_random_gene(&mut next_int);
    }
    (main_gene, hidden_gene)
}

/// The panda behind a goal's mob.
fn as_panda(mob: &dyn Mob) -> Option<&PandaEntity> {
    mob.cast_any().downcast_ref::<PandaEntity>()
}

/// Whether a worried panda may flee: vanilla `PandaAvoidGoal.canUse` before the avoid check.
fn is_worried_and_can_perform_action(mob: &dyn Mob) -> bool {
    as_panda(mob).is_some_and(|panda| panda.is_worried() && panda.can_perform_action())
}

/// Vanilla `Level.isThundering`, without the check that the dimension has weather.
fn is_thundering(world: &World) -> bool {
    let weather = world.weather.lock().unwrap_or_else(PoisonError::into_inner);
    weather.thunder_level * weather.rain_level > THUNDERING_LEVEL
}

pub struct PandaEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub main_gene: AtomicCell<PandaGene>,
    pub hidden_gene: AtomicCell<PandaGene>,
    pub flags: AtomicU8,
    pub eat_counter: AtomicI32,
    pub sneeze_counter: AtomicI32,
    pub unhappy_counter: AtomicI32,
    got_bamboo: AtomicBool,
    did_bite: AtomicBool,
}

impl PandaEntity {
    /// Vanilla `Panda` constructor: registers the goals and swaps in the panda move control.
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        *mob_entity
            .move_control
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner) = Box::new(PandaMoveControl::default());
        mob_entity.set_can_pick_up_loot(true);
        let panda = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            main_gene: AtomicCell::new(PandaGene::Normal),
            hidden_gene: AtomicCell::new(PandaGene::Normal),
            flags: AtomicU8::new(0),
            eat_counter: AtomicI32::new(0),
            sneeze_counter: AtomicI32::new(0),
            unhappy_counter: AtomicI32::new(0),
            got_bamboo: AtomicBool::new(false),
            did_bite: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(panda);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, Box::new(PandaPanicGoal::new(2.0)));
            goal_selector.add_goal(2, Box::new(PandaBreedGoal::new(1.0)));
            goal_selector.add_goal(3, Box::new(attack_goal(ATTACK_SPEED_MODIFIER)));
            goal_selector.add_goal(
                4,
                Box::new(TemptGoal::with_tag(
                    1.0,
                    &tag::Item::MINECRAFT_PANDA_FOOD,
                    false,
                )),
            );
            fleeing::apply(goal_selector, PANDA_FLEES);
            goal_selector.add_goal(
                9,
                Box::new(PandaGoal::new(
                    *LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
                    Self::can_perform_action,
                )),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));
            goal_selector.add_goal(13, Box::new(FollowParentGoal::new(1.25)));
            goal_selector.add_goal(14, Box::new(WanderAroundGoal::water_avoiding(1.0)));

            mob_arc
                .mob_entity
                .target_selector
                .add_goal(1, Box::new(hurt_by_target_goal()));
        };

        mob_arc
    }

    #[must_use]
    pub fn get_main_gene(&self) -> PandaGene {
        self.main_gene.load()
    }

    pub fn set_main_gene(&self, gene: PandaGene) {
        self.main_gene.store(gene);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::MAIN_GENE_ID,
            gene.id() as i8,
        );
    }

    #[must_use]
    pub fn get_hidden_gene(&self) -> PandaGene {
        self.hidden_gene.load()
    }

    pub fn set_hidden_gene(&self, gene: PandaGene) {
        self.hidden_gene.store(gene);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::HIDDEN_GENE_ID,
            gene.id() as i8,
        );
    }

    #[must_use]
    pub fn get_variant(&self) -> PandaGene {
        get_variant_from_genes(self.get_main_gene(), self.get_hidden_gene())
    }

    #[must_use]
    pub fn is_lazy(&self) -> bool {
        self.get_variant() == PandaGene::Lazy
    }

    #[must_use]
    pub fn is_worried(&self) -> bool {
        self.get_variant() == PandaGene::Worried
    }

    #[must_use]
    pub fn is_playful(&self) -> bool {
        self.get_variant() == PandaGene::Playful
    }

    #[must_use]
    pub fn is_brown(&self) -> bool {
        self.get_variant() == PandaGene::Brown
    }

    #[must_use]
    pub fn is_weak(&self) -> bool {
        self.get_variant() == PandaGene::Weak
    }

    #[must_use]
    pub fn is_aggressive(&self) -> bool {
        self.get_variant() == PandaGene::Aggressive
    }

    pub fn set_attributes(&self) {
        let living = &self.mob_entity.living_entity;
        if self.is_weak() {
            living.set_max_health(WEAK_MAX_HEALTH);
        }
        if self.is_lazy() {
            living.set_attribute_base(&Attributes::MOVEMENT_SPEED, f64::from(LAZY_MOVEMENT_SPEED));
        }
    }

    pub fn set_gene_from_parents(&self, parent1: &Self, parent2: &Self) {
        let mut rng = self.get_random();
        let (main_gene, hidden_gene) = genes_from_parents(
            (parent1.get_main_gene(), parent1.get_hidden_gene()),
            (parent2.get_main_gene(), parent2.get_hidden_gene()),
            |bound| rng.random_range(0..bound),
        );
        self.set_main_gene(main_gene);
        self.set_hidden_gene(hidden_gene);
    }

    #[must_use]
    pub fn has_flag(&self, flag: u8) -> bool {
        (self.flags.load(Ordering::Relaxed) & flag) != 0
    }

    pub fn set_flag(&self, flag: u8, val: bool) {
        let current = self.flags.load(Ordering::Relaxed);
        let new_flags = if val { current | flag } else { current & !flag };
        self.flags.store(new_flags, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::DATA_ID_FLAGS,
            new_flags as i8,
        );
    }

    #[must_use]
    pub fn is_sneezing(&self) -> bool {
        self.has_flag(FLAG_SNEEZE)
    }

    pub fn set_sneezing(&self, val: bool) {
        self.set_flag(FLAG_SNEEZE, val);
    }

    #[must_use]
    pub fn is_sitting(&self) -> bool {
        self.has_flag(FLAG_SIT)
    }

    pub fn set_sitting(&self, val: bool) {
        self.set_flag(FLAG_SIT, val);
    }

    /// Vanilla `Mob.lookAt(entity, yMax, xMax)`: turns the panda's body yaw and pitch towards the
    /// target right away, by at most the given angles per axis. Unlike the look control this does
    /// not wait for the next tick and does not move the head only.
    fn turn_towards(&self, target: &Arc<dyn EntityBase>, max_yaw: f32, max_pitch: f32) {
        let entity = self.get_entity();
        let pos = entity.pos.load();
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();
        let xd = target_pos.x - pos.x;
        let zd = target_pos.z - pos.z;
        let yd = if target.get_living_entity().is_some() {
            target_entity.get_eye_y() - entity.get_eye_y()
        } else {
            let bounding_box = target_entity.bounding_box.load();
            f64::midpoint(bounding_box.min.y, bounding_box.max.y) - entity.get_eye_y()
        };
        let y_rot_d = (zd.atan2(xd).to_degrees() as f32) - 90.0;
        let x_rot_d = -(yd.atan2(xd.hypot(zd)).to_degrees() as f32);
        entity.set_pitch(rotlerp(entity.pitch.load(), x_rot_d, max_pitch));
        entity
            .yaw
            .store(rotlerp(entity.yaw.load(), y_rot_d, max_yaw));
    }

    #[must_use]
    pub fn is_on_back(&self) -> bool {
        self.has_flag(FLAG_ON_BACK)
    }

    pub fn set_on_back(&self, val: bool) {
        self.set_flag(FLAG_ON_BACK, val);
    }

    #[must_use]
    pub fn is_rolling(&self) -> bool {
        self.has_flag(FLAG_ROLL)
    }

    pub fn set_rolling(&self, val: bool) {
        self.set_flag(FLAG_ROLL, val);
    }

    /// Vanilla `getUnhappyCounter`.
    #[must_use]
    pub fn get_unhappy_counter(&self) -> i32 {
        self.unhappy_counter.load(Ordering::Relaxed)
    }

    /// Vanilla `setUnhappyCounter`.
    pub fn set_unhappy_counter(&self, value: i32) {
        self.unhappy_counter.store(value, Ordering::Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::panda::UNHAPPY_COUNTER,
            VarInt(value),
        );
    }

    /// Vanilla `isEating`: the eat counter is running.
    #[must_use]
    pub fn is_eating(&self) -> bool {
        self.eat_counter.load(Ordering::Relaxed) > 0
    }

    /// Vanilla `eat`: starts or stops the eat counter.
    pub fn eat(&self, value: bool) {
        let counter = i32::from(value);
        self.eat_counter.store(counter, Ordering::Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::panda::EAT_COUNTER,
            VarInt(counter),
        );
    }

    /// Vanilla `isScared`: a worried panda during a thunderstorm.
    #[must_use]
    pub fn is_scared(&self) -> bool {
        self.is_worried() && is_thundering(&self.get_entity().world.load())
    }

    /// Vanilla `canPerformAction`: the panda is free to walk, attack, flee and look around.
    #[must_use]
    pub fn can_perform_action(&self) -> bool {
        !self.is_on_back()
            && !self.is_scared()
            && !self.is_eating()
            && !self.is_rolling()
            && !self.is_sitting()
    }
}

impl AgeableMob for PandaEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn default_baby_spawn_chance(&self) -> f32 {
        crate::entity::ageable::PANDA_BABY_SPAWN_CHANCE
    }
}

impl Animal for PandaEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_PANDA_FOOD)
    }

    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = Self::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            &EntityType::PANDA,
        ));
        if let Some(partner) = partner.cast_any().downcast_ref::<Self>() {
            baby.set_gene_from_parents(self, partner);
        }
        baby.set_attributes();
        baby
    }
}

/// Vanilla `PathfinderMob.isPanicking`: the shared panic detection (breeding partners, leash
/// pulling) only knows `EscapeDangerGoal`, so the panda-local panic goal reports itself here.
impl PathAwareEntity for PandaEntity {
    fn is_panicking(&self) -> bool {
        self.mob_entity
            .goals_selector
            .is_running::<PandaPanicGoal>()
    }
}

impl Mob for PandaEntity {
    fn get_path_aware_entity(&self) -> Option<&dyn PathAwareEntity> {
        Some(self)
    }

    fn can_dispenser_equip_into_slot(
        &self,
        slot: &pumpkin_data::data_component_impl::EquipmentSlot,
    ) -> bool {
        *slot == pumpkin_data::data_component_impl::EquipmentSlot::MAIN_HAND
            && self.mob_entity.can_pick_up_loot()
    }

    // Needs pandas eating from their hand first.
    fn wants_to_pick_up(&self, _stack: &ItemStack) -> bool {
        false
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityPandaStep, 0.15, 1.0);
    }

    fn can_be_leashed(&self) -> bool {
        false
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(if self.is_aggressive() {
            Sound::EntityPandaAggressiveAmbient
        } else if self.is_worried() {
            Sound::EntityPandaWorriedAmbient
        } else {
            Sound::EntityPandaAmbient
        })
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("MainGene", self.get_main_gene().name().to_string());
        nbt.put_string("HiddenGene", self.get_hidden_gene().name().to_string());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_main_gene(
            nbt.get_string("MainGene")
                .and_then(PandaGene::from_name)
                .unwrap_or(PandaGene::Normal),
        );
        self.set_hidden_gene(
            nbt.get_string("HiddenGene")
                .and_then(PandaGene::from_name)
                .unwrap_or(PandaGene::Normal),
        );
    }

    fn finalize_spawn(
        &self,
        _world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let mut rng = self.get_random();
        let mut next_int = |bound| rng.random_range(0..bound);
        self.set_main_gene(get_random_gene(&mut next_int));
        self.set_hidden_gene(get_random_gene(&mut next_int));
        self.set_attributes();
        self.finalize_spawn_super(group_data)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::panda::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::MAIN_GENE_ID,
            self.main_gene.load().id() as i8,
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::HIDDEN_GENE_ID,
            self.hidden_gene.load().id() as i8,
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::DATA_ID_FLAGS,
            self.flags.load(Ordering::Relaxed) as i8,
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::UNHAPPY_COUNTER,
            VarInt(self.unhappy_counter.load(Ordering::Relaxed)),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::SNEEZE_COUNTER,
            VarInt(self.sneeze_counter.load(Ordering::Relaxed)),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::panda::EAT_COUNTER,
            VarInt(self.eat_counter.load(Ordering::Relaxed)),
        );
    }

    /// Vanilla `Panda.tick` after the super tick: worried pandas sit through thunderstorms, a lost
    /// target clears the bamboo and bite state, and the unhappy counter counts down.
    fn post_tick(&self) {
        if self.is_worried() {
            if self.is_scared() && !self.get_entity().is_in_water() {
                self.set_sitting(true);
                self.eat(false);
            } else if !self.is_eating() {
                self.set_sitting(false);
            }
        }

        let target = self.mob_entity.get_target();
        if target.is_none() {
            self.got_bamboo.store(false, Ordering::Relaxed);
            self.did_bite.store(false, Ordering::Relaxed);
        }

        let unhappy_counter = self.get_unhappy_counter();
        if unhappy_counter > 0 {
            if let Some(target) = &target {
                self.turn_towards(target, UNHAPPY_LOOK_ANGLE, UNHAPPY_LOOK_ANGLE);
            }
            if UNHAPPY_SOUND_TICKS.contains(&unhappy_counter) {
                self.play_entity_sound(Sound::EntityPandaCantBreed, 1.0, 1.0);
            }
            self.set_unhappy_counter(unhappy_counter - 1);
        }

        if self.is_sitting() {
            self.get_entity().set_pitch(0.0);
        }
    }

    /// Vanilla `Panda.playAttackSound`, which `Mob.doHurtTarget` plays after a hit that hurt.
    fn on_attack(&self, _target: &dyn EntityBase) {
        self.play_entity_sound(Sound::EntityPandaBite, 1.0, 1.0);
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack)
    }
}

#[cfg(test)]
mod tests {
    use super::goals::bite;
    use super::*;
    use crate::entity::ai::goal::Goal;
    use crate::entity::ai::goal::avoid_entity::AvoidEntityGoal;
    use crate::entity::ai::test_support::world;
    use crate::entity::mob::creeper::CreeperEntity;
    use pumpkin_data::Block;
    use pumpkin_data::damage::DamageType;
    use pumpkin_data::item::Item;
    use pumpkin_util::math::vector2::Vector2;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_world::chunk::ChunkData;
    use pumpkin_world::world::BlockFlags;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    /// Tries allowed for the random flee position in the panic test.
    const PANIC_START_ATTEMPTS: usize = 20;

    fn panda(world: &Arc<World>) -> Arc<PandaEntity> {
        PandaEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::PANDA,
        ))
    }

    fn set_genes(panda: &PandaEntity, main_gene: PandaGene, hidden_gene: PandaGene) {
        panda.set_main_gene(main_gene);
        panda.set_hidden_gene(hidden_gene);
    }

    /// Runs `genes_from_parents` against scripted `(bound, result)` random calls.
    fn inherit(
        parent1: (PandaGene, PandaGene),
        parent2: (PandaGene, PandaGene),
        calls: &[(i32, i32)],
    ) -> (PandaGene, PandaGene) {
        let mut script = calls.iter();
        let genes = genes_from_parents(parent1, parent2, |bound| {
            let (expected_bound, result) = script.next().copied().unwrap_or((-1, -1));
            assert_eq!(bound, expected_bound);
            result
        });
        assert!(script.next().is_none(), "unused scripted random calls");
        genes
    }

    #[test]
    fn foal_genes_come_from_the_parents_and_mutate_independently() {
        let parent1 = (PandaGene::Lazy, PandaGene::Worried);
        let parent2 = (PandaGene::Playful, PandaGene::Aggressive);
        // First boolean true: main from parent 1, hidden from parent 2.
        let genes = inherit(
            parent1,
            parent2,
            &[(2, 1), (2, 1), (2, 0), (32, 5), (32, 7)],
        );
        assert_eq!(genes, (PandaGene::Lazy, PandaGene::Aggressive));
        // First boolean false: main from parent 2, hidden from parent 1.
        let genes = inherit(
            parent1,
            parent2,
            &[(2, 0), (2, 0), (2, 1), (32, 5), (32, 7)],
        );
        assert_eq!(genes, (PandaGene::Aggressive, PandaGene::Lazy));
        // A 1/32 mutation hits one gene and not the other.
        let genes = inherit(
            parent1,
            parent2,
            &[(2, 1), (2, 1), (2, 0), (32, 0), (16, 3), (32, 4)],
        );
        assert_eq!(genes, (PandaGene::Weak, PandaGene::Aggressive));
    }

    #[tokio::test]
    async fn weak_variant_halves_health_and_lazy_variant_slows_down() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let default_speed = panda(&world)
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::MOVEMENT_SPEED);

        let weak = panda(&world);
        set_genes(&weak, PandaGene::Weak, PandaGene::Weak);
        weak.set_attributes();
        let living = &weak.mob_entity.living_entity;
        assert!((living.get_max_health() - 10.0).abs() < f32::EPSILON);
        assert!((living.health.load() - 10.0).abs() < f32::EPSILON);
        assert!(
            (living.get_attribute_value(&Attributes::MOVEMENT_SPEED) - default_speed).abs() < 1e-9
        );

        let lazy = panda(&world);
        set_genes(&lazy, PandaGene::Lazy, PandaGene::Normal);
        lazy.set_attributes();
        let living = &lazy.mob_entity.living_entity;
        assert!((living.get_attribute_value(&Attributes::MOVEMENT_SPEED) - 0.07).abs() < 1e-6);
        assert!((living.get_max_health() - 20.0).abs() < f32::EPSILON);

        // A hidden recessive gene does not make the panda weak.
        let carrier = panda(&world);
        set_genes(&carrier, PandaGene::Weak, PandaGene::Normal);
        carrier.set_attributes();
        assert!((carrier.mob_entity.living_entity.get_max_health() - 20.0).abs() < f32::EPSILON);
        Ok(())
    }

    #[tokio::test]
    async fn genes_round_trip_through_nbt_and_default_to_normal() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let source = panda(&world);
        set_genes(&source, PandaGene::Brown, PandaGene::Weak);
        source.set_age(-100);
        let mut nbt = NbtCompound::new();
        source.write_custom_nbt(&mut nbt);
        assert_eq!(nbt.get_string("MainGene"), Some("brown"));
        assert_eq!(nbt.get_string("HiddenGene"), Some("weak"));

        let loaded = panda(&world);
        set_genes(&loaded, PandaGene::Lazy, PandaGene::Lazy);
        loaded.read_custom_nbt(&nbt);
        assert_eq!(loaded.get_main_gene(), PandaGene::Brown);
        assert_eq!(loaded.get_hidden_gene(), PandaGene::Weak);
        assert_eq!(loaded.get_age(), -100);

        set_genes(&loaded, PandaGene::Weak, PandaGene::Brown);
        loaded.read_custom_nbt(&NbtCompound::new());
        assert_eq!(loaded.get_main_gene(), PandaGene::Normal);
        assert_eq!(loaded.get_hidden_gene(), PandaGene::Normal);
        Ok(())
    }

    #[tokio::test]
    async fn offspring_gets_attributes_for_its_genes() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let parent = panda(&world);
        set_genes(&parent, PandaGene::Weak, PandaGene::Weak);
        let mate = panda(&world);
        set_genes(&mate, PandaGene::Weak, PandaGene::Weak);

        let baby = parent.get_breed_offspring(mate.as_ref());
        let foal = baby
            .cast_any()
            .downcast_ref::<PandaEntity>()
            .ok_or("offspring is not a panda")?;
        // A mutation can make the foal normal, so compare against its own variant.
        let expected = if foal.is_weak() { 10.0 } else { 20.0 };
        assert!((foal.mob_entity.living_entity.get_max_health() - expected).abs() < f32::EPSILON);
        Ok(())
    }

    fn stone_floor(world: &Arc<World>) {
        for chunk_x in -2..=1 {
            for chunk_z in -2..=1 {
                world.level.loaded_chunks.insert(
                    Vector2::new(chunk_x, chunk_z),
                    ChunkData::empty_sync(chunk_x, chunk_z),
                );
            }
        }
        let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
        for x in -20..=20 {
            for z in -20..=20 {
                world.set_block_state(
                    &BlockPos::new(x, 63, z),
                    Block::STONE.default_state.id,
                    flags,
                );
            }
        }
    }

    /// Spawns a panda standing on the stone floor.
    fn spawn_panda(world: &Arc<World>, x: f64, z: f64) -> Arc<PandaEntity> {
        let panda = PandaEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, z),
            &EntityType::PANDA,
        ));
        panda.get_entity().on_ground.store(true, Ordering::Relaxed);
        world.spawn_entity(panda.clone());
        panda
    }

    /// Runs `ticks` AI steps and tells whether the avoid goal ran during any of them.
    fn ran_avoid_goal(panda: &Arc<PandaEntity>, ticks: i32) -> bool {
        let mut ran = false;
        for tick in 1..=ticks {
            panda.get_entity().tick_count.store(tick, Ordering::Relaxed);
            panda
                .mob_entity
                .server_ai_step(panda.as_ref(), panda.as_ref());
            ran |= panda
                .mob_entity
                .goals_selector
                .is_running::<AvoidEntityGoal>();
        }
        ran
    }

    fn set_thunder(world: &World, level: f32) {
        let mut weather = world
            .weather
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        weather.rain_level = level;
        weather.thunder_level = level;
    }

    #[tokio::test]
    async fn panda_only_walks_while_it_can_perform_an_action() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = panda(&world);
        let wanted_speed =
            |panda: &Arc<PandaEntity>| f64::from(panda.mob_entity.living_entity.get_speed());
        let tick_move_control = |panda: &Arc<PandaEntity>| {
            panda
                .mob_entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .tick(panda.as_ref());
        };
        panda
            .mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_wanted_position(8.5, 64.0, 0.5, 1.0);

        panda.set_sitting(true);
        tick_move_control(&panda);
        assert!(wanted_speed(&panda).abs() < 1e-9, "a sitting panda walked");
        assert!(
            panda
                .mob_entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .has_wanted(),
            "the wanted position was dropped while sitting"
        );

        panda.set_sitting(false);
        tick_move_control(&panda);
        assert!(
            wanted_speed(&panda) > 0.0,
            "the panda never started walking"
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn worried_panda_sits_through_a_thunderstorm_and_gets_up_after() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let worried = panda(&world);
        set_genes(&worried, PandaGene::Worried, PandaGene::Normal);
        let normal = panda(&world);
        normal.set_sitting(true);

        set_thunder(&world, 1.0);
        worried.post_tick();
        normal.post_tick();
        assert!(worried.is_scared() && worried.is_sitting());
        assert!(!worried.can_perform_action());
        // Only worried pandas react to the thunder, and only they stand up by themselves.
        assert!(!normal.is_scared() && normal.is_sitting());

        // A panda that is eating keeps sitting once the storm is over.
        worried.eat(true);
        set_thunder(&world, 0.0);
        worried.post_tick();
        assert!(worried.is_sitting());
        worried.eat(false);
        worried.post_tick();
        assert!(!worried.is_sitting());
        assert!(worried.can_perform_action());
        normal.post_tick();
        assert!(normal.is_sitting());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn panda_forgets_bamboo_and_bite_when_the_target_is_gone() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = panda(&world);
        let target = self::panda(&world);
        panda.mob_entity.set_target(Some(target));
        panda.got_bamboo.store(true, Ordering::Relaxed);
        panda.did_bite.store(true, Ordering::Relaxed);

        panda.post_tick();
        assert!(panda.got_bamboo.load(Ordering::Relaxed));
        assert!(panda.did_bite.load(Ordering::Relaxed));

        panda.mob_entity.set_target(None);
        panda.post_tick();
        assert!(!panda.got_bamboo.load(Ordering::Relaxed));
        assert!(!panda.did_bite.load(Ordering::Relaxed));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn unhappy_counter_counts_down_to_zero() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = panda(&world);
        panda.set_unhappy_counter(2);
        panda.post_tick();
        assert_eq!(panda.get_unhappy_counter(), 1);
        panda.post_tick();
        assert_eq!(panda.get_unhappy_counter(), 0);
        panda.post_tick();
        assert_eq!(panda.get_unhappy_counter(), 0);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn unhappy_panda_turns_its_body_to_the_target_at_once_by_at_most_90_degrees() -> TestResult
    {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let place = |entity: &Arc<PandaEntity>, x: f64, z: f64| {
            entity.get_entity().pos.store(Vector3::new(x, 64.0, z));
        };
        let unhappy = panda(&world);
        let east = panda(&world);
        let north_east = panda(&world);
        let above = panda(&world);
        place(&unhappy, 0.5, 0.5);
        place(&east, 5.5, 0.5);
        place(&north_east, 5.5, -4.5);
        above.get_entity().pos.store(Vector3::new(5.5, 69.0, 0.5));
        let yaw_off_by = |expected: f32| {
            pumpkin_util::math::wrap_degrees(unhappy.get_entity().yaw.load() - expected).abs()
        };

        // Due east is yaw -90 and within the 90 degree limit, so one tick is enough.
        unhappy.get_entity().yaw.store(0.0);
        unhappy.mob_entity.set_target(Some(east));
        unhappy.set_unhappy_counter(3);
        unhappy.post_tick();
        assert!(yaw_off_by(-90.0) < 0.01, "the panda did not turn at once");

        // North east is yaw -135, which is 45 degrees further than the limit allows from yaw 0.
        unhappy.get_entity().yaw.store(0.0);
        unhappy.mob_entity.set_target(Some(north_east));
        unhappy.post_tick();
        assert!(yaw_off_by(-90.0) < 0.01, "the turn was not capped at 90");

        // A target 5 blocks away and 5 blocks higher is 45 degrees up: a pitch of -45, not 315.
        unhappy.get_entity().yaw.store(0.0);
        unhappy.get_entity().pitch.store(0.0);
        unhappy.mob_entity.set_target(Some(above));
        unhappy.post_tick();
        let pitch = unhappy.get_entity().pitch.load();
        assert!(
            (pitch + 45.0).abs() < 0.01,
            "the pitch towards an elevated target was {pitch}"
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn only_a_worried_panda_that_can_act_flees_a_monster() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let creeper = CreeperEntity::new(Entity::new(
            world.clone(),
            Vector3::new(3.5, 64.0, 0.5),
            &EntityType::CREEPER,
        ));
        creeper
            .get_entity()
            .on_ground
            .store(true, Ordering::Relaxed);
        world.spawn_entity(creeper);

        let calm = spawn_panda(&world, 0.5, 0.5);
        let sitting = spawn_panda(&world, 0.5, 2.5);
        set_genes(&sitting, PandaGene::Worried, PandaGene::Normal);
        sitting.set_sitting(true);
        let worried = spawn_panda(&world, 6.5, 0.5);
        set_genes(&worried, PandaGene::Worried, PandaGene::Normal);

        assert!(!ran_avoid_goal(&calm, 40), "a calm panda fled");
        assert!(!ran_avoid_goal(&sitting, 40), "a sitting panda fled");
        assert!(ran_avoid_goal(&worried, 40), "a worried panda never fled");
        world.level.shutdown().await;
        Ok(())
    }

    fn set_world_age(world: &World, world_age: i64) {
        world
            .level_time
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .world_age = world_age;
    }

    #[tokio::test]
    async fn panda_panics_only_from_environmental_damage_and_stops_when_it_sits() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let hit = spawn_panda(&world, 0.5, 0.5);
        let pricked = spawn_panda(&world, 6.5, 0.5);
        hit.damage(hit.as_ref(), 1.0, DamageType::MOB_ATTACK);
        pricked.damage(pricked.as_ref(), 1.0, DamageType::CACTUS);

        let mut goal = PandaPanicGoal::new(2.0);
        assert!(
            !goal.can_start(hit.as_ref()),
            "a panda hit by a mob panicked"
        );
        // The flee position is random and may not be reachable, so allow a few tries.
        let started = (0..PANIC_START_ATTEMPTS).any(|_| {
            goal.can_start(pricked.as_ref()) && {
                goal.start(pricked.as_ref());
                !pricked.is_navigator_idle()
            }
        });
        assert!(started, "a panda pricked by a cactus did not panic");
        assert!(goal.should_continue(pricked.as_ref()));

        pricked.set_sitting(true);
        assert!(
            !goal.should_continue(pricked.as_ref()),
            "a sitting panda kept panicking"
        );
        assert!(
            pricked.is_navigator_idle(),
            "the panic path was kept while sitting"
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn a_running_panic_goal_counts_as_panicking_for_the_shared_checks() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let calm = spawn_panda(&world, 0.5, 0.5);
        let pricked = spawn_panda(&world, 6.5, 0.5);
        pricked.damage(pricked.as_ref(), 1.0, DamageType::CACTUS);

        let mut panicked = false;
        for tick in 1..=20 {
            for panda in [&calm, &pricked] {
                set_tick(panda, tick);
                panda
                    .mob_entity
                    .server_ai_step(panda.as_ref(), panda.as_ref());
            }
            panicked = pricked
                .mob_entity
                .goals_selector
                .is_running::<PandaPanicGoal>();
            if panicked {
                break;
            }
        }
        assert!(panicked, "a panda pricked by a cactus did not panic");
        assert!(
            EntityBase::is_panicking(pricked.as_ref()),
            "the shared check did not see the panda panic goal"
        );
        assert!(!EntityBase::is_panicking(calm.as_ref()));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn breeding_needs_bamboo_nearby_and_sulks_once_per_cooldown() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let mate = spawn_panda(&world, 1.5, 0.5);
        for in_love in [&panda, &mate] {
            in_love.mob_entity.set_love_ticks(600, None);
        }
        let set_tick = |tick| panda.get_entity().tick_count.store(tick, Ordering::Relaxed);
        let plant_bamboo = |x, y, z| {
            world.set_block_state(
                &BlockPos::new(x, y, z),
                Block::BAMBOO.default_state.id,
                BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE,
            );
        };
        let mut goal = PandaBreedGoal::new(1.0);

        set_tick(100);
        assert!(!goal.can_start(panda.as_ref()), "bred without bamboo");
        assert_eq!(panda.get_unhappy_counter(), TOTAL_UNHAPPY_TIME);

        // The next sulk has to wait for the cooldown, which counts server ticks.
        panda.set_unhappy_counter(0);
        set_tick(699);
        assert!(!goal.can_start(panda.as_ref()));
        assert_eq!(panda.get_unhappy_counter(), 0);
        set_tick(700);
        assert!(!goal.can_start(panda.as_ref()));
        assert_eq!(panda.get_unhappy_counter(), TOTAL_UNHAPPY_TIME);
        panda.set_unhappy_counter(0);
        set_tick(1300);

        // One block too far sideways, and one block too high: no bamboo.
        plant_bamboo(8, 64, 0);
        plant_bamboo(0, 67, 0);
        assert!(!goal.can_start(panda.as_ref()));
        panda.set_unhappy_counter(0);

        plant_bamboo(-7, 66, 7);
        set_tick(2000);
        assert!(goal.can_start(panda.as_ref()), "bamboo was not found");
        panda.set_unhappy_counter(5);
        assert!(
            !goal.can_start(panda.as_ref()),
            "a sulking panda started to breed"
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn attack_starts_only_when_the_panda_can_act_but_goes_on_after_sitting() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        set_world_age(&world, 100);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let victim = spawn_panda(&world, 2.5, 0.5);
        panda.mob_entity.set_target(Some(victim));

        let mut goal = attack_goal(ATTACK_SPEED_MODIFIER);
        panda.set_sitting(true);
        assert!(!goal.can_start(panda.as_ref()), "a sitting panda attacked");
        panda.set_sitting(false);
        assert!(goal.can_start(panda.as_ref()));

        goal.start(panda.as_ref());
        assert!(panda.mob_entity.is_attacking());
        panda.set_sitting(true);
        assert!(
            goal.should_continue(panda.as_ref()),
            "sitting down ended the attack"
        );
        goal.stop(panda.as_ref());
        assert!(!panda.mob_entity.is_attacking());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn only_a_panda_that_is_not_aggressive_remembers_the_bite() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let victim = panda(&world);

        let calm = panda(&world);
        bite(calm.as_ref(), victim.as_ref());
        assert!(calm.did_bite.load(Ordering::Relaxed));

        let aggressive = panda(&world);
        set_genes(&aggressive, PandaGene::Aggressive, PandaGene::Aggressive);
        bite(aggressive.as_ref(), victim.as_ref());
        assert!(!aggressive.did_bite.load(Ordering::Relaxed));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn hurt_panda_hits_back_alerts_aggressive_pandas_and_forgives_bamboo() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let attacker = spawn_panda(&world, 4.5, 0.5);
        let victim = spawn_panda(&world, 0.5, 0.5);
        let guard = spawn_panda(&world, 0.5, 2.5);
        set_genes(&guard, PandaGene::Aggressive, PandaGene::Aggressive);
        let bystander = spawn_panda(&world, 2.5, 2.5);
        // The goal ignores a grudge whose timestamp it has already seen (0 at spawn).
        victim.get_entity().tick_count.store(10, Ordering::Relaxed);
        victim
            .mob_entity
            .living_entity
            .set_last_hurt_by_mob(Some(attacker.as_ref()));

        let mut goal = hurt_by_target_goal();
        assert!(goal.can_start(victim.as_ref()));
        goal.start(victim.as_ref());
        let targets = |panda: &PandaEntity, other: &Arc<PandaEntity>| {
            panda
                .mob_entity
                .get_target()
                .is_some_and(|target| target.get_entity().entity_id == other.get_entity().entity_id)
        };
        assert!(
            targets(&victim, &attacker),
            "the hurt panda did not retaliate"
        );
        assert!(
            targets(&guard, &attacker),
            "the aggressive panda was not alerted"
        );
        assert!(
            bystander.mob_entity.get_target().is_none(),
            "a calm panda was alerted"
        );

        assert!(goal.should_continue(victim.as_ref()));
        victim.got_bamboo.store(true, Ordering::Relaxed);
        assert!(
            !goal.should_continue(victim.as_ref()),
            "bamboo did not calm the panda"
        );
        goal.stop(victim.as_ref());
        assert!(victim.mob_entity.get_target().is_none());
        world.level.shutdown().await;
        Ok(())
    }
}

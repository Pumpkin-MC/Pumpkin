use std::sync::{
    Arc, PoisonError, Weak,
    atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_variant::PandaGene;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::{
        behavior::fleeing::{self, Flee},
        goal::{
            avoid_entity::not_spectator,
            escape_danger::EscapeDangerGoal,
            follow_parent::FollowParentGoal,
            look_around::RandomLookAroundGoal,
            look_at_entity::{LookAtEntityGoal, LookTargetSlot},
            swim::SwimGoal,
            tempt::TemptGoal,
            wander_around::WanderAroundGoal,
        },
        target_match::TargetMatch,
    },
    item::ItemEntity,
    mob::{
        Mob, MobEntity,
        spawn::{EntitySpawnReason, SpawnGroupData},
    },
    passive::animal::Animal,
    player::Player,
};
use crate::plugin::api::events::entity::entity_drop_item::EntityDropItemEvent;
use crate::world::World;
use goals::{PandaBreedGoal, PandaGoal, PandaSitGoal, attack_goal, hurt_by_target_goal};
use lie_on_back::PandaLieOnBackGoal;
use move_control::PandaMoveControl;
use roll::PandaRollGoal;
use sneeze::PandaSneezeGoal;

mod goals;
mod lie_on_back;
mod move_control;
mod roll;
mod sneeze;

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

/// A sitting panda that holds an item starts to eat with a chance of 1 in this many per tick.
const EATING_START_ODDS: i32 = 80;
/// Once the eat counter is above this, each tick has 1 chance in `EATING_STOP_ODDS` to end the meal.
const EATING_MIN_TICKS_BEFORE_STOP: i32 = 80;
const EATING_STOP_ODDS: i32 = 20;
/// A meal that ends with the eat counter above this finishes the held item.
const EATING_MIN_TICKS_TO_FINISH_ITEM: i32 = 100;
/// The eat sound and the client's item particles come every this many ticks of the eat counter.
const EATING_EFFECT_INTERVAL: i32 = 5;

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
    /// Vanilla `rollCounter`: the step of the roll in progress.
    roll_counter: AtomicI32,
    /// Vanilla `rollDelta`: the horizontal push the roll keeps up.
    roll_delta: AtomicCell<Vector3<f64>>,
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
            roll_counter: AtomicI32::new(0),
            roll_delta: AtomicCell::new(Vector3::new(0.0, 0.0, 0.0)),
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
            let look_target = Arc::new(LookTargetSlot::default());

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                2,
                EscapeDangerGoal::new(2.0)
                    .with_panic_tag(|_| &tag::DamageType::MINECRAFT_PANIC_ENVIRONMENTAL_CAUSES)
                    .giving_up_when(|mob| as_panda(mob).is_some_and(Self::is_sitting)),
            );
            goal_selector.add_goal(2, Box::new(PandaBreedGoal::new(1.0, look_target.clone())));
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
            goal_selector.add_goal(7, Box::new(PandaSitGoal::default()));
            goal_selector.add_goal(8, Box::new(PandaSneezeGoal));
            goal_selector.add_goal(8, Box::new(PandaLieOnBackGoal::default()));
            goal_selector.add_goal(
                9,
                Box::new(PandaGoal::new(
                    (*LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0))
                        .with_target_slot(look_target),
                    Self::can_perform_action,
                )),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));
            goal_selector.add_goal(12, Box::new(PandaRollGoal));
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
        self.get_eat_counter() > 0
    }

    /// Vanilla `getEatCounter`.
    #[must_use]
    fn get_eat_counter(&self) -> i32 {
        self.eat_counter.load(Ordering::Relaxed)
    }

    /// Vanilla `setEatCounter`: also synched, the client counts item particles from it.
    fn set_eat_counter(&self, value: i32) {
        self.eat_counter.store(value, Ordering::Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::panda::EAT_COUNTER,
            VarInt(value),
        );
    }

    /// Vanilla `eat`: starts or stops the eat counter.
    pub fn eat(&self, value: bool) {
        self.set_eat_counter(i32::from(value));
    }

    /// Vanilla `getItemBySlot(MAINHAND)`: what the panda holds in its mouth.
    fn main_hand_item(&self) -> ItemStack {
        self.mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND)
    }

    /// Whether the main hand holds anything: vanilla `!getItemBySlot(MAINHAND).isEmpty()`.
    #[must_use]
    pub fn is_holding_item(&self) -> bool {
        !self.mob_entity.is_slot_empty(&EquipmentSlot::MAIN_HAND)
    }

    /// Vanilla `setItemSlot(MAINHAND, stack)`: tells the tracking clients and returns what was held.
    fn set_main_hand_item(&self, stack: ItemStack) -> ItemStack {
        self.mob_entity
            .set_item_slot(&EquipmentSlot::MAIN_HAND, stack)
    }

    /// Vanilla `spawnAtLocation(held)` followed by `setItemSlot(MAINHAND, EMPTY)`. A plugin can
    /// cancel the drop through `EntityDropItemEvent`, and the panda then keeps what it holds.
    /// Returns whether the panda held anything.
    pub fn drop_main_hand_item(&self) -> bool {
        self.drop_main_hand_item_if(|held| self.plugin_allows_drop(held))
    }

    /// `drop_main_hand_item` with the plugin decision passed in: the mouth is only emptied once
    /// the drop is allowed.
    fn drop_main_hand_item_if(&self, allowed: impl FnOnce(&ItemStack) -> bool) -> bool {
        let held = self.main_hand_item();
        if held.is_empty() {
            return false;
        }
        if allowed(&held) {
            let held = self.set_main_hand_item(ItemStack::EMPTY.clone());
            self.get_entity().spawn_at_location(held);
        }
        true
    }

    /// Vanilla `spawnAtLocation(stack)` at the panda's feet, with the plugin hook. Returns whether
    /// an item entity was spawned.
    fn spawn_stack_at_feet(&self, stack: ItemStack) -> bool {
        if stack.is_empty() || !self.plugin_allows_drop(&stack) {
            return false;
        }
        self.get_entity().spawn_at_location(stack);
        true
    }

    /// Fires `EntityDropItemEvent` for a drop at the panda's feet; false when a plugin cancelled it.
    fn plugin_allows_drop(&self, stack: &ItemStack) -> bool {
        let entity = self.get_entity();
        let mut drop_event = EntityDropItemEvent::new(
            entity.entity_id,
            format!("minecraft:{}", stack.item.registry_key),
            stack.item_count,
        );
        if let Some(server) = entity.world.load().server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut drop_event);
        }
        !drop_event.cancelled
    }

    /// Vanilla `tryToSit`: out of the water the panda stops walking and sits down.
    pub fn try_to_sit(&self) {
        if self.get_entity().is_in_water() {
            return;
        }
        let living = &self.mob_entity.living_entity;
        let input = living.movement_input.load();
        living
            .movement_input
            .store(Vector3::new(input.x, input.y, 0.0));
        self.mob_entity.steered().navigator().stop();
        self.set_sitting(true);
    }

    /// Vanilla `handleEating` with `next_int(bound)` standing for `RandomSource.nextInt`: a
    /// sitting panda that holds an item starts to eat now and then, the meal sounds every
    /// `EATING_EFFECT_INTERVAL` ticks, and a long meal ends. The item particles are made by the
    /// client from the synched eat counter and the held item, so none are sent.
    fn handle_eating_with(&self, mut next_int: impl FnMut(i32) -> i32) {
        let holds_item = self.is_holding_item();
        if !self.is_eating()
            && self.is_sitting()
            && !self.is_scared()
            && holds_item
            && next_int(EATING_START_ODDS) == 1
        {
            self.eat(true);
        } else if !holds_item || !self.is_sitting() {
            self.eat(false);
        }
        if !self.is_eating() {
            return;
        }

        let counter = self.get_eat_counter();
        if counter % EATING_EFFECT_INTERVAL == 0 {
            let volume = 0.5 + 0.5 * f32::from(rand::random::<bool>());
            let pitch = (rand::random::<f32>() - rand::random::<f32>()).mul_add(0.2, 1.0);
            self.play_entity_sound(Sound::EntityPandaEat, volume, pitch);
        }
        if counter > EATING_MIN_TICKS_BEFORE_STOP && next_int(EATING_STOP_ODDS) == 1 {
            if counter > EATING_MIN_TICKS_TO_FINISH_ITEM
                && self
                    .main_hand_item()
                    .item
                    .has_tag(&tag::Item::MINECRAFT_PANDA_EATS_FROM_GROUND)
            {
                self.set_main_hand_item(ItemStack::EMPTY.clone());
                let entity = self.get_entity();
                entity.emit_own_game_event(self, GameEvent::Eat);
                self.set_sitting(false);
            }
            self.eat(false);
            return;
        }
        self.set_eat_counter(counter + 1);
    }

    /// Vanilla `handleEating` with the real random source.
    fn handle_eating(&self) {
        let mut rng = self.get_random();
        self.handle_eating_with(|bound| rng.random_range(0..bound));
    }

    /// Vanilla `isScared`: a worried panda during a thunderstorm.
    #[must_use]
    pub fn is_scared(&self) -> bool {
        self.is_worried() && is_thundering(&self.get_entity().world.load())
    }

    /// The last branch of vanilla `Panda.mobInteract` for food: an adult panda that cannot be
    /// bred sits down and eats the item. It drops what it held before (unless the player has
    /// infinite materials, i.e. in creative mode), holds one of the given item and the player's
    /// stack shrinks. Returns whether the interaction was handled.
    ///
    /// A plugin that cancels the drop of the old item leaves everything as it was.
    fn sit_and_eat(&self, item_stack: &mut ItemStack, gamemode: GameMode) -> bool {
        if self.is_baby() || self.is_sitting() || self.get_entity().is_in_water() {
            return false;
        }
        if gamemode != GameMode::Creative && self.drop_main_hand_item() && self.is_holding_item() {
            return false;
        }
        self.try_to_sit();
        self.eat(true);
        self.set_main_hand_item(ItemStack::new(1, item_stack.item));
        item_stack.decrement_unless_creative(gamemode, 1);
        true
    }

    /// Item entities within `range` blocks of the panda's box that it would pick up and eat, as
    /// vanilla `getEntitiesOfClass(ItemEntity.class, box.inflate(range), Panda::canPickUpAndEat)`.
    fn ground_items_to_eat(&self, range: f64) -> Vec<Arc<dyn EntityBase>> {
        let entity = self.get_entity();
        let area = entity.bounding_box.load().expand(range, range, range);
        entity
            .world
            .load()
            .get_entities_at_box(&area)
            .into_iter()
            .filter(|candidate| candidate.get_item_entity().is_some_and(can_pick_up_and_eat))
            .collect()
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

/// Vanilla `Panda.canPickUpAndEat`: an item entity that is still there, can be picked up and
/// holds something from `#minecraft:panda_eats_from_ground`.
fn can_pick_up_and_eat(item: &ItemEntity) -> bool {
    item.get_item_stack()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .item
        .has_tag(&tag::Item::MINECRAFT_PANDA_EATS_FROM_GROUND)
        && item.get_entity().is_alive()
        && item.get_pickup_delay() == 0
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

impl Mob for PandaEntity {
    fn can_dispenser_equip_into_slot(
        &self,
        slot: &pumpkin_data::data_component_impl::EquipmentSlot,
    ) -> bool {
        *slot == pumpkin_data::data_component_impl::EquipmentSlot::MAIN_HAND
            && self.mob_entity.can_pick_up_loot()
    }

    /// Vanilla `Panda.pickUpItem`: a panda with an empty mouth takes a whole stack of an item it
    /// eats from the ground, and the item is dropped for sure when the panda is killed.
    /// `Mob.wantsToPickUp` is not overridden, so the item filter lives here, as in vanilla.
    fn pick_up_item(&self, item: &ItemEntity) {
        if self.is_holding_item() || !can_pick_up_and_eat(item) {
            return;
        }
        let count = item
            .get_item_stack()
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .item_count;
        let living = &self.mob_entity.living_entity;
        if !living.pickup(item.get_entity(), u32::from(count)) {
            return;
        }
        living.on_item_pickup(item);
        let taken = std::mem::replace(
            &mut *item
                .get_item_stack()
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
            ItemStack::EMPTY.clone(),
        );
        self.mob_entity
            .set_item_slot_and_drop_when_killed(&EquipmentSlot::MAIN_HAND, taken);
        item.get_entity().remove();
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
    /// target clears the bamboo and bite state, the unhappy counter counts down, a sitting panda
    /// has pitch 0, `handleEating` runs and a rolling panda takes its next roll step.
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

        self.tick_sneeze();
        self.tick_roll();

        if self.is_sitting() {
            self.get_entity().set_pitch(0.0);
        }

        self.handle_eating();
    }

    /// Vanilla `Panda.playAttackSound`, which `Mob.doHurtTarget` plays after a hit that hurt.
    fn on_attack(&self, _target: &dyn EntityBase) {
        self.play_entity_sound(Sound::EntityPandaBite, 1.0, 1.0);
    }

    /// Vanilla `Panda.mobInteract`: a scared panda ignores the player, a panda on its back gets
    /// up, bamboo for a panda with a target calms it later, an adult that cannot be bred sits
    /// down and eats the food, and the rest is the shared animal feeding.
    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if self.is_scared() {
            return false;
        }
        if self.is_on_back() {
            self.set_on_back(false);
            return true;
        }
        let is_food = self.is_food(item_stack);
        if is_food && self.mob_entity.get_target().is_some() {
            self.got_bamboo.store(true, Ordering::Relaxed);
        }
        let breeds = self.get_age() >= 0
            && self.mob_entity.is_breeding_ready()
            && !self.mob_entity.is_in_love();
        if is_food && !breeds && !self.can_age_up() {
            return self.sit_and_eat(item_stack, player.gamemode.load());
        }
        self.animal_interact(player, item_stack)
    }

    /// Vanilla `Panda.hurtServer`: any damage makes the panda stand up first.
    fn pre_damage(&self, _damage_type: DamageType, _source: Option<&dyn EntityBase>) -> bool {
        self.set_sitting(false);
        true
    }
}

#[cfg(test)]
mod test_support {
    use std::sync::Arc;
    use std::sync::atomic::Ordering::Relaxed;

    use pumpkin_data::Block;
    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector2::Vector2;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_world::chunk::ChunkData;
    use pumpkin_world::world::BlockFlags;

    use super::PandaEntity;
    use crate::entity::{Entity, EntityBase};
    use crate::world::World;

    pub type TestResult = Result<(), Box<dyn std::error::Error>>;

    pub fn stone_floor(world: &Arc<World>) {
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

    pub fn spawn_panda(world: &Arc<World>, x: f64, z: f64) -> Arc<PandaEntity> {
        let panda = PandaEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, z),
            &EntityType::PANDA,
        ));
        panda.get_entity().on_ground.store(true, Relaxed);
        world.spawn_entity(panda.clone());
        panda
    }

    pub fn set_tick(panda: &PandaEntity, tick: i32) {
        panda.get_entity().tick_count.store(tick, Relaxed);
    }

    /// Runs `run` with a random source that answers the `(bound, result)` calls in order and
    /// fails on a different bound, a missing call or an extra one.
    pub fn scripted<T>(
        calls: &[(i32, i32)],
        run: impl FnOnce(&mut dyn FnMut(i32) -> i32) -> T,
    ) -> T {
        let mut script = calls.iter();
        let mut next_int = |bound: i32| {
            let (expected_bound, result) = script.next().copied().unwrap_or((-1, -1));
            assert_eq!(bound, expected_bound);
            result
        };
        let result = run(&mut next_int);
        assert!(script.next().is_none(), "unused scripted random calls");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{TestResult, scripted, set_tick, spawn_panda, stone_floor};
    use super::*;
    use crate::entity::ai::goal::Goal;
    use crate::entity::ai::test_support::world;
    use pumpkin_data::Block;
    use pumpkin_data::item::Item;
    use pumpkin_world::world::BlockFlags;

    /// Goal ticks the panic test gives the random flee position to be reachable.
    const PANIC_START_ATTEMPTS: i32 = 20;

    fn hold(panda: &PandaEntity, item: &'static Item) {
        panda.set_main_hand_item(ItemStack::new(1, item));
    }

    fn dropped_items(world: &Arc<World>, panda: &PandaEntity) -> usize {
        let area = panda.get_entity().bounding_box.load().expand(3.0, 3.0, 3.0);
        world
            .get_entities_at_box(&area)
            .iter()
            .filter(|entity| entity.get_item_entity().is_some())
            .count()
    }

    #[test]
    fn foal_genes_come_from_the_parents_and_mutate_independently() {
        let parent1 = (PandaGene::Lazy, PandaGene::Worried);
        let parent2 = (PandaGene::Playful, PandaGene::Aggressive);
        let inherit = |calls: &[(i32, i32)]| {
            scripted(calls, |next| genes_from_parents(parent1, parent2, next))
        };
        // The first boolean picks the parent of the main gene, then each parent gives one gene.
        assert_eq!(
            inherit(&[(2, 1), (2, 1), (2, 0), (32, 5), (32, 7)]),
            (PandaGene::Lazy, PandaGene::Aggressive)
        );
        assert_eq!(
            inherit(&[(2, 0), (2, 0), (2, 1), (32, 5), (32, 7)]),
            (PandaGene::Aggressive, PandaGene::Lazy)
        );
        // A 1 in 32 mutation hits one gene and not the other.
        assert_eq!(
            inherit(&[(2, 1), (2, 1), (2, 0), (32, 0), (16, 3), (32, 4)]),
            (PandaGene::Weak, PandaGene::Aggressive)
        );
    }

    #[tokio::test]
    async fn offspring_gets_attributes_for_its_genes() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let parent = spawn_panda(&world, 0.5, 0.5);
        let mate = spawn_panda(&world, 1.5, 0.5);
        for panda in [&parent, &mate] {
            panda.set_main_gene(PandaGene::Weak);
            panda.set_hidden_gene(PandaGene::Weak);
        }

        let baby = parent.get_breed_offspring(mate.as_ref());
        let foal = baby
            .cast_any()
            .downcast_ref::<PandaEntity>()
            .ok_or("offspring is not a panda")?;
        // A mutation can make the foal normal, so compare against its own variant.
        let expected = if foal.is_weak() { 10.0 } else { 20.0 };
        assert!((foal.mob_entity.living_entity.get_max_health() - expected).abs() < f32::EPSILON);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn genes_round_trip_through_nbt_and_default_to_normal() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let source = spawn_panda(&world, 0.5, 0.5);
        source.set_main_gene(PandaGene::Brown);
        source.set_hidden_gene(PandaGene::Weak);
        source.set_age(-100);
        let mut nbt = NbtCompound::new();
        source.write_custom_nbt(&mut nbt);

        let loaded = spawn_panda(&world, 0.5, 0.5);
        loaded.read_custom_nbt(&nbt);
        assert_eq!(loaded.get_main_gene(), PandaGene::Brown);
        assert_eq!(loaded.get_hidden_gene(), PandaGene::Weak);
        assert_eq!(loaded.get_age(), -100);

        loaded.read_custom_nbt(&NbtCompound::new());
        assert_eq!(loaded.get_main_gene(), PandaGene::Normal);
        assert_eq!(loaded.get_hidden_gene(), PandaGene::Normal);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn unhappy_panda_turns_to_its_target_by_at_most_90_degrees_with_a_signed_pitch()
    -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let unhappy = spawn_panda(&world, 0.5, 0.5);
        let north_east = spawn_panda(&world, 5.5, -4.5);
        let above = spawn_panda(&world, 5.5, 0.5);
        above.get_entity().pos.store(Vector3::new(5.5, 69.0, 0.5));
        unhappy.set_unhappy_counter(3);

        // North east is yaw -135, which is 45 degrees further than one tick may turn from 0.
        unhappy.mob_entity.set_target(Some(north_east));
        unhappy.post_tick();
        assert!((unhappy.get_entity().yaw.load() + 90.0).abs() < 0.01);

        // Five blocks away and five higher is 45 degrees up: a pitch of -45, not 315.
        unhappy.get_entity().yaw.store(0.0);
        unhappy.mob_entity.set_target(Some(above));
        unhappy.post_tick();
        assert!((unhappy.get_entity().pitch.load() + 45.0).abs() < 0.01);
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
        let plant_bamboo = |x, y, z| {
            world.set_block_state(
                &BlockPos::new(x, y, z),
                Block::BAMBOO.default_state.id,
                BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE,
            );
        };
        let mut goal = PandaBreedGoal::new(1.0, Arc::default());

        set_tick(&panda, 100);
        assert!(!goal.can_start(panda.as_ref()), "bred without bamboo");
        assert_eq!(panda.get_unhappy_counter(), TOTAL_UNHAPPY_TIME);

        panda.set_unhappy_counter(0);
        set_tick(&panda, 699);
        assert!(!goal.can_start(panda.as_ref()));
        assert_eq!(panda.get_unhappy_counter(), 0, "sulked again too early");
        set_tick(&panda, 700);
        assert!(!goal.can_start(panda.as_ref()));
        assert_eq!(panda.get_unhappy_counter(), TOTAL_UNHAPPY_TIME);
        panda.set_unhappy_counter(0);
        set_tick(&panda, 1300);

        // One block too far sideways, and one block too high: no bamboo.
        plant_bamboo(8, 64, 0);
        plant_bamboo(0, 67, 0);
        assert!(!goal.can_start(panda.as_ref()));
        panda.set_unhappy_counter(0);

        plant_bamboo(-7, 66, 7);
        set_tick(&panda, 2000);
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
    async fn the_sulk_points_the_look_goal_only_with_effective_ai() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let mate = spawn_panda(&world, 1.5, 0.5);
        for in_love in [&panda, &mate] {
            in_love.mob_entity.set_love_ticks(600, None);
        }
        let mob: Arc<dyn Mob> = panda.clone();
        let slot = Arc::new(LookTargetSlot::default());
        let mut sulk = PandaBreedGoal::new(1.0, slot.clone());
        let mut look =
            LookAtEntityGoal::new(Arc::downgrade(&mob), &EntityType::PLAYER, 6.0, 1.0, false)
                .with_target_slot(slot.clone());

        // The look goal starts from the preset target; only the sulk can replace it with the
        // nearest player (there is none here).
        slot.set_target(Some(mate.clone()));
        panda.mob_entity.set_no_ai(true);
        set_tick(&panda, 100);
        assert!(!sulk.can_start(panda.as_ref()));
        assert!(
            look.can_start(panda.as_ref()),
            "a panda without AI changed its look target"
        );
        look.stop(panda.as_ref());

        panda.mob_entity.set_no_ai(false);
        panda.set_unhappy_counter(0);
        set_tick(&panda, 1000);
        slot.set_target(Some(mate));
        assert!(!sulk.can_start(panda.as_ref()));
        assert!(
            !look.can_start(panda.as_ref()),
            "the sulk did not set the look target"
        );
        world.level.shutdown().await;
        Ok(())
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

        let step = |tick: i32| {
            for panda in [&hit, &pricked] {
                set_tick(panda, tick);
                panda
                    .mob_entity
                    .server_ai_step(panda.as_ref(), panda.as_ref());
            }
        };
        let panicked = (1..=PANIC_START_ATTEMPTS).any(|tick| {
            step(tick);
            pricked
                .mob_entity
                .goals_selector
                .is_running::<EscapeDangerGoal>()
        });
        assert!(panicked, "a panda pricked by a cactus did not panic");
        assert!(
            EntityBase::is_panicking(pricked.as_ref()),
            "the shared check did not see the panda panic"
        );
        assert!(
            !EntityBase::is_panicking(hit.as_ref()),
            "a panda hit by a mob panicked"
        );

        // The cactus is forgotten after 40 ticks, so the goal cannot start again once it gives up.
        world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .world_age += 41;
        pricked.set_sitting(true);
        for tick in 1..=4 {
            step(PANIC_START_ATTEMPTS + tick);
        }
        assert!(!EntityBase::is_panicking(pricked.as_ref()));
        assert!(
            pricked.is_navigator_idle(),
            "the panic path was kept while sitting"
        );
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
        guard.set_main_gene(PandaGene::Aggressive);
        guard.set_hidden_gene(PandaGene::Aggressive);
        let bystander = spawn_panda(&world, 2.5, 2.5);
        // The goal ignores a grudge whose timestamp it has already seen (0 at spawn).
        set_tick(&victim, 10);
        victim
            .mob_entity
            .living_entity
            .set_last_hurt_by_mob(Some(attacker.as_ref()));

        let mut goal = hurt_by_target_goal();
        assert!(goal.can_start(victim.as_ref()));
        goal.start(victim.as_ref());
        let targets_attacker = |panda: &PandaEntity| {
            panda.mob_entity.get_target().is_some_and(|target| {
                target.get_entity().entity_id == attacker.get_entity().entity_id
            })
        };
        assert!(
            targets_attacker(&victim),
            "the hurt panda did not retaliate"
        );
        assert!(
            targets_attacker(&guard),
            "the aggressive panda was not alerted"
        );
        assert!(bystander.mob_entity.get_target().is_none());

        assert!(goal.should_continue(victim.as_ref()));
        victim.got_bamboo.store(true, Ordering::Relaxed);
        assert!(
            !goal.should_continue(victim.as_ref()),
            "bamboo did not calm the panda"
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn a_cancelled_drop_keeps_the_held_item_and_an_allowed_one_drops_it() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);

        let mut asked = false;
        assert!(!panda.drop_main_hand_item_if(|_| {
            asked = true;
            true
        }));
        assert!(!asked, "an empty mouth asked a plugin");

        hold(&panda, &Item::BAMBOO);
        assert!(panda.drop_main_hand_item_if(|_| false));
        assert_eq!(panda.main_hand_item().item.id, Item::BAMBOO.id);
        assert_eq!(dropped_items(&world, &panda), 0);

        assert!(panda.drop_main_hand_item_if(|held| held.item.id == Item::BAMBOO.id));
        assert!(panda.main_hand_item().is_empty());
        assert_eq!(dropped_items(&world, &panda), 1);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn a_panda_with_an_item_sits_and_eats_through_the_real_tick() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        hold(&panda, &Item::BAMBOO);
        let steps = |from: i32, to: i32| {
            for tick in from..to {
                set_tick(&panda, tick);
                panda
                    .mob_entity
                    .server_ai_step(panda.as_ref(), panda.as_ref());
            }
        };

        steps(1, 11);
        assert!(panda.mob_entity.goals_selector.is_running::<PandaSitGoal>());
        assert!(panda.is_sitting(), "the panda did not sit down");

        panda.eat(true);
        panda.post_tick();
        assert_eq!(panda.get_eat_counter(), 2);

        // A hit stands the panda up at once, and the sit goal that is still running sits it
        // down again with its item.
        panda.damage(panda.as_ref(), 1.0, DamageType::MOB_ATTACK);
        assert!(!panda.is_sitting(), "the panda kept sitting when hurt");
        steps(11, 15);
        assert!(panda.is_sitting());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn eating_and_sitting_roll_their_chances_in_vanilla_order() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = spawn_panda(&world, 0.5, 0.5);
        hold(&panda, &Item::BAMBOO);
        let eat = |calls: &[(i32, i32)]| scripted(calls, |next| panda.handle_eating_with(next));

        eat(&[]);
        assert!(!panda.is_eating(), "a standing panda ate");
        panda.set_sitting(true);
        eat(&[(80, 0)]);
        assert!(!panda.is_eating());
        eat(&[(80, 1)]);
        // `eat(true)` sets 1 and the same tick counts on.
        assert_eq!(panda.get_eat_counter(), 2);

        // The end of a meal is rolled only above 80 ticks, the item is eaten up only above 100.
        panda.set_eat_counter(80);
        eat(&[]);
        assert_eq!(panda.get_eat_counter(), 81);
        eat(&[(20, 0)]);
        assert_eq!(panda.get_eat_counter(), 82);
        panda.set_eat_counter(100);
        eat(&[(20, 1)]);
        assert!(!panda.is_eating() && panda.is_holding_item() && panda.is_sitting());
        panda.set_eat_counter(101);
        eat(&[(20, 1)]);
        assert!(!panda.is_holding_item() && !panda.is_sitting());
        // Only `#panda_eats_from_ground` items are finished.
        hold(&panda, &Item::STICK);
        panda.set_sitting(true);
        panda.set_eat_counter(101);
        eat(&[(20, 1)]);
        assert!(panda.is_holding_item() && panda.is_sitting());

        // Bounds are halved for goal ticks (600 and 2000), and a lazy panda skips the first roll.
        let keeps = |calls: &[(i32, i32)]| {
            scripted(calls, |next| PandaSitGoal::keeps_sitting(&panda, next))
        };
        assert!(keeps(&[(300, 0), (1000, 0)]));
        assert!(!keeps(&[(300, 1)]));
        assert!(!keeps(&[(300, 0), (1000, 1)]));
        panda.set_main_gene(PandaGene::Lazy);
        assert!(keeps(&[(1000, 0)]));
        assert!(!keeps(&[(1000, 1)]));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn feeding_a_panda_that_cannot_breed_swaps_its_item_for_the_food() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let mut bamboo = ItemStack::new(3, &Item::BAMBOO);

        hold(&panda, &Item::STICK);
        assert!(panda.sit_and_eat(&mut bamboo, GameMode::Survival));
        assert!(panda.is_sitting());
        assert_eq!(panda.get_eat_counter(), 1);
        assert_eq!(panda.main_hand_item().item.id, Item::BAMBOO.id);
        assert_eq!(bamboo.item_count, 2);
        assert_eq!(dropped_items(&world, &panda), 1, "the old item was kept");
        assert!(!panda.sit_and_eat(&mut bamboo, GameMode::Survival));
        assert_eq!(bamboo.item_count, 2, "a sitting panda took more food");

        // With infinite materials the stack stays and the old item is not dropped.
        let creative = spawn_panda(&world, 8.5, 8.5);
        hold(&creative, &Item::STICK);
        assert!(creative.sit_and_eat(&mut bamboo, GameMode::Creative));
        assert_eq!(bamboo.item_count, 2);
        assert_eq!(dropped_items(&world, &creative), 0);

        let baby = spawn_panda(&world, -8.5, 8.5);
        baby.set_age(-1200);
        assert!(!baby.sit_and_eat(&mut bamboo, GameMode::Survival));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn a_panda_takes_the_whole_stack_it_eats_from_the_ground_and_nothing_else() -> TestResult
    {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let drop_item = |stack: ItemStack| {
            let item = Arc::new(ItemEntity::new_with_velocity(
                Entity::new(
                    world.clone(),
                    Vector3::new(0.5, 64.0, 0.5),
                    &EntityType::ITEM,
                ),
                stack,
                Vector3::default(),
                0,
            ));
            world.add_entity_silent(item.clone());
            item
        };
        let pick_up = || crate::entity::mob::pick_up_nearby_items(panda.as_ref());

        let stick = drop_item(ItemStack::new(1, &Item::STICK));
        let bamboo = drop_item(ItemStack::new(5, &Item::BAMBOO));
        bamboo.set_pickup_delay(ItemEntity::DEFAULT_PICKUP_DELAY);
        pick_up();
        assert!(
            !panda.is_holding_item(),
            "took a stick or an item in its delay"
        );

        bamboo.set_pickup_delay(0);
        pick_up();
        let held = panda.main_hand_item();
        assert_eq!((held.item.id, held.item_count), (Item::BAMBOO.id, 5));
        assert!(bamboo.get_entity().is_removed());
        assert!(!stick.get_entity().is_removed());
        assert!(
            panda.mob_entity.drop_chance(&EquipmentSlot::MAIN_HAND)
                >= MobEntity::GUARANTEED_DROP_CHANCE,
            "the picked up item is not a guaranteed drop"
        );

        let more = drop_item(ItemStack::new(2, &Item::BAMBOO));
        pick_up();
        assert!(!more.get_entity().is_removed(), "a full mouth took more");
        world.level.shutdown().await;
        Ok(())
    }
}

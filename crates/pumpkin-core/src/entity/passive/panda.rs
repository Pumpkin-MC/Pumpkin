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
            avoid_entity::not_spectator, follow_parent::FollowParentGoal,
            look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
            tempt::TemptGoal, wander_around::WanderAroundGoal,
        },
        target_match::TargetMatch,
    },
    item::ItemEntity,
    mob::{
        Mob, MobEntity, PathAwareEntity,
        spawn::{EntitySpawnReason, SpawnGroupData},
    },
    passive::animal::Animal,
    player::Player,
};
use crate::plugin::api::events::entity::entity_drop_item::EntityDropItemEvent;
use crate::world::World;
use goals::{
    PandaBreedGoal, PandaGoal, PandaPanicGoal, PandaSitGoal, attack_goal, hurt_by_target_goal,
};
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
            goal_selector.add_goal(7, Box::new(PandaSitGoal::default()));
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

    /// The start of vanilla `Panda.mobInteract`: a scared panda ignores the player (`Some(false)`),
    /// a panda on its back gets up (`Some(true)`), and feeding bamboo to a panda that has a target
    /// makes it forget the grudge later. `None` leaves the interaction to the feeding code.
    fn interact_before_feeding(&self, item_stack: &ItemStack) -> Option<bool> {
        if self.is_scared() {
            return Some(false);
        }
        if self.is_on_back() {
            self.set_on_back(false);
            return Some(true);
        }
        if self.is_food(item_stack) && self.mob_entity.get_target().is_some() {
            self.got_bamboo.store(true, Ordering::Relaxed);
        }
        None
    }

    /// Whether feeding takes the shared animal path of vanilla `Panda.mobInteract`: an adult that
    /// can fall in love, or a baby that can still grow up. Anything else sits down and eats.
    fn feeding_is_breeding_or_growing(&self) -> bool {
        let breeding = self.get_age() >= 0
            && self.mob_entity.is_breeding_ready()
            && !self.mob_entity.is_in_love();
        breeding || self.can_age_up()
    }

    /// The last branch of vanilla `Panda.mobInteract` for food: an adult panda that cannot be
    /// bred sits down and eats the item. It drops what it held before (unless the player has
    /// infinite materials, i.e. in creative mode), holds one of the given item and the player's
    /// stack shrinks. Returns whether the interaction was handled.
    fn sit_and_eat(&self, item_stack: &mut ItemStack, gamemode: GameMode) -> bool {
        self.sit_and_eat_if(item_stack, gamemode, |held| self.plugin_allows_drop(held))
    }

    /// `sit_and_eat` with the plugin decision passed in. The old item is only swapped out once its
    /// drop is allowed: if a plugin cancels that drop nothing changes (the panda keeps its item and
    /// the player's stack stays) and the interaction is not handled.
    fn sit_and_eat_if(
        &self,
        item_stack: &mut ItemStack,
        gamemode: GameMode,
        allowed: impl FnOnce(&ItemStack) -> bool,
    ) -> bool {
        if self.is_baby() || self.is_sitting() || self.get_entity().is_in_water() {
            return false;
        }
        let held = self.main_hand_item();
        let drops_held = gamemode != GameMode::Creative && !held.is_empty();
        if drops_held && !allowed(&held) {
            return false;
        }
        self.try_to_sit();
        self.eat(true);
        let held = self.set_main_hand_item(ItemStack::new(1, item_stack.item));
        if drops_held {
            self.get_entity().spawn_at_location(held);
        }
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
    /// has pitch 0 and `handleEating` runs.
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

        self.handle_eating();
    }

    /// Vanilla `Panda.playAttackSound`, which `Mob.doHurtTarget` plays after a hit that hurt.
    fn on_attack(&self, _target: &dyn EntityBase) {
        self.play_entity_sound(Sound::EntityPandaBite, 1.0, 1.0);
    }

    /// Vanilla `Panda.mobInteract`: the checks before feeding come from
    /// `interact_before_feeding`, an adult that cannot be bred sits down and eats the food
    /// (`sit_and_eat`), the rest is the shared animal feeding.
    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if let Some(handled) = self.interact_before_feeding(item_stack) {
            return handled;
        }
        if self.is_food(item_stack) && !self.feeding_is_breeding_or_growing() {
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
mod tests {
    use super::goals::bite;
    use super::*;
    use crate::entity::ai::goal::Goal;
    use crate::entity::ai::goal::avoid_entity::AvoidEntityGoal;
    use crate::entity::ai::pathfinder::NavigatorGoal;
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

    #[tokio::test]
    async fn scared_panda_ignores_players_and_a_panda_on_its_back_gets_up() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = panda(&world);
        let bamboo = ItemStack::new(1, &Item::BAMBOO);
        let target = self::panda(&world);

        set_genes(&panda, PandaGene::Worried, PandaGene::Normal);
        set_thunder(&world, 1.0);
        panda.set_on_back(true);
        assert_eq!(panda.interact_before_feeding(&bamboo), Some(false));
        assert!(panda.is_on_back(), "a scared panda got up for a player");

        set_thunder(&world, 0.0);
        assert_eq!(panda.interact_before_feeding(&bamboo), Some(true));
        assert!(!panda.is_on_back());

        // Bamboo only calms a panda that has something to be angry at.
        assert_eq!(panda.interact_before_feeding(&bamboo), None);
        assert!(!panda.got_bamboo.load(Ordering::Relaxed));
        panda.mob_entity.set_target(Some(target));
        assert_eq!(
            panda.interact_before_feeding(&ItemStack::new(1, &Item::CAKE)),
            None
        );
        assert!(!panda.got_bamboo.load(Ordering::Relaxed));
        assert_eq!(panda.interact_before_feeding(&bamboo), None);
        assert!(panda.got_bamboo.load(Ordering::Relaxed));
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
        assert!(
            !panda.drop_main_hand_item_if(|_| {
                asked = true;
                true
            }),
            "an empty mouth dropped something"
        );
        assert!(!asked, "an empty mouth asked a plugin");
        hold(&panda, &Item::BAMBOO);
        assert!(panda.drop_main_hand_item_if(|_| false));
        assert_eq!(
            held_item(&panda).id,
            Item::BAMBOO.id,
            "a cancelled drop emptied the mouth"
        );
        assert_eq!(dropped_items(&world, &panda), 0);

        assert!(panda.drop_main_hand_item_if(|held| held.item.id == Item::BAMBOO.id));
        assert!(panda.main_hand_item().is_empty());
        assert_eq!(dropped_items(&world, &panda), 1);
        world.level.shutdown().await;
        Ok(())
    }

    /// Puts one of `item` into the panda's mouth.
    fn hold(panda: &PandaEntity, item: &'static Item) {
        panda.set_main_hand_item(ItemStack::new(1, item));
    }

    fn held_item(panda: &PandaEntity) -> &'static Item {
        panda.main_hand_item().item
    }

    fn set_tick(panda: &PandaEntity, tick: i32) {
        panda.get_entity().tick_count.store(tick, Ordering::Relaxed);
    }

    /// Item entities next to the panda.
    fn dropped_items(world: &Arc<World>, panda: &PandaEntity) -> usize {
        let area = panda.get_entity().bounding_box.load().expand(3.0, 3.0, 3.0);
        world
            .get_entities_at_box(&area)
            .iter()
            .filter(|entity| *entity.get_entity().entity_type == EntityType::ITEM)
            .count()
    }

    #[tokio::test]
    async fn sit_goal_starts_only_for_an_adult_that_holds_an_item_and_can_act() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = panda(&world);
        let mut goal = PandaSitGoal::default();
        let check = |goal: &mut PandaSitGoal| goal.can_start(panda.as_ref());

        assert!(!check(&mut goal), "a panda with an empty mouth sat");
        hold(&panda, &Item::BAMBOO);
        assert!(check(&mut goal));

        panda.set_unhappy_counter(3);
        assert!(!check(&mut goal), "a sulking panda sat");
        panda.set_unhappy_counter(0);
        panda.eat(true);
        assert!(!check(&mut goal), "an eating panda sat");
        panda.eat(false);
        panda
            .get_entity()
            .touching_water
            .store(true, Ordering::Relaxed);
        assert!(!check(&mut goal), "a panda in the water sat");
        panda
            .get_entity()
            .touching_water
            .store(false, Ordering::Relaxed);

        panda.set_age(-100);
        assert!(!check(&mut goal), "a baby sat");
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn sit_goal_start_sits_down_and_drops_the_route() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let living = &panda.mob_entity.living_entity;
        panda
            .mob_entity
            .steered()
            .navigator()
            .set_progress(NavigatorGoal::new(
                Vector3::new(0.5, 64.0, 0.5),
                Vector3::new(5.5, 64.0, 0.5),
                1.0,
            ));
        living.movement_input.store(Vector3::new(0.0, 0.0, 1.0));
        assert!(!panda.is_navigator_idle());

        hold(&panda, &Item::BAMBOO);
        let mut goal = PandaSitGoal::default();
        assert!(goal.can_start(panda.as_ref()));
        goal.start(panda.as_ref());
        assert!(panda.is_sitting());
        assert!(
            panda.is_navigator_idle(),
            "the route was kept while sitting"
        );
        assert!(
            living.movement_input.load().z.abs() < f64::EPSILON,
            "the panda kept its forward input"
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn sit_goal_stop_drops_the_item_and_waits_before_the_next_sit() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let mut goal = PandaSitGoal::default();
        hold(&panda, &Item::BAMBOO);
        panda.set_sitting(true);
        set_tick(&panda, 100);

        // The wait is `nextInt(150) + 10` seconds: 15 seconds here.
        goal.leave_sit(&panda, |bound| {
            assert_eq!(bound, 150);
            5
        });
        assert!(!panda.is_sitting() && !panda.is_holding_item());
        assert_eq!(dropped_items(&world, &panda), 1, "the item was not dropped");
        hold(&panda, &Item::BAMBOO);
        set_tick(&panda, 399);
        assert!(!goal.can_start(panda.as_ref()), "the panda sat too early");
        set_tick(&panda, 400);
        assert!(goal.can_start(panda.as_ref()), "the panda waited too long");

        // A lazy panda draws from 50 seconds: at most 59 seconds, so tick 1280 at the latest.
        let lazy = self::panda(&world);
        set_genes(&lazy, PandaGene::Lazy, PandaGene::Normal);
        hold(&lazy, &Item::BAMBOO);
        lazy.set_sitting(true);
        set_tick(&lazy, 100);
        goal.leave_sit(&lazy, |bound| {
            assert_eq!(bound, 50);
            bound - 1
        });
        hold(&lazy, &Item::BAMBOO);
        set_tick(&lazy, 1279);
        assert!(!goal.can_start(lazy.as_ref()));
        set_tick(&lazy, 1280);
        assert!(goal.can_start(lazy.as_ref()));

        // Standing up without an item draws no wait and sets no cooldown.
        let empty = self::panda(&world);
        let mut fresh_goal = PandaSitGoal::default();
        empty.set_sitting(true);
        let mut drew_a_wait = false;
        fresh_goal.leave_sit(&empty, |_| {
            drew_a_wait = true;
            0
        });
        assert!(!drew_a_wait, "a wait was drawn without an item");
        assert!(!empty.is_sitting());
        hold(&empty, &Item::BAMBOO);
        assert!(fresh_goal.can_start(empty.as_ref()));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn sit_goal_ends_by_chance_and_in_the_water() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = panda(&world);
        let keeps = |panda: &PandaEntity, rolls: &[i32]| {
            let mut asked = Vec::new();
            let mut rolls = rolls.iter();
            let kept = PandaSitGoal::keeps_sitting(panda, |bound| {
                asked.push(bound);
                rolls.next().copied().unwrap_or(0)
            });
            (kept, asked)
        };

        // Goal tick counts are half of the server ticks: 600 -> 300 and 2000 -> 1000.
        assert_eq!(keeps(&panda, &[0, 0]), (true, vec![300, 1000]));
        assert_eq!(keeps(&panda, &[1]), (false, vec![300]));
        assert_eq!(keeps(&panda, &[0, 1]), (false, vec![300, 1000]));

        set_genes(&panda, PandaGene::Lazy, PandaGene::Normal);
        assert_eq!(keeps(&panda, &[0]), (true, vec![1000]));
        assert_eq!(keeps(&panda, &[1]), (false, vec![1000]));

        panda
            .get_entity()
            .touching_water
            .store(true, Ordering::Relaxed);
        assert_eq!(keeps(&panda, &[]), (false, vec![]));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn registered_sit_goal_sits_a_panda_that_holds_an_item_until_it_is_hurt() -> TestResult {
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
    async fn eating_starts_and_stops_with_the_sit_and_the_item() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = panda(&world);
        let always = |bound: i32| {
            assert_eq!(bound, 80);
            1
        };

        hold(&panda, &Item::BAMBOO);
        panda.handle_eating_with(always);
        assert!(!panda.is_eating(), "a standing panda ate");

        panda.set_sitting(true);
        panda.handle_eating_with(|_| 0);
        assert!(!panda.is_eating(), "eating needs a roll of 1 in 80");
        panda.handle_eating_with(always);
        // `eat(true)` sets 1 and the same tick counts on.
        assert_eq!(panda.get_eat_counter(), 2);
        panda.handle_eating_with(|_| 0);
        assert_eq!(panda.get_eat_counter(), 3);

        panda.set_sitting(false);
        panda.handle_eating_with(|_| 0);
        assert!(!panda.is_eating(), "a standing panda kept eating");

        panda.set_sitting(true);
        panda.eat(true);
        panda.set_main_hand_item(ItemStack::EMPTY.clone());
        panda.handle_eating_with(|_| 0);
        assert!(!panda.is_eating(), "the panda ate without an item");

        let scared = self::panda(&world);
        set_genes(&scared, PandaGene::Worried, PandaGene::Normal);
        set_thunder(&world, 1.0);
        hold(&scared, &Item::BAMBOO);
        scared.set_sitting(true);
        scared.handle_eating_with(|_| 1);
        assert!(!scared.is_eating(), "a scared panda started to eat");
        set_thunder(&world, 0.0);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn a_meal_ends_after_80_ticks_and_finishes_the_item_after_100() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = panda(&world);
        let meal = |panda: &PandaEntity, counter: i32, item: &'static Item, roll: i32| {
            hold(panda, item);
            panda.set_sitting(true);
            panda.set_eat_counter(counter);
            panda.handle_eating_with(|bound| {
                assert_eq!(bound, 20, "only the end of the meal is rolled");
                roll
            });
        };

        hold(&panda, &Item::BAMBOO);
        panda.set_sitting(true);
        panda.set_eat_counter(80);
        panda.handle_eating_with(|bound| {
            assert_eq!(bound, 80, "the end of a meal is rolled only above 80 ticks");
            0
        });
        assert_eq!(panda.get_eat_counter(), 81);

        meal(&panda, 81, &Item::BAMBOO, 0);
        assert_eq!(panda.get_eat_counter(), 82);
        meal(&panda, 100, &Item::BAMBOO, 1);
        assert!(!panda.is_eating());
        assert_eq!(held_item(&panda), &Item::BAMBOO);
        assert!(panda.is_sitting(), "a short meal stood the panda up");

        meal(&panda, 101, &Item::BAMBOO, 1);
        assert!(!panda.is_eating());
        assert!(!panda.is_holding_item(), "the food was not eaten");
        assert!(!panda.is_sitting(), "the panda did not get up after eating");

        // Only `#panda_eats_from_ground` items are eaten up.
        meal(&panda, 101, &Item::STICK, 1);
        assert!(!panda.is_eating());
        assert_eq!(held_item(&panda), &Item::STICK);
        assert!(panda.is_sitting());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn panda_tick_runs_the_meal_and_synchs_the_eat_counter() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = panda(&world);
        hold(&panda, &Item::BAMBOO);
        panda.set_sitting(true);
        panda.eat(true);
        let synched = &panda.get_entity().synched_data;
        synched.clear_dirty();

        panda.post_tick();
        assert_eq!(panda.get_eat_counter(), 2);
        assert!(synched.is_dirty(), "the counter step was not synched");
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn feeding_a_panda_that_cannot_breed_makes_it_sit_down_and_eat() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let mut bamboo = ItemStack::new(3, &Item::BAMBOO);

        // A panda that can fall in love takes the shared animal feeding, an adult in love eats.
        assert!(panda.feeding_is_breeding_or_growing());
        panda.mob_entity.set_love_ticks(600, None);
        assert!(!panda.feeding_is_breeding_or_growing());

        panda
            .get_entity()
            .touching_water
            .store(true, Ordering::Relaxed);
        assert!(!panda.sit_and_eat(&mut bamboo, GameMode::Survival));
        assert!(!panda.is_sitting() && !panda.is_holding_item());
        panda
            .get_entity()
            .touching_water
            .store(false, Ordering::Relaxed);

        hold(&panda, &Item::STICK);
        assert!(panda.sit_and_eat(&mut bamboo, GameMode::Survival));
        assert!(panda.is_sitting());
        assert_eq!(panda.get_eat_counter(), 1);
        assert_eq!(held_item(&panda), &Item::BAMBOO);
        assert_eq!(panda.main_hand_item().item_count, 1);
        assert_eq!(bamboo.item_count, 2);
        assert_eq!(dropped_items(&world, &panda), 1, "the old item was kept");
        assert!(!panda.sit_and_eat(&mut bamboo, GameMode::Survival));
        assert_eq!(bamboo.item_count, 2, "a sitting panda took more food");

        // With infinite materials the stack stays and the old item is not dropped.
        let creative = self::panda(&world);
        hold(&creative, &Item::STICK);
        assert!(creative.sit_and_eat(&mut bamboo, GameMode::Creative));
        assert_eq!(held_item(&creative), &Item::BAMBOO);
        assert_eq!(bamboo.item_count, 2);
        assert_eq!(dropped_items(&world, &creative), 1);

        // A baby grows up on food, and one that cannot grow up does not eat either.
        let baby = self::panda(&world);
        baby.set_age(-1200);
        assert!(baby.feeding_is_breeding_or_growing());
        baby.set_age_locked(true);
        assert!(!baby.feeding_is_breeding_or_growing());
        assert!(!baby.sit_and_eat(&mut bamboo, GameMode::Survival));
        assert!(!baby.is_sitting());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn a_cancelled_drop_stops_the_panda_from_swapping_its_item_for_food() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        hold(&panda, &Item::STICK);
        let mut bamboo = ItemStack::new(3, &Item::BAMBOO);

        assert!(!panda.sit_and_eat_if(&mut bamboo, GameMode::Survival, |_| false));
        assert!(!panda.is_sitting(), "the panda sat down");
        assert_eq!(held_item(&panda), &Item::STICK, "the old item was swapped");
        assert_eq!(bamboo.item_count, 3, "the player's stack shrank");
        assert_eq!(dropped_items(&world, &panda), 0);

        assert!(
            panda.sit_and_eat_if(&mut bamboo, GameMode::Survival, |held| {
                held.item.id == Item::STICK.id
            })
        );
        assert_eq!(held_item(&panda), &Item::BAMBOO);
        assert_eq!(bamboo.item_count, 2);
        assert_eq!(dropped_items(&world, &panda), 1);

        // Nothing is dropped with infinite materials, so no plugin is asked either.
        let creative = spawn_panda(&world, 8.5, 8.5);
        hold(&creative, &Item::STICK);
        let mut asked = false;
        assert!(
            creative.sit_and_eat_if(&mut bamboo, GameMode::Creative, |_| {
                asked = true;
                true
            })
        );
        assert!(!asked, "a creative player's panda asked a plugin");
        assert_eq!(held_item(&creative), &Item::BAMBOO);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn item_in_the_mouth_survives_nbt_and_makes_the_panda_sit() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let source = panda(&world);
        hold(&source, &Item::BAMBOO);
        let mut nbt = NbtCompound::new();
        source.write_nbt(&mut nbt);

        let loaded = panda(&world);
        loaded.read_nbt_non_mut(&nbt);
        assert_eq!(held_item(&loaded), &Item::BAMBOO);
        assert!(PandaSitGoal::default().can_start(loaded.as_ref()));
        world.level.shutdown().await;
        Ok(())
    }

    /// Puts an item entity on the floor at `x`, `z` whose pickup delay is over.
    fn drop_item(world: &Arc<World>, stack: ItemStack, x: f64, z: f64) -> Arc<ItemEntity> {
        let item = Arc::new(ItemEntity::new_with_velocity(
            Entity::new(world.clone(), Vector3::new(x, 64.0, z), &EntityType::ITEM),
            stack,
            Vector3::default(),
            0,
        ));
        world.add_entity_silent(item.clone());
        item
    }

    fn on_ground(item: &ItemEntity) -> u8 {
        item.get_item_stack()
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .item_count
    }

    fn set_mob_griefing(world: &World, value: bool) {
        world.level_info.rcu(|current| {
            let mut info = (**current).clone();
            info.game_rules.mob_griefing = value;
            info
        });
    }

    #[tokio::test]
    async fn panda_takes_the_whole_stack_of_an_item_it_eats_from_the_ground() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let bamboo = drop_item(&world, ItemStack::new(5, &Item::BAMBOO), 0.5, 0.5);

        crate::entity::mob::pick_up_nearby_items(panda.as_ref());

        let held = panda.main_hand_item();
        assert_eq!(held.item, &Item::BAMBOO);
        assert_eq!(held.item_count, 5, "the panda did not take the whole stack");
        assert_eq!(on_ground(&bamboo), 0);
        assert!(bamboo.get_entity().is_removed());
        assert!(
            panda.mob_entity.drop_chance(&EquipmentSlot::MAIN_HAND)
                >= MobEntity::GUARANTEED_DROP_CHANCE,
            "the picked up item is not a guaranteed drop"
        );

        // Cake is in the tag as well, and the sit goal takes over once the mouth is full.
        let cake = self::panda(&world);
        cake.get_entity().set_pos(Vector3::new(9.5, 64.0, 9.5));
        let item = drop_item(&world, ItemStack::new(1, &Item::CAKE), 9.5, 9.5);
        crate::entity::mob::pick_up_nearby_items(cake.as_ref());
        assert_eq!(held_item(&cake), &Item::CAKE);
        assert!(item.get_entity().is_removed());
        assert!(PandaSitGoal::default().can_start(cake.as_ref()));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn panda_leaves_items_it_cannot_eat_from_the_ground_or_cannot_take_yet() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);

        // Not in `#panda_eats_from_ground`, although `Mob.wantsToPickUp` is true for anything.
        let panda = spawn_panda(&world, 0.5, 0.5);
        let stick = drop_item(&world, ItemStack::new(1, &Item::STICK), 0.5, 0.5);
        assert!(panda.wants_to_pick_up(&ItemStack::new(1, &Item::STICK)));
        crate::entity::mob::pick_up_nearby_items(panda.as_ref());
        assert!(!panda.is_holding_item());
        assert_eq!(on_ground(&stick), 1);

        // Still in its pickup delay.
        let bamboo = drop_item(&world, ItemStack::new(1, &Item::BAMBOO), 0.5, 0.5);
        bamboo.set_pickup_delay(ItemEntity::DEFAULT_PICKUP_DELAY);
        crate::entity::mob::pick_up_nearby_items(panda.as_ref());
        assert!(!panda.is_holding_item());
        assert!(!bamboo.get_entity().is_removed());
        bamboo.set_pickup_delay(0);

        // The `mobGriefing` rule.
        set_mob_griefing(&world, false);
        crate::entity::mob::pick_up_nearby_items(panda.as_ref());
        assert!(!panda.is_holding_item(), "picked up with mobGriefing off");
        set_mob_griefing(&world, true);

        // `canPickUpLoot` is a flag of its own (a dispenser or NBT can clear it).
        panda.mob_entity.set_can_pick_up_loot(false);
        crate::entity::mob::pick_up_nearby_items(panda.as_ref());
        assert!(!panda.is_holding_item(), "picked up without canPickUpLoot");
        panda.mob_entity.set_can_pick_up_loot(true);

        // Out of reach: the pickup box grows by one block sideways only.
        let far = drop_item(&world, ItemStack::new(1, &Item::BAMBOO), 4.5, 0.5);
        far.set_pickup_delay(0);
        bamboo.get_entity().remove();
        crate::entity::mob::pick_up_nearby_items(panda.as_ref());
        assert!(!panda.is_holding_item());
        assert!(!far.get_entity().is_removed());

        // A full mouth takes nothing more.
        let other = drop_item(&world, ItemStack::new(2, &Item::BAMBOO), 0.5, 0.5);
        hold(&panda, &Item::CAKE);
        crate::entity::mob::pick_up_nearby_items(panda.as_ref());
        assert_eq!(held_item(&panda), &Item::CAKE);
        assert_eq!(on_ground(&other), 2);
        assert!(!other.get_entity().is_removed());

        // Everything works again once the panda drops what it held.
        assert!(panda.drop_main_hand_item());
        crate::entity::mob::pick_up_nearby_items(panda.as_ref());
        assert!(panda.is_holding_item());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn sit_goal_looks_for_food_on_the_ground_and_walks_to_it() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let mut goal = PandaSitGoal::default();

        // Nothing to eat: not even a stick (not in the tag) nor an item that cannot be taken yet.
        drop_item(&world, ItemStack::new(1, &Item::STICK), 3.5, 0.5);
        let delayed = drop_item(&world, ItemStack::new(1, &Item::BAMBOO), 3.5, 0.5);
        delayed.set_pickup_delay(ItemEntity::DEFAULT_PICKUP_DELAY);
        assert!(!goal.can_start(panda.as_ref()));
        goal.start(panda.as_ref());
        assert!(
            panda.is_navigator_idle(),
            "walked to something it cannot eat"
        );
        assert!(!panda.is_sitting());

        // Food beyond 6 blocks does not start the goal, but the start still walks to it (8 blocks).
        delayed.set_pickup_delay(0);
        delayed.get_entity().set_pos(Vector3::new(7.5, 64.0, 0.5));
        assert!(!goal.can_start(panda.as_ref()));
        goal.start(panda.as_ref());
        assert!(!panda.is_navigator_idle());
        assert!(!panda.is_sitting(), "sat down with an empty mouth");
        let speed = panda.mob_entity.steered().navigator().get_speed_modifier();
        assert!((speed - 1.2).abs() < 1e-9, "walking speed was {speed}");

        // Within 6 blocks it starts, and a sulking panda or one that cannot act does not.
        delayed.get_entity().set_pos(Vector3::new(5.5, 64.0, 0.5));
        assert!(goal.can_start(panda.as_ref()));
        panda.set_unhappy_counter(3);
        assert!(!goal.can_start(panda.as_ref()));
        panda.set_unhappy_counter(0);
        panda.set_on_back(true);
        assert!(!goal.can_start(panda.as_ref()));
        world.level.shutdown().await;
        Ok(())
    }
}

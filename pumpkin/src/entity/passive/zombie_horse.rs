use std::sync::{Arc, Weak, atomic::Ordering::Relaxed};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use rand::RngExt;

use crate::entity::{
    Entity, EntityBaseFuture, NBTStorage,
    ai::goal::{
        ambient_stand::AmbientStandGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::{
        animal::Animal,
        equine::{AbstractHorse, AbstractHorseData},
    },
    player::Player,
};

/// `ZombieHorse.java`'s `isFood` tag (`ItemTags.ZOMBIE_HORSE_FOOD`) is red mushroom only;
/// `TemptGoal` only supports a static item list (not a tag/predicate), so this is narrowed the
/// same way `HappyGhastEntity`'s tempt items are.
const TEMPT_ITEMS: &[&Item] = &[&Item::RED_MUSHROOM];

/// Represents a Zombie Horse.
///
/// A passive mob that can be tamed and ridden but never carries a chest and never breeds
/// attribute-inherited offspring (`ZombieHorse.getBreedOffspring` spawns a plain default
/// `ZombieHorse`, unlike Horse/Donkey/Mule).
///
/// Zombie-jockey natural-spawn (`ZombieHorse.java:132-146`) is not ported: Pumpkin has no
/// natural-spawn "finalize" hook to inject a rider mob yet (no chicken/spider jockey precedent
/// exists either), so this is a known gap rather than folded into this framework change.
///
/// Wiki: <https://minecraft.wiki/w/Zombie_Horse>
pub struct ZombieHorseEntity {
    pub mob_entity: MobEntity,
    pub horse_data: AbstractHorseData,
}

impl ZombieHorseEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let horse = Self {
            mob_entity,
            horse_data: AbstractHorseData::default(),
        };
        let mob_arc = Arc::new(horse);
        AbstractHorse::randomize_attributes(mob_arc.as_ref(), &mut rand::rng());

        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        let horse_weak: Weak<Self> = Arc::downgrade(&mob_arc);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // `ZombieHorse.java:126-129` (`addBehaviourGoals`): float + tempt only, no panic
            // goal. Wander/look/stand come from the base `AbstractHorse.registerGoals`.
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS, false)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.0)));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new_water_avoiding(0.7)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
            goal_selector.add_goal(9, AmbientStandGoal::new(horse_weak));
        };

        mob_arc
    }
}

impl NBTStorage for ZombieHorseEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_horse_nbt(nbt);
        })
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_horse_nbt(nbt);
        })
    }
}

impl Animal for ZombieHorseEntity {
    /// `ZombieHorse.isFood`: the distinct `ZOMBIE_HORSE_FOOD` tag (red mushroom only), not the
    /// generic `HORSE_FOOD` tag Horse/Donkey/Mule use.
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_ZOMBIE_HORSE_FOOD)
    }

    /// `ZombieHorse.canAgeUp` -- babies never grow up from food.
    fn can_age_up(&self) -> bool {
        false
    }
}

impl AbstractHorse for ZombieHorseEntity {
    fn horse_data(&self) -> &AbstractHorseData {
        &self.horse_data
    }

    fn angry_sound(&self) -> Option<Sound> {
        Some(Sound::EntityZombieHorseAngry)
    }

    fn eating_sound(&self) -> Option<Sound> {
        Some(Sound::EntityZombieHorseEat)
    }

    /// `ZombieHorse.canFallInLove` -- always false.
    fn can_fall_in_love(&self) -> bool {
        false
    }

    /// `ZombieHorse.randomizeAttributes`: only jump-strength and speed, using
    /// `generateZombieHorseJumpStrength`/`generateZombieHorseSpeed` (different constants from
    /// the base `AbstractHorse` formula) -- max-health stays the fixed 25 from `createAttributes`.
    fn randomize_attributes(&self, random: &mut impl RngExt)
    where
        Self: Sized,
    {
        let mut attrs = self.mob_entity.living_entity.attributes.write().unwrap();
        if let Some(a) = attrs.get_mut(&Attributes::JUMP_STRENGTH.id) {
            a.base_value =
                crate::entity::passive::equine::generate_zombie_horse_jump_strength(random);
            a.dirty.store(true, Relaxed);
        }
        if let Some(a) = attrs.get_mut(&Attributes::MOVEMENT_SPEED.id) {
            a.base_value = crate::entity::passive::equine::generate_zombie_horse_speed(random);
            a.dirty.store(true, Relaxed);
        }
    }
}

impl Mob for ZombieHorseEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        self.abstract_horse_mob_interact(player, item_stack)
    }
}

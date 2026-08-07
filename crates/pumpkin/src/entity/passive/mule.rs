use std::sync::{Arc, Weak, atomic::Ordering::Relaxed};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        ambient_stand::AmbientStandGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        run_around_like_crazy::RunAroundLikeCrazyGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::{
        animal::Animal,
        equine::{
            AbstractChestedHorse, AbstractHorse, AbstractHorseData, ChestedHorseData,
            HORSE_TEMPT_ITEMS,
        },
    },
    player::Player,
};

/// Represents a Mule, a passive mob created by breeding a horse and a donkey.
///
/// Mule.java has no `BreedGoal`/`canMate` override, so it inherits `AbstractHorse.canMate`,
/// which unconditionally returns `false` -- mules never breed in vanilla despite
/// `getBreedOffspring` existing (that method is only reachable via a goal that's never
/// registered for this species). No `HorseBreedGoal` is added here to match.
///
/// Wiki: <https://minecraft.wiki/w/Mule>
pub struct MuleEntity {
    pub mob_entity: MobEntity,
    pub horse_data: AbstractHorseData,
    pub chested_data: ChestedHorseData,
}

impl MuleEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mule = Self {
            mob_entity,
            horse_data: AbstractHorseData::default(),
            chested_data: ChestedHorseData::default(),
        };
        let mob_arc = Arc::new(mule);
        AbstractHorse::randomize_attributes(mob_arc.as_ref(), &mut rand::rng());

        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let horse_weak: Weak<Self> = Arc::downgrade(&mob_arc);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // See `horse.rs` for the priority citations (`AbstractHorse.java:134-151`); Mule
            // has no `addBehaviourGoals` override, so it also inherits the base tempt goal.
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, RunAroundLikeCrazyGoal::new(horse_weak.clone(), 1.2));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, HORSE_TEMPT_ITEMS, false)));
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

impl NBTStorage for MuleEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_animal_nbt(nbt);
            self.write_horse_nbt(nbt);
            self.write_chested_horse_nbt(nbt);
        })
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_animal_nbt(nbt);
            self.read_horse_nbt(nbt);
            self.read_chested_horse_nbt(nbt).await;
        })
    }
}

impl Animal for MuleEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_HORSE_FOOD)
    }
}

impl AbstractHorse for MuleEntity {
    fn horse_data(&self) -> &AbstractHorseData {
        &self.horse_data
    }

    fn angry_sound(&self) -> Option<Sound> {
        Some(Sound::EntityMuleAngry)
    }

    fn eating_sound(&self) -> Option<Sound> {
        Some(Sound::EntityMuleEat)
    }

    /// `AbstractChestedHorse.randomizeAttributes`: only max-health is rolled.
    fn randomize_attributes(&self, random: &mut impl RngExt)
    where
        Self: Sized,
    {
        let mut attrs = self.mob_entity.living_entity.attributes.write().unwrap();
        if let Some(a) = attrs.get_mut(&Attributes::MAX_HEALTH.id) {
            a.base_value = crate::entity::passive::equine::generate_max_health(random);
            a.dirty.store(true, Relaxed);
        }
        drop(attrs);
        let max_health = self.mob_entity.living_entity.get_max_health();
        self.mob_entity.living_entity.health.store(max_health);
    }
}

impl AbstractChestedHorse for MuleEntity {
    fn chested_data(&self) -> &ChestedHorseData {
        &self.chested_data
    }

    /// `Mule.playChestEquipsSound`: overrides the `AbstractChestedHorse` default (donkey chest
    /// sound) with `MULE_CHEST`.
    fn play_chest_equips_sound(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        world.play_sound(
            Sound::EntityMuleChest,
            SoundCategory::Neutral,
            &entity.pos.load(),
        );
    }
}

impl Mob for MuleEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        self.chested_mob_interact(player, item_stack)
    }
}

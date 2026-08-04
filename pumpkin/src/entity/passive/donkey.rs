use std::sync::{Arc, Weak, atomic::Ordering::Relaxed};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use rand::RngExt;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        horse_breed::HorseBreedGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::{
        animal::Animal,
        equine::{
            AbstractChestedHorse, AbstractHorse, AbstractHorseData, ChestedHorseData,
            apply_offspring_attribute,
        },
    },
    player::Player,
};

/// Donkey.java#canMate: a Donkey may breed with another Donkey or a Horse.
const COMPATIBLE_MATES: &[&EntityType] = &[&EntityType::DONKEY, &EntityType::HORSE];

const MIN_HEALTH: f64 = 15.0;
const MAX_HEALTH: f64 = 30.0;

/// Represents a Donkey, a passive mob that can be tamed and equipped with a chest.
///
/// Wiki: <https://minecraft.wiki/w/Donkey>
pub struct DonkeyEntity {
    pub mob_entity: MobEntity,
    pub horse_data: AbstractHorseData,
    pub chested_data: ChestedHorseData,
}

impl DonkeyEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let donkey = Self {
            mob_entity,
            horse_data: AbstractHorseData::default(),
            chested_data: ChestedHorseData::default(),
        };
        let mob_arc = Arc::new(donkey);
        AbstractHorse::randomize_attributes(mob_arc.as_ref(), &mut rand::rng());

        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(WanderAroundGoal::new(0.7)));
            goal_selector.add_goal(2, HorseBreedGoal::new(1.0, COMPATIBLE_MATES));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(4, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for DonkeyEntity {
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

impl Animal for DonkeyEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_HORSE_FOOD)
    }
}

impl AbstractHorse for DonkeyEntity {
    fn horse_data(&self) -> &AbstractHorseData {
        &self.horse_data
    }

    fn angry_sound(&self) -> Option<Sound> {
        Some(Sound::EntityDonkeyAngry)
    }

    fn eating_sound(&self) -> Option<Sound> {
        Some(Sound::EntityDonkeyEat)
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

impl AbstractChestedHorse for DonkeyEntity {
    fn chested_data(&self) -> &ChestedHorseData {
        &self.chested_data
    }

    fn play_chest_equips_sound(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        world.play_sound(
            Sound::EntityDonkeyChest,
            SoundCategory::Neutral,
            &entity.pos.load(),
        );
    }
}

impl Mob for DonkeyEntity {
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

    /// `Donkey.getBreedOffspring`: Donkey+Horse -> Mule, Donkey+Donkey -> Donkey (both handled
    /// generically by `HorseBreedGoal`/`horse_family_offspring`) plus max-health inheritance.
    fn create_offspring<'a>(
        &'a self,
        mate: &'a dyn EntityBase,
        world: &'a Arc<crate::world::World>,
    ) -> EntityBaseFuture<'a, Option<Arc<dyn EntityBase>>> {
        Box::pin(async move {
            let entity = self.get_entity();
            let baby = crate::entity::r#type::from_type(
                entity.entity_type,
                entity.pos.load(),
                world,
                Uuid::new_v4(),
            );

            let mate_max_health = mate.get_mob().map_or(MIN_HEALTH, |m| {
                m.get_mob_entity()
                    .living_entity
                    .get_attribute_base(&Attributes::MAX_HEALTH)
            });

            if let Some(baby_mob) = baby.get_mob() {
                let mut random = rand::rng();
                apply_offspring_attribute(
                    baby_mob,
                    &Attributes::MAX_HEALTH,
                    self.mob_entity
                        .living_entity
                        .get_attribute_base(&Attributes::MAX_HEALTH),
                    mate_max_health,
                    MIN_HEALTH,
                    MAX_HEALTH,
                    &mut random,
                );
            }

            Some(baby)
        })
    }
}

use std::sync::{Arc, Weak, atomic::Ordering::Relaxed};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        ambient_stand::AmbientStandGoal, follow_parent::FollowParentGoal,
        horse_breed::HorseBreedGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::{
        animal::Animal,
        equine::{AbstractHorse, AbstractHorseData, HORSE_TEMPT_ITEMS, apply_offspring_attribute},
    },
    player::Player,
};

/// Horse.java#canMate: a Horse may breed with another Horse or a Donkey.
const COMPATIBLE_MATES: &[&EntityType] = &[&EntityType::HORSE, &EntityType::DONKEY];

/// Horse.java: 7 coat colors, packed into the low byte of `DATA_ID_TYPE_VARIANT`.
const VARIANT_COUNT: u8 = 7;
/// Horse.java: 5 marking patterns, packed into the high byte of `DATA_ID_TYPE_VARIANT`.
const MARKINGS_COUNT: u8 = 5;

/// `AbstractHorse.MIN_HEALTH`/`MAX_HEALTH` used by breed-offspring attribute inheritance.
const MIN_HEALTH: f64 = 15.0;
const MAX_HEALTH: f64 = 30.0;

/// Represents a Horse, a passive mob that can be tamed, saddled, ridden and bred.
///
/// Wiki: <https://minecraft.wiki/w/Horse>
pub struct HorseEntity {
    pub mob_entity: MobEntity,
    pub horse_data: AbstractHorseData,
    /// `Horse.DATA_ID_TYPE_VARIANT`: low byte = `Variant` (0-6), high byte = `Markings` (0-4).
    /// Numeric ids only -- vanilla source has no display-name table for these, see the module
    /// doc comment on `equine::mod` for why that's fine to skip.
    variant_and_markings: std::sync::atomic::AtomicU16,
}

impl HorseEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mut random = rand::rng();
        let horse = Self {
            mob_entity,
            horse_data: AbstractHorseData::default(),
            variant_and_markings: std::sync::atomic::AtomicU16::new(
                u16::from(random.random_range(0..VARIANT_COUNT))
                    | (u16::from(random.random_range(0..MARKINGS_COUNT)) << 8),
            ),
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

            // `AbstractHorse.java:134-151` (`registerGoals`/`addBehaviourGoals`, base
            // `addBehaviourGoals` applies to Horse/Donkey/Mule): 0 float, (1 panic-when-ridden
            // deferred, no rider-tracking infra), 2 breed, 3 tempt, 4 follow parent, 6
            // water-avoiding wander, 7 look at player, 8 random look around, 9 random stand.
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, HorseBreedGoal::new(1.0, COMPATIBLE_MATES));
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

    fn variant(&self) -> u8 {
        (self.variant_and_markings.load(Relaxed) & 0xFF) as u8
    }

    fn markings(&self) -> u8 {
        ((self.variant_and_markings.load(Relaxed) >> 8) & 0xFF) as u8
    }

    fn set_variant_and_markings(&self, variant: u8, markings: u8) {
        self.variant_and_markings
            .store(u16::from(variant) | (u16::from(markings) << 8), Relaxed);
    }

    fn sync_type_variant(&self) {
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::ID_TYPE_VARIANT,
                MetaDataType::INTEGER,
                VarInt(i32::from(self.variant_and_markings.load(Relaxed))),
            )],
            None,
        );
    }
}

impl NBTStorage for HorseEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_animal_nbt(nbt);
            self.write_horse_nbt(nbt);
            nbt.put_int(
                "Variant",
                i32::from(self.variant_and_markings.load(Relaxed)),
            );
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
            if let Some(packed) = nbt.get_int("Variant") {
                self.variant_and_markings.store(packed as u16, Relaxed);
            }
        })
    }
}

impl Animal for HorseEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_HORSE_FOOD)
    }
}

impl AbstractHorse for HorseEntity {
    fn horse_data(&self) -> &AbstractHorseData {
        &self.horse_data
    }

    fn angry_sound(&self) -> Option<Sound> {
        Some(Sound::EntityHorseAngry)
    }

    fn eating_sound(&self) -> Option<Sound> {
        Some(Sound::EntityHorseEat)
    }

    /// `Horse.randomizeAttributes`: rolls max-health, speed AND jump-strength (unlike the
    /// chested horses, which only roll max-health).
    fn randomize_attributes(&self, random: &mut impl RngExt)
    where
        Self: Sized,
    {
        let mut attrs = self.mob_entity.living_entity.attributes.write().unwrap();
        if let Some(a) = attrs.get_mut(&Attributes::MAX_HEALTH.id) {
            a.base_value = crate::entity::passive::equine::generate_max_health(random);
            a.dirty.store(true, Relaxed);
        }
        if let Some(a) = attrs.get_mut(&Attributes::MOVEMENT_SPEED.id) {
            a.base_value = crate::entity::passive::equine::generate_speed(random);
            a.dirty.store(true, Relaxed);
        }
        if let Some(a) = attrs.get_mut(&Attributes::JUMP_STRENGTH.id) {
            a.base_value = crate::entity::passive::equine::generate_jump_strength(random);
            a.dirty.store(true, Relaxed);
        }
        drop(attrs);
        let max_health = self.mob_entity.living_entity.get_max_health();
        self.mob_entity.living_entity.health.store(max_health);
    }
}

impl Mob for HorseEntity {
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

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.sync_type_variant();
        })
    }

    /// `Horse.getBreedOffspring`: Horse+Donkey -> Mule is handled generically by
    /// `HorseBreedGoal`/`horse_family_offspring`; Horse+Horse -> Horse with inherited
    /// variant/markings (`Horse.java:199-217`) is genuinely Horse-specific and implemented here.
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

            let mate_horse = mate.cast_any().downcast_ref::<Self>();
            let baby_horse = baby.cast_any().downcast_ref::<Self>();

            let mut random = rand::rng();
            let mate_max_health = mate.get_mob().map_or(MIN_HEALTH, |m| {
                m.get_mob_entity()
                    .living_entity
                    .get_attribute_base(&Attributes::MAX_HEALTH)
            });

            if let (Some(mate_horse), Some(baby_horse)) = (mate_horse, baby_horse) {
                let select_skin = random.random_range(0..9);
                let variant = if select_skin < 4 {
                    self.variant()
                } else if select_skin < 8 {
                    mate_horse.variant()
                } else {
                    random.random_range(0..VARIANT_COUNT)
                };

                let select_marking = random.random_range(0..5);
                let markings = if select_marking < 2 {
                    self.markings()
                } else if select_marking < 4 {
                    mate_horse.markings()
                } else {
                    random.random_range(0..MARKINGS_COUNT)
                };

                baby_horse.set_variant_and_markings(variant, markings);
            }

            if let Some(baby_mob) = baby.get_mob() {
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

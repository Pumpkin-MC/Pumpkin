use std::sync::{
    Arc, Weak,
    atomic::{AtomicU8, Ordering},
};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{entity::EntityType, item::Item};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        active_target::ActiveTargetGoal, breed::BreedGoal,
        climb_on_top_of_powder_snow::ClimbOnTopOfPowderSnowGoal, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
        rabbit_avoid_entity::RabbitAvoidEntityGoal, rabbit_hop::RabbitHopGoal,
        raid_garden::RaidGardenGoal, revenge::RevengeGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    attributes::{Modifier, ModifierOperation},
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::text::TextComponent;

const TEMPT_ITEMS: &[&Item] = &[&Item::CARROT, &Item::GOLDEN_CARROT, &Item::DANDELION];

/// Vanilla `Rabbit.Variant`. Ids match `Variant.LEGACY_CODEC` (`RabbitType` NBT int).
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RabbitVariant {
    Brown = 0,
    White = 1,
    Black = 2,
    WhiteSplotched = 3,
    Gold = 4,
    Salt = 5,
    Evil = 99,
}

/// Sentinel meaning "no variant rolled yet"; not a valid vanilla id.
const VARIANT_UNSET: u8 = 0xFF;

impl From<u8> for RabbitVariant {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::White,
            2 => Self::Black,
            3 => Self::WhiteSplotched,
            4 => Self::Gold,
            5 => Self::Salt,
            99 => Self::Evil,
            _ => Self::Brown,
        }
    }
}

/// Vanilla `Rabbit.getRandomRabbitVariant`.
fn get_random_rabbit_variant(
    world: &crate::world::World,
    pos: pumpkin_util::math::position::BlockPos,
) -> RabbitVariant {
    let biome = world.get_biome(&pos);
    let roll = rand::random_range(0..100);

    if biome.has_tag(&tag::WorldgenBiome::MINECRAFT_SPAWNS_WHITE_RABBITS) {
        if roll < 80 {
            RabbitVariant::White
        } else {
            RabbitVariant::WhiteSplotched
        }
    } else if biome.has_tag(&tag::WorldgenBiome::MINECRAFT_SPAWNS_GOLD_RABBITS) {
        RabbitVariant::Gold
    } else if roll < 50 {
        RabbitVariant::Brown
    } else if roll < 90 {
        RabbitVariant::Salt
    } else {
        RabbitVariant::Black
    }
}

pub struct RabbitEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    variant: AtomicU8,
}

impl RabbitEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let this = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            variant: AtomicU8::new(VARIANT_UNSET),
        };
        let mob_arc = Arc::new(this);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let rabbit_weak = Arc::downgrade(&mob_arc);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, ClimbOnTopOfPowderSnowGoal::new());
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.2));
            goal_selector.add_goal(2, BreedGoal::new(0.8));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.0, TEMPT_ITEMS, false)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(0.8)));
            goal_selector.add_goal(
                4,
                RabbitAvoidEntityGoal::new(&EntityType::PLAYER, 8.0, 2.2, 2.2, rabbit_weak.clone()),
            );
            goal_selector.add_goal(
                4,
                RabbitAvoidEntityGoal::new(&EntityType::WOLF, 10.0, 2.2, 2.2, rabbit_weak),
            );
            goal_selector.add_goal(5, RaidGardenGoal::new(0.7));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                11,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );
            goal_selector.add_goal(11, Box::new(RandomLookAroundGoal::default()));
            goal_selector.add_goal(1, RabbitHopGoal::new());
        };

        mob_arc
    }

    #[must_use]
    pub fn get_variant(&self) -> RabbitVariant {
        RabbitVariant::from(self.variant.load(Ordering::Relaxed))
    }

    /// Vanilla `Rabbit.setVariant`. Un-setting `EVIL` is not implemented (vanilla itself only
    /// removes the transient attack-damage modifier on un-set and never actually un-sets a
    /// rabbit's variant in practice -- it's set once at spawn/NBT-read and never reverted).
    ///
    /// Deferred: no entity-metadata sync for the variant byte. `TrackedData::RABBIT_TYPE` is
    /// `255` (absent) for the protocol versions this server currently targets -- vanilla itself
    /// moved rabbit variant to a `DataComponents.RABBIT_VARIANT` data component in newer
    /// versions, which Pumpkin's client metadata sync does not yet cover for this mob. State is
    /// still tracked server-side (goals/attributes/NBT/genetics all work); only the client-side
    /// visual variant is unaddressed.
    pub fn set_variant(&self, variant: RabbitVariant) {
        self.variant.store(variant as u8, Ordering::Relaxed);

        if variant == RabbitVariant::Evil {
            self.mob_entity
                .living_entity
                .update_attribute(&Attributes::ARMOR, |inst| {
                    inst.base_value = 8.0;
                });
            self.mob_entity
                .living_entity
                .update_attribute(&Attributes::ATTACK_DAMAGE, |inst| {
                    inst.add_or_replace_modifier(Modifier {
                        id: "evil".to_string(),
                        amount: 5.0,
                        operation: ModifierOperation::Add,
                    });
                });

            self.mob_entity
                .goals_selector
                .lock()
                .unwrap()
                .add_goal(4, Box::new(MeleeAttackGoal::new(1.4, true)));

            let mut target_selector = self.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&self.mob_entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&self.mob_entity, &EntityType::WOLF, true),
            );
            drop(target_selector);

            let entity = &self.mob_entity.living_entity.entity;
            if (**entity.custom_name.load()).is_none() {
                entity.set_custom_name(TextComponent::translate(
                    "entity.minecraft.killer_bunny",
                    [],
                ));
            }
        }
    }
}

impl AgeableMob for RabbitEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for RabbitEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl NBTStorage for RabbitEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_ageable_nbt(nbt);
            self.write_animal_nbt(nbt);
            nbt.put_int("RabbitType", self.variant.load(Ordering::Relaxed) as i32);
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_ageable_nbt(nbt);
            self.read_animal_nbt(nbt);
            if let Some(variant) = nbt.get_int("RabbitType") {
                self.variant.store(variant as u8, Ordering::Relaxed);
            }
        })
    }
}

impl Mob for RabbitEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            if self.variant.load(Ordering::Relaxed) == VARIANT_UNSET {
                let entity = &self.mob_entity.living_entity.entity;
                let world = entity.world.load();
                let pos = entity.block_pos.load();
                let variant = get_random_rabbit_variant(&world, pos);
                self.set_variant(variant);
            } else {
                self.set_variant(self.get_variant());
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        self.animal_interact(player, item_stack, Sound::EntityRabbitAmbient)
    }

    /// Vanilla `Rabbit.getBreedOffspring`: 95% chance (`random.nextInt(20) != 0`) to inherit a
    /// parent's variant (50/50 which parent), else roll a fresh biome-based variant.
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
                uuid::Uuid::new_v4(),
            );

            if let Some(kit) = baby.cast_any().downcast_ref::<Self>() {
                let variant = if rand::random_range(0..20) != 0 {
                    let mate_rabbit = mate.cast_any().downcast_ref::<Self>();
                    if rand::random_bool(0.5) {
                        self.get_variant()
                    } else {
                        mate_rabbit.map_or_else(|| self.get_variant(), Self::get_variant)
                    }
                } else {
                    get_random_rabbit_variant(world, entity.block_pos.load())
                };
                kit.set_variant(variant);
            }

            Some(baby)
        })
    }
}

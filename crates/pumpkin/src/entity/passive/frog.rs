use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};

pub const FROG_FOOD: &[&Item] = &[&Item::SLIME_BALL];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum FrogVariant {
    Cold = 0,
    #[default]
    Temperate = 1,
    Warm = 2,
}

impl FrogVariant {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            0 => Self::Cold,
            2 => Self::Warm,
            _ => Self::Temperate,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cold => "minecraft:cold",
            Self::Temperate => "minecraft:temperate",
            Self::Warm => "minecraft:warm",
        }
    }

    #[must_use]
    pub fn from_name(s: &str) -> Self {
        match s {
            "minecraft:cold" | "cold" => Self::Cold,
            "minecraft:warm" | "warm" => Self::Warm,
            _ => Self::Temperate,
        }
    }
}

/// Represents a Frog, an amphibious mob that can eat small slimes and magma cubes.
///
/// Wiki: <https://minecraft.wiki/w/Frog>
pub struct FrogEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub variant: AtomicI32,
    pub tongue_target_id: AtomicI32,
}

impl FrogEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let frog = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            variant: AtomicI32::new(FrogVariant::Temperate.id()),
            tongue_target_id: AtomicI32::new(-1),
        };
        let mob_arc = Arc::new(frog);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(TemptGoal::new(1.0, FROG_FOOD)));
            goal_selector.add_goal(2, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(4, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn get_variant(&self) -> FrogVariant {
        FrogVariant::from_id(self.variant.load(Ordering::Relaxed))
    }

    pub fn set_variant(&self, variant: FrogVariant) {
        self.variant.store(variant.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::frog::VARIANT,
                VarInt(variant.id()),
            )],
            None,
        );
    }
}

impl AgeableMob for FrogEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for FrogEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_FROG_FOOD)
            || FROG_FOOD.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl Mob for FrogEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("variant", self.get_variant().as_str().to_string());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(variant_str) = nbt.get_string("variant") {
            self.set_variant(FrogVariant::from_name(variant_str));
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_set_variant_name(&self, name: &str) {
        self.set_variant(FrogVariant::from_name(name));
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.ageable_ai_step();
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::frog::BABY_ID,
                    true,
                )],
                None,
            );
        }
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::frog::VARIANT,
                VarInt(self.get_variant().id()),
            )],
            None,
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack, Sound::EntityFrogAmbient)
    }
}

use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    item::ItemEntity,
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};

const TEMPT_ITEMS: &[&Item] = &[&Item::WHEAT];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum MooshroomVariant {
    #[default]
    Red = 0,
    Brown = 1,
}

impl MooshroomVariant {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Brown,
            _ => Self::Red,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Brown => "brown",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "brown" => Self::Brown,
            _ => Self::Red,
        }
    }
}

pub struct MooshroomEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub variant: AtomicI32,
    pub stew_effect: AtomicCell<Option<u32>>,
    pub last_lightning_bolt_uuid: AtomicCell<Option<Uuid>>,
}

impl MooshroomEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mooshroom = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            variant: AtomicI32::new(MooshroomVariant::Red.id()),
            stew_effect: AtomicCell::new(None),
            last_lightning_bolt_uuid: AtomicCell::new(None),
        };
        let mob_arc = Arc::new(mooshroom);
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
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.25)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn get_variant(&self) -> MooshroomVariant {
        MooshroomVariant::from_id(self.variant.load(Ordering::Relaxed))
    }

    pub fn set_variant(&self, variant: MooshroomVariant) {
        self.variant.store(variant.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::mooshroom::DATA_TYPE,
                VarInt(variant.id()),
            )],
            None,
        );
    }
}

impl AgeableMob for MooshroomEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for MooshroomEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_COW_FOOD)
            || TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl Mob for MooshroomEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("Type", self.get_variant().as_str().to_string());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(variant_str) = nbt.get_string("Type") {
            self.set_variant(MooshroomVariant::from_name(variant_str));
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
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
                    pumpkin_data::tracked_data::mooshroom::DATA_BABY_ID,
                    true,
                )],
                None,
            );
        }
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::mooshroom::DATA_TYPE,
                VarInt(self.get_variant().id()),
            )],
            None,
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let item = item_stack.get_item();

        if item == &Item::BOWL && !self.is_baby() {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            let is_suspicious = self.stew_effect.swap(None).is_some();
            let sound = if is_suspicious {
                Sound::EntityMooshroomSuspiciousMilk
            } else {
                Sound::EntityMooshroomMilk
            };
            world.play_sound(sound, SoundCategory::Neutral, &pos);
            return true;
        }

        if item == &Item::BUCKET && !self.is_baby() {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            let entity = self.get_entity();
            let world = entity.world.load();
            world.play_sound(
                Sound::EntityCowMilk,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
            return true;
        }

        if item == &Item::SHEARS && !self.is_baby() {
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.play_sound(Sound::EntityMooshroomShear, SoundCategory::Players, &pos);

            let mushroom_item = if self.get_variant() == MooshroomVariant::Brown {
                &Item::BROWN_MUSHROOM
            } else {
                &Item::RED_MUSHROOM
            };

            for _ in 0..5 {
                let item_entity = Arc::new(ItemEntity::new(
                    Entity::new(world.clone(), pos, &EntityType::ITEM),
                    ItemStack::new(1, mushroom_item),
                ));
                world.spawn_entity(item_entity);
            }

            world.spawn_particle(
                pos + Vector3::new(0.0, 0.5, 0.0),
                Vector3::new(0.5, 0.5, 0.5),
                0.0,
                1,
                Particle::Explosion,
            );

            player.damage_held_item(1);
            return true;
        }

        if self.get_variant() == MooshroomVariant::Brown
            && !self.is_baby()
            && item.has_tag(&tag::Item::MINECRAFT_SMALL_FLOWERS)
        {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            self.stew_effect.store(Some(1));
            world.play_sound(Sound::EntityMooshroomEat, SoundCategory::Neutral, &pos);
            world.spawn_particle(
                pos + Vector3::new(0.0, 0.5, 0.0),
                Vector3::new(0.5, 0.5, 0.5),
                0.0,
                4,
                Particle::Effect,
            );
            return true;
        }

        self.animal_interact(player, item_stack, Sound::EntityCowAmbient)
    }
}

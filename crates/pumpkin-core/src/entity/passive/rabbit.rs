use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use rand::RngExt;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::behavior::fleeing::{self, Flee},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    ai::target_match::TargetMatch,
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};

const TEMPT_ITEMS: &[&Item] = &[&Item::CARROT, &Item::GOLDEN_CARROT, &Item::DANDELION];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum RabbitVariant {
    #[default]
    Brown = 0,
    White = 1,
    Black = 2,
    WhiteSplotched = 3,
    Gold = 4,
    Salt = 5,
    Evil = 99,
}

impl RabbitVariant {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::White,
            2 => Self::Black,
            3 => Self::WhiteSplotched,
            4 => Self::Gold,
            5 => Self::Salt,
            99 => Self::Evil,
            _ => Self::Brown,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub fn random_variant() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..6) {
            1 => Self::White,
            2 => Self::Black,
            3 => Self::WhiteSplotched,
            4 => Self::Gold,
            5 => Self::Salt,
            _ => Self::Brown,
        }
    }
}

const RABBIT_FLEES: &[Flee] = &[
    Flee::new(4, TargetMatch::Type(&EntityType::PLAYER), 8.0, 2.2, 2.2).when(is_not_evil),
    Flee::new(4, TargetMatch::Type(&EntityType::WOLF), 10.0, 2.2, 2.2).when(is_not_evil),
    Flee::new(4, TargetMatch::Monster, 4.0, 2.2, 2.2).when(is_not_evil),
];

fn is_not_evil(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<RabbitEntity>()
        .is_some_and(|rabbit| rabbit.get_variant() != RabbitVariant::Evil)
}

const KILLER_BUNNY_HUNTS: &[Hunt] = &[
    Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true),
    Hunt::new(2, TargetMatch::Type(&EntityType::WOLF), true),
];

pub struct RabbitEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub variant: AtomicI32,
    pub more_carrot_ticks: AtomicI32,
}

impl RabbitEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let variant = RabbitVariant::random_variant();
        let rabbit = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            variant: AtomicI32::new(variant.id()),
            more_carrot_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(rabbit);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.2));
            goal_selector.add_goal(2, BreedGoal::new(0.8));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.0, TEMPT_ITEMS, false)));
            fleeing::apply(goal_selector, RABBIT_FLEES);
            goal_selector.add_goal(5, Box::new(FollowParentGoal::new(0.8)));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                11,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 10.0),
            );
            goal_selector.add_goal(11, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn get_variant(&self) -> RabbitVariant {
        RabbitVariant::from_id(self.variant.load(Ordering::Relaxed))
    }

    pub fn set_variant(&self, variant: RabbitVariant) {
        self.variant.store(variant.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::rabbit::DATA_TYPE_ID,
            VarInt(variant.id()),
        );

        if variant == RabbitVariant::Evil {
            self.add_killer_bunny_goals();
        }
    }

    fn add_killer_bunny_goals(&self) {
        let goal_selector = &self.mob_entity.goals_selector;
        goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.4, true)));

        let target_selector = &self.mob_entity.target_selector;
        target_selector.add_goal(1, Box::new(RevengeGoal::new(true).alerting_others()));
        hunting::apply(target_selector, &self.mob_entity, KILLER_BUNNY_HUNTS);
    }
}

impl AgeableMob for RabbitEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for RabbitEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_RABBIT_FOOD)
            || TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl Mob for RabbitEntity {
    fn mob_sound_source(&self) -> SoundCategory {
        if self.get_variant() == RabbitVariant::Evil {
            SoundCategory::Hostile
        } else {
            SoundCategory::Neutral
        }
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_ageable_nbt(nbt);
        nbt.put_int("RabbitType", self.get_variant().id());
        nbt.put_int(
            "MoreCarrotTicks",
            self.more_carrot_ticks.load(Ordering::Relaxed),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_ageable_nbt(nbt);
        if let Some(rabbit_type) = nbt.get_int("RabbitType") {
            self.set_variant(RabbitVariant::from_id(rabbit_type));
        }
        if let Some(ticks) = nbt.get_int("MoreCarrotTicks") {
            self.more_carrot_ticks.store(ticks, Ordering::Relaxed);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::rabbit::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::rabbit::DATA_TYPE_ID,
            VarInt(self.get_variant().id()),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack, Sound::EntityRabbitAmbient)
    }
}

use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob, speed_up_seconds_when_feeding},
    ai::goal::{
        escape_danger::EscapeDangerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, tempt::TemptGoal, try_find_water::TryFindWaterGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};

const TEMPT_ITEMS: &[&Item] = &[&Item::COD, &Item::SALMON, &Item::TROPICAL_FISH];

pub struct DolphinEntity {
    pub mob_entity: MobEntity,
    pub got_fish: AtomicBool,
    pub moistness_level: AtomicI32,
    pub ageable_data: AgeableData,
}

impl DolphinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let dolphin = Self {
            mob_entity,
            got_fish: AtomicBool::new(false),
            moistness_level: AtomicI32::new(2400),
            ageable_data: AgeableData::default(),
        };
        let mob_arc = Arc::new(dolphin);
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

            goal_selector.add_goal(0, Box::new(TryFindWaterGoal));
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.6));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.2, true)));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.2, TEMPT_ITEMS, false)));
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));
        };

        {
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
        };

        mob_arc
    }

    #[must_use]
    pub fn got_fish(&self) -> bool {
        self.got_fish.load(Ordering::Relaxed)
    }

    pub fn set_got_fish(&self, val: bool) {
        self.got_fish.store(val, Ordering::Relaxed);
    }
}

impl AgeableMob for DolphinEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Mob for DolphinEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn can_attack(&self, target: &dyn EntityBase) -> bool {
        // Babies never fight back.
        !self.is_baby()
            && target.get_entity().entity_type != &EntityType::GHAST
            && self.get_mob_entity().living_entity.can_attack(target)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("GotFish", self.got_fish());
        nbt.put_int("Moistness", self.moistness_level.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(got_fish) = nbt.get_bool("GotFish") {
            self.set_got_fish(got_fish);
        }
        if let Some(moistness) = nbt.get_int("Moistness") {
            self.moistness_level.store(moistness, Ordering::Relaxed);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if !item_stack.get_item().has_tag(&tag::Item::MINECRAFT_FISHES) {
            return false;
        }

        let entity = self.get_entity();
        let world = entity.world.load();
        world.play_sound(
            Sound::EntityDolphinEat,
            SoundCategory::Neutral,
            &entity.pos.load(),
        );

        item_stack.decrement_unless_creative(player.gamemode.load(), 1);
        if self.can_age_up() {
            self.age_up(speed_up_seconds_when_feeding(-self.get_age()), true);
        } else {
            self.set_got_fish(true);
        }
        true
    }
}

use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        escape_danger::EscapeDangerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        try_find_water::TryFindWaterGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};

const TEMPT_ITEMS: &[&Item] = &[&Item::SLIME_BALL];

pub struct TadpoleEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
}

impl TadpoleEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let tadpole = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
        };
        let mob_arc = Arc::new(tadpole);
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
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.5));
            goal_selector.add_goal(2, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS)));
            goal_selector.add_goal(3, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                4,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(5, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    fn is_food(item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_FROG_FOOD)
            || item_stack.item == &Item::SLIME_BALL
    }
}

impl AgeableMob for TadpoleEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Mob for TadpoleEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_ageable_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_ageable_nbt(nbt);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.ageable_ai_step();
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if Self::is_food(item_stack) {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            let age = self
                .get_mob_entity()
                .living_entity
                .entity
                .age
                .load(std::sync::atomic::Ordering::Relaxed);
            let speedup = (-age / 10).max(1);
            self.get_mob_entity()
                .living_entity
                .entity
                .age
                .fetch_add(speedup, std::sync::atomic::Ordering::Relaxed);

            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.spawn_particle(
                pos + Vector3::new(0.0, f64::from(entity.height()), 0.0),
                Vector3::new(0.5, 0.5, 0.5),
                1.0,
                7,
                Particle::HappyVillager,
            );
            world.play_sound(Sound::EntityTadpoleGrowUp, SoundCategory::Neutral, &pos);
            return true;
        }

        if item_stack.get_item() == &Item::WATER_BUCKET {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.play_sound(Sound::ItemBucketFillTadpole, SoundCategory::Neutral, &pos);
            entity.remove();
            return true;
        }

        false
    }
}

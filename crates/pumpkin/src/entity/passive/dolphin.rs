use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
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

const MAX_MOISTNESS: i32 = 2400;

pub struct DolphinEntity {
    pub mob_entity: MobEntity,
    pub got_fish: AtomicBool,
    pub moistness_level: AtomicI32,
}

impl DolphinEntity {
    /// New dolphin with vanilla goals.
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let dolphin = Self {
            mob_entity,
            got_fish: AtomicBool::new(false),
            moistness_level: AtomicI32::new(MAX_MOISTNESS),
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
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.2, TEMPT_ITEMS)));
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

    /// `Dolphin.gotFish`.
    #[must_use]
    pub fn got_fish(&self) -> bool {
        self.got_fish.load(Ordering::Relaxed)
    }

    /// Sets and syncs `GotFish`.
    pub fn set_got_fish(&self, val: bool) {
        self.got_fish.store(val, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(pumpkin_data::tracked_data::dolphin::GOT_FISH, val);
    }

    /// `Dolphin.moistnessLevel`.
    #[must_use]
    pub fn moistness_level(&self) -> i32 {
        self.moistness_level.load(Ordering::Relaxed)
    }

    /// Sets and syncs `Moistness`.
    pub fn set_moistness_level(&self, level: i32) {
        if self.moistness_level.swap(level, Ordering::Relaxed) != level {
            self.get_entity().set_synced_data(
                pumpkin_data::tracked_data::dolphin::MOISTNESS_LEVEL,
                VarInt(level),
            );
        }
    }
}

impl Mob for DolphinEntity {
    /// `Dolphin.tick` moisture.
    fn mob_tick(&self, caller: &dyn EntityBase) {
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        if !entity.is_alive() || self.mob_entity.is_no_ai() {
            return;
        }

        if entity.is_in_water_rain_or_bubble() {
            self.set_moistness_level(MAX_MOISTNESS);
        } else {
            let moistness = self.moistness_level() - 1;
            self.set_moistness_level(moistness);
            if moistness <= 0 {
                living.damage(caller, 1.0, DamageType::DRY_OUT);
            }
        }
    }

    /// Syncs `GotFish` and `Moistness` on spawn.
    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::dolphin::GOT_FISH,
            self.got_fish(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::dolphin::MOISTNESS_LEVEL,
            VarInt(self.moistness_level()),
        );
    }

    /// Writes `GotFish` and `Moistness`.
    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("GotFish", self.got_fish());
        nbt.put_int("Moistness", self.moistness_level());
    }

    /// Reads `GotFish` and `Moistness`.
    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(got_fish) = nbt.get_bool("GotFish") {
            self.set_got_fish(got_fish);
        }
        if let Some(moistness) = nbt.get_int("Moistness") {
            self.set_moistness_level(moistness);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let item = item_stack.get_item();
        if TEMPT_ITEMS.iter().any(|i| i.id == item.id) {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            self.set_got_fish(true);
            self.mob_entity.living_entity.heal(2.0);

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
            world.play_sound(Sound::EntityDolphinEat, SoundCategory::Neutral, &pos);
            return true;
        }

        false
    }
}

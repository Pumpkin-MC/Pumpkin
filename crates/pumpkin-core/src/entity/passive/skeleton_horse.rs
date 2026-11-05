use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::ai::goal::skeleton_trap::SkeletonTrapGoal;
use crate::entity::{
    Entity, EntityBase,
    ageable::AgeableData,
    mob::{Mob, MobEntity},
    passive::abstract_horse::{
        AbstractHorse, AbstractHorseData, JUMP_SOUND_VOLUME, RiddenStep, generate_jump_strength,
        play_horse_step_sound, register_horse_goals,
    },
    passive::animal::Animal,
    player::Player,
};

const TRAP_MAX_LIFE: i32 = 18000;

pub struct SkeletonHorseEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    horse_data: AbstractHorseData,
    is_trap: AtomicBool,
    trap_time: AtomicI32,
}

impl SkeletonHorseEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let horse = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            horse_data: AbstractHorseData::default(),
            is_trap: AtomicBool::new(false),
            trap_time: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(horse);
        let mob: Arc<dyn Mob> = mob_arc.clone();
        register_horse_goals(&mob);

        mob_arc
    }

    #[must_use]
    pub fn is_trap(&self) -> bool {
        self.is_trap.load(Ordering::Relaxed)
    }

    pub fn set_trap(&self, trap: bool) {
        if self.is_trap.swap(trap, Ordering::Relaxed) == trap {
            return;
        }
        if trap {
            self.mob_entity.add_goal(1, SkeletonTrapGoal);
        } else {
            self.mob_entity
                .goals_selector
                .remove_goal::<SkeletonTrapGoal>(self);
        }
    }
}

crate::impl_abstract_horse!(
    SkeletonHorseEntity,
    {
        fn ambient_sound(&self) -> Option<Sound> {
            if self.get_entity().is_submerged_in_water() {
                Some(Sound::EntitySkeletonHorseAmbientWater)
            } else {
                self.get_entity().entity_type.ambient_sound
            }
        }

        fn play_jump_sound(&self) {
            if self.get_entity().is_in_water() {
                self.play_entity_sound(Sound::EntitySkeletonHorseJumpWater, JUMP_SOUND_VOLUME, 1.0);
            } else {
                self.play_entity_sound(Sound::EntityHorseJump, JUMP_SOUND_VOLUME, 1.0);
            }
        }

        fn randomize_attributes(&self) {
            let mut rng = self.get_random();
            self.mob_entity.living_entity.set_attribute_base(
                &Attributes::JUMP_STRENGTH,
                generate_jump_strength(|| rng.random()),
            );
        }

        fn subclass_ai_step(&self) {
            if !self.mob_entity.persistence_required.load(Ordering::Relaxed)
                && self.is_trap()
                && self.trap_time.fetch_add(1, Ordering::Relaxed) >= TRAP_MAX_LIFE
            {
                self.get_entity().remove();
            }
        }
    },
    {
        fn can_age_up(&self) -> bool {
            false
        }
    }
);

impl Animal for SkeletonHorseEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_HORSE_FOOD)
    }

    fn can_mate(&self, _partner: &dyn EntityBase) -> bool {
        false
    }
}

impl Mob for SkeletonHorseEntity {
    crate::abstract_horse_mob_hooks!();

    fn mob_play_step_sound(&self, pos: &BlockPos, state: &BlockState) {
        play_horse_step_sound(self, pos, state);
    }

    fn mob_get_swim_sound(&self) -> Option<Sound> {
        let entity = self.get_entity();
        if !entity.on_ground.load(Ordering::Relaxed) {
            return Some(Sound::EntitySkeletonHorseSwim);
        }
        if !entity.has_passengers() {
            return Some(Sound::EntitySkeletonHorseStepWater);
        }
        Some(match self.next_ridden_step() {
            RiddenStep::Walk => Sound::EntitySkeletonHorseStepWater,
            RiddenStep::Gallop => Sound::EntitySkeletonHorseGallopWater,
            RiddenStep::Quiet => Sound::EntitySkeletonHorseSwim,
        })
    }

    fn mob_play_swim_sound(&self, volume: f32) {
        if self.get_entity().on_ground.load(Ordering::Relaxed) {
            self.play_default_swim_sound(0.3);
        } else {
            self.play_default_swim_sound((volume * 25.0).min(0.1));
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_horse_nbt(nbt);
        nbt.put_bool("SkeletonTrap", self.is_trap());
        nbt.put_int("SkeletonTrapTime", self.trap_time.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_horse_nbt(nbt);
        self.set_trap(nbt.get_bool("SkeletonTrap").unwrap_or(false));
        self.trap_time.store(
            nbt.get_int("SkeletonTrapTime").unwrap_or(0),
            Ordering::Relaxed,
        );
    }

    fn mob_init_data_tracker(&self) {
        self.init_horse_data_tracker();
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if let Some(result) = self.handle_important_interactions(player, item_stack) {
            return result;
        }
        if !self.is_tamed() {
            return false;
        }
        self.horse_mob_interact(player, item_stack)
    }
}

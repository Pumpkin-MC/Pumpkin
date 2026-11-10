use std::sync::{Arc, atomic::Ordering};

use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase,
    ageable::AgeableData,
    ai::goal::{swim::SwimGoal, tempt::TemptGoal},
    mob::{
        Mob, MobEntity,
        spawn::{EntitySpawnReason, SpawnGroupData, finalize_spawn},
    },
    passive::abstract_horse::{
        AbstractHorse, AbstractHorseData, play_horse_step_sound, register_horse_goals,
    },
    passive::animal::Animal,
    player::Player,
    r#type::from_type,
};
use crate::world::World;

const ZOMBIE_HORSE_JUMP_STEP: f64 = 0.066_666_666_666_666_67;

pub struct ZombieHorseEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    horse_data: AbstractHorseData,
}

impl ZombieHorseEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let horse = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            horse_data: AbstractHorseData::default(),
        };
        let mob_arc = Arc::new(horse);
        let mob: Arc<dyn Mob> = mob_arc.clone();
        register_horse_goals(&mob);
        let goals = &mob_arc.mob_entity.goals_selector;
        goals.add_goal(0, Box::new(SwimGoal::default()));
        goals.add_goal(
            3,
            Box::new(TemptGoal::with_tag(
                1.25,
                &tag::Item::MINECRAFT_ZOMBIE_HORSE_FOOD,
                false,
            )),
        );

        mob_arc
    }
}

fn generate_zombie_horse_jump_strength(mut probability: impl FnMut() -> f64) -> f64 {
    0.5 + probability() * ZOMBIE_HORSE_JUMP_STEP
        + probability() * ZOMBIE_HORSE_JUMP_STEP
        + probability() * ZOMBIE_HORSE_JUMP_STEP
}

fn generate_zombie_horse_speed(mut probability: impl FnMut() -> f64) -> f64 {
    (9.0 + probability() + probability() + probability()) / f64::from(42.16f32)
}

crate::impl_abstract_horse!(
    ZombieHorseEntity,
    {
        fn is_mob_controlled(&self) -> bool {
            self.mob_entity
                .first_passenger()
                .is_some_and(|passenger| passenger.get_mob().is_some())
        }

        fn finalize_horse_spawn(
            &self,
            world: &Arc<World>,
            reason: EntitySpawnReason,
            group_data: Option<SpawnGroupData>,
        ) -> Option<SpawnGroupData> {
            if reason == EntitySpawnReason::Natural {
                let entity = self.get_entity();
                let zombie = from_type(
                    &EntityType::ZOMBIE,
                    entity.pos.load(),
                    world,
                    Uuid::new_v4(),
                );
                zombie.get_entity().set_rotation(entity.yaw.load(), 0.0);
                finalize_spawn(&zombie, world, reason, None);
                if let Some(mob) = zombie.get_mob() {
                    mob.get_mob_entity().set_item_slot(
                        &EquipmentSlot::MAIN_HAND,
                        ItemStack::new(1, &Item::IRON_SPEAR),
                    );
                }
                self.mob_entity.add_pending_rider(zombie);
            }
            self.randomize_attributes();
            self.finalize_spawn_super(group_data)
        }

        fn randomize_attributes(&self) {
            let mut rng = self.get_random();
            let living = &self.mob_entity.living_entity;
            living.set_attribute_base(
                &Attributes::JUMP_STRENGTH,
                generate_zombie_horse_jump_strength(|| rng.random()),
            );
            living.set_attribute_base(
                &Attributes::MOVEMENT_SPEED,
                generate_zombie_horse_speed(|| rng.random()),
            );
        }
    },
    {
        fn can_age_up(&self) -> bool {
            false
        }
    }
);

impl Animal for ZombieHorseEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_ZOMBIE_HORSE_FOOD)
    }

    fn can_mate(&self, _partner: &dyn EntityBase) -> bool {
        false
    }
}

impl Mob for ZombieHorseEntity {
    crate::abstract_horse_mob_hooks!();

    fn mob_play_step_sound(&self, pos: &BlockPos, state: &BlockState) {
        play_horse_step_sound(self, pos, state);
    }

    fn sun_protection_slot(&self) -> EquipmentSlot {
        EquipmentSlot::BODY
    }

    fn can_be_leashed(&self) -> bool {
        self.is_tamed() || !self.is_mob_controlled()
    }

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        true
    }

    fn mob_interact_with_hand(
        &self,
        _player: &Arc<Player>,
        _item_stack: &mut ItemStack,
        _hand: pumpkin_util::Hand,
    ) -> Option<bool> {
        self.mob_entity
            .persistence_required
            .store(true, Ordering::Relaxed);
        None
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_horse_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_horse_nbt(nbt);
    }

    fn mob_init_data_tracker(&self) {
        self.init_horse_data_tracker();
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if let Some(result) = self.handle_important_interactions(player, item_stack) {
            return result;
        }
        if let Some(result) = self.interact_with_held_item(player, item_stack, false) {
            return result;
        }
        self.horse_mob_interact(player, item_stack)
    }
}

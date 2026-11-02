use std::sync::{Arc, Weak};

use pumpkin_data::BlockState;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        escape_danger::EscapeDangerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity, equipment::equip_on_target},
    passive::abstract_horse::{AbstractHorse, AbstractHorseData, play_horse_step_sound},
    passive::animal::Animal,
    player::Player,
};

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
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.2));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(0.7)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

crate::impl_abstract_horse!(ZombieHorseEntity);

impl Animal for ZombieHorseEntity {
    fn is_food(&self, _item_stack: &ItemStack) -> bool {
        false
    }
}

impl Mob for ZombieHorseEntity {
    crate::abstract_horse_mob_hooks!();

    fn mob_play_step_sound(&self, pos: &BlockPos, state: &BlockState) {
        play_horse_step_sound(self, pos, state, true);
    }

    fn sun_protection_slot(&self) -> EquipmentSlot {
        EquipmentSlot::BODY
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
        // Vanilla `ZombieHorse.mobInteract`: an untamed one gets mad instead of taking the item.
        if !self.is_tamed()
            && !self.get_entity().has_passengers()
            && !item_stack.is_empty()
            && !self.is_food(item_stack)
        {
            let pitch = self.mob_entity.living_entity.get_pitch();
            self.play_entity_sound(Sound::EntityZombieHorseAngry, 0.8, pitch);
            return true;
        }

        if equip_on_target(self, player, item_stack) {
            return true;
        }

        if !self.is_baby() {
            let world = player.world();
            let ent = &self.mob_entity.living_entity.entity;
            if let Some(vehicle) = world.get_entity_by_id(ent.entity_id)
                && let Some(passenger) = world.get_player_by_id(player.entity_id())
            {
                ent.add_passenger(vehicle, passenger as Arc<dyn EntityBase>);
                return true;
            }
        }

        self.animal_interact(player, item_stack, Sound::EntityZombieHorseAmbient)
    }
}

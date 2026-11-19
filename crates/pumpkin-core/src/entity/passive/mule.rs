use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ageable::AgeableData,
    mob::{Mob, MobEntity},
    passive::abstract_horse::{
        AbstractHorse, AbstractHorseData, BEDROCK_CHESTED_INVENTORY_SIZE,
        BEDROCK_SADDLE_INVENTORY_SIZE, CHESTED_INVENTORY_COLUMNS, JUMP_SOUND_VOLUME,
        add_horse_behaviour_goals, generate_max_health, play_horse_step_sound,
        register_horse_goals,
    },
    passive::animal::Animal,
    player::Player,
};

pub struct MuleEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    horse_data: AbstractHorseData,
}

impl MuleEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mule = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            horse_data: AbstractHorseData::default(),
        };
        let mob_arc = Arc::new(mule);
        let mob: Arc<dyn Mob> = mob_arc.clone();
        register_horse_goals(&mob);
        add_horse_behaviour_goals(mob.as_ref());

        mob_arc
    }
}

crate::impl_abstract_horse!(MuleEntity, {
    fn play_jump_sound(&self) {
        self.play_entity_sound(Sound::EntityMuleJump, JUMP_SOUND_VOLUME, 1.0);
    }

    fn randomize_attributes(&self) {
        let mut rng = self.get_random();
        self.mob_entity
            .living_entity
            .set_max_health(generate_max_health(|bound| rng.random_range(0..bound)));
    }

    fn get_chest_equip_sound(&self) -> Sound {
        Sound::EntityMuleChest
    }

    fn can_carry_chest(&self) -> bool {
        true
    }

    fn bedrock_inventory_size(&self) -> i32 {
        if self.has_chest() {
            BEDROCK_CHESTED_INVENTORY_SIZE
        } else {
            BEDROCK_SADDLE_INVENTORY_SIZE
        }
    }

    fn get_inventory_columns(&self) -> i32 {
        if self.has_chest() {
            CHESTED_INVENTORY_COLUMNS
        } else {
            0
        }
    }
});

impl Animal for MuleEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_HORSE_FOOD)
    }

    fn can_mate(&self, _partner: &dyn EntityBase) -> bool {
        false
    }
}

impl Mob for MuleEntity {
    crate::abstract_horse_mob_hooks!();

    fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        crate::entity::passive::abstract_horse::can_use_slot(self, slot)
    }

    fn mob_play_step_sound(&self, pos: &BlockPos, state: &BlockState) {
        play_horse_step_sound(self, pos, state);
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_horse_nbt(nbt);
        self.write_chested_horse_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_horse_nbt(nbt);
        self.read_chested_horse_nbt(nbt);
    }

    fn mob_init_data_tracker(&self) {
        self.init_horse_data_tracker();
        self.init_chested_horse_data_tracker();
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if let Some(result) = self.handle_important_interactions(player, item_stack) {
            return result;
        }
        if let Some(result) = self.interact_with_held_item(player, item_stack, true) {
            return result;
        }
        self.horse_mob_interact(player, item_stack)
    }
}

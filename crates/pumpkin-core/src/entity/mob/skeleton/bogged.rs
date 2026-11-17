use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_data::BlockState;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
    custom_sound::CustomSound,
    item::ItemEntity,
    mob::{Mob, MobEntity, equipment::RegionalDifficulty, skeleton::SkeletonEntityBase},
    player::Player,
};
use crate::world::World;

const BOGGED_SHEAR_LOOT_TABLE: &str = "minecraft:shearing/bogged";

pub struct BoggedSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
    sheared: AtomicBool,
}

impl BoggedSkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity);
        Arc::new(Self {
            entity,
            sheared: AtomicBool::new(false),
        })
    }

    #[must_use]
    pub fn is_sheared(&self) -> bool {
        self.sheared.load(Ordering::Relaxed)
    }

    pub fn set_sheared(&self, sheared: bool) {
        self.sheared.store(sheared, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::bogged::DATA_SHEARED, sheared);
    }

    #[must_use]
    pub fn ready_for_shearing(&self) -> bool {
        !self.is_sheared()
    }

    pub fn shear(&self, category: SoundCategory, tool: &ItemStack) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        world.play_sound(Sound::EntityBoggedShear, category, &pos);

        if let Some(table) = world.get_loot_table(BOGGED_SHEAR_LOOT_TABLE) {
            let params = crate::world::loot::LootContextParameters {
                world: Some(&world),
                position: Some(pos),
                this_entity: Some(self as &dyn EntityBase),
                tool: Some(tool.clone()),
                ..Default::default()
            };
            let drop_pos = Vector3::new(pos.x, pos.y + f64::from(entity.height()), pos.z);
            for drop_stack in
                crate::world::loot::generate_loot_from_handle(&table, rand::random(), &params)
            {
                let item_entity = Arc::new(ItemEntity::new(
                    Entity::new(world.clone(), drop_pos, &EntityType::ITEM),
                    drop_stack,
                ));
                world.spawn_entity(item_entity);
            }
        }

        self.set_sheared(true);
    }
}

impl CustomSound for BoggedSkeletonEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBoggedHurt)
    }

    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBoggedDeath)
    }
}

impl Mob for BoggedSkeletonEntity {
    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        super::wants_to_pick_up(self, stack)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityBoggedStep, 0.15, 1.0);
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("sheared", self.is_sheared());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_sheared(nbt.get_bool("sheared").unwrap_or(false));
    }

    fn mob_init_data_tracker(&self) {
        self.get_entity()
            .set_synced_data(tracked_data::bogged::DATA_SHEARED, self.is_sheared());
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if item_stack.item.id == Item::SHEARS.id && self.ready_for_shearing() {
            let entity = self.get_entity();
            let world = entity.world.load();
            if let Some(server) = world.server.upgrade() {
                let mut event =
                    crate::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent {
                        player: player.clone(),
                        entity_id: entity.entity_id,
                        hand: 0,
                        cancelled: false,
                    };
                server.plugin_manager.fire_blocking(&server, &mut event);
                if event.cancelled {
                    return false;
                }
            }
            self.shear(SoundCategory::Players, item_stack);
            world.emit_game_event(GameEvent::Shear.name(), entity.pos.load());
            let _ = player.damage_item_stack(item_stack, 1);
            return true;
        }
        false
    }

    fn populate_default_equipment_slots(
        &self,
        world: &Arc<World>,
        difficulty: &RegionalDifficulty,
    ) {
        self.entity
            .populate_default_equipment_slots(world, difficulty);
    }

    fn populate_default_equipment_enchantments(&self, difficulty: &RegionalDifficulty) {
        self.entity
            .populate_default_equipment_enchantments(difficulty);
    }
}

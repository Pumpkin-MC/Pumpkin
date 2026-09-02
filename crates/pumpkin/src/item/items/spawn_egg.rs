use std::sync::Arc;

use crate::block::entities::mob_spawner::MobSpawnerBlockEntity;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::r#type::from_type;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::data_component_impl::{
    AxolotlVariantImpl, CatVariantImpl, ChickenVariantImpl, CowVariantImpl, FoxVariantImpl,
    FrogVariantImpl, HorseVariantImpl, LlamaVariantImpl, MooshroomVariantImpl, PigVariantImpl,
    RabbitVariantImpl, SheepColorImpl, ShulkerColorImpl, VillagerVariantImpl, WolfVariantImpl,
};
use pumpkin_data::entity::entity_from_egg;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use uuid::Uuid;

pub struct SpawnEggItem;

impl ItemMetadata for SpawnEggItem {
    fn ids() -> Box<[u16]> {
        pumpkin_data::entity::spawn_egg_ids()
    }
}

pub(crate) fn apply_entity_variant(item: &ItemStack, mob: &dyn EntityBase) {
    if let Some(comp) = item.get_data_component::<ChickenVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<FrogVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<WolfVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<CatVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<VillagerVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<FoxVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<MooshroomVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<RabbitVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<PigVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<CowVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<HorseVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<LlamaVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<AxolotlVariantImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<SheepColorImpl>() {
        mob.set_variant_name(&comp.value);
    } else if let Some(comp) = item.get_data_component::<ShulkerColorImpl>() {
        mob.set_variant_name(&comp.value);
    }
}

impl ItemBehaviour for SpawnEggItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        if let Some(entity_type) = entity_from_egg(item.id) {
            let world = player.world();
            let (start_pos, end_pos) = self.get_start_and_end_pos(player);
            let checker = |pos: &BlockPos, world_inner: &Arc<World>| {
                let state_id = world_inner.get_block_state_id(pos);
                if state_id == Block::AIR.default_state.id {
                    return false;
                }
                Fluid::from_state_id(state_id).is_some()
            };

            let Some((hit_pos, _)) = world.raycast(start_pos, end_pos, checker) else {
                return;
            };

            let pos = Vector3::new(
                f64::from(hit_pos.0.x) + 0.5,
                f64::from(hit_pos.0.y),
                f64::from(hit_pos.0.z) + 0.5,
            );
            let yaw = wrap_degrees(rand::random::<f32>() * 360.0) % 360.0;
            let mob = from_type(entity_type, pos, &world, Uuid::new_v4());
            mob.get_entity().set_rotation(yaw, 0.0);

            let held = player.inventory.held_item();
            let stack = if !held.is_empty() && held.item.id == item.id {
                held
            } else {
                player.inventory.off_hand_item()
            };
            apply_entity_variant(&stack, mob.as_ref());
            world.spawn_entity(mob);

            let mut main_hand = player.inventory.held_item();
            let consumed = if !main_hand.is_empty() && main_hand.item.id == item.id {
                main_hand.decrement_unless_creative(player.gamemode.load(), 1);
                player.inventory.set_held_item(main_hand);
                true
            } else {
                false
            };

            if !consumed {
                let mut off_hand = player.inventory.off_hand_item();
                if !off_hand.is_empty() && off_hand.item.id == item.id {
                    off_hand.decrement_unless_creative(player.gamemode.load(), 1);
                    player
                        .inventory
                        .set_stack_in_hand(pumpkin_util::Hand::Left, off_hand);
                }
            }
        }
    }

    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) {
        if let Some(entity_type) = entity_from_egg(item.item.id) {
            let world = player.world();

            if let Some(block_entity) = player.world().get_block_entity(&location)
                && let Some(spawner) = block_entity
                    .as_any()
                    .downcast_ref::<MobSpawnerBlockEntity>()
            {
                spawner.set_entity_type(entity_type);
                world.update_block_entity(&block_entity);
                item.decrement_unless_creative(player.gamemode.load(), 1);
                return;
            }

            let target_state = world.get_block_state(&location);
            let target_block = world.get_block(&location);
            let spawn_block_pos = if target_state.is_air()
                || target_block.id == Block::WATER.id
                || target_block.id == Block::LAVA.id
            {
                location
            } else {
                BlockPos(location.0 + face.to_offset())
            };
            let pos = Vector3::new(
                f64::from(spawn_block_pos.0.x) + 0.5,
                f64::from(spawn_block_pos.0.y),
                f64::from(spawn_block_pos.0.z) + 0.5,
            );
            let yaw = wrap_degrees(rand::random::<f32>() * 360.0) % 360.0;

            let mob = from_type(entity_type, pos, &world, Uuid::new_v4());

            mob.get_entity().set_rotation(yaw, 0.0);

            apply_entity_variant(item, mob.as_ref());

            world.spawn_entity(mob);
            item.decrement_unless_creative(player.gamemode.load(), 1);
        }
    }

    fn use_on_entity(&self, item: &mut ItemStack, player: &Player, entity: Arc<dyn EntityBase>) {
        if let Some(entity_type) = entity_from_egg(item.item.id)
            && entity.get_entity().entity_type.id == entity_type.id
        {
            let world = player.world();
            let pos = entity.get_entity().pos.load();
            let mob = from_type(entity_type, pos, &world, Uuid::new_v4());
            mob.get_entity()
                .set_rotation(rand::random::<f32>() * 360.0, 0.0);
            mob.get_entity()
                .age
                .store(-24000, std::sync::atomic::Ordering::Relaxed);
            mob.get_entity().send_meta_data(
                &[pumpkin_protocol::java::client::play::Metadata::new(
                    pumpkin_data::tracked_data::ageable_mob::DATA_BABY_ID,
                    true,
                )],
                None,
            );
            apply_entity_variant(item, mob.as_ref());
            world.spawn_entity(mob);
            item.decrement_unless_creative(player.gamemode.load(), 1);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
